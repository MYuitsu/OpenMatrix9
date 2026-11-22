// SPDX-License-Identifier: LGPL-2.1-or-later
use super::*;
use DraftCorners::{Round, Sharp, Smooth};
use om9_addon_abi::session::{add, cross, unit};
use om9_core::curve_ops::extrude_lines::{DraftCorners, smooth_draft_handle};

pub(crate) const CODE: &str = "ExtrudeCrvTapered";
const LABEL: &str = "Extrude Tapered";

pub(in crate::session) struct Defaults {
    pub(super) common: super::Defaults,
    angle: f64,
    corners: DraftCorners,
}
impl Default for Defaults {
    fn default() -> Self {
        Self {
            common: Default::default(),
            angle: 5.,
            corners: Sharp,
        }
    }
}

pub(super) struct Taper {
    pub(super) angle: f64,
    corners: DraftCorners,
    linear: Vec<(Vec<Point3>, Option<Point3>)>,
}
impl Taper {
    pub(super) fn capture(
        c: &Controller,
        inputs: &[(&Curve, Transform)],
        profiles: &ControlProfiles,
        tol: f64,
    ) -> Result<Self> {
        let mut linear = Vec::new();
        let mut curved = profiles.curved.iter().peekable();
        let limits = c.coordinator.limits();
        let mut held = Usage::default();
        for (i, &(curve, world)) in inputs.iter().enumerate() {
            if curved.next_if(|r| r.source == i).is_none() {
                let room = limits.remaining_after(held)?;
                let path = om9_core::curve_ops::linear_path::placed_vertices(
                    curve,
                    world,
                    tol,
                    &room,
                    &mut || Ok(()),
                )?;
                held.add(Usage::array::<Point3>(path.capacity())?, limits)?;
                let room = limits.remaining_after(held)?;
                let normal =
                    om9_core::curve_ops::extrude_lines::profile_plane([(curve, world)], &room)?;
                linear.push((path, normal));
            }
        }
        Ok(Self {
            angle: c.taper_defaults.angle,
            corners: c.taper_defaults.corners,
            linear,
        })
    }
    pub(super) fn options(&self, solid: bool, delete: bool) -> Vec<OptionToken> {
        vec![
            OptionToken::action("Direction"),
            OptionToken::number("DraftAngle", Some(self.angle)),
            OptionToken::toggle("Solid", solid),
            OptionToken::choice(
                "Corners",
                match self.corners {
                    Sharp => "Sharp",
                    Round => "Round",
                    Smooth => "Smooth",
                },
                &["Sharp", "Round", "Smooth"],
            ),
            OptionToken::toggle("DeleteInput", delete),
            OptionToken::action("FlipAngle"),
            OptionToken::action("ToBoundary"),
            OptionToken::action("SetBasePoint"),
        ]
    }
    pub(super) fn set_angle(&mut self, text: &str) -> Result<()> {
        let a = text
            .trim()
            .trim_end_matches('°')
            .parse::<f64>()
            .map_err(|_| Error::Invalid("Enter a draft angle in degrees"))?;
        if a == 0. {
            return Err(Error::Invalid("Cannot set draft angle to zero"));
        }
        if !a.is_finite() || a.abs() > 90. {
            return Err(Error::Invalid(
                "Draft angle must be between -90 and 90 degrees",
            ));
        }
        self.angle = a;
        Ok(())
    }
    pub(super) fn set_corners(&mut self, value: Option<&str>) -> Result<()> {
        self.corners = match value
            .map(|s| s.trim_start_matches('_').to_ascii_lowercase())
            .as_deref()
        {
            Some("sharp") => Sharp,
            Some("round") => Round,
            Some("smooth") => Smooth,
            None => match self.corners {
                Sharp => Round,
                Round => Smooth,
                Smooth => Sharp,
            },
            _ => return Err(Error::Invalid("Choose Sharp, Round or Smooth corners")),
        };
        Ok(())
    }

