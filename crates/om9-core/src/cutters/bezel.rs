// SPDX-License-Identifier: LGPL-2.1-or-later
//! Bezel Cutter Builder (`gvBezelCutter`, `OM9-L3-04-16-004`, manual book 2 pages 286-288).
//!
//! The Cutters chapter describes the same `gvBezelCutter` command the Settings chapter does,
//! but with the *dimensional* controls: Height and Length in millimetres from the stone centre,
//! a 2D scale, the overall Cutter Length, the X/Y Inside and Outside sizes of the tapered block,
//! a Cutter Angle, X/Y offsets, the Number of Cutters and the Even/Odd position.
//!
//! This owner publishes that parameter contract and the block geometry. It does **not** own a
//! second scallop service: the session that dispatches the command keeps the one `om9.settings.
//! scallop` record, and the generated bodies are M07 cutters (`om9.cutter`).
//!
//! Geometry per cutter: a tapered block between the inner end (closest to the stone) and the
//! outer end, placed on the girdle outline at one of `count` positions. The inner end is at
//! `length - cutter_length` from the axis and the outer end at `length`; X sizes are the block's
//! tangential widths, Y sizes its axial heights. A positive Cutter Angle lifts the outer end, so
//! the block leans away from the stone.
use crate::{
    Error, Result,
    cutters::{MAX_CUTTERS, MIN_CUTTER_MM, volume_mm3},
    geometry::{Mesh, Point3},
    settings::{
        add, cross, head::GemGirdle, mul, unit,
    },
};

/// Where the first cutter sits around the girdle (manual: "Position Even/Odd").
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CutterPosition {
    /// A cutter sits at the top and bottom of the stone.
    Odd,
    /// The cutters are offset by half a step from the top and bottom.
    Even,
}

impl CutterPosition {
    pub const ALL: [Self; 2] = [Self::Odd, Self::Even];

    pub fn label(self) -> &'static str {
        match self {
            Self::Odd => "Odd",
            Self::Even => "Even",
        }
    }

    pub fn from_tag(tag: &str) -> Result<Self> {
        match tag.trim().to_ascii_lowercase().as_str() {
            "odd" => Ok(Self::Odd),
            "even" => Ok(Self::Even),
            _ => Err(Error::Invalid("bezel cutter position")),
        }
    }

    /// Start fraction of the first cutter around the outline.
    pub fn start_fraction(self) -> f64 {
        match self {
            Self::Odd => 0.0,
            Self::Even => 0.5,
        }
    }
}

/// The dimensional Bezel Cutter controls of the Cutters chapter.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BezelCutterDims {
    /// Height of the cutter above the stone, mm (the block's inner-end bottom).
    pub height_mm: f64,
    /// Distance from the stone centre to the cutter's outer end, mm.
    pub length_mm: f64,
    /// 2D scale on X and Y: `100` keeps the sizes as typed.
    pub scale_2d_percent: f64,
    /// Overall radial length of the cutter, mm.
    pub cutter_length_mm: f64,
    /// Tangential width of the end closest to the stone, mm.
    pub x_inside_mm: f64,
    /// Tangential width of the end farthest from the stone, mm.
    pub x_outside_mm: f64,
    /// Axial height of the end closest to the stone, mm.
    pub y_inside_mm: f64,
    /// Axial height of the end farthest from the stone, mm.
    pub y_outside_mm: f64,
    /// Tilt of the cutter away from the radial direction, degrees.
    pub cutter_angle_deg: f64,
    /// Tangential offset of the whole block, mm.
    pub x_offset_mm: f64,
    /// Radial offset of the whole block away from the stone, mm.
    pub y_offset_mm: f64,
    /// Number of cutters around the stone.
    pub count: u8,
    pub position: CutterPosition,
    /// Axial shift of the whole pattern, mm.
    pub z_offset_mm: f64,
}

impl Default for BezelCutterDims {
    fn default() -> Self {
        Self {
            height_mm: 0.20,
            length_mm: 4.20,
            scale_2d_percent: 100.0,
            cutter_length_mm: 1.20,
            x_inside_mm: 1.10,
            x_outside_mm: 1.60,
            y_inside_mm: 0.80,
            y_outside_mm: 1.20,
            cutter_angle_deg: 12.0,
            x_offset_mm: 0.0,
            y_offset_mm: 0.0,
            count: 12,
            position: CutterPosition::Odd,
            z_offset_mm: 0.0,
        }
    }
}

