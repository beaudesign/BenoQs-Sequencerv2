//! The LED language: `{ Off | Red | Green | Orange } x { Steady | Flash | Shine }`.
//!
//! Red, Green and Orange are roles, not hues: the manual's alternate colourways [p004] are a table
//! the web app applies. `Shine` is the manual's own word for a third state, used as `Shine_Red` in
//! the Step zoom table [p014] and never defined (Q03). The controller says which state an LED is
//! in; how `Shine` looks against `Steady` is the Forge's to draw, and the owner's to rule.

use octocore::types::{ControlId, MAX_CONTROLS};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Colour {
    #[default]
    Off,
    Red,
    Green,
    Orange,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Phase {
    #[default]
    Steady,
    Flash,
    Shine,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Led {
    pub colour: Colour,
    pub phase: Phase,
}

impl Led {
    pub const OFF: Led = Led { colour: Colour::Off, phase: Phase::Steady };
    pub const fn steady(colour: Colour) -> Led {
        Led { colour, phase: Phase::Steady }
    }
    pub const fn flash(colour: Colour) -> Led {
        Led { colour, phase: Phase::Flash }
    }
    pub const fn shine(colour: Colour) -> Led {
        Led { colour, phase: Phase::Shine }
    }
}

/// One LED per control, indexed by `ControlId` (the `n` in `contracts/controls.json`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LedFrame {
    leds: [Led; MAX_CONTROLS],
}

impl Default for LedFrame {
    fn default() -> Self {
        LedFrame { leds: [Led::OFF; MAX_CONTROLS] }
    }
}

impl LedFrame {
    pub fn get(&self, id: ControlId) -> Led {
        self.leds.get(id.0 as usize).copied().unwrap_or(Led::OFF)
    }

    pub(crate) fn set(&mut self, id: ControlId, led: Led) {
        if let Some(slot) = self.leds.get_mut(id.0 as usize) {
            *slot = led;
        }
    }

    pub fn as_slice(&self) -> &[Led] {
        &self.leds
    }
}
