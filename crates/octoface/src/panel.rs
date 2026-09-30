//! The panel controller: a state machine from button presses to engine commands and LED frames.
//!
//! Five workflows are built (SPEC-0002 spike S5), each cited to the manual pages that say so and
//! covered by the fixtures in `tests/conformance/panel/`:
//!
//! - **Page view step toggle.** With EDIT steady green a matrix key toggles its step [p068]; a
//!   skipped step is un-skipped by one press [p014].
//! - **Step zoom.** Hold Step Mode and press a matrix key [p013]. The selected step's LED and the
//!   Main Mute LED follow the table on p014. TGL toggles the step and Main Mute toggles Skip [p014].
//! - **Leaving Step zoom.** ESC or the PAGE button [p015], or ZOM [p014].
//! - **The EDIT cycle.** Steady green, flashing orange (preview), flashing green (perform) [p068,
//!   p069]. In preview a key plays the step and sets nothing; in perform it does neither.
//! - **PLAY mode.** PLAY flashes orange and Program lights red; Program keeps the changes, PLAY
//!   again or Stop discards them [p066].
//!
//! Where the manual is silent the controller either does nothing or makes a provisional choice
//! that names its question (`tests/conformance/panel/QUESTIONS.md`); no fixture asserts those.
//! Time is passed in and unused so far: every gesture built is a press. Holds and double-clicks
//! wait for the answer to Q24.

use crate::layout::{Key, Layout, Role};
use crate::led::{Colour, Led, LedFrame};
use crate::view::{PageView, StepView};
use octocore::domain::Mode;
use octocore::types::{Command, ControlId, StepAttr, MAX_CONTROLS};

/// Which view the controller is showing. The other modes (Track, Grid) are not built yet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PanelMode {
    Page,
    Step,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Input {
    Down(ControlId),
    Up(ControlId),
    /// Accepted and ignored until a workflow uses it (Step Shift is the first, p015).
    Turn { control: ControlId, detents: i16 },
}

/// A request for an engine capability that does not exist yet. The fixtures assert the intent; the
/// engine side is a request to the Metronome (`journal/metronome/requests/`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Intent {
    /// Play the step's MIDI without setting it: EDIT preview [p068]. `track` and `step` are
    /// 0-based, as in `Command::SetStep`.
    Audition { track: u8, step: u8 },
    /// PLAY mode: take a snapshot of the playing page [p066].
    SnapshotTake,
    /// Program in PLAY mode: keep the changes [p066].
    SnapshotKeep,
    /// PLAY again, or Stop, in PLAY mode: discard the changes and recall the snapshot [p066].
    SnapshotRecall,
}

/// What one input produced.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Out {
    pub commands: Vec<Command>,
    pub intents: Vec<Intent>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EditState {
    /// Steady green: the step keys toggle steps [p068].
    Normal,
    /// Flashing orange: EDIT PREVIEW [p068].
    Preview,
    /// Flashing green: EDIT PERFORM [p069].
    Perform,
}

impl EditState {
    /// One click on EDIT. Preview to Normal takes two clicks [p068: "press the EDIT button twice"],
    /// which is through Perform [p069]. The Red MCC state is reached by a double-click [p070] and
    /// is not built (Q24).
    fn next(self) -> EditState {
        match self {
            EditState::Normal => EditState::Preview,
            EditState::Preview => EditState::Perform,
            EditState::Perform => EditState::Normal,
        }
    }
}

/// The step Step zoom is showing: matrix row `track` (track `track` in Page view) and step 0 to 15.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Zoom {
    track: u8,
    step: u8,
}

pub struct Panel {
    layout: Layout,
    mode: PanelMode,
    zoom: Option<Zoom>,
    edit: EditState,
    play_mode: bool,
    held: [bool; MAX_CONTROLS],
}

/// A step's LED in Page view: green on [p018, "active step (Green)"], red skipped [p014], orange with an
/// event [p034, p038]. A chord step is orange too, which no page says (Q37, provisional). A skipped step
/// shows red whatever else it is.
fn page_led(s: StepView) -> Led {
    if s.skip {
        Led::steady(Colour::Red)
    } else if s.chord || s.event {
        Led::steady(Colour::Orange)
    } else if s.on {
        Led::steady(Colour::Green)
    } else {
        Led::OFF
    }
}

