//! The null host (SPEC-0001 O4 release plan, section 4; WENGE-0004 step 1).
//!
//! A host with no plug-in and no wall clock: it hands the engine buffers of a chosen size and
//! reports a tempo taken from an analytic tempo curve, then compares what the engine emits with
//! the analytic answer. It is a test helper. It changes no engine code, and what it measures
//! is the engine as it is on the parent commit, which is the point: the guard tests built on it
//! must pass *before* any O4 code exists, or they prove nothing.
//!
//! **Every call it makes is recorded** as one NDJSON line (`trace_ndjson`), so a failing run
//! can be replayed exactly (`NullHost::replay`). That is the "host clock trace" that
//! `product.md` invariants name. The `pos` field is `null` until the engine can be given a
//! position (PR 4c).
//!
//! **The tick length is read from the engine's own constants** (`TICKS_PER_QUARTER`,
//! `DEFAULT_STEP_TICKS`), never written here. If the owner decides D0 (the tick is four times too
//! short against the manual) and the constant changes, this helper follows it.
//!
//! The tempo the host reports is an `f32`, as `RenderContext::bpm` is. Every analytic reference
//! for a constant tempo is computed from that same `f32` (97.3 arrives as 97.30000305...), not
//! from the decimal the test author typed.
#![allow(dead_code)]

use octocore::domain::TICKS_PER_QUARTER;
use octocore::rng::Rng;
use octocore::{Diagnostics, Engine, Event, EventBuffer, RenderContext};

// ------------------------------------------------------------------ tempo curves

/// A tempo the host follows, as a function of time. All three have closed-form integrals, so
/// the analytic time of any tick is exact (no numerical integration in the reference).
#[derive(Clone, Copy, Debug)]
pub enum Curve {
    Constant { bpm: f32 },
    /// Linear in time from `from` to `to` BPM over `secs`, then constant at `to`.
    Ramp { from: f64, to: f64, secs: f64 },
    /// `from` until `at_secs`, then `to`.
    Jump { from: f64, to: f64, at_secs: f64 },
}

impl Curve {
    pub fn bpm_at(&self, t: f64) -> f64 {
        match *self {
            Curve::Constant { bpm } => bpm as f64,
            Curve::Ramp { from, to, secs } => {
                if t >= secs {
                    to
                } else {
                    from + (to - from) * t / secs
                }
            }
            Curve::Jump { from, to, at_secs } => {
                if t < at_secs {
                    from
                } else {
                    to
                }
            }
        }
    }

    /// Quarter notes elapsed at time `t` seconds: the integral of `bpm / 60`.
    pub fn quarters_at(&self, t: f64) -> f64 {
        match *self {
            Curve::Constant { bpm } => bpm as f64 * t / 60.0,
            Curve::Ramp { from, to, secs } => {
                let ramp = |u: f64| (from * u + (to - from) * u * u / (2.0 * secs)) / 60.0;
                if t <= secs {
                    ramp(t)
                } else {
                    ramp(secs) + to * (t - secs) / 60.0
                }
            }
            Curve::Jump { from, to, at_secs } => {
                if t <= at_secs {
                    from * t / 60.0
                } else {
                    from * at_secs / 60.0 + to * (t - at_secs) / 60.0
                }
            }
        }
    }

    /// The time in seconds at which `q` quarter notes have elapsed: the inverse of `quarters_at`.
    pub fn time_of_quarters(&self, q: f64) -> f64 {
        match *self {
            Curve::Constant { bpm } => q * 60.0 / bpm as f64,
            Curve::Ramp { from, to, secs } => {
                let q_end = self.quarters_at(secs);
                if q <= q_end {
                    let a = (to - from) / secs; // BPM per second
                    if a.abs() < 1e-12 {
                        q * 60.0 / from
                    } else {
                        // from*t + a*t^2/2 = 60 q
                        (-from + (from * from + 2.0 * a * 60.0 * q).sqrt()) / a
                    }
                } else {
                    secs + (q - q_end) * 60.0 / to
                }
            }
            Curve::Jump { from, to, at_secs } => {
                let q_jump = from * at_secs / 60.0;
                if q <= q_jump {
                    q * 60.0 / from
                } else {
                    at_secs + (q - q_jump) * 60.0 / to
                }
            }
        }
    }

    /// The time in seconds of tick number `tick`, counted from the start of the curve.
    pub fn time_of_tick(&self, tick: f64) -> f64 {
        self.time_of_quarters(tick / TICKS_PER_QUARTER as f64)
    }
}

// ------------------------------------------------------------------ buffer sizes

/// How the host chooses the length of each call.
pub enum Sizes {
    Fixed(u32),
    /// Drawn from a seeded generator, from the sizes real hosts use (1 to 4,096).
    Random(Rng),
}

impl Sizes {
    pub const POOL: [u32; 11] = [1, 7, 32, 64, 100, 128, 256, 480, 512, 1024, 4096];

    pub fn random(seed: u64) -> Sizes {
        Sizes::Random(Rng::new(seed))
    }

    fn next(&mut self) -> u32 {
        match self {
            Sizes::Fixed(n) => *n,
            Sizes::Random(rng) => Self::POOL[rng.next_below(Self::POOL.len() as u32) as usize],
        }
    }
}

// ------------------------------------------------------------------ the host

