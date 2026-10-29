// SPDX-License-Identifier: LGPL-2.1-or-later
//! Matrix `Tools > Metal Weights` (`OM9-L2-04-19`).
//!
//! The command is read-only: it measures the selected closed B-Reps, multiplies each volume
//! by its material density and reports grams plus pennyweights. The document, selection and
//! Undo journal are never touched, so a refused open shell or missing material leaves the
//! previous valid report intact.
use super::measure::{MeasureUnit, Measurement};
use super::*;
use om9_core::geometry::Transform;
use om9_core::metal_weight::{
    DEFAULT_MATERIAL_ID, MassBreakdown, MassPart, MetalMaterialRegistry, SolidMeasure,
    closed_solid_volume_mm3,
};
use om9_core::model::Entity;

/// Matrix Tools > Metal Weights icon (`BuilderMetalWeights`).
pub const METAL_WEIGHTS: &str = "OM9-TOOLS-006";
/// The same command typed at the Command Line.
pub const METAL_WEIGHTS_COMMAND: &str = "MetalWeights";

#[derive(Debug, Clone, PartialEq)]
pub struct MetalPartReport {
    pub object: om9_core::identity::ObjectId,
    pub label: String,
    pub material_id: String,
    pub density_g_cm3: f64,
    pub volume_mm3: f64,
    pub grams: f64,
    pub dwt: f64,
}

/// The report the Command Line, status line and tests read. It is plain data; it holds no
/// kernel handle and cannot mutate the document.
#[derive(Debug, Clone, PartialEq)]
pub struct MetalWeightReport {
    pub material_id: String,
    pub density_g_cm3: f64,
    pub volume_mm3: f64,
    pub grams: f64,
    pub dwt: f64,
    pub uncertainty_grams: f64,
    pub parts: Vec<MetalPartReport>,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetalCommand {
    /// `material=<id>` from the command line; the default material otherwise.
    pub material: Option<String>,
}

pub fn handles_command(feature: &str) -> bool {
    feature == METAL_WEIGHTS || feature == "BuilderMetalWeights"
}

/// Parse `MetalWeights`, `Metal Weights` and optional `material=<id>`.
///
/// It returns `Ok(None)` for a different command, so the shared dispatcher can fall through.
/// An unknown option is a typed refusal instead of a silently ignored token.
pub fn parse_command(text: &str) -> Result<Option<MetalCommand>> {
    let trimmed = text.trim();
    let lower = trimmed.to_ascii_lowercase();
    let rest = if lower.starts_with("metalweights") {
        &trimmed["metalweights".len()..]
    } else if lower.starts_with("metal weights") {
        &trimmed["metal weights".len()..]
    } else {
        return Ok(None);
    };
    let mut material = None;
    for token in rest.split_whitespace() {
        if let Some((key, value)) = token.split_once('=')
            && key.eq_ignore_ascii_case("material")
        {
            if value.trim().is_empty() {
                return Err(Error::Missing("metal material id"));
            }
            if material.replace(value.trim().to_owned()).is_some() {
                return Err(Error::Invalid("duplicate metal material option"));
            }
        } else if token.eq_ignore_ascii_case("calculate") {
            // The only action of this command; accepting the token keeps the prompt clear.
        } else {
            return Err(Error::Invalid("metal weights option"));
        }
    }
    Ok(Some(MetalCommand { material }))
}

impl Controller {
    /// The built-in reference materials plus any user materials this session has added.
    pub fn metal_registry(&self) -> &MetalMaterialRegistry {
        &self.metal_registry
    }

    pub fn metal_registry_mut(&mut self) -> &mut MetalMaterialRegistry {
        &mut self.metal_registry
    }

    /// The last successful report. A later refusal does not clear it.
    pub fn metal_report(&self) -> Option<&MetalWeightReport> {
        self.last_metal_report.as_ref()
    }

    /// Calculate the report for the current selection using one material.
    pub fn calculate_metal_weights(&mut self, material: Option<&str>) -> Result<MetalWeightReport> {
        let ids = self.selected_ordered();
        if ids.is_empty() {
            return Err(Error::Missing("metal weight selection"));
        }
        let material = material.unwrap_or(DEFAULT_MATERIAL_ID).to_owned();
        let assignments: Vec<_> = ids.iter().map(|id| (*id, material.as_str())).collect();
        self.calculate_metal_weights_with(&assignments)
    }

