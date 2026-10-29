// SPDX-License-Identifier: LGPL-2.1-or-later
//! Metal from Gems (`OM9-L3-04-15-004`): a metal profile swept along the shared gem row.
//!
//! The sheet defines six open-row modes (Outside, Inside, Curve, Top, Round Corners, Pinched
//! Ends, Outside Center) and three closed-loop modes (Closed Outside/Inside/Top). This module
//! models the modes that are a swept section of the row envelope; `Curve` mode orients objects
//! to a picked curve and is a typed [`Error::Unsupported`] here instead of silently building a
//! different body.
//!
//! Every mode maps to one closed **section polygon** in the plane normal to the row
//! (`side`, `normal`), swept by the shared sweep helper. Two-sided modes emit two bodies;
//! merging them is the kernel owner's Boolean job, so the result keeps them separate and
//! reports both volumes. The builder never mixes a cutter (`Top`) into the metal body: the
//! mode records `is_cutter` and the caller decides which output slot it lands in.
use crate::{
    Error, Result,
    budget::Limits,
    curve_ops,
    geometry::{Curve, Mesh, Point3},
    settings::{
        MAX_INSTANCES, Station, add, cross, dot, inspect_solid, mul, norm, sub, unit,
    },
    settings::{channel_border::ChannelBorder, gems::GemRow},
};

/// One Metal from Gems mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetalPieceMode {
    Outside,
    Inside,
    Curve,
    Top,
    RoundCorners,
    PinchedEnds,
    OutsideCenter,
    ClosedOutside,
    ClosedInside,
    ClosedTop,
}

impl MetalPieceMode {
    pub const OPEN: [Self; 7] = [
        Self::Outside,
        Self::Inside,
        Self::Curve,
        Self::Top,
        Self::RoundCorners,
        Self::PinchedEnds,
        Self::OutsideCenter,
    ];
    pub const CLOSED: [Self; 3] = [Self::ClosedOutside, Self::ClosedInside, Self::ClosedTop];

    pub fn name(self) -> &'static str {
        match self {
            Self::Outside => "Outside",
            Self::Inside => "Inside",
            Self::Curve => "Curve",
            Self::Top => "Top",
            Self::RoundCorners => "Round Corners",
            Self::PinchedEnds => "Pinched Ends",
            Self::OutsideCenter => "Outside Center",
            Self::ClosedOutside => "Closed Outside",
            Self::ClosedInside => "Closed Inside",
            Self::ClosedTop => "Closed Top",
        }
    }

    pub fn from_name(text: &str) -> Option<Self> {
        let wanted = text.trim().to_ascii_lowercase().replace(['_', '-'], " ");
        let wanted = wanted.split_whitespace().collect::<Vec<_>>().join(" ");
        Self::OPEN
            .into_iter()
            .chain(Self::CLOSED)
            .find(|mode| mode.name().to_ascii_lowercase() == wanted)
    }

    /// Closed-loop modes are only valid on a closed row, and vice versa.
    pub fn wants_closed_row(self) -> bool {
        Self::CLOSED.contains(&self)
    }

    /// `Top` builds the channel cutter, not a metal body.
    pub fn is_cutter(self) -> bool {
        matches!(self, Self::Top | Self::ClosedTop)
    }
}

/// Typed parameters of one metal piece.
#[derive(Debug, Clone, PartialEq)]
pub struct MetalPieceParams {
    pub mode: MetalPieceMode,
    /// Offset of the profile from the girdle: positive moves away from the stone, negative
    /// overlaps the girdle.
    pub gem_offset_mm: f64,
    /// End-only offset, applied to the terminal stations of an open row.
    pub gem_end_offset_mm: f64,
    /// Side wall thickness.
    pub wall_mm: f64,
    /// Wall height above the girdle.
    pub height_mm: f64,
    /// Metal depth beneath the girdle.
    pub depth_mm: f64,
    /// Floor thickness of the Inside/Outside Center modes.
    pub floor_mm: f64,
    /// Width of the profile at a rounded/pinched end.
    pub profile_end_width_mm: f64,
    /// Build one body per side instead of a single body.
    pub mirror: bool,
    /// Thicken a sharp corner so the sweep does not pinch.
    pub tweak_corners: bool,
    /// The sheet's optional channel border input: build the walls on the committed border
    /// curves instead of the row offset. Resolved by [`build_with_border`].
    pub use_border: bool,
}

