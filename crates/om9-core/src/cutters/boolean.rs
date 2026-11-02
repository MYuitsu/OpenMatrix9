// SPDX-License-Identifier: LGPL-2.1-or-later
//! Interactive Boolean and the cutter body Preserve/Remove policy
//! (`OM9-L3-04-16-006/-007`, manual book 2 pages 272-273 and 290-292).
//!
//! "Interactive Boolean simply means that while running the tools … you can preview the results
//! of the object after it is cut, not the cutter itself. … Upon pressing Enter to end the command
//! the Boolean is complete." The cut preview is therefore the *target* minus the cutters, built
//! on every parameter change; Enter commits it, Cancel throws it away.
//!
//! The boolean itself is the one shared mesh CSG service ([`crate::mesh_csg`]); this module never
//! writes a second kernel. It also owns the manual's `Preserve Cutter Body` answer: after the
//! cut, the cutter bodies are either kept as their own objects or removed.
use crate::{
    Error, Result,
    cutters::volume_mm3,
    geometry::Mesh,
    mesh_csg::{BooleanOp, CsgOptions, mesh_boolean},
};

/// The manual's `Interactive Boolean` option: `Off`, `On` (cut every cutter) or `One` (cut the
/// first cutter only, for a preview that shows a single seat).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BooleanMode {
    Off,
    On,
    One,
}

impl BooleanMode {
    pub const ALL: [Self; 3] = [Self::Off, Self::On, Self::One];

    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::On => "On",
            Self::One => "One",
        }
    }

    pub fn from_tag(tag: &str) -> Result<Self> {
        match tag.trim().to_ascii_lowercase().as_str() {
            "off" | "no" | "false" => Ok(Self::Off),
            "on" | "yes" | "true" | "all" => Ok(Self::On),
            "one" | "single" | "first" => Ok(Self::One),
            _ => Err(Error::Invalid("cutter boolean mode")),
        }
    }

    /// Whether the mode asks for a preview/commit of the cut target.
    pub fn cuts(self) -> bool {
        !matches!(self, Self::Off)
    }

    /// How many cutters the mode applies.
    pub fn cutter_limit(self) -> Option<usize> {
        match self {
            Self::Off => Some(0),
            Self::On => None,
            Self::One => Some(1),
        }
    }
}

/// `Preserve Cutter Body`: keep the cutter objects after the cut, or remove them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CutterKeep {
    Preserve,
    Remove,
}

impl CutterKeep {
    pub const ALL: [Self; 2] = [Self::Preserve, Self::Remove];

    pub fn label(self) -> &'static str {
        match self {
            Self::Preserve => "Preserve",
            Self::Remove => "Remove",
        }
    }

    pub fn from_tag(tag: &str) -> Result<Self> {
        match tag.trim().to_ascii_lowercase().as_str() {
            "preserve" | "keep" | "yes" => Ok(Self::Preserve),
            "remove" | "delete" | "no" => Ok(Self::Remove),
            _ => Err(Error::Invalid("cutter preserve option")),
        }
    }

    pub fn keeps(self) -> bool {
        matches!(self, Self::Preserve)
    }
}

/// The cut result the session commits.
#[derive(Debug, Clone, PartialEq)]
pub struct CutterCut {
    /// The target after the subtraction (closed mesh).
    pub result: Mesh,
    /// Metal removed by the cut, mm³ (target volume minus result volume).
    pub removed_mm3: f64,
    /// How many cutter bodies were applied.
    pub applied: usize,
    /// Whether the cutter bodies stay in the document.
    pub keep_cutters: bool,
    pub summary: String,
}

