//! What the controller reads from the engine: the active page's steps, reduced to what the panel
//! draws. `Snapshot` carries no step data and is a frozen C layout, so the caller builds this
//! from `octocore::Grid` (`PageView::from_page`). The engine's own read model is a request to the
//! Metronome (`journal/metronome/requests/`).

use octocore::domain::{Page, STEP_COUNT, TRACK_COUNT};

/// One step as the panel shows it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct StepView {
    pub on: bool,
    pub skip: bool,
    /// The step carries a chord (p020).
    pub chord: bool,
    /// The step carries a step event (p034). The Step zoom table treats "Chord/Event" as one
    /// status [p014], except its footnote, which names events only.
    pub event: bool,
    pub hyperstep: bool,
    /// The step's velocity offset, from the engine's `StepAttr::VelocityOffset` (p015).
    pub velocity_offset: i32,
    /// The step's pitch offset, `StepAttr::PitchOffset` (p015).
    pub pitch_offset: i32,
    /// The step's length in ticks, `StepAttr::LengthTicks` (p015: the manual's 1/192).
    pub length_ticks: i32,
    /// The step's start offset, `StepAttr::StartOffset` (p016).
    pub start_offset: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageView {
    /// `steps[track][step]`, track 0 to 9 and step 0 to 15 (the manual's steps 1 to 16).
    pub steps: [[StepView; STEP_COUNT]; TRACK_COUNT],
}

impl Default for PageView {
    fn default() -> Self {
        PageView { steps: [[StepView::default(); STEP_COUNT]; TRACK_COUNT] }
    }
}

impl PageView {
    pub fn from_page(page: &Page) -> PageView {
        let mut v = PageView::default();
        for (t, track) in page.tracks.iter().enumerate() {
            for (s, step) in track.steps.iter().enumerate() {
                v.steps[t][s] = StepView {
                    on: step.active,
                    skip: step.skip,
                    chord: step.chord.count > 0,
                    event: step.event.is_some(),
                    hyperstep: step.hyperstep,
                    velocity_offset: i32::from(step.velocity_offset),
                    pitch_offset: i32::from(step.pitch_offset),
                    length_ticks: i32::from(step.length_ticks),
                    start_offset: i32::from(step.start_offset),
                };
            }
        }
        v
    }

    pub fn step(&self, track: usize, step: usize) -> StepView {
        self.steps.get(track).and_then(|t| t.get(step)).copied().unwrap_or_default()
    }
}
