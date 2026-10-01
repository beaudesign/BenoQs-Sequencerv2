//! The slop rules of `docs/05` section 4 that apply to the web app's own files, and the rule that no colour, length
//! or type size is written outside `contracts/design.tokens.json` (ADR-0008 decision 6).
//!
//! `verify:slop` and `verify:tokens` are not built yet (the Referee's). Until they are, the rules the panel can break
//! are asserted here, over the files the app is made of, and each rule is shown to fire on a snippet made to break it,
//! so a rule that could never fail cannot sit here looking like a check. The test lives in this crate for the reason
//! `tokens.rs` does: `cargo xtask baseline` floors Rust tests by name, and Node tests it does not (D-P3-8).

use std::path::{Path, PathBuf};

fn web_dir() -> PathBuf {
    // CARGO_MANIFEST_DIR = apps/web/engine
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Kind {
    Css,
    Script,
    Html,
    Json,
}

struct Source {
    name: String,
    kind: Kind,
    text: String,
}

/// The files the app is made of: `src` (TypeScript), `pages` (the page, its styles and its entry), `layout`.
/// Not `spikes`, `test`, `scripts`, `dist` or `engine`.
fn app_sources() -> Vec<Source> {
    let mut out = Vec::new();
    for dir in ["src", "pages", "layout"] {
        let Ok(entries) = std::fs::read_dir(web_dir().join(dir)) else { continue };
        for e in entries.flatten() {
            let p = e.path();
            let kind = match p.extension().and_then(|x| x.to_str()) {
                Some("css") => Kind::Css,
                Some("ts") => Kind::Script,
                Some("html") => Kind::Html,
                Some("json") => Kind::Json,
                _ => continue,
            };
            let name = format!("{dir}/{}", p.file_name().unwrap().to_string_lossy());
            if name.ends_with(".d.ts") {
                continue;
            }
            out.push(Source { name, kind, text: std::fs::read_to_string(&p).unwrap() });
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

// ---------------------------------------------------------------------------------------------
// Reading code without its comments (and, for numbers, without its strings)
// ---------------------------------------------------------------------------------------------

/// The text with comments removed. `keep_strings` false also blanks string literals, so words and digits inside
/// them are not mistaken for code.
fn code_of(text: &str, kind: Kind, keep_strings: bool) -> String {
    let b: Vec<char> = text.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    let line_comments = kind == Kind::Script;
    while i < b.len() {
        let c = b[i];
        let next = b.get(i + 1).copied().unwrap_or('\0');
        if kind != Kind::Json && c == '/' && next == '*' {
            while i + 1 < b.len() && !(b[i] == '*' && b[i + 1] == '/') {
                i += 1;
            }
            i += 2;
            out.push(' ');
        } else if kind == Kind::Html && c == '<' && next == '!' && b.get(i + 2) == Some(&'-') {
            while i + 2 < b.len() && !(b[i] == '-' && b[i + 1] == '-' && b[i + 2] == '>') {
                i += 1;
            }
            i += 3;
            out.push(' ');
        } else if line_comments && c == '/' && next == '/' && (i == 0 || b[i - 1] != ':') {
            while i < b.len() && b[i] != '\n' {
                i += 1;
            }
        } else if kind == Kind::Script && (c == '"' || c == '\'' || c == '`') {
            let q = c;
            let mut s = String::from(q);
            i += 1;
            while i < b.len() && b[i] != q {
                if b[i] == '\\' {
                    s.push(b[i]);
                    i += 1;
                }
                if i < b.len() {
                    s.push(b[i]);
                }
                i += 1;
            }
            s.push(q);
            i += 1;
            if keep_strings {
                out.push_str(&s);
            } else {
                out.push_str("\"\"");
            }
        } else {
            out.push(c);
            i += 1;
        }
    }
    out
}

fn words(code: &str) -> Vec<&str> {
    code.split(|c: char| !(c.is_alphanumeric() || c == '-' || c == '_')).filter(|w| !w.is_empty()).collect()
}

// ---------------------------------------------------------------------------------------------
// The rules. Each returns one line per violation, starting with the rule's tag.
// ---------------------------------------------------------------------------------------------

/// A length written as a number with a unit. `0` needs no unit and is not a length.
fn length_literals(code: &str) -> Vec<String> {
    let chars: Vec<char> = code.chars().collect();
    let mut found = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_ascii_digit() && (i == 0 || !(chars[i - 1].is_alphanumeric() || chars[i - 1] == '_' || chars[i - 1] == '-' || chars[i - 1] == '#')) {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                i += 1;
            }
            let number: String = chars[start..i].iter().collect();
            let unit: String = chars[i..].iter().take_while(|c| c.is_ascii_alphabetic()).collect();
            if ["px", "rem", "em", "pt", "pc", "cm", "mm", "in", "ch", "ex"].contains(&unit.as_str()) && number.parse::<f64>().is_ok_and(|n| n != 0.0) {
                found.push(format!("{number}{unit}"));
            }
        } else {
            i += 1;
        }
    }
    found
}

const NAMED_COLOURS: [&str; 34] = [
    "white", "black", "red", "green", "blue", "orange", "yellow", "gray", "grey", "silver", "purple", "pink", "brown", "cyan", "magenta", "gold", "navy", "teal",
    "maroon", "lime", "aqua", "olive", "indigo", "violet", "coral", "salmon", "tomato", "crimson", "ivory", "beige", "khaki", "tan", "lightgray", "darkgray",
];

fn has_hex_colour(code: &str) -> Option<String> {
    let b: Vec<char> = code.chars().collect();
    for i in 0..b.len() {
        if b[i] == '#' && (i == 0 || !(b[i - 1].is_alphanumeric() || b[i - 1] == '&')) {
            let run: String = b[i + 1..].iter().take_while(|c| c.is_ascii_alphanumeric()).collect();
            if [3, 4, 6, 8].contains(&run.len()) && run.chars().all(|c| c.is_ascii_hexdigit()) && run.chars().any(|c| c.is_ascii_digit()) {
                return Some(format!("#{run}"));
            }
            // A word that is all hex letters (#face, #bead) is a colour in a stylesheet and a word elsewhere; be strict.
            if [3, 4, 6, 8].contains(&run.len()) && run.chars().all(|c| c.is_ascii_hexdigit()) {
                return Some(format!("#{run}"));
            }
        }
    }
    None
}

fn declarations(css: &str) -> Vec<(String, String)> {
    // `prop: value;` pairs, wherever they are (inside rules and at-rules alike).
    let mut out = Vec::new();
    for chunk in css.split(['{', '}', ';']) {
        if let Some((p, v)) = chunk.split_once(':') {
            let p = p.trim();
            if !p.is_empty() && p.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
                out.push((p.to_string(), v.trim().to_string()));
            }
        }
    }
    out
}

fn violations(s: &Source) -> Vec<String> {
    let mut out = Vec::new();
    let with_strings = code_of(&s.text, s.kind, true);
    let no_strings = code_of(&s.text, s.kind, false);
    let lower = with_strings.to_lowercase();
    let mut bad = |tag: &str, what: String| out.push(format!("{tag} {}: {what}", s.name));

    // S1: no gradient.
    if lower.contains("gradient") {
        bad("S1", "a gradient (a gradient is not a material)".into());
    }
    // S2: no colour literal outside the tokens file.
    if let Some(hex) = has_hex_colour(&with_strings) {
        bad("S2", format!("the colour literal {hex}"));
    }
    for f in ["rgb(", "rgba(", "hsl(", "hsla(", "hwb(", "lab(", "lch(", "oklab(", "oklch(", "color-mix(", "color("] {
        if lower.match_indices(f).any(|(at, _)| at == 0 || !lower.as_bytes()[at - 1].is_ascii_alphanumeric() && lower.as_bytes()[at - 1] != b'-' && lower.as_bytes()[at - 1] != b'_') {
            bad("S2", format!("a colour function {f})"));
        }
    }
    if s.kind == Kind::Css {
        for (prop, value) in declarations(&with_strings) {
            if prop.starts_with("--") {
                continue;
            }
            for w in words(&value) {
                if NAMED_COLOURS.contains(&w.to_lowercase().as_str()) {
                    bad("S2", format!("the colour name {w} in {prop}"));
                }
            }
            // The face is the stylesheet's own: a custom property, or a keyword that means "the parent's".
            if prop == "font-family" && !(value.starts_with("var(") || value == "inherit") {
                bad("S2", format!("a font family written in the stylesheet: {value}"));
            }
            if prop == "font-size" && !value.starts_with("var(") && value != "inherit" {
                bad("S2", format!("a font size written in the stylesheet: {value}"));
            }
        }
    }
    // S2 for lengths: no length with a unit, outside the generated stylesheet and the tokens file.
    if s.kind != Kind::Json {
        for l in length_literals(&no_strings) {
            bad("S2", format!("the length {l} is written here, and lengths are the tokens' (space, type, stroke)"));
        }
        if s.kind == Kind::Script || s.kind == Kind::Html {
            for l in length_literals(&with_strings) {
                bad("S2", format!("the length {l} is written in a string"));
            }
        }
    } else {
        for l in length_literals(&s.text) {
            bad("S12", format!("the layout file holds a length, {l}"));
        }
    }
    // S4: no shadow.
    for w in ["box-shadow", "drop-shadow", "text-shadow"] {
        if lower.contains(w) {
            bad("S4", format!("{w}"));
        }
    }
    // S5: no all-caps or letter-spaced label styles.
    for w in ["text-transform", "letter-spacing", "uppercase"] {
        if lower.contains(w) {
            bad("S5", format!("{w}"));
        }
    }
    // S7: no monospace face for non-code.
    for w in ["monospace", "courier", "consolas", "menlo"] {
        if lower.contains(w) {
            bad("S7", format!("{w}"));
        }
    }
    // S9: no easing keyword and no cubic-bezier, anywhere.
    if lower.contains("cubic-bezier") {
        bad("S9", "cubic-bezier".into());
    }
    for w in words(&lower) {
        if ["ease", "ease-in", "ease-out", "ease-in-out"].contains(&w) {
            bad("S9", format!("the easing keyword {w}"));
        }
    }
    // S11: not the defaults.
    for face in ["Inter", "Google Sans", "SF Pro", "system-ui", "-apple-system", "BlinkMacSystemFont"] {
        if with_strings.contains(face) && (face != "Inter" || words(&with_strings).contains(&"Inter")) {
            bad("S11", format!("{face} as a face"));
        }
    }
    // S15: no radius but the round controls' 50%, or none at all. No SVG rx or ry (a rounded rectangle).
    if s.kind == Kind::Css {
        for (prop, value) in declarations(&with_strings) {
            if prop == "border-radius" && value != "50%" && value != "0" {
                bad("S15", format!("border-radius: {value}"));
            }
        }
    }
    if s.kind == Kind::Script {
        for w in ["borderRadius", "\"rx\"", "\"ry\"", "'rx'", "'ry'"] {
            if with_strings.contains(w) {
                bad("S15", format!("{w}"));
            }
        }
        // An object key or an assignment named rx or ry, as a whole word (`entry:` and `carry =` are not it).
        for w in ["rx", "ry"] {
            let chars: Vec<char> = with_strings.chars().collect();
            let pat: Vec<char> = w.chars().collect();
            for i in 0..chars.len().saturating_sub(1) {
                if chars[i..].starts_with(&pat) && (i == 0 || !(chars[i - 1].is_alphanumeric() || chars[i - 1] == '_')) {
                    let rest: String = chars[i + 2..].iter().skip_while(|c| c.is_whitespace()).take(1).collect();
                    let before_word_end = chars.get(i + 2).is_none_or(|c| !(c.is_alphanumeric() || *c == '_'));
                    if before_word_end && (rest == ":" || rest == "=") {
                        bad("S15", format!("{w} as a key or assignment (a rounded rectangle)"));
                    }
                }
            }
        }
    }
    // S16 (and S14): no transition; no animation but the LED flash.
    if lower.contains("transition") {
        bad("S16", "a transition".into());
    }
    let keyframes = lower.matches("@keyframes").count();
    if keyframes > 1 || (keyframes == 1 && !lower.contains("@keyframes led-flash")) {
        bad("S16", format!("{keyframes} @keyframes, and the only one allowed is led-flash"));
    }
    if s.kind == Kind::Css {
        for (prop, value) in declarations(&with_strings) {
            if (prop == "animation" || prop == "animation-name") && !value.starts_with("led-flash") {
                bad("S16", format!("{prop}: {value}"));
            }
            if prop.starts_with("animation") && prop != "animation" && prop != "animation-name" {
                bad("S16", format!("{prop}: set the animation in one declaration, `animation: led-flash …`"));
            }
        }
    }
    out
}

/// Numeric literals in the panel's drawing code: none beyond 0, 1 and 2 (halves and steps), so no pixel position is
/// written there (rule S12). Every length comes from layout.ts, which takes it from a token.
fn numeric_literals(s: &Source) -> Vec<String> {
    let code = code_of(&s.text, s.kind, false);
    let chars: Vec<char> = code.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_ascii_digit() && (i == 0 || !(chars[i - 1].is_alphanumeric() || chars[i - 1] == '_' || chars[i - 1] == '.')) {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '.' || chars[i] == '_') {
                i += 1;
            }
            let lit: String = chars[start..i].iter().collect();
            if !["0", "1", "2", "0n"].contains(&lit.as_str()) {
                out.push(format!("S12 {}: the number {lit} is written in panel code", s.name));
            }
        } else {
            i += 1;
        }
    }
    out
}