impl Default for MetalPieceParams {
    fn default() -> Self {
        Self {
            mode: MetalPieceMode::Outside,
            gem_offset_mm: 0.0,
            gem_end_offset_mm: 0.0,
            wall_mm: 0.8,
            height_mm: 1.0,
            depth_mm: 1.2,
            floor_mm: 0.6,
            profile_end_width_mm: 0.0,
            mirror: true,
            tweak_corners: false,
            use_border: false,
        }
    }
}

impl MetalPieceParams {
    pub fn validate(&self) -> Result<()> {
        if !self.gem_offset_mm.is_finite() || !(-5.0..=20.0).contains(&self.gem_offset_mm) {
            return Err(Error::Invalid("metal piece gem offset"));
        }
        if !self.gem_end_offset_mm.is_finite() || !(-20.0..=20.0).contains(&self.gem_end_offset_mm)
        {
            return Err(Error::Invalid("metal piece end offset"));
        }
        if !self.wall_mm.is_finite() || self.wall_mm <= 0.0 || self.wall_mm > 10.0 {
            return Err(Error::Invalid("metal piece wall"));
        }
        if !self.height_mm.is_finite() || !(0.0..=20.0).contains(&self.height_mm) {
            return Err(Error::Invalid("metal piece height"));
        }
        if !self.depth_mm.is_finite() || self.depth_mm < 0.0 || self.depth_mm > 20.0 {
            return Err(Error::Invalid("metal piece depth"));
        }
        if !self.floor_mm.is_finite() || self.floor_mm < 0.0 {
            return Err(Error::Invalid("metal piece floor"));
        }
        if self.floor_mm >= self.depth_mm && !matches!(self.mode, MetalPieceMode::Outside) {
            return Err(Error::Invalid("metal piece floor"));
        }
        if !self.profile_end_width_mm.is_finite()
            || !(0.0..=self.wall_mm).contains(&self.profile_end_width_mm)
        {
            return Err(Error::Invalid("metal piece end width"));
        }
        Ok(())
    }
}

/// A built metal piece.
#[derive(Debug, Clone, PartialEq)]
pub struct MetalPiece {
    pub mode: MetalPieceMode,
    pub bodies: Vec<Mesh>,
    pub volume_mm3: f64,
    pub is_cutter: bool,
    pub closed_row: bool,
    /// The walls were built on a channel border curve rather than the row offset.
    pub on_border: bool,
    /// The walls were built on a picked curve (the sheet's `Curve` mode, no border needed).
    pub on_curve: bool,
}

impl MetalPiece {
    pub fn summary(&self) -> String {
        format!(
            "{} {} bodies, {:.3} mm3{}{}",
            self.mode.name(),
            self.bodies.len(),
            self.volume_mm3,
            if self.is_cutter { " (cutter)" } else { "" },
            if self.on_border {
                " on channel border"
            } else if self.on_curve {
                " on picked curve"
            } else {
                ""
            }
        )
    }
}

/// The sweep stations of a row: one per stone, with the terminal stations pushed out by the
/// end offset for an open row.
fn stations(row: &GemRow, params: &MetalPieceParams) -> Result<Vec<Station>> {
    let mut frames: Vec<(Point3, Point3, Point3)> = Vec::with_capacity(row.len() + 2);
    for index in 0..row.len() {
        frames.push((
            row.stone(index)?.center,
            row.side_at(index)?,
            row.normal_at(index)?,
        ));
    }
    if !row.is_closed() && params.gem_end_offset_mm != 0.0 {
        let offset = params.gem_end_offset_mm;
        let first_tangent = row.tangent_at(0)?;
        frames.insert(
            0,
            (
                add(frames[0].0, mul(first_tangent, -offset)),
                frames[0].1,
                frames[0].2,
            ),
        );
        let last = frames.len() - 1;
        let last_tangent = row.tangent_at(row.len() - 1)?;
        frames.push((
            add(frames[last].0, mul(last_tangent, offset)),
            frames[last].1,
            frames[last].2,
        ));
    }
    if frames.len() > MAX_INSTANCES {
        return Err(Error::Budget("metal piece stations"));
    }
    frames
        .into_iter()
        .map(|(origin, side, normal)| Station::new(origin, side, normal))
        .collect()
}

