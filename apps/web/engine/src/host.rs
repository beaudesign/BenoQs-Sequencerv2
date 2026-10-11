//! The host: one engine, one panel controller and the byte buffers the page reads.
//!
//! State is per thread (the worklet is one thread) and held in a `Box` that is made once at `init`
//! and never moved, so the addresses of the buffers the page reads stay valid until the next
//! `init` or `reset`. Nothing here allocates after `init` except the controller's own return
//! values on an input (two small `Vec`s, only when a press produces a command or an intent).

use crate::encode::{led_byte, write_merged, EVENT_BYTES};
use octocore::domain::TRACK_COUNT;
use octocore::engine::{MAX_BPM, MAX_SAMPLE_RATE, MIN_BPM, MIN_SAMPLE_RATE, REALTIME_PER_RENDER};
use octocore::domain::STEP_COUNT;
use octocore::types::{Command, ControlId, Snapshot, StepAttr, TrackAttr, MAX_CONTROLS, MAX_EVENTS_PER_TICK};
use octocore::{Engine, EventBuffer, Realtime, RealtimeEvent, RenderContext};
use octoface::{Input, Layout, PageView, Panel, PanelMode};
use std::cell::{Cell, RefCell};

pub const ABI_VERSION: u32 = 2;
/// The records the events buffer holds: all the notes one render can make and all its clock messages (ADR-0009 decision 1).
pub const EVENT_CAPACITY: usize = MAX_EVENTS_PER_TICK + REALTIME_PER_RENDER;
/// The most frames one `render` call takes. The page asks for 128.
pub const MAX_FRAMES: u32 = 4096;

/// The 64-bit seed from the two words the page passes (WebAssembly numbers are 32-bit here).
pub fn seed(lo: u32, hi: u32) -> u64 {
    u64::from(lo) | u64::from(hi) << 32
}

pub const OK: u32 = 0;
pub const ERR_NOT_INITIALISED: u32 = 1;
pub const ERR_BAD_KIND: u32 = 2;
pub const ERR_LAYOUT_TEXT: u32 = 3;
pub const ERR_LAYOUT: u32 = 4;
pub const ERR_BAD_ARGUMENT: u32 = 5;

pub struct Host {
    engine: Engine,
    panel: Panel,
    sample_rate: f32,
    bpm: f32,
    buf: EventBuffer,
    snap: Snapshot,
    /// What the engine says in a render besides notes: the clock, Start, Continue and Stop.
    realtime: [RealtimeEvent; REALTIME_PER_RENDER],
    events: [u8; EVENT_CAPACITY * EVENT_BYTES],
    leds: [u8; MAX_CONTROLS],
    playheads: [u8; TRACK_COUNT],
    dropped_intents: u32,
}

thread_local! {
    static HOST: RefCell<Option<Box<Host>>> = const { RefCell::new(None) };
    static SCRATCH: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
    static MESSAGE_LEN: Cell<usize> = const { Cell::new(0) };
}

fn with_host<R>(f: impl FnOnce(&mut Host) -> R) -> Option<R> {
    HOST.with(|h| h.borrow_mut().as_mut().map(|b| f(b)))
}

/// Runs `f` on the scratch bytes. The page writes the layout and the spike's pattern here, and
/// reads messages back.
pub fn with_scratch<R>(f: impl FnOnce(&mut Vec<u8>) -> R) -> R {
    SCRATCH.with(|s| f(&mut s.borrow_mut()))
}

/// At least `len` bytes of scratch, and the address of the first. Valid until the next call.
pub fn scratch(len: usize) -> *mut u8 {
    with_scratch(|s| {
        if s.len() < len.max(1) {
            s.resize(len.max(1), 0);
        }
        s.as_mut_ptr()
    })
}

fn set_message(text: &str) {
    with_scratch(|s| {
        s.clear();
        s.extend_from_slice(text.as_bytes());
        if s.is_empty() {
            s.push(0);
        }
    });
    MESSAGE_LEN.with(|m| m.set(text.len()));
}

pub fn message_len() -> usize {
    MESSAGE_LEN.with(|m| m.get())
}

