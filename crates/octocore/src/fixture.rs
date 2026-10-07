//! Runner for the conformance fixture DSL described in
//! `docs/03-sequencer-core.md` §8. Fixtures live in `tests/conformance/**/*.fixture`
//! at the repo root (not under this crate) so they stay a plain, hand-writable text
//! format shared by whatever eventually consumes the manual's worked examples.
//!
//! This is test-only infrastructure (parsing happens off the audio thread), so
//! unlike `engine`/`domain` it freely uses `std` — `Vec`/`String` are fine here.
//!
//! DSL v2 (SPEC-0001 O9, WENGE-0009) adds the sample domain to the tick-domain DSL above.
//! The v1 directives (`play N step`, `expect note ...`) are unchanged.
//!
//! ```text
//! bpm 133                       tempo for `render` (default 120)
//! samplerate 44100              default 48000
//! play | stop | reset           transport commands (Command::Play / Stop / Reset)
//! render 40 buffers buffer=256  render 40 buffers of 256 samples
//! render 10 s buffer=512        render 10 seconds of audio in 512-sample buffers
//! clear events                  forget the events collected so far
//! expect event on|off|cc|bend|pressure [port=1|2] [ch=1..16] [note=N] [vel=N] [cc=N]
//!                        [val=N] [at=N|lo..hi] [len=N|lo..hi] [count=N] [absent]
//! expect balance                every NoteOn has a NoteOff per (port, channel, note)
//! track T mch V                 MIDI channel 1..32
//! track T vel V                 base velocity 0..127 (DSL v3, SPEC-0001 O10)
//! track T dir V                 direction code (1 forward, 2 reverse, 4 brownian, 5 random, ...)
//! track T grv V                 groove 0..16 (even values draw a random delay per even step)
//! track T mcc none|bend|pressure|cc N   what the track's step MCC values send
//! track T step S|all attr V     as before; `len` (ticks), `lenmul` and `mcc` are new
//! ```
//!
//! `val` matches the CC value, the pressure value, or the 14-bit bend value (8192 is centre).
//! `at` is an absolute sample position since the first `render`. `count=N` is exact,
//! `count=lo..hi` a range. Without `count=`, an `expect event` line passes if at least one
//! event matches; `absent` means none.

use crate::domain::*;
use crate::engine::{Diagnostics, Engine, EventBuffer, NoteFire, RenderContext};
use crate::types::{Command, Event};

/// The most steps one `play N step` directive runs: 1.2 million ticks. The count times 12
/// ticks has to fit a `u32`, and a typo should be an error, not minutes of ticking.
const MAX_PLAY_STEPS: u32 = 100_000;

/// One collected output event with its absolute sample position.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Emitted {
    at: u64,
    ev: Event,
}

fn parse_range(v: &str) -> Option<(i64, i64)> {
    match v.split_once("..") {
        Some((a, b)) => Some((a.parse().ok()?, b.parse().ok()?)),
        None => {
            let n: i64 = v.parse().ok()?;
            Some((n, n))
        }
    }
}

fn at_sample(e: &Event) -> u32 {
    match *e {
        Event::NoteOn { at_sample, .. }
        | Event::NoteOff { at_sample, .. }
        | Event::Cc { at_sample, .. }
        | Event::PitchBend { at_sample, .. }
        | Event::ChannelPressure { at_sample, .. } => at_sample,
    }
}

fn describe(events: &[Emitted]) -> String {
    let shown: Vec<String> = events.iter().take(24).map(|e| format!("{}:{:?}", e.at, e.ev)).collect();
    let more = if events.len() > 24 { format!(" ... and {} more", events.len() - 24) } else { String::new() };
    format!("[{}{}]", shown.join(", "), more)
}

#[derive(Default)]
struct EventFilter {
    kind: Option<String>,
    port: Option<i64>,
    ch: Option<i64>,
    note: Option<i64>,
    vel: Option<i64>,
    cc: Option<i64>,
    val: Option<i64>,
    at: Option<(i64, i64)>,
    len: Option<(i64, i64)>,
    count: Option<(usize, usize)>,
}

