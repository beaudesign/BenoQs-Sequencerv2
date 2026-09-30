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

/// What the scan of `tests/conformance` found among the `.panel` files that do not assert.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PanelScan {
    /// Every pending panel fixture, repo-relative with forward slashes.
    pub pending: Vec<String>,
    /// `(path, what is wrong)`: a pending fixture without its headers, or a `.panel` file where nothing runs it.
    pub problems: Vec<(String, String)>,
}

/// The values of every `# <key>: <value>` line, trimmed.
fn headers<'a>(text: &'a str, key: &str) -> Vec<&'a str> {
    text.lines()
        .filter_map(|l| l.trim().strip_prefix("# ")?.strip_prefix(key)?.strip_prefix(':'))
        .map(str::trim)
        .collect()
}

fn is_page(word: &str) -> bool {
    word.len() == 4 && word.starts_with('p') && word[1..].bytes().all(|b| b.is_ascii_digit())
}

fn is_question(word: &str) -> bool {
    word.len() == 3 && word.starts_with('Q') && word[1..].bytes().all(|b| b.is_ascii_digit())
}

/// What is wrong with the headers of a pending fixture. Empty when it is well formed.
///
/// One message per rule, each naming the header it is about: a `# pending:` sentence, a
/// `# manual:` list of pages (`p013, p014`), exactly one `# question:` (`Q31`), and no script
/// lines, because a pending fixture asserts nothing.
pub fn header_problems(text: &str) -> Vec<String> {
    let mut out = Vec::new();

    let pending = headers(text, "pending");
    if pending.is_empty() || pending.iter().any(|v| v.is_empty()) {
        out.push("no `# pending:` line with a sentence saying what the manual leaves open".to_string());
    }

    let manual = headers(text, "manual");
    let pages: Vec<&str> = manual.iter().flat_map(|v| v.split(|c: char| c == ',' || c.is_whitespace())).filter(|w| !w.is_empty()).collect();
    if pages.is_empty() {
        out.push("no `# manual:` line with page numbers such as `p013, p014`".to_string());
    } else if let Some(bad) = pages.iter().find(|w| !is_page(w)) {
        out.push(format!("`# manual:` holds '{bad}', which is not a page number such as `p013`"));
    }

    let question = headers(text, "question");
    match question.as_slice() {
        [q] if is_question(q) => {}
        [] => out.push("no `# question:` line with a number such as `Q31`".to_string()),
        other => out.push(format!("`# question:` must be exactly one number such as `Q31`, found {other:?}")),
    }

    if text.lines().any(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#')) {
        out.push("has script lines: a pending fixture asserts nothing, so its body is comments only".to_string());
    }
    out
}

/// Read every pending panel fixture under `<root>/tests/conformance`: a `.panel` file at any depth
/// below a directory named `pending`. Paths are repo-relative with forward slashes, sorted.
pub fn scan(root: &Path) -> PanelScan {
    fn walk(dir: &Path, in_pending: bool, root: &Path, out: &mut PanelScan) {
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        let mut entries: Vec<_> = rd.flatten().map(|e| e.path()).collect();
        entries.sort();
        for path in entries {
            if path.is_dir() {
                let pending = in_pending || path.file_name().is_some_and(|n| n == "pending");
                walk(&path, pending, root, out);
            } else if in_pending && path.extension().is_some_and(|e| e == "panel") {
                let Ok(rel) = path.strip_prefix(root) else { continue };
                let rel = rel.to_string_lossy().replace('\\', "/");
                match std::fs::read_to_string(&path) {
                    Ok(text) => {
                        for p in header_problems(&text) {
                            out.problems.push((rel.clone(), p));
                        }
                    }
                    Err(e) => out.problems.push((rel.clone(), format!("unreadable: {e}"))),
                }
                out.pending.push(rel);
            }
        }
    }
    let mut out = PanelScan::default();
    walk(&root.join("tests/conformance"), false, root, &mut out);
    out
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
        let s = scan(&repo());
        // No count here: closing the last open question empties the list, and that is progress.
        assert!(s.pending.iter().all(|f| f.starts_with("tests/conformance/") && f.contains("/pending/") && f.ends_with(".panel")));
        assert!(s.pending.iter().all(|f| !f.contains('\\')));
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
        let s = scan(&root);
        assert_eq!(s.pending, ["tests/conformance/panel/pending/naked.panel"]);
        // One message per rule: no pending sentence, no pages, no question, and a script line.
        assert_eq!(s.problems.len(), 4, "{:?}", s.problems);
        assert!(s.problems.iter().all(|p| p.0 == "tests/conformance/panel/pending/naked.panel"));
        for key in ["pending", "manual", "question", "script"] {
            assert!(s.problems.iter().any(|p| p.1.contains(key)), "{key}: {:?}", s.problems);
        }
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn the_scan_ignores_files_that_are_not_panel_fixtures_and_fixtures_that_are_not_pending() {
        let root = scratch("other", "notes.txt", "nothing to see\n");
        let live = root.join("tests/conformance/panel/edit");
        std::fs::create_dir_all(&live).unwrap();
        std::fs::write(live.join("bare.panel"), "at 0 ms click step(1,1)\n").unwrap();
        let s = scan(&root);
        assert!(s.pending.is_empty(), "{:?}", s.pending);
        assert!(s.problems.is_empty(), "{:?}", s.problems);
        let _ = std::fs::remove_dir_all(&root);
    }
    #[test]
    fn the_manual_header_accepts_a_tutorial_page() {
        assert_eq!(header_problems(&GOOD.replace("# manual: p013", "# manual: p013, T05")), Vec::<String>::new());
        let p = header_problems(&GOOD.replace("# manual: p013", "# manual: T5"));
        assert_eq!(p.len(), 1, "{p:?}");
    }

    /// A scratch tree with one `.panel` file at `rel` (relative to the root).
    fn scratch_at(tag: &str, rel: &str, text: &str) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!("xtask-panel-{}-{tag}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let file = root.join(rel);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, text).unwrap();
        root
    }

    #[test]
    fn a_panel_file_outside_the_panel_directory_is_a_problem_because_nothing_runs_it() {
        let root = scratch_at("stray", "tests/conformance/elsewhere/x.panel", "at 0 ms click step(1,1)\nexpect mode step\n");
        let s = scan(&root);
        assert!(s.pending.is_empty(), "{:?}", s.pending);
        assert_eq!(s.problems.len(), 1, "{:?}", s.problems);
        assert_eq!(s.problems[0].0, "tests/conformance/elsewhere/x.panel");
        assert!(s.problems[0].1.contains("tests/conformance/panel"), "{:?}", s.problems);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_panel_file_in_the_engine_pending_directory_is_a_problem_too() {
        let root = scratch_at("enginepending", "tests/conformance/pending/x.panel", &format!("{GOOD}"));
        let s = scan(&root);
        assert!(s.pending.is_empty(), "{:?}", s.pending);
        assert_eq!(s.problems.len(), 1, "{:?}", s.problems);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_panel_file_in_the_panel_directory_is_not_a_problem_when_it_asserts() {
        let root = scratch_at("live", "tests/conformance/panel/edit/x.panel", "at 0 ms click step(1,1)\nexpect mode step\n");
        let s = scan(&root);
        assert!(s.pending.is_empty() && s.problems.is_empty(), "{s:?}");
        let _ = std::fs::remove_dir_all(&root);
    }
}
