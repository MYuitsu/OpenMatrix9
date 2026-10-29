// SPDX-License-Identifier: LGPL-2.1-or-later
//! Matrix Surface menu binding catalog (`OM9-L2-03-17`).
//!
//! One row per documented Surface Tools command. Every row names the authored icon, the
//! Matrix command-line name, a unique command id and the verified owner candidate. The menu
//! row and the typed command line resolve to the same feature id, so both entry points run
//! one implementation (`session/surface_edit.rs`) and one transaction. History rows share
//! their base command icon but keep a distinct id and command name.

/// Why a menu row is not enabled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingState {
    Implemented,
    SessionMissing,
    OwnerMissing,
}

/// One Surface menu row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurfaceMenuBinding {
    pub l3_id: &'static str,
    pub label: &'static str,
    pub source_command: &'static str,
    /// Matrix command-line name: the authored script without `! _`, so typing it starts
    /// the same command as clicking the menu row.
    pub command_line: &'static str,
    pub icon: &'static str,
    pub feature: &'static str,
    pub owner: &'static str,
    pub state: BindingState,
    pub disabled_reason: &'static str,
}

impl SurfaceMenuBinding {
    pub fn enabled(&self) -> bool {
        matches!(self.state, BindingState::Implemented)
    }
}

pub const BINDINGS: &[SurfaceMenuBinding; 38] = &[
    SurfaceMenuBinding {
        l3_id: "OM9-L3-03-17-001",
        label: "Sweep 1",
        source_command: "Sweep 1",
        icon: "SurfaceSweepSweep1Rail",
        feature: "OM9-SURFACE-101",
        command_line: "Sweep1",
        owner: "OM9-L3-01-09-007",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    SurfaceMenuBinding {
        l3_id: "OM9-L3-03-17-002",
        label: "Sweep 1 History",
        source_command: "Sweep 1 History",
        icon: "SurfaceSweepSweep1Rail",
        feature: "OM9-SURFACE-102",
        command_line: "gvSweepHistory",
        owner: "OM9-L3-01-09-007",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    SurfaceMenuBinding {
        l3_id: "OM9-L3-03-17-003",
        label: "Sweep 2",
        source_command: "Sweep 2",
        icon: "SurfaceSweepSweep2Rails",
        feature: "OM9-SURFACE-003",
        command_line: "Sweep2",
        owner: "OM9-L3-01-09-007",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    SurfaceMenuBinding {
        l3_id: "OM9-L3-03-17-004",
        label: "Sweep 2 History",
        source_command: "Sweep 2 History",
        icon: "SurfaceSweepSweep2Rails",
        feature: "OM9-SURFACE-104",
        command_line: "gvSweep2History",
        owner: "OM9-L3-01-09-007",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    SurfaceMenuBinding {
        l3_id: "OM9-L3-03-17-005",
        label: "Profile Sweep",
        source_command: "Profile Sweep",
        icon: "SurfaceSweepProfileSweep",
        feature: "OM9-SURFACE-005",
        command_line: "gvProfileSweep",
        owner: "OM9-L3-01-09-007",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    SurfaceMenuBinding {
        l3_id: "OM9-L3-03-17-006",
        label: "Rebuild",
        source_command: "Rebuild",
        icon: "OthersSurfaceRebuild",
        feature: "OM9-SURFACE-006",
        command_line: "Rebuild",
        owner: "OM9-L3-02-04-008",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    SurfaceMenuBinding {
        l3_id: "OM9-L3-03-17-007",
        label: "Blend Surface",
        source_command: "Blend Surface",
        icon: "SurfaceBlendSurface",
        feature: "OM9-SURFACE-007",
        command_line: "BlendSrf",
        owner: "OM9-L3-02-04-007",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    SurfaceMenuBinding {
        l3_id: "OM9-L3-03-17-008",
        label: "Variable Blend Surfaces",
        source_command: "Variable Blend Surfaces",
        icon: "SurfaceVariableFilletBlendChamferVariableBlendSurfaces",
        feature: "OM9-SURFACE-008",
        command_line: "VariableBlendSrf",
        owner: "OM9-L3-02-04-007",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    SurfaceMenuBinding {
        l3_id: "OM9-L3-03-17-009",
        label: "Loft",
        source_command: "Loft",
        icon: "SurfaceLoft",
        feature: "OM9-SURFACE-109",
        command_line: "Loft",
        owner: "OM9-L3-01-09-007",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    SurfaceMenuBinding {
        l3_id: "OM9-L3-03-17-010",
        label: "Curve Network",
        source_command: "Curve Network",
        icon: "SurfaceCurveNetwork",
        feature: "OM9-SURFACE-010",
        command_line: "NetworkSrf",
        owner: "OM9-L2-03-17",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    SurfaceMenuBinding {
        l3_id: "OM9-L3-03-17-011",
        label: "Shrink Trimmed Surface",
        source_command: "Shrink Trimmed Surface",
        icon: "SurfaceSurfaceEditToolsShrinkTrimmedSurface",
        feature: "OM9-SURFACE-012",
        command_line: "ShrinkTrimmedSrf",
        owner: "OM9-L3-02-04-005",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    SurfaceMenuBinding {
        l3_id: "OM9-L3-03-17-012",
        label: "Patch",
        source_command: "Patch",
        icon: "SurfacePatch",
        feature: "OM9-SURFACE-011",
        command_line: "Patch",
        owner: "OM9-L2-03-17",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    SurfaceMenuBinding {
        l3_id: "OM9-L3-03-17-013",
        label: "Planar Curves",
        source_command: "Planar Curves",
        icon: "SurfacePlanarCurves",
        feature: "OM9-SURFACE-013",
        command_line: "PlanarSrf",
        owner: "OM9-L2-03-17",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    SurfaceMenuBinding {
        l3_id: "OM9-L3-03-17-014",
        label: "Edge Curves",
        source_command: "Edge Curves",
        icon: "SurfaceEdgeCurves",
        feature: "OM9-SURFACE-014",
        command_line: "EdgeSrf",
        owner: "OM9-L2-03-17",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    SurfaceMenuBinding {
        l3_id: "OM9-L3-03-17-015",
        label: "Plane from Corners",
        source_command: "Plane from Corners",
        icon: "SurfacePlaneCornertoCorner",
        feature: "OM9-SURFACE-015",
        command_line: "Plane",
        owner: "OM9-L2-03-17",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    SurfaceMenuBinding {
        l3_id: "OM9-L3-03-17-016",
        label: "Untrim",
        source_command: "Untrim",
        icon: "SurfaceSurfaceEditToolsUntrim",
        feature: "OM9-SURFACE-017",
        command_line: "Untrim",
        owner: "OM9-L3-02-04-005",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    SurfaceMenuBinding {
        l3_id: "OM9-L3-03-17-017",
        label: "Surface Extrude All",
        source_command: "Surface Extrude All",
        icon: "SurfaceGVDExtrudeAll",
        feature: "OM9-SURFACE-117",
        command_line: "ExtrudeCrv",
        owner: "OM9-L3-01-09-002",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    SurfaceMenuBinding {
        l3_id: "OM9-L3-03-17-018",
        label: "Rail Revolve",
        source_command: "Rail Revolve",
        icon: "SurfaceRailRevolve",
        feature: "OM9-SURFACE-019",
        command_line: "RailRevolve",
        owner: "OM9-L3-01-09-003",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    SurfaceMenuBinding {
        l3_id: "OM9-L3-03-17-019",
        label: "Sweep 4",
        source_command: "Sweep 4",
        icon: "BuilderFourProfileSweep",
        feature: "OM9-SURFACE-119",
        command_line: "gvFourProfileSweep",
        owner: "OM9-L3-01-09-007",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    SurfaceMenuBinding {
        l3_id: "OM9-L3-03-17-020",
        label: "Fillet Surfaces",
        source_command: "Fillet Surfaces",
        icon: "SurfaceFilletSurfaces",
        feature: "OM9-SURFACE-020",
        command_line: "FilletSrf",
        owner: "OM9-L2-03-17",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    SurfaceMenuBinding {
        l3_id: "OM9-L3-03-17-021",
        label: "Variable Fillet Surfaces",
        source_command: "Variable Fillet Surfaces",
        icon: "SurfaceVariableFilletBlendChamferVariableFilletSurfaces",
        feature: "OM9-SURFACE-022",
        command_line: "VariableFilletSrf",
        owner: "OM9-L2-03-17",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    SurfaceMenuBinding {
        l3_id: "OM9-L3-03-17-022",
        label: "Chamfer Surfaces",
        source_command: "Chamfer Surfaces",
        icon: "SurfaceChamferSurfaces",
        feature: "OM9-SURFACE-023",
        command_line: "ChamferSrf",
        owner: "OM9-L2-03-17",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    SurfaceMenuBinding {
        l3_id: "OM9-L3-03-17-023",
        label: "Variable Chamfer Surfaces",
        source_command: "Variable Chamfer Surfaces",
        icon: "SurfaceVariableFilletBlendChamferVariableChamferSurfaces",
        feature: "OM9-SURFACE-024",
        command_line: "VariableChamferSrf",
        owner: "OM9-L2-03-17",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    SurfaceMenuBinding {
        l3_id: "OM9-L3-03-17-024",
        label: "Offset Surface",
        source_command: "Offset Surface",
        icon: "SurfaceOffsetSurface",
        feature: "OM9-SURFACE-025",
        command_line: "OffsetSrf",
        owner: "OM9-L2-03-17",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    SurfaceMenuBinding {
        l3_id: "OM9-L3-03-17-025",
        label: "Variable Offset Surface",
        source_command: "Variable Offset Surface",
        icon: "SurfaceVariableOffsetSurface",
        feature: "OM9-SURFACE-026",
        command_line: "VariableOffsetSrf",
        owner: "OM9-L3-01-10-007",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    SurfaceMenuBinding {
        l3_id: "OM9-L3-03-17-026",
        label: "Tween Surfaces",
        source_command: "Tween Surfaces",
        icon: "SurfaceTweenSurfaces",
        feature: "OM9-SURFACE-027",
        command_line: "TweenSurfaces",
        owner: "OM9-L2-03-17",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    SurfaceMenuBinding {
        l3_id: "OM9-L3-03-17-027",
        label: "Merge Surfaces",
        source_command: "Merge Surfaces",
        icon: "SurfaceSurfaceEditToolsMerge",
        feature: "OM9-SURFACE-127",
        command_line: "MergeSrf",
        owner: "OM9-L2-03-17",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    SurfaceMenuBinding {
        l3_id: "OM9-L3-03-17-028",
        label: "Match Surfaces",
        source_command: "Match Surfaces",
        icon: "SurfaceSurfaceEditToolsMatch",
        feature: "OM9-SURFACE-128",
        command_line: "MatchSrf",
        owner: "OM9-L2-03-17",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    SurfaceMenuBinding {
        l3_id: "OM9-L3-03-17-029",
        label: "Drape",
        source_command: "Drape",
        icon: "SurfaceDrape",
        feature: "OM9-SURFACE-030",
        command_line: "Drape",
        owner: "OM9-L2-03-17",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    SurfaceMenuBinding {
        l3_id: "OM9-L3-03-17-030",
        label: "Sweep Multi",
        source_command: "Sweep Multi",
        icon: "SurfaceSweepSweepMultiRail",
        feature: "OM9-SURFACE-032",
        command_line: "gvfourSweep",
        owner: "OM9-L3-01-09-007",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    SurfaceMenuBinding {
        l3_id: "OM9-L3-03-17-031",
        label: "Split at Isocurve",
        source_command: "Split at Isocurve",
        icon: "SurfaceSurfaceEditToolsSplitatIsocurve",
        feature: "OM9-SURFACE-034",
        command_line: "Split",
        owner: "OM9-L3-02-04-005",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    SurfaceMenuBinding {
        l3_id: "OM9-L3-03-17-032",
        label: "Extend Surface",
        source_command: "Extend Surface",
        icon: "SurfaceExtendSurface",
        feature: "OM9-SURFACE-033",
        command_line: "ExtendSrf",
        owner: "OM9-L2-03-17",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    SurfaceMenuBinding {
        l3_id: "OM9-L3-03-17-033",
        label: "Extrude Normal to Surface",
        source_command: "Extrude Normal to Surface",
        icon: "SurfaceExtrudeCurveNormalToSurface",
        feature: "OM9-SURFACE-133",
        command_line: "Fin",
        owner: "OM9-L2-03-17",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    SurfaceMenuBinding {
        l3_id: "OM9-L3-03-17-034",
        label: "Unroll Surface",
        source_command: "Unroll Surface",
        icon: "SurfaceUnrollDevelopableSrf",
        feature: "OM9-SURFACE-134",
        command_line: "UnrollSrf",
        owner: "OM9-L2-03-17",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    SurfaceMenuBinding {
        l3_id: "OM9-L3-03-17-035",
        label: "Soft Edit",
        source_command: "Soft Edit",
        icon: "SurfaceSurfaceEditToolsSoftEdit",
        feature: "OM9-SURFACE-037",
        command_line: "SoftEditSrf",
        owner: "OM9-L2-03-17",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    SurfaceMenuBinding {
        l3_id: "OM9-L3-03-17-036",
        label: "Cutting Plane",
        source_command: "Cutting Plane",
        icon: "SurfacePlaneCuttingPlane",
        feature: "OM9-SURFACE-039",
        command_line: "CutPlane",
        owner: "OM9-L2-03-17",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    SurfaceMenuBinding {
        l3_id: "OM9-L3-03-17-037",
        label: "Adjust Closed Surface Seam",
        source_command: "Adjust Closed Surface Seam",
        icon: "SurfaceSurfaceEditToolsAdjustClosedSurfaceSeam",
        feature: "OM9-SURFACE-038",
        command_line: "SrfSeam",
        owner: "OM9-L3-02-04-005",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
    SurfaceMenuBinding {
        l3_id: "OM9-L3-03-17-038",
        label: "Set Surface Tangent",
        source_command: "Set Surface Tangent",
        icon: "SurfaceSurfaceEditToolsSetSurfaceTangent",
        feature: "OM9-SURFACE-040",
        command_line: "SetSurfaceTangent",
        owner: "OM9-L2-03-17",
        state: BindingState::Implemented,
        disabled_reason: "",
    },
];

pub fn by_l3(l3_id: &str) -> Option<&'static SurfaceMenuBinding> {
    BINDINGS.iter().find(|binding| binding.l3_id == l3_id)
}

pub fn by_feature(feature: &str) -> Option<&'static SurfaceMenuBinding> {
    BINDINGS.iter().find(|binding| binding.feature == feature)
}

pub fn by_icon(icon: &str) -> Option<&'static SurfaceMenuBinding> {
    BINDINGS.iter().find(|binding| binding.icon == icon)
}

/// Resolve a typed Matrix command name to its menu row.
///
/// The authored scripts read `! _Sweep1`; the accepted name is the token after the
/// underscore, compared case-insensitively, and a leading `!`/`_` is tolerated so a pasted
/// script starts the same command as the menu row.
pub fn by_command_line(text: &str) -> Option<&'static SurfaceMenuBinding> {
    let name = text
        .trim()
        .trim_start_matches('!')
        .trim()
        .trim_start_matches('_');
    if name.is_empty() {
        return None;
    }
    BINDINGS
        .iter()
        .find(|binding| binding.command_line.eq_ignore_ascii_case(name))
}

/// Resolve the visible Surface menu label, so typing `Sweep 1 History` reaches the row
/// that the icon opens.
pub fn by_label(text: &str) -> Option<&'static SurfaceMenuBinding> {
    let label = text.trim();
    if label.is_empty() {
        return None;
    }
    BINDINGS.iter().find(|binding| {
        binding.label.eq_ignore_ascii_case(label)
            || binding.source_command.eq_ignore_ascii_case(label)
    })
}

pub fn enabled(feature: &str) -> bool {
    by_feature(feature).is_some_and(SurfaceMenuBinding::enabled)
}

pub fn disabled_reason(feature: &str) -> Option<&'static str> {
    by_feature(feature).and_then(|binding| (!binding.enabled()).then_some(binding.disabled_reason))
}
