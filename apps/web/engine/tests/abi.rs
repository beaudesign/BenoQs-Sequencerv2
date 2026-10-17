//! The module's exported surface, tested the way the page uses it: through the exported functions
//! and the bytes they hand back (ADR-0008, `specs/SPEC-0002/p3-plan.md` section 4, P3b slice 1).
//! Each test runs on its own thread, and the module's state is per thread, so they do not meet.

mod common;

use common::*;
use octocore::{Command, Engine, EventBuffer, RenderContext};
use octoweb::exports::*;

const GREEN_STEADY: u8 = 2; // colour in the low two bits (off 0, red 1, green 2, orange 3), phase above (steady 0, flash 1, shine 2)
const RED_STEADY: u8 = 1;
const MIDI_CHANNEL: u32 = 8; // where `TrackAttr::MidiChannel` stands in `TrackAttr::ALL`, which is the number `octoweb_set_track` takes (ABI.md)
const ORANGE_FLASH: u8 = 3 | (1 << 2);
const GREEN_FLASH: u8 = 2 | (1 << 2);
const TRACK_PITCH: u32 = 0; // `TrackAttr::ALL` order, as for `MIDI_CHANNEL`
const STEP_ACTIVE: u32 = 0; // `StepAttr::ALL` order: the number `octoweb_set_step` takes (ABI.md)
const STEP_PITCH_OFFSET: u32 = 2;
const STEP_SKIP: u32 = 1;
const STEP_STRUM: u32 = 8; // `StepAttr::ALL` order, as above

#[test]
fn the_module_reports_abi_version_two() {
    assert_eq!(octoweb_abi(), 2, "P4c added event kind 5, octoweb_set_clock and octoweb_tick_position (ADR-0009)");
}

#[test]
fn init_builds_the_layout_from_the_controls_file() {
    let r = try_init(&layout_text(&[]), 48_000.0, 0);
    assert_eq!(r.code, 0, "{}", r.message);
}

#[test]
fn init_refuses_a_layout_that_lacks_a_control_the_controller_needs_and_names_it() {
    let r = try_init(&layout_text(&["keys.esc"]), 48_000.0, 0);
    assert_eq!(r.code, 4, "a layout the controller rejects is code 4 (ABI.md)");
    assert!(r.message.contains("keys.esc"), "the message should name the control: {:?}", r.message);
}

#[test]
fn init_refuses_text_that_is_not_a_layout() {
    let good = layout_text(&[]);
    // Text that is not `<n> <id>` per line is code 3, even when everything else in it is a good layout.
    for bad in [format!("{good}x extra.control\n"), format!("{good}5\n"), "this is not a layout".to_string(), "x matrix.r0.c1\n".to_string()] {
        let r = try_init(&bad, 48_000.0, 0);
        assert_eq!(r.code, 3, "{bad:?} should be refused as malformed text");
        assert!(r.message.contains("layout line"), "a refusal says which line: {:?}", r.message);
    }
    // A well-formed layout the controller cannot use is code 4.
    for bad in ["".to_string(), "999999 matrix.r0.c1\n".to_string(), format!("{good}0 extra.control\n")] {
        let r = try_init(&bad, 48_000.0, 0);
        assert_eq!(r.code, 4, "{bad:?} should be refused by the controller");
        assert!(!r.message.is_empty());
    }
}

#[test]
fn init_refuses_a_sample_rate_the_engine_cannot_use() {
    for sr in [0.0f32, -48_000.0, f32::NAN, f32::INFINITY, 100.0] {
        assert_ne!(try_init(&layout_text(&[]), sr, 0).code, 0, "{sr} should be refused");
    }
}

