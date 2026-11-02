// SPDX-License-Identifier: LGPL-2.1-or-later
//! Settings Associative Edit and Mass (`OM9-L3-04-15-010`).
//!
//! One prong feature stores the gem row it was built from plus its parameter block. When the
//! gem row changes (a stone is added, moved or removed) the feature rebuilds from the *shared*
//! row and reports exactly which prong ids changed. Ids are anchor + slot, so an untouched prong
//! keeps its identity across the rebuild and the caller can replace only the bodies that moved.
//!
//! Mass reuses the metal-weight owner: settings never invent a second density table or a second
//! gram conversion. A feature contributes its watertight prong volume, grouped per feature and
//! per material, and the existing [`MassBreakdown`] does the arithmetic.
use crate::{
    Error, Result,
    geometry::Mesh,
    metal_weight::{MassBreakdown, MassPart, MetalEstimate, MetalMaterialRegistry},
    settings::gems::GemRow,
    settings::prong::{ProngParams, ProngSet, ProngSite, build_prong_set, place_prongs},
};

/// A prong feature bound to a gem row.
#[derive(Debug, Clone, PartialEq)]
pub struct ProngFeature {
    pub id: String,
    pub row: GemRow,
    pub params: ProngParams,
    pub sites: Vec<ProngSite>,
    pub volume_mm3: f64,
    pub bodies: Vec<Mesh>,
}

impl ProngFeature {
    /// Build the feature from the shared row.
    pub fn new(id: impl Into<String>, row: GemRow, params: ProngParams) -> Result<Self> {
        let id = id.into();
        if id.trim().is_empty() {
            return Err(Error::Missing("prong feature id"));
        }
        let set = build_prong_set(&row, &params)?;
        Ok(Self {
            id,
            row,
            params,
            sites: set.sites,
            volume_mm3: set.volume_mm3,
            bodies: set.bodies,
        })
    }

    pub fn stable_ids(&self) -> Vec<String> {
        self.sites.iter().map(ProngSite::stable_id).collect()
    }

    pub fn count(&self) -> usize {
        self.sites.len()
    }

    pub fn mass(&self, material_id: &str, registry: &MetalMaterialRegistry) -> Result<MetalEstimate> {
        let material = registry.get(material_id)?;
        MetalEstimate::new(self.volume_mm3, material)
    }
}

/// What a rebuild changed.
#[derive(Debug, Clone, PartialEq)]
pub struct RebuildReport {
    pub feature_id: String,
    pub before: usize,
    pub after: usize,
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub kept: usize,
    pub volume_mm3: f64,
    pub changed: bool,
}

impl RebuildReport {
    pub fn summary(&self) -> String {
        format!(
            "{} {} -> {} prongs (+{} / -{}), {:.3} mm3",
            self.feature_id,
            self.before,
            self.after,
            self.added.len(),
            self.removed.len(),
            self.volume_mm3
        )
    }
}

/// Rebuild a prong feature against an updated gem row, keeping ids that still exist.
pub fn rebuild_prong_feature(
    feature: &ProngFeature,
    updated_row: &GemRow,
    params: Option<&ProngParams>,
) -> Result<(ProngFeature, RebuildReport)> {
    updated_row.validate()?;
    let params = params.cloned().unwrap_or_else(|| feature.params.clone());
    let before = feature.stable_ids();
    let set: ProngSet = build_prong_set(updated_row, &params)?;
    let after: Vec<String> = set.sites.iter().map(ProngSite::stable_id).collect();
    let added: Vec<String> = after
        .iter()
        .filter(|id| !before.contains(id))
        .cloned()
        .collect();
    let removed: Vec<String> = before
        .iter()
        .filter(|id| !after.contains(id))
        .cloned()
        .collect();
    let kept = after.iter().filter(|id| before.contains(id)).count();
    let changed = !added.is_empty() || !removed.is_empty() || kept != after.len();
    let report = RebuildReport {
        feature_id: feature.id.clone(),
        before: before.len(),
        after: after.len(),
        added,
        removed,
        kept,
        volume_mm3: set.volume_mm3,
        changed,
    };
    let rebuilt = ProngFeature {
        id: feature.id.clone(),
        row: updated_row.clone(),
        params,
        sites: set.sites,
        volume_mm3: set.volume_mm3,
        bodies: set.bodies,
    };
    Ok((rebuilt, report))
}

/// A prong edit applied through the feature, so the document update is one transaction.
pub struct FeatureEdit {
    pub site_index: usize,
    pub edit: crate::settings::prong::ProngEdit,
}

