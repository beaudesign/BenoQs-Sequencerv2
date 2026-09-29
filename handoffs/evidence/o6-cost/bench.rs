//! What applying commands costs per `Engine::render`, by command type (SPEC-0001 O6). Not part of the workspace:
//! a scratch cargo project with `octocore = { path = "<repo>/crates/octocore" }` and an empty `[workspace]`, run with
//! `cargo run --release`. Timings are wall-clock on a shared machine, so they are reported, never asserted.
use octocore::types::{Command, StepAttr, TrackAttr};
use octocore::{Engine, EventBuffer, RenderContext};
use std::time::Instant;

fn scene(e: &mut Engine) {
    for t in 0..10u8 { for s in 0..16u8 { e.grid.set_step_attr(t, s, StepAttr::Active, (s % 2 == 0) as i32); } }
}

fn make(kind: &str, k: usize, j: usize) -> Command {
    match kind {
        "SetTrack" => Command::SetTrack { track: (j % 10) as u8, attr: TrackAttr::Pitch, value: 40 + ((k + j) % 40) as i32 },
        "SetStep" => Command::SetStep { track: (j % 10) as u8, step: (j % 16) as u8, attr: StepAttr::Active, value: ((k + j) % 2) as i32 },
        "Stop" => Command::Stop,
        "Reset" => Command::Reset,
        "SetActivePage" => Command::SetActivePage { bank: (j % 9) as u8, page: (j % 4) as u8 },
        _ => unreachable!(),
    }
}

fn run(label: &str, kind: &str, link: bool, cmds_per_render: usize, n: usize) {
    let mut e = Engine::new(1);
    scene(&mut e);
    let mut ends = if link { e.open_link() } else { None };
    let ctx = RenderContext { sample_rate: 48_000.0, buffer_len: 256, bpm: 120.0, playing: true };
    let mut out = EventBuffer::new();
    let mut times = Vec::with_capacity(n);
    for k in 0..n {
        if let Some((tx, _)) = ends.as_mut() {
            for j in 0..cmds_per_render { tx.push(make(kind, k, j)); }
        }
        out.clear();
        let t = Instant::now();
        e.render(&ctx, &mut out);
        times.push(t.elapsed().as_nanos() as u64);
    }
    times.sort_unstable();
    let p = |q: f64| times[((n as f64 - 1.0) * q) as usize];
    println!("{label:<52} median {:>6} ns   p99 {:>6} ns   max {:>7} ns", p(0.5), p(0.99), times[n - 1]);
}

fn main() {
    let n = 50_000;
    run("no link", "SetTrack", false, 0, n);
    run("link, no commands (publish only)", "SetTrack", true, 0, n);
    for kind in ["SetTrack", "SetStep", "SetActivePage", "Stop", "Reset"] {
        run(&format!("link, 1 {kind} per render"), kind, true, 1, n);
        run(&format!("link, 256 {kind} per render (the limit)"), kind, true, 256, n);
    }
    run("no link (again)", "SetTrack", false, 0, n);
}