#[test]
fn calls_before_init_do_nothing_and_say_so() {
    octoweb_reset(); // the page can also drop the module's state; a test thread may have run another test
    assert_eq!(octoweb_input(0.0, 0, 5, 0), 1, "input before init is `not initialised`");
    assert_eq!(octoweb_transport(1), 1);
    assert_eq!(octoweb_set_tempo(120.0), 1);
    assert_eq!(octoweb_set_track(0, MIDI_CHANNEL, 17), 1);
    assert_eq!(octoweb_set_step(0, 0, STEP_ACTIVE, 1), 1);
    assert_eq!(octoweb_set_chord_style(0, 0, 0), 1);
    assert_eq!(octoweb_render(128), 0, "no events");
    assert_eq!(octoweb_refresh_leds(), 0);
    assert_eq!(octoweb_status(), 0);
    assert!(octoweb_events().is_null());
    assert!(octoweb_leds().is_null());
    assert!(octoweb_playheads().is_null());
}

#[test]
fn a_failed_init_leaves_the_module_uninitialised() {
    octoweb_reset();
    let r = try_init("nonsense", 48_000.0, 0);
    assert_ne!(r.code, 0);
    assert_eq!(octoweb_input(0.0, 0, 5, 0), 1);
}

#[test]
fn a_matrix_press_lights_the_step_green_and_a_second_press_puts_it_out() {
    let abi = Abi::start();
    let key = abi.matrix(2, 4);
    assert_eq!(abi.led(&key), 0, "an empty step is dark");
    abi.click(&key, 0.0);
    assert_eq!(abi.led(&key), GREEN_STEADY);
    abi.click(&key, 500.0);
    assert_eq!(abi.led(&key), 0);
}

#[test]
fn a_press_changes_only_the_leds_it_should() {
    let abi = Abi::start();
    let before = abi.leds();
    abi.click(&abi.matrix(2, 4), 0.0);
    let after = abi.leds();
    let changed: Vec<usize> = (0..512).filter(|i| before[*i] != after[*i]).collect();
    assert_eq!(changed, vec![abi.control("matrix.r2.c5") as usize]);
}

#[test]
fn refresh_says_whether_anything_changed() {
    let abi = Abi::start();
    octoweb_refresh_leds();
    assert_eq!(octoweb_refresh_leds(), 0, "nothing changed since the last refresh");
    abi.click(&abi.matrix(0, 0), 0.0);
    assert_eq!(octoweb_refresh_leds(), 1);
    assert_eq!(octoweb_refresh_leds(), 0);
}

#[test]
fn an_led_byte_is_the_colour_in_the_low_two_bits_and_the_phase_above() {
    let abi = Abi::start();
    // PLAY idles steady green and EDIT starts steady green (p066, p068).
    assert_eq!(abi.led("mode.play"), GREEN_STEADY);
    assert_eq!(abi.led("mode.edit"), GREEN_STEADY);
    abi.click("mode.edit", 0.0); // Normal to Preview: flashing orange (p068)
    assert_eq!(abi.led("mode.edit"), ORANGE_FLASH);
    abi.click("mode.edit", 200.0); // Perform: flashing green (p069)
    assert_eq!(abi.led("mode.edit"), GREEN_FLASH);
    // A skipped step is steady red (p014).
    abi.click("mode.edit", 400.0);
    abi.click(&abi.matrix(1, 1), 600.0);
    // Zoom and skip the step through the panel: hold Step Mode, press the key, then Main Mute.
    let step_mode = abi.control("mode.step");
    assert_eq!(octoweb_input(800.0, 0, step_mode, 0), 0);
    abi.click(&abi.matrix(1, 1), 850.0);
    assert_eq!(octoweb_input(900.0, 1, step_mode, 0), 0);
    abi.click("mutator.mute", 950.0);
    abi.click("keys.esc", 1000.0);
    assert_eq!(abi.led(&abi.matrix(1, 1)), RED_STEADY);
}