/// Subtract the cutters from the target.
///
/// The target and every cutter are closed meshes; an open one is refused by the CSG service. The
/// cut commits nothing: it returns the new target mesh, and the caller's transaction decides
/// whether the cutter bodies stay (`CutterKeep`).
pub fn apply(
    target: &Mesh,
    cutters: &[Mesh],
    mode: BooleanMode,
    keep: CutterKeep,
    options: CsgOptions,
) -> Result<CutterCut> {
    if !mode.cuts() {
        return Err(Error::Invalid("cutter boolean mode"));
    }
    if cutters.is_empty() {
        return Err(Error::Missing("cutter bodies"));
    }
    let limit = mode.cutter_limit().unwrap_or(cutters.len()).min(cutters.len());
    if limit == 0 {
        return Err(Error::Invalid("cutter boolean mode"));
    }
    let solid_target = crate::settings::triangulate_mesh(target)?;
    let target_volume = volume_mm3(&solid_target)?;
    let mut current = solid_target;
    let mut applied = 0;
    for cutter in &cutters[..limit] {
        let solid_cutter = crate::settings::triangulate_mesh(cutter)?;
        let (next, report) = mesh_boolean(&current, &solid_cutter, BooleanOp::Difference, options)?;
        if next.faces.is_empty() || report.output_triangles == 0 {
            return Err(Error::Invalid("cutter boolean result"));
        }
        current = next;
        applied += 1;
    }
    let result_volume = volume_mm3(&current)?;
    let removed = (target_volume - result_volume).max(0.0);
    if removed <= 1.0e-9 {
        return Err(Error::Invalid("cutter boolean removed nothing"));
    }
    let summary = format!(
        "Interactive Boolean {}: {} cutter{} applied, {:.3} mm³ removed, cutter bodies {}",
        mode.label(),
        applied,
        if applied == 1 { "" } else { "s" },
        removed,
        if keep.keeps() { "kept" } else { "removed" }
    );
    Ok(CutterCut {
        result: current,
        removed_mm3: removed,
        applied,
        keep_cutters: keep.keeps(),
        summary,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cutters::{EdgeShape, Ring, loft};
    use crate::geometry::Mesh;

    fn cylinder(radius: f64, height: f64) -> Mesh {
        loft(
            &[Ring::round(0.0, radius), Ring::round(height, radius)],
            64,
        )
        .expect("cylinder")
    }

    fn options() -> CsgOptions {
        CsgOptions::default()
    }

    #[test]
    fn the_boolean_removes_the_cutter_volume_from_the_target() {
        let target = crate::settings::triangulate_mesh(&cylinder(2.0, 4.0)).expect("target");
        let cutter = cylinder(0.5, 6.0);
        let cut = apply(&target, &[cutter], BooleanMode::On, CutterKeep::Preserve, options())
            .expect("cut");
        assert_eq!(cut.applied, 1);
        assert!(cut.keep_cutters);
        let expected = std::f64::consts::PI * 0.25 * 4.0;
        assert!((cut.removed_mm3 - expected).abs() < 0.1, "{}", cut.removed_mm3);
        assert!(crate::settings::inspect_solid(&cut.result).expect("closed").closed);
    }

    #[test]
    fn one_mode_applies_only_the_first_cutter() {
        let target = crate::settings::triangulate_mesh(&cylinder(2.0, 4.0)).expect("target");
        let cutters = vec![cylinder(0.4, 6.0), cylinder(0.4, 6.0)];
        let cut = apply(&target, &cutters, BooleanMode::One, CutterKeep::Remove, options())
            .expect("cut");
        assert_eq!(cut.applied, 1);
        assert!(!cut.keep_cutters);
    }

    #[test]
    fn switching_the_boolean_off_is_a_typed_refusal() {
        let target = cylinder(2.0, 4.0);
        let cutter = cylinder(0.5, 6.0);
        assert_eq!(
            apply(&target, &[cutter], BooleanMode::Off, CutterKeep::Preserve, options()).err(),
            Some(Error::Invalid("cutter boolean mode"))
        );
    }

    #[test]
    fn an_empty_cutter_list_is_refused_instead_of_silently_doing_nothing() {
        let target = cylinder(2.0, 4.0);
        assert_eq!(
            apply(&target, &[], BooleanMode::On, CutterKeep::Preserve, options()).err(),
            Some(Error::Missing("cutter bodies"))
        );
    }

    #[test]
    fn a_cutter_that_misses_the_target_is_refused() {
        let target = crate::settings::triangulate_mesh(&cylinder(1.0, 2.0)).expect("target");
        let mut far = cylinder(0.4, 2.0);
        for vertex in &mut far.vertices {
            vertex[0] += 50.0;
        }
        assert!(apply(&target, &[far], BooleanMode::On, CutterKeep::Preserve, options()).is_err());
    }

    #[test]
    fn the_edge_profile_helpers_are_available_to_the_policy() {
        let shape = EdgeShape::default();
        assert!((shape.scale(0.0) - 1.0).abs() < 1.0e-12);
        assert_eq!(BooleanMode::from_tag("one").expect("one"), BooleanMode::One);
        assert!(CutterKeep::from_tag("remove").expect("remove").keeps() == false);
    }
}
