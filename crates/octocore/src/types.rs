//! The command/event/snapshot interfaces of docs/03-sequencer-core.md §5-6. This is the
//! FFI-facing surface: `octoffi` wraps these `#[repr(C)]` types.

/// Identifies a control on the front panel: the `n` of a control in
/// `contracts/controls.json` (ADR-0007). `n` is unique, assigned once and never reused, and
/// is below `MAX_CONTROLS`, so it also indexes `Snapshot::leds`. The engine does not read the
/// inventory; the panel controller (`crates/octoface`) is what interprets the number.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ControlId(pub u32);

/// A per-track attribute the logical commands and the FFI setters can write. The order is
/// the C enum's order in `octoffi.h` (`OCTO_TRACK_*`), so the numbers must not change.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrackAttr {
    Pitch,
    Velocity,
    LengthFactor,
    StartFactor,
    DirectionRaw,
    Rotation,
    Amount,
    Groove,
    MidiChannel,
    Muted,
    Soloed,
    Paused,
    RecordArmed,
    IsFeeder,
    IsListener,
}

impl TrackAttr {
    pub const ALL: [TrackAttr; 15] = [
        TrackAttr::Pitch,
        TrackAttr::Velocity,
        TrackAttr::LengthFactor,
        TrackAttr::StartFactor,
        TrackAttr::DirectionRaw,
        TrackAttr::Rotation,
        TrackAttr::Amount,
        TrackAttr::Groove,
        TrackAttr::MidiChannel,
        TrackAttr::Muted,
        TrackAttr::Soloed,
        TrackAttr::Paused,
        TrackAttr::RecordArmed,
        TrackAttr::IsFeeder,
        TrackAttr::IsListener,
    ];

    pub fn from_u32(i: u32) -> Option<TrackAttr> {
        TrackAttr::ALL.get(i as usize).copied()
    }
}

/// A per-step attribute. The order is the C enum's order (`OCTO_STEP_*`).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepAttr {
    Active,
    Skip,
    PitchOffset,
    VelocityOffset,
    LengthTicks,
    LengthMultiplier,
    StartOffset,
    Amount,
    Strum,
    Hyperstep,
    /// 0 = no phrase (`None`); 1..=48 indexes `Grid::phrases` (1-based).
    Phrase,
    /// Phrase time-compression; 8 is neutral (CE v5.30 p.17). Stored only.
    PhrasePos,
}

impl StepAttr {
    pub const ALL: [StepAttr; 12] = [
        StepAttr::Active,
        StepAttr::Skip,
        StepAttr::PitchOffset,
        StepAttr::VelocityOffset,
        StepAttr::LengthTicks,
        StepAttr::LengthMultiplier,
        StepAttr::StartOffset,
        StepAttr::Amount,
        StepAttr::Strum,
        StepAttr::Hyperstep,
        StepAttr::Phrase,
        StepAttr::PhrasePos,
    ];

    pub fn from_u32(i: u32) -> Option<StepAttr> {
        StepAttr::ALL.get(i as usize).copied()
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Command {
    Play,
    Stop,
    Continue,
    Reset,
    ButtonDown { control: ControlId, velocity_mm_s: f32 },
    ButtonUp { control: ControlId },
    EncoderTurn { control: ControlId, detents: i16, angular_velocity: f32 },
    SetActivePage { bank: u8, page: u8 },
    SetMode { mode: crate::domain::Mode },
    HostTransport { ppqn_pos: u64, bpm: f32, playing: bool },
    LoadState { handle: u64 },
    /// Writes one attribute of one track of the active page. A `track` of 10 or more does
    /// nothing. Added after `LoadState` (SPEC-0001 O6), so every tag above keeps its number.
    SetTrack { track: u8, attr: TrackAttr, value: i32 },
    /// Writes one attribute of one step. A `track` of 10 or more, or a `step` of 16 or
    /// more, does nothing.
    SetStep { track: u8, step: u8, attr: StepAttr, value: i32 },
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    NoteOn { port: u8, ch: u8, note: u8, vel: u8, at_sample: u32 },
    NoteOff { port: u8, ch: u8, note: u8, at_sample: u32 },
    Cc { port: u8, ch: u8, cc: u8, val: u8, at_sample: u32 },
    /// Pitch bend, 14 bits: 0 to 16383 with 8192 the centre (no bend). The step editor works on
    /// the top 7 bits only (CE v5.30 p.91), so the engine sends `value = step MCC << 7`, and
    /// an MCC of 64 is centre. Added after `Cc`, so the tags of the events above do not change.
    PitchBend { port: u8, ch: u8, value: u16, at_sample: u32 },
    /// Channel pressure (aftertouch), 0 to 127.
    ChannelPressure { port: u8, ch: u8, value: u8, at_sample: u32 },
}

/// A MIDI system real-time message the engine can send as a master clock (ADR-0009). These are not `Event` variants: they have no
/// port and no channel, and they travel beside the events in their own list (`Engine::take_realtime`).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Realtime {
    Clock = 0xF8,
    Start = 0xFA,
    Continue = 0xFB,
    Stop = 0xFC,
}

/// A real-time message and the sample, inside the render that made it, where it falls.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RealtimeEvent {
    pub msg: Realtime,
    pub at_sample: u32,
}

/// Maximum events emitted by a single `Engine::render` call. Sized for "full
/// density" (docs §7): ten tracks, max chord polyphony, all firing at once, each
/// producing a note-on and a matched note-off.
pub const MAX_EVENTS_PER_TICK: usize = 256;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct LedColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Led {
    pub color: LedColor,
    /// The commanded target, not the currently-displayed value (docs §6).
    pub target: f32,
}

pub const MAX_CONTROLS: usize = 512;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct EncoderState {
    pub angle_radians: f32,
    pub detent_index: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct PlayheadState {
    /// The step the track fired last, 0 to 15, in any direction; `engine::NOT_PLAYING` (255) before it has fired one. It runs ahead of the
    /// sound by the engine's lead (`snapshot.rs`).
    pub step_index: u8,
    pub track_index: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct TransportState {
    pub playing: bool,
    pub tick: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct ActiveRefs {
    pub bank: u8,
    pub page: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Snapshot {
    pub generation: u64,
    pub leds: [Led; MAX_CONTROLS],
    pub encoders: [EncoderState; 20],
    pub playheads: [PlayheadState; crate::domain::TRACK_COUNT],
    pub transport: TransportState,
    pub mode: crate::domain::Mode,
    pub active: ActiveRefs,
}
