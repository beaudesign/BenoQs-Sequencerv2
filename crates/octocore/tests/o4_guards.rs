//! Guard tests for O4 (SPEC-0001, WENGE-0004, step 1 of the release plan), and tests of the
//! null host they stand on.
//!
//! **A guard passes on the parent commit and must keep passing.** These were written and run
//! against the engine as it is on `metronome/command-ring` (`c1e04c1`, the last engine change),
//! before a line of O4 code exists; that is what makes them evidence. A guard is then shown to
//! protect something by breaking the clock in a scratch copy and watching it fail
//! (`handoffs/evidence/o4-guard-mutants.txt`).
//!
//! Where each guard of section 4 of `specs/SPEC-0001/o4-release-plan.md` lives:
//!
//! | ID | Where |
//! |---|---|
//! | G1 | here: `g1_*` |
//! | G2 | `crates/octorun/tests/golden.rs`: `hello`, `chords_and_strums`, `effector` (byte-identical in every O4 PR), `phrases` and `mcc_and_transport` (expected to change in 4b only, section 4 of the plan says how) |
//! | G3 | `crates/octocore/tests/invariants.rs`: `event_times_do_not_depend_on_the_hosts_buffer_size`, `event_times_survive_a_host_that_changes_its_buffer_size_on_every_call`, `a_host_that_renders_one_sample_at_a_time_gets_the_same_event_times`, `event_order_does_not_depend_on_the_hosts_buffer_size` |
//! | G4 | `crates/octoffi/src/lib.rs`: `e4_render_push_and_claim_allocate_nothing` (with a link) and `g4_render_without_a_link_allocates_nothing` (without one) |
//! | G5 | `crates/octocore/tests/conformance.rs`: `all_conformance_fixtures_pass`, and the `conformance` gate |
//!
//! G6 to G9 belong to PR 4b and are written there.

mod common;

use common::*;
use octocore::domain::{DEFAULT_STEP_TICKS, TICKS_PER_QUARTER};
use octocore::types::StepAttr;
use octocore::Engine;

/// The first note of a pattern sounds this many ticks after Play, not on the first tick: a
/// track's step phase starts at 0 and a step of 12 ticks starts on the 12th tick, which is tick
/// index 11 (`tests/conformance/AMBIGUITIES.md`, "the first note after Play"). The guard pins it
/// so that O4 cannot move it without being noticed.
const FIRST_NOTE_TICK: u64 = DEFAULT_STEP_TICKS as u64 - 1;

/// One track, every step on: a note every `DEFAULT_STEP_TICKS`.
fn one_track_every_step() -> Engine {
    let mut e = Engine::new(1);
    for s in 0..16u8 {
        e.grid.set_step_attr(0, s, StepAttr::Active, 1);
    }
    e
}

/// Where note `k` should sound, in samples from Play, for a constant tempo. The tempo is the
/// `f32` the host reports.
fn ideal_sample(bpm: f32, sample_rate: f32, k: usize) -> f64 {
    let spt = sample_rate as f64 * 60.0 / (bpm as f64 * TICKS_PER_QUARTER as f64);
    (FIRST_NOTE_TICK + DEFAULT_STEP_TICKS as u64 * k as u64) as f64 * spt
}

// ------------------------------------------------------------------------------- G1

/// The four (tempo, rate) pairs of the baseline measurement (row A of the plan).
const PAIRS: [(f32, f32); 4] = [(120.0, 48_000.0), (133.0, 44_100.0), (97.3, 48_000.0), (200.0, 96_000.0)];

