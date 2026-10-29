// SPDX-License-Identifier: LGPL-2.1-or-later
//! Channel Border Curve (`OM9-L3-04-15-005`): the offset guide curve around a gem group.
//!
//! The result is a **curve**, never a solid: the sheet calls it a guide curve and the contract
//! says "guide curve là curve có topology hợp lệ, không giả vờ là solid". Two open polylines
//! (left/right) or two closed loops are built from the shared gem row, offset sideways from the
//! girdle by `side_offset_mm`, raised by `v_offset_mm` and extended past the last stone by
//! `end_offset_mm` with the Line/Arc/Smooth extension type.
//!
//! Corner handling follows the two documented controls: `corner_angle` (0 smooth … 1 sharp) and
//! `tweak_corners` (a slightly larger channel at the corner), plus `snap_to_corners` for a
//! square input where the sharp vertex must survive.
use crate::{
    Error, Result,
    budget::Limits,
    geometry::{Curve, Point3},
    settings::{add, cross, dot, mul, norm, sub, unit},
    settings::gems::GemRow,
};

/// How a border extends past the end of an open gem row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelExtension {
    Line,
    Arc,
    Smooth,
}

impl ChannelExtension {
    pub fn name(self) -> &'static str {
        match self {
            Self::Line => "Line",
            Self::Arc => "Arc",
            Self::Smooth => "Smooth",
        }
    }

    pub fn from_name(text: &str) -> Option<Self> {
        let wanted = text.trim().to_ascii_lowercase();
        [Self::Line, Self::Arc, Self::Smooth]
            .into_iter()
            .find(|mode| mode.name().to_ascii_lowercase() == wanted)
    }
}

/// Typed parameters of one channel border.
#[derive(Debug, Clone, PartialEq)]
pub struct ChannelBorderParams {
    /// Distance from the girdle to the border, positive moves away from the stones.
    pub side_offset_mm: f64,
    /// How far the border runs past the end of an open row.
    pub end_offset_mm: f64,
    /// Vertical offset from the girdle plane, positive is up.
    pub v_offset_mm: f64,
    /// 0 = smooth corner, 1 = sharp corner.
    pub corner_angle: f64,
    pub extension: ChannelExtension,
    pub tweak_corners: bool,
    /// Keep a square input's sharp vertices instead of rounding them.
    pub snap_to_corners: bool,
    /// The sheet's Show Prongs option: the caller previews prongs through the prong owner.
    pub show_prongs: bool,
}

impl Default for ChannelBorderParams {
    fn default() -> Self {
        Self {
            side_offset_mm: 0.0,
            end_offset_mm: 0.0,
            v_offset_mm: 0.0,
            corner_angle: 1.0,
            extension: ChannelExtension::Line,
            tweak_corners: false,
            snap_to_corners: false,
            show_prongs: false,
        }
    }
}

impl ChannelBorderParams {
    pub fn validate(&self) -> Result<()> {
        if !self.side_offset_mm.is_finite() || !(-10.0..=10.0).contains(&self.side_offset_mm) {
            return Err(Error::Invalid("channel border side offset"));
        }
        if !self.end_offset_mm.is_finite() || !(0.0..=50.0).contains(&self.end_offset_mm) {
            return Err(Error::Invalid("channel border end offset"));
        }
        if !self.v_offset_mm.is_finite() || !(-10.0..=10.0).contains(&self.v_offset_mm) {
            return Err(Error::Invalid("channel border v offset"));
        }
        if !self.corner_angle.is_finite() || !(0.0..=1.0).contains(&self.corner_angle) {
            return Err(Error::Invalid("channel border corner angle"));
        }
        Ok(())
    }
}

/// A built channel border.
#[derive(Debug, Clone, PartialEq)]
pub struct ChannelBorder {
    pub left: Curve,
    pub right: Curve,
    pub left_points: Vec<Point3>,
    pub right_points: Vec<Point3>,
    pub closed: bool,
}

impl ChannelBorder {
    pub fn summary(&self) -> String {
        format!(
            "{} border, {} + {} points",
            if self.closed { "closed" } else { "open" },
            self.left_points.len(),
            self.right_points.len()
        )
    }
}

fn side_points(row: &GemRow, params: &ChannelBorderParams, sign: f64) -> Result<Vec<Point3>> {
    let mut points = Vec::with_capacity(row.len() + 2);
    for index in 0..row.len() {
        let stone = row.stone(index)?;
        let side = row.side_at(index)?;
        let normal = row.normal_at(index)?;
        let radius = stone.radius_mm() + params.side_offset_mm;
        if radius <= 1.0e-6 {
            return Err(Error::Invalid("channel border side offset"));
        }
        points.push(add(
            stone.center,
            add(mul(side, sign * radius), mul(normal, params.v_offset_mm)),
        ));
    }
    Ok(points)
}

