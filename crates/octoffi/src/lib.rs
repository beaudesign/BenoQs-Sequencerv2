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

/// The attribute enums live in `octocore` (`TrackAttr`, `StepAttr`), where the logical commands
/// `SetTrack` and `SetStep` use them too. These names are the C header's.
pub type OctoTrackAttr = octocore::types::TrackAttr;
pub type OctoStepAttr = octocore::types::StepAttr;

// Setters return `false` (and do nothing), and getters return 0, for an out-of-range
// `track` or `step`: the same "caller mistake is a no-op, not a crash" contract as the rest
// of this boundary. The clamping tables live in `octocore::attrs`.

/// # Safety
/// `engine` must be a live pointer from `octocore_engine_new`.
#[no_mangle]
pub unsafe extern "C" fn octocore_track_set_i32(engine: *mut Engine, track: u8, attr: OctoTrackAttr, value: i32) -> bool {
    match engine.as_mut() {
        Some(engine) => engine.grid.set_track_attr(track, attr, value),
        None => false,
    }
}

/// # Safety
/// `engine` must be a live pointer from `octocore_engine_new`. Returns 0 for
/// an out-of-range `track` — indistinguishable from a real 0 value; callers
/// that need to tell these apart should validate `track` themselves first
/// (it's always `< 10`).
#[no_mangle]
pub unsafe extern "C" fn octocore_track_get_i32(engine: *const Engine, track: u8, attr: OctoTrackAttr) -> i32 {
    match engine.as_ref() {
        Some(engine) => engine.grid.track_attr(track, attr),
        None => 0,
    }
}

/// # Safety
/// `engine` must be a live pointer from `octocore_engine_new`.
#[no_mangle]
pub unsafe extern "C" fn octocore_step_set_i32(engine: *mut Engine, track: u8, step: u8, attr: OctoStepAttr, value: i32) -> bool {
    match engine.as_mut() {
        Some(engine) => engine.grid.set_step_attr(track, step, attr, value),
        None => false,
    }
}

/// # Safety
/// `engine` must be a live pointer from `octocore_engine_new`. Same
/// out-of-range convention as `octocore_track_get_i32`.
#[no_mangle]
pub unsafe extern "C" fn octocore_step_get_i32(engine: *const Engine, track: u8, step: u8, attr: OctoStepAttr) -> i32 {
    match engine.as_ref() {
        Some(engine) => engine.grid.step_attr(track, step, attr),
        None => 0,
    }
}

// --- Link: the command ring and the snapshot (SPEC-0001 O6) ---
//
// The engine is single-threaded: only the audio thread may call `octocore_engine_render`,
// and the grid setters above are for offline use and tests, never while a render can run.
// Everything a host does from another thread goes through the two ends made here.
// `octocore_engine_open_link` hands out one `OctoSender` for the main thread (commands in)
// and one `OctoReader` for the render thread (snapshots out). Each end belongs to one
// thread at a time. Both are wait-free and neither allocates after this call. They share
// their buffers with the engine by reference count, not by pointer into it, so they stay
// safe to use after `octocore_engine_free`; they just stop having anything to say.

/// Bumped when the header gains something a caller must know about. 1 was the header
/// before the link; 2 adds `OctoSender`, `OctoReader`, `OctoSnapshot` and the commands
/// `OCTO_CMD_SET_TRACK` and `OCTO_CMD_SET_STEP`. Nothing that 1 declared has changed.
pub const OCTOFFI_ABI_VERSION: u32 = 2;

#[no_mangle]
pub extern "C" fn octocore_abi_version() -> u32 {
    OCTOFFI_ABI_VERSION
}

/// The main-thread end of the command ring. Opaque to C.
pub struct OctoSender {
    inner: octocore::CommandSender,
}

/// The render-thread end of the snapshot buffer. Opaque to C.
pub struct OctoReader {
    inner: octocore::SnapshotReader,
}

/// Counters of the command ring. `pushed == applied + dropped + (still waiting)`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OctoLinkStats {
    /// Commands pushed.
    pub pushed: u64,
    /// Commands overwritten before the engine took them, plus words that were not a command.
    pub dropped: u64,
    /// Commands the engine took and applied.
    pub applied: u64,
}

/// Opens the link, once per engine. Writes the two ends to `*sender` and `*reader` and
/// returns 0; -1 for a null engine, -2 for a null out pointer, -3 if the link is already
/// open (nothing is written then). Allocates, so call it at setup, not on the audio thread.
///
/// # Safety
/// `engine` must be a live pointer from `octocore_engine_new`. `sender` and `reader` must
/// each point to one writable pointer. Free the ends with `octocore_sender_free` and
/// `octocore_reader_free`.
#[no_mangle]
pub unsafe extern "C" fn octocore_engine_open_link(
    engine: *mut Engine,
    sender: *mut *mut OctoSender,
    reader: *mut *mut OctoReader,
) -> i32 {
    let Some(engine) = engine.as_mut() else { return -1 };
    if sender.is_null() || reader.is_null() {
        return -2;
    }
    match engine.open_link() {
        Some((tx, rx)) => {
            *sender = Box::into_raw(Box::new(OctoSender { inner: tx }));
            *reader = Box::into_raw(Box::new(OctoReader { inner: rx }));
            0
        }
        None => -3,
    }
}

