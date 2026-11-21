// SPDX-License-Identifier: LGPL-2.1-or-later
use super::*;
use om9_core::{
    annotation::{Annotation, Kind, SLOT},
    budget::Limits,
    geometry::{Mesh, NurbsSurface},
};

fn line(start: Point3, end: Point3) -> Curve {
    Curve::Line {
        start,
        end,
        domain: [0., 1.],
    }
}

pub(super) fn annotation_parts(entity: &Entity, limits: &Limits) -> Result<Vec<Entity>> {
    let annotation = Annotation::read(entity)?;
    let layout = annotation.layout(limits)?;
    let child = |geometry| {
        let mut piece = child_entity(entity, geometry);
        piece.extensions.remove(SLOT);
        piece
    };
    let mut parts = Vec::new();
    if annotation.kind != Kind::Text {
        for [a, b] in layout.lines {
            if a != b {
                parts.push(child(Geometry::Curve(line(a, b))));
            }
        }
        if !layout.text.is_empty() {
            let text = Annotation {
                kind: Kind::Text,
                points: Vec::new(),
                text: Some(layout.text),
                ..annotation
            };
            let mut piece = child(text.geometry(limits)?);
            piece
                .extensions
                .insert(SLOT.to_owned(), text.extension(limits)?);
            parts.push(piece);
        }
        return Ok(parts);
    }
    // Each pen segment retains the original line or Bezier control polygon.
    let mut contours = Vec::new();
    crate::session::solid_geometry::for_each_text_glyph(
        &layout.text,
        annotation.style.height_mm,
        true,
        |mut pen: GlyphPen, advance| {
            pen.finish();
            for mut contour in pen.contours {
                for p in contour.iter_mut().flatten() {
                    p[0] += advance;
                }
                contours.push(contour);
            }
        },
    )?;
    if contours.is_empty() {
        return Err(Error::Invalid("text produced no contour"));
    }
    let mut min = [f64::INFINITY; 2];
    let mut max = [f64::NEG_INFINITY; 2];
    for p in contours.iter().flatten().flatten() {
        for i in 0..2 {
            min[i] = min[i].min(p[i]);
            max[i] = max[i].max(p[i]);
        }
    }
    let place = |p: Point3| {
        std::array::from_fn(|i| {
            annotation.text_point[i]
                + annotation.x[i] * (p[0] - (min[0] + max[0]) * 0.5)
                + annotation.y[i] * (p[1] - (min[1] + max[1]) * 0.5)
        })
    };
    for contour in contours {
        let segments = contour
            .into_iter()
            .map(|poles| {
                let degree = poles.len() - 1;
                if degree == 1 {
                    return line(place(poles[0]), place(poles[1]));
                }
                let knots = [vec![0.; degree + 1], vec![1.; degree + 1]].concat();
                Curve::Nurbs(om9_core::geometry::NurbsCurve {
                    degree,
                    weights: vec![1.; poles.len()],
                    poles: poles.into_iter().map(place).collect(),
                    knots,
                    domain: [0., 1.],
                    periodic: false,
                    reversed: false,
                    seam: None,
                })
            })
            .collect();
        let curve = Curve::Polycurve {
            segments,
            reversed: false,
        };
        curve.validate(limits)?;
        parts.push(child(Geometry::Curve(curve)));
    }
    Ok(parts)
}

#[derive(Default)]
struct GlyphPen {
    contours: Vec<Vec<Vec<Point3>>>,
    segments: Vec<Vec<Point3>>,
    start: Point3,
    position: Point3,
}

impl GlyphPen {
    fn segment(&mut self, tail: &[Point3]) {
        let mut poles = vec![self.position];
        poles.extend_from_slice(tail);
        self.position = *tail.last().expect("segment endpoint");
        self.segments.push(poles);
    }
    fn finish(&mut self) {
        if self.segments.is_empty() {
            return;
        }
        if self.position != self.start {
            self.segment(&[self.start]);
        }
        self.contours.push(std::mem::take(&mut self.segments));
    }
}

impl skrifa::outline::OutlinePen for GlyphPen {
    fn move_to(&mut self, x: f32, y: f32) {
        self.finish();
        self.start = [x.into(), y.into(), 0.];
        self.position = self.start;
    }
    fn line_to(&mut self, x: f32, y: f32) {
        let end = [x.into(), y.into(), 0.];
        if self.position != end {
            self.segment(&[end]);
        }
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.segment(&[[x1.into(), y1.into(), 0.], [x.into(), y.into(), 0.]]);
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.segment(&[
            [x1.into(), y1.into(), 0.],
            [x2.into(), y2.into(), 0.],
            [x.into(), y.into(), 0.],
        ]);
    }
    fn close(&mut self) {
        self.finish();
    }
}

/// OM9's eight-pole trilinear cage has six bilinear boundary patches and twelve lattice edges.
pub(super) fn cage_parts(entity: &Entity, mesh: &Mesh) -> Result<Vec<Entity>> {
    let corners = om9_core::cage::mesh_corners(mesh)?;
    let mut parts = Vec::with_capacity(18);
    let mut add = |geometry| {
        let mut part = child_entity(entity, geometry);
        part.extensions.remove(om9_core::model::CAGE_EXTENSION_SLOT);
        parts.push(part);
    };
    for axis in 0..3 {
        let bit = 1 << axis;
        for i in 0..8 {
            if i & bit == 0 {
                add(Geometry::Curve(line(corners[i], corners[i | bit])));
            }
        }
        let u = 1 << ((axis + 1) % 3);
        let v = 1 << ((axis + 2) % 3);
        for side in [0, bit] {
            let poles = [side, side | v, side | u, side | u | v]
                .map(|i| corners[i])
                .to_vec();
            add(Geometry::Surface(NurbsSurface {
                degree: [1; 2],
                size: [2; 2],
                poles,
                weights: vec![1.; 4],
                knots: [vec![0., 0., 1., 1.], vec![0., 0., 1., 1.]],
                domain: [[0., 1.]; 2],
                periodic: [false; 2],
                reversed: side == 0,
            }));
        }
    }
    Ok(parts)
}
