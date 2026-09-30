//! The gate table and the evaluation of the two gates that exist today.
//!
//! Every gate in `contracts/verification.report.schema.json` is listed. A gate with no
//! implementation reports `not_implemented`, never `pass`. `harness/required-gates.txt`
//! says which gates must be `pass` for the run to succeed. That file only grows.

use std::collections::BTreeSet;

use crate::baseline::{Baseline, Violation};
use crate::cargo_out::Parsed;
use crate::scope::{self, Scan};

pub struct GateDef {
    pub name: &'static str,
    pub owner: &'static str,
}

/// Same order as the old `justfile` zone list, with `persistence` (in the schema and in
/// docs/02) added at the end.
pub const GATES: &[GateDef] = &[
    GateDef { name: "timing", owner: "metronome" },
    GateDef { name: "conformance", owner: "metronome" },
    GateDef { name: "a11y", owner: "referee" },
    GateDef { name: "arch", owner: "conductor" },
    GateDef { name: "slop", owner: "curator" },
    GateDef { name: "tokens", owner: "curator" },
    GateDef { name: "regressions", owner: "referee" },
    GateDef { name: "determinism", owner: "referee" },
    GateDef { name: "persistence", owner: "conductor" },
    GateDef { name: "scope", owner: "referee" },
];

/// The gates that `evaluate` computes. Every other gate reports `not_implemented`.
/// `xtask gate <name>` uses this list, so it cannot disagree with `xtask verify`. A test
/// checks that this list and the match in `evaluate` say the same thing.
pub const WIRED: &[&str] = &["conformance", "regressions", "determinism", "scope"];

pub fn is_wired(name: &str) -> bool {
    WIRED.contains(&name)
}

pub fn is_known(name: &str) -> bool {
    GATES.iter().any(|g| g.name == name)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Pass,
    Fail,
    #[allow(dead_code)] // part of the report schema; no gate reports it yet
    Skipped,
    NotImplemented,
}

impl Status {
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Pass => "pass",
            Status::Fail => "fail",
            Status::Skipped => "skipped",
            Status::NotImplemented => "not_implemented",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Comparator {
    Gte,
    Eq,
}

impl Comparator {
    pub fn as_str(self) -> &'static str {
        match self {
            Comparator::Gte => "gte",
            Comparator::Eq => "eq",
        }
    }

