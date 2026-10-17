//! The `Curve` chord style (SPEC-0005 r1, WENGE-0033): where in time the notes of a roll start.
//!
//! The Octopus rolls a chord with one fixed table: at strum level `L` the note `k` of `n` starts
//! `strum_offset_ticks(L, k)` ticks late (CE v5.30 p.22). `Curve` keeps the roll's span, the
//! table's own end point `strum_offset_ticks(L, n)`, and spaces the notes along a bent curve
//! inside it: an accelerating roll (a bouncing ball), an even one, or one that drags.
//!
//! The engine reads integers and never computes a curve: the starts are the pinned table in
//! `curve_table.rs`, made from `W(x) = (e^(a x) - 1) / (e^a - 1)` with `a = 4 * warp` and rounded
//! half up (`tests/curve.rs` regenerates it and fails if it differs). So a native build and the
//! wasm build cannot differ by a rounding. The style draws no random number and never starts a
//! note earlier than the table's end point allows, so the engine's lookahead is as it was.

use crate::curve_table::STARTS;
use crate::tables;

/// The five stored warps. A negative warp puts the notes close together late (the roll
/// accelerates, like a bouncing ball); a positive warp puts them close together early (it
/// drags). The names are the strip's presets (`Accel`, `Bounce`, `Rit`); `Even` and `Drag` are
/// stored and tested and are not on the button.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Warp {
    /// -1.0
    Bounce,
    /// -0.5
    Accel,
    /// 0.0: the notes are evenly spaced
    Even,
    /// 0.5
    Rit,
    /// 1.0
    Drag,
}

impl Warp {
    /// In table order, from -1 to 1.
    pub const ALL: [Warp; 5] = [Warp::Bounce, Warp::Accel, Warp::Even, Warp::Rit, Warp::Drag];

    /// The warp in tenths: -10, -5, 0, 5 or 10.
    pub const fn tenths(self) -> i8 {
        match self {
            Warp::Bounce => -10,
            Warp::Accel => -5,
            Warp::Even => 0,
            Warp::Rit => 5,
            Warp::Drag => 10,
        }
    }

    /// `None` for anything that is not one of the five stored warps.
    pub const fn from_tenths(tenths: i32) -> Option<Warp> {
        match tenths {
            -10 => Some(Warp::Bounce),
            -5 => Some(Warp::Accel),
            0 => Some(Warp::Even),
            5 => Some(Warp::Rit),
            10 => Some(Warp::Drag),
            _ => None,
        }
    }

    const fn index(self) -> usize {
        match self {
            Warp::Bounce => 0,
            Warp::Accel => 1,
            Warp::Even => 2,
            Warp::Rit => 3,
            Warp::Drag => 4,
        }
    }
}

/// How a track spaces the notes of a roll (a strummed chord, or a strummed single note).
/// `Manual` is the Octopus's own table and the default: with it the engine plays what it played
/// before this style existed, byte for byte (CV-1).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChordStyle {
    #[default]
    Manual,
    Curve(Warp),
}

impl ChordStyle {
    /// The style as a page sends it: `style` 0 is `Manual` (and `warp_tenths` is ignored), 1 is
    /// `Curve` with `warp_tenths` one of -10, -5, 0, 5, 10. Anything else is `None`.
    pub const fn from_wire(style: i32, warp_tenths: i32) -> Option<ChordStyle> {
        match style {
            0 => Some(ChordStyle::Manual),
            1 => match Warp::from_tenths(warp_tenths) {
                Some(w) => Some(ChordStyle::Curve(w)),
                None => None,
            },
            _ => None,
        }
    }

    /// The inverse of `from_wire`: `(style, warp_tenths)`, with a warp of 0 for `Manual`.
    pub const fn to_wire(self) -> (u8, i8) {
        match self {
            ChordStyle::Manual => (0, 0),
            ChordStyle::Curve(w) => (1, w.tenths()),
        }
    }
}

/// Ticks after the first note at which note `k` (1 is the first) of a roll of `n` notes starts,
/// at strum `level` (0 to 9, the magnitude; the direction reverses the notes before this is
/// asked). A strummed single note is a roll of seven (`n` = 7).
///
/// `Manual` is `tables::strum_offset_ticks(level, k)`; it does not depend on `n`, as the manual's
/// table does not.
pub fn roll_start_ticks(style: ChordStyle, level: u8, k: u8, n: u8) -> u32 {
    match style {
        ChordStyle::Manual => tables::strum_offset_ticks(level, k),
        ChordStyle::Curve(warp) => {
            // No roll: a single note without a strum, level 0, or the first note, which is never late.
            if level == 0 || k <= 1 || n < 2 {
                return 0;
            }
            let level = level.min(9);
            let n = n.min(7);
            let k = k.min(n);
            STARTS[warp.index()][(level - 1) as usize][(n - 2) as usize][(k - 1) as usize] as u32
        }
    }
}
