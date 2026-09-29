//! More baseline measurements of the CURRENT clock (nothing here is O4 code).
//! (C) does the lateness left by a tempo ramp persist once the tempo is constant again?
//! (S) does shifting the tempo the host reports earlier by the lookahead remove the lateness?
//! (F) when does the f32 step accumulator fire a step, against the exact tick?
use octocore::types::StepAttr;
use octocore::{Engine, Event, EventBuffer, RenderContext};

const SR: f64 = 48_000.0;

fn notes(bpm_at: &dyn Fn(f64) -> f64, buffer: u32, seconds: f64) -> Vec<f64> {
    let mut e = Engine::new(1);
    for s in 0..16u8 { e.grid.set_step_attr(0, s, StepAttr::Active, 1); }
    let mut out = EventBuffer::new();
    let mut clock = 0u64; let mut v = Vec::new();
    let total = (seconds * SR) as u64;
    while clock < total {
        let ctx = RenderContext { sample_rate: SR as f32, buffer_len: buffer, bpm: bpm_at(clock as f64 / SR) as f32, playing: true };
        out.clear(); e.render(&ctx, &mut out);
        for ev in out.as_slice() { if let Event::NoteOn { at_sample, .. } = ev { v.push((clock + *at_sample as u64) as f64); } }
        clock += buffer as u64;
    }
    v
}

// ideal onset time (in samples) of tick i for a tempo curve given as ticks-per-second integral, by bisection
fn ideal_time(beats_at: &dyn Fn(f64) -> f64, tick: f64) -> f64 {
    let target = tick / 192.0; let (mut lo, mut hi) = (0.0f64, 200.0f64);
    for _ in 0..80 { let mid = 0.5 * (lo + hi); if beats_at(mid) < target { lo = mid } else { hi = mid } }
    0.5 * (lo + hi) * SR
}

fn main() {
    let which = std::env::args().nth(1).unwrap_or_default(); // C, S or F; F takes the horizon in ticks as a second argument
    if which == "C" || which.is_empty() {
        println!("(C) 60 to 120 BPM ramp over 4 s, then 120 BPM constant to 16 s. Lateness in ms of every 20th note (buffer 256):");
        let bpm = |t: f64| if t < 4.0 { 60.0 + 15.0 * t } else { 120.0 };
        // beats: t + 15 t^2/120 up to 4 s (= 4 + 2 = 6 beats), then 6 + 2 (t-4)
        let beats = |t: f64| if t < 4.0 { t + 0.125 * t * t } else { 6.0 + 2.0 * (t - 4.0) };
        let v = notes(&bpm, 256, 16.0);
        for (k, &t) in v.iter().enumerate() {
            if k % 20 == 0 {
                let ideal = ideal_time(&beats, (11 + 12 * k) as f64);
                println!("   note {k:>3} ideal {:6.2} s  late {:6.2} ms", ideal / SR, (t - ideal) / SR * 1e3);
            }
        }
    }
    if which == "S" || which.is_empty() {
        println!("(S) 60 to 180 BPM ramp, buffer 256, the host reports the tempo of `shift` ms LATER than the buffer start:");
        let beats = |t: f64| t + 0.0625 * t * t;
        for shift_ms in [0.0, 10.0, 20.0, 25.0, 30.0, 40.0] {
            let sh = shift_ms / 1e3;
            let v = notes(&|t| 60.0 + 7.5 * (t + sh), 256, 16.0);
            let mut last = 0.0;
            for (k, &t) in v.iter().enumerate() { last = t - ideal_time(&beats, (11 + 12 * k) as f64); }
            println!("   shift {shift_ms:>4} ms: last note {:.2} ms late", last / SR * 1e3);
        }
    }
    if which == "F" {
        let horizon: u64 = std::env::args().nth(2).and_then(|a| a.parse().ok()).unwrap_or(100_000_000);
        println!("(F) f32 accumulator: when does the k-th step fire, against the exact tick ceil(k * step), over {horizon} ticks");
        let mut worst_all: i64 = 0; let mut nonzero = 0; let mut total = 0;
        let mut rows: Vec<(i64, i64, i64, u32, u32, u64)> = Vec::new();
        for den in 1..=16u32 { for num in 1..=16u32 {
            if gcd(num, den) != 1 { continue; }
            total += 1;
            let mult = num as f32 / den as f32;
            let step_ticks = (12.0f32 / mult.max(1e-6)).max(1.0);
            let (en, ed): (u64, u64) = if 12 * den < num { (1, 1) } else { (12 * den as u64, num as u64) };
            let mut acc = 0.0f32; let mut k: u64 = 0;
            let (mut lo, mut hi) = (0i64, 0i64); let mut errs_late = 0u64;
            let (mut lo_first, mut hi_first) = (0i64, 0i64); // first tenth of the horizon
            for tick in 1..=horizon {
                acc += 1.0;
                if acc + 1e-6 >= step_ticks {
                    acc -= step_ticks; k += 1;
                    let exact = (k * en + ed - 1) / ed;
                    let err = tick as i64 - exact as i64;
                    if err < lo { lo = err } if err > hi { hi = err }
                    if tick <= horizon / 10 { if err < lo_first { lo_first = err } if err > hi_first { hi_first = err } }
                    if err != 0 { errs_late += 1; }
                }
            }
            let w = lo.abs().max(hi.abs());
            if w > worst_all { worst_all = w }
            if w != 0 { nonzero += 1; }
            rows.push((w, lo, hi, num, den, errs_late));
            let _ = (lo_first, hi_first);
        }}
        rows.sort_by(|a, b| b.0.cmp(&a.0).then(b.5.cmp(&a.5)));
        println!("   {nonzero} of {total} multipliers ever fire a step on a different tick from exact arithmetic; the worst error is {worst_all} tick(s)");
        for r in rows.iter().take(8) { println!("   x{}/{}: error range {}..{} ticks, {} steps on a wrong tick out of the run", r.3, r.4, r.1, r.2, r.5); }
        let hist = |n: i64| rows.iter().filter(|r| r.0 == n).count();
        println!("   multipliers whose worst error is 1 tick: {}, 2 ticks: {}, 3 or more: {}", hist(1), hist(2), rows.iter().filter(|r| r.0 >= 3).count());
    }
}
fn gcd(a: u32, b: u32) -> u32 { if b == 0 { a } else { gcd(b, a % b) } }
