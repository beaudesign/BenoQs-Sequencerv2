//! Reads and writes single attributes of a track or a step by number. This is the code the
//! FFI setters used to hold, moved here so that the logical commands `SetTrack` and
//! `SetStep` and the direct setters share one clamping table (SPEC-0001 O6, test C1).

use crate::domain::{Grid, STEP_COUNT, TRACK_COUNT};
use crate::types::{StepAttr, TrackAttr};

fn track_in_range(track: u8) -> bool {
    (track as usize) < TRACK_COUNT
}

fn step_in_range(step: u8) -> bool {
    (step as usize) < STEP_COUNT
}

impl Grid {
    /// Writes `value`, clamped to the attribute's range, into the active page. Returns
    /// `false` and changes nothing when `track` is out of range.
    pub fn set_track_attr(&mut self, track: u8, attr: TrackAttr, value: i32) -> bool {
        if !track_in_range(track) {
            return false;
        }
        let t = &mut self.active_page_mut().tracks[track as usize];
        match attr {
            TrackAttr::Pitch => t.pitch = value.clamp(0, 127) as u8,
            TrackAttr::Velocity => t.velocity = value.clamp(0, 127) as u8,
            TrackAttr::LengthFactor => t.length_factor = value.clamp(0, 16) as u8,
            TrackAttr::StartFactor => t.start_factor = value.clamp(0, 16) as u8,
            TrackAttr::DirectionRaw => t.direction_raw = value.clamp(1, 16) as u8,
            TrackAttr::Rotation => t.rotation = value.clamp(0, 255) as u8,
            TrackAttr::Amount => t.amount = value.clamp(-128, 127) as i8,
            TrackAttr::Groove => t.groove = value.clamp(0, 16) as u8,
            TrackAttr::MidiChannel => t.midi_channel = value.clamp(1, 32) as u8,
            TrackAttr::Muted => t.muted = value != 0,
            TrackAttr::Soloed => t.soloed = value != 0,
            TrackAttr::Paused => t.paused = value != 0,
            TrackAttr::RecordArmed => t.record_armed = value != 0,
            TrackAttr::IsFeeder => t.is_feeder = value != 0,
            TrackAttr::IsListener => t.is_listener = value != 0,
        }
        true
    }

    /// Reads an attribute of the active page. Returns 0 when `track` is out of range, which
    /// is indistinguishable from a real 0; check the index first if that matters.
    pub fn track_attr(&self, track: u8, attr: TrackAttr) -> i32 {
        if !track_in_range(track) {
            return 0;
        }
        let t = &self.active_page().tracks[track as usize];
        match attr {
            TrackAttr::Pitch => t.pitch as i32,
            TrackAttr::Velocity => t.velocity as i32,
            TrackAttr::LengthFactor => t.length_factor as i32,
            TrackAttr::StartFactor => t.start_factor as i32,
            TrackAttr::DirectionRaw => t.direction_raw as i32,
            TrackAttr::Rotation => t.rotation as i32,
            TrackAttr::Amount => t.amount as i32,
            TrackAttr::Groove => t.groove as i32,
            TrackAttr::MidiChannel => t.midi_channel as i32,
            TrackAttr::Muted => t.muted as i32,
            TrackAttr::Soloed => t.soloed as i32,
            TrackAttr::Paused => t.paused as i32,
            TrackAttr::RecordArmed => t.record_armed as i32,
            TrackAttr::IsFeeder => t.is_feeder as i32,
            TrackAttr::IsListener => t.is_listener as i32,
        }
    }

    /// Like `set_track_attr`, for a step. `false` when `track` or `step` is out of range.
    pub fn set_step_attr(&mut self, track: u8, step: u8, attr: StepAttr, value: i32) -> bool {
        if !track_in_range(track) || !step_in_range(step) {
            return false;
        }
        let s = &mut self.active_page_mut().tracks[track as usize].steps[step as usize];
        match attr {
            StepAttr::Active => s.active = value != 0,
            StepAttr::Skip => s.skip = value != 0,
            StepAttr::PitchOffset => s.pitch_offset = value.clamp(-128, 127) as i8,
            StepAttr::VelocityOffset => s.velocity_offset = value.clamp(-128, 127) as i8,
            StepAttr::LengthTicks => s.length_ticks = value.clamp(1, 192) as u8,
            StepAttr::LengthMultiplier => s.length_multiplier = value.clamp(1, 8) as u8,
            StepAttr::StartOffset => s.start_offset = value.clamp(-5, 5) as i8,
            StepAttr::Amount => s.amount = value.clamp(-128, 127) as i8,
            StepAttr::Strum => s.strum = value.clamp(-9, 9) as i8,
            StepAttr::Hyperstep => s.hyperstep = value != 0,
            StepAttr::Phrase => {
                s.phrase = if value <= 0 { None } else { Some(value.clamp(1, 48) as u8) };
            }
            StepAttr::PhrasePos => s.phrase_pos = value.clamp(1, 16) as u8,
        }
        true
    }

    /// Reads a step attribute of the active page. 0 when out of range.
    pub fn step_attr(&self, track: u8, step: u8, attr: StepAttr) -> i32 {
        if !track_in_range(track) || !step_in_range(step) {
            return 0;
        }
        let s = &self.active_page().tracks[track as usize].steps[step as usize];
        match attr {
            StepAttr::Active => s.active as i32,
            StepAttr::Skip => s.skip as i32,
            StepAttr::PitchOffset => s.pitch_offset as i32,
            StepAttr::VelocityOffset => s.velocity_offset as i32,
            StepAttr::LengthTicks => s.length_ticks as i32,
            StepAttr::LengthMultiplier => s.length_multiplier as i32,
            StepAttr::StartOffset => s.start_offset as i32,
            StepAttr::Amount => s.amount as i32,
            StepAttr::Strum => s.strum as i32,
            StepAttr::Hyperstep => s.hyperstep as i32,
            StepAttr::Phrase => s.phrase.map(|p| p as i32).unwrap_or(0),
            StepAttr::PhrasePos => s.phrase_pos as i32,
        }
    }
}
