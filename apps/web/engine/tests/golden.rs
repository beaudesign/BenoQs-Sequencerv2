//! The spike's export: a pattern run through the headless runner inside the module gives the
//! native golden hash (spike S2 repeats this in an AudioWorklet).

#![cfg(feature = "spike")]

mod common;

use common::*;
use octoweb::exports::*;

const PATTERNS: [&str; 5] = ["hello", "chords_and_strums", "effector", "phrases", "mcc_and_transport"];

fn run(source: &str) -> (bool, String) {
    write_scratch(source.as_bytes());
    let r = octoweb_spike_run(source.len() as u32);
    let len = (r & 0x7fff_ffff) as usize;
    (r >> 31 == 1, String::from_utf8_lossy(&read_scratch(len)).into_owned())
}

#[test]
fn the_five_golden_patterns_hash_to_their_native_goldens() {
    for name in PATTERNS {
        let source = std::fs::read_to_string(repo_root().join(format!("examples/{name}.pattern"))).unwrap();
        let golden = std::fs::read_to_string(repo_root().join(format!("examples/golden/{name}.sha256"))).unwrap();
        let want = golden.lines().next().unwrap().strip_prefix("events ").expect("golden line").to_string();
        let (is_error, got) = run(&source);
        assert!(!is_error, "{name}: {got}");
        assert_eq!(got, want, "{name}");
    }
}

#[test]
fn a_pattern_that_does_not_parse_comes_back_as_a_message() {
    let (is_error, text) = run("this is not a pattern\n");
    assert!(is_error);
    assert!(!text.is_empty());
}
