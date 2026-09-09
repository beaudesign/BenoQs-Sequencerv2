//! `octocore` — the WENGE / Octopus v2 sequencer core (D1, SPEC.md §3).
//! Owner: Metronome. Behavioural source of truth: the CE OS v5.30 reference
//! manual at `reference/manual/`. Ambiguities and deliberate deferrals live in
//! `tests/conformance/AMBIGUITIES.md`.

pub mod domain;
pub mod engine;
pub mod fixture;
pub mod rng;
pub mod scale;
pub mod tables;
pub mod types;

pub use domain::Grid;
pub use engine::{Engine, EventBuffer, RenderContext};
pub use types::{Command, Event};
