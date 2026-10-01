//! `contracts/controls.json` against its schema, and against the controller (ADR-0007).

mod support;

use support::{inventory, read_json, repo_root};

#[test]
fn controls_json_is_valid_against_its_schema() {
    let schema = read_json("contracts/controls.schema.json");
    let doc = read_json("contracts/controls.json");
    let validator = jsonschema::draft202012::new(&schema).expect("the schema compiles");
    let errors: Vec<String> = validator.iter_errors(&doc).map(|e| format!("{} at {}", e, e.instance_path)).collect();
    assert!(errors.is_empty(), "controls.json does not match controls.schema.json:\n{}", errors.join("\n"));
}

#[test]
fn numbers_and_ids_are_unique() {
    let doc = read_json("contracts/controls.json");
    let controls = doc["controls"].as_array().unwrap();
    let mut n = std::collections::BTreeSet::new();
    let mut id = std::collections::BTreeSet::new();
    for c in controls {
        assert!(n.insert(c["n"].as_u64().unwrap()), "number used twice: {}", c["n"]);
        assert!(id.insert(c["id"].as_str().unwrap().to_string()), "id used twice: {}", c["id"]);
    }
    for g in doc["open"].as_array().unwrap() {
        assert!(id.insert(g["id"].as_str().unwrap().to_string()), "an open group reuses an id: {}", g["id"]);
    }
}

#[test]
fn every_zone_named_by_a_control_group_or_relation_exists() {
    let doc = read_json("contracts/controls.json");
    let zones: std::collections::BTreeSet<&str> = doc["zones"].as_array().unwrap().iter().map(|z| z["id"].as_str().unwrap()).collect();
    let check = |z: &str, what: &str| assert!(zones.contains(z), "{what} names the zone `{z}`, which the file does not define");
    for c in doc["controls"].as_array().unwrap() {
        check(c["zone"].as_str().unwrap(), c["id"].as_str().unwrap());
    }
    for g in doc["open"].as_array().unwrap() {
        check(g["zone"].as_str().unwrap(), g["id"].as_str().unwrap());
    }
    for r in doc["relations"].as_array().unwrap() {
        check(r["a"].as_str().unwrap(), "a relation");
        check(r["b"].as_str().unwrap(), "a relation");
    }
}

#[test]
fn every_cited_page_exists_in_the_manual() {
    let doc = read_json("contracts/controls.json");
    let mut cited = std::collections::BTreeSet::new();
    let mut collect = |v: &serde_json::Value| {
        for p in v["cite"].as_array().unwrap() {
            cited.insert(p.as_str().unwrap().to_string());
        }
    };
    for key in ["zones", "relations", "controls", "open"] {
        for item in doc[key].as_array().unwrap() {
            collect(item);
        }
    }
    let missing: Vec<&String> = cited.iter().filter(|p| !repo_root().join(format!("reference/manual/pages/{p}.txt")).exists()).collect();
    assert!(missing.is_empty(), "cited pages that are not in reference/manual/pages/: {missing:?}");
}

#[test]
fn every_question_the_inventory_names_exists() {
    let doc = read_json("contracts/controls.json");
    let text = std::fs::read_to_string(repo_root().join("tests/conformance/panel/QUESTIONS.md")).unwrap();
    let known = |q: &str| {
        let n: u32 = q[1..].parse().unwrap();
        (1..=20).contains(&n) || text.lines().any(|l| l.starts_with(&format!("| {q} |")))
    };
    for c in doc["controls"].as_array().unwrap() {
        if let Some(p) = c.get("pending") {
            assert!(known(p["question"].as_str().unwrap()), "{} names a question that does not exist", c["id"]);
        }
    }
    for g in doc["open"].as_array().unwrap() {
        assert!(known(g["question"].as_str().unwrap()), "{} names a question that does not exist", g["id"]);
    }
}

#[test]
fn the_controller_can_be_built_from_the_inventory() {
    // `inventory()` panics if the layout cannot be built: a control the controller needs is
    // missing, a number or an id is used twice, or a number is not below MAX_CONTROLS.
    let inv = inventory();
    assert!(inv.numbers.len() >= 200);
}
