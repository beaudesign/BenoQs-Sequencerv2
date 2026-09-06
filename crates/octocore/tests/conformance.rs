//! Walks `tests/conformance/**/*.fixture` (top-level, shared with any future
//! non-Rust consumer) and runs each one through `octocore::fixture::run_fixture`.
//! `docs/03-sequencer-core.md` §8: "Gate: 100%. No expected failures, no skips."
//! Fixtures describing not-yet-implemented behaviour belong in
//! `tests/conformance/pending/`, which this walk deliberately excludes.

use std::path::{Path, PathBuf};

fn conformance_root() -> PathBuf {
    // CARGO_MANIFEST_DIR = crates/octocore ; the fixtures live at the repo root.
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/conformance")
}

fn collect_fixtures(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().map(|n| n == "pending").unwrap_or(false) {
                continue;
            }
            collect_fixtures(&path, out);
        } else if path.extension().map(|e| e == "fixture").unwrap_or(false) {
            out.push(path);
        }
    }
}

#[test]
fn all_conformance_fixtures_pass() {
    let root = conformance_root();
    let mut fixtures = Vec::new();
    collect_fixtures(&root, &mut fixtures);

    assert!(!fixtures.is_empty(), "no fixtures found under {}", root.display());

    let mut failures = Vec::new();
    for path in &fixtures {
        let source = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
        if let Err(msg) = octocore::fixture::run_fixture(&source) {
            failures.push(format!("{}: {}", path.display(), msg));
        }
    }

    assert!(failures.is_empty(), "conformance failures:\n{}", failures.join("\n"));
}
