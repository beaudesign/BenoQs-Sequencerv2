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
            ["track", t, "step", s, "event", attr, amt, range] => {
                let ti: usize = t.parse().map_err(|_| format!("{}: bad track index", ctx()))?;
                let si: usize = s.parse().map_err(|_| format!("{}: bad step index", ctx()))?;
                let map_attr = match *attr {
                    "vel" => MapAttr::Vel,
                    "pit" => MapAttr::Pit,
                    "len" => MapAttr::Len,
                    "sta" => MapAttr::Sta,
                    "amt" => MapAttr::Amt,
                    "grv" => MapAttr::Grv,
                    "mcc" => MapAttr::Mcc,
                    other => return Err(format!("{}: unknown map-factor attr `{}`", ctx(), other)),
                };
                let amt: i8 = amt.trim_start_matches('+').parse().map_err(|_| format!("{}: bad amt", ctx()))?;
                let range: u8 = range.parse().map_err(|_| format!("{}: bad range", ctx()))?;
                engine.grid.active_page_mut().tracks[ti].steps[si].event = Some(StepEvent {
                    target_track: ti as TrackIndex,
                    kind: StepEventKind::ScaleMap { attr: map_attr, amt, range },
                });
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
                    "amt" => step.amount = v as i8,
                    "phrase" => step.phrase = if v == 0 { None } else { Some(v as u8) },
                    "skip" => step.skip = v != 0,
                    "active" => step.active = v != 0,
                    other => return Err(format!("{}: unknown step attribute `{}`", ctx(), other)),
                }
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
                let mut want_vel_offset: Option<i32> = None;
                for kv in rest {
                    let (k, v) = kv.split_once('=').ok_or_else(|| format!("{}: bad expect clause `{}`", ctx(), kv))?;
                    match k {
                        "track" => want_track = Some(v.parse().map_err(|_| format!("{}: bad track", ctx()))?),
                        "pit" => {
                            want_pit_offset =
                                Some(v.trim_start_matches('+').parse().map_err(|_| format!("{}: bad pit", ctx()))?)
                        }
                        "vel" => {
                            want_vel_offset =
                                Some(v.trim_start_matches('+').parse().map_err(|_| format!("{}: bad vel", ctx()))?)
                        }
                        other => return Err(format!("{}: unknown expect key `{}`", ctx(), other)),
                    }
                }
                let want_track = want_track.ok_or_else(|| format!("{}: expect needs track=", ctx()))?;
                let base_pitch = engine.grid.active_page().tracks[want_track].pitch as i32;
                let base_vel = engine.grid.active_page().tracks[want_track].velocity as i32;
                let expected_pitch = want_pit_offset.map(|off| base_pitch + off);
                let expected_vel = want_vel_offset.map(|off| base_vel + off);

                let matched = fired.iter().any(|f| {
                    f.track as usize == want_track
                        && expected_pitch.map(|p| p == f.pitch as i32).unwrap_or(true)
                        && expected_vel.map(|v| v == f.velocity as i32).unwrap_or(true)
                });
                if !matched {
                    return Err(format!(
                        "{}: no fired note matched track={} pit_offset={:?} vel_offset={:?} (base_pitch={} base_vel={}); fired={:?}",
                        ctx(),
                        want_track,
                        want_pit_offset,
                        want_vel_offset,
                        base_pitch,
                        base_vel,
                        fired.iter().map(|f| (f.track, f.pitch, f.velocity)).collect::<Vec<_>>()
                    ));
                }
            }
            other => return Err(format!("{}: unrecognised directive: {:?}", ctx(), other)),
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
