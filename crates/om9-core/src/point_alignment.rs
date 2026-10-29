// SPDX-License-Identifier: LGPL-2.1-or-later
//! SetPt changes canonical point/control coordinates, never an object's placement origin.
use crate::{
    Error, Result,
    budget::Limits,
    coords::{LocalFrame, LocalPoint, WorldPoint},
    geometry::{Curve, Geometry, Point3, Transform},
    model::Entity,
};

pub fn set_points(
    entity: &Entity,
    target: Point3,
    axes: [bool; 3],
    indices: &[usize],
    frame: Option<&dyn LocalFrame>,
    limits: &Limits,
) -> Result<Geometry> {
    if entity.extensions.contains_key(crate::annotation::SLOT) {
        return Err(Error::Unsupported(
            "SetPt on annotation: use Edit Dimension or Edit Text",
        ));
    }
    if !axes.iter().any(|axis| *axis) {
        return Err(Error::Invalid("SetPt needs at least one coordinate axis"));
    }
    let inverse = crate::top11::inverse_affine(entity.placement)?;
    let target = WorldPoint::try_new(target)?;
    let target = match frame {
        Some(f) => f.to_local(target)?.xyz(),
        None => target.xyz(),
    };
    let mut index = 0;
    let mut edit = |local: Point3| -> Result<Point3> {
        let current = index;
        index += 1;
        if !indices.is_empty() && !indices.contains(&current) {
            return Ok(local);
        }
        let world = WorldPoint::try_new(entity.placement.point(local)?)?;
        let mut xyz = match frame {
            Some(f) => f.to_local(world)?.xyz(),
            None => world.xyz(),
        };
        for axis in 0..3 {
            if axes[axis] {
                xyz[axis] = target[axis];
            }
        }
        let world = match frame {
            Some(f) => f.to_world(LocalPoint::try_new(xyz)?)?.xyz(),
            None => xyz,
        };
        inverse.point(world)
    };
    fn curve(c: &mut Curve, edit: &mut impl FnMut(Point3) -> Result<Point3>) -> Result<()> {
        match c {
            Curve::Line { start, end, .. } => {
                *start = edit(*start)?;
                *end = edit(*end)?;
            }
            Curve::Nurbs(n) => {
                for p in &mut n.poles {
                    *p = edit(*p)?;
                }
            }
            Curve::Polycurve { segments, .. } => {
                for c in segments {
                    curve(c, edit)?;
                }
            }
            Curve::Circle { .. } => {
                return Err(Error::Unsupported(
                    "SetPt circle control points require explicit NURBS conversion",
                ));
            }
        }
        Ok(())
    }
    let mut geometry = entity.geometry.clone();
    match &mut geometry {
        Geometry::Point(p) => *p = edit(*p)?,
        Geometry::Curve(c) => curve(c, &mut edit)?,
        Geometry::Surface(s) => {
            for p in &mut s.poles {
                *p = edit(*p)?;
            }
        }
        Geometry::Mesh(m) => {
            for p in &mut m.vertices {
                *p = edit(*p)?;
            }
            m.normals.clear();
        }
        Geometry::Cloud(c) => {
            for p in &mut c.points {
                *p = edit(*p)?;
            }
            c.normals.clear();
        }
        _ => {
            return Err(Error::Unsupported(
                "SetPt control-point editing is unavailable for this geometry",
            ));
        }
    }
    if indices.iter().any(|i| *i >= index) {
        return Err(Error::Invalid("SetPt control point index"));
    }
    geometry.validate(limits)?;
    Ok(geometry)
}

/// Native-independent read-only outline for a canonical geometry/placement preview.
pub fn outline(
    geometry: &Geometry,
    placement: Transform,
    limits: &Limits,
) -> Result<Vec<[Point3; 2]>> {
    match geometry {
        Geometry::Curve(c) => {
            let p = crate::curve_ops::sample_curve(c, 65, limits)?;
            p.windows(2)
                .map(|p| Ok([placement.point(p[0])?, placement.point(p[1])?]))
                .collect()
        }
        Geometry::Point(p) => {
            let p = placement.point(*p)?;
            Ok(vec![[p, p]])
        }
        Geometry::Cloud(c) => c
            .points
            .iter()
            .map(|p| {
                let p = placement.point(*p)?;
                Ok([p, p])
            })
            .collect(),
        Geometry::Mesh(m) => {
            let mut wire = Vec::new();
            for f in &m.faces {
                for i in 0..f.len() {
                    wire.push([
                        placement.point(m.vertices[f[i] as usize])?,
                        placement.point(m.vertices[f[(i + 1) % f.len()] as usize])?,
                    ]);
                }
            }
            Ok(wire)
        }
        Geometry::Surface(s) => {
            let mut wire = Vec::new();
            for axis in 0..2 {
                for i in 0..=8 {
                    let fixed = s.domain[1 - axis][0]
                        + (s.domain[1 - axis][1] - s.domain[1 - axis][0]) * i as f64 / 8.;
                    let iso = if axis == 0 {
                        crate::surface_kernel::SurfaceIso::U
                    } else {
                        crate::surface_kernel::SurfaceIso::V
                    };
                    let c = Curve::Nurbs(crate::surface_kernel::isocurve(s, iso, fixed, limits)?);
                    wire.extend(outline(&Geometry::Curve(c), placement, limits)?);
                }
            }
            Ok(wire)
        }
        _ => {
            let b =
                crate::utilities::bounds(geometry).ok_or(Error::Unsupported("preview bounds"))?;
            let corners: Vec<Point3> = (0..8)
                .map(|i| {
                    placement.point(std::array::from_fn(|a| {
                        if i & (1 << a) == 0 {
                            b.min[a]
                        } else {
                            b.max[a]
                        }
                    }))
                })
                .collect::<Result<_>>()?;
            Ok((0..8)
                .flat_map(|i| {
                    (0..3).filter_map(move |a| {
                        if i & (1 << a) == 0 {
                            Some((i, i | (1 << a)))
                        } else {
                            None
                        }
                    })
                })
                .map(|(i, j)| [corners[i], corners[j]])
                .collect())
        }
    }
}