#[test]
fn the_status_word_says_playing_and_zoomed() {
    let abi = Abi::start();
    assert_eq!(octoweb_status(), 0);
    assert_eq!(octoweb_transport(1), 0);
    assert_eq!(octoweb_status() & 1, 1, "bit 0 is the transport");
    assert_eq!(octoweb_transport(0), 0);
    assert_eq!(octoweb_status() & 1, 0);
    let step_mode = abi.control("mode.step");
    octoweb_input(0.0, 0, step_mode, 0);
    abi.click(&abi.matrix(0, 0), 10.0);
    octoweb_input(20.0, 1, step_mode, 0);
    assert_eq!(octoweb_status() & 2, 2, "bit 1 is Step zoom");
    abi.click("keys.esc", 100.0);
    assert_eq!(octoweb_status() & 2, 0);
}

#[test]
fn reset_drops_the_module_and_it_can_start_again() {
    let abi = Abi::start();
    abi.click(&abi.matrix(0, 0), 0.0);
    assert_eq!(octoweb_reset(), 0);
    assert_eq!(octoweb_input(0.0, 0, 5, 0), 1);
    let abi = Abi::start();
    assert_eq!(abi.led(&abi.matrix(0, 0)), 0, "a new start is a new engine");
}

#[test]
fn the_ids_the_page_cannot_press_do_nothing() {
    let abi = Abi::start();
    let before = abi.leds();
    assert_eq!(octoweb_input(0.0, 0, 511, 0), 0, "a control the controller ignores is not an error");
    assert_eq!(octoweb_input(0.0, 0, u32::MAX, 0), 0);
    assert_eq!(octoweb_input(0.0, 3, 5, 0), 2, "an unknown kind is refused");
    assert_eq!(abi.leds(), before);
}

#[test]
fn an_intent_the_engine_cannot_act_on_is_counted_and_changes_nothing() {
    let abi = Abi::start();
    assert_eq!(octoweb_dropped_intents(), 0);
    abi.click("mode.edit", 0.0); // EDIT preview: a matrix key asks to audition a step (p068)
    let before = abi.leds();
    abi.click(&abi.matrix(3, 3), 100.0);
    assert_eq!(octoweb_dropped_intents(), 1, "the Audition intent has no engine capability behind it yet");
    assert_eq!(abi.leds(), before, "and nothing was set");
}

/// What the engine does when driven directly, with the same program, sample rate, tempo and blocks.
fn direct_blocks(seed: u64, program: impl Fn(&mut Engine), blocks: usize, bpm: f32, sample_rate: f32) -> Vec<Vec<[u8; EVENT_BYTES]>> {
    let mut engine = Engine::new(seed);
    program(&mut engine);
    engine.handle_command(Command::Play);
    let mut buf = EventBuffer::new();
    (0..blocks)
        .map(|_| {
            buf.clear();
            engine.render(&RenderContext { sample_rate, buffer_len: 128, bpm, playing: engine.is_running() }, &mut buf);
            buf.as_slice().iter().map(wire).collect()
        })
        .collect()
}

#[test]
fn a_pattern_played_through_the_module_is_the_engines_own_output_byte_for_byte() {
    let seed = 7;
    let abi = Abi::start_with(48_000.0, seed);
    // Program by pressing: a four-on-the-floor line on track 0 and an off-beat on track 3.
    let steps = [(0usize, 0usize), (0, 4), (0, 8), (0, 12), (3, 2), (3, 10)];
    for (i, (row, step)) in steps.iter().enumerate() {
        abi.click(&abi.matrix(*row, *step), i as f64 * 100.0);
    }
    assert_eq!(octoweb_set_tempo(133.0), 0);
    assert_eq!(octoweb_transport(1), 0);

    let blocks = 4 * 48_000 / 128; // four seconds
    let want = direct_blocks(
        seed,
        |e| {
            for (row, step) in steps {
                e.grid.active_page_mut().tracks[row].steps[step].active = true;
            }
        },
        blocks,
        133.0,
        48_000.0,
    );
    let mut total = 0;
    for (i, expected) in want.iter().enumerate() {
        let got = abi.render(128);
        assert_eq!(&got, expected, "block {i}");
        total += got.len();
    }
    assert!(total >= 8, "the test should hear something ({total} events)");
}

