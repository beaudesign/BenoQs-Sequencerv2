//! Rules about the fixture files themselves (ADR-0007 decision 4): every fixture is cited to a
//! manual page; a pending fixture carries the ambiguity, the pages and a numbered question, and
//! asserts nothing; an asserting fixture asserts something.

mod support;

use std::path::Path;
use support::{panel_files, repo_root};

fn header<'a>(text: &'a str, key: &str) -> Vec<&'a str> {
    text.lines().filter_map(|l| l.trim().strip_prefix("# ").and_then(|l| l.strip_prefix(key)).and_then(|l| l.strip_prefix(':'))).map(str::trim).collect()
}

fn body_lines(text: &str) -> Vec<&str> {
    text.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')).collect()
}

fn name(p: &Path) -> String {
    p.strip_prefix(repo_root().join("tests/conformance/panel")).unwrap_or(p).display().to_string()
}

fn is_page(word: &str) -> bool {
    let w = word.trim_matches(|c: char| c == ',' || c == ' ');
    (w.len() == 4 && w.starts_with('p') && w[1..].chars().all(|c| c.is_ascii_digit()))
        || (w.len() == 3 && w.starts_with('T') && w[1..].chars().all(|c| c.is_ascii_digit()))
}

fn page_exists(word: &str) -> bool {
    let w = word.trim();
    let file = match w.strip_prefix('T') {
        Some(n) => format!("tutorial-{n}.txt"),
        None => format!("{w}.txt"),
    };
    repo_root().join("reference/manual/pages").join(file).exists()
}

/// Q01 to Q20 are findings.md section 6. Q21 onward must be in QUESTIONS.md.
pub fn question_exists(q: &str) -> bool {
    let Some(n) = q.strip_prefix('Q').and_then(|d| d.parse::<u32>().ok()) else { return false };
    if q.len() != 3 {
        return false;
    }
    if (1..=20).contains(&n) {
        return true;
    }
    let text = std::fs::read_to_string(repo_root().join("tests/conformance/panel/QUESTIONS.md")).expect("QUESTIONS.md");
    text.lines().any(|l| l.starts_with(&format!("| {q} |")))
}

#[test]
fn every_panel_fixture_is_cited_to_a_manual_page() {
    let (fixtures, pending) = panel_files();
    let mut bad = Vec::new();
    for p in fixtures.iter().chain(pending.iter()) {
        let text = std::fs::read_to_string(p).unwrap();
        let manual = header(&text, "manual");
        let ok = manual.len() == 1 && manual[0].split(',').all(|w| is_page(w) && page_exists(w));
        if !ok || header(&text, "panel fixture").len() != 1 || header(&text, "workflow").len() != 1 {
            bad.push(name(p));
        }
    }
    assert!(bad.is_empty(), "these need one `# panel fixture:`, one `# workflow:` and one `# manual:` line of real page numbers: {bad:?}");
}

#[test]
fn a_pending_fixture_names_its_ambiguity_and_question_and_asserts_nothing() {
    // No assertion that the list is non-empty: closing the last open question empties it, and the
    // walk is shown to work by the asserting fixtures below and by the runner.
    let (_, pending) = panel_files();
    let mut bad = Vec::new();
    for p in &pending {
        let text = std::fs::read_to_string(p).unwrap();
        let why = header(&text, "pending");
        let q = header(&text, "question");
        let ok = why.len() == 1 && why[0].len() > 20 && q.len() == 1 && question_exists(q[0]) && body_lines(&text).is_empty();
        if !ok {
            bad.push(name(p));
        }
    }
    assert!(bad.is_empty(), "a pending fixture needs `# pending:` (a sentence), `# question:` (a Q number that exists) and no script lines: {bad:?}");
}

#[test]
fn an_asserting_fixture_asserts_something() {
    let (fixtures, _) = panel_files();
    let bad: Vec<String> = fixtures
        .iter()
        .filter(|p| !body_lines(&std::fs::read_to_string(p).unwrap()).iter().any(|l| l.starts_with("expect ")))
        .map(|p| name(p))
        .collect();
    assert!(bad.is_empty(), "no `expect` line in: {bad:?}");
}

#[test]
fn a_fixture_name_is_not_used_twice() {
    let (fixtures, pending) = panel_files();
    let mut seen = std::collections::BTreeSet::new();
    let mut dup = Vec::new();
    for p in fixtures.iter().chain(pending.iter()) {
        let stem = p.file_stem().unwrap().to_string_lossy().to_string();
        if !seen.insert(stem) {
            dup.push(name(p));
        }
    }
    assert!(dup.is_empty(), "file names must be unique across directories: {dup:?}");
}
