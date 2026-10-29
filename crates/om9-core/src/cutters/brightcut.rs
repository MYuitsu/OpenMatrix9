// SPDX-License-Identifier: LGPL-2.1-or-later
//! Bright cutter: the raised bright-cut facets around a stone (`BuilderBrightCutter`) and along a
//! stone row (`BuilderBrightCutChannel`).
//!
//! The manual gives the single-stone builder three controls — Height, Length and 2D Scale — plus
//! the Count and Angle of the facet row. A facet is one closed block: its inner edge sits on the
//! girdle, its outer edge is `Length` away, and the top face slopes from the inner height down to
//! the outer height by `Angle`, which is the bright cut itself.
//!
//! The channel variant places the same facets along an ordered stone row, on one side or, with
//! `Mirror=on`, on both sides of the channel.

use crate::{
    Error, Result,
    cutters::{MAX_CUTTERS, MIN_CUTTER_MM, place, solid, volume_mm3},
    gem::GemSpec,
    geometry::{Mesh, Point3, Transform},
};

/// Which bright cutter is being built.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BrightMode {
    /// One ring of facets around a single stone.
    Single,
    /// One row of facets along an ordered stone row.
    Channel,
}

impl BrightMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Single => "Single",
            Self::Channel => "Channel",
        }
    }
}

/// The typed parameters of the bright cutter.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BrightParams {
    /// How far the facet rises above the girdle plane, mm.
    pub height_mm: f64,
    /// How far the facet reaches outward from the girdle, mm.
    pub length_mm: f64,
    /// The 2D scale of Height and Length, percent.
    pub scale_2d_percent: f64,
    /// The number of facets around the stone (Single) or the number of facets along a side.
    pub count: usize,
    /// The slope of the bright cut, degrees.
    pub angle_deg: f64,
    /// How far the facet sinks below the girdle plane, mm.
    pub depth_mm: f64,
    /// Mirror the row: build the facets on both sides of the channel.
    pub mirror: bool,
    /// Ring segments of the inner edge (the facet is a block, so this only shapes its inner arc).
    pub segments: usize,
}

impl Default for BrightParams {
    fn default() -> Self {
        Self {
            height_mm: 0.30,
            length_mm: 1.20,
            scale_2d_percent: 100.0,
            count: 6,
            angle_deg: 45.0,
            depth_mm: 0.25,
            mirror: true,
            segments: 6,
        }
    }
}

impl BrightParams {
    /// Defaults measured from the stone: the facet reaches 18% of the stone width outward.
    pub fn for_gem(spec: &GemSpec) -> Result<Self> {
        let width = spec.width_mm.max(spec.length_mm);
        if width <= 0.0 {
            return Err(Error::Invalid("bright cutter gem"));
        }
        let params = Self {
            length_mm: (width * 0.18).max(MIN_CUTTER_MM * 4.0),
            ..Self::default()
        };
        params.validate()?;
        Ok(params)
    }

    /// The 2D scale as a factor.
    pub fn scale(&self) -> f64 {
        self.scale_2d_percent / 100.0
    }

    pub fn validate(&self) -> Result<()> {
        let values = [
            self.height_mm,
            self.length_mm,
            self.scale_2d_percent,
            self.angle_deg,
            self.depth_mm,
        ];
        if values.iter().any(|value| !value.is_finite()) {
            return Err(Error::Invalid("bright cutter size"));
        }
        if self.height_mm < 0.0 || self.depth_mm < MIN_CUTTER_MM {
            return Err(Error::Invalid("bright cutter height"));
        }
        if self.length_mm < MIN_CUTTER_MM * 2.0 {
            return Err(Error::Invalid("bright cutter length"));
        }
        if !(10.0..=400.0).contains(&self.scale_2d_percent) {
            return Err(Error::Invalid("bright cutter 2d scale"));
        }
        if !(3..=MAX_CUTTERS).contains(&self.count) {
            return Err(Error::Invalid("bright cutter count"));
        }
        if self.angle_deg.abs() > 85.0 {
            return Err(Error::Invalid("bright cutter angle"));
        }
        if !(3..=128).contains(&self.segments) {
            return Err(Error::Invalid("bright cutter segments"));
        }
        Ok(())
    }

    /// The top-face drop across one facet, mm.
    pub fn slope_mm(&self) -> f64 {
        self.length_mm * self.angle_deg.to_radians().tan()
    }
}

/// The built bright cutter in world space.
#[derive(Debug, Clone, PartialEq)]
pub struct BrightCutter {
    pub mode: BrightMode,
    /// The stones the cutter was built for, in row order (stable provenance).
    pub gem_ids: Vec<String>,
    /// One closed body per facet.
    pub bodies: Vec<Mesh>,
    pub volumes_mm3: Vec<f64>,
    pub total_volume_mm3: f64,
    pub summary: String,
}

impl BrightCutter {
    pub fn count(&self) -> usize {
        self.bodies.len()
    }

    pub fn is_cutter(&self) -> bool {
        true
    }
}