    /// Calculate a per-object report. `assignments` must name each selected object exactly
    /// once; a missing or extra id is refused so a report can never silently omit a body.
    pub fn calculate_metal_weights_with(
        &mut self,
        assignments: &[(om9_core::identity::ObjectId, &str)],
    ) -> Result<MetalWeightReport> {
        let ids = self.selected_ordered();
        if ids.is_empty() {
            return Err(Error::Missing("metal weight selection"));
        }
        if assignments.len() != ids.len() {
            return Err(Error::Invalid("metal weight material assignment"));
        }
        let mut assigned = std::collections::BTreeMap::new();
        for (id, material) in assignments {
            if !ids.contains(id) {
                return Err(Error::Invalid("metal weight material assignment"));
            }
            if assigned.insert(*id, (*material).to_owned()).is_some() {
                return Err(Error::Invalid("duplicate metal body"));
            }
        }

        let mut parts = Vec::with_capacity(ids.len());
        let mut objects = Vec::with_capacity(ids.len());
        for id in &ids {
            let entity = self
                .document()
                .entities
                .get(id)
                .ok_or(Error::Missing("metal weight object"))?;
            let volume = self.measure_metal_entity(entity)?;
            let material = assigned
                .remove(id)
                .ok_or(Error::Missing("metal material assignment"))?;
            let label = entity_label(entity, id);
            parts.push(MassPart::new(format!("{label} [{id}]"), volume, material));
            objects.push((*id, label));
        }

        let registry = self.metal_registry.clone();
        let breakdown = MassBreakdown::compute(&parts, &registry)?;
        let mixed = breakdown
            .entries
            .iter()
            .map(|entry| entry.material_id.as_str())
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            > 1;
        let first_material = registry
            .get(&breakdown.entries[0].material_id)
            .cloned()
            .map_err(|_| Error::Missing("metal material density"))?;
        let material_name = if mixed {
            "Mixed materials".to_owned()
        } else {
            first_material.name.clone()
        };
        let material_id = if mixed {
            "mixed".to_owned()
        } else {
            first_material.id.clone()
        };
        let mut report_parts = Vec::with_capacity(ids.len());
        for ((id, label), entry) in objects.iter().zip(breakdown.entries.iter()) {
            report_parts.push(MetalPartReport {
                object: *id,
                label: label.clone(),
                material_id: entry.material_id.clone(),
                density_g_cm3: entry.density_g_cm3,
                volume_mm3: entry.volume_mm3,
                grams: entry.grams,
                dwt: entry.dwt,
            });
        }
        let text = report_text(
            &material_name,
            &material_id,
            if mixed {
                0.0
            } else {
                first_material.density_g_cm3
            },
            breakdown.total_volume_mm3,
            breakdown.total_grams,
            breakdown.total_dwt,
            breakdown.total_uncertainty_grams,
            &report_parts,
        );
        let report = MetalWeightReport {
            material_id,
            density_g_cm3: if mixed {
                0.0
            } else {
                first_material.density_g_cm3
            },
            volume_mm3: breakdown.total_volume_mm3,
            grams: breakdown.total_grams,
            dwt: breakdown.total_dwt,
            uncertainty_grams: breakdown.total_uncertainty_grams,
            parts: report_parts,
            text,
        };
        self.last_metal_report = Some(report.clone());
        self.report_measurement(Measurement::with_text(
            METAL_WEIGHTS_COMMAND,
            report.grams,
            MeasureUnit::Gram,
            report.text.clone(),
        ));
        Ok(report)
    }

    pub fn dispatch_metal_tool(&mut self, feature: &str) -> Result<()> {
        if !handles_command(feature) {
            return Err(Error::Unsupported("metal weights command"));
        }
        self.calculate_metal_weights(None)?;
        Ok(())
    }

    #[cfg(feature = "occt-brep")]
    fn measure_metal_entity(&self, entity: &Entity) -> Result<f64> {
        let Geometry::Brep(brep) = &entity.geometry else {
            return Err(Error::Invalid("metal weights need closed BRep solids"));
        };
        let brep = if entity.placement == Transform::identity() {
            brep.clone()
        } else {
            om9_kernel::transform_ops::transform_brep(
                brep,
                &entity.placement,
                0.1,
                self.coordinator.limits(),
            )
            .map_err(|_| Error::Invalid("metal weight geometry"))?
        };
        let measure = om9_kernel::creation::measure(&brep, self.coordinator.limits())
            .map_err(|_| Error::Invalid("metal weight geometry"))?;
        if !measure.closed || measure.solids == 0 {
            return Err(Error::Invalid("metal weight requires a closed solid"));
        }
        closed_solid_volume_mm3(SolidMeasure {
            volume: measure.volume_mm3.abs(),
            unit: Unit::Millimeter,
            closed: measure.closed,
            solids: measure.solids,
        })
    }

    #[cfg(not(feature = "occt-brep"))]
    fn measure_metal_entity(&self, _entity: &Entity) -> Result<f64> {
        Err(Error::Unsupported("metal weights require OCCT adapter"))
    }
}

fn entity_label(entity: &Entity, id: &om9_core::identity::ObjectId) -> String {
    if entity.label.trim().is_empty() {
        id.to_string()
    } else {
        entity.label.clone()
    }
}

#[allow(clippy::too_many_arguments)]
fn report_text(
    material_name: &str,
    material_id: &str,
    density: f64,
    volume: f64,
    grams: f64,
    dwt: f64,
    uncertainty: f64,
    parts: &[MetalPartReport],
) -> String {
    let mut text = format!(
        "Metal Weights: {} ({}) — {} mm³ · {} g · {} dwt",
        material_name,
        material_id,
        Measurement::format_value(volume),
        Measurement::format_value(grams),
        Measurement::format_value(dwt),
    );
    if density > 0.0 {
        text.push_str(&format!(
            " · density {} g/cm³",
            Measurement::format_value(density)
        ));
    }
    if uncertainty > 0.0 {
        text.push_str(&format!(" · ±{} g", Measurement::format_value(uncertainty)));
    }
    if parts.len() > 1 {
        for part in parts {
            text.push_str(&format!(
                " | {}: {} g ({} dwt)",
                part.label,
                Measurement::format_value(part.grams),
                Measurement::format_value(part.dwt)
            ));
        }
    }
    text
}
