//! Golden event streams (SPEC-0001 O7, WENGE-0007) and the determinism they prove.
//!
//! Each pattern in `examples/` has a two-line file in `examples/golden/` with the SHA-256 of
//! its event log and of its Standard MIDI File. The engine is deterministic for a seed, so a
//! changed hash means the engine now plays something different. If that change is intended,
//! regenerate with `just golden` and say why in the commit. The `determinism` gate reads this
//! test binary: it passes when every test here passes and all five patterns ran.

use std::path::{Path, PathBuf};

fn examples() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples")
}

fn run(name: &str) -> octorun::Output {
    let path = examples().join(format!("{name}.pattern"));
    let source = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    octorun::run_pattern(&source).unwrap_or_else(|e| panic!("{name}: {e}"))
}

fn check(name: &str) {
    let out = run(name);
    assert!(out.events > 0, "{name}: a golden pattern that plays nothing proves nothing");
    let path = examples().join("golden").join(format!("{name}.sha256"));
    let want = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}. Run `just golden`.", path.display()));
    assert_eq!(
        out.golden_text(),
        want,
        "\n{name}: the engine now plays this pattern differently.\nIf that is intended, run `just golden`, review the diff of the log with `just run examples/{name}`, and explain it in the commit."
    );
}

macro_rules! golden {
    ($($name:ident),+ $(,)?) => { $( #[test] fn $name() { check(stringify!($name)) } )+ };
}

golden!(hello, chords_and_strums, phrases, effector, mcc_and_transport);

/// Same pattern, same seed, same bytes. Run three times, in one process.
#[test]
fn a_pattern_gives_identical_bytes_every_time() {
    for name in ["hello", "mcc_and_transport"] {
        let first = run(name);
        for _ in 0..2 {
            let again = run(name);
            assert_eq!(first.ndjson, again.ndjson, "{name}: event log differs between runs");
            assert_eq!(first.smf, again.smf, "{name}: MIDI file differs between runs");
        }
    }
}

/// A golden hash of a stream with refused notes would lock in a wrong performance. Every
/// example must play in full: nothing refused, deferred or late.
#[test]
fn no_golden_pattern_overflows_the_queue_or_runs_late() {
    for name in ["hello", "chords_and_strums", "phrases", "effector", "mcc_and_transport"] {
        let out = run(name);
        assert!(out.warnings().is_empty(), "{name}: {:?} ({:?})", out.warnings(), out.diagnostics);
    }
}

/// A different seed is a different performance. Without this, the tests above could pass on a
/// runner that ignored the seed. Random direction (dir 5) and a random groove (grv 4) draw
/// from the seeded generator.
#[test]
fn the_seed_matters() {
    let pattern = |seed: u32| {
        let src = format!(
            "seed {seed}\nbpm 200\ntrack 0 dir 5\ntrack 0 grv 4\ntrack 0 step all active 1\ntrack 0 step all pit +1\ntrack 0 step 3 pit +9\ntrack 0 step 9 pit -7\nplay\nrender 3 s buffer=64\n"
        );
        octorun::run_pattern(&src).unwrap().ndjson
    };
    assert_eq!(pattern(1), pattern(1), "the same seed repeats");
    assert_ne!(pattern(1), pattern(2), "a different seed changes the performance");
}

/// Every pattern has a golden file and every golden file a pattern, so a new example cannot
/// go in without its hash and a removed one cannot leave a stale hash behind.
#[test]
fn every_example_has_a_golden_file_and_the_other_way_round() {
    let names = |dir: PathBuf, ext: &str| {
        let mut v: Vec<String> = std::fs::read_dir(&dir)
            .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
            .flatten()
            .filter(|e| e.path().extension().map_or(false, |x| x == ext))
            .map(|e| e.path().file_stem().unwrap().to_string_lossy().into_owned())
            .collect();
        v.sort();
        v
    };
    let patterns = names(examples(), "pattern");
    let goldens = names(examples().join("golden"), "sha256");
    assert_eq!(patterns, goldens);
    assert!(patterns.len() >= 5, "the spec asks for golden hashes of 5 patterns, found {}", patterns.len());
}