/// Apply one prong edit and keep the feature's totals consistent.
pub fn edit_feature_prong(feature: &mut ProngFeature, edit: &FeatureEdit) -> Result<Mesh> {
    let body = crate::settings::prong::edit_prong(
        &mut feature.sites,
        edit.site_index,
        &feature.params,
        &edit.edit,
    )?;
    if edit.site_index >= feature.bodies.len() {
        return Err(Error::Missing("prong body"));
    }
    feature.bodies[edit.site_index] = body.clone();
    let mut total = 0.0;
    for body in &feature.bodies {
        total += crate::settings::inspect_solid(body)?.volume_mm3;
    }
    feature.volume_mm3 = total;
    Ok(body)
}

/// One line of a settings mass report: one feature, one material.
#[derive(Debug, Clone, PartialEq)]
pub struct FeatureMassPart {
    pub feature_id: String,
    pub volume_mm3: f64,
    pub material_id: String,
    pub instances: u32,
}

impl FeatureMassPart {
    pub fn new(feature_id: impl Into<String>, volume_mm3: f64, material_id: impl Into<String>) -> Self {
        Self {
            feature_id: feature_id.into(),
            volume_mm3,
            material_id: material_id.into(),
            instances: 1,
        }
    }

    fn as_mass_part(&self) -> MassPart {
        MassPart::new(
            self.feature_id.clone(),
            self.volume_mm3,
            self.material_id.clone(),
        )
        .with_instances(self.instances)
    }
}

/// Mass of every settings feature, per material, in grams and dwt.
pub fn settings_mass(
    parts: &[FeatureMassPart],
    registry: &MetalMaterialRegistry,
) -> Result<MassBreakdown> {
    if parts.is_empty() {
        return Err(Error::Missing("settings mass parts"));
    }
    for part in parts {
        if part.instances == 0 {
            return Err(Error::Invalid("settings mass instances"));
        }
        if part.feature_id.trim().is_empty() {
            return Err(Error::Missing("settings mass feature id"));
        }
    }
    let mass_parts: Vec<MassPart> = parts.iter().map(FeatureMassPart::as_mass_part).collect();
    MassBreakdown::compute(&mass_parts, registry)
}

/// Total prong volume of a set of sites, for a caller that already placed them.
pub fn sites_volume_mm3(sites: &[ProngSite], params: &ProngParams) -> Result<f64> {
    if sites.is_empty() {
        return Err(Error::Missing("prong sites"));
    }
    let mut total = 0.0;
    for site in sites {
        let body = crate::settings::prong::build_prong(site, params)?;
        total += crate::settings::inspect_solid(&body)?.volume_mm3;
    }
    Ok(total)
}