// ---------------------------------------------------------------------------------------------
// The app
// ---------------------------------------------------------------------------------------------

/// Files whose job is to be generic about where things go.
fn is_generator(name: &str) -> bool {
    name == "src/tokens-css.ts"
}

#[test]
fn the_scan_sees_the_files_the_app_is_made_of() {
    let names: Vec<String> = app_sources().into_iter().map(|s| s.name).collect();
    for want in [
        "pages/app.css",
        "pages/app.html",
        "pages/app.ts",
        "src/panel.ts",
        "src/layout.ts",
        "src/strip.ts",
        "src/tokens-css.ts",
        "layout/panel.layout.json",
    ] {
        assert!(names.iter().any(|n| n == want), "{want} is not in the scan: {names:?}");
    }
    assert!(!names.iter().any(|n| n.starts_with("test/") || n.starts_with("spikes/")), "tests and spikes are not the app");
}

#[test]
fn the_app_keeps_the_slop_rules_and_writes_no_colour_length_or_face_of_its_own() {
    let mut all = Vec::new();
    for s in app_sources() {
        all.extend(violations(&s));
    }
    assert!(all.is_empty(), "the app breaks rules of docs/05 section 4:\n{}", all.join("\n"));
}

#[test]
fn the_panel_code_writes_no_pixel_position() {
    let mut all = Vec::new();
    for s in app_sources() {
        if ["src/panel.ts", "pages/app.ts", "src/strip.ts"].contains(&s.name.as_str()) {
            all.extend(numeric_literals(&s));
        }
    }
    assert!(all.is_empty(), "{}", all.join("\n"));
}

