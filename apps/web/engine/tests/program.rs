//! The programs in `golden/programs.json`, played through the module's exported functions, give the
//! digests the file pins. The Node test and spike S2's AudioWorklet play the same programs and must
//! get the same digests: that is what makes "the worklet plays what the native engine plays" a
//! statement with bytes in it.

mod common;

use common::*;
use octoweb::exports::*;

/// FNV-1a, 64 bits: small enough to write the same way in Rust and in TypeScript.
fn fnv1a(hash: &mut u64, bytes: &[u8]) {
    for b in bytes {
        *hash ^= *b as u64;
        *hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
}

fn digest_of(name: &str) -> (String, usize) {
    let text = std::fs::read_to_string(repo_root().join("apps/web/engine/golden/programs.json")).unwrap();
    let doc: serde_json::Value = serde_json::from_str(&text).unwrap();
    let rate = doc["sample_rate"].as_f64().unwrap() as f32;
    let frames = doc["frames_per_block"].as_u64().unwrap() as u32;
    let gap = doc["click_gap_ms"].as_f64().unwrap();
    let p = &doc["programs"][name];
    let abi = Abi::start_with(rate, p["seed"].as_u64().unwrap());
    for (i, s) in p["steps"].as_array().unwrap().iter().enumerate() {
        let (row, step) = (s[0].as_u64().unwrap() as usize, s[1].as_u64().unwrap() as usize);
        abi.click(&abi.matrix(row, step), i as f64 * gap);
    }
    assert_eq!(octoweb_set_tempo(p["bpm"].as_f64().unwrap() as f32), 0);
    assert_eq!(octoweb_transport(1), 0);
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    let mut events = 0;
    for block in 0..p["blocks"].as_u64().unwrap() as u32 {
        for e in abi.render(frames) {
            fnv1a(&mut hash, &block.to_le_bytes());
            fnv1a(&mut hash, &e);
            events += 1;
        }
    }
    (format!("fnv1a64:{hash:016x}"), events)
}

fn pinned(name: &str) -> String {
    let text = std::fs::read_to_string(repo_root().join("apps/web/engine/golden/programs.json")).unwrap();
    let doc: serde_json::Value = serde_json::from_str(&text).unwrap();
    doc["programs"][name]["digest"].as_str().unwrap().to_string()
}

#[test]
fn the_pressed_program_gives_its_pinned_digest() {
    let (got, events) = digest_of("pressed");
    assert!(events >= 20, "the program should be heard ({events} events)");
    assert_eq!(got, pinned("pressed"), "{events} events");
}

#[test]
fn the_dense_program_gives_its_pinned_digest() {
    let (got, events) = digest_of("dense");
    assert!(events >= 500, "the program should be dense ({events} events)");
    assert_eq!(got, pinned("dense"), "{events} events");
}