#[test]
fn the_first_note_on_is_a_real_note_on() {
    let abi = Abi::start();
    abi.click(&abi.matrix(0, 0), 0.0);
    octoweb_transport(1);
    let mut first = None;
    for _ in 0..400 {
        if let Some(e) = abi.render(128).into_iter().find(|e| e[0] == 0) {
            first = Some(e);
            break;
        }
    }
    let e = first.expect("a Note On within about a second");
    assert!((1..=2).contains(&e[1]), "port {}", e[1]);
    assert!((1..=16).contains(&e[2]), "channel {}", e[2]);
    assert!(e[3] < 128, "note {}", e[3]);
    assert!(e[4] >= 1 && e[4] < 128 && e[5] == 0, "velocity {} (at least 1, MIDI)", e[4]);
    assert_eq!(&e[6..8], &[0, 0], "reserved bytes are zero");
}

#[test]
fn stop_leaves_the_receiver_holding_no_note() {
    use std::collections::BTreeMap;
    let abi = Abi::start();
    for s in [0, 2, 4, 6, 8, 10, 12, 14] {
        abi.click(&abi.matrix(0, s), s as f64 * 100.0);
    }
    octoweb_transport(1);
    // The receiver: how many Note Ons each (port, channel, note) has had without a Note Off.
    let mut held: BTreeMap<(u8, u8, u8), i32> = BTreeMap::new();
    let apply = |events: Vec<[u8; EVENT_BYTES]>, held: &mut BTreeMap<(u8, u8, u8), i32>| {
        for e in events {
            match e[0] {
                0 => *held.entry((e[1], e[2], e[3])).or_default() += 1,
                1 => *held.entry((e[1], e[2], e[3])).or_default() -= 1,
                _ => {}
            }
        }
    };
    // Play until a note is sounding, then stop at once.
    let mut sounding = false;
    for _ in 0..1_000 {
        apply(abi.render(128), &mut held);
        if held.values().any(|v| *v > 0) {
            sounding = true;
            break;
        }
    }
    assert!(sounding, "the test should stop with a note on");
    octoweb_transport(0);
    let mut note_ons_after_stop = 0;
    for _ in 0..(48_000 / 128) {
        let events = abi.render(128);
        note_ons_after_stop += events.iter().filter(|e| e[0] == 0).count();
        apply(events, &mut held);
    }
    assert!(held.values().all(|v| *v == 0), "the receiver still holds notes: {held:?}");
    assert_eq!(note_ons_after_stop, 0, "a render after Stop must not start the transport again");
    assert_eq!(octoweb_status() & 1, 0);
}

#[test]
fn the_playhead_moves_while_the_transport_runs() {
    let abi = Abi::start();
    abi.click(&abi.matrix(0, 0), 0.0);
    abi.click(&abi.matrix(0, 8), 100.0);
    octoweb_transport(1);
    let mut seen = std::collections::BTreeSet::new();
    for _ in 0..(2 * 48_000 / 128) {
        abi.render(128);
        seen.insert(abi.playheads()[0]);
    }
    assert!(seen.len() >= 4, "track 0's playhead visited {seen:?}");
    assert!(seen.iter().all(|s| *s < 16));
}

#[cfg(feature = "measure")]
#[test]
fn render_and_the_led_refresh_allocate_nothing() {
    let abi = Abi::start();
    for row in 0..10 {
        for step in 0..16 {
            abi.click(&abi.matrix(row, step), 0.0);
        }
    }
    octoweb_set_tempo(240.0);
    octoweb_transport(1);
    let before = octoweb_alloc_count();
    for _ in 0..(3 * 48_000 / 128) {
        octoweb_render(128);
    }
    for _ in 0..200 {
        octoweb_refresh_leds();
    }
    assert_eq!(octoweb_alloc_count(), before, "render and the LED refresh must not allocate");
}

