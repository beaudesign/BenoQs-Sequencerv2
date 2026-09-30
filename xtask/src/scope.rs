//! The `scope` gate (ADR-0006, `specs/SPEC-0002/removal-plan.md` section 5).
//!
//! The live tree must not describe the product this repo used to be aimed at. The rule is a
//! test: a list of banned phrases (`harness/scope-banned.txt`), a list of places where history
//! keeps its words (`harness/scope-allow.txt`), and a scan of every file git tracks or would
//! track. A phrase outside the allow-list fails the gate.
//!
//! Matching, so that a test can pin it:
//! - case-insensitive;
//! - whole words: the characters on either side of a match must not be letters or digits.
//!   An underscore, a slash, a colon or a hyphen is a boundary, so `octo_x_room` style
//!   identifiers and paths are caught;
//! - words separated by any run of whitespace, including a line break, match a phrase written
//!   with single spaces. A phrase split by comment leaders (`///`) is not found;
//! - the path is scanned as well as the content, so an empty file cannot bring a name back.
//!
//! The banned list can grow freely. Removing a phrase from it is a loosening and needs an ADR:
//! `MIN_BANNED` is the floor the gate holds the list to.

use std::path::Path;
use std::process::{Command, Stdio};

/// How many phrases the list must hold. ADR-0006 and D9 of SPEC-0002 name seventeen. Adding a
/// phrase does not need this to change; dropping one does, and that needs an ADR.
pub const MIN_BANNED: usize = 17;

/// The scan must have looked at a real tree. A gate that scanned nothing must not pass.
pub const MIN_FILES_SCANNED: usize = 50;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hit {
    pub path: String,
    /// 1-based line where the phrase starts. 0 means the phrase is in the path.
    pub line: usize,
    pub phrase: String,
}

#[derive(Debug, Clone, Default)]
pub struct Scan {
    pub hits: Vec<Hit>,
    pub files_scanned: usize,
    pub files_allowed: usize,
    /// Not valid UTF-8, so not searched. Reported, never silent.
    pub files_not_text: usize,
    /// Listed by git but gone or a symlink (the target is scanned under its own path).
    pub files_skipped: usize,
    pub banned_listed: usize,
    /// Reasons the scan cannot be trusted, for example a missing list file.
    pub problems: Vec<String>,
    pub duration_s: f64,
}

impl Scan {
    /// A scan of a healthy tree that found nothing, for tests of the gate table.
    #[cfg(test)]
    pub fn clean() -> Scan {
        Scan { files_scanned: MIN_FILES_SCANNED, banned_listed: MIN_BANNED, ..Scan::default() }
    }
}

/// One entry per line. Blank lines and lines starting with `#` are ignored. Entries are
/// trimmed and repeated ones dropped. Phrases are lowercased, because matching ignores case.
pub fn parse_phrases(text: &str) -> Vec<String> {
    parse_lines(text, true)
}

/// The allow-list. Paths are case-sensitive (`specs/SPEC-0002/`), so they are not lowercased.
pub fn parse_allow(text: &str) -> Vec<String> {
    parse_lines(text, false)
}

fn parse_lines(text: &str, lowercase: bool) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let line = if lowercase { line.to_lowercase() } else { line.to_string() };
        if !out.contains(&line) {
            out.push(line);
        }
    }
    out
}

/// An entry ending in `/` allows a directory and everything below it. Any other entry allows
/// exactly that file.
pub fn is_allowed(path: &str, allow: &[String]) -> bool {
    allow.iter().any(|a| if a.ends_with('/') { path.starts_with(a.as_str()) } else { path == a })
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric()
}

/// Every banned phrase in `text`, with the 1-based line it starts on, in order of position.
pub fn find(text: &str, banned: &[String]) -> Vec<(usize, String)> {
    // Lowercase, and turn every run of whitespace into one space, remembering the line of
    // each byte so that a phrase split across a line break is reported once, where it starts.
    let mut norm = String::with_capacity(text.len());
    let mut line_of: Vec<usize> = Vec::with_capacity(text.len());
    let mut line = 1usize;
    let mut last_was_space = false;
    for ch in text.chars() {
        if ch.is_whitespace() {
            if !last_was_space {
                norm.push(' ');
                line_of.push(line);
                last_was_space = true;
            }
            if ch == '\n' {
                line += 1;
            }
            continue;
        }
        last_was_space = false;
        for lower in ch.to_lowercase() {
            norm.push(lower);
            for _ in 0..lower.len_utf8() {
                line_of.push(line);
            }
        }
    }

    let mut found: Vec<(usize, usize, String)> = Vec::new();
    for phrase in banned {
        if phrase.is_empty() {
            continue;
        }
        for (start, _) in norm.match_indices(phrase.as_str()) {
            let end = start + phrase.len();
            let before_ok = norm[..start].chars().next_back().is_none_or(|c| !is_word_char(c));
            let after_ok = norm[end..].chars().next().is_none_or(|c| !is_word_char(c));
            if before_ok && after_ok {
                found.push((start, line_of[start], phrase.clone()));
            }
        }
    }
    found.sort();
    found.into_iter().map(|(_, line, phrase)| (line, phrase)).collect()
}