impl BezelCutterDims {
    pub fn validate(&self) -> Result<()> {
        for (value, name) in [
            (self.height_mm, "bezel cutter height"),
            (self.length_mm, "bezel cutter length"),
            (self.scale_2d_percent, "bezel cutter 2d scale"),
            (self.cutter_length_mm, "bezel cutter cutter length"),
            (self.x_inside_mm, "bezel cutter x inside"),
            (self.x_outside_mm, "bezel cutter x outside"),
            (self.y_inside_mm, "bezel cutter y inside"),
            (self.y_outside_mm, "bezel cutter y outside"),
            (self.cutter_angle_deg, "bezel cutter angle"),
            (self.x_offset_mm, "bezel cutter x offset"),
            (self.y_offset_mm, "bezel cutter y offset"),
            (self.z_offset_mm, "bezel cutter z offset"),
        ] {
            if !value.is_finite() {
                return Err(Error::Invalid(name));
            }
        }
        if !(3..=(MAX_CUTTERS.min(u8::MAX as usize) as u8)).contains(&self.count) {
            return Err(Error::Invalid("bezel cutter count"));
        }
        if !(10.0..=400.0).contains(&self.scale_2d_percent) {
            return Err(Error::Invalid("bezel cutter 2d scale"));
        }
        if self.cutter_length_mm < MIN_CUTTER_MM {
            return Err(Error::Invalid("bezel cutter cutter length"));
        }
        if self.length_mm <= self.cutter_length_mm {
            return Err(Error::Invalid("bezel cutter length"));
        }
        for size in [
            self.x_inside_mm,
            self.x_outside_mm,
            self.y_inside_mm,
            self.y_outside_mm,
        ] {
            if size < MIN_CUTTER_MM {
                return Err(Error::Invalid("bezel cutter end size"));
            }
        }
        if !(-85.0..=85.0).contains(&self.cutter_angle_deg) {
            return Err(Error::Invalid("bezel cutter angle"));
        }
        if !(-20.0..=20.0).contains(&self.z_offset_mm) {
            return Err(Error::Invalid("bezel cutter z offset"));
        }
        Ok(())
    }

    /// The four (x, y) corners of the inner and outer ends in the block's local frame: tangential
    /// `t` and axial `n`, both in mm.
    pub fn ends(&self) -> ((f64, f64, f64, f64), (f64, f64, f64, f64)) {
        let scale = self.scale_2d_percent / 100.0;
        let inner_radius = (self.length_mm - self.cutter_length_mm - self.y_offset_mm).max(0.0);
        let outer_radius = (self.length_mm + self.y_offset_mm).max(inner_radius + MIN_CUTTER_MM);
        let x_in = self.x_inside_mm * scale;
        let x_out = self.x_outside_mm * scale;
        let y_in = self.y_inside_mm * scale;
        let y_out = self.y_outside_mm * scale;
        let rise = self.cutter_length_mm * self.cutter_angle_deg.to_radians().tan();
        let base = self.height_mm + self.z_offset_mm;
        (
            (inner_radius, x_in, base, y_in),
            (outer_radius, x_out, base + rise, y_out),
        )
    }
}

/// The built bezel cutters.
#[derive(Debug, Clone, PartialEq)]
pub struct BezelCutterSet {
    pub gem_id: String,
    pub cutters: Vec<Mesh>,
    pub volumes_mm3: Vec<f64>,
    pub total_volume_mm3: f64,
    pub summary: String,
}

impl BezelCutterSet {
    pub fn count(&self) -> usize {
        self.cutters.len()
    }

    pub fn is_cutter(&self) -> bool {
        true
    }
}

