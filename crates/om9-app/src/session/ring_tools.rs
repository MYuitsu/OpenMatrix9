// SPDX-License-Identifier: LGPL-2.1-or-later
//! Matrix jewelry ring/profile tool commands (`OM9-L2-04-02`).
//!
//! The icons in the Builder/Tools palettes carry stable Matrix feature ids. Several of them
//! were catalogued before a service existed; this module is the one dispatcher that maps an
//! id to either the existing builder session (`Profile Placer`, `Outside Rail`), the existing
//! four-rail metadata command, or the profile merge/cap services. No command inserts geometry
//! directly outside the host transaction.
use om9_core::{builder::BuilderKind, curve_ops, geometry::Geometry, surface_ops};

use super::{Controller, Error, Result};

/// `Builder Profile Placer` icon (`OM9-TOOLS-002`).
pub const PROFILE_PLACER: &str = "OM9-TOOLS-002";
/// `Builder Outside Ring Rail` icon (`OM9-TOOLS-003`).
pub const OUTSIDE_RING_RAIL: &str = "OM9-TOOLS-003";
/// `Builder Custom Rail` icon (`OM9-TOOLS-004`). The same id backs the four-rail profile
/// command when five or more curves are selected; a smaller selection is treated as a custom
/// closed rail instead of failing with an unrelated "four rails" message.
pub const CUSTOM_RAIL: &str = "OM9-TOOLS-004";
/// `Builder Four Profile` icon (`OM9-TOOLS-032`).
pub const FOUR_RAIL_PROFILE: &str = "OM9-TOOLS-032";
/// `Builder Profile End Cap` icon (`OM9-TOOLS-033`).
pub const PROFILE_END_CAP: &str = "OM9-TOOLS-033";
/// `Builder Profile Merge` icon (`OM9-TOOLS-034`).
pub const PROFILE_MERGE: &str = "OM9-TOOLS-034";
/// `Builder Custom Rail Advanced` icon (`OM9-TOOLS-035`).
pub const CUSTOM_RAIL_ADVANCED: &str = "OM9-TOOLS-035";

/// Whether this module owns the feature id.
pub fn handles_command(feature: &str) -> bool {
    matches!(
        feature,
        PROFILE_PLACER
            | OUTSIDE_RING_RAIL
            | CUSTOM_RAIL
            | FOUR_RAIL_PROFILE
            | PROFILE_END_CAP
            | PROFILE_MERGE
            | CUSTOM_RAIL_ADVANCED
    )
}

/// The typed command names that reach the same service.
pub fn command_name(text: &str) -> Option<&'static str> {
    match text.trim().to_ascii_lowercase().as_str() {
        "profileplacer" | "profile placer" => Some(PROFILE_PLACER),
        "outsideringrail" | "outside ring rail" => Some(OUTSIDE_RING_RAIL),
        "customrail" | "custom rail" => Some(CUSTOM_RAIL),
        "fourrailprofile" | "four rail profile" | "fourprofile" => Some(FOUR_RAIL_PROFILE),
        "profileendcap" | "profile end cap" | "endcap" => Some(PROFILE_END_CAP),
        "profilemerge" | "profile merge" | "mergesections" => Some(PROFILE_MERGE),
        "customrailadvanced" | "custom rail advanced" | "advancedrail" => {
            Some(CUSTOM_RAIL_ADVANCED)
        }
        _ => None,
    }
}

impl Controller {
    /// Dispatch one Matrix ring/profile tool id.
    pub fn dispatch_ring_tool(&mut self, feature: &str) -> Result<()> {
        if self.is_running() {
            return Err(Error::Busy);
        }
        match feature {
            PROFILE_PLACER => self.begin_builder(BuilderKind::RailProfile),
            OUTSIDE_RING_RAIL => self.begin_builder(BuilderKind::OutsideRail),
            CUSTOM_RAIL => {
                if self.selected_ordered().len() >= 5 {
                    self.dispatch_four_rail_profile(super::surface_edit::FOUR_RAIL_PROFILE)
                } else {
                    self.dispatch_custom_rail(CUSTOM_RAIL)
                }
            }
            FOUR_RAIL_PROFILE => {
                self.dispatch_four_rail_profile(super::surface_edit::FOUR_RAIL_PROFILE)
            }
            CUSTOM_RAIL_ADVANCED => self.dispatch_custom_rail(CUSTOM_RAIL_ADVANCED),
            PROFILE_MERGE => self.dispatch_profile_merge(PROFILE_MERGE),
            PROFILE_END_CAP => self.dispatch_profile_endcap(PROFILE_END_CAP),
            _ => Err(Error::Unsupported("ring tool")),
        }
    }