/// Scan the given repo-relative paths. `read` returns the file's bytes, or `None` when the
/// path is gone or is a symlink.
pub fn scan_files(
    paths: &[String],
    allow: &[String],
    banned: &[String],
    read: &dyn Fn(&str) -> Option<Vec<u8>>,
) -> Scan {
    let mut scan = Scan { banned_listed: banned.len(), ..Scan::default() };
    for path in paths {
        if is_allowed(path, allow) {
            scan.files_allowed += 1;
            continue;
        }
        for (_, phrase) in find(path, banned) {
            scan.hits.push(Hit { path: path.clone(), line: 0, phrase });
        }
        let Some(bytes) = read(path) else {
            scan.files_skipped += 1;
            continue;
        };
        let Ok(text) = String::from_utf8(bytes) else {
            scan.files_not_text += 1;
            continue;
        };
        scan.files_scanned += 1;
        for (line, phrase) in find(&text, banned) {
            scan.hits.push(Hit { path: path.clone(), line, phrase });
        }
    }
    scan
}

/// Tracked files plus files git would track (not ignored), so a new file is scanned before
/// it is committed. `None` when git cannot list the tree.
fn list_files(root: &Path) -> Option<Vec<String>> {
    let out = Command::new("git")
        .args(["ls-files", "-z", "--cached", "--others", "--exclude-standard"])
        .current_dir(root)
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let mut paths: Vec<String> = out
        .stdout
        .split(|b| *b == 0)
        .filter(|s| !s.is_empty())
        .map(|s| String::from_utf8_lossy(s).into_owned())
        .collect();
    paths.sort();
    paths.dedup();
    Some(paths)
}