/// The Step zoom table on p014: the selected step's LED, then the Main Mute LED.
///
/// The table's "Step Status" is one of Off, On, Chord/Event and Hyperstep, each with and without
/// Skip. A step can be several at once and the table does not order them: Hyperstep, then
/// Chord/Event, then On is the controller's choice (Q35). The footnote, "If Event & Skip but the
/// underlying step is toggled off", turns the Main Mute LED from flashing to steady orange.
fn zoom_leds(s: StepView) -> (Led, Led) {
    use Colour::{Green, Orange, Red};
    let off = Led::OFF;
    if s.hyperstep {
        return if s.skip { (Led::flash(Red), Led::shine(Red)) } else { (Led::shine(Red), off) };
    }
    if s.chord || s.event {
        return if s.skip {
            let mute = if s.event && !s.on { Led::steady(Orange) } else { Led::flash(Orange) };
            (Led::flash(Red), mute)
        } else {
            (Led::flash(Orange), off)
        };
    }
    match (s.on, s.skip) {
        (false, false) => (Led::flash(Red), off),
        (false, true) => (Led::flash(Red), Led::steady(Red)),
        (true, false) => (Led::flash(Green), off),
        (true, true) => (Led::flash(Red), Led::flash(Green)),
    }
}

impl Panel {
    pub fn new(layout: Layout) -> Panel {
        Panel {
            layout,
            mode: PanelMode::Page,
            zoom: None,
            edit: EditState::Normal,
            play_mode: false,
            held: [false; MAX_CONTROLS],
        }
    }

    pub fn mode(&self) -> PanelMode {
        self.mode
    }

    fn is_held(&self, role: Role) -> bool {
        self.held.get(self.layout.role(role).0 as usize).copied().unwrap_or(false)
    }

    /// One input. `view` is the engine's active page as the caller reads it, before this input.
    pub fn input(&mut self, _now_ms: u64, input: Input, view: &PageView) -> Out {
        let mut out = Out::default();
        match input {
            Input::Down(id) => {
                let Some(slot) = self.held.get_mut(id.0 as usize) else { return out };
                if std::mem::replace(slot, true) {
                    // A second press of a key already down (a second pointer, a repeat): not a press.
                    return out;
                }
                match self.layout.key(id) {
                    Some(Key::Matrix { row, step }) => self.matrix_down(row, step, view, &mut out),
                    Some(Key::Role(role)) => self.role_down(role, view, &mut out),
                    None => {}
                }
            }
            Input::Up(id) => {
                if let Some(slot) = self.held.get_mut(id.0 as usize) {
                    *slot = false;
                }
            }
            Input::Turn { .. } => {}
        }
        out
    }

    fn matrix_down(&mut self, row: u8, step: u8, view: &PageView, out: &mut Out) {
        match self.mode {
            PanelMode::Page => {
                if self.is_held(Role::StepMode) {
                    // Hold Step Mode and press a key [p013]. Works in every EDIT state (Q34).
                    self.mode = PanelMode::Step;
                    self.zoom = Some(Zoom { track: row, step });
                    out.commands.push(Command::SetMode { mode: Mode::Step });
                    return;
                }
                match self.edit {
                    EditState::Normal => {
                        let s = view.step(row as usize, step as usize);
                        // One press un-skips a skipped step and leaves it as it was [p014].
                        let (attr, value) = if s.skip { (StepAttr::Skip, 0) } else { (StepAttr::Active, i32::from(!s.on)) };
                        out.commands.push(Command::SetStep { track: row, step, attr, value });
                    }
                    EditState::Preview => out.intents.push(Intent::Audition { track: row, step }),
                    EditState::Perform => {}
                }
            }
            PanelMode::Step => {
                // A row 0 key selects another step of the same track [p013]. Rows 1 to 9 hold the
                // attribute values, not built yet.
                if row == 0 {
                    if let Some(z) = &mut self.zoom {
                        z.step = step;
                    }
                }
            }
        }
    }

    fn leave_zoom(&mut self, out: &mut Out) {
        if self.mode == PanelMode::Step {
            self.mode = PanelMode::Page;
            self.zoom = None;
            out.commands.push(Command::SetMode { mode: Mode::Page });
        }
    }

    fn role_down(&mut self, role: Role, view: &PageView, out: &mut Out) {
        match role {
            // Held for the zoom gesture. A bare click does nothing that the manual describes (Q31).
            Role::StepMode => {}
            // ESC, the PAGE button [p015] and ZOM [p014] each leave Step zoom. In Page view ESC has
            // nothing to cancel yet (selections are not built; p018).
            Role::Esc | Role::PageMode | Role::Zom => self.leave_zoom(out),
            Role::Tgl => {
                if let (PanelMode::Step, Some(z)) = (self.mode, self.zoom) {
                    let on = view.step(z.track as usize, z.step as usize).on;
                    out.commands.push(Command::SetStep { track: z.track, step: z.step, attr: StepAttr::Active, value: i32::from(!on) });
                }
            }
            Role::Mute => {
                if let (PanelMode::Step, Some(z)) = (self.mode, self.zoom) {
                    let skip = view.step(z.track as usize, z.step as usize).skip;
                    out.commands.push(Command::SetStep { track: z.track, step: z.step, attr: StepAttr::Skip, value: i32::from(!skip) });
                }
            }
            Role::Edit => self.edit = self.edit.next(),
            Role::Play => {
                if self.play_mode {
                    self.play_mode = false;
                    out.intents.push(Intent::SnapshotRecall);
                } else {
                    self.play_mode = true;
                    out.intents.push(Intent::SnapshotTake);
                }
            }
            Role::Program => {
                if self.play_mode {
                    self.play_mode = false;
                    out.intents.push(Intent::SnapshotKeep);
                }
            }
            Role::Stop => {
                out.commands.push(Command::Stop);
                if self.play_mode {
                    // Stopping the sequencer loses the changes made in PLAY mode [p066].
                    self.play_mode = false;
                    out.intents.push(Intent::SnapshotRecall);
                }
            }
        }
    }

