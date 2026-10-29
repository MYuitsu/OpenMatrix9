// SPDX-License-Identifier: LGPL-2.1-or-later
//! Solid menu binding catalogue (`OM9-L2-03-18`). One row per documented Solid Tools
//! command. A row is enabled only when its owning kernel service and command adapter are
//! both present; otherwise the menu keeps the authored icon and shows the exact reason.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingState {
    Implemented,
    SessionMissing,
    OwnerMissing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SolidMenuBinding {
    pub l3_id: &'static str,
    pub label: &'static str,
    pub source_command: &'static str,
    pub icon: &'static str,
    pub feature: &'static str,
    pub owner: &'static str,
    pub state: BindingState,
    pub reason: &'static str,
}

impl SolidMenuBinding {
    pub fn enabled(&self) -> bool {
        matches!(self.state, BindingState::Implemented)
    }
}

#[allow(clippy::too_many_arguments)]
const fn row(
    l3_id: &'static str,
    label: &'static str,
    source_command: &'static str,
    icon: &'static str,
    feature: &'static str,
    owner: &'static str,
    state: BindingState,
    reason: &'static str,
) -> SolidMenuBinding {
    SolidMenuBinding {
        l3_id,
        label,
        source_command,
        icon,
        feature,
        owner,
        state,
        reason,
    }
}

use BindingState::Implemented;

pub const BINDINGS: [SolidMenuBinding; 21] = [
    row(
        "OM9-L3-03-18-001",
        "Boolean Union",
        "Boolean Union",
        "SolidUnion",
        "OM9-SOLID-003",
        "OM9-L3-01-10-001",
        Implemented,
        "",
    ),
    row(
        "OM9-L3-03-18-002",
        "Boolean Difference",
        "Boolean Difference",
        "SolidDifference",
        "OM9-SOLID-001",
        "OM9-L3-01-10-002",
        Implemented,
        "",
    ),
    row(
        "OM9-L3-03-18-003",
        "Boolean Intersection",
        "Boolean Intersection",
        "SolidIntersection",
        "OM9-SOLID-002",
        "OM9-L3-01-10-003",
        Implemented,
        "",
    ),
    row(
        "OM9-L3-03-18-004",
        "Boolean Two Objects",
        "Boolean Two Objects",
        "SolidBooleanTwoObjects",
        "OM9-SOLID-004",
        "OM9-L3-01-10-001",
        Implemented,
        "",
    ),
    row(
        "OM9-L3-03-18-005",
        "Cap Planar",
        "Cap Planar",
        "SolidCapPlanarHoles",
        "OM9-SOLID-005",
        "OM9-L3-01-10-005",
        Implemented,
        "",
    ),
    row(
        "OM9-L3-03-18-006",
        "Extract Surface",
        "Extract Surface",
        "SolidExtractSurface",
        "OM9-SOLID-006",
        "OM9-L3-01-10-008",
        Implemented,
        "",
    ),
    row(
        "OM9-L3-03-18-007",
        "Fillet Edge",
        "Fillet Edge",
        "SolidFilletEdgeFilletEdge",
        "OM9-SOLID-007",
        "OM9-L3-01-10-006",
        Implemented,
        "",
    ),
    row(
        "OM9-L3-03-18-008",
        "Text",
        "Text",
        "SolidText",
        "OM9-SOLID-008",
        "OM9-L3-01-09-009",
        Implemented,
        "",
    ),
    row(
        "OM9-L3-03-18-009",
        "Extrude Curve Straight",
        "Extrude Curve Straight",
        "SolidExtrudePlanarCurveStraight",
        "OM9-SOLID-009",
        "OM9-L3-01-09-002",
        Implemented,
        "",
    ),
    row(
        "OM9-L3-03-18-010",
        "Solid Extrude All",
        "Solid Extrude All",
        "SolidGVDAllExtrude",
        "OM9-SOLID-010",
        "OM9-L3-01-09-002",
        Implemented,
        "",
    ),
    row(
        "OM9-L3-03-18-011",
        "Pipe",
        "Pipe",
        "SolidPipe",
        "OM9-SOLID-011",
        "OM9-L3-01-09-004",
        Implemented,
        "",
    ),
    row(
        "OM9-L3-03-18-012",
        "Box",
        "Box",
        "SolidBoxCornertoCornerHeight",
        "OM9-SOLID-012",
        "OM9-L3-01-09-001",
        Implemented,
        "",
    ),
    row(
        "OM9-L3-03-18-013",
        "Sphere",
        "Sphere",
        "SolidSphereCenterRadius",
        "OM9-SOLID-014",
        "OM9-L3-01-09-001",
        Implemented,
        "",
    ),
    row(
        "OM9-L3-03-18-014",
        "Ellipsoid",
        "Ellipsoid",
        "SolidEllipsoidFromCenter",
        "OM9-SOLID-013",
        "OM9-L3-01-09-001",
        Implemented,
        "",
    ),
    row(
        "OM9-L3-03-18-015",
        "Torus",
        "Torus",
        "SolidTorus",
        "OM9-SOLID-015",
        "OM9-L3-01-09-001",
        Implemented,
        "",
    ),
    row(
        "OM9-L3-03-18-016",
        "Cylinder",
        "Cylinder",
        "SolidCylinder",
        "OM9-SOLID-016",
        "OM9-L3-01-09-001",
        Implemented,
        "",
    ),
    row(
        "OM9-L3-03-18-017",
        "Tube",
        "Tube",
        "SolidTube",
        "OM9-SOLID-017",
        "OM9-L3-01-09-004",
        Implemented,
        "",
    ),
    row(
        "OM9-L3-03-18-018",
        "Pyramid",
        "Pyramid",
        "SolidPyramid",
        "OM9-SOLID-018",
        "OM9-L3-01-09-001",
        Implemented,
        "",
    ),
    row(
        "OM9-L3-03-18-019",
        "Cone",
        "Cone",
        "SolidCone",
        "OM9-SOLID-019",
        "OM9-L3-01-09-001",
        Implemented,
        "",
    ),
    row(
        "OM9-L3-03-18-020",
        "Truncated Cone",
        "Truncated Cone",
        "SolidTruncatedCone",
        "OM9-SOLID-020",
        "OM9-L3-01-09-001",
        Implemented,
        "",
    ),
    row(
        "OM9-L3-03-18-021",
        "Shell",
        "Shell",
        "SolidShell",
        "OM9-SOLID-029",
        "OM9-L3-01-10-007",
        Implemented,
        "",
    ),
];

