//! `octoface`: the panel controller (SPEC-0002 `tech.md` section 4, ADR-0007). Owner: Panelwright.
//!
//! A pure state machine. In: button down and up, encoder turns, and time as a millisecond count
//! the caller passes in, so a fixture can replay a session exactly. Out: engine commands, intents
//! for engine capabilities that do not exist yet, and an LED frame. Every behaviour is cited to a
//! manual page and covered by a panel fixture in `tests/conformance/panel/`.
//!
//! No runtime dependencies beyond `octocore`'s types. Read `contracts/controls.json` in the
//! caller and pass `(n, id)` pairs to [`Layout::from_pairs`].

#![forbid(unsafe_code)]

pub mod layout;
pub mod led;
pub mod panel;
pub mod view;

pub use layout::{Key, Layout, LayoutError, Role};
pub use led::{Colour, Led, LedFrame, Phase};
pub use panel::{Input, Intent, Out, Panel, PanelMode};
pub use view::{PageView, StepView};