    /// The LED frame for the controller's state and the page `view`.
    pub fn leds(&self, view: &PageView) -> LedFrame {
        let mut f = LedFrame::default();
        match (self.mode, self.zoom) {
            (PanelMode::Step, Some(z)) => {
                // Row 0 is the zoomed track's 16 steps [p013]. The unselected ones show their Page
                // view colours, which the manual does not say (Q28).
                for step in 0..16u8 {
                    let s = view.step(z.track as usize, step as usize);
                    f.set(self.layout.matrix(0, step as usize), if step == z.step { zoom_leds(s).0 } else { page_led(s) });
                }
                f.set(self.layout.role(Role::Mute), zoom_leds(view.step(z.track as usize, z.step as usize)).1);
                f.set(self.layout.role(Role::Zom), Led::steady(Colour::Red)); // p014
            }
            _ => {
                for row in 0..10 {
                    for step in 0..16 {
                        f.set(self.layout.matrix(row, step), page_led(view.step(row, step)));
                    }
                }
            }
        }
        f.set(
            self.layout.role(Role::Edit),
            match self.edit {
                EditState::Normal => Led::steady(Colour::Green),
                EditState::Preview => Led::flash(Colour::Orange),
                EditState::Perform => Led::flash(Colour::Green),
            },
        );
        if self.play_mode {
            f.set(self.layout.role(Role::Play), Led::flash(Colour::Orange));
            f.set(self.layout.role(Role::Program), Led::steady(Colour::Red));
        } else {
            f.set(self.layout.role(Role::Play), Led::steady(Colour::Green));
        }
        f
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn panel() -> (Panel, Layout) {
        let mut pairs: Vec<(u32, String)> = Vec::new();
        let mut n = 0;
        for r in 0..10 {
            for c in 1..=16 {
                pairs.push((n, format!("matrix.r{r}.c{c}")));
                n += 1;
            }
        }
        for role in Role::ALL {
            pairs.push((n, role.id().to_string()));
            n += 1;
        }
        let layout = Layout::from_pairs(pairs.iter().map(|(n, id)| (*n, id.as_str()))).unwrap();
        (Panel::new(layout.clone()), layout)
    }

    #[test]
    fn a_key_pressed_twice_without_a_release_is_one_press() {
        let (mut p, l) = panel();
        let v = PageView::default();
        let k = l.matrix(3, 4);
        assert_eq!(p.input(0, Input::Down(k), &v).commands.len(), 1);
        assert_eq!(p.input(5, Input::Down(k), &v), Out::default());
        p.input(10, Input::Up(k), &v);
        assert_eq!(p.input(20, Input::Down(k), &v).commands.len(), 1);
    }

    #[test]
    fn encoder_turns_and_unknown_controls_do_nothing_yet() {
        let (mut p, l) = panel();
        let v = PageView::default();
        assert_eq!(p.input(0, Input::Turn { control: l.matrix(0, 0), detents: 3 }, &v), Out::default());
        assert_eq!(p.input(0, Input::Down(ControlId(511)), &v), Out::default());
        assert_eq!(p.input(0, Input::Down(ControlId(u32::MAX)), &v), Out::default());
        assert_eq!(p.input(0, Input::Up(ControlId(u32::MAX)), &v), Out::default());
    }

    #[test]
    fn releasing_step_mode_before_the_key_means_no_zoom() {
        let (mut p, l) = panel();
        let v = PageView::default();
        p.input(0, Input::Down(l.role(Role::StepMode)), &v);
        p.input(10, Input::Up(l.role(Role::StepMode)), &v);
        let out = p.input(20, Input::Down(l.matrix(3, 4)), &v);
        assert_eq!(p.mode(), PanelMode::Page);
        assert!(matches!(out.commands.as_slice(), [Command::SetStep { .. }]));
    }
}