    /// Accept a user-authored closed ring rail and record it as its own feature. The rail is
    /// validated as closed/planar/non-self-intersecting before it can feed a sweep.
    fn dispatch_custom_rail(&mut self, feature: &'static str) -> Result<()> {
        if self.selection.len() != 1 {
            return Err(Error::Missing("select one closed rail curve"));
        }
        let id = *self.selection.iter().next().expect("one selected");
        let entity = self
            .document()
            .entities
            .get(&id)
            .ok_or(Error::Missing("custom rail curve"))?;
        let Geometry::Curve(curve) = &entity.geometry else {
            return Err(Error::Invalid("custom rail must be a curve"));
        };
        curve_ops::validate_ring_rail(curve, 1.0e-6, self.coordinator.limits())?;
        self.commit_geometry(feature, "CustomRail", Geometry::Curve(curve.clone()))
    }

    /// Join compatible profile sections into one curve. Winding/plane/connection checks live
    /// in `curve_ops::merge_profile_sections`.
    fn dispatch_profile_merge(&mut self, feature: &'static str) -> Result<()> {
        let ordered = self.selected_ordered();
        if ordered.len() < 2 {
            return Err(Error::Invalid("select two profile sections"));
        }
        let mut curves = Vec::with_capacity(ordered.len());
        for id in ordered {
            let entity = self
                .document()
                .entities
                .get(&id)
                .ok_or(Error::Missing("profile section"))?;
            let Geometry::Curve(curve) = &entity.geometry else {
                return Err(Error::Invalid("profile section curve"));
            };
            curves.push(curve.clone());
        }
        let curve = curve_ops::merge_profile_sections(&curves, 1.0e-6, self.coordinator.limits())?;
        self.commit_geometry(feature, "ProfileMerge", Geometry::Curve(curve))
    }

    /// Close/end-cap a selected profile or open shell. A curve selection gets a planar cap
    /// surface; an open B-Rep gets the kernel's planar cap and becomes a closed solid.
    fn dispatch_profile_endcap(&mut self, feature: &'static str) -> Result<()> {
        let ordered = self.selected_ordered();
        if ordered.is_empty() {
            return Err(Error::Missing("select a profile or open shell"));
        }
        if ordered.len() == 1
            && let Some(entity) = self.document().entities.get(&ordered[0])
            && let Geometry::Brep(_) = &entity.geometry
        {
            return self.cap_open_shell(feature, ordered[0]);
        }
        let mut curves = Vec::with_capacity(ordered.len());
        for id in ordered {
            let entity = self
                .document()
                .entities
                .get(&id)
                .ok_or(Error::Missing("profile section"))?;
            let Geometry::Curve(curve) = &entity.geometry else {
                return Err(Error::Invalid("end cap needs closed curves"));
            };
            if !curve_ops::is_closed(curve, 1.0e-6, self.coordinator.limits())? {
                return Err(Error::Invalid("end cap needs closed curves"));
            }
            curves.push(curve.clone());
        }
        let surface = surface_ops::planar_surface_from_curves(&curves, self.coordinator.limits())?;
        self.commit_geometry(feature, "ProfileEndCap", Geometry::Surface(surface))
    }

    #[cfg(feature = "occt-brep")]
    fn cap_open_shell(
        &mut self,
        feature: &'static str,
        id: om9_core::identity::ObjectId,
    ) -> Result<()> {
        let entity = self
            .document()
            .entities
            .get(&id)
            .ok_or(Error::Missing("open shell"))?;
        let Geometry::Brep(brep) = &entity.geometry else {
            return Err(Error::Invalid("end cap needs an open shell"));
        };
        let capped = om9_kernel::editing::cap_planar(
            brep,
            1.0e-7,
            brep.display.deflection_mm,
            self.coordinator.limits(),
        )
        .map_err(|error| Error::Io(format!("profile end cap: {error}")))?;
        self.commit_geometry(feature, "ProfileEndCap", Geometry::Brep(capped))
    }

    #[cfg(not(feature = "occt-brep"))]
    fn cap_open_shell(
        &mut self,
        _feature: &'static str,
        _id: om9_core::identity::ObjectId,
    ) -> Result<()> {
        Err(Error::Unsupported(
            "end cap of an open shell needs the OCCT kernel",
        ))
    }
}