/// The closed 2D section of a mode, in `(side, normal)` coordinates. A positive `side` is the
/// left of travel, so the outside wall is at `+half`.
fn section(mode: MetalPieceMode, params: &MetalPieceParams, gem_radius: f64) -> Vec<(f64, f64)> {
    let half = gem_radius + params.gem_offset_mm;
    let wall = params.wall_mm;
    let height = params.height_mm;
    let depth = params.depth_mm;
    let floor = params.floor_mm.min(depth * 0.9);
    match mode {
        MetalPieceMode::Outside
        | MetalPieceMode::Top
        | MetalPieceMode::RoundCorners
        | MetalPieceMode::PinchedEnds
        | MetalPieceMode::ClosedOutside
        | MetalPieceMode::ClosedTop => vec![
            (half, -depth),
            (half + wall, -depth),
            (half + wall, height),
            (half, height),
        ],
        MetalPieceMode::Inside | MetalPieceMode::ClosedInside => vec![
            (-(half + wall), -depth),
            (half + wall, -depth),
            (half + wall, 0.0),
            (half, 0.0),
            (half, -depth + floor),
            (-half, -depth + floor),
            (-half, 0.0),
            (-(half + wall), 0.0),
        ],
        MetalPieceMode::OutsideCenter => vec![
            (-(half + wall), -depth),
            (half + wall, -depth),
            (half + wall, height),
            (half, height),
            (half, -depth + floor),
            (-half, -depth + floor),
            (-half, height),
            (-(half + wall), height),
        ],
        // Curve mode is refused before a section is built.
        MetalPieceMode::Curve => Vec::new(),
    }
}

fn end_taper(mode: MetalPieceMode, params: &MetalPieceParams) -> f64 {
    match mode {
        MetalPieceMode::RoundCorners | MetalPieceMode::PinchedEnds => {
            (1.0 - params.profile_end_width_mm / params.wall_mm).clamp(0.05, 1.0)
        }
        _ => 1.0,
    }
}

/// The turn angle at a station, in degrees (0 for a straight row).
fn turn_deg(stations: &[Station], index: usize, closed: bool) -> f64 {
    let count = stations.len();
    if count < 3 {
        return 0.0;
    }
    let previous = if index == 0 {
        if closed {
            Some(count - 1)
        } else {
            None
        }
    } else {
        Some(index - 1)
    };
    let next = if index + 1 == count {
        if closed {
            Some(0)
        } else {
            None
        }
    } else {
        Some(index + 1)
    };
    let (Some(previous), Some(next)) = (previous, next) else {
        return 0.0;
    };
    let incoming = sub(stations[index].origin, stations[previous].origin);
    let outgoing = sub(stations[next].origin, stations[index].origin);
    let (Ok(incoming), Ok(outgoing)) = (unit(incoming), unit(outgoing)) else {
        return 0.0;
    };
    dot(incoming, outgoing).clamp(-1.0, 1.0).acos().to_degrees()
}

