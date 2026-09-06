//! The Octopus data model: Grid -> Bank -> Page -> Track -> Step.
//!
//! Dimensions and the attribute table are ports of docs/03-sequencer-core.md §2,
//! which is the authoritative source over the archived v1 JS/C++ wherever the two
//! disagree (v1 predates this spec and is explicitly not to be merged — see
//! adr/0002). Fixed-size arrays throughout, no heap allocation, in keeping with D1's
//! "no_std-friendly... zero allocation after init".

pub const TRACK_COUNT: usize = 10;
pub const STEP_COUNT: usize = 16;
pub const BANK_COUNT: usize = 10;
pub const PAGE_COUNT: usize = 16;
pub const PAGE_SET_COUNT: usize = 16;
pub const PHRASE_COUNT: usize = 16;
pub const PHRASE_NOTE_COUNT: usize = 8;
pub const CHORD_POOL_MAX: usize = 6; // + the base pitch = 7 simultaneous notes, matching v1's cap
pub const CHAIN_MEMBERS_MAX: usize = TRACK_COUNT - 1;
pub const MAX_SCALE_INTERVALS: usize = 12;
/// PROVISIONAL — see tests/conformance/AMBIGUITIES.md "user-programmed directions".
pub const USER_DIRECTION_COUNT: usize = 11; // dir values 6..=16
pub const TICKS_PER_QUARTER: u32 = 192;
pub const DEFAULT_STEP_TICKS: u32 = 12; // 1/16 note at 192 PPQN

/// Track indices run 0..=9 and correspond to panel rows 0 (bottom) to 9 (top).
/// docs/03-sequencer-core.md §2: "Do not silently flip this." The effector's
/// feeder/listener direction and step-event "following step" timing both depend on
/// higher indices being processed before lower ones in a tick.
pub type TrackIndex = u8;

/// Fixed directions 1..=5, plus 6+ meaning "user-programmed, index = value - 6".
///
/// PROVISIONAL mapping for 4/5 — see AMBIGUITIES.md "direction 4/5 assignment". Both
/// archived prior ports (v1 JS `_dir` handling and `archive/v1-cpp-juce`'s
/// `Direction` enum) independently agree that 4 is a biased random walk ("brownian")
/// and 5 is a uniform jump ("random"), which is the *opposite* pairing from the
/// order the words appear in docs/03-sequencer-core.md §3's prose ("random,
/// brownian"). Two independent prior implementations outrank one un-cited prose
/// sentence, so that's what's implemented here — but neither prior implementation
/// had the real manual either, so this is still open until someone checks it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Forward,
    Reverse,
    PingPong,
    Brownian,
    Random,
    /// `docs/03-sequencer-core.md` §3: "the user selects an ordering by lighting
    /// cells in rows above row 0... if all rows are empty, the direction is
    /// random." Index into `Page::user_directions` (raw value 6..=16 maps to
    /// index 0..=10).
    UserProgrammed(u8),
}

impl Direction {
    pub fn from_raw(v: u8) -> Direction {
        match v {
            1 => Direction::Forward,
            2 => Direction::Reverse,
            3 => Direction::PingPong,
            4 => Direction::Brownian,
            5 => Direction::Random,
            n if n >= 6 => Direction::UserProgrammed((n - 6).min(USER_DIRECTION_COUNT as u8 - 1)),
            _ => Direction::Forward,
        }
    }

    pub fn to_raw(self) -> u8 {
        match self {
            Direction::Forward => 1,
            Direction::Reverse => 2,
            Direction::PingPong => 3,
            Direction::Brownian => 4,
            Direction::Random => 5,
            Direction::UserProgrammed(i) => 6 + i,
        }
    }
}

/// A user-programmed direction (dir 6+): for each of the 16 step columns, which
/// rows (above row 0) light that column.
///
/// PROVISIONAL data shape and traversal rule — see AMBIGUITIES.md
/// "user-programmed directions". The runtime currently falls back to
/// `Direction::Random` for *any* `UserProgrammed` direction, empty or not, until the
/// real column-order traversal is specified from the manual; the data shape is kept
/// here so that traversal can be implemented later without a state-model change.
#[derive(Clone, Copy, Debug)]
pub struct UserDirection {
    /// One bitmask per row (bit `c` set = column `c` is lit in that row).
    pub rows: [u16; TRACK_COUNT - 1],
}

impl Default for UserDirection {
    fn default() -> Self {
        UserDirection { rows: [0; TRACK_COUNT - 1] }
    }
}

