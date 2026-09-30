//! Parse the human-readable output of `cargo test` into per-binary test results.
//!
//! Test names are the unit of the ratchet: a test that disappears or stops passing is
//! a regression even if a new test was added elsewhere, so a bare count is not enough.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Ok,
    Failed,
    Ignored,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestResult {
    /// `<binary>::<test path>`, for example `octocore::engine::tests::foo`.
    pub id: String,
    pub outcome: Outcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinaryRun {
    pub binary: String,
    pub tests: Vec<TestResult>,
    /// From the `test result:` line, used to cross-check the parsed names.
    pub summary_passed: Option<usize>,
    pub summary_failed: Option<usize>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Parsed {
    pub binaries: Vec<BinaryRun>,
}

impl Parsed {
    pub fn all_tests(&self) -> impl Iterator<Item = &TestResult> {
        self.binaries.iter().flat_map(|b| b.tests.iter())
    }

    pub fn passed(&self) -> Vec<&str> {
        self.all_tests().filter(|t| t.outcome == Outcome::Ok).map(|t| t.id.as_str()).collect()
    }

    pub fn failed(&self) -> Vec<&str> {
        self.all_tests().filter(|t| t.outcome == Outcome::Failed).map(|t| t.id.as_str()).collect()
    }

    pub fn binary(&self, name: &str) -> Option<&BinaryRun> {
        self.binaries.iter().find(|b| b.binary == name && !b.tests.is_empty())
    }

    /// Test ids that appear more than once. Two binaries with the same name would make
    /// the ratchet ambiguous, so the runner fails loudly instead of guessing.
    pub fn duplicate_ids(&self) -> Vec<String> {
        let mut seen = std::collections::BTreeSet::new();
        let mut dups = std::collections::BTreeSet::new();
        for t in self.all_tests() {
            if !seen.insert(t.id.clone()) {
                dups.insert(t.id.clone());
            }
        }
        dups.into_iter().collect()
    }

    /// Binaries whose parsed passing and failing counts disagree with cargo's own
    /// summary line. That means the parser missed something, which must not be silent.
    pub fn summary_mismatches(&self) -> Vec<String> {
        let mut out = Vec::new();
        for b in &self.binaries {
            let ok = b.tests.iter().filter(|t| t.outcome == Outcome::Ok).count();
            let bad = b.tests.iter().filter(|t| t.outcome == Outcome::Failed).count();
            if let Some(p) = b.summary_passed {
                if p != ok {
                    out.push(format!("{}: cargo reports {p} passed, parsed {ok}", b.binary));
                }
            }
            if let Some(f) = b.summary_failed {
                if f != bad {
                    out.push(format!("{}: cargo reports {f} failed, parsed {bad}", b.binary));
                }
            }
        }
        out
    }
}

/// Strip a trailing `-<hex hash>` from a test binary file name.
fn binary_name(path: &str) -> String {
    let file = path.rsplit(['/', '\\']).next().unwrap_or(path);
    let file = file.strip_suffix(".exe").unwrap_or(file);
    match file.rsplit_once('-') {
        Some((name, hash)) if hash.len() >= 8 && hash.chars().all(|c| c.is_ascii_hexdigit()) => {
            name.to_string()
        }
        _ => file.to_string(),
    }
}

fn leading_number_before(s: &str, word: &str) -> Option<usize> {
    let idx = s.find(word)?;
    let head = s[..idx].trim_end();
    let digits: String = head.chars().rev().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }
    digits.chars().rev().collect::<String>().parse().ok()
}

pub fn parse(output: &str) -> Parsed {
    let mut parsed = Parsed::default();
    let mut current: Option<usize> = None;

    for line in output.lines() {
        let trimmed = line.trim_start();

        if let Some(rest) = trimmed.strip_prefix("Running ") {
            if let (Some(open), Some(close)) = (rest.rfind('('), rest.rfind(')')) {
                if open < close {
                    parsed.binaries.push(BinaryRun {
                        binary: binary_name(&rest[open + 1..close]),
                        tests: Vec::new(),
                        summary_passed: None,
                        summary_failed: None,
                    });
                    current = Some(parsed.binaries.len() - 1);
                }
            }
            continue;
        }

        if let Some(krate) = trimmed.strip_prefix("Doc-tests ") {
            parsed.binaries.push(BinaryRun {
                binary: format!("doctests:{}", krate.trim()),
                tests: Vec::new(),
                summary_passed: None,
                summary_failed: None,
            });
            current = Some(parsed.binaries.len() - 1);
            continue;
        }

        if let Some(rest) = line.strip_prefix("test result:") {
            if let Some(i) = current {
                parsed.binaries[i].summary_passed = leading_number_before(rest, "passed");
                parsed.binaries[i].summary_failed = leading_number_before(rest, "failed");
            }
            continue;
        }

        // Test lines start at column 0: `test name ... ok`.
        if let Some(rest) = line.strip_prefix("test ") {
            let Some(i) = current else { continue };
            let Some((name, status)) = rest.rsplit_once(" ... ") else { continue };
            let outcome = if status.starts_with("ok") {
                Outcome::Ok
            } else if status.starts_with("FAILED") {
                Outcome::Failed
            } else if status.starts_with("ignored") {
                Outcome::Ignored
            } else {
                continue;
            };
            let id = format!("{}::{}", parsed.binaries[i].binary, name.trim());
            parsed.binaries[i].tests.push(TestResult { id, outcome });
        }
    }
    parsed
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
   Compiling octocore v0.1.0
     Running unittests src/lib.rs (target/debug/deps/octocore-b6533c2f0b8612d9)

running 3 tests
test engine::tests::plays ... ok
test engine::tests::stops ... FAILED
test engine::tests::slow ... ignored

failures:

---- engine::tests::stops stdout ----

test result: FAILED. 1 passed; 1 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running tests/conformance.rs (target/debug/deps/conformance-605c23d93e1823ac)

running 1 test
test conformance_fixtures ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests octocore

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
";

    #[test]
    fn parses_binaries_and_outcomes() {
        let p = parse(SAMPLE);
        let names: Vec<_> = p.binaries.iter().map(|b| b.binary.as_str()).collect();
        assert_eq!(names, ["octocore", "conformance", "doctests:octocore"]);
        assert_eq!(p.passed(), ["octocore::engine::tests::plays", "conformance::conformance_fixtures"]);
        assert_eq!(p.failed(), ["octocore::engine::tests::stops"]);
        assert_eq!(p.binaries[0].tests.len(), 3);
    }

    #[test]
    fn a_test_shaped_line_inside_failure_output_trips_the_summary_check() {
        // Captured stdout of a failing test is printed at column 0, so the parser
        // cannot tell a real result from a line that only looks like one. It is
        // counted, and the cross-check against cargo's own summary line catches it.
        let noisy = SAMPLE.replace(
            "---- engine::tests::stops stdout ----\n",
            "---- engine::tests::stops stdout ----\ntest inside failure output ... ok\n",
        );
        assert!(!parse(&noisy).summary_mismatches().is_empty());
    }

    #[test]
    fn summary_line_agrees_when_output_is_clean() {
        let p = parse(SAMPLE);
        assert!(p.summary_mismatches().is_empty(), "{:?}", p.summary_mismatches());
        assert_eq!(p.binaries[0].summary_passed, Some(1));
        assert_eq!(p.binaries[0].summary_failed, Some(1));
    }

    #[test]
    fn binary_names_lose_their_hash_but_keep_dashes_in_names() {
        assert_eq!(binary_name("target/debug/deps/octocore-b6533c2f0b8612d9"), "octocore");
        assert_eq!(binary_name("target/debug/deps/my-crate-0123456789abcdef"), "my-crate");
        assert_eq!(binary_name("target/debug/deps/nohash"), "nohash");
        assert_eq!(binary_name("target\\debug\\deps\\octocore-b6533c2f0b8612d9.exe"), "octocore");
    }

    #[test]
    fn duplicate_ids_are_reported() {
        let dup = "\
     Running unittests src/lib.rs (target/debug/deps/a-0123456789abcdef)
test x ... ok
     Running unittests src/lib.rs (target/debug/deps/a-fedcba9876543210)
test x ... ok
";
        assert_eq!(parse(dup).duplicate_ids(), ["a::x"]);
    }

    #[test]
    fn test_lines_before_any_binary_are_ignored() {
        assert!(parse("test orphan ... ok\n").binaries.is_empty());
    }
}
