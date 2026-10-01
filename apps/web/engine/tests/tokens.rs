//! `contracts/design.tokens.json` against its schema and against the rules of ADR-0008 decision 6.
//!
//! The test asserts rules and not values, so a change of hue passes and a change that breaks a rule
//! does not. Each rule is also shown to fail on a document made to break it, so a rule that could
//! never fire cannot sit here looking like a check (the ratchet's reason for existing).
//!
//! It lives in this crate, and not in the web package, because `cargo xtask baseline` reads `cargo test`
//! output: a test here is floored by name, and a test in Node is not (D-P3-8).

use serde_json::{json, Value};
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------------------------
// Reading
// ---------------------------------------------------------------------------------------------

fn repo_root() -> PathBuf {
    // CARGO_MANIFEST_DIR = apps/web/engine
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn read_json(rel: &str) -> Value {
    let path = repo_root().join(rel);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{} is not JSON: {e}", path.display()))
}

fn tokens() -> Value {
    read_json("contracts/design.tokens.json")
}

fn schema_errors(doc: &Value) -> Vec<String> {
    let schema = read_json("contracts/design.tokens.schema.json");
    let validator = jsonschema::draft202012::new(&schema).expect("the schema compiles");
    validator.iter_errors(doc).map(|e| format!("{} at {}", e, e.instance_path)).collect()
}

// ---------------------------------------------------------------------------------------------
// Colour arithmetic
// ---------------------------------------------------------------------------------------------

type Rgb = [u8; 3];

/// `#rrggbb`, lower case, and nothing else.
fn parse_hex(s: &str) -> Option<Rgb> {
    let b = s.as_bytes();
    if b.len() != 7 || b[0] != b'#' || !b[1..].iter().all(|c| matches!(c, b'0'..=b'9' | b'a'..=b'f')) {
        return None;
    }
    let v = |i: usize| u8::from_str_radix(&s[i..i + 2], 16).ok();
    Some([v(1)?, v(3)?, v(5)?])
}

fn linear(c: u8) -> f64 {
    let c = f64::from(c) / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// WCAG relative luminance.
fn luminance(c: Rgb) -> f64 {
    0.2126 * linear(c[0]) + 0.7152 * linear(c[1]) + 0.0722 * linear(c[2])
}

/// WCAG contrast ratio, from 1 to 21.
fn contrast(a: Rgb, b: Rgb) -> f64 {
    let (la, lb) = (luminance(a), luminance(b));
    let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

fn lab(c: Rgb) -> [f64; 3] {
    let (r, g, b) = (linear(c[0]), linear(c[1]), linear(c[2]));
    let x = 0.4124564 * r + 0.3575761 * g + 0.1804375 * b;
    let y = 0.2126729 * r + 0.7151522 * g + 0.0721750 * b;
    let z = 0.0193339 * r + 0.1191920 * g + 0.9503041 * b;
    let f = |t: f64| if t > 216.0 / 24389.0 { t.cbrt() } else { (24389.0 / 27.0 * t + 16.0) / 116.0 };
    let (fx, fy, fz) = (f(x / 0.95047), f(y), f(z / 1.08883));
    [116.0 * fy - 16.0, 500.0 * (fx - fy), 200.0 * (fy - fz)]
}

/// CIEDE2000 (Sharma, Wu and Dalal, 2005), the stricter of the usual measures for "near".
fn delta_e2000(l1: [f64; 3], l2: [f64; 3]) -> f64 {
    let [ll1, a1, b1] = l1;
    let [ll2, a2, b2] = l2;
    let rad = std::f64::consts::PI / 180.0;
    let c1 = a1.hypot(b1);
    let c2 = a2.hypot(b2);
    let cbar7 = (((c1 + c2) / 2.0).powi(7)).max(0.0);
    let g = 0.5 * (1.0 - (cbar7 / (cbar7 + 25f64.powi(7))).sqrt());
    let (ap1, ap2) = ((1.0 + g) * a1, (1.0 + g) * a2);
    let (cp1, cp2) = (ap1.hypot(b1), ap2.hypot(b2));
    let hue = |b: f64, a: f64| {
        if b == 0.0 && a == 0.0 {
            0.0
        } else {
            let h = b.atan2(a) / rad;
            if h < 0.0 {
                h + 360.0
            } else {
                h
            }
        }
    };
    let (hp1, hp2) = (hue(b1, ap1), hue(b2, ap2));
    let dl = ll2 - ll1;
    let dc = cp2 - cp1;
    let dh = if cp1 * cp2 == 0.0 {
        0.0
    } else if (hp2 - hp1).abs() <= 180.0 {
        hp2 - hp1
    } else if hp2 - hp1 > 180.0 {
        hp2 - hp1 - 360.0
    } else {
        hp2 - hp1 + 360.0
    };
    let dhh = 2.0 * (cp1 * cp2).sqrt() * (dh / 2.0 * rad).sin();
    let lbar = (ll1 + ll2) / 2.0;
    let cbar = (cp1 + cp2) / 2.0;
    let hbar = if cp1 * cp2 == 0.0 {
        hp1 + hp2
    } else if (hp1 - hp2).abs() <= 180.0 {
        (hp1 + hp2) / 2.0
    } else if hp1 + hp2 < 360.0 {
        (hp1 + hp2 + 360.0) / 2.0
    } else {
        (hp1 + hp2 - 360.0) / 2.0
    };
    let t = 1.0 - 0.17 * ((hbar - 30.0) * rad).cos() + 0.24 * ((2.0 * hbar) * rad).cos() + 0.32 * ((3.0 * hbar + 6.0) * rad).cos()
        - 0.20 * ((4.0 * hbar - 63.0) * rad).cos();
    let dtheta = 30.0 * (-((hbar - 275.0) / 25.0).powi(2)).exp();
    let rc = 2.0 * (cbar.powi(7) / (cbar.powi(7) + 25f64.powi(7))).sqrt();
    let sl = 1.0 + 0.015 * (lbar - 50.0).powi(2) / (20.0 + (lbar - 50.0).powi(2)).sqrt();
    let sc = 1.0 + 0.045 * cbar;
    let sh = 1.0 + 0.015 * cbar * t;
    let rt = -((2.0 * dtheta) * rad).sin() * rc;
    ((dl / sl).powi(2) + (dc / sc).powi(2) + (dhh / sh).powi(2) + rt * (dc / sc) * (dhh / sh)).sqrt()
}

fn delta_e(a: Rgb, b: Rgb) -> f64 {
    delta_e2000(lab(a), lab(b))
}

// ---------------------------------------------------------------------------------------------
// The rules
// ---------------------------------------------------------------------------------------------

/// The colours the AI-design cluster of rule S8 names (`docs/05` section 4).
const S8_CLUSTER: [&str; 4] = ["#0b0b0b", "#111111", "#f4f1ea", "#d97757"];

const COLOUR_KEYS: [&str; 9] = ["led.red", "led.green", "led.orange", "led.off", "ink", "ink.quiet", "surface", "focus", "focus.inner"];
const LIT: [&str; 3] = ["led.red", "led.green", "led.orange"];
const NEUTRAL: [&str; 6] = ["led.off", "ink", "ink.quiet", "surface", "focus", "focus.inner"];
const SPACE_SCALE: [u64; 8] = [0, 4, 8, 12, 20, 32, 52, 84];
const REFUSED_FACES: [&str; 8] = ["inter", "google sans", "sf pro", "system-ui", "-apple-system", "blinkmacsystemfont", "ui-sans-serif", "segoe ui"];

fn colour<'a>(doc: &'a Value, key: &str) -> Option<&'a Value> {
    doc["colour"].get(key)
}

fn rgb(doc: &Value, key: &str) -> Option<Rgb> {
    colour(doc, key)?["value"].as_str().and_then(parse_hex)
}

fn neutral(c: Rgb) -> bool {
    c[0] == c[1] && c[1] == c[2]
}

/// Every broken rule, each line starting with the rule's tag.
fn rule_violations(doc: &Value) -> Vec<String> {
    let mut out = Vec::new();

    // T1: every colour is #rrggbb; every space value is one of the eight.
    for key in COLOUR_KEYS {
        match colour(doc, key).and_then(|c| c["value"].as_str()) {
            None => out.push(format!("T1 colour.{key} is missing or not a string")),
            Some(s) if parse_hex(s).is_none() => out.push(format!("T1 colour.{key} is {s:?}, not #rrggbb")),
            Some(_) => {}
        }
    }
    if let Some(space) = doc["space"].as_object() {
        for (k, v) in space {
            match v["value"].as_u64() {
                Some(n) if SPACE_SCALE.contains(&n) => {}
                other => out.push(format!("T1 space.{k} is {other:?}, not one of {SPACE_SCALE:?}")),
            }
        }
        if space.len() != SPACE_SCALE.len() {
            out.push(format!("T1 space has {} values, the scale has {} and no ninth", space.len(), SPACE_SCALE.len()));
        }
    } else {
        out.push("T1 space is missing".into());
    }

    let (Some(ink), Some(quiet), Some(surface)) = (rgb(doc, "ink"), rgb(doc, "ink.quiet"), rgb(doc, "surface")) else {
        out.push("T1 ink, ink.quiet or surface cannot be read; the other colour rules were not run".into());
        return out;
    };

    // T2: text against the ground, 7:1.
    for (name, c) in [("ink", ink), ("ink.quiet", quiet)] {
        let r = contrast(c, surface);
        if r < 7.0 {
            out.push(format!("T2 {name} against surface is {r:.2}:1, the floor is 7:1"));
        }
    }

    // T3: an unlit LED differs from each lit one by more than hue.
    if let Some(off) = rgb(doc, "led.off") {
        for key in LIT {
            if let Some(lit) = rgb(doc, key) {
                let r = contrast(off, lit);
                if r < 3.0 {
                    out.push(format!("T3 led.off against {key} is {r:.2}:1, the floor is 3:1"));
                }
            }
        }
    }

    // T4: nothing within ΔE 3 of the S8 cluster, unless the token says why (rule S8's own exit).
    for key in COLOUR_KEYS {
        let Some(c) = rgb(doc, key) else { continue };
        let reasons = colour(doc, key).and_then(|t| t["suppress"].as_array()).cloned().unwrap_or_default();
        let waived = reasons.iter().any(|r| r["rule"] == "S8" && r["why"].as_str().is_some_and(|w| !w.trim().is_empty()));
        for bad in S8_CLUSTER {
            let d = delta_e(c, parse_hex(bad).expect("the cluster is hex"));
            if d < 3.0 && !waived {
                out.push(format!("T4 colour.{key} is within ΔE {d:.2} of {bad} (rule S8)"));
            }
        }
    }

    // T5: the focus ring is neutral and shows on the ground and on every lit LED.
    if let (Some(f), Some(fi)) = (rgb(doc, "focus"), rgb(doc, "focus.inner")) {
        if !neutral(f) || !neutral(fi) {
            out.push("T5 focus and focus.inner must each have equal red, green and blue".into());
        }
        if contrast(f, surface).max(contrast(fi, surface)) < 3.0 {
            out.push("T5 neither focus nor focus.inner reaches 3:1 against surface".into());
        }
        for key in LIT {
            if let Some(lit) = rgb(doc, key) {
                if contrast(f, lit).max(contrast(fi, lit)) < 3.0 {
                    out.push(format!("T5 neither focus nor focus.inner reaches 3:1 against {key}"));
                }
            }
        }
    }

    // T6: fewer than three flashes a second (WCAG 2.3.1), or the file calls the rate a finding.
    let flash = &doc["flash"]["period_ms"];
    match flash["value"].as_f64() {
        Some(ms) if ms > 0.0 => {
            let per_second = 1000.0 / ms;
            let admitted = flash["finding"].as_str().is_some_and(|s| !s.trim().is_empty());
            if per_second >= 3.0 && !admitted {
                out.push(format!("T6 flash.period_ms {ms} flashes {per_second:.2} times a second; fewer than 3 is the floor"));
            }
        }
        _ => out.push("T6 flash.period_ms is missing or not a positive number".into()),
    }

    // T7: the only keys with a hue are the three lit roles.
    for key in NEUTRAL {
        if let Some(c) = rgb(doc, key) {
            if !neutral(c) {
                out.push(format!("T7 colour.{key} has a hue ({}); only the led roles may", colour(doc, key).unwrap()["value"]));
            }
        }
    }

    // T8: the three tokens that wait on the manual say so, in the shape `controls.json` uses.
    for (group, key) in [("flash", "period_ms"), ("flash", "shine"), ("type", "family")] {
        let p = &doc[group][key]["pending"];
        let said = |f: &str| p[f].as_str().is_some_and(|s| !s.trim().is_empty());
        if !(said("question") && said("note")) {
            out.push(format!("T8 {group}.{key} carries no `pending` with a question and a note"));
        }
    }

    // T9: the three lit roles are told apart from each other by more than a shade.
    for (i, a) in LIT.iter().enumerate() {
        for b in &LIT[i + 1..] {
            if let (Some(ca), Some(cb)) = (rgb(doc, a), rgb(doc, b)) {
                let d = delta_e(ca, cb);
                if d < 15.0 {
                    out.push(format!("T9 {a} and {b} are ΔE {d:.1} apart; the floor is 15"));
                }
            }
        }
        if let Some(c) = rgb(doc, a) {
            if neutral(c) {
                out.push(format!("T9 {a} is grey; a lit LED is the one place colour is allowed"));
            }
        }
    }

    // T10: the type face is not one of the defaults rule S11 names, and no group the design
    // system forbids (docs/05 section 2.3: no radius scale; section 4: no shadow, gradient or easing).
    if let Some(face) = doc["type"]["family"]["value"].as_str() {
        let lower = face.to_lowercase();
        for refused in REFUSED_FACES {
            if lower.split(',').any(|f| f.trim().trim_matches('"').trim_matches('\'') == refused) {
                out.push(format!("T10 type.family names {refused}, which rule S11 refuses"));
            }
        }
    }
    if let Some(top) = doc.as_object() {
        for k in top.keys() {
            let k = k.to_lowercase();
            if ["radius", "shadow", "gradient", "easing", "motion", "transition"].iter().any(|bad| k.contains(bad)) {
                out.push(format!("T10 the file has a `{k}` group; there is none by design (ADR-0008 decision 6)"));
            }
        }
    }

    // T11: the measure is the 68-character line the design system sets.
    if doc["type"]["measure"]["value"] != json!(68) {
        out.push("T11 type.measure is not 68".into());
    }

    out
}

/// What a broken document is rejected for: the schema, and the rules.
fn everything_wrong(doc: &Value) -> Vec<String> {
    let mut out: Vec<String> = schema_errors(doc).into_iter().map(|e| format!("schema {e}")).collect();
    out.extend(rule_violations(doc));
    out
}

fn with(mut doc: Value, path: &[&str], value: Value) -> Value {
    let mut at = &mut doc;
    for key in &path[..path.len() - 1] {
        at = &mut at[*key];
    }
    at[path[path.len() - 1]] = value;
    doc
}

fn has(found: &[String], tag: &str) -> bool {
    found.iter().any(|l| l.starts_with(tag))
}

// ---------------------------------------------------------------------------------------------
// The file
// ---------------------------------------------------------------------------------------------

#[test]
fn tokens_json_is_valid_against_its_schema() {
    let errors = schema_errors(&tokens());
    assert!(errors.is_empty(), "design.tokens.json does not match design.tokens.schema.json:\n{}", errors.join("\n"));
}

#[test]
fn the_tokens_keep_every_rule() {
    let found = rule_violations(&tokens());
    assert!(found.is_empty(), "design.tokens.json breaks its rules:\n{}", found.join("\n"));
}

#[test]
fn the_tokens_say_what_they_are() {
    let doc = tokens();
    assert_eq!(doc["schema"], "design.tokens/1");
    let colours: Vec<&str> = doc["colour"].as_object().unwrap().keys().map(String::as_str).collect();
    let mut want = COLOUR_KEYS.to_vec();
    want.sort_unstable();
    let mut got = colours;
    got.sort_unstable();
    assert_eq!(got, want, "the nine colour roles and no tenth (CLAUDE.md rule 6)");
    let space: Vec<u64> = (0..8).map(|i| doc["space"][i.to_string()]["value"].as_u64().unwrap()).collect();
    assert_eq!(space, SPACE_SCALE, "docs/05 section 2.2");
}

// ---------------------------------------------------------------------------------------------
// Each rule can fail
// ---------------------------------------------------------------------------------------------

#[test]
fn t1_a_colour_that_is_not_rrggbb_is_refused() {
    for bad in ["#abc", "#FFFFFF", "white", "rgb(0,0,0)", "#12345g", "#1234567"] {
        let found = everything_wrong(&with(tokens(), &["colour", "ink", "value"], json!(bad)));
        assert!(has(&found, "T1"), "{bad} was accepted: {found:?}");
        assert!(has(&found, "schema"), "{bad}: the schema accepted it too");
    }
}

#[test]
fn t1_a_space_value_off_the_scale_or_a_ninth_is_refused() {
    let found = everything_wrong(&with(tokens(), &["space", "3", "value"], json!(13)));
    assert!(has(&found, "T1") && has(&found, "schema"), "{found:?}");
    let found = everything_wrong(&with(tokens(), &["space", "8"], json!({ "value": 140 })));
    assert!(has(&found, "T1") && has(&found, "schema"), "{found:?}");
}

#[test]
fn t2_text_that_is_too_quiet_against_the_ground_is_refused() {
    let doc = tokens();
    let surface = rgb(&doc, "surface").unwrap();
    // Walk ink toward the ground until it fails: the rule is about the ratio, not a colour.
    let mut quiet_enough = None;
    for g in 0..=255u8 {
        let c = [g, g, g];
        if contrast(c, surface) < 7.0 && contrast(c, surface) > 1.5 {
            quiet_enough = Some(g);
            break;
        }
    }
    let g = quiet_enough.expect("some grey sits under 7:1");
    let hex = format!("#{g:02x}{g:02x}{g:02x}");
    assert!(has(&rule_violations(&with(doc.clone(), &["colour", "ink", "value"], json!(hex))), "T2"));
    assert!(has(&rule_violations(&with(doc, &["colour", "ink.quiet", "value"], json!(hex))), "T2"));
}

#[test]
fn t3_an_unlit_led_that_looks_like_a_lit_one_is_refused() {
    let doc = tokens();
    let lit = doc["colour"]["led.green"]["value"].clone();
    let found = rule_violations(&with(doc, &["colour", "led.off", "value"], lit));
    assert!(has(&found, "T3"), "{found:?}");
}

#[test]
fn t4_a_colour_near_the_s8_cluster_is_refused_unless_it_says_why() {
    for (key, bad) in [("surface", "#111111"), ("surface", "#0b0b0b"), ("ink", "#f4f1ea"), ("led.orange", "#d97757"), ("led.orange", "#da7858")] {
        let found = rule_violations(&with(tokens(), &["colour", key, "value"], json!(bad)));
        assert!(has(&found, "T4"), "{key} = {bad}: {found:?}");
    }
    let waived = with(
        with(tokens(), &["colour", "surface", "value"], json!("#111111")),
        &["colour", "surface", "suppress"],
        json!([{ "rule": "S8", "why": "the owner chose it after seeing the sketch" }]),
    );
    assert!(!has(&rule_violations(&waived), "T4"), "a stated reason is the exit");
    let empty = with(waived, &["colour", "surface", "suppress"], json!([{ "rule": "S8", "why": "  " }]));
    assert!(has(&rule_violations(&empty), "T4"), "an empty reason is not a reason");
}

#[test]
fn t5_a_focus_ring_that_has_a_hue_or_cannot_be_seen_is_refused() {
    let found = rule_violations(&with(tokens(), &["colour", "focus", "value"], json!("#ff0000")));
    assert!(has(&found, "T5"), "{found:?}");
    let surface = tokens()["colour"]["surface"]["value"].clone();
    let both = with(with(tokens(), &["colour", "focus", "value"], surface.clone()), &["colour", "focus.inner", "value"], surface);
    let found = rule_violations(&both);
    assert!(has(&found, "T5"), "{found:?}");
}

#[test]
fn t5_a_ring_that_vanishes_on_one_lit_led_is_refused() {
    // The ring is two strokes, so it can show on the ground with one and on a lit LED with the other.
    // Make both strokes the lit green's own grey: they vanish on green and nowhere else.
    let doc = tokens();
    let green = rgb(&doc, "led.green").unwrap();
    let surface = rgb(&doc, "surface").unwrap();
    let g = (0..=255u8).find(|g| contrast([*g; 3], green) < 3.0 && contrast([*g; 3], surface) >= 3.0).expect("a grey that hides on green");
    let hex = format!("#{g:02x}{g:02x}{g:02x}");
    let both = with(with(doc, &["colour", "focus", "value"], json!(hex)), &["colour", "focus.inner", "value"], json!(hex));
    let found = rule_violations(&both);
    assert!(found.iter().any(|l| l.starts_with("T5") && l.contains("led.green")), "{found:?}");
}

#[test]
fn t6_three_flashes_a_second_is_refused_and_two_and_a_half_is_not() {
    let at = |ms: u64| rule_violations(&with(tokens(), &["flash", "period_ms", "value"], json!(ms)));
    assert!(has(&at(333), "T6"), "333 ms is 3.003 a second");
    assert!(!has(&at(334), "T6"), "334 ms is 2.994 a second");
    assert!(has(&at(100), "T6"));
    assert!(has(&at(0), "T6"));
    let admitted = with(tokens(), &["flash", "period_ms", "value"], json!(100));
    let admitted = with(admitted, &["flash", "period_ms", "finding"], json!("the manual gives 10 Hz; this is a finding for the owner"));
    assert!(!has(&rule_violations(&admitted), "T6"), "the file may call the rate a finding");
}

#[test]
fn t7_a_neutral_that_has_a_hue_is_refused() {
    for key in NEUTRAL {
        let found = rule_violations(&with(tokens(), &["colour", key, "value"], json!("#3a3a46")));
        assert!(found.iter().any(|l| l.starts_with("T7") && l.contains(key)), "{key}: {found:?}");
    }
}

#[test]
fn t8_a_token_that_waits_on_the_manual_must_say_so() {
    for (group, key) in [("flash", "period_ms"), ("flash", "shine"), ("type", "family")] {
        let mut doc = tokens();
        doc[group][key].as_object_mut().unwrap().remove("pending");
        let found = rule_violations(&doc);
        assert!(found.iter().any(|l| l.starts_with("T8") && l.contains(key)), "{group}.{key}: {found:?}");
        let empty = with(tokens(), &[group, key, "pending"], json!({ "question": "Q03", "note": "" }));
        assert!(has(&rule_violations(&empty), "T8"), "an empty note is not a note");
    }
}

#[test]
fn t9_lit_roles_that_cannot_be_told_apart_are_refused() {
    let doc = tokens();
    let red = doc["colour"]["led.red"]["value"].clone();
    let found = rule_violations(&with(doc.clone(), &["colour", "led.orange", "value"], red));
    assert!(has(&found, "T9"), "{found:?}");
    let grey = rule_violations(&with(doc, &["colour", "led.green", "value"], json!("#808080")));
    assert!(grey.iter().any(|l| l.starts_with("T9") && l.contains("grey")), "{grey:?}");
}

#[test]
fn t10_a_refused_face_and_a_forbidden_group_are_refused() {
    for face in ["Inter, sans-serif", "\"Google Sans\", Arial", "system-ui", "-apple-system, Helvetica", "Helvetica, \"SF Pro\""] {
        let found = rule_violations(&with(tokens(), &["type", "family", "value"], json!(face)));
        assert!(has(&found, "T10"), "{face}: {found:?}");
    }
    for group in ["radius", "shadow", "gradient", "easing"] {
        let found = everything_wrong(&with(tokens(), &[group], json!({ "sm": { "value": 4 } })));
        assert!(has(&found, "T10") && has(&found, "schema"), "{group}: {found:?}");
    }
}

#[test]
fn t11_the_measure_is_sixty_eight() {
    assert!(has(&rule_violations(&with(tokens(), &["type", "measure", "value"], json!(80))), "T11"));
}

// ---------------------------------------------------------------------------------------------
// The arithmetic the rules stand on
// ---------------------------------------------------------------------------------------------

#[test]
fn the_contrast_ratio_matches_the_wcag_figures() {
    assert!((contrast([0, 0, 0], [255, 255, 255]) - 21.0).abs() < 1e-9);
    assert!((contrast([255, 255, 255], [255, 255, 255]) - 1.0).abs() < 1e-9);
    // #767676 on white is the well-known 4.54:1 (the lightest grey that passes AA).
    assert!((contrast([0x76, 0x76, 0x76], [255, 255, 255]) - 4.54).abs() < 0.01);
}

#[test]
fn ciede2000_reproduces_the_published_test_pairs() {
    // Sharma, Wu and Dalal (2005), Table 1, pairs 1 to 3 and 17.
    let pairs: [([f64; 3], [f64; 3], f64); 4] = [
        ([50.0, 2.6772, -79.7751], [50.0, 0.0, -82.7485], 2.0425),
        ([50.0, 3.1571, -77.2803], [50.0, 0.0, -82.7485], 2.8615),
        ([50.0, 2.8361, -74.0200], [50.0, 0.0, -82.7485], 3.4412),
        ([50.0, 2.5, 0.0], [50.0, 0.0, -2.5], 4.3065),
    ];
    for (a, b, want) in pairs {
        let got = delta_e2000(a, b);
        assert!((got - want).abs() < 5e-5, "{a:?} {b:?}: got {got}, the paper says {want}");
        let back = delta_e2000(b, a);
        assert!((back - want).abs() < 5e-5, "the measure must be symmetric: {back}");
    }
    assert!(delta_e([10, 20, 30], [10, 20, 30]).abs() < 1e-9);
}

#[test]
fn the_cluster_colours_are_far_from_one_another_in_the_metric_they_are_judged_by() {
    // A sanity check on the reference set: if two cluster entries were within ΔE 3 of each other the
    // rule would name one colour twice. (#0b0b0b and #111111 are close, and that is why both are listed.)
    let c: Vec<Rgb> = S8_CLUSTER.iter().map(|s| parse_hex(s).unwrap()).collect();
    assert!(delta_e(c[0], c[2]) > 20.0 && delta_e(c[2], c[3]) > 10.0 && delta_e(c[1], c[3]) > 10.0);
}
