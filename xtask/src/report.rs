//! Build the `verification.report/1` document (contracts/verification.report.schema.json).

use crate::gates::{Assertions, Gate};
use crate::json::Json;

pub struct Report {
    pub commit: String,
    pub branch: Option<String>,
    pub role: Option<String>,
    pub ran_at: String,
    pub duration_s: f64,
    pub hardware: String,
    pub gates: Vec<Gate>,
    pub assertions: Assertions,
}

pub fn to_json(r: &Report) -> Json {
    let mut top = vec![
        ("schema", Json::str("verification.report/1")),
        ("commit", Json::str(r.commit.clone())),
    ];
    if let Some(b) = &r.branch {
        top.push(("branch", Json::str(b.clone())));
    }
    if let Some(role) = &r.role {
        top.push(("role", Json::str(role.clone())));
    }
    top.push(("ran_at", Json::str(r.ran_at.clone())));
    top.push(("duration_s", Json::Num(r.duration_s)));
    top.push(("hardware", Json::str(r.hardware.clone())));
    top.push(("gates", Json::Arr(r.gates.iter().map(gate_json).collect())));
    top.push(("assertions", assertions_json(&r.assertions)));
    Json::obj(top)
}

fn gate_json(g: &Gate) -> Json {
    let mut o = vec![("name", Json::str(g.name)), ("status", Json::str(g.status.as_str()))];
    if let Some(d) = g.duration_s {
        o.push(("duration_s", Json::Num(d)));
    }
    let metrics = g
        .metrics
        .iter()
        .map(|m| {
            let mut mo = vec![("key", Json::str(m.key.clone())), ("value", Json::Num(m.value))];
            if let Some(u) = m.unit {
                mo.push(("unit", Json::str(u)));
            }
            mo.push(("threshold", Json::Num(m.threshold)));
            mo.push(("comparator", Json::str(m.cmp.as_str())));
            mo.push(("result", Json::str(if m.passes() { "pass" } else { "fail" })));
            Json::obj(mo)
        })
        .collect();
    o.push(("metrics", Json::Arr(metrics)));
    if !g.failures.is_empty() {
        let f = g
            .failures
            .iter()
            .map(|f| Json::obj(vec![("subject", Json::str(f.subject.clone())), ("detail", Json::str(f.detail.clone()))]))
            .collect();
        o.push(("failures", Json::Arr(f)));
    }
    Json::obj(o)
}

fn assertions_json(a: &Assertions) -> Json {
    Json::obj(vec![
        ("total", Json::Int(a.total as i64)),
        ("previous_total", Json::Int(a.previous_total as i64)),
        (
            "by_gate",
            Json::Obj(a.by_gate.iter().map(|(k, v)| (k.clone(), Json::Int(*v as i64))).collect()),
        ),
        ("added_this_commit", Json::Arr(a.added.iter().map(|s| Json::str(s.clone())).collect())),
        (
            "removed_this_commit",
            Json::Arr(
                a.removed
                    .iter()
                    .map(|(id, adr)| Json::obj(vec![("id", Json::str(id.clone())), ("adr", Json::str(adr.clone()))]))
                    .collect(),
            ),
        ),
    ])
}

/// `YYYY-MM-DDTHH:MM:SSZ` from Unix seconds. Civil-from-days by Howard Hinnant.
pub fn rfc3339_utc(unix_secs: u64) -> String {
    let days = (unix_secs / 86_400) as i64;
    let rem = unix_secs % 86_400;
    let (h, mi, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}T{h:02}:{mi:02}:{s:02}Z")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::baseline::Baseline;
    use crate::cargo_out::parse;
    use crate::gates::{evaluate, Inputs, TestRun};
    use std::collections::BTreeSet;

    #[test]
    fn rfc3339_known_values() {
        assert_eq!(rfc3339_utc(0), "1970-01-01T00:00:00Z");
        assert_eq!(rfc3339_utc(951_782_400), "2000-02-29T00:00:00Z"); // leap day
        assert_eq!(rfc3339_utc(1_782_734_400), "2026-06-29T12:00:00Z");
        assert_eq!(rfc3339_utc(4_102_444_799), "2099-12-31T23:59:59Z");
    }

    const OUT: &str = "\
     Running unittests src/lib.rs (target/debug/deps/octocore-0123456789abcdef)
test a::one ... ok
test a::bad ... FAILED

test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/conformance.rs (target/debug/deps/conformance-0123456789abcdef)
test conformance_fixtures ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
";

    fn sample_report() -> Report {
        let run = TestRun { parsed: parse(OUT), duration_s: 1.25, build_error: None };
        let fixtures: BTreeSet<String> = ["tests/conformance/x.fixture".to_string()].into();
        let mut baseline = Baseline::default();
        baseline.tests.insert("octocore::a::one".into());
        baseline.tests.insert("octocore::gone".into());
        baseline.fixtures.insert("tests/conformance/x.fixture".into());
        let mut base = baseline.clone();
        base.tests.insert("octocore::quoted \"name\"".into());
        let inputs = Inputs { run: &run, fixtures: &fixtures, baseline: &baseline, base: Some(&base), adr_exists: &|_| false, scope: &crate::scope::Scan::clean() };
        let (gates, assertions) = evaluate(&inputs);
        Report {
            commit: "6ef921f".into(),
            branch: Some("referee/ratchet".into()),
            role: Some("referee".into()),
            ran_at: rfc3339_utc(1_782_734_400),
            duration_s: 3.5,
            hardware: "linux-x86_64".into(),
            gates,
            assertions,
        }
    }

    fn schema() -> serde_json::Value {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../contracts/verification.report.schema.json");
        serde_json::from_str(&std::fs::read_to_string(path).expect("schema file")).expect("schema is json")
    }

    fn validator() -> jsonschema::Validator {
        jsonschema::options().should_validate_formats(true).build(&schema()).expect("schema compiles")
    }

    #[test]
    fn report_conforms_to_schema() {
        let text = to_json(&sample_report()).pretty();
        let instance: serde_json::Value = serde_json::from_str(&text).expect("report is valid json");
        let errors: Vec<String> = validator().iter_errors(&instance).map(|e| format!("{e} at {}", e.instance_path)).collect();
        assert!(errors.is_empty(), "{errors:#?}\n{text}");
    }

    #[test]
    fn the_sample_covers_pass_fail_and_not_implemented() {
        let r = sample_report();
        let statuses: BTreeSet<&str> = r.gates.iter().map(|g| g.status.as_str()).collect();
        assert!(statuses.contains("fail") && statuses.contains("not_implemented"), "{statuses:?}");
    }

    #[test]
    fn the_validator_actually_rejects_bad_reports() {
        let text = to_json(&sample_report()).pretty();
        let mut instance: serde_json::Value = serde_json::from_str(&text).unwrap();
        instance["gates"][0]["status"] = "green".into();
        assert!(!validator().is_valid(&instance));

        let mut instance: serde_json::Value = serde_json::from_str(&text).unwrap();
        instance.as_object_mut().unwrap().remove("assertions");
        assert!(!validator().is_valid(&instance));

        let mut instance: serde_json::Value = serde_json::from_str(&text).unwrap();
        instance["commit"] = "not-a-hash".into();
        assert!(!validator().is_valid(&instance));
    }
}
