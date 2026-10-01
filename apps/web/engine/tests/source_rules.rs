//! Rules about the crate's own files, so they cannot drift (ADR-0008 decisions 1 and 3).

use std::path::{Path, PathBuf};

fn crate_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn sources() -> Vec<(String, String)> {
    let mut out = Vec::new();
    for e in std::fs::read_dir(crate_dir().join("src")).expect("src").flatten() {
        let p = e.path();
        if p.extension().is_some_and(|x| x == "rs") {
            out.push((p.file_name().unwrap().to_string_lossy().into_owned(), std::fs::read_to_string(&p).unwrap()));
        }
    }
    out.sort();
    out
}

/// The word `unsafe` in code, ignoring comments and the lint name `unsafe_code`.
fn has_unsafe_keyword(text: &str) -> bool {
    text.lines()
        .map(|l| l.split("//").next().unwrap_or(""))
        .any(|code| code.split(|c: char| !(c.is_alphanumeric() || c == '_')).any(|w| w == "unsafe"))
}

#[test]
fn the_unsafe_keyword_appears_only_in_the_counting_allocator() {
    let found: Vec<String> = sources().into_iter().filter(|(_, t)| has_unsafe_keyword(t)).map(|(n, _)| n).collect();
    assert_eq!(found, vec!["measure.rs".to_string()], "ADR-0008 decision 3: only measure.rs may hold `unsafe`");
}

#[test]
fn the_lint_is_allowed_in_exactly_the_export_shims_and_the_allocator() {
    let allowing: Vec<String> = sources().into_iter().filter(|(_, t)| t.contains("allow(unsafe_code)")).map(|(n, _)| n).collect();
    assert_eq!(allowing, vec!["exports.rs".to_string(), "measure.rs".to_string()]);
    let lib = std::fs::read_to_string(crate_dir().join("src/lib.rs")).unwrap();
    assert!(lib.contains("#![deny(unsafe_code)]"), "the crate root must deny the lint");
}

#[test]
fn the_abi_document_lists_every_export_and_nothing_else() {
    let exports = sources().into_iter().find(|(n, _)| n == "exports.rs").expect("exports.rs").1;
    let mut in_code: Vec<String> = exports
        .lines()
        .filter_map(|l| l.trim().strip_prefix("pub extern \"C\" fn "))
        .map(|rest| rest.split('(').next().unwrap().to_string())
        .collect();
    in_code.sort();
    assert!(!in_code.is_empty());
    let doc = std::fs::read_to_string(crate_dir().join("ABI.md")).expect("apps/web/engine/ABI.md");
    let mut in_doc: Vec<String> = doc
        .lines()
        .filter_map(|l| l.trim().strip_prefix("| `octoweb_"))
        .map(|rest| format!("octoweb_{}", rest.split('`').next().unwrap().split('(').next().unwrap()))
        .collect();
    in_doc.sort();
    assert_eq!(in_code, in_doc, "ABI.md and exports.rs list different functions");
}

#[test]
fn every_export_is_a_one_line_shim() {
    let exports = std::fs::read_to_string(crate_dir().join("src/exports.rs")).unwrap();
    for (i, line) in exports.lines().enumerate() {
        if line.trim_start().starts_with("pub extern \"C\" fn ") {
            assert!(line.trim_end().ends_with('}'), "exports.rs line {}: an export is one line that calls safe code: {line}", i + 1);
        }
    }
}