    pub(super) fn preview(
        &self,
        session: &ExtrudeLines,
        c: &Controller,
        h: f64,
        mut reply: Reply,
    ) -> Result<Reply> {
        if h == 0. {
            return Ok(reply);
        }
        let axis = session.axis;
        let distance = h.abs() * self.angle.to_radians().tan();
        let k = c.document().page_unit.factor();
        let limit = c.coordinator.limits();
        let mut emit = |a: Point3, b: Point3| -> Result<()> {
            if reply.preview.len() as u64 >= limit.max_outputs {
                return Err(Error::Budget("Taper preview"));
            }
            reply.preview.push(Geo::Line {
                start: scale(a, k),
                end: scale(b, k),
            });
            Ok(())
        };
        for (path, normal) in self
            .linear
            .iter()
            .map(|(path, normal)| (path, *normal))
            .chain(session.profiles.curved.iter().map(|r| (&r.display, None)))
        {
            let n = path.len();
            if n < 2 {
                continue;
            }
            let closed = om9_addon_abi::session::norm(sub(path[0], path[n - 1])) < 1e-7;
            let count = if closed { n - 1 } else { n };
            if closed
                && normal
                    .is_some_and(|n| om9_core::curve_ops::extrude_lines::is_inclined_draft(n, axis))
            {
                for pair in path.windows(2) {
                    let lateral = unit(cross(axis, sub(pair[1], pair[0])));
                    let shift = add(scale(axis, h), scale(lateral, distance));
                    let a = add(pair[0], shift);
                    let b = add(pair[1], shift);
                    emit(pair[0], a)?;
                    emit(a, b)?;
                    emit(b, pair[1])?;
                }
                continue;
            }
            let mut first = None;
            let mut previous = None;
            for i in 0..count {
                let p = path[i];
                let center = add(p, scale(axis, h));
                let before = if i == 0 && !closed {
                    0
                } else {
                    (i + count - 1) % count
                };
                let after = if i + 1 == count && !closed {
                    i
                } else {
                    (i + 1) % count
                };
                let mut u = unit(sub(p, path[before]));
                let mut v = unit(sub(path[after], p));
                if before == i {
                    u = v;
                }
                if after == i {
                    v = u;
                }
                let left = unit(cross(axis, u));
                let right = unit(cross(axis, v));
                let a = add(center, scale(left, distance));
                let b = add(center, scale(right, distance));
                let cos = dot(left, right).clamp(-1., 1.);
                let turn = dot(axis, cross(left, right));
                let outside = turn * distance < 0. && cos < 1. - 1e-8 && self.corners != Sharp;
                let (start, end) = if outside {
                    let handle =
                        smooth_draft_handle(om9_addon_abi::session::norm(sub(b, a)), dot(u, v));
                    let ta = add(a, scale(u, handle));
                    let tb = sub(b, scale(v, handle));
                    let mut prior = a;
                    for step in 1..=8 {
                        let t = f64::from(step) / 8.;
                        let point = if self.corners == Round {
                            let angle = t * turn.atan2(cos);
                            add(
                                center,
                                scale(
                                    add(
                                        scale(left, angle.cos()),
                                        scale(cross(axis, left), angle.sin()),
                                    ),
                                    distance,
                                ),
                            )
                        } else {
                            let s = 1. - t;
                            add(
                                add(scale(a, s * s * s), scale(ta, 3. * s * s * t)),
                                add(scale(tb, 3. * s * t * t), scale(b, t * t * t)),
                            )
                        };
                        emit(prior, point)?;
                        prior = point;
                    }
                    (a, b)
                } else {
                    let corner = if cos <= -1. + 1e-8 {
                        a
                    } else {
                        add(center, scale(add(left, right), distance / (1. + cos)))
                    };
                    (corner, corner)
                };
                if let Some(prior) = previous {
                    emit(prior, start)?;
                } else {
                    first = Some(start);
                }
                if i % (count / 12).max(1) == 0 || i + 1 == count {
                    emit(p, start)?;
                    if start != end {
                        emit(p, end)?;
                    }
                }
                previous = Some(end);
            }
            if closed && let (Some(last), Some(head)) = (previous, first) {
                emit(last, head)?;
            }
        }
        Ok(reply)
    }
}