/// Checks one finished run against the constant-tempo ideal. `Err` says what failed and where
/// the trace of the run was saved.
fn check_constant_tempo(label: &str, host: &NullHost, bpm: f32, sample_rate: f32) -> Result<Deviation, String> {
    let notes = host.note_on_samples();
    let dev = deviation(&notes, |k| ideal_sample(bpm, sample_rate, k));
    let us = |samples: f64| samples / sample_rate as f64 * 1e6;
    let d = host.diagnostics();
    let mut problems = Vec::new();
    if dev.notes < 50 {
        problems.push(format!("only {} notes: a run with no notes tests nothing", dev.notes));
    }
    // The `docs/03` section 7 target.
    if us(dev.sigma) > 120.0 {
        problems.push(format!("sigma {:.1} us is over the docs/03 target of 120 us", us(dev.sigma)));
    }
    if us(dev.max_abs) > 500.0 {
        problems.push(format!("max {:.1} us is over the docs/03 target of 500 us", us(dev.max_abs)));
    }
    // The tighter fence (plan section 3): today's engine sits at 1.02 samples and sigma 0.29,
    // and a 500 us target is 24 samples, which could hide a real regression.
    if dev.max_abs > 1.05 {
        problems.push(format!("max {:.3} samples is over 1.05", dev.max_abs));
    }
    if dev.sigma > 0.3 {
        problems.push(format!("sigma {:.3} samples is over 0.3", dev.sigma));
    }
    if (d.late_events, d.queue_overflows, d.deferred_events, d.unusable_tempo_renders) != (0, 0, 0, 0) {
        problems.push(format!("the engine refused, deferred or delayed events: {d:?}"));
    }
    // Shown with `--nocapture`: the measured numbers, which are the evidence.
    eprintln!("{label}: {} notes, max {:.3} samples ({:.1} us), sigma {:.3} samples ({:.1} us), mean {:+.3}", dev.notes, dev.max_abs, us(dev.max_abs), dev.sigma, us(dev.sigma), dev.mean);
    if problems.is_empty() {
        Ok(dev)
    } else {
        Err(format!("{label}: {}. Trace: {}", problems.join("; "), host.save_trace(label)))
    }
}

/// G1, fixed buffer sizes. Constant tempo: every note lands within a sample of the analytic
/// time, and the size of the host's buffer does not matter. Passes on `c1e04c1`: worst 0.995
/// samples over the 44 runs in `handoffs/evidence/o4-guards-pass.txt`. Where a tick is a whole
/// number of samples (120 BPM at 48 kHz: 125; 200 BPM at 96 kHz: 150) the deviation is exactly 0;
/// where it is not (133 BPM at 44.1 kHz, 97.3 BPM at 48 kHz) it is the floor of a fraction, which
/// has a spread of 1/sqrt(12) = 0.289 samples for any engine that emits whole samples. That is
/// why the fence is 0.3, and why those two pairs are the ones that carry the guard. This is the
/// first content for the `verify:timing` gate.
#[test]
fn g1_constant_tempo_timing_is_within_a_sample_at_every_buffer_size() {
    let mut failing = Vec::new();
    for (bpm, sr) in PAIRS {
        for buffer in [1u32, 7, 64, 480, 1024, 4096] {
            let seconds = if buffer == 1 { 3.0 } else { 10.0 };
            let mut host = NullHost::new(one_track_every_step(), sr);
            host.play(&Curve::Constant { bpm }, seconds, Sizes::Fixed(buffer));
            if let Err(e) = check_constant_tempo(&format!("g1-{bpm}bpm-{sr}hz-buffer{buffer}"), &host, bpm, sr) {
                failing.push(e);
            }
        }
    }
    assert!(failing.is_empty(), "\n{}", failing.join("\n"));
}

/// G1, random buffer sizes: 20 seeded sequences (5 for each pair), the host changing its buffer
/// size on every call across 1 to 4,096 samples.
#[test]
fn g1_constant_tempo_timing_holds_for_random_buffer_sequences() {
    let mut failing = Vec::new();
    for (i, (bpm, sr)) in PAIRS.into_iter().enumerate() {
        for k in 0..5u64 {
            let seed = 9000 + i as u64 * 10 + k;
            let mut host = NullHost::new(one_track_every_step(), sr);
            host.play(&Curve::Constant { bpm }, 10.0, Sizes::random(seed));
            if let Err(e) = check_constant_tempo(&format!("g1-random-seed{seed}"), &host, bpm, sr) {
                failing.push(e);
            }
        }
    }
    assert!(failing.is_empty(), "\n{}", failing.join("\n"));
}

// ------------------------------------------------------------------- the null host itself

