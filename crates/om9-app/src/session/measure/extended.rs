// SPDX-License-Identifier: LGPL-2.1-or-later
use super::*;
#[derive(Debug, Clone, Default)]
pub struct MeasureOptions {
    pub continuous: bool,
    pub ortho: bool,
    pub iso: Option<(bool, f64)>,
}
pub(super) fn constrain_point(p: &MeasurePending, point: Point3) -> Result<Point3> {
    if !p.options.ortho
        || p.points.is_empty()
        || (p.feature == ANGLE && p.points.len().is_multiple_of(2))
    {
        return Ok(point);
    }
    use om9_core::coords::{LocalFrame, LocalPoint, WorldPoint};
    let a = p
        .plane
        .to_local(WorldPoint::try_new(*p.points.last().unwrap())?)?
        .xyz();
    let mut b = p.plane.to_local(WorldPoint::try_new(point)?)?.xyz();
    let axis = (0..3)
        .max_by(|i, j| (b[*i] - a[*i]).abs().total_cmp(&(b[*j] - a[*j]).abs()))
        .unwrap();
    for i in 0..3 {
        if i != axis {
            b[i] = a[i];
        }
    }
    Ok(p.plane.to_world(LocalPoint::try_new(b)?)?.xyz())
}
pub(super) fn gem_weights(
    document: &Document,
    entity: &om9_core::model::Entity,
    summary: &mut ObjectSummary,
    path: &mut Vec<om9_core::identity::DefinitionId>,
) {
    if let Some(gem) = entity.extensions.get(GEM_EXTENSION_SLOT) {
        summary.gems += 1;
        let weight = serde_json::from_str::<serde_json::Value>(&gem.payload)
            .ok()
            .and_then(|v| v.get("carat").and_then(|v| v.as_f64()));
        match weight {
            Some(v) if v.is_finite() && v >= 0. && (summary.known_carats + v).is_finite() => {
                summary.known_carats += v
            }
            _ => summary.gems_without_weight += 1,
        }
    }
    if let Geometry::Instance(id) = entity.geometry {
        if path.contains(&id) {
            return;
        }
        if let Some(def) = document.definitions.get(&id) {
            path.push(id);
            for member in &def.members {
                gem_weights(document, member, summary, path);
            }
            path.pop();
        }
    }
}
impl Controller {
    pub(super) fn selected_circle_radius_available(&self) -> bool {
        self.selected_single_curve()
            .is_ok_and(|c| matches!(c, Curve::Circle { .. }))
    }
    pub(super) fn selected_surface_available(&self) -> bool {
        let ids = self.selected_ordered();
        ids.len() == 1
            && self
                .document()
                .entities
                .get(&ids[0])
                .is_some_and(|e| matches!(e.geometry, Geometry::Surface(_)))
    }
    pub(super) fn selected_single_curve(&self) -> Result<Curve> {
        let ids = self.selected_ordered();
        if ids.len() != 1 {
            return Err(Error::Invalid("one curve selection"));
        }
        Ok(selected_curves(self.document(), &ids)?.remove(0))
    }
    pub(super) fn curvature_at_pick(&self, point: Point3) -> Result<Measurement> {
        let c = self.selected_single_curve()?;
        let (f, _, _) = om9_core::curve_surface_ops::closest_point_on_curve(
            &c,
            point,
            self.coordinator.limits(),
        )?;
        self.curvature_at_fraction(f)
    }
    pub fn curvature_at_fraction(&self, f: f64) -> Result<Measurement> {
        let c = self.selected_single_curve()?;
        let r = om9_core::measure_geometry::curvature_radius(&c, f, self.coordinator.limits())?;
        Ok(Measurement::new(
            RADIUS,
            &format!(
                "Local curvature radius at station {}",
                Measurement::format_value(f)
            ),
            r,
            MeasureUnit::Millimetre,
        ))
    }
    pub(super) fn measure_option_input(
        &mut self,
        p: &mut MeasurePending,
        text: &str,
    ) -> Option<Result<()>> {
        let (key, value) = text.split_once('=').unwrap_or((text, ""));
        match key.to_ascii_lowercase().as_str() {
            "continuous" | "ortho" => {
                let setting = match key.to_ascii_lowercase().as_str() {
                    "continuous" => &mut p.options.continuous,
                    _ => &mut p.options.ortho,
                };
                let value = if value.is_empty() {
                    Ok(!*setting)
                } else {
                    match value.to_ascii_lowercase().as_str() {
                        "yes" | "on" => Ok(true),
                        "no" | "off" => Ok(false),
                        _ => Err(Error::Invalid("Yes/No measurement option")),
                    }
                };
                Some(value.map(|v| *setting = v))
            }
            "u" | "v" if p.feature == LENGTH => Some(
                value
                    .parse::<f64>()
                    .map_err(|_| Error::Invalid("isocurve parameter"))
                    .and_then(|v| {
                        if !v.is_finite() {
                            return Err(Error::Invalid("isocurve parameter"));
                        }
                        p.options.iso = Some((key.eq_ignore_ascii_case("u"), v));
                        Ok(())
                    }),
            ),
            _ => None,
        }
    }
    pub fn measure_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        match self.measure_pending.as_ref().map(|p| p.feature) {
            Some(LENGTH) => vec![("U=", "U="), ("V=", "V="), ("Cancel", "Cancel")],
            Some(RADIUS) => vec![("Station=", "Station="), ("Cancel", "Cancel")],
            Some(_) => vec![
                ("Objects", "Objects"),
                ("Continuous", "Continuous"),
                ("Ortho", "Ortho"),
                ("Cancel", "Cancel"),
            ],
            None => vec![],
        }
    }
    pub fn measure_preview(&self, point: Point3) -> Result<Option<(String, Vec<[Point3; 2]>)>> {
        let Some(p) = &self.measure_pending else {
            return Ok(None);
        };
        if p.feature == RADIUS {
            return Ok(Some((
                self.curvature_at_pick(point)?.label().into(),
                vec![],
            )));
        }
        let mut points = p.points.clone();
        points.push(constrain_point(p, point)?);
        if points.len() != points_needed(p.feature) {
            return Ok(None);
        }
        let m = measure_points(p.feature, &points)?;
        let wire = points
            .as_chunks::<2>()
            .0
            .iter()
            .map(|p| [p[0], p[1]])
            .collect();
        Ok(Some((m.label().into(), wire)))
    }
    pub fn measure_selected_objects(&self, feature: &str) -> Result<Measurement> {
        let ids = self.selected_ordered();
        if ids.len() != 2 {
            return Err(Error::Invalid(
                "object measurement needs exactly two objects",
            ));
        }
        let a = &self.document().entities[&ids[0]];
        let b = &self.document().entities[&ids[1]];
        for e in [a, b] {
            if matches!(e.geometry, Geometry::Surface(_))
                && e.extensions
                    .contains_key(om9_core::model::TRIM_SURFACE_EXTENSION_SLOT)
            {
                return Err(Error::Unsupported(
                    "object measurement on a trimmed NURBS surface requires canonical trimmed BRep",
                ));
            }
        }
        if feature == ANGLE {
            let curves = selected_curves(self.document(), &ids)?;
            let endpoints = curves
                .iter()
                .map(|c| match c {
                    Curve::Line { start, end, .. } => Ok([*start, *end]),
                    _ => Err(Error::Unsupported(
                        "object angle needs two straight lines; choose point mode for other angles",
                    )),
                })
                .collect::<Result<Vec<_>>>()?;
            return measure_points(
                ANGLE,
                &[
                    endpoints[0][0],
                    endpoints[0][1],
                    endpoints[1][0],
                    endpoints[1][1],
                ],
            );
        }
        #[cfg(feature = "occt-brep")]
        {
            let d = om9_kernel::measurement::distance(
                &a.geometry,
                a.placement,
                &b.geometry,
                b.placement,
                self.coordinator.limits(),
            )
            .map_err(|e| Error::Io(e.to_string()))?;
            Ok(Measurement::new(
                DISTANCE,
                "Minimum object distance",
                d.millimetres,
                MeasureUnit::Millimetre,
            ))
        }
        #[cfg(not(feature = "occt-brep"))]
        {
            let _ = (a, b);
            Err(Error::Unsupported("object distance requires OCCT adapter"))
        }
    }
    pub fn measure_isocurve(&self, along_u: bool, value: f64) -> Result<Measurement> {
        let ids = self.selected_ordered();
        if ids.len() != 1 {
            return Err(Error::Invalid("one surface selection"));
        }
        let e = &self.document().entities[&ids[0]];
        if e.extensions
            .contains_key(om9_core::model::TRIM_SURFACE_EXTENSION_SLOT)
        {
            return Err(Error::Unsupported(
                "trimmed surface isocurve measurement requires canonical trimmed BRep",
            ));
        }
        let Geometry::Surface(s) = &e.geometry else {
            return Err(Error::Invalid("surface selection"));
        };
        let iso = if along_u {
            om9_core::surface_kernel::SurfaceIso::U
        } else {
            om9_core::surface_kernel::SurfaceIso::V
        };
        let c = Curve::Nurbs(om9_core::surface_kernel::isocurve(
            s,
            iso,
            value,
            self.coordinator.limits(),
        )?);
        let c = om9_core::top11::bake_curve(&c, e.placement)?;
        let length = curve_ops::arc_length(&c, self.coordinator.limits())?;
        if !length.is_finite() {
            return Err(Error::Invalid("measured isocurve length"));
        }
        Ok(Measurement::new(
            LENGTH,
            if along_u {
                "U isocurve length"
            } else {
                "V isocurve length"
            },
            length,
            MeasureUnit::Millimetre,
        ))
    }
    pub(super) fn selected_length(&self, ids: &[ObjectId]) -> Result<Measurement> {
        if ids.is_empty() {
            return Err(Error::Missing("length selection"));
        }
        let mut total = 0.;
        for id in ids {
            let e = &self.document().entities[id];
            if e.extensions.contains_key(om9_core::annotation::SLOT) {
                return Err(Error::Invalid(
                    "Length expects model curves/edges, not annotation strokes",
                ));
            }
            match &e.geometry {
                Geometry::Curve(c) => {
                    total += curve_ops::arc_length(
                        &om9_core::top11::bake_curve(c, e.placement)?,
                        self.coordinator.limits(),
                    )?
                }
                #[cfg(feature = "occt-brep")]
                Geometry::Brep(b) => {
                    let selected = self
                        .subselection
                        .iter()
                        .filter(|s| s.root == *id)
                        .collect::<Vec<_>>();
                    if selected.is_empty() {
                        total += om9_kernel::measurement::edge_length(
                            b,
                            None,
                            e.placement,
                            self.coordinator.limits(),
                        )
                        .map_err(|e| Error::Io(e.to_string()))?;
                    } else {
                        for s in selected {
                            self.document().resolve(&s.source)?;
                            if !s.path.is_empty() {
                                return Err(Error::Unsupported("instance member edge measurement"));
                            }
                            let om9_scene::scene::Subelement::BrepEdge(edge) = s.subset else {
                                return Err(Error::Invalid(
                                    "Length sub-selection requires BRep edges",
                                ));
                            };
                            let index = b
                                .display
                                .edges
                                .iter()
                                .position(|id| *id == edge)
                                .ok_or(Error::Stale("BRep edge"))?;
                            total += om9_kernel::measurement::edge_length(
                                b,
                                Some(index),
                                e.placement,
                                self.coordinator.limits(),
                            )
                            .map_err(|e| Error::Io(e.to_string()))?;
                        }
                    }
                }
                _ => {
                    return Err(Error::Unsupported(
                        "Length expects curves, canonical BRep edges, or a surface isocurve",
                    ));
                }
            }
        }
        if !total.is_finite() {
            return Err(Error::Invalid("total measured length"));
        }
        Ok(Measurement::new(
            LENGTH,
            "Length",
            total,
            MeasureUnit::Millimetre,
        ))
    }
    pub fn measure_selected_integral(&self, feature: &str) -> Result<Measurement> {
        let ids = self.selected_ordered();
        if ids.is_empty() {
            return Err(Error::Missing("area/volume selection"));
        }
        let mut total = 0.;
        for id in ids {
            let e = &self.document().entities[&id];
            if matches!(e.geometry, Geometry::Surface(_))
                && e.extensions
                    .contains_key(om9_core::model::TRIM_SURFACE_EXTENSION_SLOT)
            {
                return Err(Error::Unsupported(
                    "area of trimmed NURBS surfaces requires canonical trimmed BRep",
                ));
            }
            match &e.geometry {
                Geometry::Surface(s) if feature == AREA => {
                    let mut s = s.clone();
                    for p in &mut s.poles {
                        *p = e.placement.point(*p)?;
                    }
                    total += om9_core::measure_geometry::surface_area(
                        &s,
                        1e-7,
                        self.coordinator.limits(),
                    )?;
                }
                #[cfg(feature = "occt-brep")]
                Geometry::Brep(b) => {
                    let b = if e.placement == Transform::identity() {
                        b.clone()
                    } else {
                        om9_kernel::transform_ops::transform_brep(
                            b,
                            &e.placement,
                            0.1,
                            self.coordinator.limits(),
                        )
                        .map_err(|e| Error::Io(e.to_string()))?
                    };
                    let m = om9_kernel::creation::measure(&b, self.coordinator.limits())
                        .map_err(|e| Error::Io(e.to_string()))?;
                    if feature == VOLUME && (!m.closed || m.solids == 0) {
                        return Err(Error::Invalid("volume requires a closed solid"));
                    }
                    total += if feature == AREA {
                        m.area_mm2
                    } else {
                        m.volume_mm3.abs()
                    };
                }
                _ => {
                    return Err(Error::Unsupported(
                        "area/volume needs canonical BRep; area also supports NURBS surfaces",
                    ));
                }
            }
        }
        if !total.is_finite() {
            return Err(Error::Invalid("total measured area/volume"));
        }
        Ok(Measurement::new(
            if feature == AREA { AREA } else { VOLUME },
            if feature == AREA { "Area" } else { "Volume" },
            total,
            if feature == AREA {
                MeasureUnit::SquareMillimetre
            } else {
                MeasureUnit::CubicMillimetre
            },
        ))
    }
}