/// Build the metal piece of a row.
pub fn build(row: &GemRow, params: &MetalPieceParams) -> Result<MetalPiece> {
    params.validate()?;
    row.validate()?;
    if row.len() < 2 {
        return Err(Error::Invalid("metal piece gem row"));
    }
    if params.mode == MetalPieceMode::Curve {
        return Err(Error::Unsupported("metal piece curve mode"));
    }
    if row.is_closed() != params.mode.wants_closed_row() {
        return Err(Error::Invalid("metal piece mode on row shape"));
    }
    let gem_radius = row
        .stones
        .iter()
        .map(|stone| stone.radius_mm())
        .fold(0.0_f64, f64::max);
    let base = section(params.mode, params, gem_radius);
    if base.len() < 3 {
        return Err(Error::Invalid("metal piece section"));
    }
    let stations = stations(row, params)?;
    let taper = end_taper(params.mode, params);
    let mirrored = matches!(
        params.mode,
        MetalPieceMode::Outside
            | MetalPieceMode::Top
            | MetalPieceMode::RoundCorners
            | MetalPieceMode::PinchedEnds
            | MetalPieceMode::ClosedOutside
            | MetalPieceMode::ClosedTop
    );
    let sides: Vec<f64> = if mirrored && params.mirror {
        vec![1.0, -1.0]
    } else {
        vec![1.0]
    };
    let mut bodies = Vec::new();
    let mut volume = 0.0;
    for sign in sides {
        let mut shaped: Vec<Vec<(f64, f64)>> = Vec::with_capacity(stations.len());
        for index in 0..stations.len() {
            let mut section = base.clone();
            let mut corner = 1.0;
            if params.tweak_corners {
                let turn = turn_deg(&stations, index, row.is_closed());
                if turn > 15.0 {
                    // "the channel will get slightly larger at the corner to accommodate for
                    // pinching": at most a 10 % widening at a 90° corner.
                    corner = 1.0 + 0.1 * (turn / 90.0).min(1.0);
                }
            }
            if taper < 1.0 && !row.is_closed() {
                let nearest = index.min(stations.len() - 1 - index);
                let factor = match nearest {
                    0 => taper,
                    1 => (1.0 + taper) * 0.5,
                    _ => 1.0,
                };
                if params.mode == MetalPieceMode::RoundCorners && nearest == 0 {
                    let smooth = factor * factor * (3.0 - 2.0 * factor);
                    for point in &mut section {
                        point.1 *= smooth * 0.5 + 0.5;
                    }
                }
                for point in &mut section {
                    point.0 *= factor;
                }
            }
            for point in &mut section {
                point.0 *= sign * corner;
            }
            shaped.push(section);
        }
        let body = sweep_section_varying(&stations, &shaped, row.is_closed())?;
        let report = inspect_solid(&body)?;
        volume += report.volume_mm3;
        bodies.push(body);
    }
    Ok(MetalPiece {
        mode: params.mode,
        bodies,
        volume_mm3: volume,
        is_cutter: params.mode.is_cutter(),
        closed_row: row.is_closed(),
        on_border: false,
        on_curve: false,
    })
}

/// Build the metal piece so its walls follow a committed channel border instead of the
/// row offset.
///
/// The sheet's Metal Piece takes an optional channel border input; using the border the user
/// already sees keeps the metal and the shared border curve coincident. Each wall is the wall
/// section swept along its own border polyline, with the inner face exactly on the curve.
pub fn build_with_border(
    row: &GemRow,
    border: &ChannelBorder,
    params: &MetalPieceParams,
) -> Result<MetalPiece> {
    params.validate()?;
    row.validate()?;
    if row.len() < 2 {
        return Err(Error::Invalid("metal piece gem row"));
    }
    // `Curve` is exactly the border-driven build: the sheet orients the metal to a picked
    // curve, and the channel border is that curve here. Without a curve, `build` refuses it.
    if params.mode != MetalPieceMode::Curve
        && row.is_closed() != params.mode.wants_closed_row()
    {
        return Err(Error::Invalid("metal piece mode on row shape"));
    }
    if border.closed != row.is_closed() {
        return Err(Error::Invalid("metal piece border row shape"));
    }
    let normal = row.normal_at(0)?;
    let section = vec![
        (0.0, -params.depth_mm),
        (params.wall_mm, -params.depth_mm),
        (params.wall_mm, params.height_mm),
        (0.0, params.height_mm),
    ];
    let mut bodies = Vec::new();
    let mut volume = 0.0;
    for (sign, points) in [(1.0, &border.left_points), (-1.0, &border.right_points)] {
        let stations = border_stations(points, sign, normal, border.closed)?;
        let sections = vec![section.clone(); stations.len()];
        let body = sweep_section_varying(&stations, &sections, border.closed)?;
        volume += inspect_solid(&body)?.volume_mm3;
        bodies.push(body);
    }
    Ok(MetalPiece {
        mode: params.mode,
        bodies,
        volume_mm3: volume,
        is_cutter: params.mode.is_cutter(),
        closed_row: row.is_closed(),
        on_border: true,
        on_curve: false,
    })
}