fn parse_layout(bytes: &[u8]) -> Result<Layout, (u32, String)> {
    let text = std::str::from_utf8(bytes).map_err(|e| (ERR_LAYOUT_TEXT, format!("the layout is not UTF-8: {e}")))?;
    let mut pairs: Vec<(u32, &str)> = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let bad = || (ERR_LAYOUT_TEXT, format!("layout line {}: expected `<n> <id>`, found `{line}`", i + 1));
        let (n, id) = line.split_once(' ').ok_or_else(bad)?;
        let n: u32 = n.trim().parse().map_err(|_| bad())?;
        pairs.push((n, id.trim()));
    }
    Layout::from_pairs(pairs).map_err(|e| (ERR_LAYOUT, e.to_string()))
}

/// Makes the engine and the panel. The layout is read from the first `layout_len` bytes of scratch.
/// On success the module is ready. On failure it is left uninitialised, and the reason is in
/// scratch (`message_len` bytes).
pub fn init(sample_rate: f32, seed: u64, layout_len: usize) -> u32 {
    let built = (|| {
        if !(sample_rate.is_finite() && (MIN_SAMPLE_RATE..=MAX_SAMPLE_RATE).contains(&sample_rate)) {
            return Err((ERR_BAD_ARGUMENT, format!("sample rate {sample_rate} is outside {MIN_SAMPLE_RATE} to {MAX_SAMPLE_RATE}")));
        }
        let bytes = with_scratch(|s| s.get(..layout_len).map(<[u8]>::to_vec));
        let bytes = bytes.ok_or((ERR_BAD_ARGUMENT, format!("layout length {layout_len} is more than the scratch memory")))?;
        let layout = parse_layout(&bytes)?;
        Ok(Box::new(Host {
            engine: Engine::new(seed),
            panel: Panel::new(layout),
            sample_rate,
            bpm: 120.0,
            buf: EventBuffer::new(),
            snap: Snapshot::zeroed(),
            realtime: [RealtimeEvent { msg: Realtime::Clock, at_sample: 0 }; REALTIME_PER_RENDER],
            events: [0; EVENT_CAPACITY * EVENT_BYTES],
            leds: [0; MAX_CONTROLS],
            playheads: [0; TRACK_COUNT],
            dropped_intents: 0,
        }))
    })();
    match built {
        Ok(host) => {
            HOST.with(|h| *h.borrow_mut() = Some(host));
            set_message("");
            OK
        }
        Err((code, message)) => {
            HOST.with(|h| *h.borrow_mut() = None);
            set_message(&message);
            code
        }
    }
}

pub fn reset() -> u32 {
    HOST.with(|h| *h.borrow_mut() = None);
    set_message("");
    OK
}

impl Host {
    fn input(&mut self, now_ms: f64, kind: u32, control: u32, detents: i32) -> u32 {
        let id = ControlId(control);
        let input = match kind {
            0 => Input::Down(id),
            1 => Input::Up(id),
            2 => Input::Turn { control: id, detents: detents.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16 },
            _ => return ERR_BAD_KIND,
        };
        let view = PageView::from_page(self.engine.grid.active_page());
        // `as u64` saturates, and a NaN becomes 0.
        let out = self.panel.input(now_ms as u64, input, &view);
        for c in out.commands {
            self.engine.handle_command(c);
        }
        self.dropped_intents = self.dropped_intents.saturating_add(out.intents.len() as u32);
        OK
    }

    fn refresh_leds(&mut self) -> bool {
        let view = PageView::from_page(self.engine.grid.active_page());
        let frame = self.panel.leds(&view);
        let mut changed = false;
        for (slot, led) in self.leds.iter_mut().zip(frame.as_slice()) {
            let b = led_byte(*led);
            if *slot != b {
                *slot = b;
                changed = true;
            }
        }
        changed
    }

    fn render(&mut self, frames: u32) -> u32 {
        let frames = frames.min(MAX_FRAMES);
        self.buf.clear();
        if frames == 0 {
            return 0;
        }
        let ctx = RenderContext { sample_rate: self.sample_rate, buffer_len: frames, bpm: self.bpm, playing: self.engine.is_running() };
        self.engine.render(&ctx, &mut self.buf);
        let realtime = self.engine.take_realtime(&mut self.realtime);
        let records = write_merged(&mut self.events, self.buf.as_slice(), &self.realtime[..realtime]);
        self.engine.snapshot(&mut self.snap);
        for (slot, p) in self.playheads.iter_mut().zip(self.snap.playheads.iter()) {
            *slot = p.step_index;
        }
        records as u32
    }

