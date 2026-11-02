// SPDX-License-Identifier: LGPL-2.1-or-later
//! Matrix Transform menu binding catalog (`OM9-L2-04-01`).
//!
//! One row per catalogued Transform command. Every row names the authored icon (the
//! catalog and `icons.rs` own the artwork), the Matrix command-line name, the unique
//! command id and the verified state. A row is only enabled when a command body
//! exists in `session/transform.rs`; a row without an owner stays visible and
//! disabled with the recorded reason, so clicking it can never silently do nothing.

/// Why a Transform menu row is not enabled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingState {
    Implemented,
    /// The command body is missing; the reason names the missing owner service.
    SessionMissing,
    /// The milestone tree defers this row to M08.
    DeferredM08,
}

/// One Transform menu row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransformMenuBinding {
    pub l3_id: &'static str,
    pub label: &'static str,
    /// Matrix command-line name, so typing it starts the same command as clicking.
    pub command_line: &'static str,
    /// The authored icon/command key of the row: `MainMenu.ini` `IconN=` and the `icons.rs`
    /// entry both use it, so the catalog, the authored menu and the icon table cannot drift
    /// apart.
    pub command_key: &'static str,
    pub feature: &'static str,
    pub state: BindingState,
    pub disabled_reason: &'static str,
}

impl TransformMenuBinding {
    pub fn enabled(&self) -> bool {
        matches!(self.state, BindingState::Implemented)
    }
}

/// The L3 row that owns menu-wide availability itself (no command of its own).
pub const AVAILABILITY_L3: &str = "OM9-L3-04-01-042";