/// Extend an open end with the selected extension type.
fn extend(
    points: &mut Vec<Point3>,
    sign: f64,
    normal: Point3,
    params: &ChannelBorderParams,
) -> Result<()> {
    if params.end_offset_mm <= 0.0 || points.len() < 2 {
        return Ok(());
    }
    let last = points[points.len() - 1];
    let previous = points[points.len() - 2];
    let outward = unit(sub(last, previous))?;
    // The border side direction is perpendicular to travel and to the girdle normal.
    let side = unit(cross(normal, outward))?;
    let distance = params.end_offset_mm;
    match params.extension {
        ChannelExtension::Line => points.push(add(last, mul(outward, distance))),
        ChannelExtension::Arc => {
            // Quarter arc of radius = end offset, bending toward the row.
            let center = add(last, mul(side, sign * distance));
            const STEPS: usize = 6;
            for step in 1..=STEPS {
                let angle = std::f64::consts::FRAC_PI_2 * step as f64 / STEPS as f64;
                let (sin, cos) = angle.sin_cos();
                points.push(add(
                    center,
                    add(
                        mul(side, -sign * distance * cos),
                        mul(outward, distance * sin),
                    ),
                ));
            }
        }
        ChannelExtension::Smooth => {
            // Continue the curvature of the last two segments. A straight row stays straight.
            let last_turn = points.len().checked_sub(3).map(|index| {
                let a = unit(sub(points[index + 1], points[index])).unwrap_or([1.0, 0.0, 0.0]);
                let b = unit(sub(points[index + 2], points[index + 1]))
                    .unwrap_or([1.0, 0.0, 0.0]);
                dot(a, b).clamp(-1.0, 1.0).acos()
            });
            match last_turn {
                Some(turn) if turn > 1.0e-3 => {
                    let segment = norm(sub(last, previous));
                    let radius = segment / turn;
                    let turn_sign = if dot(cross(outward, side), normal) >= 0.0 {
                        1.0
                    } else {
                        -1.0
                    };
                    const STEPS: usize = 6;
                    for step in 1..=STEPS {
                        let angle = (distance / radius).min(std::f64::consts::FRAC_PI_2)
                            * step as f64
                            / STEPS as f64;
                        let (sin, cos) = angle.sin_cos();
                        points.push(add(
                            last,
                            add(
                                mul(outward, radius * sin),
                                mul(side, turn_sign * sign * radius * (1.0 - cos)),
                            ),
                        ));
                    }
                }
                _ => points.push(add(last, mul(outward, distance))),
            }
        }
    }
    Ok(())
}

/// Round or sharpen the interior corners of a polyline.
fn shape_corners(points: &mut Vec<Point3>, params: &ChannelBorderParams, closed: bool) {
    if params.snap_to_corners || points.len() < 3 {
        return;
    }
    let sharpness = params.corner_angle.clamp(0.0, 1.0);
    if sharpness >= 0.999 && !params.tweak_corners {
        return;
    }
    let count = points.len();
    let mut shaped = Vec::with_capacity(count * 2);
    for index in 0..count {
        let interior = closed || (index > 0 && index + 1 < count);
        if !interior {
            shaped.push(points[index]);
            continue;
        }
        let previous = points[(index + count - 1) % count];
        let point = points[index];
        let next = points[(index + 1) % count];
        let to_previous = norm(sub(point, previous));
        let to_next = norm(sub(next, point));
        let cut = (1.0 - sharpness) * 0.25 * to_previous.min(to_next);
        if cut > 1.0e-9 {
            let a = add(point, mul(unit(sub(previous, point)).unwrap_or([0.0; 3]), cut));
            let b = add(point, mul(unit(sub(next, point)).unwrap_or([0.0; 3]), cut));
            shaped.push(a);
            shaped.push(b);
        } else {
            shaped.push(point);
        }
        if params.tweak_corners {
            // "the channel will get slightly larger at the corner": push the vertex out
            // along the bisector by a small fraction of the shorter segment.
            let incoming = unit(sub(point, previous)).unwrap_or([0.0; 3]);
            let outgoing = unit(sub(next, point)).unwrap_or([0.0; 3]);
            let bisector = unit(sub(outgoing, incoming));
            if let Ok(bisector) = bisector {
                let bump = 0.05 * to_previous.min(to_next);
                let moved = add(point, mul(bisector, bump));
                shaped.push(moved);
            }
        }
    }
    *points = shaped;
}

fn curve_from_points(points: &[Point3], closed: bool, limits: &Limits) -> Result<Curve> {
    if points.len() < 2 {
        return Err(Error::Invalid("channel border points"));
    }
    let mut segments = Vec::with_capacity(points.len());
    for index in 0..points.len() - 1 {
        segments.push(Curve::Line {
            start: points[index],
            end: points[index + 1],
            domain: [0.0, 1.0],
        });
    }
    if closed {
        segments.push(Curve::Line {
            start: points[points.len() - 1],
            end: points[0],
            domain: [0.0, 1.0],
        });
    }
    let curve = Curve::Polycurve {
        segments,
        reversed: false,
    };
    curve.validate(limits)?;
    Ok(curve)
}