    fn holds(self, value: f64, threshold: f64) -> bool {
        match self {
            Comparator::Gte => value >= threshold,
            Comparator::Eq => value == threshold,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Metric {
    pub key: String,
    pub value: f64,
    pub unit: Option<&'static str>,
    pub threshold: f64,
    pub cmp: Comparator,
}

impl Metric {
    fn new(key: &str, value: usize, unit: &'static str, cmp: Comparator, threshold: usize) -> Metric {
        Metric { key: key.to_string(), value: value as f64, unit: Some(unit), threshold: threshold as f64, cmp }
    }

    pub fn passes(&self) -> bool {
        self.cmp.holds(self.value, self.threshold)
    }
}

#[derive(Debug, Clone)]
pub struct Failure {
    pub subject: String,
    pub detail: String,
}

impl Failure {
    fn new(subject: impl Into<String>, detail: impl Into<String>) -> Failure {
        Failure { subject: subject.into(), detail: detail.into() }
    }
}

impl From<Violation> for Failure {
    fn from(v: Violation) -> Failure {
        Failure { subject: v.subject, detail: v.detail }
    }
}

#[derive(Debug, Clone)]
pub struct Gate {
    pub name: &'static str,
    pub owner: &'static str,
    pub status: Status,
    pub duration_s: Option<f64>,
    pub metrics: Vec<Metric>,
    pub failures: Vec<Failure>,
}

/// What running `cargo test --workspace` produced.
pub struct TestRun {
    pub parsed: Parsed,
    pub duration_s: f64,
    /// Set when cargo did not produce usable results, for example a compile error.
    pub build_error: Option<String>,
}

pub struct Inputs<'a> {
    pub run: &'a TestRun,
    /// Conformance fixtures on disk (excluding `pending/`), as repo-relative paths.
    pub fixtures: &'a BTreeSet<String>,
    pub baseline: &'a Baseline,
    /// The baseline at the base ref, when `--base` was given.
    pub base: Option<&'a Baseline>,
    pub adr_exists: &'a dyn Fn(&str) -> bool,
    /// The scan of the tree for the banned phrases (`scope`).
    pub scope: &'a Scan,
}

#[derive(Debug, Clone, Default)]
pub struct Assertions {
    pub total: usize,
    pub previous_total: usize,
    pub by_gate: Vec<(String, usize)>,
    pub added: Vec<String>,
    pub removed: Vec<(String, String)>,
}

/// One assertion is one passing test case or one conformance fixture. That is a proxy
/// for the docs/07 "assertion" until per-assertion counting exists. See harness/README.md.
pub fn evaluate(inp: &Inputs) -> (Vec<Gate>, Assertions) {
    let passing: BTreeSet<String> = inp.run.parsed.passed().into_iter().map(String::from).collect();
    let failed: Vec<String> = inp.run.parsed.failed().into_iter().map(String::from).collect();

    let base_violations: Option<Vec<Violation>> = inp.base.map(|b| inp.baseline.check_against_base(b, inp.adr_exists));

    let gates = GATES
        .iter()
        .map(|def| match def.name {
            "conformance" => conformance(def, inp),
            "determinism" => determinism(def, inp),
            "scope" => scope_gate(def, inp.scope),
            "regressions" => regressions(def, inp, &passing, &failed, base_violations.as_deref()),
            _ => Gate {
                name: def.name,
                owner: def.owner,
                status: Status::NotImplemented,
                duration_s: None,
                metrics: Vec::new(),
                failures: Vec::new(),
            },
        })
        .collect();

    let removed = match inp.base {
        Some(b) => inp
            .baseline
            .removed
            .difference(&b.removed)
            .map(|(_, id, adr)| (id.clone(), adr.clone()))
            .collect(),
        None => Vec::new(),
    };

    let assertions = Assertions {
        total: passing.len() + inp.fixtures.len(),
        previous_total: inp.baseline.total(),
        by_gate: vec![
            ("regressions".to_string(), passing.len()),
            ("conformance".to_string(), inp.fixtures.len()),
        ],
        added: inp.baseline.added(&passing, inp.fixtures),
        removed,
    };
    (gates, assertions)
}

fn finish(def: &GateDef, duration_s: f64, metrics: Vec<Metric>, failures: Vec<Failure>) -> Gate {
    let ok = failures.is_empty() && metrics.iter().all(Metric::passes);
    Gate {
        name: def.name,
        owner: def.owner,
        status: if ok { Status::Pass } else { Status::Fail },
        duration_s: Some(duration_s),
        metrics,
        failures,
    }
}

fn conformance(def: &GateDef, inp: &Inputs) -> Gate {
    let mut failures = Vec::new();
    if let Some(e) = &inp.run.build_error {
        failures.push(Failure::new("cargo test", e.clone()));
    }
    let (n_ok, n_bad) = match inp.run.parsed.binary("conformance") {
        Some(b) => {
            for t in b.tests.iter().filter(|t| t.outcome == crate::cargo_out::Outcome::Failed) {
                failures.push(Failure::new(t.id.clone(), "conformance test failed"));
            }
            let ok = b.tests.iter().filter(|t| t.outcome == crate::cargo_out::Outcome::Ok).count();
            let bad = b.tests.iter().filter(|t| t.outcome == crate::cargo_out::Outcome::Failed).count();
            (ok, bad)
        }
        None => {
            if inp.run.build_error.is_none() {
                failures.push(Failure::new("conformance", "the conformance test binary did not run"));
            }
            (0, 0)
        }
    };
    let metrics = vec![
        Metric::new("conformance.tests_passed", n_ok, "tests", Comparator::Gte, 1),
        Metric::new("conformance.tests_failed", n_bad, "tests", Comparator::Eq, 0),
        Metric::new("conformance.fixtures", inp.fixtures.len(), "fixtures", Comparator::Gte, inp.baseline.fixtures.len()),
    ];
    finish(def, inp.run.duration_s, metrics, failures)
}

/// Golden event streams (SPEC-0001 O7). The `golden` test binary of `octorun` runs each
/// example pattern headless and compares the SHA-256 of its event log and MIDI file with
/// `examples/golden/`. The gate passes when at least `MIN_GOLDEN` of those tests ran and none failed.
const MIN_GOLDEN: usize = 5;

fn determinism(def: &GateDef, inp: &Inputs) -> Gate {
    let mut failures = Vec::new();
    if let Some(e) = &inp.run.build_error {
        failures.push(Failure::new("cargo test", e.clone()));
    }
    let (n_ok, n_bad) = match inp.run.parsed.binary("golden") {
        Some(b) => {
            for t in b.tests.iter().filter(|t| t.outcome == crate::cargo_out::Outcome::Failed) {
                failures.push(Failure::new(t.id.clone(), "golden or determinism test failed"));
            }
            let ok = b.tests.iter().filter(|t| t.outcome == crate::cargo_out::Outcome::Ok).count();
            let bad = b.tests.iter().filter(|t| t.outcome == crate::cargo_out::Outcome::Failed).count();
            (ok, bad)
        }
        None => {
            if inp.run.build_error.is_none() {
                failures.push(Failure::new("determinism", "the octorun golden test binary did not run"));
            }
            (0, 0)
        }
    };
    let metrics = vec![
        Metric::new("determinism.golden_tests_passed", n_ok, "tests", Comparator::Gte, MIN_GOLDEN),
        Metric::new("determinism.golden_tests_failed", n_bad, "tests", Comparator::Eq, 0),
    ];
    finish(def, inp.run.duration_s, metrics, failures)
}

/// The rooms product must stay out of the tree (ADR-0006, `specs/SPEC-0002/removal-plan.md`
/// section 5). Passes when no banned phrase sits outside the allow-list, the banned list still
/// has its floor of phrases, and the scan looked at a real tree.
pub fn scope_gate(def: &GateDef, scan: &Scan) -> Gate {
    let mut failures = Vec::new();
    for p in &scan.problems {
        failures.push(Failure::new("scope", p.clone()));
    }
    for h in &scan.hits {
        let at = if h.line == 0 { format!("{} (path)", h.path) } else { format!("{}:{}", h.path, h.line) };
        failures.push(Failure::new(
            at,
            format!("banned phrase '{}'. Remove it. An allow-list entry needs the Referee (harness/scope-allow.txt)", h.phrase),
        ));
    }
    let metrics = vec![
        Metric::new("scope.banned_phrases_found", scan.hits.len(), "hits", Comparator::Eq, 0),
        Metric::new("scope.banned_phrases_listed", scan.banned_listed, "phrases", Comparator::Gte, scope::MIN_BANNED),
        Metric::new("scope.files_scanned", scan.files_scanned, "files", Comparator::Gte, scope::MIN_FILES_SCANNED),
    ];
    finish(def, scan.duration_s, metrics, failures)
}

fn regressions(
    def: &GateDef,
    inp: &Inputs,
    passing: &BTreeSet<String>,
    failed: &[String],
    base_violations: Option<&[Violation]>,
) -> Gate {
    let mut failures = Vec::new();
    if let Some(e) = &inp.run.build_error {
        failures.push(Failure::new("cargo test", e.clone()));
    } else if passing.is_empty() && failed.is_empty() {
        failures.push(Failure::new("cargo test", "no tests ran"));
    }
    for id in failed {
        failures.push(Failure::new(id.clone(), "test failed"));
    }
    for id in inp.run.parsed.duplicate_ids() {
        failures.push(Failure::new(id, "test id appears twice; two test binaries share a name"));
    }
    for m in inp.run.parsed.summary_mismatches() {
        failures.push(Failure::new("cargo test output", format!("parser disagrees with cargo: {m}")));
    }
    if inp.baseline.total() == 0 {
        failures.push(Failure::new(
            "harness/baseline.txt",
            "missing or empty. Run `cargo xtask baseline` on a green tree and commit the result",
        ));
    }

    let run_violations = inp.baseline.check_run(passing, inp.fixtures);
    let n_run_violations = run_violations.len();
    failures.extend(run_violations.into_iter().map(Failure::from));

    let mut metrics = vec![
        Metric::new("regressions.tests_passed", passing.len(), "tests", Comparator::Gte, inp.baseline.tests.len()),
        Metric::new("regressions.tests_failed", failed.len(), "tests", Comparator::Eq, 0),
        Metric::new("regressions.baseline_entries_not_passing", n_run_violations, "entries", Comparator::Eq, 0),
    ];
    if let Some(bv) = base_violations {
        metrics.push(Metric::new("regressions.removed_without_adr", bv.len(), "entries", Comparator::Eq, 0));
        failures.extend(bv.iter().cloned().map(Failure::from));
    }
    finish(def, inp.run.duration_s, metrics, failures)
}

/// Exit code for the whole run. Fail if any gate failed, or if any required gate is not
/// `pass`. A `not_implemented` gate that is not required does not fail the run, but it
/// is printed and written to the report so it cannot pass for green.
pub fn run_succeeds(gates: &[Gate], required: &BTreeSet<String>) -> bool {
    gates.iter().all(|g| g.status != Status::Fail)
        && gates.iter().filter(|g| required.contains(g.name)).all(|g| g.status == Status::Pass)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cargo_out::parse;

    const OUT: &str = "\
     Running unittests src/lib.rs (target/debug/deps/octocore-0123456789abcdef)
test a::one ... ok
test a::two ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/conformance.rs (target/debug/deps/conformance-0123456789abcdef)
test conformance_fixtures ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
";

    fn set(items: &[&str]) -> BTreeSet<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    fn baseline() -> Baseline {
        Baseline {
            tests: set(&["octocore::a::one", "octocore::a::two", "conformance::conformance_fixtures"]),
            fixtures: set(&["tests/conformance/x.fixture"]),
            removed: BTreeSet::new(),
        }
    }

    fn eval(out: &str, baseline: &Baseline, base: Option<&Baseline>) -> Vec<Gate> {
        let run = TestRun { parsed: parse(out), duration_s: 1.0, build_error: None };
        let fixtures = set(&["tests/conformance/x.fixture"]);
        let inputs = Inputs { run: &run, fixtures: &fixtures, baseline, base, adr_exists: &|_| false, scope: &Scan::clean() };
        evaluate(&inputs).0
    }

    fn status(gates: &[Gate], name: &str) -> Status {
        gates.iter().find(|g| g.name == name).unwrap().status
    }

    #[test]
    fn the_wired_list_matches_what_evaluate_computes() {
        let gates = eval(&full_run(), &baseline(), None);
        for g in &gates {
            assert_eq!(
                g.status != Status::NotImplemented,
                is_wired(g.name),
                "{}: WIRED and evaluate() disagree",
                g.name
            );
        }
        for name in WIRED {
            assert!(is_known(name), "{name} is in WIRED but not in GATES");
        }
    }

    #[test]
    fn every_schema_gate_is_listed_exactly_once() {
        let schema = [
            "determinism", "slop", "tokens", "timing", "conformance", "a11y", "arch",
            "persistence", "regressions", "scope",
        ];
        assert_eq!(GATES.len(), schema.len());
        for name in schema {
            assert_eq!(GATES.iter().filter(|g| g.name == name).count(), 1, "{name}");
        }
    }

    #[test]
    fn a_clean_run_passes_the_two_wired_gates_and_the_rest_are_not_implemented() {
        let gates = eval(&full_run(), &baseline(), None);
        assert_eq!(status(&gates, "conformance"), Status::Pass);
        assert_eq!(status(&gates, "regressions"), Status::Pass);
        assert_eq!(status(&gates, "determinism"), Status::Pass);
        assert_eq!(status(&gates, "slop"), Status::NotImplemented);
        assert_eq!(status(&gates, "scope"), Status::Pass);
        assert_eq!(gates.iter().filter(|g| g.status == Status::NotImplemented).count(), GATES.len() - WIRED.len());
    }

    #[test]
    fn deleting_a_test_fails_regressions() {
        let without_two = OUT.replace("test a::two ... ok\n", "").replace("2 passed", "1 passed");
        let gates = eval(&without_two, &baseline(), None);
        assert_eq!(status(&gates, "regressions"), Status::Fail);
        let g = gates.iter().find(|g| g.name == "regressions").unwrap();
        assert!(g.failures.iter().any(|f| f.subject == "octocore::a::two"));
        assert!(!run_succeeds(&gates, &set(&["regressions"])));
    }

    #[test]
    fn a_failing_test_fails_the_run_even_when_nothing_is_required() {
        let bad = OUT.replace("test a::one ... ok", "test a::one ... FAILED").replace("2 passed; 0 failed", "1 passed; 1 failed");
        let gates = eval(&bad, &baseline(), None);
        assert!(!run_succeeds(&gates, &BTreeSet::new()));
    }

    #[test]
    fn a_required_gate_that_is_not_implemented_fails_the_run() {
        let gates = eval(&full_run(), &baseline(), None);
        assert!(run_succeeds(&gates, &set(&["conformance", "regressions", "determinism"])));
        assert!(!run_succeeds(&gates, &set(&["conformance", "regressions", "timing"])));
    }

    #[test]
    fn an_unrequired_stub_does_not_fail_the_run_but_is_never_reported_as_pass() {
        let gates = eval(&full_run(), &baseline(), None);
        assert!(run_succeeds(&gates, &BTreeSet::new()));
        assert_ne!(status(&gates, "timing"), Status::Pass);
    }

    #[test]
    fn an_empty_baseline_fails_regressions() {
        let gates = eval(OUT, &Baseline::default(), None);
        assert_eq!(status(&gates, "regressions"), Status::Fail);
    }

    #[test]
    fn the_conformance_gate_fails_when_its_binary_disappears() {
        let no_conf = OUT.split("     Running tests/conformance.rs").next().unwrap();
        let gates = eval(no_conf, &baseline(), None);
        assert_eq!(status(&gates, "conformance"), Status::Fail);
    }

    // Names of the next two tests still say "two"/"both": renaming a test drops it from the
    // ratchet baseline, and the name is not worth an ADR. Determinism is the third wired gate.
    #[test]
    fn a_build_error_fails_both_wired_gates() {
        let run = TestRun { parsed: Parsed::default(), duration_s: 0.5, build_error: Some("error[E0432]".into()) };
        let fixtures = set(&["tests/conformance/x.fixture"]);
        let inputs = Inputs { run: &run, fixtures: &fixtures, baseline: &baseline(), base: None, adr_exists: &|_| true, scope: &Scan::clean() };
        let (gates, _) = evaluate(&inputs);
        assert_eq!(status(&gates, "conformance"), Status::Fail);
        assert_eq!(status(&gates, "regressions"), Status::Fail);
        assert_eq!(status(&gates, "determinism"), Status::Fail);
    }

    #[test]
    fn base_check_catches_a_baseline_edited_to_hide_a_deletion() {
        let base = baseline();
        let mut now = baseline();
        now.tests.remove("octocore::a::two");
        // The run still passes everything the edited baseline lists...
        let without_two = OUT.replace("test a::two ... ok\n", "").replace("2 passed", "1 passed");
        let without_base = eval(&without_two, &now, None);
        assert_eq!(status(&without_base, "regressions"), Status::Pass);
        // ...so only the comparison against the base ref can catch it.
        let with_base = eval(&without_two, &now, Some(&base));
        assert_eq!(status(&with_base, "regressions"), Status::Fail);
    }

    #[test]
    fn assertions_count_tests_plus_fixtures_and_list_additions() {
        let run = TestRun { parsed: parse(OUT), duration_s: 1.0, build_error: None };
        let fixtures = set(&["tests/conformance/x.fixture", "tests/conformance/y.fixture"]);
        let mut b = baseline();
        b.tests.remove("octocore::a::two");
        let inputs = Inputs { run: &run, fixtures: &fixtures, baseline: &b, base: None, adr_exists: &|_| true, scope: &Scan::clean() };
        let (_, a) = evaluate(&inputs);
        assert_eq!(a.total, 3 + 2);
        assert_eq!(a.previous_total, 2 + 1);
        assert_eq!(a.added, ["octocore::a::two", "tests/conformance/y.fixture"]);
    }

    const GOLDEN: &str = "\
     Running tests/golden.rs (target/debug/deps/golden-0123456789abcdef)
test chords_and_strums ... ok
test effector ... ok
test hello ... ok
test mcc_and_transport ... ok
test phrases ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
";

    /// The output of a workspace run: the octocore and conformance binaries plus the golden one.
    fn full_run() -> String {
        format!("{OUT}\n{GOLDEN}")
    }

    #[test]
    fn the_determinism_gate_passes_on_five_golden_tests_and_fails_otherwise() {
        let out = full_run();
        assert_eq!(status(&eval(&out, &baseline(), None), "determinism"), Status::Pass);

        // No golden binary at all.
        assert_eq!(status(&eval(OUT, &baseline(), None), "determinism"), Status::Fail);

        // One golden test fails.
        let one_bad = out.replace("test hello ... ok", "test hello ... FAILED").replace("5 passed; 0 failed", "4 passed; 1 failed");
        assert_eq!(status(&eval(&one_bad, &baseline(), None), "determinism"), Status::Fail);

        // Only four ran: a pattern was dropped without saying so.
        let four = out.replace("test phrases ... ok\n", "").replace("5 passed", "4 passed");
        assert_eq!(status(&eval(&four, &baseline(), None), "determinism"), Status::Fail);
    }

    #[test]
    fn the_scope_gate_fails_on_a_hit_and_passes_on_a_clean_scan() {
        let def = GATES.iter().find(|g| g.name == "scope").unwrap();
        assert_eq!(scope_gate(def, &Scan::clean()).status, Status::Pass);

        let mut dirty = Scan::clean();
        dirty.hits.push(scope::Hit { path: "docs/a.md".into(), line: 7, phrase: "x".into() });
        let g = scope_gate(def, &dirty);
        assert_eq!(g.status, Status::Fail);
        assert_eq!(g.failures[0].subject, "docs/a.md:7");

        let mut in_path = Scan::clean();
        in_path.hits.push(scope::Hit { path: "crates/x/Cargo.toml".into(), line: 0, phrase: "x".into() });
        assert_eq!(scope_gate(def, &in_path).failures[0].subject, "crates/x/Cargo.toml (path)");
    }

    #[test]
    fn the_scope_gate_fails_when_it_scanned_almost_nothing_or_the_list_shrank() {
        let def = GATES.iter().find(|g| g.name == "scope").unwrap();
        let mut empty = Scan::clean();
        empty.files_scanned = 3;
        assert_eq!(scope_gate(def, &empty).status, Status::Fail, "a scan of three files proves nothing");

        let mut shrunk = Scan::clean();
        shrunk.banned_listed = scope::MIN_BANNED - 1;
        assert_eq!(scope_gate(def, &shrunk).status, Status::Fail, "dropping a phrase is a loosening");
    }

    #[test]
    fn the_scope_gate_reports_a_missing_list_file_as_a_failure() {
        let def = GATES.iter().find(|g| g.name == "scope").unwrap();
        let mut s = Scan::clean();
        s.problems.push("harness/scope-allow.txt: No such file or directory".into());
        let g = scope_gate(def, &s);
        assert_eq!(g.status, Status::Fail);
        assert!(g.failures[0].detail.contains("scope-allow.txt"));
    }
}