/// Build the sheet's `Curve` mode on a picked curve instead of a committed border.
///
/// The sheet: "When you select this option you are prompted to select a curve on the target
/// surface. This restrains the orientation of the objects to both the surface and the curve."
/// The walls therefore follow the picked curve, oriented by the row normal, with the same
/// section as [`build_with_border`]. The curve is sampled by arc length so a long picked curve
/// gets one station per gem, not one per control point.
pub fn build_along_curve(
    row: &GemRow,
    curve: &Curve,
    params: &MetalPieceParams,
    limits: &Limits,
) -> Result<MetalPiece> {
    params.validate()?;
    row.validate()?;
    if row.len() < 2 {
        return Err(Error::Invalid("metal piece gem row"));
    }
    if params.mode != MetalPieceMode::Curve {
        return Err(Error::Invalid("metal piece curve mode option"));
    }
    let normal = row.normal_at(0)?;
    let closed = curve_ops::is_closed(curve, 1.0e-9, limits)?;
    let length = curve_ops::arc_length(curve, limits)?;
    if !length.is_finite() || length <= 1.0e-9 {
        return Err(Error::Invalid("metal piece curve length"));
    }
    let step = row
        .stones
        .iter()
        .map(|stone| stone.diameter_mm)
        .fold(0.0_f64, f64::max)
        .max(0.2);
    let count = ((length / step).ceil() as usize).clamp(2, MAX_INSTANCES);
    let section = vec![
        (0.0, -params.depth_mm),
        (params.wall_mm, -params.depth_mm),
        (params.wall_mm, params.height_mm),
        (0.0, params.height_mm),
    ];
    let mut left = Vec::with_capacity(count);
    let mut right = Vec::with_capacity(count);
    for index in 0..count {
        // A closed curve wraps, so its last sample must not repeat the first one.
        let target = if closed {
            length * index as f64 / count as f64
        } else {
            length * index as f64 / (count - 1) as f64
        };
        let fraction = curve_ops::parameter_at_length(curve, target, limits)?;
        let point = curve_ops::point_at(curve, fraction, limits)?;
        let tangent = curve_ops::tangent_at(curve, fraction, limits)?;
        let side = unit(cross(normal, unit(tangent)?))?;
        left.push(Station::new(point, side, normal)?);
        right.push(Station::new(point, mul(side, -1.0), normal)?);
    }
    let mut bodies = Vec::with_capacity(2);
    let mut volume = 0.0;
    for stations in [left, right] {
        let sections = vec![section.clone(); stations.len()];
        let body = sweep_section_varying(&stations, &sections, closed)?;
        volume += inspect_solid(&body)?.volume_mm3;
        bodies.push(body);
    }
    Ok(MetalPiece {
        mode: params.mode,
        bodies,
        volume_mm3: volume,
        is_cutter: false,
        closed_row: row.is_closed(),
        on_border: false,
        on_curve: true,
    })
}

/// Stations along a border polyline: origin on the curve, `x` pointing away from the stones.
fn border_stations(
    points: &[Point3],
    sign: f64,
    normal: Point3,
    closed: bool,
) -> Result<Vec<Station>> {
    if points.len() < 2 {
        return Err(Error::Invalid("metal piece border points"));
    }
    if points.len() > MAX_INSTANCES {
        return Err(Error::Budget("metal piece border stations"));
    }
    let mut stations = Vec::with_capacity(points.len());
    for (index, point) in points.iter().enumerate() {
        let previous = if index > 0 {
            Some(points[index - 1])
        } else if closed {
            points.last().copied()
        } else {
            None
        };
        let next = if index + 1 < points.len() {
            Some(points[index + 1])
        } else if closed {
            points.first().copied()
        } else {
            None
        };
        let tangent = match (previous, next) {
            (Some(previous), Some(next)) => sub(next, previous),
            (None, Some(next)) => sub(next, *point),
            (Some(previous), None) => sub(*point, previous),
            (None, None) => return Err(Error::Invalid("metal piece border tangent")),
        };
        let side = unit(cross(normal, unit(tangent)?))?;
        if !side.iter().all(|value| value.is_finite()) {
            return Err(Error::Invalid("metal piece border tangent"));
        }
        stations.push(Station::new(*point, mul(side, sign), normal)?);
    }
    Ok(stations)
}

