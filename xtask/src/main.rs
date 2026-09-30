//! `cargo xtask`: the WENGE gate runner. Owned by the Referee.
//!
//! ```text
//! cargo xtask verify [--release] [--base <git-ref>]   run every gate, write the report
//! cargo xtask gate <name> [--release] [--base <ref>]  run one gate (exit 42: not implemented)
//!                                                     `gate scope` reads files only and takes a second
//! cargo xtask baseline [--release] [--remove <test|fixture> <id> --adr ADR-NNNN]...
//! ```
//!
//! See harness/README.md for what each gate means and how the ratchet works.

mod baseline;
mod cargo_out;
mod gates;
mod json;
mod panel;
mod report;
mod scope;

use std::collections::BTreeSet;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use baseline::Baseline;
use gates::{Gate, Inputs, Status, TestRun};

const EXIT_NOT_IMPLEMENTED: u8 = 42;

struct Opts {
    release: bool,
    base: Option<String>,
    removals: Vec<(String, String, String)>,
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some((cmd, rest)) = args.split_first() else {
        print_help();
        return ExitCode::from(2);
    };
    let result = match cmd.as_str() {
        "verify" => parse_opts(rest).and_then(|o| verify(&o)),
        "gate" => match rest.split_first() {
            Some((name, more)) => parse_opts(more).and_then(|o| gate(name, &o)),
            None => Err("usage: cargo xtask gate <name>".to_string()),
        },
        "baseline" => parse_opts(rest).and_then(|o| update_baseline(&o)),
        "help" | "--help" | "-h" => {
            print_help();
            Ok(ExitCode::SUCCESS)
        }
        other => Err(format!("unknown command '{other}'. Try `cargo xtask help`")),
    };
    match result {
        Ok(code) => code,
        Err(e) => {
            eprintln!("xtask: {e}");
            ExitCode::from(2)
        }
    }
}

fn print_help() {
    eprintln!(
        "cargo xtask verify [--release] [--base <git-ref>]\n\
         cargo xtask gate <name> [--release] [--base <git-ref>]\n\
         cargo xtask baseline [--release] [--remove <test|fixture> <id> --adr ADR-NNNN]...\n\
         \n\
         Exit codes: 0 ok, 1 a gate failed or a required gate is not `pass`, 2 usage or\n\
         internal error, 42 (gate only) the gate is not implemented."
    );
}

fn parse_opts(args: &[String]) -> Result<Opts, String> {
    let mut o = Opts { release: false, base: None, removals: Vec::new() };
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--release" => o.release = true,
            "--base" => {
                i += 1;
                o.base = Some(args.get(i).ok_or("--base needs a git ref")?.clone());
            }
            "--remove" => {
                let kind = args.get(i + 1).ok_or("--remove needs <test|fixture> <id> --adr ADR-NNNN")?;
                let id = args.get(i + 2).ok_or("--remove needs <test|fixture> <id> --adr ADR-NNNN")?;
                if args.get(i + 3).map(String::as_str) != Some("--adr") {
                    return Err("--remove must be followed by --adr ADR-NNNN".to_string());
                }
                let adr = args.get(i + 4).ok_or("--adr needs an id like ADR-0007")?;
                if !matches!(kind.as_str(), "test" | "fixture") {
                    return Err(format!("--remove kind must be test or fixture, not '{kind}'"));
                }
                o.removals.push((kind.clone(), id.clone(), adr.clone()));
                i += 4;
            }
            other => return Err(format!("unknown option '{other}'")),
        }
        i += 1;
    }
    Ok(o)
}

/// The repository the command is run in. The working directory decides, so a git
/// worktree, or a shared `CARGO_TARGET_DIR`, verifies the tree it is standing in and not
/// the one this binary was compiled from. Falls back to the compile-time location only
/// when the working directory is not inside a git checkout.
fn root() -> PathBuf {
    let from_cwd = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .stderr(Stdio::null())
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| PathBuf::from(s.trim()))
        .filter(|p| p.join("harness/required-gates.txt").is_file());
    from_cwd.unwrap_or_else(|| {
        Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("xtask lives one level below the root").to_path_buf()
    })
}