impl EventFilter {
    fn matches(&self, e: &Emitted) -> bool {
        // `val` is the CC value, the pressure value or the 14-bit bend value.
        let (kind, port, ch, note, vel, cc, val): (&str, u8, u8, Option<u8>, Option<u8>, Option<u8>, Option<i64>) = match e.ev {
            Event::NoteOn { port, ch, note, vel, .. } => ("on", port, ch, Some(note), Some(vel), None, None),
            Event::NoteOff { port, ch, note, .. } => ("off", port, ch, Some(note), None, None, None),
            Event::Cc { port, ch, cc, val, .. } => ("cc", port, ch, None, None, Some(cc), Some(val as i64)),
            Event::PitchBend { port, ch, value, .. } => ("bend", port, ch, None, None, None, Some(value as i64)),
            Event::ChannelPressure { port, ch, value, .. } => ("pressure", port, ch, None, None, None, Some(value as i64)),
        };
        let eq = |want: Option<i64>, got: Option<u8>| match (want, got) {
            (None, _) => true,
            (Some(w), Some(g)) => w == g as i64,
            (Some(_), None) => false,
        };
        self.kind.as_deref().map_or(true, |k| k == kind)
            && eq(self.port, Some(port))
            && eq(self.ch, Some(ch))
            && eq(self.note, note)
            && eq(self.vel, vel)
            && eq(self.cc, cc)
            && match (self.val, val) {
                (None, _) => true,
                (Some(w), Some(g)) => w == g,
                (Some(_), None) => false,
            }
            && self.at.map_or(true, |(lo, hi)| (lo..=hi).contains(&(e.at as i64)))
    }
}

fn check_expect_event(events: &[Emitted], f: &EventFilter) -> Result<(), String> {
    let mut hits: Vec<&Emitted> = events.iter().filter(|e| f.matches(e)).collect();
    if let Some((lo, hi)) = f.len {
        // Length of a note is the time to the next NoteOff of the same key.
        hits.retain(|on| {
            let Event::NoteOn { port, ch, note, .. } = on.ev else { return false };
            events
                .iter()
                .filter(|o| o.at >= on.at)
                .find(|o| matches!(o.ev, Event::NoteOff { port: p, ch: c, note: n, .. } if p == port && c == ch && n == note))
                .map_or(false, |off| (lo..=hi).contains(&((off.at - on.at) as i64)))
        });
    }
    let ok = match f.count {
        Some((lo, hi)) => (lo..=hi).contains(&hits.len()),
        None => !hits.is_empty(),
    };
    if ok {
        Ok(())
    } else {
        let want = match f.count {
            Some((lo, hi)) if lo == hi => lo.to_string(),
            Some((lo, hi)) => format!("{lo} to {hi}"),
            None => "at least 1".to_string(),
        };
        Err(format!("expected {want} matching event(s), found {}. events: {}", hits.len(), describe(events)))
    }
}

fn check_balance(events: &[Emitted]) -> Result<(), String> {
    let mut net: std::collections::BTreeMap<(u8, u8, u8), i64> = std::collections::BTreeMap::new();
    for e in events {
        match e.ev {
            Event::NoteOn { port, ch, note, .. } => *net.entry((port, ch, note)).or_default() += 1,
            Event::NoteOff { port, ch, note, .. } => *net.entry((port, ch, note)).or_default() -= 1,
            Event::Cc { .. } | Event::PitchBend { .. } | Event::ChannelPressure { .. } => {}
        }
    }
    let unbalanced: Vec<String> =
        net.iter().filter(|(_, n)| **n != 0).map(|((p, c, n), d)| format!("port {p} ch {c} note {n}: {d:+}")).collect();
    if unbalanced.is_empty() {
        Ok(())
    } else {
        Err(format!("NoteOn and NoteOff do not balance: {}", unbalanced.join("; ")))
    }
}

/// Everything a script rendered, for tools that want the event stream itself rather than a
/// pass or a fail (`octorun`). `expect` lines still run and still fail the script.
#[derive(Debug, Clone, PartialEq)]
pub struct Played {
    /// Every event the `render` directives produced, in emission order, with its absolute
    /// sample position. `clear events` forgets events for `expect` only, never here.
    pub events: Vec<(u64, Event)>,
    /// `(sample, bpm)`: the tempo in force from that sample on. Starts with `(0, 120.0)`.
    pub tempo: Vec<(u64, f32)>,
    pub sample_rate: f32,
    /// Samples rendered in total.
    pub samples: u64,
    /// False if a `samplerate` directive changed the rate after rendering began. The sample
    /// positions above then mix two rates and a converter must refuse them.
    pub sample_rate_constant: bool,
    /// The engine's counters at the end of the script: events it refused, deferred or sent
    /// late. A pattern that overloads the event queue plays incompletely, and a tool that
    /// records the stream should say so.
    pub diagnostics: Diagnostics,
}