#[test]
fn the_buffers_the_page_reads_do_not_move() {
    let abi = Abi::start();
    let (e, l, p) = (octoweb_events(), octoweb_leds(), octoweb_playheads());
    abi.click(&abi.matrix(0, 0), 0.0);
    octoweb_transport(1);
    for _ in 0..100 {
        octoweb_render(128);
    }
    octoweb_refresh_leds();
    assert_eq!((octoweb_events(), octoweb_leds(), octoweb_playheads()), (e, l, p));
}

#[test]
fn a_block_larger_than_the_limit_is_cut_down_and_zero_frames_make_nothing() {
    let abi = Abi::start();
    abi.click(&abi.matrix(0, 0), 0.0);
    octoweb_transport(1);
    assert_eq!(octoweb_render(0), 0);
    assert!(octoweb_render(u32::MAX) <= 512, "never more events than the buffer holds: 256 for notes and 256 for the clock");
    // An absurd request is the longest block the module takes, and no more: the same events as 4096 frames.
    let ask = |frames: u32| {
        let abi = Abi::start();
        abi.click(&abi.matrix(0, 0), 0.0);
        octoweb_transport(1);
        abi.render(frames)
    };
    assert_eq!(ask(u32::MAX), ask(4096));
}

#[test]
fn a_tempo_the_engine_cannot_use_is_refused() {
    let _abi = Abi::start();
    assert_eq!(octoweb_set_tempo(120.0), 0);
    for bpm in [0.0f32, -1.0, f32::NAN, f32::INFINITY, 5000.0] {
        assert_ne!(octoweb_set_tempo(bpm), 0, "{bpm} should be refused");
    }
}

#[test]
fn the_attribute_number_the_page_sends_is_the_engines_own_order() {
    assert_eq!(octocore::types::TrackAttr::ALL[MIDI_CHANNEL as usize], octocore::types::TrackAttr::MidiChannel);
    assert_eq!(octocore::types::TrackAttr::ALL.len(), 15, "a new attribute is added at the end, or ABI.md says where it went");
}

/// The (port, channel) of every Note On in `blocks` blocks of 128 frames.
fn note_on_routes(abi: &Abi, blocks: usize) -> Vec<(u8, u8)> {
    let mut routes = Vec::new();
    for _ in 0..blocks {
        routes.extend(abi.render(128).into_iter().filter(|e| e[0] == 0).map(|e| (e[1], e[2])));
    }
    routes
}

#[test]
fn a_track_set_to_a_channel_above_16_plays_on_port_two_and_the_rest_stay_on_port_one() {
    let abi = Abi::start();
    abi.click(&abi.matrix(0, 0), 0.0);
    abi.click(&abi.matrix(1, 0), 100.0);
    abi.click(&abi.matrix(2, 0), 200.0);
    assert_eq!(octoweb_set_track(0, MIDI_CHANNEL, 17), 0, "17 is port 2, channel 1");
    assert_eq!(octoweb_set_track(1, MIDI_CHANNEL, 32), 0, "32 is port 2, channel 16");
    assert_eq!(octoweb_set_track(2, MIDI_CHANNEL, 3), 0, "3 is port 1, channel 3");
    assert_eq!(octoweb_set_tempo(240.0), 0);
    assert_eq!(octoweb_transport(1), 0);
    let mut routes = note_on_routes(&abi, 600);
    routes.sort_unstable();
    routes.dedup();
    assert_eq!(routes, vec![(1, 3), (2, 1), (2, 16)], "each track on the port and channel its number says, and none on the default");
}

#[test]
fn a_track_that_was_never_set_still_plays_on_port_one_channel_one() {
    let abi = Abi::start();
    abi.click(&abi.matrix(0, 0), 0.0);
    octoweb_transport(1);
    let mut routes = note_on_routes(&abi, 600);
    routes.dedup();
    assert_eq!(routes, vec![(1, 1)]);
}