// ---------------------------------------------------------------- environment

fn git(root: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).current_dir(root).stderr(Stdio::null()).output().ok()?;
    out.status.success().then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn commit_hash(root: &Path) -> String {
    match git(root, &["rev-parse", "HEAD"]) {
        Some(h) if (7..=40).contains(&h.len()) && h.chars().all(|c| c.is_ascii_hexdigit()) => h,
        _ => "0000000".to_string(),
    }
}

fn adr_exists(root: &Path, adr: &str) -> bool {
    let Some(num) = adr.strip_prefix("ADR-") else { return false };
    if num.len() != 4 || !num.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    let prefix = format!("{num}-");
    std::fs::read_dir(root.join("adr"))
        .map(|rd| rd.flatten().any(|e| e.file_name().to_string_lossy().starts_with(&prefix)))
        .unwrap_or(false)
}

fn conformance_fixtures(root: &Path) -> BTreeSet<String> {
    fn walk(dir: &Path, root: &Path, out: &mut BTreeSet<String>) {
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        for entry in rd.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.file_name().is_some_and(|n| n == "pending") {
                    continue;
                }
                walk(&path, root, out);
            } else if path.extension().is_some_and(|e| e == "fixture" || e == "panel") {
                if let Ok(rel) = path.strip_prefix(root) {
                    out.insert(rel.to_string_lossy().replace('\\', "/"));
                }
            }
        }
    }
    let mut out = BTreeSet::new();
    walk(&root.join("tests/conformance"), root, &mut out);
    out
}

fn load_baseline(root: &Path) -> Result<Baseline, String> {
    let path = root.join("harness/baseline.txt");
    match std::fs::read_to_string(&path) {
        Ok(text) => Baseline::parse(&text).map_err(|e| format!("harness/baseline.txt: {e}")),
        Err(_) => Ok(Baseline::default()),
    }
}

fn load_required(root: &Path) -> Result<BTreeSet<String>, String> {
    let path = root.join("harness/required-gates.txt");
    let text = std::fs::read_to_string(&path).map_err(|e| format!("harness/required-gates.txt: {e}"))?;
    let mut out = BTreeSet::new();
    for (n, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if !gates::is_known(line) {
            return Err(format!("harness/required-gates.txt line {}: unknown gate '{line}'", n + 1));
        }
        out.insert(line.to_string());
    }
    Ok(out)
}

/// The baseline as of `--base`. Errors on an unknown ref (a typo must not silently turn
/// the ratchet diff off). `Ok(None)` when the ref has no baseline yet.
fn load_base_baseline(root: &Path, base: &str) -> Result<Option<Baseline>, String> {
    if git(root, &["rev-parse", "--verify", "--quiet", &format!("{base}^{{commit}}")]).is_none() {
        return Err(format!("--base '{base}' is not a known git ref. In CI, fetch with fetch-depth: 0"));
    }
    match git(root, &["show", &format!("{base}:harness/baseline.txt")]) {
        Some(text) => Baseline::parse(&text).map(Some).map_err(|e| format!("baseline at {base}: {e}")),
        None => {
            println!("note: {base} has no harness/baseline.txt, so the ratchet diff is skipped");
            Ok(None)
        }
    }
}

// -------------------------------------------------------------------- running

