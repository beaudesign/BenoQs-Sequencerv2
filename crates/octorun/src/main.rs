//! `octorun PATTERN [--out DIR] [--stdout] [--golden]`. See `just run`.

use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "\
usage: octorun PATTERN [--out DIR]
       octorun PATTERN --stdout
       octorun PATTERN --golden

Runs PATTERN (the fixture language, see crates/octocore/src/fixture.rs) headless.
By default writes DIR/NAME.ndjson and DIR/NAME.mid (DIR defaults to target/octorun) and prints
one line with the event count and the SHA-256 of each file.
  --stdout   write the event log to standard output and nothing else
  --golden   print the two-line text kept in examples/golden/NAME.sha256
PATTERN may omit its .pattern extension.";

fn resolve(arg: &str) -> Result<PathBuf, String> {
    let direct = PathBuf::from(arg);
    if direct.is_file() {
        return Ok(direct);
    }
    let with_ext = PathBuf::from(format!("{arg}.pattern"));
    if with_ext.is_file() {
        return Ok(with_ext);
    }
    Err(format!("no such pattern: {arg} (also tried {arg}.pattern)"))
}

fn run() -> Result<(), String> {
    let mut pattern: Option<String> = None;
    let mut out_dir = PathBuf::from("target/octorun");
    let (mut to_stdout, mut golden) = (false, false);
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "-h" | "--help" => {
                println!("{USAGE}");
                return Ok(());
            }
            "--stdout" => to_stdout = true,
            "--golden" => golden = true,
            "--out" => out_dir = PathBuf::from(args.next().ok_or("--out needs a directory")?),
            other if other.starts_with('-') => return Err(format!("unknown option {other}\n{USAGE}")),
            other => {
                if pattern.replace(other.to_string()).is_some() {
                    return Err(format!("one pattern at a time\n{USAGE}"));
                }
            }
        }
    }
    let path = resolve(&pattern.ok_or_else(|| USAGE.to_string())?)?;
    let source = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let out = octorun::run_pattern(&source).map_err(|e| format!("{}: {e}", path.display()))?;
    for w in out.warnings() {
        eprintln!("octorun: warning: {}: {w}", path.display());
    }

    if to_stdout {
        print!("{}", out.ndjson);
        return Ok(());
    }
    if golden {
        print!("{}", out.golden_text());
        return Ok(());
    }
    let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("out");
    std::fs::create_dir_all(&out_dir).map_err(|e| format!("{}: {e}", out_dir.display()))?;
    let write = |file: PathBuf, bytes: &[u8]| std::fs::write(&file, bytes).map_err(|e| format!("{}: {e}", file.display()));
    write(out_dir.join(format!("{name}.ndjson")), out.ndjson.as_bytes())?;
    write(out_dir.join(format!("{name}.mid")), &out.smf)?;
    println!(
        "{name}: {} events, {:.2} s. events sha256 {}  midi sha256 {}  -> {}",
        out.events,
        out.samples as f64 / 48_000.0,
        out.ndjson_sha256(),
        out.smf_sha256(),
        out_dir.display()
    );
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("octorun: {e}");
            ExitCode::FAILURE
        }
    }
}