#[test]
fn set_track_refuses_a_track_or_an_attribute_that_does_not_exist_and_changes_nothing() {
    let abi = Abi::start();
    abi.click(&abi.matrix(0, 0), 0.0);
    assert_eq!(octoweb_set_track(0, MIDI_CHANNEL, 17), 0);
    for (track, attr) in [(10, MIDI_CHANNEL), (255, MIDI_CHANNEL), (u32::MAX, MIDI_CHANNEL), (0, 15), (0, 99), (0, u32::MAX)] {
        assert_eq!(octoweb_set_track(track, attr, 3), 5, "track {track} attribute {attr} is out of range (ABI.md code 5)");
    }
    octoweb_transport(1);
    let mut routes = note_on_routes(&abi, 600);
    routes.dedup();
    assert_eq!(routes, vec![(2, 1)], "the refused calls did not touch track 0");
}

#[test]
fn a_channel_outside_1_to_32_is_the_engines_to_clamp_not_a_crash() {
    let abi = Abi::start();
    abi.click(&abi.matrix(0, 0), 0.0);
    for v in [i32::MIN, -1, 0, 33, 1000, i32::MAX] {
        assert_eq!(octoweb_set_track(0, MIDI_CHANNEL, v), 0, "{v}");
    }
    octoweb_transport(1);
    for (port, channel) in note_on_routes(&abi, 600) {
        assert!((1..=2).contains(&port) && (1..=16).contains(&channel), "port {port} channel {channel}");
    }
}

// ----- P6b: the page can write a step (specs/SPEC-0002/p6-runs-when-opened.md), so that it can open with a pattern -----

#[test]
fn set_step_writes_one_attribute_of_one_step_and_the_panel_and_the_notes_show_it() {
    let abi = Abi::start();
    assert_eq!(abi.led(&abi.matrix(3, 5)), 0, "an empty pattern: the step is dark");
    assert_eq!(octoweb_set_step(3, 5, STEP_ACTIVE, 1), 0);
    assert_eq!(abi.led(&abi.matrix(3, 5)), GREEN_STEADY, "an active step is green (manual p.18)");
    assert_eq!(abi.led(&abi.matrix(3, 6)), 0, "and only that step");
    assert_eq!(octoweb_set_step(3, 5, STEP_SKIP, 1), 0);
    assert_eq!(abi.led(&abi.matrix(3, 5)), RED_STEADY, "a skipped step is red (manual p.14)");
    assert_eq!(octoweb_set_step(3, 5, STEP_SKIP, 0), 0);
    // It plays: the track's pitch plus the step's offset.
    assert_eq!(octoweb_set_track(3, TRACK_PITCH, 48), 0);
    assert_eq!(octoweb_set_step(3, 5, STEP_PITCH_OFFSET, 7), 0);
    octoweb_transport(1);
    let mut notes = Vec::new();
    for _ in 0..400 {
        notes.extend(abi.render(128).into_iter().filter(|e| e[0] == 0).map(|e| e[3]));
    }
    assert!(!notes.is_empty(), "the step sounded");
    assert!(notes.iter().all(|n| *n == 55), "48 + 7, got {notes:?}");
}

#[test]
fn set_step_refuses_a_track_a_step_or_an_attribute_that_does_not_exist_and_changes_nothing() {
    let abi = Abi::start();
    let before = abi.leds();
    for (track, step, attr) in [(10, 0, STEP_ACTIVE), (255, 0, STEP_ACTIVE), (u32::MAX, 0, STEP_ACTIVE), (0, 16, STEP_ACTIVE), (0, u32::MAX, STEP_ACTIVE), (0, 0, 12), (0, 0, u32::MAX)] {
        assert_eq!(octoweb_set_step(track, step, attr, 1), 5, "track {track} step {step} attribute {attr} is out of range (ABI.md code 5)");
    }
    assert_eq!(abi.leds(), before, "nothing was written");
}