/// Re-place the sites of a row without building bodies, the cheap first step of a rebuild.
pub fn reflow_sites(row: &GemRow, params: &ProngParams) -> Result<Vec<ProngSite>> {
    place_prongs(row, params)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::gems::{GemStone, RowShape, straight_row};

    fn row() -> GemRow {
        straight_row(
            &[2.0, 2.0, 2.0],
            6.0,
            [0.0; 3],
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0],
        )
        .unwrap()
    }

    fn params() -> ProngParams {
        ProngParams {
            mode: crate::settings::prong::ProngMode::FourProngsAt(0.0),
            ..ProngParams::default()
        }
    }

    #[test]
    fn a_feature_rebuilds_from_the_shared_row() {
        let feature = ProngFeature::new("prongs-1", row(), params()).unwrap();
        assert_eq!(feature.count(), 12);
        assert_eq!(feature.stable_ids().len(), 12);
        assert!(feature.volume_mm3 > 0.0);
    }

    #[test]
    fn adding_a_stone_adds_prongs_and_keeps_the_existing_ids() {
        let feature = ProngFeature::new("prongs-1", row(), params()).unwrap();
        let before = feature.stable_ids();
        let mut stones = feature.row.stones.clone();
        stones.push(
            GemStone::round("g3", [18.0, 0.0, 0.0], 2.0, [0.0, 0.0, 1.0]).unwrap(),
        );
        let updated = GemRow::open(stones).unwrap();
        let (rebuilt, report) = rebuild_prong_feature(&feature, &updated, None).unwrap();
        assert!(report.changed);
        assert_eq!(report.kept, before.len());
        assert_eq!(report.after, before.len() + 4);
        assert!(rebuilt.count() > feature.count());
        for id in &before {
            assert!(rebuilt.stable_ids().contains(id));
        }
    }

    #[test]
    fn removing_a_stone_removes_only_its_prong_ids() {
        let feature = ProngFeature::new("prongs-1", row(), params()).unwrap();
        let mut stones = feature.row.stones.clone();
        stones.remove(1);
        let updated = GemRow::open(stones).unwrap();
        let (rebuilt, report) = rebuild_prong_feature(&feature, &updated, None).unwrap();
        assert_eq!(report.removed.len(), 4);
        assert!(report.removed.iter().all(|id| id.starts_with("gem_02_")));
        assert!(!rebuilt.stable_ids().iter().any(|id| id.starts_with("gem_02_")));
    }

    #[test]
    fn an_unchanged_rebuild_is_reported_as_unchanged() {
        let feature = ProngFeature::new("prongs-1", row(), params()).unwrap();
        let (_, report) = rebuild_prong_feature(&feature, &feature.row.clone(), None).unwrap();
        assert!(!report.changed);
        assert!(report.added.is_empty() && report.removed.is_empty());
        assert_eq!(report.kept, report.after);
    }

    #[test]
    fn a_closed_row_feature_round_trips() {
        let stones = (0..6)
            .map(|index| {
                let angle = std::f64::consts::TAU * index as f64 / 6.0;
                GemStone::round(
                    format!("g{index}"),
                    [8.0 * angle.cos(), 8.0 * angle.sin(), 0.0],
                    2.0,
                    [0.0, 0.0, 1.0],
                )
                .unwrap()
            })
            .collect();
        let closed = GemRow::new(stones, RowShape::Closed).unwrap();
        let feature = ProngFeature::new("prongs-c", closed, params()).unwrap();
        assert!(feature.count() > 0);
        let (_, report) = rebuild_prong_feature(&feature, &feature.row.clone(), None).unwrap();
        assert!(!report.changed);
    }

    #[test]
    fn a_prong_edit_updates_the_feature_volume_and_one_body() {
        let mut feature = ProngFeature::new("prongs-1", row(), params()).unwrap();
        let before = feature.volume_mm3;
        let edit = FeatureEdit {
            site_index: 0,
            edit: crate::settings::prong::ProngEdit {
                diameter_mm: Some(1.6),
                ..Default::default()
            },
        };
        edit_feature_prong(&mut feature, &edit).unwrap();
        assert_eq!(feature.bodies.len(), feature.count());
        assert!(feature.volume_mm3 > before);
    }

    #[test]
    fn mass_uses_the_shared_material_registry() {
        let feature = ProngFeature::new("prongs-1", row(), params()).unwrap();
        let registry = MetalMaterialRegistry::builtin();
        let material = registry
            .iter()
            .next()
            .map(|material| material.id.clone())
            .unwrap();
        let estimate = feature.mass(&material, &registry).unwrap();
        assert!((estimate.grams - feature.volume_mm3 / 1000.0 * estimate.density_g_cm3).abs() < 1.0e-9);
        assert!(estimate.dwt > 0.0);
    }

    #[test]
    fn a_multi_feature_mass_report_groups_by_material() {
        let feature = ProngFeature::new("prongs-1", row(), params()).unwrap();
        let registry = MetalMaterialRegistry::builtin();
        let material = registry
            .iter()
            .next()
            .map(|material| material.id.clone())
            .unwrap();
        let parts = vec![
            FeatureMassPart::new("prongs-1", feature.volume_mm3, material.clone()),
            FeatureMassPart::new("metal-1", 4.0, material.clone()),
        ];
        let breakdown = settings_mass(&parts, &registry).unwrap();
        assert!(!breakdown.entries.is_empty());
        assert!((breakdown.total_grams - (feature.volume_mm3 + 4.0) / 1000.0 * breakdown.entries[0].density_g_cm3).abs() < 1.0e-9);
    }

    #[test]
    fn a_duplicate_feature_key_is_refused_by_the_shared_owner() {
        let registry = MetalMaterialRegistry::builtin();
        let material = registry
            .iter()
            .next()
            .map(|material| material.id.clone())
            .unwrap();
        let parts = vec![
            FeatureMassPart::new("prongs-1", 1.0, material.clone()),
            FeatureMassPart::new("prongs-1", 2.0, material),
        ];
        assert!(settings_mass(&parts, &registry).is_err());
    }

    #[test]
    fn an_empty_feature_id_is_refused() {
        assert_eq!(
            ProngFeature::new("  ", row(), params()).err(),
            Some(Error::Missing("prong feature id"))
        );
    }

    #[test]
    fn sites_volume_and_reflow_agree_with_the_set_builder() {
        let feature = ProngFeature::new("prongs-1", row(), params()).unwrap();
        let sites = reflow_sites(&feature.row, &feature.params).unwrap();
        assert_eq!(sites.len(), feature.count());
        let volume = sites_volume_mm3(&sites, &feature.params).unwrap();
        assert!((volume - feature.volume_mm3).abs() < 1.0e-6);
    }
}
