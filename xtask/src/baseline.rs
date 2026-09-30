//! The ratchet floor, made literal (docs/07 `verify:regressions`).
//!
//! `harness/baseline.txt` lists every test and fixture that must keep passing. Two
//! checks use it:
//!
//! 1. The current run must still pass everything listed. A deleted, renamed, ignored or
//!    failing test is a violation even if another test was added elsewhere.
//! 2. Against a base ref (`--base`), no entry may leave the file unless a `removed` line
//!    names an ADR that exists. This closes the hole of deleting a test and its baseline
//!    line in the same change.
//!
//! The format is plain text so a diff of it is readable in a pull request.

use std::collections::BTreeSet;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Baseline {
    pub tests: BTreeSet<String>,
    pub fixtures: BTreeSet<String>,
    /// `(kind, id, adr)` for every deliberate removal, kept forever.
    pub removed: BTreeSet<(String, String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    pub subject: String,
    pub detail: String,
}

impl Violation {
    fn new(subject: &str, detail: &str) -> Violation {
        Violation { subject: subject.to_string(), detail: detail.to_string() }
    }
}

const HEADER: &str = "\
# harness/baseline.txt: the assertion ratchet floor (docs/07 verify:regressions).
#
# Every `test` and `fixture` line must keep passing. Lines are only added by
# `cargo xtask baseline`. A line may leave this file only through
# `cargo xtask baseline --remove <kind> <id> --adr ADR-NNNN`, which records a `removed`
# line that names an ADR. Owned by the Referee. Do not edit by hand.
schema baseline/1
";

impl Baseline {
    pub fn parse(text: &str) -> Result<Baseline, String> {
        let mut b = Baseline::default();
        let mut saw_schema = false;
        for (n, raw) in text.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let lineno = n + 1;
            if let Some(v) = line.strip_prefix("schema ") {
                if v.trim() != "baseline/1" {
                    return Err(format!("line {lineno}: unsupported schema '{}'", v.trim()));
                }
                saw_schema = true;
            } else if let Some(id) = line.strip_prefix("test ") {
                b.tests.insert(id.trim().to_string());
            } else if let Some(id) = line.strip_prefix("fixture ") {
                b.fixtures.insert(id.trim().to_string());
            } else if let Some(rest) = line.strip_prefix("removed ") {
                let parts: Vec<&str> = rest.split_whitespace().collect();
                if parts.len() != 3 || !matches!(parts[0], "test" | "fixture") {
                    return Err(format!("line {lineno}: expected 'removed <test|fixture> <id> <ADR-NNNN>'"));
                }
                b.removed.insert((parts[0].to_string(), parts[1].to_string(), parts[2].to_string()));
            } else {
                return Err(format!("line {lineno}: unrecognised line '{line}'"));
            }
        }
        if !saw_schema {
            return Err("missing 'schema baseline/1' line".to_string());
        }
        Ok(b)
    }

    pub fn render(&self) -> String {
        let mut out = String::from(HEADER);
        for t in &self.tests {
            out.push_str(&format!("test {t}\n"));
        }
        for f in &self.fixtures {
            out.push_str(&format!("fixture {f}\n"));
        }
        for (kind, id, adr) in &self.removed {
            out.push_str(&format!("removed {kind} {id} {adr}\n"));
        }
        out
    }

    pub fn total(&self) -> usize {
        self.tests.len() + self.fixtures.len()
    }

    fn is_removed(&self, kind: &str, id: &str) -> bool {
        self.removed.iter().any(|(k, i, _)| k == kind && i == id)
    }

    /// Everything the baseline requires that the current run does not deliver.
    pub fn check_run(&self, passing: &BTreeSet<String>, fixtures: &BTreeSet<String>) -> Vec<Violation> {
        let mut out = Vec::new();
        for t in &self.tests {
            if !passing.contains(t) && !self.is_removed("test", t) {
                out.push(Violation::new(t, "in the baseline but not passing in this run (deleted, renamed, ignored or failing)"));
            }
        }
        for f in &self.fixtures {
            if !fixtures.contains(f) && !self.is_removed("fixture", f) {
                out.push(Violation::new(f, "in the baseline but no longer present as a conformance fixture"));
            }
        }
        out
    }

    /// Entries that were in `base` and are gone here without a removal record that names
    /// an ADR that exists. Also flags deleted removal records.
    pub fn check_against_base(&self, base: &Baseline, adr_exists: &dyn Fn(&str) -> bool) -> Vec<Violation> {
        fn check(now: &Baseline, kind: &str, id: &str, adr_exists: &dyn Fn(&str) -> bool, out: &mut Vec<Violation>) {
            match now.removed.iter().find(|(k, i, _)| k == kind && i == id) {
                None => out.push(Violation::new(id, &format!("{kind} left the baseline with no 'removed' record"))),
                Some((_, _, adr)) if !adr_exists(adr) => {
                    out.push(Violation::new(id, &format!("{kind} removal cites {adr}, which does not exist in adr/")))
                }
                Some(_) => {}
            }
        }
        let mut out = Vec::new();
        for t in base.tests.iter().filter(|t| !self.tests.contains(*t)) {
            check(self, "test", t, adr_exists, &mut out);
        }
        for f in base.fixtures.iter().filter(|f| !self.fixtures.contains(*f)) {
            check(self, "fixture", f, adr_exists, &mut out);
        }
        for r in &base.removed {
            if !self.removed.contains(r) {
                out.push(Violation::new(&r.1, "a removal record was deleted from the baseline"));
            }
        }
        out
    }

    /// Entries in the run that the baseline does not know yet.
    pub fn added(&self, passing: &BTreeSet<String>, fixtures: &BTreeSet<String>) -> Vec<String> {
        let mut v: Vec<String> = passing.difference(&self.tests).cloned().collect();
        v.extend(fixtures.difference(&self.fixtures).cloned());
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(items: &[&str]) -> BTreeSet<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    fn base() -> Baseline {
        Baseline { tests: set(&["a::one", "a::two"]), fixtures: set(&["f/x.fixture"]), removed: BTreeSet::new() }
    }

    #[test]
    fn render_then_parse_round_trips() {
        let mut b = base();
        b.removed.insert(("test".into(), "a::old".into(), "ADR-0007".into()));
        assert_eq!(Baseline::parse(&b.render()).unwrap(), b);
    }

    #[test]
    fn parse_rejects_garbage_and_missing_schema() {
        assert!(Baseline::parse("test a::one\n").is_err());
        assert!(Baseline::parse("schema baseline/1\nbogus line\n").is_err());
        assert!(Baseline::parse("schema baseline/1\nremoved test only-two\n").is_err());
        assert!(Baseline::parse("schema baseline/2\n").is_err());
    }

    #[test]
    fn deleting_a_test_is_a_violation_even_if_another_was_added() {
        let b = base();
        let passing = set(&["a::one", "a::brand_new"]);
        let v = b.check_run(&passing, &set(&["f/x.fixture"]));
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].subject, "a::two");
        assert_eq!(b.added(&passing, &set(&["f/x.fixture"])), ["a::brand_new"]);
    }

    #[test]
    fn a_failing_or_ignored_test_counts_as_not_passing() {
        // The caller only puts tests that passed into `passing`, so ignoring a test is
        // the same as deleting it as far as the ratchet is concerned.
        let v = base().check_run(&set(&["a::one"]), &set(&["f/x.fixture"]));
        assert_eq!(v.len(), 1);
    }

    #[test]
    fn a_missing_fixture_is_a_violation() {
        let v = base().check_run(&set(&["a::one", "a::two"]), &set(&[]));
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].subject, "f/x.fixture");
    }

    #[test]
    fn a_removed_entry_with_an_existing_adr_is_allowed() {
        let mut now = base();
        now.tests.remove("a::two");
        now.removed.insert(("test".into(), "a::two".into(), "ADR-0007".into()));
        assert!(now.check_against_base(&base(), &|adr| adr == "ADR-0007").is_empty());
        assert!(now.check_run(&set(&["a::one"]), &set(&["f/x.fixture"])).is_empty());
    }

    #[test]
    fn editing_the_baseline_to_hide_a_deletion_is_caught_against_the_base() {
        let mut now = base();
        now.tests.remove("a::two");
        let v = now.check_against_base(&base(), &|_| true);
        assert_eq!(v.len(), 1);
        assert!(v[0].detail.contains("no 'removed' record"));
    }

    #[test]
    fn a_removal_citing_a_missing_adr_is_caught() {
        let mut now = base();
        now.tests.remove("a::two");
        now.removed.insert(("test".into(), "a::two".into(), "ADR-9999".into()));
        let v = now.check_against_base(&base(), &|_| false);
        assert_eq!(v.len(), 1);
        assert!(v[0].detail.contains("does not exist"));
    }

    #[test]
    fn deleting_a_removal_record_is_caught() {
        let mut older = base();
        older.tests.remove("a::two");
        older.removed.insert(("test".into(), "a::two".into(), "ADR-0007".into()));
        let v = base().check_against_base(&older, &|_| true);
        assert!(v.iter().any(|v| v.detail.contains("removal record was deleted")));
    }

    #[test]
    fn growing_the_baseline_is_always_fine() {
        let mut now = base();
        now.tests.insert("a::three".into());
        assert!(now.check_against_base(&base(), &|_| false).is_empty());
    }
}