// ----- SPEC-0005 r1 S2 (WENGE-0034): the page can set how a track spaces a roll (CV-10, and the engine half of CV-11) -----

/// Samples to the tick at 120 BPM and 48 kHz: 60 / 120 / 48 seconds.
const TICK: i64 = 500;
/// The starts of the seven strikes of a strum-9 single note, in ticks after the first (SPEC-0005 section 4 and the manual's table).
const MANUAL_9: [i64; 7] = [0, 5, 10, 17, 23, 30, 45];
const BOUNCE_9: [i64; 7] = [0, 22, 34, 40, 43, 44, 45];
const ACCEL_9: [i64; 7] = [0, 15, 25, 33, 38, 42, 45];
const RIT_9: [i64; 7] = [0, 3, 7, 12, 20, 30, 45];

/// 120 BPM, step 1 of track `track` on (a single note with strum 9: the engine strikes it seven times), on channel `track + 1`.
fn strum_nine(track: u32) {
    assert_eq!(octoweb_set_tempo(120.0), 0);
    assert_eq!(octoweb_set_track(track, MIDI_CHANNEL, track as i32 + 1), 0);
    assert_eq!(octoweb_set_step(track, 0, STEP_ACTIVE, 1), 0);
    assert_eq!(octoweb_set_step(track, 0, STEP_STRUM, 9), 0);
}

/// The Note Ons of the first `blocks` blocks as (channel, sample since the first block), in the order they come.
fn note_ons(abi: &Abi, blocks: usize) -> Vec<(u8, i64)> {
    let mut out = Vec::new();
    for b in 0..blocks as i64 {
        for e in abi.render(128) {
            if e[0] == 0 {
                out.push((e[2], b * 128 + i64::from(u32::from_le_bytes([e[8], e[9], e[10], e[11]]))));
            }
        }
    }
    out
}

/// The starts of the first seven Note Ons on `channel`, in ticks after the first.
fn roll_ticks(ons: &[(u8, i64)], channel: u8) -> Vec<i64> {
    let at: Vec<i64> = ons.iter().filter(|(c, _)| *c == channel).map(|(_, t)| *t).take(7).collect();
    assert_eq!(at.len(), 7, "a strum-9 single note strikes seven times, got {at:?}");
    at.iter().map(|t| (t - at[0]) / TICK).collect()
}

#[test]
fn set_chord_style_refuses_a_track_a_style_or_a_warp_that_does_not_exist_and_changes_nothing() {
    let abi = Abi::start();
    strum_nine(0);
    assert_eq!(octoweb_set_chord_style(0, 1, -5), 0, "Accel");
    for (track, style, warp) in [
        (10, 1, -5),
        (255, 1, -5),
        (u32::MAX, 1, -5), // a track that does not exist
        (0, 2, -5),
        (0, 99, -5),
        (0, u32::MAX, -5), // a style that does not exist
        (0, 1, -6),
        (0, 1, 3),
        (0, 1, 11),
        (0, 1, -11),
        (0, 1, 100),
        (0, 1, i32::MIN),
        (0, 1, i32::MAX), // a warp that is not one of the five stored values
    ] {
        assert_eq!(octoweb_set_chord_style(track, style, warp), 5, "track {track} style {style} warp {warp} is out of range (ABI.md code 5)");
    }
    octoweb_transport(1);
    assert_eq!(roll_ticks(&note_ons(&abi, 800), 1), ACCEL_9, "the refused calls did not touch track 0: it is still Accel");
}

#[test]
fn set_chord_style_takes_the_five_stored_warps_and_manual_ignores_the_warp() {
    for warp in [-10, -5, 0, 5, 10] {
        let _abi = Abi::start();
        assert_eq!(octoweb_set_chord_style(0, 1, warp), 0, "Curve at warp {warp} tenths");
    }
    for warp in [-10, 0, 7, 1000] {
        let _abi = Abi::start();
        assert_eq!(octoweb_set_chord_style(3, 0, warp), 0, "Manual takes any warp and ignores it ({warp})");
    }
}