/// Sweep a section that may change per station (end tapering). Same topology as
/// [`crate::settings::sweep_section`], with one section per station.
fn sweep_section_varying(
    stations: &[Station],
    sections: &[Vec<(f64, f64)>],
    closed_path: bool,
) -> Result<Mesh> {
    if stations.len() != sections.len() || stations.len() < 2 {
        return Err(Error::Invalid("metal piece sweep"));
    }
    let ring = sections[0].len();
    if ring < 3 || sections.iter().any(|section| section.len() != ring) {
        return Err(Error::Invalid("metal piece sweep section"));
    }
    let mut vertices: Vec<Point3> = Vec::with_capacity(stations.len() * ring);
    for (station, section) in stations.iter().zip(sections) {
        for point in section {
            vertices.push(add(
                station.origin,
                add(mul(station.x, point.0), mul(station.y, point.1)),
            ));
        }
    }
    let mut faces: Vec<Vec<u32>> = Vec::new();
    let spans = if closed_path {
        stations.len()
    } else {
        stations.len() - 1
    };
    for station in 0..spans {
        let next = (station + 1) % stations.len();
        for index in 0..ring {
            let following = (index + 1) % ring;
            faces.push(vec![
                (station * ring + index) as u32,
                (station * ring + following) as u32,
                (next * ring + following) as u32,
                (next * ring + index) as u32,
            ]);
        }
    }
    if !closed_path {
        for (station, forward) in [(0usize, false), (stations.len() - 1, true)] {
            let section = &sections[station];
            let centroid = section.iter().fold([0.0, 0.0], |acc, point| {
                [acc[0] + point.0, acc[1] + point.1]
            });
            let centroid = (centroid[0] / ring as f64, centroid[1] / ring as f64);
            let center = vertices.len() as u32;
            vertices.push(add(
                stations[station].origin,
                add(
                    mul(stations[station].x, centroid.0),
                    mul(stations[station].y, centroid.1),
                ),
            ));
            for index in 0..ring {
                let following = (index + 1) % ring;
                let a = (station * ring + index) as u32;
                let b = (station * ring + following) as u32;
                if forward {
                    faces.push(vec![center, a, b]);
                } else {
                    faces.push(vec![center, b, a]);
                }
            }
        }
    }
    let mut mesh = Mesh {
        vertices,
        faces,
        normals: Vec::new(),
        colors: Vec::new(),
        corner_uvs: Vec::new(),
    };
    if crate::settings::signed_volume(&mesh.vertices, &mesh.faces) < 0.0 {
        for face in &mut mesh.faces {
            face.reverse();
        }
    }
    Ok(mesh)
}

/// The wall offset from the row centre, exposed for the viewport handle position.
pub fn wall_offset_mm(row: &GemRow, index: usize, params: &MetalPieceParams) -> Result<f64> {
    let radius = row.stone(index)?.radius_mm();
    let half = radius + params.gem_offset_mm;
    Ok(half + params.wall_mm * 0.5)
}

/// The unit side direction of the wall at a stone.
pub fn wall_side(row: &GemRow, index: usize) -> Result<Point3> {
    unit(row.side_at(index)?)
}

