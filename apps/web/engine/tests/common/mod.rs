//! The page's view of the module: the exported functions only, and the bytes they hand back.
//! Everything here reads memory through the pointers the module reports, as the worklet does.

#![allow(dead_code)]

use octoweb::exports::*;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const EVENT_BYTES: usize = 12;

pub fn repo_root() -> PathBuf {
    // CARGO_MANIFEST_DIR = apps/web/engine
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

/// `id` to `n` for every control in `contracts/controls.json`.
pub fn controls() -> BTreeMap<String, u32> {
    let path = repo_root().join("contracts/controls.json");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
    let doc: serde_json::Value = serde_json::from_str(&text).expect("controls.json is JSON");
    doc["controls"]
        .as_array()
        .expect("controls is an array")
        .iter()
        .map(|c| (c["id"].as_str().expect("id").to_string(), c["n"].as_u64().expect("n") as u32))
        .collect()
}

/// The layout text the page sends at start: one `n id` per line, minus the ids in `without`.
pub fn layout_text(without: &[&str]) -> String {
    controls().iter().filter(|(id, _)| !without.contains(&id.as_str())).map(|(id, n)| format!("{n} {id}\n")).collect()
}

pub fn write_scratch(bytes: &[u8]) {
    let p = octoweb_scratch(bytes.len() as u32);
    assert!(!p.is_null(), "octoweb_scratch gave no memory");
    // SAFETY: the module just made `bytes.len()` bytes available at `p`, and nothing else uses them.
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), p, bytes.len()) };
}

pub fn read_scratch(len: usize) -> Vec<u8> {
    let p = octoweb_scratch(len as u32);
    assert!(!p.is_null());
    // SAFETY: `len` bytes were made available at `p` by the call above.
    unsafe { std::slice::from_raw_parts(p, len).to_vec() }
}

fn read(p: *const u8, len: usize) -> Vec<u8> {
    assert!(!p.is_null(), "the module gave a null pointer");
    // SAFETY: the module documents each buffer's length (ABI.md) and keeps it for its lifetime.
    unsafe { std::slice::from_raw_parts(p, len).to_vec() }
}

/// What `octoweb_init` returned, and the message it left in scratch if it refused.
pub struct InitResult {
    pub code: u32,
    pub message: String,
}

pub fn try_init(layout: &str, sample_rate: f32, seed: u64) -> InitResult {
    write_scratch(layout.as_bytes());
    let code = octoweb_init(sample_rate, seed as u32, (seed >> 32) as u32, layout.len() as u32);
    let message = String::from_utf8_lossy(&read_scratch(octoweb_message_len() as usize)).into_owned();
    InitResult { code, message }
}

pub struct Abi {
    pub n: BTreeMap<String, u32>,
    pub sample_rate: f32,
}

impl Abi {
    pub fn start() -> Abi {
        Abi::start_with(48_000.0, 0)
    }

    pub fn start_with(sample_rate: f32, seed: u64) -> Abi {
        let r = try_init(&layout_text(&[]), sample_rate, seed);
        assert_eq!(r.code, 0, "init failed: {}", r.message);
        Abi { n: controls(), sample_rate }
    }

    pub fn control(&self, id: &str) -> u32 {
        *self.n.get(id).unwrap_or_else(|| panic!("`{id}` is not in controls.json"))
    }

    pub fn matrix(&self, row: usize, step: usize) -> String {
        format!("matrix.r{row}.c{}", step + 1)
    }

    /// A click: down at `t`, up 50 ms later.
    pub fn click(&self, id: &str, t: f64) {
        let c = self.control(id);
        assert_eq!(octoweb_input(t, 0, c, 0), 0);
        assert_eq!(octoweb_input(t + 50.0, 1, c, 0), 0);
    }

    pub fn led(&self, id: &str) -> u8 {
        octoweb_refresh_leds();
        read(octoweb_leds(), 512)[self.control(id) as usize]
    }

    pub fn leds(&self) -> Vec<u8> {
        octoweb_refresh_leds();
        read(octoweb_leds(), 512)
    }

    pub fn playheads(&self) -> [u8; 10] {
        read(octoweb_playheads(), 10).try_into().unwrap()
    }

    /// Renders one block and returns its events as the 12-byte records the page reads.
    pub fn render(&self, frames: u32) -> Vec<[u8; EVENT_BYTES]> {
        let n = octoweb_render(frames) as usize;
        let bytes = read(octoweb_events(), n * EVENT_BYTES);
        bytes.chunks(EVENT_BYTES).map(|c| c.try_into().unwrap()).collect()
    }
}

/// The wire form of an engine event, written here from the layout in `ABI.md` and not taken from
/// the crate: `[kind, port, channel, d1, d2 low, d2 high, 0, 0, at_sample as u32, little endian]`.
pub fn wire(e: &octocore::Event) -> [u8; EVENT_BYTES] {
    use octocore::Event::*;
    let (kind, port, ch, d1, d2, at): (u8, u8, u8, u8, u16, u32) = match *e {
        NoteOn { port, ch, note, vel, at_sample } => (0, port, ch, note, vel as u16, at_sample),
        NoteOff { port, ch, note, at_sample } => (1, port, ch, note, 0, at_sample),
        Cc { port, ch, cc, val, at_sample } => (2, port, ch, cc, val as u16, at_sample),
        PitchBend { port, ch, value, at_sample } => (3, port, ch, 0, value, at_sample),
        ChannelPressure { port, ch, value, at_sample } => (4, port, ch, value, 0, at_sample),
    };
    let mut b = [0u8; EVENT_BYTES];
    b[0] = kind;
    b[1] = port;
    b[2] = ch;
    b[3] = d1;
    b[4..6].copy_from_slice(&d2.to_le_bytes());
    b[8..12].copy_from_slice(&at.to_le_bytes());
    b
}