#[cfg(feature = "occt-brep")]
impl ExtrudeLines {
    pub(super) fn finish_taper_boundary(&self, c: &mut Controller) -> Result<Reply> {
        let face = super::boundary::boundary_face(c)?;
        let limits = c.coordinator.limits();
        let tol = c.document().effective_tolerance()?.absolute_mm / c.document().page_unit.factor();
        let boundary = om9_kernel::extrusion::boundary::BoundaryProjection::new(
            face,
            self.axis,
            self.profiles.start,
            tol,
            om9_kernel::creation::DEFAULT_DEFLECTION_MM,
            limits,
        )?;
        self.finish_taper(c, 0., Some(&boundary))
    }

    pub(super) fn finish_taper(
        &self,
        c: &mut Controller,
        h: f64,
        boundary: Option<&om9_kernel::extrusion::boundary::BoundaryProjection>,
    ) -> Result<Reply> {
        let taper = self.taper.as_ref().ok_or(Error::Missing("taper options"))?;
        let limits = c.coordinator.limits();
        if self.ids.len() as u64 > limits.max_outputs {
            return Err(Error::Budget("Taper output count"));
        }
        let count = self
            .ids
            .len()
            .checked_mul(1 + usize::from(self.delete))
            .ok_or(Error::Overflow("Taper transaction"))?;
        let usage = Usage::array::<Mutation>(count)?;
        usage.check(limits)?;
        let mut changes = Vec::with_capacity(count);
        if self.delete {
            for id in &self.ids {
                changes.push(Mutation::Delete {
                    source: c.document().stamp(*id)?,
                });
            }
        }
        c.publish_extrude_solids(
            (CODE, LABEL),
            &self.ids,
            changes,
            Vec::with_capacity(self.ids.len()),
            usage,
            |c, id, room| {
                let e = &c.document().entities[id];
                let Geometry::Curve(curve) = &e.geometry else {
                    return Err(Error::Stale("Taper profile"));
                };
                let world = om9_core::top11::bake_curve(curve, e.placement)?;
                let identity = Transform::identity();
                let profile = self.solid_profile(c, &world, identity, room)?;
                let body = if let Some(boundary) = boundary {
                    boundary.tapered(
                        &profile,
                        self.axis,
                        taper.angle.to_radians(),
                        taper.corners,
                        self.solid,
                        room,
                    )?
                } else {
                    om9_kernel::extrusion::tapered(
                        &profile,
                        self.axis,
                        h,
                        taper.angle.to_radians(),
                        taper.corners,
                        self.solid,
                        om9_kernel::creation::DEFAULT_DEFLECTION_MM,
                        room,
                    )
                    .map_err(|e| Error::Io(format!("Extrude Tapered: {e}")))?
                };
                Ok((body, identity))
            },
        )?;
        let distance_mm = h * c.document().page_unit.factor();
        let defaults = &mut c.taper_defaults;
        defaults.angle = taper.angle;
        defaults.corners = taper.corners;
        defaults.common.solid = self.solid;
        defaults.common.delete = self.delete;
        if boundary.is_none() {
            defaults.common.distance_mm = Some(distance_mm);
        }
        Ok(Reply {
            done: true,
            state: serde_json::json!(self.ids.len()),
            ..Default::default()
        })
    }
}

impl Controller {
    pub(in crate::session) fn start_tapered(&mut self) -> Result<()> {
        if !cfg!(feature = "occt-brep") {
            return Err(Error::Missing("exact BRep kernel"));
        }
        self.start_interactive_session(CODE, LABEL, "om9".into(), false, false)
    }
}
