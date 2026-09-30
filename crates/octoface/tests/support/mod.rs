//! Shared by the panel tests: the repository root, the control inventory read from
//! `contracts/controls.json`, and the list of panel fixture files.

#![allow(dead_code)]

use octoface::Layout;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub fn repo_root() -> PathBuf {
    // CARGO_MANIFEST_DIR = crates/octoface
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub fn read_json(rel: &str) -> serde_json::Value {
    let path = repo_root().join(rel);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{} is not JSON: {e}", path.display()))
}

/// The inventory, as the controller and the fixture runner use it.
pub struct Inventory {
    pub layout: Layout,
    /// `id` to `n`, for every control in the file.
    pub numbers: BTreeMap<String, u32>,
}

pub fn inventory() -> Inventory {
    let doc = read_json("contracts/controls.json");
    let controls = doc["controls"].as_array().expect("controls is an array");
    let pairs: Vec<(u32, String)> =
        controls.iter().map(|c| (c["n"].as_u64().expect("n") as u32, c["id"].as_str().expect("id").to_string())).collect();
    let layout = Layout::from_pairs(pairs.iter().map(|(n, id)| (*n, id.as_str()))).expect("the inventory builds a layout");
    let numbers = pairs.into_iter().map(|(n, id)| (id, n)).collect();
    Inventory { layout, numbers }
}

/// Every `.panel` file under `tests/conformance/panel`, split into asserting and pending.
pub fn panel_files() -> (Vec<PathBuf>, Vec<PathBuf>) {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().is_some_and(|x| x == "panel") {
                out.push(p);
            }
        }
    }
    let panel_root = repo_root().join("tests/conformance/panel");
    let mut all = Vec::new();
    walk(&panel_root, &mut all);
    all.sort();
    // `pending` is a directory below the panel root, not anywhere in the absolute path.
    all.into_iter().partition(|p| !p.strip_prefix(&panel_root).unwrap_or(p).components().any(|c| c.as_os_str() == "pending"))
}