/// The scan the gate runs: the tree under `root`, the two list files in `harness/`.
pub fn scan_repo(root: &Path) -> Scan {
    let start = std::time::Instant::now();
    let mut problems = Vec::new();
    let mut read_list = |name: &str, parse: fn(&str) -> Vec<String>| -> Vec<String> {
        match std::fs::read_to_string(root.join("harness").join(name)) {
            Ok(text) => parse(&text),
            Err(e) => {
                problems.push(format!("harness/{name}: {e}"));
                Vec::new()
            }
        }
    };
    let banned = read_list("scope-banned.txt", parse_phrases);
    let allow = read_list("scope-allow.txt", parse_allow);

    let Some(paths) = list_files(root) else {
        let mut scan = Scan { banned_listed: banned.len(), problems, ..Scan::default() };
        scan.problems.push("git could not list the files of this checkout".to_string());
        scan.duration_s = start.elapsed().as_secs_f64();
        return scan;
    };
    let read = |p: &str| -> Option<Vec<u8>> {
        let full = root.join(p);
        let meta = std::fs::symlink_metadata(&full).ok()?;
        if meta.file_type().is_symlink() || !meta.is_file() {
            return None;
        }
        std::fs::read(full).ok()
    };
    let mut scan = scan_files(&paths, &allow, &banned, &read);
    scan.problems.extend(problems);
    scan.duration_s = start.elapsed().as_secs_f64();
    scan
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    // The mechanics are tested with made-up phrases, so that this file does not contain the
    // words the gate exists to keep out of the tree.
    fn banned() -> Vec<String> {
        parse_phrases("flux capacitor\nwarp\n")
    }

    fn scan(files: &[(&str, &str)], allow: &[&str]) -> Scan {
        let map: BTreeMap<String, Vec<u8>> =
            files.iter().map(|(p, c)| (p.to_string(), c.as_bytes().to_vec())).collect();
        let paths: Vec<String> = map.keys().cloned().collect();
        let allow: Vec<String> = allow.iter().map(|s| s.to_string()).collect();
        scan_files(&paths, &allow, &banned(), &|p| map.get(p).cloned())
    }

    #[test]
    fn a_file_containing_a_banned_phrase_fails() {
        let s = scan(&[("docs/a.md", "line one\nthe Flux Capacitor is here\n")], &[]);
        assert_eq!(s.hits, [Hit { path: "docs/a.md".into(), line: 2, phrase: "flux capacitor".into() }]);
    }

    #[test]
    fn the_same_file_under_an_allow_listed_prefix_passes() {
        let s = scan(&[("journal/a.md", "the flux capacitor")], &["journal/"]);
        assert!(s.hits.is_empty());
        assert_eq!(s.files_allowed, 1);
        assert_eq!(s.files_scanned, 0);
    }

    #[test]
    fn a_prefix_entry_allows_the_directory_and_a_file_entry_allows_only_that_file() {
        let allow = parse_allow("journal/\nharness/baseline.txt\n");
        assert!(is_allowed("journal/x/y.md", &allow));
        assert!(!is_allowed("journals/x.md", &allow), "a prefix must end at a slash");
        assert!(is_allowed("harness/baseline.txt", &allow));
        assert!(!is_allowed("harness/baseline.txt.bak", &allow), "a file entry is exact");
    }

    #[test]
    fn a_phrase_split_across_a_line_break_is_reported_once_at_the_line_it_starts() {
        let s = scan(&[("a.md", "x\nthe flux\ncapacitor works\n")], &[]);
        assert_eq!(s.hits.len(), 1);
        assert_eq!(s.hits[0].line, 2);
        let s = scan(&[("a.md", "flux \r\n\t capacitor")], &[]);
        assert_eq!(s.hits.len(), 1, "any run of whitespace joins the words");
    }

    #[test]
    fn whole_words_only() {
        let s = scan(&[("a.md", "warpath warped swarp\nwarp,\n(warp)\nre-warp\nwarp_core\n")], &[]);
        let lines: Vec<usize> = s.hits.iter().map(|h| h.line).collect();
        // Not `warpath`, `warped` or `swarp`. Yes: a comma, brackets, a hyphen and an underscore
        // are boundaries.
        assert_eq!(lines, [2, 3, 4, 5]);
    }

    #[test]
    fn matching_ignores_case_and_repeats_count_each_time() {
        let s = scan(&[("a.md", "WARP warp Warp")], &[]);
        assert_eq!(s.hits.len(), 3);
    }

    #[test]
    fn a_banned_word_in_a_path_fails_even_when_the_file_is_empty() {
        let s = scan(&[("crates/warp/Cargo.toml", "")], &[]);
        assert_eq!(s.hits, [Hit { path: "crates/warp/Cargo.toml".into(), line: 0, phrase: "warp".into() }]);
    }

    #[test]
    fn a_file_that_is_not_text_is_counted_and_not_silently_passed() {
        let map: BTreeMap<String, Vec<u8>> = [("plate.jpg".to_string(), vec![0xff, 0xd8, 0xff, 0xe0])].into();
        let s = scan_files(&["plate.jpg".to_string()], &[], &banned(), &|p| map.get(p).cloned());
        assert_eq!(s.files_not_text, 1);
        assert_eq!(s.files_scanned, 0);
    }

    #[test]
    fn a_missing_file_or_a_link_is_counted_as_skipped() {
        let s = scan_files(&["gone.md".to_string()], &[], &banned(), &|_| None);
        assert_eq!(s.files_skipped, 1);
    }

    #[test]
    fn the_allow_list_keeps_the_case_of_a_path() {
        let allow = parse_allow("specs/SPEC-0001/\n");
        assert_eq!(allow, ["specs/SPEC-0001/"]);
        assert!(is_allowed("specs/SPEC-0001/README.md", &allow));
        assert!(!is_allowed("specs/spec-0001/README.md", &allow));
    }

    #[test]
    fn the_list_parser_drops_comments_blanks_and_repeats_and_lowercases() {
        let l = parse_phrases("# a comment\n\n  Flux Capacitor  \nflux capacitor\nWarp\n");
        assert_eq!(l, ["flux capacitor", "warp"]);
    }

    #[test]
    fn the_shared_project_state_name_and_the_concurrency_crate_are_not_banned() {
        // Two things the owner's word "world model" could have swept up by mistake. Read the real
        // list and check that neither is in it and neither would be matched.
        let root = crate::root();
        let banned = parse_phrases(&std::fs::read_to_string(root.join("harness/scope-banned.txt")).unwrap());
        assert!(find("the shared world model is journal/STATE.md", &banned).is_empty());
        assert!(find("crates/octocore/loom/link_mutants.py and tests/loom_sync.rs", &banned).is_empty());
    }

    #[test]
    fn the_real_lists_meet_the_floor_and_the_allow_list_covers_its_own_files() {
        let root = crate::root();
        let banned = parse_phrases(&std::fs::read_to_string(root.join("harness/scope-banned.txt")).unwrap());
        let allow = parse_allow(&std::fs::read_to_string(root.join("harness/scope-allow.txt")).unwrap());
        assert!(banned.len() >= MIN_BANNED, "the banned list has {} entries, the floor is {MIN_BANNED}", banned.len());
        assert!(is_allowed("harness/scope-banned.txt", &allow), "the banned list names the phrases, so it must be allowed");
        assert!(is_allowed("harness/scope-allow.txt", &allow));
        assert!(is_allowed("adr/0006-web-sequencer-supersedes-the-rooms-scope.md", &allow));
        // Paths are case-sensitive. The first red run of this gate flagged 51 hits under
        // specs/SPEC-0001/ and specs/SPEC-0002/ because the allow-list had been lowercased.
        assert!(is_allowed("specs/SPEC-0002/README.md", &allow));
        assert!(is_allowed("specs/SPEC-0001/findings.md", &allow));
    }
}
