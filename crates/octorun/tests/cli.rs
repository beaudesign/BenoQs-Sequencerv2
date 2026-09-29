//! The `octorun` binary itself: exit codes, files written, and byte-identical output on a
//! second run.

use std::path::{Path, PathBuf};
use std::process::Command;

fn octorun() -> Command {
    Command::new(env!("CARGO_BIN_EXE_octorun"))
}

fn example(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples").join(name)
}

fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("octorun-cli-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

#[test]
fn writes_the_log_and_the_midi_file_and_prints_their_hashes_twice_the_same() {
    let (a, b) = (scratch("a"), scratch("b"));
    let mut lines = Vec::new();
    for dir in [&a, &b] {
        let out = octorun().arg(example("hello")).arg("--out").arg(dir).output().unwrap();
        assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
        let line = String::from_utf8(out.stdout).unwrap();
        // The line ends with the output directory, which differs; compare up to it.
        lines.push(line.split("  -> ").next().unwrap().to_string());
    }
    assert_eq!(lines[0], lines[1], "the same pattern prints the same hashes");
    for file in ["hello.ndjson", "hello.mid"] {
        assert_eq!(std::fs::read(a.join(file)).unwrap(), std::fs::read(b.join(file)).unwrap(), "{file}");
    }
    let mid = std::fs::read(a.join("hello.mid")).unwrap();
    assert_eq!(&mid[..4], b"MThd");
    let _ = (std::fs::remove_dir_all(&a), std::fs::remove_dir_all(&b));
}

#[test]
fn the_extension_is_optional_and_stdout_and_golden_modes_write_nothing_else() {
    let with_ext = octorun().arg(example("hello.pattern")).arg("--stdout").output().unwrap();
    let without = octorun().arg(example("hello")).arg("--stdout").output().unwrap();
    assert!(with_ext.status.success() && without.status.success());
    assert_eq!(with_ext.stdout, without.stdout);
    assert!(String::from_utf8(with_ext.stdout).unwrap().starts_with("{\"octorun\":\"events/1\""));

    let golden = octorun().arg(example("hello")).arg("--golden").output().unwrap();
    let want = std::fs::read_to_string(example("golden/hello.sha256")).unwrap();
    assert_eq!(String::from_utf8(golden.stdout).unwrap(), want);
}

/// Ten tracks whose every step plays a 4-note phrase, each note delayed by 250 ticks and held
/// for 255 more, keep thousands of events waiting at once, more than the queue holds. The
/// engine then refuses notes, and the runner has to say so.
#[test]
fn a_pattern_that_overloads_the_engine_says_so_on_stderr_and_a_clean_one_does_not() {
    let dir = scratch("overload");
    std::fs::create_dir_all(&dir).unwrap();
    let mut src = String::from("seed 1\nbpm 120\n");
    for n in 0..4 {
        src.push_str(&format!("phrase 0 note {n} pit {}\nphrase 0 note {n} sta 250\nphrase 0 note {n} len 255\nphrase 0 note {n} enabled 1\n", n * 3));
    }
    for t in 0..10 {
        src.push_str(&format!("track {t} mch {}\ntrack {t} step all active 1\ntrack {t} step all phrase 1\n", t + 1));
    }
    src.push_str("play\nrender 10 s buffer=64\n");
    let file = dir.join("overload.pattern");
    std::fs::write(&file, src).unwrap();
    let out = octorun().arg(&file).arg("--out").arg(&dir).output().unwrap();
    assert!(out.status.success(), "the output is still written: {}", String::from_utf8_lossy(&out.stderr));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("warning") && err.contains("refused") && err.contains("queue"), "{err}");
    assert!(dir.join("overload.mid").exists());

    let clean = octorun().arg(example("hello")).arg("--stdout").output().unwrap();
    assert!(clean.stderr.is_empty(), "{}", String::from_utf8_lossy(&clean.stderr));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn bad_input_is_an_error_message_and_a_failing_exit_code_not_a_panic() {
    let missing = octorun().arg("no/such/pattern").output().unwrap();
    assert!(!missing.status.success());
    assert!(String::from_utf8_lossy(&missing.stderr).contains("no such pattern"));

    let dir = scratch("bad");
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("broken.pattern");
    std::fs::write(&file, "bpm fast\n").unwrap();
    let bad = octorun().arg(&file).arg("--stdout").output().unwrap();
    assert!(!bad.status.success());
    assert!(String::from_utf8_lossy(&bad.stderr).contains("bad bpm"), "{}", String::from_utf8_lossy(&bad.stderr));

    let huge = dir.join("huge.pattern");
    std::fs::write(&huge, "render 100000 s buffer=64\n").unwrap();
    let out = octorun().arg(&huge).arg("--stdout").output().unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("sample limit"));

    let unknown = octorun().arg("--frobnicate").output().unwrap();
    assert!(!unknown.status.success());
    let help = octorun().arg("--help").output().unwrap();
    assert!(help.status.success() && String::from_utf8_lossy(&help.stdout).contains("usage: octorun"));
    let _ = std::fs::remove_dir_all(&dir);
}
