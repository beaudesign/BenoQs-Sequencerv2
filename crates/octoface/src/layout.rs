//! The controller's view of `contracts/controls.json`: which control number is which role.
//!
//! The contract is JSON and the controller takes no runtime dependencies, so the caller reads the
//! file (the tests use `serde_json`; the web app's JavaScript can) and hands over `(n, id)` pairs.
//! The controller reads the role from the `id`. Controls it does not use are ignored. A control it
//! does use that the file lacks is an error at construction, never a panic later (ADR-0007
//! decision 5).

use octocore::domain::{STEP_COUNT, TRACK_COUNT};
use octocore::types::{ControlId, MAX_CONTROLS};

/// A named control the controller acts on. The matrix is addressed by row and step instead.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    /// PAGE mode button [p015].
    PageMode,
    /// Step Mode button [p013].
    StepMode,
    /// PLAY button of the MODE block [p066].
    Play,
    /// EDIT LED button [p068].
    Edit,
    Esc,
    Program,
    /// TGL mutator [p014].
    Tgl,
    /// ZOM mutator [p014].
    Zom,
    /// Main Mute button [p014].
    Mute,
    /// Transport Stop button [p049].
    Stop,
    /// VEL edit knob [p015].
    VelKnob,
    /// PIT edit knob [p015].
    PitKnob,
    /// LEN edit knob [p015].
    LenKnob,
    /// STA edit knob [p016].
    StaKnob,
}

impl Role {
    pub const ALL: [Role; 14] = [
        Role::PageMode,
        Role::StepMode,
        Role::Play,
        Role::Edit,
        Role::Esc,
        Role::Program,
        Role::Tgl,
        Role::Zom,
        Role::Mute,
        Role::Stop,
        Role::VelKnob,
        Role::PitKnob,
        Role::LenKnob,
        Role::StaKnob,
    ];

    /// The `id` of this control in `contracts/controls.json`.
    pub const fn id(self) -> &'static str {
        match self {
            Role::PageMode => "mode.page",
            Role::StepMode => "mode.step",
            Role::Play => "mode.play",
            Role::Edit => "mode.edit",
            Role::Esc => "keys.esc",
            Role::Program => "keys.program",
            Role::Tgl => "mutator.tgl",
            Role::Zom => "mutator.zom",
            Role::Mute => "mutator.mute",
            Role::Stop => "transport.stop",
            Role::VelKnob => "edit.enc.vel",
            Role::PitKnob => "edit.enc.pit",
            Role::LenKnob => "edit.enc.len",
            Role::StaKnob => "edit.enc.sta",
        }
    }

    const fn index(self) -> usize {
        self as usize
    }
}

/// What a control number means to the controller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    /// A matrix key: `row` 0 to 9 (row 0 at the bottom, track `row` in Page view), `step` 0 to 15
    /// (the manual's steps 1 to 16).
    Matrix { row: u8, step: u8 },
    Role(Role),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LayoutError {
    NumberOutOfRange { id: String, n: u32 },
    DuplicateNumber(u32),
    DuplicateId(String),
    /// The controller needs this control and the inventory does not list it.
    Missing(String),
}

impl std::fmt::Display for LayoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LayoutError::NumberOutOfRange { id, n } => {
                write!(f, "control `{id}` has n = {n}, which is not below MAX_CONTROLS ({MAX_CONTROLS})")
            }
            LayoutError::DuplicateNumber(n) => write!(f, "control number {n} is used twice"),
            LayoutError::DuplicateId(id) => write!(f, "control id `{id}` is used twice"),
            LayoutError::Missing(id) => write!(f, "the controller needs control `{id}` and the inventory has none"),
        }
    }
}

impl std::error::Error for LayoutError {}

#[derive(Clone, Debug)]
pub struct Layout {
    matrix: [[ControlId; STEP_COUNT]; TRACK_COUNT],
    roles: [ControlId; Role::ALL.len()],
    by_n: [Option<Key>; MAX_CONTROLS],
}

fn parse_matrix(id: &str) -> Option<(u8, u8)> {
    let mut parts = id.split('.');
    if parts.next()? != "matrix" {
        return None;
    }
    let row: u8 = parts.next()?.strip_prefix('r')?.parse().ok()?;
    let col: u8 = parts.next()?.strip_prefix('c')?.parse().ok()?;
    if parts.next().is_some() || row as usize >= TRACK_COUNT || col == 0 || col as usize > STEP_COUNT {
        return None;
    }
    Some((row, col - 1))
}