/// Ceilings for a script run by a tool, so a typo (`render 10000000 s`) is an error and not
/// an out-of-memory. Fixtures run without ceilings.
#[derive(Debug, Clone, Copy)]
pub struct Limits {
    pub max_samples: u64,
    pub max_events: usize,
}

impl Limits {
    pub const NONE: Limits = Limits { max_samples: u64::MAX, max_events: usize::MAX };
}

pub fn run_fixture(source: &str) -> Result<(), String> {
    run_script(source, Limits::NONE).map(|_| ())
}

/// Runs a script (the fixture DSL, v1 to v3) and returns what it rendered.
pub fn run_script(source: &str, limits: Limits) -> Result<Played, String> {
    let mut engine = Engine::new(0);
    let mut all: Vec<(u64, Event)> = Vec::new();
    let mut tempo: Vec<(u64, f32)> = vec![(0, 120.0)];
    let mut sample_rate_constant = true;
    let mut fired: Vec<NoteFire> = Vec::new();
    // Sample-domain state (DSL v2).
    let mut bpm: f32 = 120.0;
    let mut sample_rate: f32 = 48_000.0;
    let mut playing = false;
    let mut clock: u64 = 0;
    let mut events: Vec<Emitted> = Vec::new();

    for (lineno, raw_line) in source.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let ctx = || format!("line {}: `{}`", lineno + 1, line);
        let tokens: Vec<&str> = line.split_whitespace().collect();

        match tokens.as_slice() {
            ["seed", n] => {
                let seed: u64 = n.parse().map_err(|_| format!("{}: bad seed", ctx()))?;
                engine = Engine::new(seed);
            }
            ["page.tracks", list, "enabled"] => {
                for t in list.split(',') {
                    let ti: usize = t.trim().parse().map_err(|_| format!("{}: bad track index", ctx()))?;
                    engine.grid.active_page_mut().tracks[ti].steps[0].active = true;
                }
            }
            ["track", t, "role", role] => {
                let ti: usize = t.parse().map_err(|_| format!("{}: bad track index", ctx()))?;
                let track = &mut engine.grid.active_page_mut().tracks[ti];
                let (feeder, listener) = match *role {
                    "feeder" => (true, false),
                    "listener" => (false, true),
                    "both" => (true, true),
                    "none" => (false, false),
                    other => return Err(format!("{}: unknown role `{}`", ctx(), other)),
                };
                track.is_feeder = feeder;
                track.is_listener = listener;
            }
            ["track", t, "step", s, attr, value] => {
                let ti: usize = t.parse().map_err(|_| format!("{}: bad track index", ctx()))?;
                if ti >= TRACK_COUNT {
                    return Err(format!("{}: track index out of range", ctx()));
                }
                let range = if *s == "all" {
                    0..STEP_COUNT
                } else {
                    let si: usize = s.parse().map_err(|_| format!("{}: bad step index", ctx()))?;
                    if si >= STEP_COUNT {
                        return Err(format!("{}: step index out of range", ctx()));
                    }
                    si..si + 1
                };
                let v: i32 = value.trim_start_matches('+').parse().map_err(|_| format!("{}: bad value", ctx()))?;
                for si in range {
                    let step = &mut engine.grid.active_page_mut().tracks[ti].steps[si];
                    match *attr {
                        "pit" => step.pitch_offset = v as i8,
                        "vel" => step.velocity_offset = v as i8,
                        "sta" => step.start_offset = v as i8,
                        "strum" => step.strum = v as i8,
                        "amt" => step.amount = v as i8,
                        "phrase" => step.phrase = if v == 0 { None } else { Some(v as u8) },
                        "skip" => step.skip = v != 0,
                        "active" => step.active = v != 0,
                        "len" => step.length_ticks = v.clamp(1, 255) as u8,
                        "lenmul" => step.length_multiplier = v.clamp(1, 8) as u8,
                        "mcc" => step.mcc_value = Some(v.clamp(0, 127) as u8),
                        other => return Err(format!("{}: unknown step attribute `{}`", ctx(), other)),
                    }
                }
            }
            ["track", t, "vel", v] => {
                let ti: usize = t.parse().map_err(|_| format!("{}: bad track index", ctx()))?;
                let vel: u8 = v.parse().ok().filter(|v| *v <= 127).ok_or_else(|| format!("{}: velocity is 0..=127", ctx()))?;
                if ti >= TRACK_COUNT {
                    return Err(format!("{}: track index out of range", ctx()));
                }
                engine.grid.active_page_mut().tracks[ti].velocity = vel;
            }
            ["track", t, "mcc", kind @ ("none" | "bend" | "pressure")] => {
                let ti: usize = t.parse().map_err(|_| format!("{}: bad track index", ctx()))?;
                if ti >= TRACK_COUNT {
                    return Err(format!("{}: track index out of range", ctx()));
                }
                engine.grid.active_page_mut().tracks[ti].mcc = match *kind {
                    "bend" => Mcc::Bend,
                    "pressure" => Mcc::Pressure,
                    _ => Mcc::None,
                };
            }
            ["track", t, "mcc", "cc", n] => {
                let ti: usize = t.parse().map_err(|_| format!("{}: bad track index", ctx()))?;
                let cc: u8 = n.parse().ok().filter(|n| *n <= 127).ok_or_else(|| format!("{}: controller is 0..=127", ctx()))?;
                if ti >= TRACK_COUNT {
                    return Err(format!("{}: track index out of range", ctx()));
                }
                engine.grid.active_page_mut().tracks[ti].mcc = Mcc::Cc(cc);
            }
            ["track", t, attr @ ("dir" | "grv"), v] => {
                let ti: usize = t.parse().map_err(|_| format!("{}: bad track index", ctx()))?;
                if ti >= TRACK_COUNT {
                    return Err(format!("{}: track index out of range", ctx()));
                }
                let max = if *attr == "dir" { 255 } else { 16 };
                let n: u8 = v.parse().ok().filter(|n| *n <= max).ok_or_else(|| format!("{}: {} is 0..={}", ctx(), attr, max))?;
                let track = &mut engine.grid.active_page_mut().tracks[ti];
                if *attr == "dir" {
                    track.direction_raw = n;
                } else {
                    track.groove = n;
                }
            }
            ["track", t, "mch", v] => {
                let ti: usize = t.parse().map_err(|_| format!("{}: bad track index", ctx()))?;
                let ch: u8 = v.parse().map_err(|_| format!("{}: bad channel", ctx()))?;
                if ti >= TRACK_COUNT || !(1..=32).contains(&ch) {
                    return Err(format!("{}: track or channel out of range (channel is 1..=32)", ctx()));
                }
                engine.grid.active_page_mut().tracks[ti].midi_channel = ch;
            }
            ["bpm", v] => {
                bpm = v.parse().map_err(|_| format!("{}: bad bpm", ctx()))?;
                match tempo.last_mut() {
                    Some(last) if last.0 == clock => last.1 = bpm,
                    _ => tempo.push((clock, bpm)),
                }
            }
            ["samplerate", v] => {
                let rate: f32 = v.parse().map_err(|_| format!("{}: bad sample rate", ctx()))?;
                if clock > 0 && rate != sample_rate {
                    sample_rate_constant = false;
                }
                sample_rate = rate;
            }
            ["play"] => {
                engine.handle_command(Command::Play);
                playing = true;
            }
            ["stop"] => {
                engine.handle_command(Command::Stop);
                playing = false;
            }
            ["reset"] => {
                engine.handle_command(Command::Reset);
                playing = false;
            }
            ["render", n, unit, buf] => {
                let buffer_len: u32 = buf
                    .strip_prefix("buffer=")
                    .and_then(|v| v.parse().ok())
                    .filter(|l| *l > 0)
                    .ok_or_else(|| format!("{}: render needs buffer=<samples>", ctx()))?;
                let count: u64 = match *unit {
                    "buffers" => n.parse().map_err(|_| format!("{}: bad buffer count", ctx()))?,
                    "s" => {
                        let secs: f64 = n.parse().map_err(|_| format!("{}: bad seconds", ctx()))?;
                        (secs * sample_rate as f64 / buffer_len as f64).ceil() as u64
                    }
                    other => return Err(format!("{}: render unit must be `buffers` or `s`, not `{}`", ctx(), other)),
                };
                if clock.saturating_add(count.saturating_mul(buffer_len as u64)) > limits.max_samples {
                    return Err(format!("{}: renders past the {} sample limit", ctx(), limits.max_samples));
                }
                let mut out = EventBuffer::new();
                for _ in 0..count {
                    out.clear();
                    engine.render(&RenderContext { sample_rate, buffer_len, bpm, playing }, &mut out);
                    for e in out.as_slice() {
                        events.push(Emitted { at: clock + at_sample(e) as u64, ev: *e });
                        all.push((clock + at_sample(e) as u64, *e));
                    }
                    if all.len() > limits.max_events {
                        return Err(format!("{}: more than {} events", ctx(), limits.max_events));
                    }
                    clock += buffer_len as u64;
                }
            }
            ["clear", "events"] => events.clear(),
            ["expect", "balance"] => check_balance(&events).map_err(|e| format!("{}: {}", ctx(), e))?,
            ["expect", "event", kind, rest @ ..] => {
                if !matches!(*kind, "on" | "off" | "cc" | "bend" | "pressure") {
                    return Err(format!("{}: event kind must be on, off, cc, bend or pressure", ctx()));
                }
                let mut f = EventFilter { kind: Some(kind.to_string()), ..Default::default() };
                for kv in rest {
                    if *kv == "absent" {
                        f.count = Some((0, 0));
                        continue;
                    }
                    let (k, v) = kv.split_once('=').ok_or_else(|| format!("{}: bad expect clause `{}`", ctx(), kv))?;
                    let num = || v.parse::<i64>().map_err(|_| format!("{}: bad value in `{}`", ctx(), kv));
                    let range = || parse_range(v).ok_or_else(|| format!("{}: bad range in `{}`", ctx(), kv));
                    match k {
                        "port" => f.port = Some(num()?),
                        "ch" => f.ch = Some(num()?),
                        "note" => f.note = Some(num()?),
                        "vel" => f.vel = Some(num()?),
                        "cc" => f.cc = Some(num()?),
                        "val" => f.val = Some(num()?),
                        "at" => f.at = Some(range()?),
                        "len" => f.len = Some(range()?),
                        "count" => {
                            let (lo, hi) = range()?;
                            if lo < 0 || hi < lo {
                                return Err(format!("{}: bad count range in `{}`", ctx(), kv));
                            }
                            f.count = Some((lo as usize, hi as usize));
                        }
                        other => return Err(format!("{}: unknown expect event key `{}`", ctx(), other)),
                    }
                }
                check_expect_event(&events, &f).map_err(|e| format!("{}: {}", ctx(), e))?;
            }
            ["phrase", n, "note", i, attr, value] => {
                let pi: usize = n.parse().map_err(|_| format!("{}: bad phrase index", ctx()))?;
                let ni: usize = i.parse().map_err(|_| format!("{}: bad phrase-note index", ctx()))?;
                if pi >= PHRASE_COUNT || ni >= PHRASE_NOTE_COUNT {
                    return Err(format!("{}: phrase/note out of range", ctx()));
                }
                let note = &mut engine.grid.phrases[pi].notes[ni];
                let v: i32 = value.trim_start_matches('+').parse().map_err(|_| format!("{}: bad value", ctx()))?;
                match *attr {
                    "pit" => note.pitch_offset = v as i8,
                    "vel" => note.velocity_offset = v as i8,
                    "len" => note.length_ticks = v.clamp(0, 255) as u8,
                    "sta" => note.start_ticks = v.clamp(0, 255) as u8,
                    "enabled" => note.enabled = v != 0,
                    other => return Err(format!("{}: unknown phrase-note attribute `{}`", ctx(), other)),
                }
            }
            ["flatten", list] => {
                let mut selected = Vec::new();
                for t in list.split(',') {
                    selected.push(t.trim().parse::<u8>().map_err(|_| format!("{}: bad track index", ctx()))?);
                }
                engine
                    .grid
                    .active_page_mut()
                    .apply_flatten(&selected)
                    .map_err(|e| format!("{}: {}", ctx(), e))?;
            }
            ["play", n, step_word] if step_word.starts_with("step") => {
                // A "step" is DEFAULT_STEP_TICKS (12) ticks (a 1/16 note at
                // the default 1x multiplier) — one call to `step_once_for_test`
                // is one *tick*, not one step; see docs/03-sequencer-core.md §2.
                let steps: u32 = n.parse().map_err(|_| format!("{}: bad step count", ctx()))?;
                if steps > MAX_PLAY_STEPS {
                    return Err(format!("{}: play {steps} step is too long, the most is {MAX_PLAY_STEPS} steps", ctx()));
                }
                let ticks = steps * DEFAULT_STEP_TICKS;
                for _ in 0..ticks {
                    engine.step_once_for_test();
                    fired.extend_from_slice(engine.last_tick_fires.as_slice());
                }
            }
            ["expect", "note", rest @ ..] => {
                let mut want_track: Option<usize> = None;
                let mut want_pit_offset: Option<i32> = None;
                for kv in rest {
                    let (k, v) = kv.split_once('=').ok_or_else(|| format!("{}: bad expect clause `{}`", ctx(), kv))?;
                    match k {
                        "track" => want_track = Some(v.parse().map_err(|_| format!("{}: bad track", ctx()))?),
                        "pit" => {
                            want_pit_offset =
                                Some(v.trim_start_matches('+').parse().map_err(|_| format!("{}: bad pit", ctx()))?)
                        }
                        other => return Err(format!("{}: unknown expect key `{}`", ctx(), other)),
                    }
                }
                let want_track = want_track.ok_or_else(|| format!("{}: expect needs track=", ctx()))?;
                let base_pitch = engine.grid.active_page().tracks[want_track].pitch as i32;
                let expected_pitch = want_pit_offset.map(|off| base_pitch + off);

                let matched = fired.iter().any(|f| {
                    f.track as usize == want_track && expected_pitch.map(|p| p == f.pitch as i32).unwrap_or(true)
                });
                if !matched {
                    return Err(format!(
                        "{}: no fired note matched track={} pit_offset={:?} (base_pitch={}); fired={:?}",
                        ctx(),
                        want_track,
                        want_pit_offset,
                        base_pitch,
                        fired.iter().map(|f| (f.track, f.pitch)).collect::<Vec<_>>()
                    ));
                }
            }
            other => return Err(format!("{}: unrecognised directive: {:?}", ctx(), other)),
        }
    }

    Ok(Played { events: all, tempo, sample_rate, samples: clock, sample_rate_constant, diagnostics: engine.diagnostics() })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- DSL v2 (WENGE-0009). These test the language, not the engine. ----

    const ONE_NOTE: &str = "\
        seed 1\n\
        page.tracks 0 enabled\n";

    fn v2(body: &str) -> Result<(), String> {
        run_fixture(&format!("{ONE_NOTE}{body}"))
    }

    #[test]
    fn v2_render_collects_note_on_and_off_with_the_right_length() {
        // 120 BPM at 48 kHz is 500 samples per tick (a quarter note is 48 ticks), so a 12-tick step is 6000 samples.
        // 32 buffers of 512 is 16,384 samples: one note, on and off (the pattern repeats every 96,000 samples).
        v2("play\nrender 32 buffers buffer=512\n\
            expect event on count=1\n\
            expect event off count=1\n\
            expect event on len=6000\n\
            expect event on at=0..8191\n")
            .unwrap();
    }

    #[test]
    fn v2_render_seconds_rounds_up_to_whole_buffers() {
        v2("play\nrender 0.5 s buffer=1000\nexpect event on count=1\n").unwrap();
    }

    #[test]
    fn v2_absent_and_count_are_checked() {
        let seen = "play\nrender 32 buffers buffer=512\n";
        assert!(v2(&format!("{seen}expect event on absent\n")).is_err());
        assert!(v2(&format!("{seen}expect event on count=2\n")).is_err());
        v2(&format!("{seen}expect event cc absent\n")).unwrap();
        v2("play\nrender 1 buffers buffer=64\nexpect event on absent\n").unwrap();
    }

    #[test]
    fn v2_filters_by_port_channel_note_and_velocity() {
        v2("track 0 mch 17\nplay\nrender 32 buffers buffer=512\n\
            expect event on port=2 ch=1 count=1\n\
            expect event on port=1 absent\n")
            .unwrap();
        assert!(v2("play\nrender 32 buffers buffer=512\nexpect event on vel=0\n").is_err());
    }

    #[test]
    fn v2_balance_catches_a_note_still_sounding() {
        // At 5632 samples (11 buffers) the note is on and its off, a step (6000 samples) after it, is not.
        let err = v2("play\nrender 11 buffers buffer=512\nexpect balance\n").unwrap_err();
        assert!(err.contains("do not balance"), "{err}");
        v2("play\nrender 32 buffers buffer=512\nexpect balance\n").unwrap();
    }

    #[test]
    fn v2_filter_matching_is_exact_on_the_pure_functions() {
        let on = |at, port, ch, note, vel| Emitted { at, ev: Event::NoteOn { port, ch, note, vel, at_sample: 0 } };
        let off = |at, port, ch, note| Emitted { at, ev: Event::NoteOff { port, ch, note, at_sample: 0 } };
        let cc = |at, port, ch, cc, val| Emitted { at, ev: Event::Cc { port, ch, cc, val, at_sample: 0 } };
        let events = [on(10, 1, 1, 60, 100), off(110, 1, 1, 60), on(20, 1, 2, 60, 90), cc(30, 2, 3, 123, 0)];

        let f = |kind: &str| EventFilter { kind: Some(kind.into()), ..Default::default() };
        assert!(check_expect_event(&events, &EventFilter { count: Some((2, 2)), ..f("on") }).is_ok());
        assert!(check_expect_event(&events, &EventFilter { ch: Some(2), vel: Some(90), count: Some((1, 1)), ..f("on") }).is_ok());
        assert!(check_expect_event(&events, &EventFilter { at: Some((15, 25)), count: Some((1, 1)), ..f("on") }).is_ok());
        assert!(check_expect_event(&events, &EventFilter { at: Some((11, 19)), ..f("on") }).is_err());
        assert!(check_expect_event(&events, &EventFilter { cc: Some(123), port: Some(2), ch: Some(3), val: Some(0), ..f("cc") }).is_ok());
        assert!(check_expect_event(&events, &EventFilter { cc: Some(64), ..f("cc") }).is_err());
        // len is measured to the next NoteOff of the same (port, ch, note): 100 for ch 1, none for ch 2.
        assert!(check_expect_event(&events, &EventFilter { len: Some((100, 100)), count: Some((1, 1)), ..f("on") }).is_ok());
        assert!(check_expect_event(&events, &EventFilter { ch: Some(2), len: Some((0, 1_000_000)), ..f("on") }).is_err());
        assert!(check_expect_event(&events, &EventFilter { count: Some((1, 3)), ..f("on") }).is_ok());
        assert!(check_expect_event(&events, &EventFilter { count: Some((3, 5)), ..f("on") }).is_err());
        // Balance: ch 2 note 60 never turns off.
        assert!(check_balance(&events).is_err());
        assert!(check_balance(&events[..2]).is_ok());
        assert!(check_balance(&[]).is_ok());
    }

    #[test]
    fn v2_clear_events_forgets_earlier_output() {
        v2("play\nrender 1 buffers buffer=512\nclear events\nexpect event on absent\n").unwrap();
    }

    #[test]
    fn v2_step_all_and_len_attributes() {
        run_fixture(
            "seed 1\npage.tracks 0 enabled\ntrack 0 step all active 1\ntrack 0 step all len 6\n\
             play\nrender 2 s buffer=512\nexpect event on len=3000\n",
        )
        .unwrap();
    }

    #[test]
    fn v2_bad_input_is_an_error_not_a_panic() {
        for bad in [
            "render 1 buffers\n",
            "render 1 buffers buffer=0\n",
            "render x buffers buffer=64\n",
            "render 1 hours buffer=64\n",
            "expect event note\n",
            "expect event on ch\n",
            "expect event on wat=1\n",
            "expect event on count=3..1\n",
            "expect event on count=-1\n",
            "expect event on at=5..x\n",
            "track 99 mch 1\n",
            "track 0 mch 33\n",
            "track 0 step 16 active 1\n",
            "track 0 step all bogus 1\n",
            "bpm fast\n",
        ] {
            assert!(v2(bad).is_err(), "should reject: {bad}");
        }
    }

    /// Ref: CE v5.30 p.59-60. Track 9 (feeder, step PIT +3) and track 5 (a
    /// *listening feeder* — both a listener of track 9 and a feeder of track 0,
    /// step PIT +1) both independently reach track 0 (the effector sums *every*
    /// feeder above a listener, not just the nearest one — p.59-60's own
    /// multi-feeder dry-run example works the same way). Track 5 must forward
    /// its *post-modulation* value ("the resulting values will modulate the
    /// corresponding listeners below it", p.59) — own +1 plus the +3 it itself
    /// received from track 9 — so track 0 sees track 9's direct +3, plus track
    /// 5's forwarded +4, for a total of +7. Before the post-modulation-publish
    /// fix, this fixture asserted +4 (track 5 forwarding its raw +1 instead);
    /// the value changing here *is* the regression test for that fix.
    #[test]
    fn role_both_receives_and_publishes() {
        let fixture = "\
            seed 1\n\
            page.tracks 9,5,0 enabled\n\
            track 9 role feeder\n\
            track 5 role both\n\
            track 0 role listener\n\
            track 9 step 0 pit +3\n\
            track 5 step 0 pit +1\n\
            play 1 step\n\
            expect note track=0 pit=+7\n";
        run_fixture(fixture).unwrap();
    }

    /// A track with no role at all (neither feeder nor listener) sits outside
    /// the effector entirely and must not receive a feeder's offsets, even
    /// though the additive-down-the-index model would otherwise sum every
    /// feeder above it regardless of what's in between. Not a direct manual
    /// quote — see the AMBIGUITIES.md entry this guards.
    #[test]
    fn non_listener_does_not_receive_effector_feed() {
        let fixture = "\
            seed 1\n\
            page.tracks 9,0 enabled\n\
            track 9 role feeder\n\
            track 9 step 0 pit +5\n\
            play 1 step\n\
            expect note track=0 pit=+0\n";
        run_fixture(fixture).unwrap();
    }

    // ------------------------------------------------------------ run_script (WENGE-0007)

    const SCRIPT_ONE_NOTE: &str = "seed 1\nbpm 120\ntrack 0 step 0 active 1\nplay\nrender 20 buffers buffer=512\n";

    #[test]
    fn run_script_returns_every_event_even_after_clear_events() {
        let src = format!("{SCRIPT_ONE_NOTE}clear events\nrender 20 buffers buffer=512\nexpect event on absent\n");
        let played = run_script(&src, Limits::NONE).unwrap();
        assert_eq!(played.events.len(), 2, "the NoteOn and NoteOff from before the clear: {:?}", played.events);
        assert_eq!(played.samples, 40 * 512);
        assert_eq!(played.tempo, vec![(0, 120.0)]);
        assert!(played.sample_rate_constant);
    }

    #[test]
    fn run_script_records_tempo_changes_at_the_sample_they_take_effect() {
        let src = "bpm 100\nrender 2 buffers buffer=256\nbpm 140\nbpm 150\nrender 1 buffers buffer=256\n";
        let played = run_script(src, Limits::NONE).unwrap();
        assert_eq!(played.tempo, vec![(0, 100.0), (512, 150.0)], "a second bpm at the same sample replaces the first");
    }

    #[test]
    fn run_script_flags_a_sample_rate_change_after_rendering_began() {
        let ok = run_script("samplerate 44100\nrender 1 buffers buffer=64\nsamplerate 44100\n", Limits::NONE).unwrap();
        assert!(ok.sample_rate_constant && ok.sample_rate == 44_100.0);
        let late = run_script("render 1 buffers buffer=64\nsamplerate 96000\n", Limits::NONE).unwrap();
        assert!(!late.sample_rate_constant);
    }

    #[test]
    fn run_script_enforces_its_limits_and_still_runs_expects() {
        let lim = Limits { max_samples: 1000, max_events: 10 };
        assert!(run_script("render 2 buffers buffer=512\n", lim).unwrap_err().contains("sample limit"));
        let dense = "seed 1\ntrack 0 step all active 1\ntrack 0 step all len 1\nplay\nrender 4 s buffer=64\n";
        assert!(run_script(dense, Limits { max_samples: u64::MAX, max_events: 10 }).unwrap_err().contains("events"));
        assert!(run_script("expect event on\n", Limits::NONE).is_err(), "an expect that fails fails the script");
    }

    #[test]
    fn v3_dir_and_grv_set_the_track_and_reject_bad_values() {
        assert!(run_script("track 0 dir 5\ntrack 0 grv 16\n", Limits::NONE).is_ok());
        assert!(run_script("track 0 grv 17\n", Limits::NONE).unwrap_err().contains("grv is 0..=16"));
        assert!(run_script("track 10 dir 1\n", Limits::NONE).unwrap_err().contains("out of range"));
    }

    #[test]
    fn a_play_count_that_overflows_is_an_error_not_a_panic_or_a_wrap() {
        // 400,000,000 steps is 4.8 billion ticks, more than a u32 holds. It used to
        // overflow (a panic in a debug build, a wrap to a short run in a release build).
        for n in ["400000000", "4294967295", "100001"] {
            let e = run_fixture(&format!("play {n} step\n")).unwrap_err();
            assert!(e.contains("play") && e.contains("too long"), "{n}: {e}");
        }
        assert!(run_fixture("play 1 step\n").is_ok());
    }
}