/// Whether the row is continuous enough for a sweep: adjacent stones must not jump more than
/// 1.5 diameters, which the sheet calls a row discontinuity.
pub fn check_row_continuity(row: &GemRow) -> Result<()> {
    for (a, b) in row.pairs() {
        let left = row.stone(a)?;
        let right = row.stone(b)?;
        let distance = norm(sub(right.center, left.center));
        let limit = (left.diameter_mm + right.diameter_mm) * 1.5;
        if distance > limit {
            return Err(Error::Invalid("gem row discontinuity"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::budget::Limits;
    use crate::settings::channel_border::{ChannelBorderParams, build_border};
    use crate::settings::gems::{GemStone, RowShape, straight_row};

    fn open_row() -> GemRow {
        straight_row(&[2.0, 2.0, 2.0], 0.0, [0.0; 3], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]).unwrap()
    }

    fn closed_row() -> GemRow {
        let stones = (0..8)
            .map(|index| {
                let angle = std::f64::consts::TAU * index as f64 / 8.0;
                GemStone::round(
                    format!("g{index}"),
                    [10.0 * angle.cos(), 10.0 * angle.sin(), 0.0],
                    2.0,
                    [0.0, 0.0, 1.0],
                )
                .unwrap()
            })
            .collect();
        GemRow::new(stones, RowShape::Closed).unwrap()
    }

    #[test]
    fn every_open_mode_builds_a_closed_body_or_a_typed_refusal() {
        for mode in MetalPieceMode::OPEN {
            let params = MetalPieceParams {
                mode,
                ..MetalPieceParams::default()
            };
            if mode == MetalPieceMode::Curve {
                assert_eq!(
                    build(&open_row(), &params).err(),
                    Some(Error::Unsupported("metal piece curve mode"))
                );
                continue;
            }
            let piece = build(&open_row(), &params).unwrap();
            assert!(!piece.bodies.is_empty());
            assert!(piece.volume_mm3 > 0.0);
            for body in &piece.bodies {
                inspect_solid(body).unwrap();
            }
        }
    }

    #[test]
    fn curve_mode_follows_the_picked_curve() {
        let limits = Limits::default();
        let curve = crate::curve_ops::line_to_nurbs([0.0, 0.0, 0.0], [6.0, 0.0, 0.0], &limits)
            .expect("picked line");
        let params = MetalPieceParams {
            mode: MetalPieceMode::Curve,
            wall_mm: 0.8,
            ..MetalPieceParams::default()
        };
        let piece = build_along_curve(&open_row(), &curve, &params, &limits).expect("curve piece");
        assert!(piece.on_curve);
        assert!(!piece.on_border);
        assert_eq!(piece.mode, MetalPieceMode::Curve);
        assert_eq!(piece.bodies.len(), 2);
        assert!(piece.summary().contains("picked curve"), "{}", piece.summary());
        for (index, body) in piece.bodies.iter().enumerate() {
            assert!(inspect_solid(body).expect("closed wall").closed);
            // The wall follows the picked line: the inner face sits on it (y == 0) and the wall
            // body stays on its own side, spanning the sampled curve length.
            let side = if index == 0 { 1.0 } else { -1.0 };
            assert!(
                body.vertices.iter().all(|vertex| vertex[1] * side >= -1.0e-9),
                "body {index} crossed the picked curve"
            );
            assert!(body.vertices.iter().any(|vertex| vertex[1] * side > 0.0));
            assert!(body.vertices.iter().any(|vertex| vertex[0] >= 5.9));
        }
        // A mode that does not follow a curve cannot be built from one.
        let outside = MetalPieceParams::default();
        assert_eq!(
            build_along_curve(&open_row(), &curve, &outside, &limits).err(),
            Some(Error::Invalid("metal piece curve mode option"))
        );
    }

    #[test]
    fn outside_mirrors_into_two_bodies_and_inside_is_one_u_channel() {
        let outside = build(&open_row(), &MetalPieceParams::default()).unwrap();
        assert_eq!(outside.bodies.len(), 2);
        let inside = build(
            &open_row(),
            &MetalPieceParams {
                mode: MetalPieceMode::Inside,
                ..MetalPieceParams::default()
            },
        )
        .unwrap();
        assert_eq!(inside.bodies.len(), 1);
        assert!(!inside.is_cutter);
    }

    #[test]
    fn top_mode_is_a_cutter_and_is_never_reported_as_metal() {
        let piece = build(
            &open_row(),
            &MetalPieceParams {
                mode: MetalPieceMode::Top,
                ..MetalPieceParams::default()
            },
        )
        .unwrap();
        assert!(piece.is_cutter);
        assert!(piece.summary().contains("cutter"));
    }

    #[test]
    fn a_closed_mode_requires_a_closed_row() {
        assert_eq!(
            build(
                &open_row(),
                &MetalPieceParams {
                    mode: MetalPieceMode::ClosedOutside,
                    ..MetalPieceParams::default()
                }
            )
            .err(),
            Some(Error::Invalid("metal piece mode on row shape"))
        );
        assert_eq!(
            build(
                &closed_row(),
                &MetalPieceParams {
                    mode: MetalPieceMode::Outside,
                    ..MetalPieceParams::default()
                }
            )
            .err(),
            Some(Error::Invalid("metal piece mode on row shape"))
        );
        let piece = build(
            &closed_row(),
            &MetalPieceParams {
                mode: MetalPieceMode::ClosedOutside,
                ..MetalPieceParams::default()
            },
        )
        .unwrap();
        assert!(piece.closed_row);
        for body in &piece.bodies {
            inspect_solid(body).unwrap();
        }
    }

    #[test]
    fn a_border_driven_piece_follows_the_border_curve() {
        let row = open_row();
        let border = build_border(&row, &ChannelBorderParams::default(), &Limits::default())
            .expect("border");
        let params = MetalPieceParams::default();
        let piece = build_with_border(&row, &border, &params).expect("border piece");
        assert!(piece.on_border);
        assert_eq!(piece.bodies.len(), 2, "{}", piece.summary());
        let normal = row.normal_at(0).unwrap();
        for (points, body) in [(&border.left_points, &piece.bodies[0]), (&border.right_points, &piece.bodies[1])] {
            let report = inspect_solid(body).expect("closed wall");
            assert!(report.closed);
            assert!(report.volume_mm3 > 0.0);
            // The inner face sits exactly on the shared border curve: each border point has a
            // vertex one wall height above it and one metal depth below it.
            for point in points {
                let top = add(*point, mul(normal, params.height_mm));
                let bottom = add(*point, mul(normal, -params.depth_mm));
                assert!(
                    body.vertices.iter().any(|vertex| norm(sub(*vertex, top)) < 1.0e-9),
                    "missing top vertex at {point:?}"
                );
                assert!(
                    body.vertices.iter().any(|vertex| norm(sub(*vertex, bottom)) < 1.0e-9),
                    "missing bottom vertex at {point:?}"
                );
            }
        }
    }

    #[test]
    fn curve_mode_follows_the_border_it_is_given() {
        let row = open_row();
        let border = build_border(&row, &ChannelBorderParams::default(), &Limits::default())
            .expect("border");
        let params = MetalPieceParams {
            mode: MetalPieceMode::Curve,
            wall_mm: 0.8,
            ..MetalPieceParams::default()
        };
        // Without a picked curve the row-offset build still refuses the mode.
        assert_eq!(
            build(&row, &params).err(),
            Some(Error::Unsupported("metal piece curve mode"))
        );
        let piece = build_with_border(&row, &border, &params).expect("curve on border");
        assert!(piece.on_border);
        assert_eq!(piece.mode, MetalPieceMode::Curve);
        assert_eq!(piece.bodies.len(), 2, "{}", piece.summary());
        for body in &piece.bodies {
            assert!(inspect_solid(body).expect("closed wall").closed);
        }
    }

    #[test]
    fn a_border_for_another_row_shape_is_refused() {
        let row = open_row();
        let border = build_border(&row, &ChannelBorderParams::default(), &Limits::default())
            .expect("border");
        let closed = closed_row();
        let closed_params = MetalPieceParams {
            mode: MetalPieceMode::ClosedOutside,
            ..MetalPieceParams::default()
        };
        assert_eq!(
            build_with_border(&closed, &border, &closed_params).err(),
            Some(Error::Invalid("metal piece border row shape"))
        );
    }

    #[test]
    fn a_zero_wall_is_refused() {
        assert_eq!(
            MetalPieceParams {
                wall_mm: 0.0,
                ..MetalPieceParams::default()
            }
            .validate()
            .err(),
            Some(Error::Invalid("metal piece wall"))
        );
    }

    #[test]
    fn the_end_treatment_narrows_the_terminal_station() {
        let pinched = build(
            &open_row(),
            &MetalPieceParams {
                mode: MetalPieceMode::PinchedEnds,
                profile_end_width_mm: 0.4,
                ..MetalPieceParams::default()
            },
        )
        .unwrap();
        let plain = build(
            &open_row(),
            &MetalPieceParams {
                mode: MetalPieceMode::Outside,
                ..MetalPieceParams::default()
            },
        )
        .unwrap();
        assert!(pinched.volume_mm3 < plain.volume_mm3);
    }

    #[test]
    fn an_end_offset_extends_the_wall_past_the_last_stone() {
        let plain = build(&open_row(), &MetalPieceParams::default()).unwrap();
        let extended = build(
            &open_row(),
            &MetalPieceParams {
                gem_end_offset_mm: 1.0,
                ..MetalPieceParams::default()
            },
        )
        .unwrap();
        let extent = |mesh: &Mesh| {
            mesh.vertices
                .iter()
                .map(|point| point[0])
                .fold(f64::NEG_INFINITY, f64::max)
        };
        assert!(extent(&extended.bodies[0]) > extent(&plain.bodies[0]));
    }

    #[test]
    fn a_row_jump_is_a_typed_discontinuity() {
        let mut stones = open_row().stones;
        stones[2].center[0] = 50.0;
        let row = GemRow::open(stones).unwrap();
        assert_eq!(
            check_row_continuity(&row).err(),
            Some(Error::Invalid("gem row discontinuity"))
        );
    }

    #[test]
    fn the_wall_handle_sits_one_half_wall_outside_the_offset() {
        let row = open_row();
        let params = MetalPieceParams::default();
        let offset = wall_offset_mm(&row, 0, &params).unwrap();
        assert!((offset - (1.0 + params.wall_mm * 0.5)).abs() < 1.0e-9);
        assert!((norm(wall_side(&row, 0).unwrap()) - 1.0).abs() < 1.0e-9);
    }
}
