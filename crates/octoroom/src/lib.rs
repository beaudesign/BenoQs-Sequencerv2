//! `octoroom` — the WENGE room system (D3, `docs/06-shell-and-rooms.md`).
//! Owner: Sceneshaper. Pure-Rust parts only in this crate as it stands: no
//! Metal compute exists here yet (this dev environment has no Xcode/Metal
//! toolchain) — see `README.md` for exactly what's implemented versus
//! deliberately left out.

pub mod acoustics;
pub mod geometry;
pub mod intent;
pub mod rng;

pub use geometry::{synthesize_geometry, Bands, RoomGeometry, Surface};
pub use intent::{LightCharacter, MaterialHint, RoomIntent, RoomScale};
