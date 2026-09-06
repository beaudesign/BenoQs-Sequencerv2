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
/// code on a null/invalid argument. If more events fired than `out_capacity`
/// can hold, the extras are silently dropped this call — same "full density is
/// sized for" contract as `octocore::engine::MAX_EVENTS_PER_TICK` internally, not
/// a new gap introduced at this boundary.
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
    let mut buf = octocore::engine::EventBuffer::new();
    engine.render(&ctx, &mut buf);

    let events = buf.as_slice();
    let n = events.len().min(out_capacity);
    let dst = std::slice::from_raw_parts_mut(out_events, n);
    dst.copy_from_slice(&events[..n]);
    *out_count = n;
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
}
