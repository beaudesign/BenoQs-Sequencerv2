//! The command/event/snapshot interfaces, ported verbatim (field-for-field) from
//! docs/03-sequencer-core.md §5-6. This is the FFI-facing surface: `octoffi` wraps
//! these `#[repr(C)]` types for the Swift side once it exists.

/// Identifies a physical control. Namespace-shared with `panel.truth.json` once that
/// contract exists (docs/03-sequencer-core.md §5: "the same identifier used in
/// panel.truth.json. One namespace for the whole system.") — currently just an
/// opaque numeric id, since panel.truth.json does not exist yet (see
/// reference/NOTES.md).
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ControlId(pub u32);

#[repr(C)]
#[derive(Clone, Copy, Debug)]
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
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    NoteOn { port: u8, ch: u8, note: u8, vel: u8, at_sample: u32 },
    NoteOff { port: u8, ch: u8, note: u8, at_sample: u32 },
    Cc { port: u8, ch: u8, cc: u8, val: u8, at_sample: u32 },
}

/// Maximum events emitted by a single `Engine::tick_samples` call. Sized for "full
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
    /// The commanded target, not the currently-displayed value — docs §6: "the
    /// renderer runs the rise/decay simulation itself at frame rate... the core
    /// says 'on', the renderer says how 'on' looks 4.2ms later."
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