/// Build the bright cutter around one stone.
pub fn build_single(
    spec: &GemSpec,
    placement: &Transform,
    gem_id: &str,
    params: &BrightParams,
) -> Result<BrightCutter> {
    params.validate()?;
    placement.validate()?;
    let radius = spec.length_mm.max(spec.width_mm) * 0.5;
    if radius < MIN_CUTTER_MM {
        return Err(Error::Invalid("bright cutter gem"));
    }
    let r0 = radius + 0.02;
    let scale = params.scale();
    let r1 = r0 + params.length_mm * scale;
    let half_width = (std::f64::consts::TAU * r0 / params.count as f64) * 0.34;
    if half_width < MIN_CUTTER_MM {
        return Err(Error::Invalid("bright cutter count"));
    }
    let girdle_half = spec.girdle_mm * 0.5;
    let z_inner = girdle_half + params.height_mm * scale;
    let z_outer = z_inner - params.slope_mm() * scale;
    // The facet sinks below the girdle by Depth and, when the bright angle is steep, reaches
    // deeper still: the cutter has to contain its own facet.
    let z_bottom = (z_outer - MIN_CUTTER_MM).min(-params.depth_mm);
    let mut bodies = Vec::with_capacity(params.count);
    let mut volumes = Vec::with_capacity(params.count);
    for index in 0..params.count {
        let theta = std::f64::consts::TAU * index as f64 / params.count as f64;
        let (sin, cos) = theta.sin_cos();
        let radial = [cos, sin, 0.0];
        let tangential = [-sin, cos, 0.0];
        let local = block(
            [0.0, 0.0, 0.0],
            radial,
            tangential,
            r0,
            r1,
            half_width,
            z_bottom,
            z_inner,
            z_outer,
        )?;
        let mesh = place(&local, placement)?;
        volumes.push(volume_mm3(&mesh)?);
        bodies.push(mesh);
    }
    let total = volumes.iter().sum();
    let summary = format!(
        "Bright Cutter {:.2} mm × {} on {gem_id} · {:.3} mm³",
        params.length_mm * scale,
        bodies.len(),
        total
    );
    Ok(BrightCutter {
        mode: BrightMode::Single,
        gem_ids: vec![gem_id.to_owned()],
        bodies,
        volumes_mm3: volumes,
        total_volume_mm3: total,
        summary,
    })
}

/// Build the bright-cut channel along an ordered stone row.
pub fn build_channel(
    stones: &[(String, GemSpec, Transform)],
    params: &BrightParams,
) -> Result<BrightCutter> {
    if stones.is_empty() {
        return Err(Error::Missing("bright cutter gems"));
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
    let mut across = crate::settings::cross([0.0, 0.0, 1.0], tangent);
    if crate::settings::norm(across) < 1.0e-9 {
        across = [1.0, 0.0, 0.0];
    }
    let across = crate::settings::unit(across)?;
    let sides: &[f64] = if params.mirror { &[1.0, -1.0] } else { &[1.0] };
    let scale = params.scale();
    let mut bodies = Vec::with_capacity(count * sides.len());
    let mut volumes = Vec::with_capacity(count * sides.len());
    for (index, (_, spec, _)) in stones.iter().enumerate() {
        let centre = centres[index];
        let radius = spec.length_mm.max(spec.width_mm) * 0.5;
        let r0 = radius + 0.02;
        let r1 = r0 + params.length_mm * scale;
        let half_width = (spec.length_mm * 0.5).max(MIN_CUTTER_MM);
        let girdle_half = spec.girdle_mm * 0.5;
        let z_inner = girdle_half + params.height_mm * scale;
        let z_outer = z_inner - params.slope_mm() * scale;
        let z_bottom = (z_outer - MIN_CUTTER_MM).min(-params.depth_mm);
        for side in sides {
            let radial = crate::settings::mul(across, *side);
            let mesh = block(
                centre,
                radial,
                tangent,
                r0,
                r1,
                half_width,
                centre[2] + z_bottom,
                centre[2] + z_inner,
                centre[2] + z_outer,
            )?;
            volumes.push(volume_mm3(&mesh)?);
            bodies.push(mesh);
        }
    }
    let total = volumes.iter().sum();
    let summary = format!(
        "Bright Cut Channel {:.2} mm × {} on {} stone{} · {:.3} mm³",
        params.length_mm * scale,
        bodies.len(),
        count,
        if count == 1 { "" } else { "s" },
        total
    );
    Ok(BrightCutter {
        mode: BrightMode::Channel,
        gem_ids: stones.iter().map(|(id, _, _)| id.clone()).collect(),
        bodies,
        volumes_mm3: volumes,
        total_volume_mm3: total,
        summary,
    })
}

/// One closed bright-cut facet block.
#[allow(clippy::too_many_arguments)]
fn block(
    centre: Point3,
    radial: Point3,
    tangential: Point3,
    r0: f64,
    r1: f64,
    half_width: f64,
    z_bottom: f64,
    z_inner: f64,
    z_outer: f64,
) -> Result<Mesh> {
    let at = |radius: f64, side: f64, z: f64| -> Point3 {
        [
            centre[0] + radial[0] * radius + tangential[0] * side * half_width,
            centre[1] + radial[1] * radius + tangential[1] * side * half_width,
            z,
        ]
    };
    let vertices = vec![
        at(r0, -1.0, z_bottom),
        at(r1, -1.0, z_bottom),
        at(r1, 1.0, z_bottom),
        at(r0, 1.0, z_bottom),
        at(r0, -1.0, z_inner),
        at(r1, -1.0, z_outer),
        at(r1, 1.0, z_outer),
        at(r0, 1.0, z_inner),
    ];
    let faces = vec![
        vec![0, 3, 2, 1],
        vec![4, 5, 6, 7],
        vec![0, 1, 5, 4],
        vec![1, 2, 6, 5],
        vec![2, 3, 7, 6],
        vec![3, 0, 4, 7],
    ];
    solid(vertices, faces)
}
