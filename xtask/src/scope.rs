//! The `scope` gate (ADR-0006, `specs/SPEC-0002/removal-plan.md` section 5).
//!
//! The live tree must not describe the product this repo used to be aimed at. The rule is a
//! test: a list of banned phrases (`harness/scope-banned.txt`), a list of places where history
//! keeps its words (`harness/scope-allow.txt`), and a scan of every file git tracks or would
//! track. A phrase outside the allow-list fails the gate.
//!
//! Matching, so that a test can pin it:
//! - case-insensitive;
//! - every run of characters that are not letters or digits (spaces, line breaks, hyphens,
//!   underscores, slashes, dots, comment marks) is one separator. So `flux_capacitor`,
//!   `flux-capacitor`, a slug and a phrase split over two comment lines are all the phrase, and a
//!   phrase in the list is written with any separators and normalised the same way;
//! - whole words: a match must start and end at a separator or at the edge of the text. The
//!   last word may carry a plural `s`, so a banned phrase and its plural are one phrase;
//! - the path is scanned as well as the content, so an empty file cannot bring a name back.
//!
//! What it does not do: it matches phrases, not ideas, and a name with no separator at all
//! (`FluxCapacitor`) is one word and gets through. Review still applies.
//!
//! Failures that are not hits: a file that is not UTF-8 text and does not have a known binary
//! extension (a Latin-1 or UTF-16 paste would otherwise pass unread); a tracked symlink whose
//! target is allow-listed or outside the tree; a missing list file.
//!
//! The banned list can grow freely. Removing or changing a phrase is a loosening and needs an
//! ADR: `MIN_BANNED` is a floor on the count, and a test pins each phrase of D9 by fingerprint.
//! The allow-list is pinned by a test too.

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
    /// Not text and not searched: a known binary type (a picture, a font). Counted, never silent.
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

fn parse_lines(text: &str, phrases: bool) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let line = if phrases { normalise(line).0 } else { line.to_string() };
        if line.is_empty() {
            continue;
        }
        if !out.contains(&line) {
            out.push(line);
        }
    }
    out
}

/// Lowercase, and every run of characters that are not letters or digits becomes one space, with
/// none at either end. Also returns the 1-based line of each byte of the result, so that a hit
/// is reported at the line where its first word starts.
fn normalise(text: &str) -> (String, Vec<usize>) {
    let mut norm = String::with_capacity(text.len());
    let mut line_of: Vec<usize> = Vec::with_capacity(text.len());
    let mut line = 1usize;
    let mut pending_space = false;
    for ch in text.chars() {
        if !is_word_char(ch) {
            if ch == '\n' {
                line += 1;
            }
            // A separator is written only once a word follows, so there is none at the ends.
            pending_space = !norm.is_empty();
            continue;
        }
        if pending_space {
            norm.push(' ');
            line_of.push(line);
            pending_space = false;
        }
        for lower in ch.to_lowercase() {
            norm.push(lower);
            for _ in 0..lower.len_utf8() {
                line_of.push(line);
            }
        }
    }
    (norm, line_of)
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
    let (norm, line_of) = normalise(text);

    let mut found: Vec<(usize, usize, String)> = Vec::new();
    for phrase in banned {
        if phrase.is_empty() {
            continue;
        }
        for (start, _) in norm.match_indices(phrase.as_str()) {
            let end = start + phrase.len();
            let before_ok = norm[..start].chars().next_back().is_none_or(|c| !is_word_char(c));
            let boundary_at = |at: usize| norm[at..].chars().next().is_none_or(|c| !is_word_char(c));
            let after_ok = boundary_at(end) || (norm[end..].starts_with('s') && boundary_at(end + 1));
            if before_ok && after_ok {
                found.push((start, line_of[start], phrase.clone()));
            }
        }
    }
    found.sort();
    found.into_iter().map(|(_, line, phrase)| (line, phrase)).collect()
}

/// File types that are not text. A file with any other extension that is not UTF-8 fails the
/// scan. The list only grows with the Referee's sign-off; it exists so that a picture or a font
/// in `apps/web` does not fail the gate, and so that a `.md` in the wrong encoding does.
const BINARY_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "webp", "avif", "ico", "pdf", "woff", "woff2", "ttf", "otf", "wasm",
    "zip", "gz", "mid", "midi", "mp3", "wav", "ogg", "mp4", "webm",
];