#[test]
fn no_token_value_is_written_anywhere_else_in_the_app() {
    let tokens: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(web_dir().join("../../contracts/design.tokens.json")).unwrap()).unwrap();
    let mut hexes: Vec<String> = Vec::new();
    for (_, leaf) in tokens["colour"].as_object().unwrap() {
        hexes.push(leaf["value"].as_str().unwrap().to_lowercase());
    }
    let mut found = Vec::new();
    for s in app_sources() {
        if is_generator(&s.name) {
            continue;
        }
        let lower = s.text.to_lowercase();
        for h in &hexes {
            if lower.contains(h) {
                found.push(format!("{}: {h} is a colour token's value", s.name));
            }
        }
        let family = tokens["type"]["family"]["value"].as_str().unwrap().to_lowercase();
        if lower.contains(&family) {
            found.push(format!("{}: the type family of the tokens", s.name));
        }
    }
    assert!(found.is_empty(), "{}", found.join("\n"));
}

#[test]
fn the_stylesheets_use_only_custom_properties_that_exist() {
    // `--lit` is the stylesheet's own (the colour a key is lit in, which the flash keyframes read); every other
    // `var(--x)` must be a token property, whose names are the ones `tokens-css.ts` writes.
    let generator = app_sources().into_iter().find(|s| s.name == "src/tokens-css.ts").unwrap().text;
    for s in app_sources().into_iter().filter(|s| s.kind == Kind::Css) {
        let code = code_of(&s.text, Kind::Css, true);
        for part in code.split("var(--").skip(1) {
            let name: String = part.chars().take_while(|c| c.is_ascii_lowercase() || *c == '-').collect();
            let family = name.split('-').next().unwrap_or("");
            assert!(
                name == "lit" || ["colour", "space", "type", "stroke", "flash"].contains(&family),
                "{}: var(--{name}) is not a token property",
                s.name
            );
            assert!(name == "lit" || generator.contains(&format!("\"{family}-")) || generator.contains(&format!("`{family}-")), "{}: the generator writes no --{family}-… property", s.name);
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Each rule can fail
// ---------------------------------------------------------------------------------------------

fn css(text: &str) -> Source {
    Source { name: "x.css".into(), kind: Kind::Css, text: text.into() }
}
fn ts(text: &str) -> Source {
    Source { name: "x.ts".into(), kind: Kind::Script, text: text.into() }
}
fn caught(s: &Source, tag: &str) -> bool {
    violations(s).iter().any(|l| l.starts_with(tag))
}

#[test]
fn s1_a_gradient_is_caught() {
    for text in ["a { background: linear-gradient(red, blue); }", "a { background: RADIAL-GRADIENT(circle, #fff, #000); }", "a { background: conic-gradient(red, blue); }"] {
        assert!(caught(&css(text), "S1"), "{text}");
    }
    assert!(caught(&ts("const g = '<linearGradient id=\"a\"/>';"), "S1"));
    assert!(!caught(&css("a { fill: var(--colour-ink); }"), "S1"));
}

#[test]
fn s2_a_colour_a_length_or_a_face_written_outside_the_tokens_is_caught() {
    for text in [
        "a { color: #fff; }",
        "a { color: #1f1f1f; }",
        "a { color: #1f1f1f80; }",
        "a { color: #face; }",
        "a { color: rgb(1, 2, 3); }",
        "a { color: hsl(0 0% 0%); }",
        "a { color: oklch(0.5 0 0); }",
        "a { color: white; }",
        "a { border: 1px solid orange; }",
        "a { padding: 12px; }",
        "a { margin: 0.5rem; }",
        "a { font-size: 14px; }",
        "a { font-size: larger; }",
        "a { font-family: Georgia, serif; }",
    ] {
        assert!(caught(&css(text), "S2"), "{text}");
    }
    assert!(caught(&ts("el.style.color = '#ffffff';"), "S2"));
    assert!(caught(&ts("el.setAttribute('style', 'width: 20px');"), "S2"));
    for ok in [
        "a { color: var(--colour-ink); font-size: var(--type-size-body); }",
        "a { padding: 0; margin: 0 var(--space-2); }",
        "a { width: 100%; height: 100vh; }",
        "a { fill: none; stroke: currentColor; }",
        "a { font-family: inherit; font: inherit; }",
        "/* 12px and #fff in a comment are not code */ a { color: var(--colour-ink); }",
        "a { color: var(--colour-ink); } #start { margin: 0; }",
    ] {
        assert!(!caught(&css(ok), "S2"), "{ok}");
    }
    assert!(!caught(&ts("// a 20px gap, #fff\nconst id = '#panel'; const n = 'ab' + 1;"), "S2"));
}

#[test]
fn s4_a_shadow_is_caught() {
    for text in ["a { box-shadow: 0 0 var(--space-1) var(--colour-ink); }", "a { filter: drop-shadow(0 0 var(--space-1) var(--colour-ink)); }", "a { text-shadow: none; }"] {
        assert!(caught(&css(text), "S4"), "{text}");
    }
}

#[test]
fn s5_s7_s11_caps_monospace_and_default_faces_are_caught() {
    assert!(caught(&css("a { text-transform: uppercase; }"), "S5"));
    assert!(caught(&css("a { letter-spacing: var(--space-1); }"), "S5"));
    assert!(caught(&css("a { font-family: monospace; }"), "S7"));
    assert!(caught(&css("a { font-family: Menlo, var(--type-family); }"), "S7"));
    for face in ["Inter, sans-serif", "\"Google Sans\"", "SF Pro Text", "system-ui", "-apple-system", "BlinkMacSystemFont"] {
        assert!(caught(&css(&format!("a {{ font-family: {face}; }}")), "S11"), "{face}");
    }
    assert!(!caught(&ts("const interface = 1; const winter = 'Interval';"), "S11"), "Inter as a whole word only");
}

#[test]
fn s9_an_easing_keyword_is_caught() {
    for text in ["a { animation: led-flash 1s ease-in-out infinite; }", "a { animation: led-flash 1s cubic-bezier(0, 0, 1, 1); }", "a { animation-timing-function: ease; }", "a { x: ease-out; }"] {
        assert!(caught(&css(text), "S9"), "{text}");
    }
    assert!(!caught(&css("a { animation: led-flash var(--flash-period) steps(1, end) infinite; }"), "S9"));
    assert!(!caught(&ts("// this is easy, there is no ease here\nconst increase = 1;"), "S9"), "words that contain ease, and comments, are not keywords");
}

#[test]
fn s15_a_radius_other_than_the_round_controls_is_caught() {
    for text in ["a { border-radius: 4px; }", "a { border-radius: var(--space-1); }", "a { border-radius: 50% 0; }"] {
        assert!(caught(&css(text), "S15"), "{text}");
    }
    assert!(!caught(&css("a { border-radius: 50%; } b { border-radius: 0; }"), "S15"));
    assert!(caught(&ts("rect.setAttribute(\"rx\", r);"), "S15"));
    assert!(caught(&ts("el.style.borderRadius = x;"), "S15"));
    assert!(caught(&ts("const a = { rx: 4, cx: 1 };"), "S15"));
    assert!(caught(&ts("let ry = 3;"), "S15"));
    assert!(!caught(&ts("const entry: number = 1; let carry = 2; const retry = { ry2: 1 };"), "S15"), "rx and ry as whole words only");
}

#[test]
fn s16_a_transition_or_an_animation_other_than_the_led_flash_is_caught() {
    for text in [
        "a { transition: fill var(--flash-period); }",
        "a { animation: spin 1s infinite; }",
        "a { animation-name: pulse; }",
        "a { animation-duration: 1s; }",
        "@keyframes pulse { 0% { fill: none; } }",
        "@keyframes led-flash { 0% { fill: none; } } @keyframes other { 0% { fill: none; } }",
    ] {
        assert!(caught(&css(text), "S16"), "{text}");
    }
    assert!(!caught(&css("@keyframes led-flash { 0% { fill: var(--lit); } } a { animation: led-flash var(--flash-period) steps(1, end) infinite; }"), "S16"));
    assert!(caught(&ts("el.style.transition = 'none';"), "S16"));
}

#[test]
fn s12_a_pixel_position_written_in_panel_code_is_caught() {
    let found = |text: &str| !numeric_literals(&ts(text)).is_empty();
    assert!(found("circle.setAttribute('cx', 120);"));
    assert!(found("const x = col * 60;"));
    assert!(found("const r = 26.5;"));
    assert!(found("const big = 1_000;"));
    assert!(!found("const half = (n - 1) / 2; const first = i === 0; const dy = -1 * 2; const seed = 0n;"));
    assert!(!found("// 60 px in a comment\nconst label = 'a 60px string';"));
    let layout = Source { name: "layout/x.json".into(), kind: Kind::Json, text: "{\"gap\": \"12px\"}".into() };
    assert!(caught(&layout, "S12"), "a length in the layout file");
}