/// Build both border curves for a gem row.
pub fn build_border(
    row: &GemRow,
    params: &ChannelBorderParams,
    limits: &Limits,
) -> Result<ChannelBorder> {
    params.validate()?;
    row.validate()?;
    if row.is_empty() {
        return Err(Error::Missing("channel border gem row"));
    }
    let normal = row.normal_at(0)?;
    let mut left_points = side_points(row, params, 1.0)?;
    let mut right_points = side_points(row, params, -1.0)?;
    if !row.is_closed() {
        // Extend both ends, mirroring the side sign so the border keeps its width.
        for (sign, points) in [(1.0, &mut left_points), (-1.0, &mut right_points)] {
            extend(points, sign, normal, params)?;
            points.reverse();
            extend(points, -sign, normal, params)?;
            points.reverse();
        }
    }
    shape_corners(&mut left_points, params, row.is_closed());
    shape_corners(&mut right_points, params, row.is_closed());
    let left = curve_from_points(&left_points, row.is_closed(), limits)?;
    let right = curve_from_points(&right_points, row.is_closed(), limits)?;
    Ok(ChannelBorder {
        left,
        right,
        left_points,
        right_points,
        closed: row.is_closed(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn the_border_is_a_valid_curve_not_a_solid() {
        let limits = Limits::default();
        let border = build_border(&open_row(), &ChannelBorderParams::default(), &limits).unwrap();
        border.left.validate(&limits).unwrap();
        border.right.validate(&limits).unwrap();
        assert!(matches!(border.left, Curve::Polycurve { .. }));
        assert_eq!(border.left_points.len(), 3);
    }

    #[test]
    fn the_side_offset_moves_the_border_off_the_girdle() {
        let limits = Limits::default();
        let plain = build_border(&open_row(), &ChannelBorderParams::default(), &limits).unwrap();
        let offset = build_border(
            &open_row(),
            &ChannelBorderParams {
                side_offset_mm: 0.5,
                ..ChannelBorderParams::default()
            },
            &limits,
        )
        .unwrap();
        assert!(offset.left_points[1][1] > plain.left_points[1][1]);
        assert!(offset.right_points[1][1] < plain.right_points[1][1]);
    }

    #[test]
    fn the_v_offset_lifts_the_border_off_the_girdle_plane() {
        let limits = Limits::default();
        let raised = build_border(
            &open_row(),
            &ChannelBorderParams {
                v_offset_mm: 0.25,
                ..ChannelBorderParams::default()
            },
            &limits,
        )
        .unwrap();
        assert!((raised.left_points[1][2] - 0.25).abs() < 1.0e-9);
    }

    #[test]
    fn a_line_extension_adds_one_point_and_an_arc_adds_several() {
        let limits = Limits::default();
        let line = build_border(
            &open_row(),
            &ChannelBorderParams {
                end_offset_mm: 1.0,
                extension: ChannelExtension::Line,
                ..ChannelBorderParams::default()
            },
            &limits,
        )
        .unwrap();
        let arc = build_border(
            &open_row(),
            &ChannelBorderParams {
                end_offset_mm: 1.0,
                extension: ChannelExtension::Arc,
                ..ChannelBorderParams::default()
            },
            &limits,
        )
        .unwrap();
        assert_eq!(line.left_points.len(), 5);
        assert!(arc.left_points.len() > line.left_points.len());
        line.left.validate(&limits).unwrap();
        arc.left.validate(&limits).unwrap();
    }

    #[test]
    fn a_closed_row_builds_two_closed_loops() {
        let limits = Limits::default();
        let border = build_border(&closed_row(), &ChannelBorderParams::default(), &limits).unwrap();
        assert!(border.closed);
        border.left.validate(&limits).unwrap();
        assert_eq!(border.left_points.len(), 8);
    }

    #[test]
    fn a_smooth_corner_replaces_the_sharp_vertex() {
        let limits = Limits::default();
        let row = open_row();
        let sharp = build_border(&row, &ChannelBorderParams::default(), &limits).unwrap();
        let smooth = build_border(
            &row,
            &ChannelBorderParams {
                corner_angle: 0.0,
                ..ChannelBorderParams::default()
            },
            &limits,
        )
        .unwrap();
        assert!(smooth.left_points.len() > sharp.left_points.len());
        smooth.left.validate(&limits).unwrap();
    }

    #[test]
    fn snap_to_corners_keeps_the_original_vertex_count() {
        let limits = Limits::default();
        let row = open_row();
        let border = build_border(
            &row,
            &ChannelBorderParams {
                corner_angle: 0.0,
                snap_to_corners: true,
                ..ChannelBorderParams::default()
            },
            &limits,
        )
        .unwrap();
        assert_eq!(border.left_points.len(), 3);
    }

    #[test]
    fn an_invalid_corner_angle_is_refused() {
        assert_eq!(
            ChannelBorderParams {
                corner_angle: 1.5,
                ..ChannelBorderParams::default()
            }
            .validate()
            .err(),
            Some(Error::Invalid("channel border corner angle"))
        );
    }

    #[test]
    fn an_extension_name_round_trips() {
        assert_eq!(ChannelExtension::from_name("arc"), Some(ChannelExtension::Arc));
        assert_eq!(ChannelExtension::from_name("nope"), None);
    }
}