/// Queues a command for the engine's next render. Wait-free; if the ring is full the
/// oldest command is overwritten and counted as dropped. Returns `false` only for a null
/// sender. A `cmd` whose tag is not one of the `OCTO_CMD_*` values is undefined behaviour,
/// as it is for `octocore_engine_handle_command`.
///
/// # Safety
/// `sender` must be a live pointer from `octocore_engine_open_link`, used by one thread.
#[no_mangle]
pub unsafe extern "C" fn octocore_sender_push(sender: *mut OctoSender, cmd: Command) -> bool {
    match sender.as_mut() {
        Some(s) => {
            s.inner.push(cmd);
            true
        }
        None => false,
    }
}

/// Copies the ring's counters to `*out`. Returns 0, -1 for a null sender, -2 for a null `out`.
///
/// # Safety
/// `sender` must be a live pointer from `octocore_engine_open_link`. `out` must point to one
/// writable `OctoLinkStats`.
#[no_mangle]
pub unsafe extern "C" fn octocore_sender_stats(sender: *const OctoSender, out: *mut OctoLinkStats) -> i32 {
    let Some(s) = sender.as_ref() else { return -1 };
    let Some(out) = out.as_mut() else { return -2 };
    let r = s.inner.stats();
    *out = OctoLinkStats { pushed: r.pushed, dropped: r.dropped, applied: r.received };
    0
}

/// Takes the newest snapshot the engine has published, if there is a new one, and returns a
/// pointer to it. Before the first publish, and while nothing new has been published, it
/// returns the last one (an all-zero snapshot with generation 0 to begin with): compare
/// `generation` to tell. The pointer stays valid until `octocore_reader_free` and is the same
/// on every call; its contents change only inside this function. Returns null for a null reader.
///
/// # Safety
/// `reader` must be a live pointer from `octocore_engine_open_link`, used by one thread, and
/// the caller must not read through a previously returned pointer while this runs.
#[no_mangle]
pub unsafe extern "C" fn octocore_reader_claim(reader: *mut OctoReader) -> *const octocore::types::Snapshot {
    match reader.as_mut() {
        Some(r) => {
            r.inner.claim();
            r.inner.snapshot() as *const octocore::types::Snapshot
        }
        None => std::ptr::null(),
    }
}

/// # Safety
/// `sender` must be null or a pointer from `octocore_engine_open_link` not yet freed.
#[no_mangle]
pub unsafe extern "C" fn octocore_sender_free(sender: *mut OctoSender) {
    if !sender.is_null() {
        drop(Box::from_raw(sender));
    }
}

