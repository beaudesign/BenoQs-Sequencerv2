//! `octoffi` — the C ABI boundary over `octocore`. Owner: Conductor
//! (`docs/08-agent-operating-model.md` §2: `crates/octoffi/` is one of the few
//! paths Conductor holds directly, alongside `contracts/`, rather than handing to
//! a build role, because an FFI boundary is exactly the kind of narrow, frozen
//! interface the whole multi-agent model depends on staying stable).
//!
//! Everything here is a thin, allocation-light wrapper: no logic lives in this
//! crate, only pointer/lifetime bookkeeping and the C-callable entry points Swift
//! (`octopanel`/`octoshell`) will eventually link against via `octoffi.h`
//! (hand-written alongside this file, not yet run through `cbindgen` — that's
//! not installed in this dev environment; the header must be kept in sync by
//! hand until it is).
//!
//! Every function takes a raw `*mut Engine` obtained from `octocore_engine_new`
//! and must not be called with a pointer obtained any other way, after
//! `octocore_engine_free`, or from more than one thread at a time (the engine has
//! no internal synchronisation — docs/03-sequencer-core.md's timing budget forbids
//! locks on the audio thread, so the *caller* owns the single-writer discipline,
//! same as any real-time audio engine's public C API).

use octocore::{Command, Engine, Event};

/// Opaque handle. Swift/C never sees the real `Engine` layout.
#[no_mangle]
pub extern "C" fn octocore_engine_new(seed: u64) -> *mut Engine {
    Box::into_raw(Box::new(Engine::new(seed)))
}

/// # Safety
/// `engine` must be a pointer returned by `octocore_engine_new` and not yet
/// freed. Passing anything else is undefined behaviour, same as `free()`.
#[no_mangle]
pub unsafe extern "C" fn octocore_engine_free(engine: *mut Engine) {
    if !engine.is_null() {
        drop(Box::from_raw(engine));
    }
}

/// # Safety
/// `engine` must be a live pointer from `octocore_engine_new`.
#[no_mangle]
pub unsafe extern "C" fn octocore_engine_handle_command(engine: *mut Engine, cmd: Command) {
    let Some(engine) = engine.as_mut() else { return };
    engine.handle_command(cmd);
}

#[repr(C)]
pub struct OctoRenderParams {
    pub sample_rate: f32,
    pub buffer_len: u32,
    pub bpm: f32,
    pub playing: bool,
}

/// Renders exactly `params.buffer_len` samples and writes up to
/// `out_capacity` emitted `Event`s into `out_events` (caller-owned buffer),
/// writing the actual count into `*out_count`. Returns 0 on success, a negative
/// code on a null/invalid argument. If more events are due than `out_capacity`
/// can hold, the extras are kept and come out first in the next call (late, at
/// sample 0), never dropped; `octocore_engine_diagnostics` counts how often that
/// happened. Capacity beyond `MAX_EVENTS_PER_TICK` (256) is not used.
///
/// # Safety
/// `engine` must be a live pointer from `octocore_engine_new`. `out_events` must
/// point to at least `out_capacity` valid, writable `Event` slots. `out_count`
/// must point to one valid, writable `usize`.
#[no_mangle]
pub unsafe extern "C" fn octocore_engine_render(
    engine: *mut Engine,
    params: OctoRenderParams,
    out_events: *mut Event,
    out_capacity: usize,
    out_count: *mut usize,
) -> i32 {
    let Some(engine) = engine.as_mut() else { return -1 };
    if out_events.is_null() || out_count.is_null() {
        return -2;
    }

    let ctx = octocore::engine::RenderContext {
        sample_rate: params.sample_rate,
        buffer_len: params.buffer_len,
        bpm: params.bpm,
        playing: params.playing,
    };
    let mut buf = octocore::engine::EventBuffer::with_limit(out_capacity);
    engine.render(&ctx, &mut buf);

    let events = buf.as_slice();
    let n = events.len().min(out_capacity);
    let dst = std::slice::from_raw_parts_mut(out_events, n);
    dst.copy_from_slice(&events[..n]);
    *out_count = n;
    0
}

/// Copies the engine's health counters to `*out`. Returns 0 on success, -1 for a null
/// engine, -2 for a null `out`. The counters are cumulative since `octocore_engine_new`.
///
/// # Safety
/// `engine` must be a live pointer from `octocore_engine_new`. `out` must point to one
/// valid, writable `Diagnostics`.
#[no_mangle]
pub unsafe extern "C" fn octocore_engine_diagnostics(engine: *const Engine, out: *mut octocore::Diagnostics) -> i32 {
    let Some(engine) = engine.as_ref() else { return -1 };
    let Some(out) = out.as_mut() else { return -2 };
    *out = engine.diagnostics();
    0
}