/// NH1. The analytic tempo curves are what they say: the closed-form quarter count is the
/// integral of the tempo (checked by Simpson's rule), and `time_of_quarters` inverts it.
#[test]
fn nh1_the_analytic_curves_integrate_and_invert() {
    let curves = [
        Curve::Constant { bpm: 133.0 },
        Curve::Ramp { from: 60.0, to: 180.0, secs: 16.0 },
        Curve::Ramp { from: 180.0, to: 60.0, secs: 8.0 },
        Curve::Jump { from: 120.0, to: 140.0, at_secs: 5.0 },
    ];
    for c in curves {
        // Simpson's rule over [0, t] for the continuous curves. A jump has a step in the
        // integrand, so it is integrated over its two halves separately.
        let simpson = |a: f64, b: f64| {
            let n = 20_000;
            let h = (b - a) / n as f64;
            let mut s = c.bpm_at(a) + c.bpm_at(b);
            for i in 1..n {
                s += c.bpm_at(a + i as f64 * h) * if i % 2 == 1 { 4.0 } else { 2.0 };
            }
            s * h / 3.0 / 60.0
        };
        for t in [0.5, 4.0, 16.0, 20.0, 40.0] {
            let numeric = match c {
                Curve::Jump { at_secs, .. } if t > at_secs => simpson(0.0, at_secs) + simpson(at_secs, t),
                _ => simpson(0.0, t),
            };
            let closed = c.quarters_at(t);
            assert!((numeric - closed).abs() < 1e-6 * closed.max(1.0), "{c:?} at {t} s: numeric {numeric}, closed form {closed}");
            let back = c.time_of_quarters(closed);
            assert!((back - t).abs() < 1e-9 * t.max(1.0), "{c:?}: time_of_quarters(quarters_at({t})) = {back}");
        }
    }
}

/// NH2. The tempo the host reports is the `f32` the engine sees, and the constant curve's
/// reference uses that `f32`: for a constant tempo the curve agrees with the hand formula the
/// guard uses, to a billionth of a sample.
#[test]
fn nh2_the_constant_curve_uses_the_f32_tempo_the_engine_sees() {
    let c = Curve::Constant { bpm: 97.3 };
    assert_eq!(c.bpm_at(0.0), 97.3f32 as f64);
    assert_ne!(c.bpm_at(0.0), 97.3f64, "97.3 is not exactly representable as an f32, which is the point");
    for (bpm, sr) in PAIRS {
        let curve = Curve::Constant { bpm };
        for k in [0usize, 1, 17, 1000] {
            let tick = (FIRST_NOTE_TICK + DEFAULT_STEP_TICKS as u64 * k as u64) as f64;
            let from_curve = curve.time_of_tick(tick) * sr as f64;
            assert!((from_curve - ideal_sample(bpm, sr, k)).abs() < 1e-6, "{bpm} BPM at {sr} Hz, note {k}: {from_curve} vs {}", ideal_sample(bpm, sr, k));
        }
    }
}

/// NH3. A trace replays exactly: the same calls into a fresh engine give the same events, and
/// the trace has one parseable line per call. This is what makes a failure reproducible.
#[test]
fn nh3_a_trace_replays_to_the_same_events() {
    let mut host = NullHost::new(one_track_every_step(), 48_000.0);
    host.play(&Curve::Ramp { from: 90.0, to: 150.0, secs: 3.0 }, 4.0, Sizes::random(3));
    let trace = host.trace_ndjson();
    assert_eq!(trace.lines().count(), host.calls.len());
    assert!(trace.lines().next().unwrap().starts_with("{\"n\":0,\"start\":0,\"len\":"));
    assert!(trace.contains("\"pos\":null"));
    let again = NullHost::replay(one_track_every_step(), 48_000.0, &trace);
    assert_eq!(again.calls, host.calls);
    assert_eq!(again.events, host.events);
    assert!(host.events.len() > 20, "a replay of nothing proves nothing");
}

/// NH4. The host reports the tempo of the curve at the start of each buffer, as a real host
/// does: on a ramp the tempo it reports changes from call to call, and only from call to call.
#[test]
fn nh4_the_host_reports_the_tempo_at_the_start_of_each_buffer() {
    let mut host = NullHost::new(one_track_every_step(), 48_000.0);
    let curve = Curve::Ramp { from: 60.0, to: 180.0, secs: 16.0 };
    host.play(&curve, 2.0, Sizes::Fixed(4096));
    for call in &host.calls {
        let t = call.start_sample as f64 / 48_000.0;
        assert_eq!(call.bpm, curve.bpm_at(t) as f32, "call {}", call.n);
        assert!(call.playing);
    }
    assert!(host.calls.windows(2).all(|w| w[1].bpm > w[0].bpm), "the reported tempo rises with every call");
}