pub fn by_feature(feature: &str) -> Option<&'static SolidMenuBinding> {
    BINDINGS.iter().find(|binding| binding.feature == feature)
}

pub fn enabled(feature: &str) -> bool {
    by_feature(feature).is_some_and(SolidMenuBinding::enabled)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_solid_command_has_one_binding_and_a_visible_reason() {
        assert_eq!(BINDINGS.len(), 21);
        let mut features = std::collections::BTreeSet::new();
        for binding in &BINDINGS {
            assert!(features.insert(binding.feature));
            assert!(binding.l3_id.starts_with("OM9-L3-03-18-"));
            assert!(!binding.owner.is_empty());
            if binding.enabled() {
                assert!(binding.reason.is_empty());
            } else {
                assert!(!binding.reason.is_empty());
            }
        }
        assert!(by_feature("OM9-SOLID-003").unwrap().enabled());
        assert!(by_feature("OM9-SOLID-008").unwrap().enabled());
        assert!(by_feature("OM9-SOLID-029").unwrap().enabled());
    }

    #[test]
    fn recovered_unimplemented_commands_remain_in_solid_menu() {
        let menu = crate::menu_catalog::SOLID;
        let start = menu.iter().position(|key| *key == "SolidBooleanTwoObjects").unwrap();
        assert_eq!(&menu[start + 1..start + 3], &["SolidBooleanSplit", "SolidCreateSolid"]);
    }
}