fn run_tests(root: &Path, release: bool) -> Result<TestRun, String> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let mut args = vec!["test", "--workspace", "--no-fail-fast"];
    if release {
        args.push("--release");
    }
    println!("running: cargo {}", args.join(" "));

    let start = Instant::now();
    let (mut reader, writer) = std::io::pipe().map_err(|e| format!("pipe: {e}"))?;
    let mut cmd = Command::new(cargo);
    cmd.args(&args)
        .current_dir(root)
        .stdin(Stdio::null())
        .stdout(writer.try_clone().map_err(|e| e.to_string())?)
        .stderr(writer);
    let mut child = cmd.spawn().map_err(|e| format!("could not start cargo: {e}"))?;
    drop(cmd); // release our copies of the pipe writer so the read below sees EOF
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes).map_err(|e| format!("reading cargo output: {e}"))?;
    let status = child.wait().map_err(|e| e.to_string())?;
    let duration_s = start.elapsed().as_secs_f64();

    let output = String::from_utf8_lossy(&bytes).into_owned();
    let log_dir = root.join("harness/report");
    let _ = std::fs::create_dir_all(&log_dir);
    let _ = std::fs::write(log_dir.join("cargo-test.log"), &output);

    let parsed = cargo_out::parse(&output);
    let build_error = if !status.success() && parsed.failed().is_empty() {
        let tail: Vec<&str> = output.lines().rev().take(15).collect::<Vec<_>>().into_iter().rev().collect();
        Some(format!("cargo test exited with {status} and no failing test was reported. Last lines:\n{}", tail.join("\n")))
    } else {
        None
    };
    Ok(TestRun { parsed, duration_s, build_error })
}

struct Outcome {
    gates: Vec<Gate>,
    assertions: gates::Assertions,
    duration_s: f64,
}

fn run_all(root: &Path, o: &Opts) -> Result<(Outcome, Baseline), String> {
    let baseline = load_baseline(root)?;
    let base = match &o.base {
        Some(r) => load_base_baseline(root, r)?,
        None => None,
    };
    let fixtures = conformance_fixtures(root);
    let panel_scan = panel::scan(root);
    let start = Instant::now();
    let run = run_tests(root, o.release)?;
    let adr = |id: &str| adr_exists(root, id);
    let scope_scan = scope::scan_repo(root);
    let inputs = Inputs {
        run: &run,
        fixtures: &fixtures,
        panel_scan: &panel_scan,
        baseline: &baseline,
        base: base.as_ref(),
        adr_exists: &adr,
        scope: &scope_scan,
    };
    let (gates, assertions) = gates::evaluate(&inputs);
    Ok((Outcome { gates, assertions, duration_s: start.elapsed().as_secs_f64() }, baseline))
}

// ------------------------------------------------------------------- commands

fn verify(o: &Opts) -> Result<ExitCode, String> {
    let root = root();
    let required = load_required(&root)?;
    let (outcome, _) = run_all(&root, o)?;

    let rep = report::Report {
        commit: commit_hash(&root),
        branch: git(&root, &["rev-parse", "--abbrev-ref", "HEAD"]),
        role: std::env::var("WENGE_ROLE").ok().filter(|r| !r.is_empty()),
        ran_at: report::rfc3339_utc(SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)),
        duration_s: outcome.duration_s,
        hardware: format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
        gates: outcome.gates.clone(),
        assertions: outcome.assertions.clone(),
    };
    let out_path = root.join("harness/report/latest.json");
    std::fs::create_dir_all(out_path.parent().unwrap()).map_err(|e| e.to_string())?;
    std::fs::write(&out_path, report::to_json(&rep).pretty()).map_err(|e| format!("writing report: {e}"))?;

    print_summary(&outcome, &required);
    println!("report: harness/report/latest.json   log: harness/report/cargo-test.log");

    let ok = gates::run_succeeds(&outcome.gates, &required);
    println!("result: {}", if ok { "PASS" } else { "FAIL" });
    Ok(if ok { ExitCode::SUCCESS } else { ExitCode::from(1) })
}

fn print_summary(outcome: &Outcome, required: &BTreeSet<String>) {
    println!();
    println!("{:<14}{:<18}{:<13}{}", "gate", "status", "owner", "required");
    for g in &outcome.gates {
        println!(
            "{:<14}{:<18}{:<13}{}",
            g.name,
            g.status.as_str(),
            g.owner,
            if required.contains(g.name) { "yes" } else { "no" }
        );
    }
    println!();
    for g in outcome.gates.iter().filter(|g| g.status == Status::Fail) {
        println!("FAILED {}:", g.name);
        for f in g.failures.iter().take(20) {
            println!("  {}: {}", f.subject, f.detail);
        }
        if g.failures.len() > 20 {
            println!("  ... and {} more, see harness/report/latest.json", g.failures.len() - 20);
        }
    }
    for g in outcome.gates.iter().filter(|g| required.contains(g.name) && g.status != Status::Pass && g.status != Status::Fail) {
        println!("REQUIRED but {}: {}", g.status.as_str(), g.name);
    }
    let count = |s: Status| outcome.gates.iter().filter(|g| g.status == s).count();
    println!(
        "{} pass, {} fail, {} not_implemented. assertions: {} (baseline {}), {} added",
        count(Status::Pass),
        count(Status::Fail),
        count(Status::NotImplemented),
        outcome.assertions.total,
        outcome.assertions.previous_total,
        outcome.assertions.added.len()
    );
}