impl UserDirection {
    pub fn is_empty(&self) -> bool {
        self.rows.iter().all(|&r| r == 0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChainBase {
    /// Attribute values (multiplier, len/sta factors, mcc, etc.) are read from
    /// whichever track is currently playing.
    Individual,
    /// Attribute values are always read from the chain head, regardless of which
    /// member is currently playing.
    Head,
}

/// The effector role (docs/03-sequencer-core.md §3, "The effector"). Track 0 can
/// never be a feeder — enforced where roles are assigned, not in this type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrackRole {
    None,
    Feeder,
    Listener,
}

/// Track-level MCC attribute: which CC the track emits, or one of the two special
/// channel-wide targets. `docs/03-sequencer-core.md` §3 "MCC".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mcc {
    None,
    Cc(u8),
    Bend,
    Pressure,
}

#[derive(Clone, Copy, Debug)]
pub struct ChordPool {
    /// Semitone offsets above the step's base pitch. The base pitch itself (offset
    /// 0) is always an implicit pool member.
    pub offsets: [i8; CHORD_POOL_MAX],
    pub count: u8,
    /// How many notes of the pool actually sound (docs §3 "Chords"): if
    /// `polyphony >= count + 1` every pool note (plus the base) plays; otherwise
    /// `polyphony` notes are picked from the pool without replacement.
    pub polyphony: u8,
}

impl Default for ChordPool {
    fn default() -> Self {
        ChordPool { offsets: [0; CHORD_POOL_MAX], count: 0, polyphony: 1 }
    }
}

/// A step event targets another track and either sets an absolute attribute or
/// toggles a transport-adjacent flag. docs/03-sequencer-core.md §3: "Events applied
/// to higher-indexed tracks execute on the following step, because of the top-down
/// processing order." — handled by the engine's deferred-event queue, not here.
#[derive(Clone, Copy, Debug)]
pub enum StepEventKind {
    SetPos(u8),
    SetDir(u8),
    SetMch(u8),
    ToggleMute,
    ToggleSolo,
    ToggleRecord,
}

#[derive(Clone, Copy, Debug)]
pub struct StepEvent {
    pub target_track: TrackIndex,
    pub kind: StepEventKind,
}

#[derive(Clone, Copy, Debug)]
pub struct Step {
    pub active: bool,
    pub skip: bool,
    pub pitch_offset: i8,
    pub velocity_offset: i8,
    pub length_ticks: u8,
    pub length_multiplier: u8,
    pub start_offset: i8,
    /// GRV at step level: a phrase index 1..=16, or `None` for no phrase.
    /// (GRV at track level means something else entirely — the shuffle amount,
    /// `Track::groove`. Same attribute code, two unrelated meanings; that is the
    /// spec's own table, not a bug introduced here.)
    pub phrase: Option<u8>,
    /// MCC at step level: the value this step emits on the track's configured CC
    /// (or bend/pressure target).
    pub mcc_value: Option<u8>,
    pub chord: ChordPool,
    /// -9..=9. Sign selects strike direction (up/down); magnitude indexes
    /// `tables::strum_offset_ticks`.
    pub strum: i8,
    /// docs/03-sequencer-core.md §3 "Hypersteps": true if pressing a step button in
    /// another row while *this* step is held should carry this step's PIT/VEL into
    /// the other row. The cross-track wiring is runtime input state, handled by
    /// `Engine::handle_command`, not stored per-step.
    pub hyperstep: bool,
    pub event: Option<StepEvent>,
}

impl Default for Step {
    fn default() -> Self {
        Step {
            active: false,
            skip: false,
            pitch_offset: 0,
            velocity_offset: 0,
            length_ticks: DEFAULT_STEP_TICKS as u8,
            length_multiplier: 1,
            start_offset: 0,
            phrase: None,
            mcc_value: None,
            chord: ChordPool::default(),
            strum: 0,
            hyperstep: false,
            event: None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Track {
    pub pitch: u8,
    pub velocity: u8,
    /// Neutral = 8 (v1 `len_factor`/`sta_factor`; the tick loop treats 8 as ×1.0).
    pub length_factor: u8,
    pub start_factor: u8,
    pub direction_raw: u8,
    /// POS attribute: a static rotation/phase offset added to the runtime step
    /// cursor before indexing into `steps` (docs §2: "track rotation / phase").
    /// Settable live via a step event (`StepEventKind::SetPos`).
    pub rotation: u8,
    pub amount: i8,
    /// Shuffle amount 0..=16, indexes `tables::grv_delay_ticks`.
    pub groove: u8,
    pub mcc: Mcc,
    /// 1..=32: channels 1..=16 are port 1, 17..=32 are port 2.
    pub midi_channel: u8,
    pub multiplier_num: u8,
    pub multiplier_den: u8,
    pub paused: bool,
    pub muted: bool,
    pub soloed: bool,
    pub record_armed: bool,
    pub role: TrackRole,
    pub chain_head: Option<TrackIndex>,
    pub chain_members: [Option<TrackIndex>; CHAIN_MEMBERS_MAX],
    pub chain_member_count: u8,
    pub chain_base: ChainBase,
    pub steps: [Step; STEP_COUNT],
}

impl Track {
    pub fn multiplier(&self) -> f32 {
        if self.multiplier_den == 0 {
            1.0
        } else {
            self.multiplier_num as f32 / self.multiplier_den as f32
        }
    }

    pub fn direction(&self) -> Direction {
        Direction::from_raw(self.direction_raw)
    }

    /// Manual default PIT values for tracks 9..0: C3 D3 E3 G3 A3 C5 D5 E5 G5 A5.
    /// Ported from `defaultTrack` in `archive/v1-max4live/octopus_data.js`.
    /// PROVISIONAL — not yet cross-checked against a manual page number.
    const DEFAULT_PITCH: [u8; TRACK_COUNT] = [57, 55, 52, 50, 48, 60, 62, 64, 67, 69];

    pub fn default_for_index(index: TrackIndex) -> Self {
        Track {
            pitch: Self::DEFAULT_PITCH[(index as usize).min(TRACK_COUNT - 1)],
            velocity: 100,
            length_factor: 8,
            start_factor: 8,
            direction_raw: 1,
            rotation: 0,
            amount: 0,
            groove: 0,
            mcc: Mcc::None,
            midi_channel: 1,
            multiplier_num: 1,
            multiplier_den: 1,
            paused: false,
            muted: false,
            soloed: false,
            record_armed: false,
            role: TrackRole::None,
            chain_head: None,
            chain_members: [None; CHAIN_MEMBERS_MAX],
            chain_member_count: 0,
            chain_base: ChainBase::Individual,
            steps: [Step::default(); STEP_COUNT],
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ScaleForce {
    pub enabled: bool,
    pub locked: bool,
    pub root: u8,
    pub intervals: [i8; MAX_SCALE_INTERVALS],
    pub interval_count: u8,
}

impl ScaleForce {
    pub fn major(root: u8) -> Self {
        let default_intervals: [i8; 7] = [0, 2, 4, 5, 7, 9, 11];
        let mut intervals = [0i8; MAX_SCALE_INTERVALS];
        intervals[..7].copy_from_slice(&default_intervals);
        ScaleForce { enabled: false, locked: false, root, intervals, interval_count: 7 }
    }

    pub fn intervals(&self) -> &[i8] {
        &self.intervals[..self.interval_count as usize]
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Page {
    pub pitch_offset: i8,
    pub velocity_factor: u8,
    pub length: u8,
    /// docs/03-sequencer-core.md §2 lists FLT as page-level flattening in prose but
    /// marks it Track-column in the attribute table with no Page column at all in
    /// that table — the table has no way to express a page-level attribute, so the
    /// prose (unambiguous: "page-level flattening") wins here. See AMBIGUITIES.md.
    pub flatten: bool,
    pub scale: ScaleForce,
    pub cluster_mode: bool,
    pub mute_pattern: [bool; TRACK_COUNT],
    pub user_directions: [UserDirection; USER_DIRECTION_COUNT],
    pub tracks: [Track; TRACK_COUNT],
}

impl Page {
    pub fn default_page() -> Self {
        let mut tracks = [Track::default_for_index(0); TRACK_COUNT];
        for (i, t) in tracks.iter_mut().enumerate() {
            *t = Track::default_for_index(i as TrackIndex);
        }
        Page {
            pitch_offset: 0,
            velocity_factor: 8,
            length: STEP_COUNT as u8,
            flatten: false,
            scale: ScaleForce::major(60),
            cluster_mode: false,
            mute_pattern: [false; TRACK_COUNT],
            user_directions: [UserDirection::default(); USER_DIRECTION_COUNT],
            tracks,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Bank {
    pub pages: [Page; PAGE_COUNT],
}

impl Bank {
    pub fn default_bank() -> Self {
        Bank { pages: [Page::default_page(); PAGE_COUNT] }
    }
}

/// A phrase: 8 notes, each with VEL/PIT/LEN/STA offsets.
/// docs/03-sequencer-core.md §3 "Phrases".
#[derive(Clone, Copy, Debug)]
pub struct PhraseNote {
    pub pitch_offset: i8,
    pub velocity_offset: i8,
    pub length_ticks: u8,
    pub start_offset: i8,
    pub enabled: bool,
}

impl Default for PhraseNote {
    fn default() -> Self {
        PhraseNote {
            pitch_offset: 0,
            velocity_offset: 0,
            length_ticks: DEFAULT_STEP_TICKS as u8,
            start_offset: 0,
            enabled: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhraseType {
    /// Play the enabled notes in order, as programmed.
    Fixed,
    /// PROVISIONAL — types 2/3 are not described anywhere available to this port;
    /// kept as a named-but-unimplemented placeholder rather than guessed at. See
    /// AMBIGUITIES.md "phrase types 2 and 3".
    Reserved2,
    Reserved3,
    /// docs §3: "Type 4 randomises the programmed note attributes."
    Randomized,
}

#[derive(Clone, Copy, Debug)]
pub struct Phrase {
    pub notes: [PhraseNote; PHRASE_NOTE_COUNT],
    pub phrase_type: PhraseType,
    /// How many of the (enabled) notes actually fire — same "probabilistic rest"
    /// mechanic as chord polyphony (docs §3: "a one-note step with polyphony 2
    /// plays half the time").
    pub polyphony: u8,
}

impl Default for Phrase {
    fn default() -> Self {
        Phrase {
            notes: [PhraseNote::default(); PHRASE_NOTE_COUNT],
            phrase_type: PhraseType::Fixed,
            polyphony: PHRASE_NOTE_COUNT as u8,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoutingMode {
    Octopus,
    Fixed,
}

#[derive(Clone, Copy, Debug)]
pub struct FixedRouting {
    pub base_channel_port1: u8,
    pub base_channel_port2: u8,
}

impl Default for FixedRouting {
    fn default() -> Self {
        FixedRouting { base_channel_port1: 1, base_channel_port2: 1 }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ActivePage {
    pub bank: u8,
    pub page: u8,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Grid,
    Page,
    Track,
    Step,
}

/// The whole instrument state. Every level from `Bank` down to `Step` is a plain
/// fixed-size `Copy` type — deliberately, since `Page` (~6KB) is what gets copied
/// once per tick as the engine's per-tick snapshot (see `engine.rs`), and that copy
/// needs to stay cheap. `Grid` itself does not: ten `Bank`s (~96KB each) is
/// ~1MB, too large to construct as a stack value without risking overflow on a
/// constrained thread stack (this is exactly what blew up the test harness during
/// development), so only the outermost `banks` array is heap-backed. This is a
/// one-time allocation at `Engine::new`, consistent with D1's "zero allocation
/// *after* init".
#[derive(Clone, Debug)]
pub struct Grid {
    pub tempo_bpm: f32,
    pub active_page: ActivePage,
    pub mode: Mode,
    pub routing_mode: RoutingMode,
    pub fixed_routing: FixedRouting,
    pub global_scale: ScaleForce,
    pub phrases: [Phrase; PHRASE_COUNT],
    pub banks: Vec<Bank>,
}

impl Grid {
    pub fn default_grid() -> Self {
        let mut banks = Vec::with_capacity(BANK_COUNT);
        for _ in 0..BANK_COUNT {
            banks.push(Bank::default_bank());
        }
        Grid {
            tempo_bpm: 120.0,
            active_page: ActivePage { bank: 0, page: 0 },
            mode: Mode::Grid,
            routing_mode: RoutingMode::Octopus,
            fixed_routing: FixedRouting::default(),
            global_scale: ScaleForce::major(0),
            phrases: [Phrase::default(); PHRASE_COUNT],
            banks,
        }
    }

    pub fn active_page(&self) -> &Page {
        &self.banks[self.active_page.bank as usize].pages[self.active_page.page as usize]
    }

    pub fn active_page_mut(&mut self) -> &mut Page {
        &mut self.banks[self.active_page.bank as usize].pages[self.active_page.page as usize]
    }
}

pub fn clamp_midi(v: i32) -> u8 {
    v.clamp(0, 127) as u8
}