fn has_binary_extension(path: &str) -> bool {
    match path.rsplit_once('.') {
        Some((_, ext)) => BINARY_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()),
        None => false,
    }
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
        let text = match String::from_utf8(bytes) {
            Ok(text) => text,
            Err(_) if has_binary_extension(path) => {
                scan.files_not_text += 1;
                continue;
            }
            Err(_) => {
                // Text in another encoding (a Latin-1 or UTF-16 paste) would pass unread.
                scan.problems.push(format!(
                    "{path}: not valid UTF-8 text and not a known binary type, so it cannot be searched. \
                     Convert it to UTF-8 (or, for a new binary type, add its extension to BINARY_EXTENSIONS in xtask/src/scope.rs, which needs the Referee)"
                ));
                continue;
            }
        };
        scan.files_scanned += 1;
        for (line, phrase) in find(&text, banned) {
            scan.hits.push(Hit { path: path.clone(), line, phrase });
        }
    }
    scan
}

/// Where a link at `path` (repo-relative) points, as a repo-relative path with `.` and `..`
/// resolved, or `None` when the target is absolute or leaves the tree.
fn resolve_link(path: &str, target: &str) -> Option<String> {
    if target.starts_with('/') {
        return None;
    }
    let mut parts: Vec<&str> = path.split('/').collect();
    parts.pop(); // the link's own name: the target is relative to its directory
    for comp in target.split('/') {
        match comp {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            c => parts.push(c),
        }
    }
    Some(parts.join("/"))
}