/// # Safety
/// `engine` must be a live pointer from `octocore_engine_new`.
#[no_mangle]
pub unsafe extern "C" fn octocore_engine_is_running(engine: *const Engine) -> bool {
    match engine.as_ref() {
        Some(engine) => engine.is_running(),
        None => false,
    }
}

// --- Grid-mutation surface ---
//
// This doesn't need panel.truth.json's ControlId scheme at all — that's for
// *physical actuation* events (a button/encoder somewhere on the panel),
// which this crate has no coordinate system for yet. Programming a pattern
// is a purely logical operation (track N, step M, this attribute, this
// value) that a test harness, a Swift data-entry UI, or anything else can
// perform without knowing where a control lives on the panel. Everything
// crosses as `i32`: it covers every field type here (`u8`/`i8`/`bool`)
// without precision loss, and keeps the C surface to one setter/one getter
// per (Track|Step) rather than one function per field.

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OctoTrackAttr {
    Pitch,
    Velocity,
    LengthFactor,
    StartFactor,
    DirectionRaw,
    Rotation,
    Amount,
    Groove,
    MidiChannel,
    Muted,
    Soloed,
    Paused,
    RecordArmed,
    IsFeeder,
    IsListener,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OctoStepAttr {
    Active,
    Skip,
    PitchOffset,
    VelocityOffset,
    LengthTicks,
    LengthMultiplier,
    StartOffset,
    Amount,
    Strum,
    Hyperstep,
    /// 0 = no phrase (`None`); 1..=48 indexes `Grid::phrases` (1-based).
    Phrase,
    /// Phrase time-compression; 8 is neutral (CE v5.30 p.17). Stored only.
    PhrasePos,
}

/// Returns `false` (and does nothing / returns 0) for an out-of-range
/// `track`/`step` index rather than panicking — the same "caller mistake is
/// a no-op, not a crash" contract as the rest of this boundary.
fn track_in_range(track: u8) -> bool {
    (track as usize) < octocore::domain::TRACK_COUNT
}
fn step_in_range(step: u8) -> bool {
    (step as usize) < octocore::domain::STEP_COUNT
}

/// # Safety
/// `engine` must be a live pointer from `octocore_engine_new`.
#[no_mangle]
pub unsafe extern "C" fn octocore_track_set_i32(engine: *mut Engine, track: u8, attr: OctoTrackAttr, value: i32) -> bool {
    let Some(engine) = engine.as_mut() else { return false };
    if !track_in_range(track) {
        return false;
    }
    let t = &mut engine.grid.active_page_mut().tracks[track as usize];
    match attr {
        OctoTrackAttr::Pitch => t.pitch = value.clamp(0, 127) as u8,
        OctoTrackAttr::Velocity => t.velocity = value.clamp(0, 127) as u8,
        OctoTrackAttr::LengthFactor => t.length_factor = value.clamp(0, 16) as u8,
        OctoTrackAttr::StartFactor => t.start_factor = value.clamp(0, 16) as u8,
        OctoTrackAttr::DirectionRaw => t.direction_raw = value.clamp(1, 16) as u8,
        OctoTrackAttr::Rotation => t.rotation = value.clamp(0, 255) as u8,
        OctoTrackAttr::Amount => t.amount = value.clamp(-128, 127) as i8,
        OctoTrackAttr::Groove => t.groove = value.clamp(0, 16) as u8,
        OctoTrackAttr::MidiChannel => t.midi_channel = value.clamp(1, 32) as u8,
        OctoTrackAttr::Muted => t.muted = value != 0,
        OctoTrackAttr::Soloed => t.soloed = value != 0,
        OctoTrackAttr::Paused => t.paused = value != 0,
        OctoTrackAttr::RecordArmed => t.record_armed = value != 0,
        OctoTrackAttr::IsFeeder => t.is_feeder = value != 0,
        OctoTrackAttr::IsListener => t.is_listener = value != 0,
    }
    true
}

/// # Safety
/// `engine` must be a live pointer from `octocore_engine_new`. Returns 0 for
/// an out-of-range `track` — indistinguishable from a real 0 value; callers
/// that need to tell these apart should validate `track` themselves first
/// (it's always `< 10`).
#[no_mangle]
pub unsafe extern "C" fn octocore_track_get_i32(engine: *const Engine, track: u8, attr: OctoTrackAttr) -> i32 {
    let Some(engine) = engine.as_ref() else { return 0 };
    if !track_in_range(track) {
        return 0;
    }
    let t = &engine.grid.active_page().tracks[track as usize];
    match attr {
        OctoTrackAttr::Pitch => t.pitch as i32,
        OctoTrackAttr::Velocity => t.velocity as i32,
        OctoTrackAttr::LengthFactor => t.length_factor as i32,
        OctoTrackAttr::StartFactor => t.start_factor as i32,
        OctoTrackAttr::DirectionRaw => t.direction_raw as i32,
        OctoTrackAttr::Rotation => t.rotation as i32,
        OctoTrackAttr::Amount => t.amount as i32,
        OctoTrackAttr::Groove => t.groove as i32,
        OctoTrackAttr::MidiChannel => t.midi_channel as i32,
        OctoTrackAttr::Muted => t.muted as i32,
        OctoTrackAttr::Soloed => t.soloed as i32,
        OctoTrackAttr::Paused => t.paused as i32,
        OctoTrackAttr::RecordArmed => t.record_armed as i32,
        OctoTrackAttr::IsFeeder => t.is_feeder as i32,
        OctoTrackAttr::IsListener => t.is_listener as i32,
    }
}

/// # Safety
/// `engine` must be a live pointer from `octocore_engine_new`.
#[no_mangle]
pub unsafe extern "C" fn octocore_step_set_i32(engine: *mut Engine, track: u8, step: u8, attr: OctoStepAttr, value: i32) -> bool {
    let Some(engine) = engine.as_mut() else { return false };
    if !track_in_range(track) || !step_in_range(step) {
        return false;
    }
    let s = &mut engine.grid.active_page_mut().tracks[track as usize].steps[step as usize];
    match attr {
        OctoStepAttr::Active => s.active = value != 0,
        OctoStepAttr::Skip => s.skip = value != 0,
        OctoStepAttr::PitchOffset => s.pitch_offset = value.clamp(-128, 127) as i8,
        OctoStepAttr::VelocityOffset => s.velocity_offset = value.clamp(-128, 127) as i8,
        OctoStepAttr::LengthTicks => s.length_ticks = value.clamp(1, 192) as u8,
        OctoStepAttr::LengthMultiplier => s.length_multiplier = value.clamp(1, 8) as u8,
        OctoStepAttr::StartOffset => s.start_offset = value.clamp(-5, 5) as i8,
        OctoStepAttr::Amount => s.amount = value.clamp(-128, 127) as i8,
        OctoStepAttr::Strum => s.strum = value.clamp(-9, 9) as i8,
        OctoStepAttr::Hyperstep => s.hyperstep = value != 0,
        OctoStepAttr::Phrase => {
            s.phrase = if value <= 0 { None } else { Some(value.clamp(1, 48) as u8) };
        }
        OctoStepAttr::PhrasePos => s.phrase_pos = value.clamp(1, 16) as u8,
    }
    true
}

/// # Safety
/// `engine` must be a live pointer from `octocore_engine_new`. Same
/// out-of-range convention as `octocore_track_get_i32`.
#[no_mangle]
pub unsafe extern "C" fn octocore_step_get_i32(engine: *const Engine, track: u8, step: u8, attr: OctoStepAttr) -> i32 {
    let Some(engine) = engine.as_ref() else { return 0 };
    if !track_in_range(track) || !step_in_range(step) {
        return 0;
    }
    let s = &engine.grid.active_page().tracks[track as usize].steps[step as usize];
    match attr {
        OctoStepAttr::Active => s.active as i32,
        OctoStepAttr::Skip => s.skip as i32,
        OctoStepAttr::PitchOffset => s.pitch_offset as i32,
        OctoStepAttr::VelocityOffset => s.velocity_offset as i32,
        OctoStepAttr::LengthTicks => s.length_ticks as i32,
        OctoStepAttr::LengthMultiplier => s.length_multiplier as i32,
        OctoStepAttr::StartOffset => s.start_offset as i32,
        OctoStepAttr::Amount => s.amount as i32,
        OctoStepAttr::Strum => s.strum as i32,
        OctoStepAttr::Hyperstep => s.hyperstep as i32,
        OctoStepAttr::Phrase => s.phrase.map(|p| p as i32).unwrap_or(0),
        OctoStepAttr::PhrasePos => s.phrase_pos as i32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_free_roundtrip_is_safe() {
        let e = octocore_engine_new(1);
        assert!(!e.is_null());
        unsafe {
            assert!(!octocore_engine_is_running(e));
            octocore_engine_handle_command(e, Command::Play);
            assert!(octocore_engine_is_running(e));
            octocore_engine_free(e);
        }
    }

    #[test]
    fn render_fills_caller_buffer() {
        let e = octocore_engine_new(1);
        unsafe {
            (*e).grid.active_page_mut().tracks[0].steps[0].active = true;
            octocore_engine_handle_command(e, Command::Play);

            let params = OctoRenderParams { sample_rate: 48_000.0, buffer_len: 4096, bpm: 120.0, playing: true };
            let mut events = [Event::Cc { port: 0, ch: 0, cc: 0, val: 0, at_sample: 0 }; 64];
            let mut count: usize = 0;
            let rc = octocore_engine_render(e, params, events.as_mut_ptr(), events.len(), &mut count);
            assert_eq!(rc, 0);
            // Not asserting count > 0 here — depends on exact tick alignment
            // within one buffer, which render_emits_note_with_at_sample_in_range
            // in octocore already covers properly over many buffers. This test
            // is about the FFI plumbing (no crash, no garbage pointer writes),
            // not the sequencing logic.

            octocore_engine_free(e);
        }
    }

    #[test]
    fn track_and_step_set_get_round_trip() {
        let e = octocore_engine_new(1);
        unsafe {
            assert!(octocore_track_set_i32(e, 3, OctoTrackAttr::Pitch, 60));
            assert_eq!(octocore_track_get_i32(e, 3, OctoTrackAttr::Pitch), 60);
            assert!(octocore_track_set_i32(e, 3, OctoTrackAttr::Muted, 1));
            assert_eq!(octocore_track_get_i32(e, 3, OctoTrackAttr::Muted), 1);

            assert!(octocore_step_set_i32(e, 3, 5, OctoStepAttr::Active, 1));
            assert!(octocore_step_set_i32(e, 3, 5, OctoStepAttr::PitchOffset, -7));
            assert_eq!(octocore_step_get_i32(e, 3, 5, OctoStepAttr::Active), 1);
            assert_eq!(octocore_step_get_i32(e, 3, 5, OctoStepAttr::PitchOffset), -7);

            assert!(octocore_step_set_i32(e, 3, 5, OctoStepAttr::Phrase, 12));
            assert_eq!(octocore_step_get_i32(e, 3, 5, OctoStepAttr::Phrase), 12);
            assert!(octocore_step_set_i32(e, 3, 5, OctoStepAttr::Phrase, 0));
            assert_eq!(octocore_step_get_i32(e, 3, 5, OctoStepAttr::Phrase), 0);

            // A different step on the same track is untouched.
            assert_eq!(octocore_step_get_i32(e, 3, 6, OctoStepAttr::Active), 0);

            octocore_engine_free(e);
        }
    }

    #[test]
    fn out_of_range_indices_are_a_no_op_not_a_crash() {
        let e = octocore_engine_new(1);
        unsafe {
            assert!(!octocore_track_set_i32(e, 200, OctoTrackAttr::Pitch, 60));
            assert_eq!(octocore_track_get_i32(e, 200, OctoTrackAttr::Pitch), 0);
            assert!(!octocore_step_set_i32(e, 0, 200, OctoStepAttr::Active, 1));
            assert_eq!(octocore_step_get_i32(e, 0, 200, OctoStepAttr::Active), 0);
            octocore_engine_free(e);
        }
    }

    /// Programming a pattern purely through the FFI surface (no direct
    /// `octocore` struct access, unlike the other tests here) and confirming
    /// it actually plays — proves the Grid-mutation surface is sufficient on
    /// its own to drive real playback, not just store values nobody reads.
    #[test]
    fn a_pattern_programmed_entirely_through_ffi_actually_plays() {
        let e = octocore_engine_new(1);
        unsafe {
            assert!(octocore_track_set_i32(e, 0, OctoTrackAttr::Pitch, 60));
            assert!(octocore_step_set_i32(e, 0, 0, OctoStepAttr::Active, 1));
            assert!(octocore_step_set_i32(e, 0, 0, OctoStepAttr::PitchOffset, 3));
            octocore_engine_handle_command(e, Command::Play);

            let params = OctoRenderParams { sample_rate: 48_000.0, buffer_len: 4096, bpm: 120.0, playing: true };
            let mut events = [Event::Cc { port: 0, ch: 0, cc: 0, val: 0, at_sample: 0 }; 64];
            let mut count: usize = 0;
            octocore_engine_render(e, params, events.as_mut_ptr(), events.len(), &mut count);

            let has_note_on_63 = events[..count].iter().any(|ev| matches!(ev, Event::NoteOn { note: 63, .. }));
            assert!(has_note_on_63, "expected a NoteOn at 60+3=63 from the pattern programmed via FFI; got {:?}", &events[..count]);

            octocore_engine_free(e);
        }
    }

    /// SPEC-0001 O5. `out_capacity` is the caller's room per call. Events that do not fit are
    /// carried to the next call. Before, the extras were dropped at this boundary, and a
    /// dropped NoteOff is a note that never ends.
    #[test]
    fn a_small_caller_buffer_delays_events_and_never_drops_them() {
        // (absolute sample, event without its buffer-relative position), for `buffers`
        // renders of 512 samples with a caller buffer of `capacity` events.
        fn run(capacity: usize, buffers: usize) -> Vec<(u64, [u8; 4])> {
            let e = octocore_engine_new(7);
            let mut all = Vec::new();
            unsafe {
                for t in 0..10u8 {
                    for st in 0..16u8 {
                        assert!(octocore_step_set_i32(e, t, st, OctoStepAttr::Active, 1));
                    }
                }
                octocore_engine_handle_command(e, Command::Play);
                let mut events = vec![Event::Cc { port: 0, ch: 0, cc: 0, val: 0, at_sample: 0 }; capacity];
                for b in 0..buffers {
                    let params = OctoRenderParams { sample_rate: 48_000.0, buffer_len: 512, bpm: 120.0, playing: true };
                    let mut count = 0usize;
                    assert_eq!(octocore_engine_render(e, params, events.as_mut_ptr(), events.len(), &mut count), 0);
                    for ev in &events[..count] {
                        let (at, key) = match *ev {
                            Event::NoteOn { port, ch, note, at_sample, .. } => (at_sample, [1, port, ch, note]),
                            Event::NoteOff { port, ch, note, at_sample } => (at_sample, [2, port, ch, note]),
                            Event::Cc { port, ch, cc, at_sample, .. } => (at_sample, [3, port, ch, cc]),
                        };
                        all.push((b as u64 * 512 + at as u64, key));
                    }
                }
                octocore_engine_free(e);
            }
            all
        }

        let buffers = 600;
        let reference = run(256, buffers);
        let small = run(8, buffers);
        assert!(reference.len() > 400, "the scenario must produce plenty of events, got {}", reference.len());

        // Whatever the small run has not delivered by the end is still queued behind the
        // caller's small buffer, so it can only be among the last events of the reference.
        let mut remaining: Vec<(u64, [u8; 4])> = small.iter().map(|(_, k)| (0, *k)).collect();
        let mut missing = Vec::new();
        for (t, k) in &reference {
            match remaining.iter().position(|(_, rk)| rk == k) {
                Some(i) => {
                    remaining.swap_remove(i);
                }
                None => missing.push(*t),
            }
        }
        let cutoff = (buffers as u64 - 24) * 512;
        let early: Vec<u64> = missing.iter().copied().filter(|t| *t < cutoff).collect();
        assert!(early.is_empty(), "{} events were dropped, first at sample {:?}", early.len(), early.first());
    }

    #[test]
    fn diagnostics_are_readable_and_null_safe() {
        let e = octocore_engine_new(1);
        unsafe {
            let mut d = octocore::Diagnostics { queue_overflows: 9, ..Default::default() };
            assert_eq!(octocore_engine_diagnostics(e, &mut d), 0);
            assert_eq!(d, octocore::Diagnostics::default());
            assert_eq!(octocore_engine_diagnostics(std::ptr::null(), &mut d), -1);
            assert_eq!(octocore_engine_diagnostics(e, std::ptr::null_mut()), -2);

            // A tempo of 0 with the transport running is counted, and the engine survives it.
            octocore_engine_handle_command(e, Command::Play);
            let params = OctoRenderParams { sample_rate: 48_000.0, buffer_len: 512, bpm: 0.0, playing: true };
            let mut events = [Event::Cc { port: 0, ch: 0, cc: 0, val: 0, at_sample: 0 }; 8];
            let mut count = 0usize;
            assert_eq!(octocore_engine_render(e, params, events.as_mut_ptr(), events.len(), &mut count), 0);
            assert_eq!(octocore_engine_diagnostics(e, &mut d), 0);
            assert_eq!(d.unusable_tempo_renders, 1);
            octocore_engine_free(e);
        }
    }
}