/// One call the host made, as it is written to the trace.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Call {
    pub n: u64,
    pub start_sample: u64,
    pub len: u32,
    pub bpm: f32,
    pub playing: bool,
}

pub struct NullHost {
    pub engine: Engine,
    pub sample_rate: f32,
    /// Samples handed to the engine so far.
    pub clock: u64,
    /// Every event, with its absolute sample.
    pub events: Vec<(u64, Event)>,
    pub calls: Vec<Call>,
    out: EventBuffer,
}

impl NullHost {
    pub fn new(engine: Engine, sample_rate: f32) -> NullHost {
        NullHost { engine, sample_rate, clock: 0, events: Vec::new(), calls: Vec::new(), out: EventBuffer::new() }
    }

    /// One call, recorded.
    pub fn render(&mut self, len: u32, bpm: f32, playing: bool) {
        let ctx = RenderContext { sample_rate: self.sample_rate, buffer_len: len, bpm, playing };
        self.out.clear();
        self.engine.render(&ctx, &mut self.out);
        for e in self.out.as_slice() {
            self.events.push((self.clock + at_sample(e) as u64, *e));
        }
        self.calls.push(Call { n: self.calls.len() as u64, start_sample: self.clock, len, bpm, playing });
        self.clock += len as u64;
    }

    /// Plays for `seconds`, the transport running, reporting the tempo of `curve` at the start
    /// of every buffer (the only thing a host that reports `bpm` once per call can say).
    pub fn play(&mut self, curve: &Curve, seconds: f64, mut sizes: Sizes) {
        let end = (seconds * self.sample_rate as f64) as u64;
        let start = self.clock;
        while self.clock - start < end {
            let t = (self.clock - start) as f64 / self.sample_rate as f64;
            let bpm = curve.bpm_at(t) as f32;
            let len = sizes.next();
            self.render(len, bpm, true);
        }
    }

    pub fn diagnostics(&self) -> Diagnostics {
        self.engine.diagnostics()
    }

    /// Absolute sample of every NoteOn, in the order emitted.
    pub fn note_on_samples(&self) -> Vec<u64> {
        self.events.iter().filter(|(_, e)| matches!(e, Event::NoteOn { .. })).map(|(s, _)| *s).collect()
    }

    // ------------------------------------------------------------ the host clock trace

    /// One NDJSON line per call: `{"n":0,"start":0,"len":64,"bpm":120.0,"playing":true,"pos":null}`.
    pub fn trace_ndjson(&self) -> String {
        let mut s = String::new();
        for c in &self.calls {
            s.push_str(&format!(
                "{{\"n\":{},\"start\":{},\"len\":{},\"bpm\":{:?},\"playing\":{},\"pos\":null}}\n",
                c.n, c.start_sample, c.len, c.bpm, c.playing
            ));
        }
        s
    }

    /// Feeds a trace back to a fresh engine, call for call.
    pub fn replay(engine: Engine, sample_rate: f32, trace: &str) -> NullHost {
        let mut host = NullHost::new(engine, sample_rate);
        for line in trace.lines().filter(|l| !l.trim().is_empty()) {
            let len: u32 = field(line, "len").parse().expect("trace: len");
            let bpm: f32 = field(line, "bpm").parse().expect("trace: bpm");
            let playing: bool = field(line, "playing").parse().expect("trace: playing");
            host.render(len, bpm, playing);
        }
        host
    }

    /// Writes the trace where a failing test can point at it, and returns the path.
    pub fn save_trace(&self, label: &str) -> String {
        let path = std::env::temp_dir().join(format!("o4-trace-{label}.ndjson"));
        let _ = std::fs::write(&path, self.trace_ndjson());
        path.display().to_string()
    }
}

fn field<'a>(line: &'a str, name: &str) -> &'a str {
    let key = format!("\"{name}\":");
    let start = line.find(&key).unwrap_or_else(|| panic!("trace line has no {name}: {line}")) + key.len();
    let rest = &line[start..];
    let end = rest.find([',', '}']).unwrap_or(rest.len());
    &rest[..end]
}

pub fn at_sample(e: &Event) -> u32 {
    match *e {
        Event::NoteOn { at_sample, .. }
        | Event::NoteOff { at_sample, .. }
        | Event::Cc { at_sample, .. }
        | Event::PitchBend { at_sample, .. }
        | Event::ChannelPressure { at_sample, .. } => at_sample,
    }
}

// ------------------------------------------------------------------ measurement

/// How far a list of times is from the ideal, in samples.
#[derive(Debug, Clone, Copy)]
pub struct Deviation {
    pub notes: usize,
    pub max_abs: f64,
    pub mean: f64,
    pub sigma: f64,
}

/// `actual[k]` against `ideal(k)`, both in samples.
pub fn deviation(actual: &[u64], ideal: impl Fn(usize) -> f64) -> Deviation {
    let (mut max_abs, mut sum, mut sq) = (0.0f64, 0.0f64, 0.0f64);
    for (k, &t) in actual.iter().enumerate() {
        let d = t as f64 - ideal(k);
        max_abs = max_abs.max(d.abs());
        sum += d;
        sq += d * d;
    }
    let n = actual.len().max(1) as f64;
    let mean = sum / n;
    Deviation { notes: actual.len(), max_abs, mean, sigma: (sq / n - mean * mean).max(0.0).sqrt() }
}