/// Why a tracked symlink is a problem, or `None`. A link is scanned as nothing, and its target is
/// scanned under its own path, which is only enough when that path is not on the allow-list.
/// Otherwise a link from a live directory to history would let a banned phrase into the live
/// tree without being read.
pub fn link_problem(path: &str, target: &str, allow: &[String]) -> Option<String> {
    if is_allowed(path, allow) {
        return None; // the link is history itself
    }
    let Some(resolved) = resolve_link(path, target) else {
        return Some(format!("{path}: a symlink to '{target}', which is absolute or outside the tree, so it cannot be searched"));
    };
    if is_allowed(&resolved, allow) || is_allowed(&format!("{resolved}/"), allow) {
        return Some(format!(
            "{path}: a symlink to '{resolved}', which is on the allow-list, so its content would sit in the live tree unread"
        ));
    }
    None
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
    for p in &paths {
        if let Ok(target) = std::fs::read_link(root.join(p)) {
            if let Some(why) = link_problem(p, &target.to_string_lossy(), &allow) {
                scan.problems.push(why);
            }
        }
    }
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
    fn the_plural_of_the_last_word_is_the_same_phrase() {
        // The constitution (`CLAUDE.md`, line 4) held a banned phrase in the plural. The first version of the gate matched only the
        // singular and let the line through.
        let s = scan(&[("a.md", "flux capacitors\nwarps\nwarpss\nflux capacitorsx\n")], &[]);
        let lines: Vec<usize> = s.hits.iter().map(|h| h.line).collect();
        assert_eq!(lines, [1, 2], "a plural counts, a doubled s or a longer word does not");
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
        let jpeg = vec![0xff, 0xd8, 0xff, 0xe0, 0x00, 0x10, b'J', b'F', b'I', b'F', 0x00];
        let map: BTreeMap<String, Vec<u8>> = [("plate.jpg".to_string(), jpeg)].into();
        let s = scan_files(&["plate.jpg".to_string()], &[], &banned(), &|p| map.get(p).cloned());
        assert_eq!(s.files_not_text, 1);
        assert_eq!(s.files_scanned, 0);
        assert!(s.problems.is_empty(), "a picture is not a problem: {:?}", s.problems);
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
    #[test]
    fn punctuation_between_the_words_of_a_phrase_still_matches() {
        // The review of P1 found `flux_capacitor`, `flux-capacitor` and a slug got through.
        let s = scan(
            &[("a.md", "flux_capacitor\nflux-capacitor\nflux.capacitor\nflux/capacitor\nfluxcapacitor\n")],
            &[],
        );
        let lines: Vec<usize> = s.hits.iter().map(|h| h.line).collect();
        assert_eq!(lines, [1, 2, 3, 4], "no separator at all is one word, not the phrase");
        let s = scan(&[("a.md", "pub fn x() {\n    // flux\n    /// capacitor\n}")], &[]);
        assert_eq!(s.hits.len(), 1, "comment marks between the words do not hide a phrase");
        let s = scan(&[("docs/flux-capacitor.md", "")], &[]);
        assert_eq!(s.hits.len(), 1, "a slug in a path is caught");
    }

    #[test]
    fn a_phrase_from_the_list_is_normalised_like_the_text() {
        // Two spaces, a hyphen and capitals in the list must not make an entry that matches nothing.
        let l = parse_phrases("Flux  Capacitor\nflux-capacitor\n--\nwarp\n");
        assert_eq!(l, ["flux capacitor", "warp"]);
    }

    #[test]
    fn a_file_in_the_wrong_encoding_fails_the_scan_and_a_binary_file_does_not() {
        let mut map: BTreeMap<String, Vec<u8>> = BTreeMap::new();
        // Latin-1: a stray 0xE9. UTF-16 with a byte order mark: what a Windows shell writes.
        map.insert("docs/latin1.md".into(), b"the flux capacitor caf\xe9".to_vec());
        let mut utf16 = vec![0xff, 0xfe];
        for c in "flux capacitor".encode_utf16() {
            utf16.extend_from_slice(&c.to_le_bytes());
        }
        map.insert("docs/utf16.md".into(), utf16);
        map.insert("web/font.woff2".into(), vec![0x77, 0x4f, 0x46, 0x32, 0x00, 0xff]);
        let paths: Vec<String> = map.keys().cloned().collect();
        let s = scan_files(&paths, &[], &banned(), &|p| map.get(p).cloned());
        assert_eq!(s.problems.len(), 2, "{:?}", s.problems);
        assert!(s.problems.iter().any(|p| p.contains("docs/latin1.md")));
        assert!(s.problems.iter().any(|p| p.contains("docs/utf16.md")));
        assert_eq!(s.files_not_text, 1, "the font is a known binary type");
        // On the allow-list, a file in any encoding is history and is not read.
        let s = scan_files(&paths, &["docs/".to_string()], &banned(), &|p| map.get(p).cloned());
        assert!(s.problems.is_empty());
    }

    #[test]
    fn a_link_to_allow_listed_content_from_outside_the_allow_list_is_a_problem() {
        let allow = parse_allow("journal/\nadr/\n");
        // The only link in the tree today: CLAUDE.md to agents/CLAUDE.md. Fine.
        assert_eq!(link_problem("CLAUDE.md", "agents/CLAUDE.md", &allow), None);
        assert_eq!(link_problem("docs/x.md", "../journal/a.md", &allow).is_some(), true);
        assert_eq!(link_problem("docs/x.md", "../adr", &allow).is_some(), true, "a directory link too");
        assert_eq!(link_problem("docs/x.md", "../../outside.md", &allow).is_some(), true, "out of the tree");
        assert_eq!(link_problem("docs/x.md", "/etc/passwd", &allow).is_some(), true, "absolute");
        // A link that already sits in history is history.
        assert_eq!(link_problem("journal/x.md", "a.md", &allow), None);
    }

    #[test]
    fn the_real_banned_list_holds_the_seventeen_of_d9() {
        // The count alone could be met by swapping a phrase for nonsense (the review of P1 did
        // exactly that). Each of the seventeen is pinned by a fingerprint of its normal form, so
        // this file does not spell the words the gate keeps out. Adding phrases needs nothing
        // here; dropping or changing one fails this test, and that needs an ADR (CLAUDE.md rule 3).
        fn fnv(s: &str) -> u64 {
            let mut h: u64 = 0xcbf29ce484222325;
            for b in s.bytes() {
                h ^= b as u64;
                h = h.wrapping_mul(0x100000001b3);
            }
            h
        }
        const D9: [u64; 17] = [
            0x4c70ed641534a375, 0x7ddd558af1749b90, 0x4bb13e8f6fac7e8d, 0x4c41199a73712503,
            0x15251ecc66245ad8, 0xc147dbe881147cfb, 0x757a178effb00029, 0x5bb5dec2f4ab98e1,
            0x7f50f31f5f344ce4, 0x8cfd4eb82c16bd4d, 0x2f88f4de8bf93883, 0x6806b52ffe1dbb7f,
            0x1f7639805f9a4e4c, 0xe2f4638d85b24def, 0x0c7d882148c56ce9, 0xc9eb501f7561419d,
            0xaec40dc12894e10a,
        ];
        let root = crate::root();
        let banned = parse_phrases(&std::fs::read_to_string(root.join("harness/scope-banned.txt")).unwrap());
        let have: Vec<u64> = banned.iter().map(|p| fnv(p)).collect();
        for want in D9 {
            assert!(have.contains(&want), "a phrase of D9 (fingerprint {want:#018x}) is missing from harness/scope-banned.txt");
        }
    }

    #[test]
    fn the_real_allow_list_is_the_pinned_set() {
        // Widening the allow-list widens the gate. It is pinned here so that it cannot happen
        // in one line of a text file: the change shows in this test, and needs the Referee.
        let root = crate::root();
        let allow = parse_allow(&std::fs::read_to_string(root.join("harness/scope-allow.txt")).unwrap());
        assert_eq!(
            allow,
            [
                "adr/", "handoffs/", "journal/", "specs/SPEC-0001/", "specs/SPEC-0002/", "archive/",
                "reference/", "harness/baseline.txt", "harness/scope-banned.txt", "harness/scope-allow.txt",
            ]
        );
    }
}
