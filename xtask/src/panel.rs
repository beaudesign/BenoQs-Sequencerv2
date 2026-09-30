//! Panel fixtures as the `conformance` gate sees them (ADR-0007 decision 4).
//!
//! A panel fixture is `tests/conformance/panel/**/*.panel`. The walker in `main.rs` counts the
//! ones that assert, like any other fixture. The ones in a `pending/` directory do not assert and
//! are not counted in the ratchet; this module reads them. A pending fixture closes by a hardware
//! check or the owner's ruling, never by a guess, so it must say what is ambiguous, where the
//! manual is, and which numbered question holds it:
//!
//! ```text
//! # pending: one sentence, the ambiguity
//! # manual: p013, p014
//! # question: Q31
//! ```
//!
//! A pending file without those three headers fails the gate. `crates/octoface/tests/fixture_files.rs`
//! checks the same rules from inside the test suite, and also that the question exists in
//! `QUESTIONS.md` or `findings.md`; the gate's check does not depend on that test still being there.

use std::path::Path;

/// What the scan of the `pending/` directories found.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Pending {
    /// Every pending panel fixture, repo-relative with forward slashes.
    pub files: Vec<String>,
    /// `(path, what is wrong)`.
    pub problems: Vec<(String, String)>,
}

/// What is wrong with the headers of a pending fixture. Empty when it is well formed.
pub fn header_problems(text: &str) -> Vec<String> {
    let _ = text;
    Vec::new()
}

/// Read every pending panel fixture under `<root>/tests/conformance`.
pub fn scan_pending(root: &Path) -> Pending {
    let _ = root;
    Pending::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: &str = "\
# panel fixture: a click on Step Mode with no key
# manual: p013
# workflow: Step zoom
# pending: p013 describes hold and press only.
# question: Q31
# No assertions.
";

    fn repo() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
    }

    #[test]
    fn a_pending_fixture_with_its_three_headers_is_well_formed() {
        assert_eq!(header_problems(GOOD), Vec::<String>::new());
    }

    #[test]
    fn each_missing_header_is_named() {
        for key in ["pending", "manual", "question"] {
            let without: String = GOOD.lines().filter(|l| !l.starts_with(&format!("# {key}:"))).map(|l| format!("{l}\n")).collect();
            let p = header_problems(&without);
            assert_eq!(p.len(), 1, "{key}: {p:?}");
            assert!(p[0].contains(key), "{key}: {p:?}");
        }
    }

    #[test]
    fn an_empty_header_counts_as_missing() {
        let empty = GOOD.replace("# pending: p013 describes hold and press only.", "# pending:");
        let p = header_problems(&empty);
        assert_eq!(p.len(), 1, "{p:?}");
        assert!(p[0].contains("pending"));
    }

    #[test]
    fn the_manual_header_needs_a_page_number_and_the_question_a_q_number() {
        let p = header_problems(&GOOD.replace("# manual: p013", "# manual: the front panel chapter"));
        assert_eq!(p.len(), 1, "{p:?}");
        assert!(p[0].contains("manual"));
        for bad in ["Q3", "Q031", "q31", "31", "Q31 and Q32"] {
            let p = header_problems(&GOOD.replace("# question: Q31", &format!("# question: {bad}")));
            assert_eq!(p.len(), 1, "{bad}: {p:?}");
            assert!(p[0].contains("question"), "{bad}: {p:?}");
        }
    }

    #[test]
    fn a_pending_fixture_that_asserts_is_a_problem() {
        let p = header_problems(&format!("{GOOD}at 0 ms click step(1,1)\nexpect mode step\n"));
        assert_eq!(p.len(), 1, "{p:?}");
        assert!(p[0].contains("assert"), "{p:?}");
    }

    #[test]
    fn the_scan_reads_the_real_pending_directory_and_finds_it_clean() {
        let s = scan_pending(&repo());
        assert!(s.files.len() >= 11, "{:?}", s.files);
        assert!(s.files.iter().all(|f| f.starts_with("tests/conformance/") && f.contains("/pending/") && f.ends_with(".panel")));
        assert!(s.files.iter().all(|f| !f.contains('\\')));
        assert!(s.problems.is_empty(), "{:?}", s.problems);
    }

    /// A scratch tree with one pending file in it.
    fn scratch(tag: &str, file: &str, text: &str) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!("xtask-panel-{}-{tag}", std::process::id()));
        let dir = root.join("tests/conformance/panel/pending");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(file), text).unwrap();
        root
    }

    #[test]
    fn the_scan_reports_a_pending_file_with_no_headers_by_path() {
        let root = scratch("bare", "naked.panel", "at 0 ms click step(1,1)\n");
        let s = scan_pending(&root);
        assert_eq!(s.files, ["tests/conformance/panel/pending/naked.panel"]);
        assert_eq!(s.problems.len(), 1, "{:?}", s.problems);
        assert_eq!(s.problems[0].0, "tests/conformance/panel/pending/naked.panel");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn the_scan_ignores_files_that_are_not_panel_fixtures_and_fixtures_that_are_not_pending() {
        let root = scratch("other", "notes.txt", "nothing to see\n");
        let live = root.join("tests/conformance/panel/edit");
        std::fs::create_dir_all(&live).unwrap();
        std::fs::write(live.join("bare.panel"), "at 0 ms click step(1,1)\n").unwrap();
        let s = scan_pending(&root);
        assert!(s.files.is_empty(), "{:?}", s.files);
        assert!(s.problems.is_empty(), "{:?}", s.problems);
        let _ = std::fs::remove_dir_all(&root);
    }
}