    fn status(&self) -> u32 {
        u32::from(self.engine.is_running()) | (u32::from(self.panel.mode() == PanelMode::Step) << 1)
    }
}

pub fn input(now_ms: f64, kind: u32, control: u32, detents: i32) -> u32 {
    with_host(|h| h.input(now_ms, kind, control, detents)).unwrap_or(ERR_NOT_INITIALISED)
}

/// The stopgap of ADR-0008 decision 4: the page, as the host, starts and stops the transport.
pub fn transport(play: bool) -> u32 {
    with_host(|h| h.engine.handle_command(if play { Command::Play } else { Command::Stop })).map_or(ERR_NOT_INITIALISED, |()| OK)
}

pub fn set_tempo(bpm: f32) -> u32 {
    with_host(|h| {
        if bpm.is_finite() && (MIN_BPM..=MAX_BPM).contains(&bpm) {
            h.bpm = bpm;
            OK
        } else {
            ERR_BAD_ARGUMENT
        }
    })
    .unwrap_or(ERR_NOT_INITIALISED)
}

/// One attribute of one track, through the engine's own `SetTrack` (ADR-0009 decision 7). `attr` is the attribute's place in
/// `TrackAttr::ALL`; a track or attribute that does not exist is refused, and the value is the engine's to clamp.
pub fn set_track(track: u32, attr: u32, value: i32) -> u32 {
    with_host(|h| {
        let track = u8::try_from(track).ok().filter(|t| usize::from(*t) < TRACK_COUNT);
        match (track, TrackAttr::from_u32(attr)) {
            (Some(track), Some(attr)) => {
                h.engine.handle_command(Command::SetTrack { track, attr, value });
                OK
            }
            _ => ERR_BAD_ARGUMENT,
        }
    })
    .unwrap_or(ERR_NOT_INITIALISED)
}

/// One attribute of one step, through the engine's own `SetStep` (P6, `specs/SPEC-0002/p6-runs-when-opened.md`). `attr` is the attribute's place in
/// `StepAttr::ALL`; a track, step or attribute that does not exist is refused, and the value is the engine's to clamp.
pub fn set_step(track: u32, step: u32, attr: u32, value: i32) -> u32 {
    with_host(|h| {
        let track = u8::try_from(track).ok().filter(|t| usize::from(*t) < TRACK_COUNT);
        let step = u8::try_from(step).ok().filter(|s| usize::from(*s) < STEP_COUNT);
        match (track, step, StepAttr::from_u32(attr)) {
            (Some(track), Some(step), Some(attr)) => {
                h.engine.handle_command(Command::SetStep { track, step, attr, value });
                OK
            }
            _ => ERR_BAD_ARGUMENT,
        }
    })
    .unwrap_or(ERR_NOT_INITIALISED)
}

/// Whether the engine is the MIDI clock master (ADR-0009 decision 2). Off at `init`.
pub fn set_clock(master: bool) -> u32 {
    with_host(|h| h.engine.set_clock_master(master)).map_or(ERR_NOT_INITIALISED, |()| OK)
}

/// Where the audio is on the engine's tick grid, in ticks, at the end of the last render. 0 before `init`.
pub fn tick_position() -> f64 {
    with_host(|h| h.engine.tick_position()).unwrap_or(0.0)
}

pub fn render(frames: u32) -> u32 {
    with_host(|h| h.render(frames)).unwrap_or(0)
}

pub fn refresh_leds() -> u32 {
    with_host(|h| u32::from(h.refresh_leds())).unwrap_or(0)
}

pub fn status() -> u32 {
    with_host(|h| h.status()).unwrap_or(0)
}

pub fn dropped_intents() -> u32 {
    with_host(|h| h.dropped_intents).unwrap_or(0)
}

pub fn events_ptr() -> *const u8 {
    with_host(|h| h.events.as_ptr()).unwrap_or(std::ptr::null())
}

pub fn leds_ptr() -> *const u8 {
    with_host(|h| h.leds.as_ptr()).unwrap_or(std::ptr::null())
}

pub fn playheads_ptr() -> *const u8 {
    with_host(|h| h.playheads.as_ptr()).unwrap_or(std::ptr::null())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_seed_is_the_two_words_joined_low_first() {
        assert_eq!(seed(1, 0), 1);
        assert_eq!(seed(0, 1), 1 << 32);
        assert_eq!(seed(0xdead_beef, 0x1234_5678), 0x1234_5678_dead_beef);
    }
}
