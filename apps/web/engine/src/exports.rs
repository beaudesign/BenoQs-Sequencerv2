//! The export shims. One line each, calling safe code in `host`. See `ABI.md`.
#![allow(unsafe_code)] // `#[no_mangle]` only; this file holds no `unsafe` keyword (tests/source_rules.rs)

use crate::host;

#[no_mangle]
pub extern "C" fn octoweb_abi() -> u32 { host::ABI_VERSION }
#[no_mangle]
pub extern "C" fn octoweb_scratch(len: u32) -> *mut u8 { host::scratch(len as usize) }
#[no_mangle]
pub extern "C" fn octoweb_init(sample_rate: f32, seed_lo: u32, seed_hi: u32, layout_len: u32) -> u32 { host::init(sample_rate, host::seed(seed_lo, seed_hi), layout_len as usize) }
#[no_mangle]
pub extern "C" fn octoweb_message_len() -> u32 { host::message_len() as u32 }
#[no_mangle]
pub extern "C" fn octoweb_reset() -> u32 { host::reset() }
#[no_mangle]
pub extern "C" fn octoweb_input(now_ms: f64, kind: u32, control: u32, detents: i32) -> u32 { host::input(now_ms, kind, control, detents) }
#[no_mangle]
pub extern "C" fn octoweb_transport(play: u32) -> u32 { host::transport(play != 0) }
#[no_mangle]
pub extern "C" fn octoweb_set_tempo(bpm: f32) -> u32 { host::set_tempo(bpm) }
#[no_mangle]
pub extern "C" fn octoweb_set_track(track: u32, attr: u32, value: i32) -> u32 { host::set_track(track, attr, value) }
#[no_mangle]
pub extern "C" fn octoweb_render(frames: u32) -> u32 { host::render(frames) }
#[no_mangle]
pub extern "C" fn octoweb_events() -> *const u8 { host::events_ptr() }
#[no_mangle]
pub extern "C" fn octoweb_refresh_leds() -> u32 { host::refresh_leds() }
#[no_mangle]
pub extern "C" fn octoweb_leds() -> *const u8 { host::leds_ptr() }
#[no_mangle]
pub extern "C" fn octoweb_playheads() -> *const u8 { host::playheads_ptr() }
#[no_mangle]
pub extern "C" fn octoweb_status() -> u32 { host::status() }
#[no_mangle]
pub extern "C" fn octoweb_dropped_intents() -> u32 { host::dropped_intents() }
#[cfg(feature = "measure")]
#[no_mangle]
pub extern "C" fn octoweb_alloc_count() -> u32 { crate::measure::allocations() }
#[cfg(feature = "spike")]
#[no_mangle]
pub extern "C" fn octoweb_spike_run(len: u32) -> u32 { crate::spike::run(len as usize) }
