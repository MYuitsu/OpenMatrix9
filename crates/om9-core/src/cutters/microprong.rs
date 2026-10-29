// SPDX-License-Identifier: LGPL-2.1-or-later
//! Micro-prong cutter: the row of scallops a channel leaves between its stones
//! (`BuilderMicroProngCutter`).
//!
//! One cutter is a rounded scallop sunk into the channel wall at a station along the stone row.
//! `Count=0` places one scallop in every gap between two stones, which is the manual's default
//! for a channel; a positive `Count` distributes that many scallops evenly along the row so the
//! spacing is independent of the stones.
//!
//! `Angle=` shears each scallop along the row tangent, which is how a micro-prong leans towards
//! the stone it holds (`Angle=0` is straight down).

use crate::{
    Error, Result,
    cutters::{MAX_CUTTERS, MIN_CUTTER_MM, loft, place, volume_mm3, Ring},
    gem::GemSpec,
    geometry::{Mesh, Point3, Transform},
};

/// The typed parameters of the micro-prong cutter.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MicroProngParams {
    /// The scallop radius, mm.
    pub radius_mm: f64,
    /// How far the scallop sinks below the girdle plane, mm.
    pub depth_mm: f64,
    /// How far the scallop rises above the girdle plane, mm.
    pub height_mm: f64,
    /// `0` places one scallop per gap; a positive value places exactly this many along the row.
    pub count: usize,
    /// Extra room between the row ends and the first/last scallop, mm.
    pub spacing_mm: f64,
    /// The lean of the scallop along the row tangent, degrees.
    pub angle_deg: f64,
    /// Ring segments of one scallop.
    pub segments: usize,
}

impl Default for MicroProngParams {
    fn default() -> Self {
        Self {
            radius_mm: 0.22,
            depth_mm: 0.40,
            height_mm: 0.30,
            count: 0,
            spacing_mm: 0.0,
            angle_deg: 0.0,
            segments: 16,
        }
    }
}

impl MicroProngParams {
    /// Defaults measured from the row: a scallop about an eighth of the widest stone, sunk a
    /// quarter of the deepest pavilion.
    pub fn for_row(stones: &[GemSpec]) -> Result<Self> {
        let widest = stones.iter().map(|spec| spec.width_mm).fold(0.0, f64::max);
        let deepest = stones
            .iter()
            .map(|spec| spec.pavilion_depth_mm)
            .fold(0.0, f64::max);
        if widest <= 0.0 || deepest <= 0.0 {
            return Err(Error::Invalid("microprong gems"));
        }
        let params = Self {
            radius_mm: (widest * 0.12).clamp(MIN_CUTTER_MM * 2.0, 1.20),
            depth_mm: (deepest * 0.25).max(MIN_CUTTER_MM),
            height_mm: 0.30,
            ..Self::default()
        };
        params.validate()?;
        Ok(params)
    }

    pub fn validate(&self) -> Result<()> {
        let values = [
            self.radius_mm,
            self.depth_mm,
            self.height_mm,
            self.spacing_mm,
            self.angle_deg,
        ];
        if values.iter().any(|value| !value.is_finite()) {
            return Err(Error::Invalid("microprong size"));
        }
        if self.radius_mm < MIN_CUTTER_MM || self.depth_mm < MIN_CUTTER_MM {
            return Err(Error::Invalid("microprong radius"));
        }
        if self.radius_mm > 4.0 {
            return Err(Error::Invalid("microprong radius"));
        }
        if self.height_mm < 0.0 || self.spacing_mm < 0.0 {
            return Err(Error::Invalid("microprong height"));
        }
        if self.angle_deg.abs() > 60.0 {
            return Err(Error::Invalid("microprong angle"));
        }
        if !(6..=128).contains(&self.segments) {
            return Err(Error::Invalid("microprong segments"));
        }
        if self.count > MAX_CUTTERS {
            return Err(Error::Invalid("microprong count"));
        }
        Ok(())
    }
}

/// The built micro-prong set in world space.
#[derive(Debug, Clone, PartialEq)]
pub struct MicroProngCutter {
    /// The stones the set was built along, in row order (stable provenance).
    pub gem_ids: Vec<String>,
    pub cutters: Vec<Mesh>,
    pub volumes_mm3: Vec<f64>,
    pub total_volume_mm3: f64,
    pub summary: String,
}

impl MicroProngCutter {
    pub fn count(&self) -> usize {
        self.cutters.len()
    }

