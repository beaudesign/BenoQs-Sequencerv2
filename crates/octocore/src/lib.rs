//! `octocore` — the WENGE / Octopus v2 sequencer core (D1, SPEC.md §3).
//! Owner: Metronome. Behavioural source of truth: the CE OS v5.30 reference
//! manual at `reference/manual/`. Ambiguities and deliberate deferrals live in
//! `tests/conformance/AMBIGUITIES.md`.

// `docs/02` section 3 keeps `unsafe` for `octoffi` and the renderer. The command ring and the
// snapshot buffer are written with atomic words to honour that (SPEC-0001 O6, D1); this
// makes the compiler enforce it instead of a reader.
#![forbid(unsafe_code)]

pub mod attrs;
pub mod domain;
pub mod engine;
pub mod fixture;
pub mod link;
pub mod phrases;
pub mod ring;
pub mod rng;
pub mod scale;
pub mod snapshot;
pub mod steps;
pub mod sync;
pub mod tables;
pub mod triple;
pub mod types;
pub mod wire;

pub use domain::Grid;
pub use engine::{Diagnostics, Engine, EventBuffer, RenderContext, QUEUE_CAP};
pub use link::{CommandSender, SnapshotReader};
pub use types::{Command, Event, Realtime, RealtimeEvent};