impl Layout {
    /// Builds the layout from `(n, id)` pairs, one per control in the inventory.
    pub fn from_pairs<'a>(pairs: impl IntoIterator<Item = (u32, &'a str)>) -> Result<Layout, LayoutError> {
        let mut matrix: [[Option<ControlId>; STEP_COUNT]; TRACK_COUNT] = [[None; STEP_COUNT]; TRACK_COUNT];
        let mut roles: [Option<ControlId>; Role::ALL.len()] = [None; Role::ALL.len()];
        let mut by_n: [Option<Key>; MAX_CONTROLS] = [None; MAX_CONTROLS];
        let mut seen_n = std::collections::BTreeSet::new();
        let mut seen_id = std::collections::BTreeSet::new();

        for (n, id) in pairs {
            if n as usize >= MAX_CONTROLS {
                return Err(LayoutError::NumberOutOfRange { id: id.to_string(), n });
            }
            if !seen_n.insert(n) {
                return Err(LayoutError::DuplicateNumber(n));
            }
            if !seen_id.insert(id.to_string()) {
                return Err(LayoutError::DuplicateId(id.to_string()));
            }
            if let Some((row, step)) = parse_matrix(id) {
                matrix[row as usize][step as usize] = Some(ControlId(n));
                by_n[n as usize] = Some(Key::Matrix { row, step });
            } else if let Some(role) = Role::ALL.iter().copied().find(|r| r.id() == id) {
                roles[role.index()] = Some(ControlId(n));
                by_n[n as usize] = Some(Key::Role(role));
            }
        }

        let mut full = [[ControlId(0); STEP_COUNT]; TRACK_COUNT];
        for (r, row) in matrix.iter().enumerate() {
            for (s, cell) in row.iter().enumerate() {
                full[r][s] = cell.ok_or_else(|| LayoutError::Missing(format!("matrix.r{r}.c{}", s + 1)))?;
            }
        }
        let mut role_ids = [ControlId(0); Role::ALL.len()];
        for role in Role::ALL {
            role_ids[role.index()] = roles[role.index()].ok_or_else(|| LayoutError::Missing(role.id().to_string()))?;
        }
        Ok(Layout { matrix: full, roles: role_ids, by_n })
    }

    /// What the controller does with this control, if anything.
    pub fn key(&self, id: ControlId) -> Option<Key> {
        self.by_n.get(id.0 as usize).copied().flatten()
    }

    /// The control at matrix `row` (0 to 9, row 0 at the bottom) and `step` (0 to 15).
    ///
    /// Panics if either is out of range. Both ranges are constants of the layout
    /// (`TRACK_COUNT`, `STEP_COUNT`), so a caller that loops over them cannot hit it; an address
    /// that arrives from outside goes through `key`, which returns `None` instead.
    pub fn matrix(&self, row: usize, step: usize) -> ControlId {
        self.matrix[row][step]
    }

    pub fn role(&self, role: Role) -> ControlId {
        self.roles[role.index()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn full() -> Vec<(u32, String)> {
        let mut v = Vec::new();
        let mut n = 0;
        for r in 0..10 {
            for c in 1..=16 {
                v.push((n, format!("matrix.r{r}.c{c}")));
                n += 1;
            }
        }
        for role in Role::ALL {
            v.push((n, role.id().to_string()));
            n += 1;
        }
        v
    }

    fn build(v: &[(u32, String)]) -> Result<Layout, LayoutError> {
        Layout::from_pairs(v.iter().map(|(n, id)| (*n, id.as_str())))
    }

    #[test]
    fn a_complete_inventory_builds_and_maps_both_ways() {
        let l = build(&full()).unwrap();
        assert_eq!(l.key(l.matrix(3, 4)), Some(Key::Matrix { row: 3, step: 4 }));
        assert_eq!(l.key(l.role(Role::Esc)), Some(Key::Role(Role::Esc)));
        assert_eq!(l.key(ControlId(511)), None);
        assert_eq!(l.key(ControlId(99_999)), None);
    }

    #[test]
    fn unused_controls_are_ignored() {
        let mut v = full();
        v.push((400, "chord.b1".to_string()));
        let l = build(&v).unwrap();
        assert_eq!(l.key(ControlId(400)), None);
    }

    #[test]
    fn a_missing_matrix_key_or_role_is_an_error_not_a_panic() {
        let mut v = full();
        v.retain(|(_, id)| id != "matrix.r9.c16");
        assert_eq!(build(&v).unwrap_err(), LayoutError::Missing("matrix.r9.c16".to_string()));
        let mut v = full();
        v.retain(|(_, id)| id != "keys.esc");
        assert_eq!(build(&v).unwrap_err(), LayoutError::Missing("keys.esc".to_string()));
    }

    #[test]
    fn a_number_or_an_id_used_twice_is_an_error() {
        let mut v = full();
        v.push((0, "chord.b1".to_string()));
        assert_eq!(build(&v).unwrap_err(), LayoutError::DuplicateNumber(0));
        let mut v = full();
        v.push((400, "keys.esc".to_string()));
        assert_eq!(build(&v).unwrap_err(), LayoutError::DuplicateId("keys.esc".to_string()));
    }

    #[test]
    fn a_number_at_or_above_max_controls_is_an_error() {
        let mut v = full();
        v.push((MAX_CONTROLS as u32, "chord.b1".to_string()));
        assert!(matches!(build(&v), Err(LayoutError::NumberOutOfRange { .. })));
    }

    #[test]
    fn matrix_ids_outside_the_grid_are_not_matrix_keys() {
        for id in ["matrix.r10.c1", "matrix.r0.c0", "matrix.r0.c17", "matrix.r0", "matrix.r0.c1.x", "matrix.x.c1"] {
            assert_eq!(parse_matrix(id), None, "{id}");
        }
        assert_eq!(parse_matrix("matrix.r9.c16"), Some((9, 15)));
    }
}