    pub fn is_cutter(&self) -> bool {
        true
    }
}

/// Build one scallop per station along the ordered stone row.
pub fn build_row(
    stones: &[(String, GemSpec, Transform)],
    params: &MicroProngParams,
) -> Result<MicroProngCutter> {
    if stones.is_empty() {
        return Err(Error::Missing("microprong gems"));
    }
    params.validate()?;
    let centres = stones
        .iter()
        .map(|(_, _, placement)| placement.point([0.0, 0.0, 0.0]))
        .collect::<Result<Vec<_>>>()?;
    let count = centres.len();
    let tangent = if count >= 2 {
        crate::settings::unit(crate::settings::sub(centres[count - 1], centres[0]))?
    } else {
        [1.0, 0.0, 0.0]
    };
    let girdle = stones
        .iter()
        .map(|(_, spec, _)| spec.girdle_mm * 0.5)
        .fold(0.0, f64::max);

    // Stations: one per gap by default, or the requested count spread evenly along the row.
    let mut stations: Vec<Point3> = Vec::new();
    if params.count == 0 {
        if count == 1 {
            stations.push(centres[0]);
        } else {
            for pair in centres.windows(2) {
                stations.push(crate::settings::mul(
                    crate::settings::add(pair[0], pair[1]),
                    0.5,
                ));
            }
        }
    } else {
        let mut total = 0.0;
        for pair in centres.windows(2) {
            total += crate::settings::norm(crate::settings::sub(pair[1], pair[0]));
        }
        let start = crate::settings::add(
            centres[0],
            crate::settings::mul(tangent, params.spacing_mm),
        );
        let usable = (total - params.spacing_mm * 2.0).max(0.0);
        for index in 0..params.count {
            let t = if params.count == 1 {
                0.0
            } else {
                index as f64 / (params.count - 1) as f64
            };
            stations.push(crate::settings::add(
                start,
                crate::settings::mul(tangent, usable * t),
            ));
        }
    }
    if stations.is_empty() {
        return Err(Error::Invalid("microprong row"));
    }

    let shear = params.angle_deg.to_radians().tan();
    let mut cutters = Vec::with_capacity(stations.len());
    let mut volumes = Vec::with_capacity(stations.len());
    for station in stations {
        let rings = scallop_rings(params, girdle);
        let local = loft(&rings, params.segments)?;
        let placed = place(&local, &Transform::translation(station))?;
        // Lean the scallop along the row tangent when Angle is not zero.
        let mesh = if shear.abs() > 1.0e-9 {
            let z_bottom = station[2] - params.depth_mm;
            let vertices = placed
                .vertices
                .iter()
                .map(|point| {
                    let lift = (point[2] - z_bottom) * shear;
                    [
                        point[0] + tangent[0] * lift,
                        point[1] + tangent[1] * lift,
                        point[2],
                    ]
                })
                .collect::<Vec<_>>();
            let sheared = Mesh {
                vertices,
                faces: placed.faces.clone(),
                normals: Vec::new(),
                colors: placed.colors.clone(),
                corner_uvs: placed.corner_uvs.clone(),
            };
            crate::settings::inspect_solid(&sheared)?;
            sheared
        } else {
            placed
        };
        let volume = volume_mm3(&mesh)?;
        volumes.push(volume);
        cutters.push(mesh);
    }
    let total: f64 = volumes.iter().sum();
    let summary = format!(
        "Micro Prong Cutter {:.2} mm × {} on {} stone{} · {:.3} mm³",
        params.radius_mm,
        cutters.len(),
        count,
        if count == 1 { "" } else { "s" },
        total
    );
    Ok(MicroProngCutter {
        gem_ids: stones.iter().map(|(id, _, _)| id.clone()).collect(),
        cutters,
        volumes_mm3: volumes,
        total_volume_mm3: total,
        summary,
    })
}

/// The rounded scallop rings: a bottom pole, a curved wall and the top ring at the girdle plane.
fn scallop_rings(params: &MicroProngParams, girdle_half: f64) -> Vec<Ring> {
    let steps = 5;
    let mut rings = Vec::with_capacity(steps + 1);
    let top = girdle_half + params.height_mm;
    let bottom = -params.depth_mm;
    for step in 0..=steps {
        let t = step as f64 / steps as f64;
        let z = bottom + (top - bottom) * t;
        let radius = params.radius_mm * (1.0 - (1.0 - t) * (1.0 - t)).sqrt();
        rings.push(Ring::round(z, radius));
    }
    rings
}
