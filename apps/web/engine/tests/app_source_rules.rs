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
            // The weights and the OpenType features belong to the face the contract names (ADR-0011 decision 5): a number or a tag written here is a second contract.
            if (prop == "font-weight" || prop == "font-feature-settings" || prop == "font-variation-settings") && !value.starts_with("var(") && value != "inherit" && value != "normal" {
                bad("S2", format!("{prop} written in the stylesheet: {value}"));
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
    // S15: no radius but the round controls' 50%, or none at all, or the control's one corner (`--radius-control`, ADR-0011 decision 6) in a rule of the
    // strip, and nowhere else. No SVG rx or ry (a rounded rectangle).
    if s.kind == Kind::Css {
        for (selector, body) in css_blocks(&with_strings) {
            for (prop, value) in declarations(&body) {
                if prop == "border-radius" && value != "50%" && value != "0" {
                    let the_strips = value == "var(--radius-control)" && selector.split(',').all(is_strip_selector);
                    if !the_strips {
                        bad("S15", format!("border-radius: {value} in `{}`", selector.trim()));
                    }
                }
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
    // S17: the six panel colours (ADR-0010) are the panel's. In a stylesheet only a rule whose every selector begins at #panel or #panel-frame
    // may use one; a script or the page names none (the stylesheet draws, the script does not).
    if s.kind == Kind::Css {
        for (selector, body) in css_blocks(&with_strings) {
            for role in panel_roles_named(&body) {
                for sel in selector.split(',') {
                    if !is_panel_selector(sel) {
                        bad("S17", format!("`{}` uses --{role}, which is the panel's: only a rule that starts at #panel or #panel-frame may", sel.trim()));
                    }
                }
            }
        }
    } else {
        for role in panel_roles_named(&with_strings) {
            bad("S17", format!("--{role} is written outside a stylesheet; the six panel colours are drawn by the stylesheet's #panel rules"));
        }
    }
    out
}

/// The six colour roles ADR-0010 adds, as custom properties. They are the plate's, the frame's and the keys', and nothing else's.
const PANEL_ROLES: [&str; 6] = ["colour-plate", "colour-frame", "colour-plate-ink", "colour-plate-ink-quiet", "colour-key", "colour-key-edge"];

/// The panel roles named in `text` as custom properties (`--colour-plate`, wherever it is written), each as a whole name:
/// `--colour-plate-ink` is one role and not `--colour-plate` followed by something.
fn panel_roles_named(text: &str) -> Vec<&'static str> {
    let mut out = Vec::new();
    for part in text.split("--").skip(1) {
        let name: String = part.chars().take_while(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '-').collect();
        if let Some(role) = PANEL_ROLES.iter().find(|r| **r == name) {
            out.push(*role);
        }
    }
    out
}

/// The innermost blocks of a stylesheet, as (selector, body): `a, b { x: y }` is one; in `@keyframes n { 0% { x: y } }` the inner
/// block is one and the at-rule is not.
fn css_blocks(css: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    // (prelude, where the body starts, whether a block opened inside it)
    let mut open: Vec<(String, usize, bool)> = Vec::new();
    let mut start = 0;
    for (i, c) in css.char_indices() {
        match c {
            '{' => {
                if let Some(parent) = open.last_mut() {
                    parent.2 = true;
                }
                open.push((css[start..i].trim().to_string(), i + 1, false));
                start = i + 1;
            }
            '}' => {
                if let Some((prelude, body_at, nested)) = open.pop() {
                    if !nested {
                        out.push((prelude, css[body_at..i].to_string()));
                    }
                }
                start = i + 1;
            }
            ';' => start = i + 1,
            _ => {}
        }
    }
    out
}

/// A selector that begins at the panel: its first compound is `#panel` or `#panel-frame` (with classes, attributes or pseudo-classes
/// after it), and not `#panelx` or `main #panel`.
fn is_panel_selector(selector: &str) -> bool {
    let first = selector.trim().split(|c: char| c.is_whitespace() || c == '>' || c == '+' || c == '~').next().unwrap_or("");
    let id: String = first.chars().take_while(|c| !matches!(c, '.' | '[' | ':')).collect();
    id == "#panel" || id == "#panel-frame"
}

/// A selector that begins at the strip: its first compound is `#strip`, with classes, attributes or pseudo-classes after it.
fn is_strip_selector(selector: &str) -> bool {
    let first = selector.trim().split(|c: char| c.is_whitespace() || c == '>' || c == '+' || c == '~').next().unwrap_or("");
    let id: String = first.chars().take_while(|c| !matches!(c, '.' | '[' | ':')).collect();
    id == "#strip"
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

/// Whether the code reads the address: the word `location` (`window.location`, `globalThis.location`, a bare `location.search`), outside
/// comments and strings.
fn reads_location(s: &Source) -> bool {
    words(&code_of(&s.text, s.kind, false)).contains(&"location")
}

#[test]
fn only_the_page_entry_reads_the_address() {
    // D-P4-2: the `?route=` stand-in is read in one place, so it can be found and removed when Track zoom exists.
    let readers: Vec<String> = app_sources().into_iter().filter(|s| s.kind == Kind::Script && reads_location(s)).map(|s| s.name).collect();
    assert_eq!(readers, vec!["pages/app.ts".to_string()], "the address is read in pages/app.ts and nowhere else");
}

#[test]
fn reading_the_address_is_caught_and_the_word_in_a_comment_or_a_string_is_not() {
    assert!(reads_location(&ts("const q = window.location.search;")));
    assert!(reads_location(&ts("const q = globalThis.location?.search;")));
    assert!(reads_location(&ts("const q = location.search;")));
    assert!(reads_location(&ts("const { search } = document.location;")));
    assert!(!reads_location(&ts("// the location of the file\nconst x = 1;")));
    assert!(!reads_location(&ts("const s = 'location.search';")));
    assert!(!reads_location(&ts("const relocation = 1; const locations = [];")), "a longer word is not the word");
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

/// The custom properties `tokens-css.ts` writes, from the tokens file. The names are the file's keys with `.` and `_`
/// made `-`, except the three the generator renames; `test/tokens-css.test.ts` pins the generator to the same list.
fn token_properties(tokens: &serde_json::Value) -> std::collections::BTreeSet<String> {
    let mut out = std::collections::BTreeSet::new();
    for (k, _) in tokens["colour"].as_object().unwrap() {
        out.insert(format!("colour-{}", k.replace(['.', '_'], "-")));
    }
    for (k, _) in tokens["space"].as_object().unwrap() {
        out.insert(format!("space-{k}"));
    }
    for name in [
        "radius-control",
        "type-family",
        "type-features",
        "type-weight-regular",
        "type-weight-strong",
        "type-size-label",
        "type-size-body",
        "type-measure",
        "stroke-hairline",
        "stroke-focus",
        "stroke-focus-inner",
        "flash-period",
        "flash-shine",
    ] {
        out.insert(name.to_string());
    }
    out
}

fn custom_properties_used(css: &str) -> Vec<String> {
    css.split("var(--").skip(1).map(|part| part.chars().take_while(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '-').collect()).collect()
}

#[test]
fn the_stylesheets_use_only_custom_properties_that_exist() {
    // `--lit` is the stylesheet's own (the colour a key is lit in, which the flash keyframes read) and `--panel-aspect` is the page's (the plate's
    // aspect, set by the script that draws the panel, which sizes the case); every other `var(--x)` must be a property the generated tokens.css defines.
    let tokens: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(web_dir().join("../../contracts/design.tokens.json")).unwrap()).unwrap();
    let known = token_properties(&tokens);
    let mut used_any = false;
    for s in app_sources().into_iter().filter(|s| s.kind == Kind::Css) {
        for name in custom_properties_used(&code_of(&s.text, Kind::Css, true)) {
            used_any = true;
            assert!(name == "lit" || name == "panel-aspect" || known.contains(&name), "{}: var(--{name}) is not a property of the tokens stylesheet", s.name);
        }
    }
    assert!(used_any, "the scan found no var() in the app's stylesheet");
}

#[test]
fn a_property_that_is_not_a_token_is_caught() {
    let tokens: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(web_dir().join("../../contracts/design.tokens.json")).unwrap()).unwrap();
    let known = token_properties(&tokens);
    assert!(known.contains("colour-led-green") && known.contains("colour-focus-inner") && known.contains("space-6") && known.contains("flash-period"));
    for name in custom_properties_used("a { color: var(--colour-brand); margin: var(--space-9); font: var(--type-size-huge); }") {
        assert!(!known.contains(&name), "{name} should be unknown");
    }
    assert_eq!(custom_properties_used("a { fill: var(--colour-led-off); } b { x: var(--lit) }"), vec!["colour-led-off", "lit"]);
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
        "a { font-weight: 700; }",
        "a { font-weight: bold; }",
        "a { font-feature-settings: \"cv01\", \"zero\"; }",
        "a { font-variation-settings: \"wght\" 510; }",
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
        "a { font-weight: var(--type-weight-strong); font-feature-settings: var(--type-features); } b { font-weight: inherit; font-feature-settings: normal; }",
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
    // ADR-0011 decision 6: the control's one corner, in the strip's rules and nowhere else.
    for text in [
        "a { border-radius: var(--radius-control); }",
        "#panel .face { border-radius: var(--radius-control); }",
        "#strip button, a { border-radius: var(--radius-control); }",
        "#stripx { border-radius: var(--radius-control); }",
        "main #strip { border-radius: var(--radius-control); }",
        "#strip button { border-radius: calc(var(--radius-control) * 2); }",
        "#strip button { border-radius: var(--radius-control) 0; }",
    ] {
        assert!(caught(&css(text), "S15"), "{text}");
    }
    assert!(!caught(&css("#strip button, #strip select, #strip input { border-radius: var(--radius-control); } #strip .field input { border-radius: var(--radius-control); }"), "S15"));
    assert!(is_strip_selector("#strip") && is_strip_selector(" #strip button") && is_strip_selector("#strip:focus-visible") && !is_strip_selector("#stripx") && !is_strip_selector("main #strip"));
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

#[test]
fn s17_a_panel_colour_outside_the_panel_is_caught() {
    for text in [
        "#strip button { background: var(--colour-plate); }",
        "html, body { background: var(--colour-frame); }",
        ".plate { fill: var(--colour-plate); }",
        "#panel, #strip { fill: var(--colour-key); }",
        "main #panel { fill: var(--colour-key-edge); }",
        "#panelx .a { fill: var(--colour-plate-ink); }",
        "#strip p { color: var(--colour-plate-ink-quiet); }",
        "@media (min-width: 0) { .x { fill: var(--colour-key); } }",
        "@keyframes led-flash { 0% { fill: var(--colour-plate); } }",
        ":root { --colour-plate: var(--lit); } a { fill: var(--colour-plate); }",
    ] {
        assert!(caught(&css(text), "S17"), "{text}");
    }
    assert!(caught(&ts("el.style.setProperty('fill', 'var(--colour-plate)');"), "S17"));
    assert!(caught(&ts("const x = '--colour-key-edge';"), "S17"));
    let page = Source { name: "x.html".into(), kind: Kind::Html, text: "<style>a { fill: var(--colour-frame) }</style>".into() };
    assert!(caught(&page, "S17"));
}

#[test]
fn s17_inside_the_panel_the_six_colours_are_allowed_and_elsewhere_the_other_roles_are() {
    for ok in [
        "#panel .plate { fill: var(--colour-plate); }",
        "#panel-frame { background: var(--colour-frame); padding: var(--space-4); }",
        "#panel .legend.caption, #panel .legend.row { fill: var(--colour-plate-ink-quiet); }",
        "#panel .control[data-down] .face { stroke: var(--colour-plate-ink); }",
        "#panel:focus-visible { stroke: var(--colour-key); }",
        "#panel > .face { stroke: var(--colour-key-edge); }",
        "@media (min-width: 0) { #panel .face { fill: var(--colour-key); } }",
        "#strip p { color: var(--colour-ink-quiet); } html, body { background: var(--colour-surface); }",
        "#strip button { background: var(--colour-ink); }",
    ] {
        assert!(!caught(&css(ok), "S17"), "{ok}");
    }
    // Whole names: the ink on the plate is not the plate, and a longer name that is no role is no role.
    assert_eq!(panel_roles_named("a { fill: var(--colour-plate-ink); stroke: var(--colour-platex); b: var(--colour-key-edge) }"), vec!["colour-plate-ink", "colour-key-edge"]);
    assert!(!caught(&ts("const i = 0; i--; const p = 'colour-plate';"), "S17"), "a decrement and a name without its dashes are not custom properties");
}

/// What in the app's files brings a drawing back into the ground: a module named for it, the call that draws it, an element or a rule called `#ground`, or an
/// SVG written in the page (the panel's is made by script). The owner, Tue 2026-10-13 12:52 London: "Let's remove the top shapes above they offer nothing"
/// (ADR-0011 decision 10, withdrawn). Comments are not code: they may say where the drawings were.
fn ground_drawings(sources: &[Source]) -> Vec<String> {
    let mut found = Vec::new();
    for s in sources {
        let code = code_of(&s.text, s.kind, true);
        if s.name.ends_with("ground.ts") {
            found.push(format!("{}: a module that draws the ground", s.name));
        }
        for needle in ["#ground", "id=\"ground\"", "drawGround"] {
            if code.contains(needle) {
                found.push(format!("{}: `{needle}`; the ground has no drawings", s.name));
            }
        }
        if s.kind == Kind::Html && code.contains("<svg") {
            found.push(format!("{}: an SVG written in the page; the only one on it is the panel's, made by script", s.name));
        }
    }
    found
}

#[test]
fn the_ground_has_no_drawings() {
    let found = ground_drawings(&app_sources());
    assert!(found.is_empty(), "{}", found.join("\n"));
}

#[test]
fn a_drawing_brought_back_into_the_ground_is_caught() {
    let named = |name: &str, kind: Kind, text: &str| Source { name: name.into(), kind, text: text.into() };
    for s in [
        named("src/ground.ts", Kind::Script, "export const x = 1;"),
        ts("import { drawGround } from './ground.ts';"),
        ts("stage.prepend(drawGround(document, tokens));"),
        css("#ground .line { fill: none; stroke: var(--colour-edge-control); }"),
        named("pages/app.html", Kind::Html, "<div id=\"stage\"><svg id=\"ground\"></svg></div>"),
        named("pages/app.html", Kind::Html, "<div id=\"stage\"><svg viewBox=\"0 0 1 1\"></svg></div>"),
    ] {
        assert!(!ground_drawings(&[s]).is_empty());
    }
    assert!(ground_drawings(&[css("/* the ground's drawings (#ground) were removed */ #strip { color: var(--colour-ink); }")]).is_empty(), "a comment is not a drawing");
    assert!(ground_drawings(&[ts("// drawGround was here\nconst x = 1;")]).is_empty(), "a comment is not a drawing");
    assert!(ground_drawings(&[named("pages/app.html", Kind::Html, "<div id=\"stage\"><div id=\"panel-frame\"></div></div>")]).is_empty());
    assert!(ground_drawings(&[css("html { background: var(--colour-surface); }")]).is_empty(), "the word is not the rule: a background is not the ground's drawings");
}

#[test]
fn the_blocks_of_a_stylesheet_are_read_as_the_rule_scan_needs() {
    let blocks = css_blocks("a, b { x: y; } @keyframes k { 0% { fill: red; } 50% { fill: none; } } @media m { c .d { e: f; } } g { h: i }");
    let selectors: Vec<&str> = blocks.iter().map(|(s, _)| s.as_str()).collect();
    assert_eq!(selectors, vec!["a, b", "0%", "50%", "c .d", "g"]);
    assert_eq!(blocks[0].1.trim(), "x: y;");
    assert!(is_panel_selector("#panel") && is_panel_selector(" #panel-frame ") && is_panel_selector("#panel .a > b") && is_panel_selector("#panel[data-x]"));
    assert!(!is_panel_selector("#panelx") && !is_panel_selector("main #panel") && !is_panel_selector(".panel") && !is_panel_selector("panel"));
}

/// The panel's rules in the page stylesheet: the bodies of every block whose selectors all begin at the panel.
fn panel_bodies() -> Vec<(String, String)> {
    let text = std::fs::read_to_string(web_dir().join("pages/app.css")).unwrap();
    css_blocks(&code_of(&text, Kind::Css, true)).into_iter().filter(|(sel, _)| sel.split(',').all(is_panel_selector)).collect()
}

#[test]
fn the_panel_is_drawn_in_its_own_six_colours_and_each_is_used() {
    let bodies = panel_bodies();
    assert!(bodies.len() >= 8, "the panel has {} rules of its own", bodies.len());
    for role in PANEL_ROLES {
        assert!(bodies.iter().any(|(_, b)| panel_roles_named(b).contains(&role)), "no #panel rule uses --{role}");
    }
}

/// What the strip and the page own (ADR-0011 decision 3): the panel's rules name none of these. `flash-shine` is the strip's ink under another name.
const STRIP_OWN: [&str; 7] = ["colour-surface", "colour-ink", "colour-ink-quiet", "colour-surface-raised", "colour-edge", "colour-edge-control", "flash-shine"];

/// The strip's custom properties named in the bodies of the rules that begin at the panel.
fn strip_properties_in_panel_rules(css: &str) -> Vec<String> {
    let mut found = Vec::new();
    for (sel, body) in css_blocks(&code_of(css, Kind::Css, true)) {
        if !sel.split(',').all(is_panel_selector) {
            continue;
        }
        for name in custom_properties_used(&body) {
            if STRIP_OWN.contains(&name.as_str()) {
                found.push(format!("`{sel}` uses --{name}, which is the strip's"));
            }
        }
    }
    found
}

#[test]
fn the_panel_borrows_nothing_from_the_strip() {
    // docs/05 section 1: two worlds. The strip's ground, tones and ink are not the plate's; the shine is told in the plate's ink (flash.shine names `ink`, the strip's).
    let text = std::fs::read_to_string(web_dir().join("pages/app.css")).unwrap();
    let found = strip_properties_in_panel_rules(&text);
    assert!(found.is_empty(), "{}", found.join("\n"));
}

#[test]
fn a_strip_tone_in_a_panel_rule_is_caught() {
    for text in [
        "#panel .face { stroke: var(--colour-edge); }",
        "#panel-frame { background: var(--colour-surface-raised); }",
        "#panel .label { fill: var(--colour-edge-control); }",
        "#panel .led { stroke: var(--flash-shine); }",
    ] {
        assert_eq!(strip_properties_in_panel_rules(text).len(), 1, "{text}");
    }
    assert!(strip_properties_in_panel_rules("#strip button { border: var(--stroke-hairline) solid var(--colour-edge-control); } #panel .face { stroke: var(--colour-key-edge); }").is_empty());
}

#[test]
fn the_strip_and_the_page_are_not_drawn_in_the_panel_colours() {
    // The converse of the above, over the real stylesheet: S17 already refuses it, and this names the strip so a failure says so.
    let text = std::fs::read_to_string(web_dir().join("pages/app.css")).unwrap();
    for (sel, body) in css_blocks(&code_of(&text, Kind::Css, true)) {
        if sel.contains("#strip") || sel.contains("html") || sel.contains("body") || sel.contains("main") {
            assert!(panel_roles_named(&body).is_empty(), "`{sel}` uses a panel colour");
        }
    }
}

#[test]
fn the_panels_parts_are_styled_only_under_the_panel() {
    // The classes panel.ts gives the things it draws. A rule that names one without starting at #panel would style the panel from outside it, past S17 and the test above.
    const PARTS: [&str; 15] = ["frame", "plate", "face", "nut", "ball", "legend", "led", "dot", "label", "ring-inner", "ring-outer", "control", "knob", "inert", "key"];
    let text = std::fs::read_to_string(web_dir().join("pages/app.css")).unwrap();
    let mut found = Vec::new();
    for (sel, _) in css_blocks(&code_of(&text, Kind::Css, true)) {
        for one in sel.split(',') {
            let classes = one.split('.').skip(1).map(|c| c.chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '-').collect::<String>());
            if classes.into_iter().any(|c| PARTS.contains(&c.as_str())) && !is_panel_selector(one) {
                found.push(format!("`{}` styles a part of the panel from outside #panel", one.trim()));
            }
        }
    }
    assert!(found.is_empty(), "{}", found.join("\n"));
}