fn gate(name: &str, o: &Opts) -> Result<ExitCode, String> {
    let Some(def) = gates::GATES.iter().find(|g| g.name == name) else {
        let names: Vec<&str> = gates::GATES.iter().map(|g| g.name).collect();
        return Err(format!("unknown gate '{name}'. Gates: {}", names.join(", ")));
    };
    if !gates::is_wired(name) {
        println!("{name}: not yet implemented, owned by {}, see docs/07-verification.md", def.owner);
        return Ok(ExitCode::from(EXIT_NOT_IMPLEMENTED));
    }
    let root = root();
    let g = if name == "scope" {
        // The scope gate reads files, not test output, so it runs alone in under a second.
        let scan = scope::scan_repo(&root);
        println!(
            "scanned {} files; {} on the allow-list, {} not text, {} skipped (gone or links)",
            scan.files_scanned, scan.files_allowed, scan.files_not_text, scan.files_skipped
        );
        gates::scope_gate(def, &scan)
    } else {
        let (outcome, _) = run_all(&root, o)?;
        outcome.gates.iter().find(|g| g.name == name).expect("gate is in the table").clone()
    };
    for m in &g.metrics {
        println!("{:<44}{} {:<3} {}", m.key, m.value, m.cmp.as_str(), m.threshold);
    }
    for f in g.failures.iter().take(20) {
        println!("  {}: {}", f.subject, f.detail);
    }
    println!("{name}: {}", g.status.as_str());
    Ok(if g.status == Status::Pass { ExitCode::SUCCESS } else { ExitCode::from(1) })
}