/// # Safety
/// `reader` must be null or a pointer from `octocore_engine_open_link` not yet freed.
#[no_mangle]
pub unsafe extern "C" fn octocore_reader_free(reader: *mut OctoReader) {
    if !reader.is_null() {
        drop(Box::from_raw(reader));
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
                            Event::PitchBend { port, ch, at_sample, .. } => (at_sample, [4, port, ch, 0]),
                            Event::ChannelPressure { port, ch, at_sample, .. } => (at_sample, [5, port, ch, 0]),
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

    /// `octoffi.h` is written by hand, so the bytes of the new events are pinned here: tag
    /// values, and where each field sits in the 12-byte `Event`. Little-endian layouts, which
    /// is every platform this ships on. Only bytes that hold data are read, never padding.
    #[cfg(target_endian = "little")]
    #[test]
    fn new_event_variants_match_the_layout_in_the_c_header() {
        assert_eq!(std::mem::size_of::<Event>(), 12, "adding variants must not grow Event");
        fn bytes(e: &Event, at: &[usize]) -> Vec<u8> {
            let base = e as *const Event as *const u8;
            // SAFETY: `at` lists offsets of initialised bytes inside the 12 bytes of `*e`.
            at.iter().map(|i| unsafe { *base.add(*i) }).collect()
        }
        // Tag at 0..4.
        let tag = |e: &Event| bytes(e, &[0, 1, 2, 3]);
        let bend = Event::PitchBend { port: 2, ch: 9, value: 0x1234, at_sample: 0xAABB_CCDD };
        let pressure = Event::ChannelPressure { port: 1, ch: 16, value: 100, at_sample: 0x0102_0304 };
        assert_eq!(tag(&Event::NoteOn { port: 0, ch: 0, note: 0, vel: 0, at_sample: 0 }), [0, 0, 0, 0]);
        assert_eq!(tag(&Event::Cc { port: 0, ch: 0, cc: 0, val: 0, at_sample: 0 }), [2, 0, 0, 0]);
        assert_eq!(tag(&bend), [3, 0, 0, 0], "OCTO_EVT_PITCH_BEND = 3");
        assert_eq!(tag(&pressure), [4, 0, 0, 0], "OCTO_EVT_CHANNEL_PRESSURE = 4");
        // struct { uint8_t port, ch; uint16_t value; uint32_t at_sample; }
        assert_eq!(bytes(&bend, &[4, 5, 6, 7, 8, 9, 10, 11]), [2, 9, 0x34, 0x12, 0xDD, 0xCC, 0xBB, 0xAA]);
        // struct { uint8_t port, ch, value; uint32_t at_sample; }
        assert_eq!(bytes(&pressure, &[4, 5, 6, 8, 9, 10, 11]), [1, 16, 100, 4, 3, 2, 1]);
    }

    /// The test above checks the Rust bytes. This one reads `octoffi.h` itself, so an edit to
    /// the header that disagrees with the Rust side fails a test (review nit 11).
    #[test]
    fn the_header_lists_the_event_tags_and_diagnostic_fields_in_the_rust_order() {
        let h = include_str!("../octoffi.h");
        let tags: Vec<&str> = h
            .lines()
            .map(str::trim)
            .filter(|l| l.starts_with("OCTO_EVT_"))
            .map(|l| l.split(|c: char| c == ' ' || c == ',' || c == '=').next().unwrap())
            .collect();
        assert_eq!(
            tags,
            ["OCTO_EVT_NOTE_ON", "OCTO_EVT_NOTE_OFF", "OCTO_EVT_CC", "OCTO_EVT_PITCH_BEND", "OCTO_EVT_CHANNEL_PRESSURE"],
            "tags are numbered by position: 0 to 4, and the Rust `Event` tags follow the same order"
        );
        assert!(h.contains("OCTO_EVT_NOTE_ON = 0,"));
        assert!(h.contains("struct { uint8_t port, ch; uint16_t value; uint32_t at_sample; } pitch_bend;"));
        assert!(h.contains("struct { uint8_t port, ch, value; uint32_t at_sample; } channel_pressure;"));

        let start = h.find("typedef struct {").and_then(|_| h.find("uint32_t queue_overflows")).expect("OctoDiagnostics");
        let end = h[start..].find("} OctoDiagnostics;").expect("end of OctoDiagnostics") + start;
        let fields: Vec<&str> = h[start..end]
            .lines()
            .filter_map(|l| l.trim().strip_prefix("uint32_t "))
            .map(|l| l.split(';').next().unwrap())
            .collect();
        assert_eq!(
            fields,
            ["queue_overflows", "queue_high_water", "deferred_events", "late_events", "unusable_tempo_renders"],
            "the fields of `Diagnostics` (repr(C)) in declaration order"
        );
        assert_eq!(std::mem::size_of::<octocore::Diagnostics>(), 5 * 4);
    }

    // ---- SPEC-0001 O6: the link (WENGE-0006, tests F1 to F6 and E4) ----

    use std::alloc::{GlobalAlloc, Layout, System};
    use std::cell::Cell;

    /// Counts the allocations made on the calling thread, so the allocation test is not
    /// disturbed by other tests running in parallel.
    struct CountingAlloc;

    thread_local! {
        static ALLOCS: Cell<usize> = const { Cell::new(0) };
    }

    // SAFETY: every method forwards to `System` unchanged, and only bumps a thread-local
    // counter, which needs no allocation itself.
    unsafe impl GlobalAlloc for CountingAlloc {
        unsafe fn alloc(&self, l: Layout) -> *mut u8 {
            ALLOCS.with(|c| c.set(c.get() + 1));
            System.alloc(l)
        }
        unsafe fn alloc_zeroed(&self, l: Layout) -> *mut u8 {
            ALLOCS.with(|c| c.set(c.get() + 1));
            System.alloc_zeroed(l)
        }
        unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
            ALLOCS.with(|c| c.set(c.get() + 1));
            System.realloc(p, l, n)
        }
        unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
            System.dealloc(p, l)
        }
    }

    #[global_allocator]
    static GLOBAL: CountingAlloc = CountingAlloc;

    fn allocations() -> usize {
        ALLOCS.with(|c| c.get())
    }

    const PARAMS: OctoRenderParams = OctoRenderParams { sample_rate: 48_000.0, buffer_len: 256, bpm: 120.0, playing: true };

    /// Renders one buffer through the C function and returns the events.
    unsafe fn render_once(e: *mut Engine, params: OctoRenderParams) -> Vec<Event> {
        let mut events = [Event::Cc { port: 0, ch: 0, cc: 0, val: 0, at_sample: 0 }; 64];
        let mut count = 0usize;
        assert_eq!(octocore_engine_render(e, params, events.as_mut_ptr(), events.len(), &mut count), 0);
        events[..count].to_vec()
    }

    unsafe fn open(e: *mut Engine) -> (*mut OctoSender, *mut OctoReader) {
        let (mut s, mut r) = (std::ptr::null_mut(), std::ptr::null_mut());
        assert_eq!(octocore_engine_open_link(e, &mut s, &mut r), 0);
        assert!(!s.is_null() && !r.is_null());
        (s, r)
    }

    /// The header as of `9e4860b`, before the link, one code line per entry with comments and
    /// blank lines removed. Every one of them must still be in the header: the link only adds.
    const OLD_HEADER_CODE: &[&str] = &[
    "#ifndef OCTOFFI_H",
    "#define OCTOFFI_H",
    "#include <stdbool.h>",
    "#include <stdint.h>",
    "#include <stddef.h>",
    "#ifdef __cplusplus",
    "extern \"C\" {",
    "#endif",
    "typedef struct OctoEngine OctoEngine;",
    "typedef enum {",
    "OCTO_CMD_PLAY = 0,",
    "OCTO_CMD_STOP,",
    "OCTO_CMD_CONTINUE,",
    "OCTO_CMD_RESET,",
    "OCTO_CMD_BUTTON_DOWN,",
    "OCTO_CMD_BUTTON_UP,",
    "OCTO_CMD_ENCODER_TURN,",
    "OCTO_CMD_SET_ACTIVE_PAGE,",
    "OCTO_CMD_SET_MODE,",
    "OCTO_CMD_HOST_TRANSPORT,",
    "OCTO_CMD_LOAD_STATE,",
    "} OctoCommandTag;",
    "typedef struct {",
    "OctoCommandTag tag;",
    "union {",
    "struct { uint32_t control; float velocity_mm_s; } button_down;",
    "struct { uint32_t control; } button_up;",
    "struct { uint32_t control; int16_t detents; float angular_velocity; } encoder_turn;",
    "struct { uint8_t bank; uint8_t page; } set_active_page;",
    "struct { uint8_t mode; } set_mode;",
    "struct { uint64_t ppqn_pos; float bpm; bool playing; } host_transport;",
    "struct { uint64_t handle; } load_state;",
    "};",
    "} OctoCommand;",
    "typedef enum {",
    "OCTO_EVT_NOTE_ON = 0,",
    "OCTO_EVT_NOTE_OFF,",
    "OCTO_EVT_CC,",
    "OCTO_EVT_PITCH_BEND,",
    "OCTO_EVT_CHANNEL_PRESSURE,",
    "} OctoEventTag;",
    "typedef struct {",
    "OctoEventTag tag;",
    "union {",
    "struct { uint8_t port, ch, note, vel; uint32_t at_sample; } note_on;",
    "struct { uint8_t port, ch, note; uint32_t at_sample; } note_off;",
    "struct { uint8_t port, ch, cc, val; uint32_t at_sample; } cc;",
    "struct { uint8_t port, ch; uint16_t value; uint32_t at_sample; } pitch_bend;",
    "struct { uint8_t port, ch, value; uint32_t at_sample; } channel_pressure;",
    "};",
    "} OctoEvent;",
    "typedef struct {",
    "float sample_rate;",
    "uint32_t buffer_len;",
    "float bpm;",
    "bool playing;",
    "} OctoRenderParams;",
    "OctoEngine *octocore_engine_new(uint64_t seed);",
    "void octocore_engine_free(OctoEngine *engine);",
    "void octocore_engine_handle_command(OctoEngine *engine, OctoCommand cmd);",
    "int32_t octocore_engine_render(OctoEngine *engine, OctoRenderParams params,",
    "OctoEvent *out_events, size_t out_capacity,",
    "size_t *out_count);",
    "bool octocore_engine_is_running(const OctoEngine *engine);",
    "typedef struct {",
    "uint32_t queue_overflows;",
    "uint32_t queue_high_water;",
    "uint32_t deferred_events;",
    "uint32_t late_events;",
    "uint32_t unusable_tempo_renders;",
    "} OctoDiagnostics;",
    "int32_t octocore_engine_diagnostics(const OctoEngine *engine, OctoDiagnostics *out);",
    "typedef enum {",
    "OCTO_TRACK_PITCH = 0,",
    "OCTO_TRACK_VELOCITY,",
    "OCTO_TRACK_LENGTH_FACTOR,",
    "OCTO_TRACK_START_FACTOR,",
    "OCTO_TRACK_DIRECTION_RAW,",
    "OCTO_TRACK_ROTATION,",
    "OCTO_TRACK_AMOUNT,",
    "OCTO_TRACK_GROOVE,",
    "OCTO_TRACK_MIDI_CHANNEL,",
    "OCTO_TRACK_MUTED,",
    "OCTO_TRACK_SOLOED,",
    "OCTO_TRACK_PAUSED,",
    "OCTO_TRACK_RECORD_ARMED,",
    "OCTO_TRACK_IS_FEEDER,",
    "OCTO_TRACK_IS_LISTENER,",
    "} OctoTrackAttr;",
    "typedef enum {",
    "OCTO_STEP_ACTIVE = 0,",
    "OCTO_STEP_SKIP,",
    "OCTO_STEP_PITCH_OFFSET,",
    "OCTO_STEP_VELOCITY_OFFSET,",
    "OCTO_STEP_LENGTH_TICKS,",
    "OCTO_STEP_LENGTH_MULTIPLIER,",
    "OCTO_STEP_START_OFFSET,",
    "OCTO_STEP_AMOUNT,",
    "OCTO_STEP_STRUM,",
    "OCTO_STEP_HYPERSTEP,",
    "OCTO_STEP_PHRASE,",
    "OCTO_STEP_PHRASE_POS,",
    "} OctoStepAttr;",
    "bool octocore_track_set_i32(OctoEngine *engine, uint8_t track, OctoTrackAttr attr, int32_t value);",
    "int32_t octocore_track_get_i32(const OctoEngine *engine, uint8_t track, OctoTrackAttr attr);",
    "bool octocore_step_set_i32(OctoEngine *engine, uint8_t track, uint8_t step, OctoStepAttr attr, int32_t value);",
    "int32_t octocore_step_get_i32(const OctoEngine *engine, uint8_t track, uint8_t step, OctoStepAttr attr);",
    "#ifdef __cplusplus",
    "}",
    "#endif",
    "#endif",
];

    #[test]
    fn f4_every_declaration_the_header_had_before_the_link_is_still_there() {
        let h = include_str!("../octoffi.h");
        let now: Vec<&str> = h.lines().map(|l| l.split("//").next().unwrap().trim()).filter(|l| !l.is_empty()).collect();
        let missing: Vec<&&str> = OLD_HEADER_CODE.iter().filter(|old| !now.contains(old)).collect();
        assert!(missing.is_empty(), "lines the header lost: {missing:?}");
        assert_eq!(OLD_HEADER_CODE.len(), 111, "the list above is the whole old header");
    }

    #[test]
    fn f5_the_abi_version_is_the_same_in_the_header_the_constant_and_the_function() {
        let h = include_str!("../octoffi.h");
        let line = h.lines().find(|l| l.starts_with("#define OCTOFFI_ABI_VERSION ")).expect("the header defines OCTOFFI_ABI_VERSION");
        let n: u32 = line.rsplit(' ').next().unwrap().trim().parse().unwrap();
        assert_eq!(n, OCTOFFI_ABI_VERSION);
        assert_eq!(octocore_abi_version(), n);
        assert_eq!(n, 2, "1 was the header before the link");
    }

    #[test]
    fn f1_the_header_declares_the_link_surface_in_the_order_the_snapshot_is_laid_out() {
        let h = include_str!("../octoffi.h");
        for needle in [
            "typedef struct OctoSender OctoSender;",
            "typedef struct OctoReader OctoReader;",
            "uint32_t octocore_abi_version(void);",
            "int32_t octocore_engine_open_link(OctoEngine *engine, OctoSender **sender, OctoReader **reader);",
            "bool octocore_sender_push(OctoSender *sender, OctoCommand cmd);",
            "int32_t octocore_sender_stats(const OctoSender *sender, OctoLinkStats *out);",
            "const OctoSnapshot *octocore_reader_claim(OctoReader *reader);",
            "void octocore_sender_free(OctoSender *sender);",
            "void octocore_reader_free(OctoReader *reader);",
            "OCTO_CMD_SET_TRACK,",
            "OCTO_CMD_SET_STEP,",
            "struct { uint8_t track; OctoTrackAttr attr; int32_t value; } set_track;",
            "struct { uint8_t track; uint8_t step; OctoStepAttr attr; int32_t value; } set_step;",
        ] {
            assert!(h.contains(needle), "the header does not contain: {needle}");
        }
        let a = h.find("uint64_t generation;").expect("OctoSnapshot");
        let b = h[a..].find("} OctoSnapshot;").expect("end of OctoSnapshot") + a;
        let fields: Vec<&str> = h[a..b].lines().map(|l| l.split("//").next().unwrap().trim()).filter(|l| !l.is_empty()).collect();
        assert_eq!(
            fields,
            [
                "uint64_t generation;",
                "OctoLed leds[OCTO_MAX_CONTROLS];",
                "OctoEncoderState encoders[OCTO_ENCODER_COUNT];",
                "OctoPlayheadState playheads[OCTO_TRACK_COUNT];",
                "OctoTransportState transport;",
                "uint8_t mode;",
                "OctoActiveRefs active;",
            ],
            "the fields of `octocore::types::Snapshot`, in order"
        );
    }

    /// The strongest check on the hand-written header: a real C compiler reads it, and
    /// `_Static_assert`s built from the Rust sizes and offsets must hold. Skipped, with a
    /// message, on a machine with no `cc`; CI has one.
    #[cfg(target_endian = "little")]
    #[test]
    fn f1_a_c_compiler_agrees_with_rust_about_every_size_and_offset_in_the_header() {
        use octocore::types::Snapshot;
        use std::mem::{offset_of, size_of};
        let cc = std::process::Command::new("cc").arg("--version").output();
        if cc.is_err() {
            eprintln!("no C compiler found: skipping the header layout check");
            return;
        }
        let dir = std::env::temp_dir().join(format!("octoffi-header-check-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let src = dir.join("check.c");
        let assertions = [
            ("sizeof(OctoSnapshot)", size_of::<Snapshot>()),
            ("offsetof(OctoSnapshot, generation)", offset_of!(Snapshot, generation)),
            ("offsetof(OctoSnapshot, leds)", offset_of!(Snapshot, leds)),
            ("offsetof(OctoSnapshot, encoders)", offset_of!(Snapshot, encoders)),
            ("offsetof(OctoSnapshot, playheads)", offset_of!(Snapshot, playheads)),
            ("offsetof(OctoSnapshot, transport)", offset_of!(Snapshot, transport)),
            ("offsetof(OctoSnapshot, mode)", offset_of!(Snapshot, mode)),
            ("offsetof(OctoSnapshot, active)", offset_of!(Snapshot, active)),
            ("sizeof(OctoLed)", size_of::<octocore::types::Led>()),
            ("sizeof(OctoTransportState)", size_of::<octocore::types::TransportState>()),
            ("sizeof(OctoCommand)", size_of::<Command>()),
            ("sizeof(OctoEvent)", size_of::<Event>()),
            ("sizeof(OctoDiagnostics)", size_of::<octocore::Diagnostics>()),
            ("sizeof(OctoRenderParams)", size_of::<OctoRenderParams>()),
            ("sizeof(OctoLinkStats)", size_of::<OctoLinkStats>()),
            ("sizeof(OctoTrackAttr)", size_of::<OctoTrackAttr>()),
            ("sizeof(OctoStepAttr)", size_of::<OctoStepAttr>()),
            // Where the new command payloads start, which `new_command_variants_match_...` checks on the Rust side.
            ("offsetof(OctoCommand, set_track.track)", 8),
            ("offsetof(OctoCommand, set_track.attr)", 12),
            ("offsetof(OctoCommand, set_track.value)", 16),
            ("offsetof(OctoCommand, set_step.track)", 8),
            ("offsetof(OctoCommand, set_step.step)", 9),
            ("offsetof(OctoCommand, set_step.attr)", 12),
            ("offsetof(OctoCommand, set_step.value)", 16),
            ("OCTO_CMD_LOAD_STATE", 10),
            ("OCTO_CMD_SET_TRACK", 11),
            ("OCTO_CMD_SET_STEP", 12),
            ("OCTO_TRACK_IS_LISTENER", 14),
            ("OCTO_STEP_PHRASE_POS", 11),
        ];
        let mut c = String::from("#include <stddef.h>\n#include \"octoffi.h\"\n");
        for (expr, want) in assertions {
            c += &format!("_Static_assert({expr} == {want}, \"{expr}\");\n");
        }
        // Every enumerator against the Rust value it must equal. The attribute numbers travel
        // on the wire, so a header with two enumerators swapped would make a C caller's
        // Velocity write land on LengthFactor, and no size or offset check can see that (an
        // independent review swapped two and every other test stayed green).
        fn upper_snake(camel: &str) -> String {
            let mut out = String::new();
            for (i, ch) in camel.chars().enumerate() {
                if ch.is_uppercase() && i > 0 {
                    out.push('_');
                }
                out.extend(ch.to_uppercase());
            }
            out
        }
        for (i, a) in octocore::types::TrackAttr::ALL.iter().enumerate() {
            assert_eq!(*a as usize, i, "TrackAttr::ALL is in discriminant order");
            c += &format!("_Static_assert(OCTO_TRACK_{} == {i}, \"OCTO_TRACK_{}\");\n", upper_snake(&format!("{a:?}")), upper_snake(&format!("{a:?}")));
        }
        for (i, a) in octocore::types::StepAttr::ALL.iter().enumerate() {
            assert_eq!(*a as usize, i, "StepAttr::ALL is in discriminant order");
            c += &format!("_Static_assert(OCTO_STEP_{} == {i}, \"OCTO_STEP_{}\");\n", upper_snake(&format!("{a:?}")), upper_snake(&format!("{a:?}")));
        }
        {
            use octocore::types::{Command as C, ControlId};
            let all = [
                ("PLAY", C::Play),
                ("STOP", C::Stop),
                ("CONTINUE", C::Continue),
                ("RESET", C::Reset),
                ("BUTTON_DOWN", C::ButtonDown { control: ControlId(0), velocity_mm_s: 0.0 }),
                ("BUTTON_UP", C::ButtonUp { control: ControlId(0) }),
                ("ENCODER_TURN", C::EncoderTurn { control: ControlId(0), detents: 0, angular_velocity: 0.0 }),
                ("SET_ACTIVE_PAGE", C::SetActivePage { bank: 0, page: 0 }),
                ("SET_MODE", C::SetMode { mode: octocore::domain::Mode::Grid }),
                ("HOST_TRANSPORT", C::HostTransport { ppqn_pos: 0, bpm: 0.0, playing: false }),
                ("LOAD_STATE", C::LoadState { handle: 0 }),
                ("SET_TRACK", C::SetTrack { track: 0, attr: octocore::types::TrackAttr::Pitch, value: 0 }),
                ("SET_STEP", C::SetStep { track: 0, step: 0, attr: octocore::types::StepAttr::Active, value: 0 }),
            ];
            for (name, cmd) in all {
                // The tag is the first four bytes of the `repr(C)` enum.
                let tag = unsafe { *(&cmd as *const C as *const u32) };
                c += &format!("_Static_assert(OCTO_CMD_{name} == {tag}, \"OCTO_CMD_{name}\");\n");
            }
        }
        c += "int main(void) { return 0; }\n";
        std::fs::write(&src, c).unwrap();
        let out = std::process::Command::new("cc")
            .args(["-std=c11", "-Wall", "-Werror", "-fsyntax-only", "-I"])
            .arg(env!("CARGO_MANIFEST_DIR"))
            .arg(&src)
            .output()
            .unwrap();
        let _ = std::fs::remove_dir_all(&dir);
        assert!(out.status.success(), "the C compiler rejected the header or a layout differs:\n{}", String::from_utf8_lossy(&out.stderr));
    }

    #[cfg(target_endian = "little")]
    #[test]
    fn new_command_variants_match_the_layout_in_the_c_header() {
        use octocore::types::{StepAttr, TrackAttr};
        fn bytes(c: &Command, at: &[usize]) -> Vec<u8> {
            let base = c as *const Command as *const u8;
            // SAFETY: `at` lists offsets of initialised bytes inside the 24 bytes of `*c`.
            at.iter().map(|i| unsafe { *base.add(*i) }).collect()
        }
        assert_eq!(std::mem::size_of::<Command>(), 24, "the new variants must not grow Command");
        let t = Command::SetTrack { track: 0xAB, attr: TrackAttr::Groove, value: 0x0102_0304 };
        assert_eq!(bytes(&t, &[0, 1, 2, 3]), [11, 0, 0, 0], "OCTO_CMD_SET_TRACK = 11");
        assert_eq!(bytes(&t, &[8]), [0xAB]);
        assert_eq!(bytes(&t, &[12, 13, 14, 15]), [7, 0, 0, 0], "Groove is track attribute 7");
        assert_eq!(bytes(&t, &[16, 17, 18, 19]), [4, 3, 2, 1]);
        let s = Command::SetStep { track: 0xAB, step: 0xCD, attr: StepAttr::Phrase, value: -2 };
        assert_eq!(bytes(&s, &[0, 1, 2, 3]), [12, 0, 0, 0], "OCTO_CMD_SET_STEP = 12");
        assert_eq!(bytes(&s, &[8, 9]), [0xAB, 0xCD]);
        assert_eq!(bytes(&s, &[12, 13, 14, 15]), [10, 0, 0, 0], "Phrase is step attribute 10");
        assert_eq!(bytes(&s, &[16, 17, 18, 19]), [0xFE, 0xFF, 0xFF, 0xFF]);
    }

    #[test]
    fn f2_null_and_repeated_calls_are_refused_without_harm() {
        unsafe {
            let e = octocore_engine_new(1);
            let (mut s, mut r) = (std::ptr::null_mut(), std::ptr::null_mut());
            assert_eq!(octocore_engine_open_link(std::ptr::null_mut(), &mut s, &mut r), -1);
            assert_eq!(octocore_engine_open_link(e, std::ptr::null_mut(), &mut r), -2);
            assert_eq!(octocore_engine_open_link(e, &mut s, std::ptr::null_mut()), -2);
            assert!(s.is_null() && r.is_null(), "nothing is written on an error");
            assert_eq!(octocore_engine_open_link(e, &mut s, &mut r), 0);
            let (mut s2, mut r2) = (std::ptr::null_mut(), std::ptr::null_mut());
            assert_eq!(octocore_engine_open_link(e, &mut s2, &mut r2), -3);
            assert!(s2.is_null() && r2.is_null());

            assert!(!octocore_sender_push(std::ptr::null_mut(), Command::Play));
            let mut stats = OctoLinkStats::default();
            assert_eq!(octocore_sender_stats(std::ptr::null(), &mut stats), -1);
            assert_eq!(octocore_sender_stats(s, std::ptr::null_mut()), -2);
            assert!(octocore_reader_claim(std::ptr::null_mut()).is_null());
            octocore_sender_free(std::ptr::null_mut());
            octocore_reader_free(std::ptr::null_mut());

            octocore_sender_free(s);
            octocore_reader_free(r);
            octocore_engine_free(e);
        }
    }

    #[test]
    fn f3_a_pattern_can_be_edited_played_and_watched_through_the_c_functions_alone() {
        use octocore::types::{StepAttr, TrackAttr};
        unsafe {
            let e = octocore_engine_new(3);
            let (s, r) = open(e);
            let before = octocore_reader_claim(r);
            assert_eq!((*before).generation, 0, "nothing published before the first render");

            assert!(octocore_sender_push(s, Command::SetTrack { track: 0, attr: TrackAttr::Pitch, value: 61 }));
            assert!(octocore_sender_push(s, Command::SetStep { track: 0, step: 0, attr: StepAttr::Active, value: 1 }));
            assert!(octocore_sender_push(s, Command::SetMode { mode: octocore::domain::Mode::Step }));
            let mut events = Vec::new();
            for _ in 0..8 {
                events.extend(render_once(e, PARAMS));
            }
            assert!(events.iter().any(|ev| matches!(ev, Event::NoteOn { note: 61, .. })), "the note the ring programmed sounded: {events:?}");

            let snap = octocore_reader_claim(r);
            assert_eq!(snap, before, "the same pointer every time");
            assert_eq!((*snap).generation, 8);
            assert!((*snap).transport.playing);
            assert_eq!((*snap).mode, octocore::domain::Mode::Step);
            let again = octocore_reader_claim(r);
            assert_eq!((*again).generation, 8, "nothing new: the last snapshot again");

            let mut st = OctoLinkStats::default();
            assert_eq!(octocore_sender_stats(s, &mut st), 0);
            assert_eq!(st, OctoLinkStats { pushed: 3, dropped: 0, applied: 3 });

            octocore_sender_free(s);
            octocore_reader_free(r);
            octocore_engine_free(e);
        }
    }

    #[test]
    fn f6_the_ends_outlive_the_engine_without_a_crash() {
        unsafe {
            let e = octocore_engine_new(3);
            let (s, r) = open(e);
            render_once(e, PARAMS);
            assert_eq!((*octocore_reader_claim(r)).generation, 1);
            octocore_engine_free(e);
            for k in 0..2000 {
                assert!(octocore_sender_push(s, Command::SetTrack { track: 1, attr: octocore::types::TrackAttr::Pitch, value: k % 100 }));
            }
            assert_eq!((*octocore_reader_claim(r)).generation, 1, "the engine is gone, so nothing new");
            let mut st = OctoLinkStats::default();
            assert_eq!(octocore_sender_stats(s, &mut st), 0);
            assert_eq!(st.pushed, 2000);
            octocore_sender_free(s);
            octocore_reader_free(r);
        }
    }

    #[test]
    fn e4_render_push_and_claim_allocate_nothing() {
        use octocore::types::{StepAttr, TrackAttr};
        unsafe {
            let e = octocore_engine_new(3);
            let (s, r) = open(e);
            for track in 0..10u8 {
                octocore_step_set_i32(e, track, 0, StepAttr::Active, 1);
                octocore_step_set_i32(e, track, 4, StepAttr::Active, 1);
            }
            let mut events = [Event::Cc { port: 0, ch: 0, cc: 0, val: 0, at_sample: 0 }; 64];
            let mut count = 0usize;
            // One warm-up pass, so anything that allocates once on first use is not counted.
            octocore_sender_push(s, Command::Play);
            octocore_engine_render(e, PARAMS, events.as_mut_ptr(), events.len(), &mut count);
            octocore_reader_claim(r);

            let before = allocations();
            let mut snapshots = 0;
            for k in 0..1000i32 {
                octocore_sender_push(s, Command::SetTrack { track: (k % 10) as u8, attr: TrackAttr::Pitch, value: 30 + k % 60 });
                octocore_engine_render(e, PARAMS, events.as_mut_ptr(), events.len(), &mut count);
                if (*octocore_reader_claim(r)).generation > 0 {
                    snapshots += 1;
                }
            }
            assert_eq!(allocations() - before, 0, "the audio, main and render paths must not allocate");
            assert_eq!(snapshots, 1000);
            octocore_sender_free(s);
            octocore_reader_free(r);
            octocore_engine_free(e);
        }
    }
}
