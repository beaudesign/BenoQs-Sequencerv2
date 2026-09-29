//! Baseline measurements of the CURRENT clock (nothing here is O4 code): how far do note times
//! land from the analytic answer for (A) a constant tempo, (B) a 60 to 180 BPM ramp, and how
//! long does the f32 step accumulator stay exact (D)?
use octocore::types::{StepAttr};
use octocore::{Engine, Event, EventBuffer, RenderContext};

const SR: f64 = 48_000.0;

fn note_on_samples(bpm_at: &dyn Fn(f64) -> f64, buffer: u32, seconds: f64, sr: f64) -> Vec<f64> {
    let mut e = Engine::new(1);
    // one track, every step on: a note every 12 ticks
    for s in 0..16u8 { e.grid.set_step_attr(0, s, StepAttr::Active, 1); }
    let mut out = EventBuffer::new();
    let mut clock = 0u64;
    let mut v = Vec::new();
    let total = (seconds * sr) as u64;
    while clock < total {
        let bpm = bpm_at(clock as f64 / sr);
        let ctx = RenderContext { sample_rate: sr as f32, buffer_len: buffer, bpm: bpm as f32, playing: true };
        out.clear();
        e.render(&ctx, &mut out);
        for ev in out.as_slice() {
            if let Event::NoteOn { at_sample, .. } = ev { v.push((clock + *at_sample as u64) as f64); }
        }
        clock += buffer as u64;
    }
    v
}

fn main() {
    // (A) constant tempo: ideal note k at tick 11 + 12k, tick i at i * spt
    println!("(A) constant tempo, deviation of note onsets from the ideal grid, in samples (48 kHz: 1 sample = 20.8 us)");
    for (bpm, sr) in [(120.0, 48_000.0), (133.0, 44_100.0), (97.3, 48_000.0), (200.0, 96_000.0)] {
        for buf in [64u32, 480, 1024, 4096] {
            let v = note_on_samples(&|_| bpm, buf, 20.0, sr);
            let spt = sr * 60.0 / (bpm * 192.0);
            let mut worst = 0.0f64; let mut sum = 0.0; let mut sq = 0.0;
            for (k, &t) in v.iter().enumerate() {
                let ideal = (11 + 12 * k) as f64 * spt;
                let d = t - ideal;
                worst = worst.max(d.abs()); sum += d; sq += d * d;
            }
            let n = v.len() as f64; let mean = sum / n; let sd = (sq / n - mean * mean).sqrt();
            println!("  {bpm:>6} BPM {sr:>7} Hz buffer {buf:>4}: {} notes, max |dev| {worst:.3} samples ({:.1} us), sigma {sd:.3} samples ({:.1} us)", v.len(), worst / sr * 1e6, sd / sr * 1e6);
        }
    }

    // (B) tempo ramp 60 -> 180 BPM linear in time over T = 16 s (= 8 bars of 4/4 = 32 beats).
    // bpm(t) = 60 + 7.5 t; beats(t) = t + 0.0625 t^2. Host reports bpm at the start of each buffer.
    println!("(B) 60 to 180 BPM ramp over 8 bars (16 s), host reports the tempo at the start of each buffer");
    let ticks_at = |t: f64| (t + 0.0625 * t * t) * 192.0; // ticks elapsed at time t
    let time_of_tick = |i: f64| { // solve t + 0.0625 t^2 = i/192
        let b = i / 192.0; (-1.0 + (1.0 + 4.0 * 0.0625 * b).sqrt()) / (2.0 * 0.0625)
    };
    let _ = ticks_at;
    for buf in [32u32, 64, 128, 256, 512, 1024, 2048] {
        let v = note_on_samples(&|t| 60.0 + 7.5 * t, buf, 16.0, SR);
        let mut worst = 0.0f64; let mut last = 0.0;
        for (k, &t) in v.iter().enumerate() {
            let ideal = time_of_tick((11 + 12 * k) as f64) * SR;
            let d = t - ideal;
            worst = worst.max(d.abs()); last = d;
        }
        println!("  buffer {buf:>4}: {} notes, max |dev| {worst:.1} samples ({:.2} ms), last note {last:.1} samples ({:.2} ms) {}", v.len(), worst / SR * 1e3, last / SR * 1e3, if last > 0.0 {"late"} else {"early"});
    }

    // (D) the f32 step accumulator: replicate `accum += 1.0; if accum + 1e-6 < step_ticks {return}; accum -= step_ticks`
    // for every multiplier num/den that can occur and compare with exact rational arithmetic.
    println!("(D) f32 step accumulator vs exact arithmetic, first divergence in ticks (10^8 ticks = 72 h at 120 BPM)");
    let horizon: u64 = 100_000_000;
    let mut diverging = 0; let mut total = 0;
    for den in 1..=16u32 { for num in 1..=16u32 {
        if gcd(num, den) != 1 { continue; }
        total += 1;
        let mult = num as f32 / den as f32;
        let step_ticks = (12.0f32 / mult.max(1e-6)).max(1.0);
        // exact: step_ticks = 12*den/num (or 1 if that is < 1): fires when the count of ticks since last fire reaches it, carrying the fraction
        let exact_num = if 12 * den < num { 1 } else { 12 * den }; // numerator over `exact_den`
        let exact_den = if 12 * den < num { 1 } else { num };
        let mut accum = 0.0f32; let mut fired: u64 = 0; let mut first_div: Option<u64> = None;
        for tick in 1..=horizon {
            accum += 1.0;
            if accum + 1e-6 >= step_ticks { accum -= step_ticks; fired += 1; }
            // exact count of firings by `tick`: number of k >= 1 with k * exact_num/exact_den <= tick (fires when accumulated ticks reach it)
            let want = (tick * exact_den as u64) / exact_num as u64;
            if fired != want && first_div.is_none() { first_div = Some(tick); break; }
        }
        if let Some(t) = first_div { diverging += 1; println!("  x{num}/{den}: step every {:.6} ticks, first differs at tick {t} ({:.1} s at 120 BPM)", 12.0 * den as f64 / num as f64, t as f64 / 384.0); }
    }}
    println!("  {diverging} of {total} multipliers diverge from exact arithmetic within {horizon} ticks");
}
fn gcd(a: u32, b: u32) -> u32 { if b == 0 { a } else { gcd(b, a % b) } }
