//! Ref: docs/06-shell-and-rooms.md §4, pipeline step 1 ("structured
//! extraction"): "turn prose into a typed struct... dimensions, primary
//! materials, light character, scale." That step needs a language model; none
//! is available in this environment (no network access), so this module does
//! NOT attempt real prose-to-struct extraction. `RoomIntent` is the type step 1
//! WOULD produce — hand-author instances until step 1 exists for real.
//! `RoomIntent::stairwell_3am` is docs/06 §3's own worked example, transcribed
//! as a typed value instead of guessed at.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaterialHint {
    ConcreteBare,
    WoodDry,
    CarpetedStudio,
    Glass,
    Cardboard,
    Stone,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LightCharacter {
    Warm,
    Neutral,
    Cool,
    Sodium,
    Dim,
    Bright,
}

/// Ref docs/06 §4: "scale" as one of the four fields structured extraction
/// produces. Coarse buckets rather than exact dimensions, since a prompt like
/// "a narrow concrete stairwell" gives a scale impression before it gives
/// numbers — `dimensions_m` on `RoomIntent` carries the numbers when the prompt
/// (or a human editing the struct directly, per §4: "a user can bypass the
/// prompt entirely and edit the struct") gives them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoomScale {
    Intimate,
    Small,
    Medium,
    Large,
    Cavernous,
}

#[derive(Clone, Debug)]
pub struct RoomIntent {
    pub id: String,
    pub name: String,
    pub prompt_space: String,
    pub prompt_session: String,
    /// `[length, height, depth]` in metres, matching the room contract's
    /// `bounds_m` ordering (docs/06 §3). `None` per axis means "let geometry
    /// synthesis pick something from `scale`", not zero.
    pub dimensions_m: [Option<f32>; 3],
    pub primary_material: MaterialHint,
    pub light_character: LightCharacter,
    pub scale: RoomScale,
}

impl RoomIntent {
    /// Ref docs/06-shell-and-rooms.md §3's own room contract example, verbatim:
    /// `bounds_m: [2.1, 11.4, 3.0]`, prompt text, concrete_bare material.
    pub fn stairwell_3am() -> Self {
        RoomIntent {
            id: "stairwell-3am".to_string(),
            name: "Stairwell, 3am".to_string(),
            prompt_space: "A narrow concrete stairwell in an empty building. Bare walls, a steel handrail, one window high up with sodium light coming through.".to_string(),
            prompt_session: "Standing at the bottom, instrument on a flight case.".to_string(),
            dimensions_m: [Some(2.1), Some(11.4), Some(3.0)],
            primary_material: MaterialHint::ConcreteBare,
            light_character: LightCharacter::Sodium,
            scale: RoomScale::Large, // tall and narrow, per the prompt
        }
    }
}
