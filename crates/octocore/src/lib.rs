//! `octocore` — the WENGE / Octopus v2 sequencer core (D1, SPEC.md §3).
//! Owner: Metronome. Behavioural source of truth: the CE OS v5.30 reference
//! manual, which this repo does not yet have a copy of (see reference/NOTES.md).
//! Where that manual isn't available to cite, constants and behaviour are ported
//! from the archived v1 implementations instead and flagged PROVISIONAL — see
//! `tests/conformance/AMBIGUITIES.md` and the module-level docs in `tables.rs`,
//! `domain.rs`, and `engine.rs`.

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