pub const BINDINGS: &[TransformMenuBinding; 41] = &[
    TransformMenuBinding {
        l3_id: "OM9-L3-04-01-001",
        label: "Scale 3D",
        command_line: "Scale",
        command_key: "TransformScaleScale3D",
        feature: "OM9-TRANSFORM-001",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    TransformMenuBinding {
        l3_id: "OM9-L3-04-01-002",
        label: "Scale2D",
        command_line: "Scale2D",
        command_key: "TransformScaleScale2D",
        feature: "OM9-TRANSFORM-002",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    TransformMenuBinding {
        l3_id: "OM9-L3-04-01-003",
        label: "Scale1D",
        command_line: "Scale1D",
        command_key: "TransformScaleScale1D",
        feature: "OM9-TRANSFORM-004",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    TransformMenuBinding {
        l3_id: "OM9-L3-04-01-004",
        label: "Scale Non-Uniform",
        command_line: "ScaleNU",
        command_key: "TransformScaleNonUniformScale",
        feature: "OM9-TRANSFORM-003",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    TransformMenuBinding {
        l3_id: "OM9-L3-04-01-005",
        label: "Orient 2 Points",
        command_line: "Orient",
        command_key: "TransformOrient2Points",
        feature: "OM9-TRANSFORM-005",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    TransformMenuBinding {
        l3_id: "OM9-L3-04-01-006",
        label: "Orient 3 Points",
        command_line: "Orient3Pt",
        command_key: "TransformOrient3Points",
        feature: "OM9-TRANSFORM-006",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    TransformMenuBinding {
        l3_id: "OM9-L3-04-01-007",
        label: "Rotate3D",
        command_line: "Rotate3D",
        command_key: "TransformRotate3D",
        feature: "OM9-TRANSFORM-008",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    TransformMenuBinding {
        l3_id: "OM9-L3-04-01-008",
        label: "Bend",
        command_line: "Bend",
        command_key: "TransformBend",
        feature: "OM9-TRANSFORM-007",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    TransformMenuBinding {
        l3_id: "OM9-L3-04-01-009",
        label: "Array Polar",
        command_line: "ArrayPolar",
        command_key: "TransformArrayPolar",
        feature: "OM9-TRANSFORM-009",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    TransformMenuBinding {
        l3_id: "OM9-L3-04-01-010",
        label: "Project To C-Plane",
        command_line: "ProjectToCPlane",
        command_key: "TransformProjecttoCPlane",
        feature: "OM9-TRANSFORM-010",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    TransformMenuBinding {
        l3_id: "OM9-L3-04-01-011",
        label: "Smart Flow",
        command_line: "SmartFlow",
        command_key: "gvSmartFlow",
        feature: "OM9-TRANSFORM-011",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    TransformMenuBinding {
        l3_id: "OM9-L3-04-01-012",
        label: "Smart Flow Rigid",
        command_line: "SmartFlowRigid",
        command_key: "gvSmartFlowRigid",
        feature: "OM9-TRANSFORM-012",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    TransformMenuBinding {
        l3_id: "OM9-L3-04-01-013",
        label: "Smart Flow Break",
        command_line: "SmartFlowBreak",
        command_key: "gvSmartFlowBreak",
        feature: "OM9-TRANSFORM-014",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    TransformMenuBinding {
        l3_id: "OM9-L3-04-01-014",
        label: "Smart Flow Pull Down Curves",
        command_line: "SmartFlowPullDownCurves",
        command_key: "gvSmartFlowPullDownCurves",
        feature: "OM9-TRANSFORM-015",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    TransformMenuBinding {
        l3_id: "OM9-L3-04-01-015",
        label: "Orient on Surface",
        command_line: "OrientOnSrf",
        command_key: "TransformOrientOnSurface",
        feature: "OM9-TRANSFORM-013",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    TransformMenuBinding {
        l3_id: "OM9-L3-04-01-016",
        label: "Orient Perpendicular to Curve",
        command_line: "OrientOnCrv",
        command_key: "TransformOrientPerpendiculartoCurve",
        feature: "OM9-TRANSFORM-016",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    TransformMenuBinding {
        l3_id: "OM9-L3-04-01-017",
        label: "Orient on Curve",
        command_line: "OrientOnCrv",
        command_key: "TransformOrientOrientOnCurve",
        feature: "OM9-TRANSFORM-017",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    TransformMenuBinding {
        l3_id: "OM9-L3-04-01-018",
        label: "Symmetry",
        command_line: "Symmetry",
        command_key: "TransformSymmetry",
        feature: "OM9-TRANSFORM-018",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    TransformMenuBinding {
        l3_id: "OM9-L3-04-01-019",
        label: "Solid Points ON",
        command_line: "SolidPtOn",
        command_key: "TransformSolidPtOn",
        feature: "OM9-TRANSFORM-019",
        state: BindingState::SessionMissing,
        disabled_reason: "canonical solid control-point selection/edit service is not implemented; curve/surface grips do not enable solid editing",
    },
    TransformMenuBinding {
        l3_id: "OM9-L3-04-01-020",
        label: "Cage Edit",
        command_line: "CageEdit",
        command_key: "TransformCageEditingCageEdit",
        feature: "OM9-TRANSFORM-020",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    TransformMenuBinding {
        l3_id: "OM9-L3-04-01-021",
        label: "ReleaseFromCage",
        command_line: "ReleaseFromCage",
        command_key: "TransformCageEditingReleaseFromCage",
        feature: "OM9-TRANSFORM-022",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    TransformMenuBinding {
        l3_id: "OM9-L3-04-01-022",
        label: "Array Along Curve",
        command_line: "ArrayCrv",
        command_key: "TransformArrayAlongCurve",
        feature: "OM9-TRANSFORM-021",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    TransformMenuBinding {
        l3_id: "OM9-L3-04-01-023",
        label: "Array Linear",
        command_line: "ArrayLinear",
        command_key: "TransformArrayArrayLinear",
        feature: "OM9-TRANSFORM-024",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    TransformMenuBinding {
        l3_id: "OM9-L3-04-01-024",
        label: "Array Along Surface",
        command_line: "ArraySrf",
        command_key: "TransformArrayAlongSurface",
        feature: "OM9-TRANSFORM-023",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    TransformMenuBinding {
        l3_id: "OM9-L3-04-01-025",
        label: "Array Curve on Surface",
        command_line: "ArrayCrvOnSrf",
        command_key: "TransformArrayAlongCurveonSurface",
        feature: "OM9-TRANSFORM-026",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    TransformMenuBinding {
        l3_id: "OM9-L3-04-01-026",
        label: "Array",
        command_line: "Array",
        command_key: "TransformArrayRectangular",
        feature: "OM9-TRANSFORM-025",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    TransformMenuBinding {
        l3_id: "OM9-L3-04-01-027",
        label: "Flow Along Curve",
        command_line: "Flow",
        command_key: "TransformFlowAlongCurve",
        feature: "OM9-TRANSFORM-027",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    TransformMenuBinding {
        l3_id: "OM9-L3-04-01-028",
        label: "Flow Along Surface",
        command_line: "FlowAlongSrf",
        command_key: "TransformFlowAlongSurface",
        feature: "OM9-TRANSFORM-028",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    TransformMenuBinding {
        l3_id: "OM9-L3-04-01-029",
        label: "Splop",
        command_line: "Splop",
        command_key: "TransformSplop",
        feature: "OM9-TRANSFORM-029",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    TransformMenuBinding {
        l3_id: "OM9-L3-04-01-030",
        label: "Set Points",
        command_line: "SetPt",
        command_key: "TransformSetPoints",
        feature: "OM9-TRANSFORM-030",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    TransformMenuBinding {
        l3_id: "OM9-L3-04-01-031",
        label: "Scale By Plane",
        command_line: "ScaleByPlane",
        command_key: "TransformScaleScaleByPlane",
        feature: "OM9-TRANSFORM-031",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    TransformMenuBinding {
        l3_id: "OM9-L3-04-01-032",
        label: "Stretch",
        command_line: "Stretch",
        command_key: "TransformStretch",
        feature: "OM9-TRANSFORM-032",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    TransformMenuBinding {
        l3_id: "OM9-L3-04-01-033",
        label: "Taper",
        command_line: "Taper",
        command_key: "TransformTaper",
        feature: "OM9-TRANSFORM-033",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    TransformMenuBinding {
        l3_id: "OM9-L3-04-01-034",
        label: "Shear",
        command_line: "Shear",
        command_key: "TransformShear",
        feature: "OM9-TRANSFORM-034",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    TransformMenuBinding {
        l3_id: "OM9-L3-04-01-035",
        label: "Twist",
        command_line: "Twist",
        command_key: "TransformTwist",
        feature: "OM9-TRANSFORM-036",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    TransformMenuBinding {
        l3_id: "OM9-L3-04-01-036",
        label: "Maelstrom",
        command_line: "Maelstrom",
        command_key: "TransformMaelstrom",
        feature: "OM9-TRANSFORM-035",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    TransformMenuBinding {
        l3_id: "OM9-L3-04-01-037",
        label: "Make Down Facing",
        command_line: "MakeDownFacing",
        command_key: "BuilderMakeDownFacing",
        feature: "OM9-TRANSFORM-037",
        state: BindingState::SessionMissing,
        disabled_reason: "no verified owner candidate (UNRESOLVED_BLOCK_ENABLE) for the facing solve",
    },
    TransformMenuBinding {
        l3_id: "OM9-L3-04-01-038",
        label: "Remap to C-Plane",
        command_line: "RemapCPlane",
        command_key: "TransformOrientRemaptoCPlane",
        feature: "OM9-TRANSFORM-038",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    TransformMenuBinding {
        l3_id: "OM9-L3-04-01-039",
        label: "MoveUVN",
        command_line: "MoveUVN",
        command_key: "TransformMoveUVN",
        feature: "OM9-TRANSFORM-039",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    TransformMenuBinding {
        l3_id: "OM9-L3-04-01-040",
        label: "Soft Move",
        command_line: "SoftMove",
        command_key: "TransformSoftMove",
        feature: "OM9-TRANSFORM-040",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    TransformMenuBinding {
        l3_id: "OM9-L3-04-01-041",
        label: "Create Cage",
        command_line: "Cage",
        command_key: "TransformCageEditingCreateCage",
        feature: "OM9-TRANSFORM-041",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
];

/// The binding for one command id, if the catalog names it.
pub fn by_feature(feature: &str) -> Option<&'static TransformMenuBinding> {
    BINDINGS.iter().find(|binding| binding.feature == feature)
}

/// The binding for one Matrix command-line name. Comparison ignores case, spaces,
/// hyphens and underscores, so `scale3d` and `Scale` both reach the authored `! _Scale`
/// row.
pub fn by_command(name: &str) -> Option<&'static TransformMenuBinding> {
    let wanted = normalise_command(name);
    BINDINGS
        .iter()
        .find(|binding| normalise_command(binding.command_line) == wanted)
        .or_else(|| {
            BINDINGS
                .iter()
                .find(|binding| normalise_command(binding.command_key) == wanted)
        })
        .or_else(|| by_label(name))
}

/// The binding for one documented row label (`Scale 3-D`, `Scale Non-Uniform`). The label
/// is what the authored menu prints, so typing it must reach the same row.
pub fn by_label(name: &str) -> Option<&'static TransformMenuBinding> {
    let wanted = normalise_command(name);
    BINDINGS
        .iter()
        .find(|binding| normalise_command(binding.label) == wanted)
}

/// The comparison form of a command name: lower case, without separators.
fn normalise_command(name: &str) -> String {
    name.chars()
        .filter(|character| !character.is_whitespace() && *character != '-' && *character != '_')
        .flat_map(char::to_lowercase)
        .collect()
}

/// Why a feature is unavailable, or `None` when it is enabled.
pub fn unavailable_reason(feature: &str) -> Option<&'static str> {
    let binding = by_feature(feature)?;
    (!binding.enabled()).then_some(binding.disabled_reason)
}