#[test]
fn each_preset_moves_the_starts_of_a_roll_to_the_table_and_off_is_the_manuals() {
    for (name, style, warp, want) in [("Manual", 0, 0, MANUAL_9), ("Accel", 1, -5, ACCEL_9), ("Bounce", 1, -10, BOUNCE_9), ("Rit", 1, 5, RIT_9)] {
        let abi = Abi::start();
        strum_nine(0);
        assert_eq!(octoweb_set_chord_style(0, style, warp), 0);
        octoweb_transport(1);
        assert_eq!(roll_ticks(&note_ons(&abi, 800), 1), want, "{name}");
    }
}

#[test]
fn the_starts_the_page_gets_are_the_engines_own_table() {
    use octocore::curve::{roll_start_ticks, ChordStyle};
    for (style, warp, want) in [(0, 0, MANUAL_9), (1, -5, ACCEL_9), (1, -10, BOUNCE_9), (1, 5, RIT_9)] {
        let chosen = ChordStyle::from_wire(style, warp).expect("a preset the page can send");
        let table: Vec<i64> = (1..=7).map(|k| i64::from(roll_start_ticks(chosen, 9, k, 7))).collect();
        assert_eq!(table, want, "{chosen:?}: the pinned numbers above are what octocore makes");
    }
}

#[test]
fn a_style_belongs_to_the_track_it_was_set_on() {
    let abi = Abi::start();
    strum_nine(0);
    strum_nine(1);
    assert_eq!(octoweb_set_chord_style(1, 1, -10), 0, "only track 1 is Bounce");
    octoweb_transport(1);
    let ons = note_ons(&abi, 800);
    assert_eq!(roll_ticks(&ons, 1), MANUAL_9, "track 0 was never set");
    assert_eq!(roll_ticks(&ons, 2), BOUNCE_9);
}

#[test]
fn a_style_set_while_the_transport_runs_takes_effect_from_the_next_step_the_track_fires() {
    let abi = Abi::start();
    strum_nine(0);
    octoweb_transport(1);
    let first = roll_ticks(&note_ons(&abi, 600), 1);
    assert_eq!(first, MANUAL_9);
    assert_eq!(octoweb_set_chord_style(0, 1, 5), 0, "Rit, while running");
    let later = note_ons(&abi, 4000); // a loop of sixteen steps is 192 ticks, 96 000 samples: 750 blocks
    let rolls: Vec<Vec<i64>> = later.chunks(7).filter(|c| c.len() == 7).map(|c| c.iter().map(|(_, t)| (t - c[0].1) / TICK).collect()).collect();
    assert!(rolls.len() >= 2, "the pattern came round again");
    assert_eq!(rolls.last().unwrap(), &RIT_9.to_vec(), "after the change every roll is Rit");
}

#[test]
fn setting_manual_by_the_setter_is_the_engine_that_was_never_asked_byte_for_byte() {
    let run = |ask: bool| -> Vec<[u8; EVENT_BYTES]> {
        let abi = Abi::start_with(48_000.0, 7);
        strum_nine(0);
        strum_nine(4);
        if ask {
            assert_eq!(octoweb_set_chord_style(0, 1, -10), 0, "Bounce, then back");
            assert_eq!(octoweb_set_chord_style(0, 0, 0), 0);
            assert_eq!(octoweb_set_chord_style(4, 0, 0), 0);
        }
        octoweb_transport(1);
        (0..1500).flat_map(|_| abi.render(128)).collect()
    };
    let (plain, asked) = (run(false), run(true));
    assert!(plain.len() > 40, "the run made notes");
    assert_eq!(plain, asked, "Off is the Octopus (CV-1) through the page's setter too");
}
