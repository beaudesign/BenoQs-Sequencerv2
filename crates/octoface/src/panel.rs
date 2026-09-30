//! The panel controller. STUB: red first (SPEC-0002 S5). Every input does nothing and every LED
//! is off, so the panel fixtures fail. The behaviour arrives in the next commit.

use crate::layout::Layout;
use crate::led::LedFrame;
use crate::view::PageView;
use octocore::types::{Command, ControlId};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PanelMode {
    Page,
    Step,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Input {
    Down(ControlId),
    Up(ControlId),
    Turn { control: ControlId, detents: i16 },
}

/// A request for an engine capability that does not exist yet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Intent {
    Audition { track: u8, step: u8 },
    SnapshotTake,
    SnapshotKeep,
    SnapshotRecall,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Out {
    pub commands: Vec<Command>,
    pub intents: Vec<Intent>,
}

pub struct Panel {
    _layout: Layout,
}

impl Panel {
    pub fn new(layout: Layout) -> Panel {
        Panel { _layout: layout }
    }

    pub fn mode(&self) -> PanelMode {
        PanelMode::Page
    }

    pub fn input(&mut self, _now_ms: u64, _input: Input, _view: &PageView) -> Out {
        Out::default()
    }

    pub fn leds(&self, _view: &PageView) -> LedFrame {
        LedFrame::default()
    }
}