/// Build the dimensional bezel cutters around the gem's girdle.
pub fn build(girdle: &GemGirdle, dims: &BezelCutterDims) -> Result<BezelCutterSet> {
    girdle.validate()?;
    dims.validate()?;
    let ((r0, x_in, n0, y_in), (r1, x_out, n1, y_out)) = dims.ends();
    let count = dims.count as usize;
    let mut cutters = Vec::with_capacity(count);
    let mut volumes = Vec::with_capacity(count);
    for index in 0..count {
        let fraction = index as f64 / count as f64 + dims.position.start_fraction() / count as f64;
        let point = girdle.point_at_fraction(fraction)?;
        let outward = girdle.outward_at(point)?;
        let tangent = unit(cross(girdle.normal, outward))?;
        let centre = add(point, mul(outward, dims.y_offset_mm));
        let place = |radius: f64, tangent_offset: f64, axial: f64| -> Point3 {
            add(
                add(
                    add(centre, mul(outward, radius)),
                    mul(tangent, tangent_offset + dims.x_offset_mm),
                ),
                mul(girdle.normal, axial),
            )
        };
        // Two rectangles (inner and outer end) joined into one six-sided block.
        // The canonical box order and face list: `-x` is the inner end, `+x` the outer end,
        // `y` is tangential and `z` axial, so every face winds outward in one right-handed
        // frame before the placement maps it onto the girdle.
        let corners = vec![
            place(r0, -x_in * 0.5, n0),
            place(r1, -x_out * 0.5, n1),
            place(r1, x_out * 0.5, n1),
            place(r0, x_in * 0.5, n0),
            place(r0, -x_in * 0.5, n0 + y_in),
            place(r1, -x_out * 0.5, n1 + y_out),
            place(r1, x_out * 0.5, n1 + y_out),
            place(r0, x_in * 0.5, n0 + y_in),
        ];
        let faces = vec![
            vec![0, 3, 2, 1], // axial low
            vec![4, 5, 6, 7], // axial high
            vec![0, 1, 5, 4], // tangential low
            vec![1, 2, 6, 5], // outer end (+r)
            vec![2, 3, 7, 6], // tangential high
            vec![3, 0, 4, 7], // inner end (-r)
        ];
        let mesh = crate::cutters::solid(corners.to_vec(), faces)?;
        let volume = volume_mm3(&mesh)?;
        if volume <= 1.0e-9 {
            return Err(Error::Invalid("bezel cutter volume"));
        }
        cutters.push(mesh);
        volumes.push(volume);
    }
    let total = volumes.iter().sum::<f64>();
    let summary = format!(
        "Bezel cutters: {} x {:.2} mm at {:.2} mm from the stone ({} position)",
        cutters.len(),
        dims.cutter_length_mm,
        dims.length_mm,
        dims.position.label()
    );
    Ok(BezelCutterSet {
        gem_id: girdle.gem_id.clone(),
        cutters,
        volumes_mm3: volumes,
        total_volume_mm3: total,
        summary,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_gem() -> GemGirdle {
        GemGirdle::round("gem:b1", [0.0, 0.0, 0.0], 6.5, [0.0, 0.0, 1.0]).expect("girdle")
    }

    #[test]
    fn the_dimensional_builder_makes_one_closed_block_per_cutter() {
        let girdle = round_gem();
        let dims = BezelCutterDims::default();
        let set = build(&girdle, &dims).expect("bezel cutters");
        assert_eq!(set.count(), dims.count as usize);
        for cutter in &set.cutters {
            assert!(crate::settings::inspect_solid(cutter).expect("closed").closed);
        }
        assert!(set.total_volume_mm3 > 0.0);
        assert!(set.summary.contains("Odd"));
    }

    #[test]
    fn odd_and_even_positions_start_at_different_fractions() {
        assert!((CutterPosition::Odd.start_fraction() - 0.0).abs() < 1.0e-12);
        assert!((CutterPosition::Even.start_fraction() - 0.5).abs() < 1.0e-12);
    }

    #[test]
    fn the_2d_scale_grows_the_block() {
        let girdle = round_gem();
        let plain = build(&girdle, &BezelCutterDims::default()).expect("plain");
        let mut dims = BezelCutterDims::default();
        dims.scale_2d_percent = 150.0;
        let scaled = build(&girdle, &dims).expect("scaled");
        assert!(scaled.total_volume_mm3 > plain.total_volume_mm3 * 1.5);
    }

    #[test]
    fn an_impossible_block_is_refused() {
        let girdle = round_gem();
        let mut dims = BezelCutterDims::default();
        dims.length_mm = 1.0;
        dims.cutter_length_mm = 2.0;
        assert_eq!(
            build(&girdle, &dims).err(),
            Some(Error::Invalid("bezel cutter length"))
        );
        let mut dims = BezelCutterDims::default();
        dims.count = 0;
        assert_eq!(
            build(&girdle, &dims).err(),
            Some(Error::Invalid("bezel cutter count"))
        );
    }
}
