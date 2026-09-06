//! Runner for the conformance fixture DSL described in
//! `docs/03-sequencer-core.md` §8. Fixtures live in `tests/conformance/**/*.fixture`
//! at the repo root (not under this crate) so they stay a plain, hand-writable text
//! format shared by whatever eventually consumes the manual's worked examples.
//!
//! This is test-only infrastructure (parsing happens off the audio thread), so
//! unlike `engine`/`domain` it freely uses `std` — `Vec`/`String` are fine here.

use crate::domain::*;
use crate::engine::{Engine, NoteFire};

pub fn run_fixture(source: &str) -> Result<(), String> {
    let mut engine = Engine::new(0);
    let mut fired: Vec<NoteFire> = Vec::new();

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
                engine.grid.active_page_mut().tracks[ti].role = match *role {
                    "feeder" => TrackRole::Feeder,
                    "listener" => TrackRole::Listener,
                    "none" => TrackRole::None,
                    other => return Err(format!("{}: unknown role `{}`", ctx(), other)),
                };
            }
            ["track", t, "step", s, attr, value] => {
                let ti: usize = t.parse().map_err(|_| format!("{}: bad track index", ctx()))?;
                let si: usize = s.parse().map_err(|_| format!("{}: bad step index", ctx()))?;
                let step = &mut engine.grid.active_page_mut().tracks[ti].steps[si];
                let v: i32 = value.trim_start_matches('+').parse().map_err(|_| format!("{}: bad value", ctx()))?;
                match *attr {
                    "pit" => step.pitch_offset = v as i8,
                    "vel" => step.velocity_offset = v as i8,
                    "sta" => step.start_offset = v as i8,
                    "strum" => step.strum = v as i8,
                    other => return Err(format!("{}: unknown step attribute `{}`", ctx(), other)),
                }
            }
            ["play", n, step_word] if step_word.starts_with("step") => {
                // A "step" is DEFAULT_STEP_TICKS (12) PPQN ticks (a 1/16 note at
                // the default 1x multiplier) — one call to `step_once_for_test`
                // is one *tick*, not one step; see docs/03-sequencer-core.md §2.
                let steps: u32 = n.parse().map_err(|_| format!("{}: bad step count", ctx()))?;
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

    Ok(())
}