fn update_baseline(o: &Opts) -> Result<ExitCode, String> {
    let root = root();
    let mut baseline = load_baseline(&root)?;
    let fixtures = conformance_fixtures(&root);
    let run = run_tests(&root, o.release)?;
    if run.build_error.is_some() || !run.parsed.failed().is_empty() {
        println!("refusing to update the baseline: the tests do not pass. See harness/report/cargo-test.log");
        return Ok(ExitCode::from(1));
    }
    let dups = run.parsed.duplicate_ids();
    if !dups.is_empty() {
        return Err(format!("duplicate test ids, cannot build a baseline: {dups:?}"));
    }
    let passing: BTreeSet<String> = run.parsed.passed().into_iter().map(String::from).collect();

    for (kind, id, adr) in &o.removals {
        if !adr_exists(&root, adr) {
            return Err(format!("{adr} does not exist in adr/. Write the ADR first"));
        }
        let was_listed = if kind == "test" { baseline.tests.remove(id) } else { baseline.fixtures.remove(id) };
        if !was_listed {
            return Err(format!("{kind} '{id}' is not in the baseline"));
        }
        baseline.removed.insert((kind.clone(), id.clone(), adr.clone()));
    }

    let missing = baseline.check_run(&passing, &fixtures);
    if !missing.is_empty() {
        println!("refusing to drop entries silently. Each needs --remove <kind> <id> --adr ADR-NNNN:");
        for v in &missing {
            println!("  {}: {}", v.subject, v.detail);
        }
        return Ok(ExitCode::from(1));
    }

    let added = baseline.added(&passing, &fixtures);
    baseline.tests.extend(passing);
    baseline.fixtures.extend(fixtures);
    std::fs::write(root.join("harness/baseline.txt"), baseline.render()).map_err(|e| e.to_string())?;
    println!(
        "baseline updated: {} tests, {} fixtures, {} added, {} removal records",
        baseline.tests.len(),
        baseline.fixtures.len(),
        added.len(),
        baseline.removed.len()
    );
    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(s: &[&str]) -> Vec<String> {
        s.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn options_parse() {
        let o = parse_opts(&args(&["--release", "--base", "origin/main"])).unwrap();
        assert!(o.release);
        assert_eq!(o.base.as_deref(), Some("origin/main"));
    }

    #[test]
    fn a_removal_needs_an_adr() {
        assert!(parse_opts(&args(&["--remove", "test", "a::b"])).is_err());
        assert!(parse_opts(&args(&["--remove", "test", "a::b", "--adr"])).is_err());
        assert!(parse_opts(&args(&["--remove", "widget", "a::b", "--adr", "ADR-0001"])).is_err());
        let o = parse_opts(&args(&["--remove", "test", "a::b", "--adr", "ADR-0001", "--release"])).unwrap();
        assert_eq!(o.removals, [("test".to_string(), "a::b".to_string(), "ADR-0001".to_string())]);
        assert!(o.release);
    }

    #[test]
    fn unknown_options_are_errors() {
        assert!(parse_opts(&args(&["--frobnicate"])).is_err());
        assert!(parse_opts(&args(&["--base"])).is_err());
    }

    #[test]
    fn adr_ids_must_be_well_formed_and_exist() {
        let r = root();
        assert!(adr_exists(&r, "ADR-0001"));
        assert!(!adr_exists(&r, "ADR-9999"));
        assert!(!adr_exists(&r, "0001"));
        assert!(!adr_exists(&r, "ADR-1"));
    }

    #[test]
    fn fixtures_exclude_pending_and_use_forward_slashes() {
        let f = conformance_fixtures(&root());
        assert!(f.iter().all(|p| p.starts_with("tests/conformance/") && (p.ends_with(".fixture") || p.ends_with(".panel"))));
        assert!(f.iter().all(|p| !p.contains("/pending/")));
        assert!(f.iter().all(|p| !p.contains('\\')));
        assert!(!f.is_empty());
    }

    #[test]
    fn panel_fixtures_count_as_fixtures_and_pending_ones_do_not() {
        let f = conformance_fixtures(&root());
        assert!(f.iter().any(|p| p.ends_with(".fixture")), "the engine fixtures are still counted");
        assert!(f.iter().any(|p| p.starts_with("tests/conformance/panel/") && p.ends_with(".panel")), "the panel fixtures are counted");
        assert!(f.iter().all(|p| !p.starts_with("tests/conformance/panel/pending/")));
    }

    #[test]
    fn a_panel_file_outside_the_panel_directory_is_not_counted_as_a_fixture() {
        // The runner only reads tests/conformance/panel/, so a count that included a stray file
        // would be an assertion that never ran.
        let root = std::env::temp_dir().join(format!("xtask-main-{}-stray", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        for rel in ["tests/conformance/a.fixture", "tests/conformance/elsewhere/stray.panel", "tests/conformance/panel/edit/live.panel"] {
            let f = root.join(rel);
            std::fs::create_dir_all(f.parent().unwrap()).unwrap();
            std::fs::write(f, "").unwrap();
        }
        let got: Vec<String> = conformance_fixtures(&root).into_iter().collect();
        assert_eq!(got, ["tests/conformance/a.fixture", "tests/conformance/panel/edit/live.panel"]);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn required_gates_file_only_names_known_gates() {
        let req = load_required(&root()).unwrap();
        assert!(req.contains("conformance") && req.contains("regressions"));
    }

    #[test]
    fn every_required_gate_can_be_run_on_its_own() {
        // `verify` requires these to pass, so `xtask gate <name>` must be able to run them
        // and not answer "not implemented" (exit 42) while `verify` says the gate is real.
        for name in load_required(&root()).unwrap() {
            assert!(gates::is_wired(&name), "{name} is required by verify but `xtask gate {name}` is not wired");
        }
    }

    #[test]
    fn the_root_is_the_checkout_the_command_runs_in() {
        let r = root();
        assert!(r.join("harness/required-gates.txt").is_file());
        assert!(r.join("Cargo.toml").is_file());
    }
}
