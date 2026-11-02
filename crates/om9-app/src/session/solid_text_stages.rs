// SPDX-License-Identifier: LGPL-2.1-or-later
//! Typed text stages for a pending solid command.
//!
//! One method per family (slab, torus, ellipsoid, boss, pipe, fillet, shell, extract, cap,
//! boolean, sphere, box, text) plus the shared option/commit tail. Extracted unchanged from
//! `submit_solid_pending`; each stage falls back to the generic tail exactly like the original
//! chain did after its feature check stopped matching.
use super::solid::*;
use super::*;

impl Controller {
    #[cfg(feature = "occt-brep")]
    pub(super) fn submit_solid_slab_text(
        &mut self,
        mut pending: SolidPending,
        text: &str,
    ) -> Option<Result<()>> {
        let input = text.trim();
        if input.eq_ignore_ascii_case("cancel") || input.eq_ignore_ascii_case("esc") {
            self.cancel();
            return Some(Ok(()));
        }
        if input.eq_ignore_ascii_case("undo") {
            if pending.slab_offset.take().is_none() && pending.slab_distance.take().is_none() {
                pending.slab_source = None;
            }
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if pending.slab_source.is_none() {
            self.solid_pending = Some(pending);
            return Some(Err(Error::Missing("select a Slab curve")));
        }
        if let Some((key, value)) = input.split_once('=') {
            if key.eq_ignore_ascii_case("InCPlane") {
                if pending.slab_offset.is_some() {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Invalid("set InCPlane before picking the side")));
                }
                let yes = value.eq_ignore_ascii_case("yes");
                if !yes && !value.eq_ignore_ascii_case("no") {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Invalid("InCPlane must be Yes or No")));
                }
                let source = pending.slab_source.expect("Slab source captured");
                let result = (|| -> Result<[f64; 3]> {
                    if self.document().stamp(source.object)? != source {
                        return Err(Error::Stale("Slab source"));
                    }
                    let entity = self
                        .document()
                        .entities
                        .get(&source.object)
                        .ok_or(Error::Stale("Slab source"))?;
                    let geometry = SlabSourceGeometry::capture(entity, self.coordinator.limits())?;
                    if !yes {
                        return geometry
                            .normal(pending.slab_cplane_normal, self.coordinator.limits());
                    }
                    let n = pending.slab_cplane_normal;
                    geometry.supports_plane(n)?;
                    Ok(n)
                })();
                match result {
                    Ok(normal) => {
                        pending.slab_normal = normal;
                        pending.slab_in_cplane = yes;
                        self.solid_pending = Some(pending);
                        return Some(Ok(()));
                    }
                    Err(error) => {
                        self.solid_pending = Some(pending);
                        return Some(Err(error));
                    }
                }
            }
        }
        if input.eq_ignore_ascii_case("ThroughPoint") {
            if pending.slab_offset.is_some() {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Invalid(
                    "ThroughPoint must be selected before the height",
                )));
            }
            pending.slab_distance = None;
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if let Some((key, value)) = input.split_once('=') {
            if key.eq_ignore_ascii_case("BothSides") {
                let yes = value.eq_ignore_ascii_case("yes");
                if !yes && !value.eq_ignore_ascii_case("no") {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Invalid("BothSides must be Yes or No")));
                }
                pending.slab_both_sides = yes;
                self.solid_pending = Some(pending);
                return Some(Ok(()));
            }
        }
        let raw = input
            .split_once('=')
            .map(|(key, value)| {
                if (pending.slab_offset.is_none() && key.eq_ignore_ascii_case("Distance"))
                    || (pending.slab_offset.is_some() && key.eq_ignore_ascii_case("Height"))
                {
                    value
                } else {
                    input
                }
            })
            .unwrap_or(input);
        let Some(value) = raw
            .parse::<f64>()
            .ok()
            .filter(|v| v.is_finite() && *v != 0.0)
        else {
            self.solid_pending = Some(pending);
            return Some(Err(Error::Invalid("Slab value must be finite and nonzero")));
        };
        let value = match self.document().page_unit.length_to_mm(value) {
            Ok(v) => v,
            Err(error) => {
                self.solid_pending = Some(pending);
                return Some(Err(error));
            }
        };
        if !value.is_finite() || !(1e-7..=1e9).contains(&value.abs()) {
            self.solid_pending = Some(pending);
            return Some(Err(Error::Invalid(
                "Slab dimension outside supported range",
            )));
        }
        if pending.slab_offset.is_none() {
            if value <= 0.0 {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Invalid("Slab distance must be positive")));
            }
            pending.slab_distance = Some(value);
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        let source = pending.slab_source.expect("Slab source captured");
        let result = self.solid_slab_commit_polyline(
            source,
            pending.slab_normal,
            pending.slab_offset.expect("Slab offset"),
            value,
            pending.slab_both_sides,
        );
        if result.is_err() {
            self.solid_pending = Some(pending);
        }
        return Some(result);
    }

    pub(super) fn submit_solid_torus_text(
        &mut self,
        mut pending: SolidPending,
        text: &str,
    ) -> Option<Result<()>> {
        let input = text.trim();
        let mode = input
            .strip_prefix("Mode=")
            .unwrap_or(input)
            .trim_start_matches('_');
        if mode.eq_ignore_ascii_case("vertical") || mode.eq_ignore_ascii_case("v") {
            if !pending.points.is_empty() || !pending.values.is_empty() {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Invalid(
                    "choose Vertical before the first point",
                )));
            }
            pending.torus_vertical = true;
            pending.torus_around = false;
            pending.torus_two_point = false;
            pending.torus_three_point = false;
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if mode.eq_ignore_ascii_case("aroundcurve") || mode.eq_ignore_ascii_case("a") {
            if !pending.points.is_empty() || !pending.values.is_empty() {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Invalid("choose AroundCurve before first point")));
            }
            pending.torus_around = true;
            pending.torus_vertical = false;
            pending.torus_two_point = false;
            pending.torus_three_point = false;
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if pending.torus_around
            && pending.points.is_empty()
            && input.to_ascii_lowercase().starts_with("oncurve=")
        {
            let value = input
                .split_once('=')
                .and_then(|(_, v)| v.parse::<f64>().ok());
            self.solid_pending = Some(pending);
            return Some(match value {
                Some(value) => self.solid_cone_around_center(value),
                None => Err(Error::Invalid("OnCurve fraction")),
            });
        }
        if mode.eq_ignore_ascii_case("2point") || mode.eq_ignore_ascii_case("p") {
            if !pending.points.is_empty() || !pending.values.is_empty() {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Invalid("choose 2Point before the first point")));
            }
            pending.torus_two_point = true;
            pending.torus_vertical = false;
            pending.torus_three_point = false;
            pending.torus_around = false;
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if mode.eq_ignore_ascii_case("3point") {
            if !pending.points.is_empty() || !pending.values.is_empty() {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Invalid("choose 3Point before the first point")));
            }
            pending.torus_three_point = true;
            pending.torus_vertical = false;
            pending.torus_two_point = false;
            pending.torus_around = false;
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if input.eq_ignore_ascii_case("undo") || input.eq_ignore_ascii_case("u") {
            if pending.torus_vertical && pending.points.len() == 2 {
                pending.points.pop();
                if let Some(frame) = pending.cone_plane.as_deref_mut() {
                    if let Some(original) = frame.fit_input_plane.take() {
                        frame.plane = original;
                    }
                }
                self.solid_pending = Some(pending);
                return Some(Ok(()));
            }
            if pending.torus_three_point && pending.cone_flags.radius_phase() == 3 {
                pending.points.pop();
                pending.cone_flags.set_radius_phase(2);
                self.solid_pending = Some(pending);
                return Some(Ok(()));
            }
            if pending.torus_three_point && matches!(pending.cone_flags.radius_phase(), 1 | 2) {
                let phase = pending.cone_flags.radius_phase() - 1;
                pending.cone_flags.set_radius_phase(phase);
                self.solid_pending = Some(pending);
                return Some(Ok(()));
            }
            if pending.points.pop().is_none() {
                pending.values.pop();
            }
            if pending.points.is_empty() {
                pending.cone_plane = None;
            }
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if pending.torus_three_point && pending.points.len() == 2 {
            let lower = input.to_ascii_lowercase();
            if lower == "radius" || lower == "radius=" {
                pending.cone_flags.set_radius_phase(1);
                self.solid_pending = Some(pending);
                return Some(Ok(()));
            }
            if let Some(raw) = lower.strip_prefix("radius=") {
                let value = raw.parse::<f64>().ok();
                self.solid_pending = Some(pending);
                return Some(match value {
                    Some(v) => self.solid_cone_radius_value(v, true),
                    None => Err(Error::Invalid("base circle radius")),
                });
            }
            if pending.cone_flags.radius_phase() == 1 && !input.contains(',') {
                let value = input.parse::<f64>().ok();
                self.solid_pending = Some(pending);
                return Some(match value {
                    Some(v) => self.solid_cone_radius_value(v, true),
                    None => Err(Error::Invalid("base circle radius")),
                });
            }
        }
        if input.contains(',') {
            let point: std::result::Result<Vec<f64>, _> =
                input.split(',').map(|v| v.trim().parse::<f64>()).collect();
            self.solid_pending = Some(pending);
            return Some(match point {
                Ok(v) if v.len() == 3 && v.iter().all(|x| x.is_finite()) => {
                    self.solid_torus_accept_point([v[0], v[1], v[2]])
                }
                _ => Err(Error::Invalid("torus point must be x,y,z")),
            });
        }
        if (pending.torus_two_point && pending.points.is_empty())
            || (pending.torus_three_point && pending.points.len() < 3)
            || (pending.torus_around && pending.points.is_empty())
            || (pending.torus_vertical && pending.points.is_empty())
        {
            self.solid_pending = Some(pending);
            return Some(Err(Error::Invalid("enter first diameter endpoint")));
        }
        if !pending.points.is_empty() {
            let Some(value) = input
                .parse::<f64>()
                .ok()
                .filter(|v| v.is_finite() && *v > 1e-9)
            else {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Invalid("torus radius must be positive")));
            };
            let plane = pending.cone_plane.as_deref().expect("torus frame").plane;
            let origin = if pending.points.len() == 1 {
                pending.points[0]
            } else {
                *pending.points.last().expect("torus ring point")
            };
            let axis = if pending.points.len() == 1 {
                plane.x
            } else {
                let center = match pending.torus_base() {
                    Ok((center, _, _)) => center,
                    Err(error) => {
                        self.solid_pending = Some(pending);
                        return Some(Err(error));
                    }
                };
                let d = std::array::from_fn::<_, 3, _>(|i| origin[i] - center[i]);
                let len = d[0].hypot(d[1]).hypot(d[2]);
                d.map(|x| x / len)
            };
            let point = std::array::from_fn(|i| origin[i] + value * axis[i]);
            self.solid_pending = Some(pending);
            return Some(self.solid_torus_accept_point(point));
        }
        self.submit_solid_generic_tail(pending, text)
    }

    pub(super) fn submit_solid_ellipsoid_text(
        &mut self,
        mut pending: SolidPending,
        text: &str,
    ) -> Option<Result<()>> {
        let input = text.trim();
        let mode = input
            .strip_prefix("Mode=")
            .or_else(|| input.strip_prefix("mode="))
            .unwrap_or(input);
        if mode
            .trim_start_matches('_')
            .eq_ignore_ascii_case("aroundcurve")
            || mode.eq_ignore_ascii_case("a")
        {
            if !pending.points.is_empty() || !pending.values.is_empty() {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Invalid(
                    "choose AroundCurve before the first point",
                )));
            }
            pending.ellipsoid_around = true;
            pending.ellipsoid_corner = false;
            pending.ellipsoid_diameter = false;
            pending.ellipsoid_foci = false;
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if pending.ellipsoid_around
            && pending.points.is_empty()
            && input.to_ascii_lowercase().starts_with("oncurve=")
        {
            let value = input
                .split_once('=')
                .and_then(|(_, s)| s.parse::<f64>().ok());
            self.solid_pending = Some(pending);
            return Some(match value {
                Some(value) => self.solid_cone_around_center(value),
                None => Err(Error::Invalid("OnCurve fraction must be 0..1")),
            });
        }
        if mode
            .trim_start_matches('_')
            .eq_ignore_ascii_case("fromfoci")
            || mode.eq_ignore_ascii_case("f")
        {
            if !pending.points.is_empty() || !pending.values.is_empty() {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Invalid(
                    "choose FromFoci before the first point",
                )));
            }
            pending.ellipsoid_foci = true;
            pending.ellipsoid_diameter = false;
            pending.ellipsoid_corner = false;
            pending.ellipsoid_around = false;
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if pending.ellipsoid_foci && input.to_ascii_lowercase().starts_with("markfoci") {
            let value = match input.to_ascii_lowercase().as_str() {
                "markfoci" => Some(!pending.ellipsoid_mark_foci),
                "markfoci=yes" | "markfoci=on" | "markfoci=true" => Some(true),
                "markfoci=no" | "markfoci=off" | "markfoci=false" => Some(false),
                _ => None,
            };
            if let Some(value) = value {
                pending.ellipsoid_mark_foci = value;
                self.solid_pending = Some(pending);
                return Some(Ok(()));
            }
            self.solid_pending = Some(pending);
            return Some(Err(Error::Invalid("MarkFoci must be Yes or No")));
        }
        if mode.trim_start_matches('_').eq_ignore_ascii_case("corner")
            || mode.eq_ignore_ascii_case("c")
        {
            if !pending.points.is_empty() || !pending.values.is_empty() {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Invalid("choose Corner before the first point")));
            }
            pending.ellipsoid_corner = true;
            pending.ellipsoid_diameter = false;
            pending.ellipsoid_foci = false;
            pending.ellipsoid_around = false;
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if mode
            .trim_start_matches('_')
            .eq_ignore_ascii_case("diameter")
            || mode.trim_start_matches('_').eq_ignore_ascii_case("d")
        {
            if !pending.points.is_empty() || !pending.values.is_empty() {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Invalid(
                    "choose Diameter before the first point",
                )));
            }
            pending.ellipsoid_diameter = true;
            pending.ellipsoid_foci = false;
            pending.ellipsoid_corner = false;
            pending.ellipsoid_around = false;
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if input.eq_ignore_ascii_case("undo") || input.eq_ignore_ascii_case("u") {
            if pending.points.pop().is_none() {
                pending.values.pop();
            }
            if pending.points.is_empty() {
                pending.cone_plane = None;
            }
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if input.contains(',') {
            let point: std::result::Result<Vec<f64>, _> = input
                .split(',')
                .map(|part| part.trim().parse::<f64>())
                .collect();
            self.solid_pending = Some(pending);
            return Some(match point {
                Ok(v) if v.len() == 3 && v.iter().all(|v| v.is_finite()) => {
                    self.solid_ellipsoid_accept_point([v[0], v[1], v[2]])
                }
                _ => Err(Error::Invalid("ellipsoid point must be x,y,z")),
            });
        }
        if (pending.ellipsoid_diameter && pending.points.is_empty())
            || (pending.ellipsoid_foci && pending.points.len() < 3)
            || (pending.ellipsoid_corner && pending.points.len() < 2)
            || (pending.ellipsoid_around && pending.points.is_empty())
        {
            self.solid_pending = Some(pending);
            return Some(Err(Error::Invalid("enter a point for this stage")));
        }
        if !pending.points.is_empty() {
            let value = input
                .parse::<f64>()
                .ok()
                .filter(|v| v.is_finite() && *v > 1e-9);
            let Some(value) = value else {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Invalid("ellipsoid axis must be positive")));
            };
            let plane = pending
                .cone_plane
                .as_deref()
                .expect("ellipsoid frame")
                .plane;
            let center = pending.ellipsoid_center();
            let first = if pending.points.len() >= 2 {
                let d = std::array::from_fn::<_, 3, _>(|i| pending.points[1][i] - center[i]);
                let u = (0..3).map(|i| d[i] * plane.x[i]).sum::<f64>();
                let v = (0..3).map(|i| d[i] * plane.y[i]).sum::<f64>();
                std::array::from_fn(|i| (u * plane.x[i] + v * plane.y[i]) / u.hypot(v))
            } else {
                plane.x
            };
            let n = plane.normal();
            let axis = match (pending.ellipsoid_corner, pending.points.len()) {
                (true, _) => n,
                (_, 1) => first,
                (_, 2) => [
                    n[1] * first[2] - n[2] * first[1],
                    n[2] * first[0] - n[0] * first[2],
                    n[0] * first[1] - n[1] * first[0],
                ],
                _ => n,
            };
            let point = std::array::from_fn(|i| center[i] + value * axis[i]);
            self.solid_pending = Some(pending);
            return Some(self.solid_ellipsoid_accept_point(point));
        }
        self.submit_solid_generic_tail(pending, text)
    }

    pub(super) fn submit_solid_boss_text(
        &mut self,
        mut pending: SolidPending,
        text: &str,
    ) -> Option<Result<()>> {
        let input = text.trim();
        if input.eq_ignore_ascii_case("cancel") || input.eq_ignore_ascii_case("esc") {
            self.cancel();
            return Some(Ok(()));
        }
        if input.eq_ignore_ascii_case("undo") {
            if let Some(targets) = pending.difference_targets.take() {
                self.clear_selection();
                let ids: Vec<_> = targets.iter().map(|s| s.object).collect();
                let result = self.select_objects(&ids);
                self.solid_pending = Some(pending);
                return Some(result);
            }
            if let Some(id) = self.selected_ordered().last().copied() {
                let _ = self.remove_objects(&[id]);
            }
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if !input.is_empty() {
            self.solid_pending = Some(pending);
            return Some(Err(Error::Invalid("Boss requires selection or Enter")));
        }
        if pending.difference_targets.is_none() {
            let ids = self.selected_ordered();
            if ids.is_empty() {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Missing("select closed planar curves")));
            }
            if ids.len() as u64 > self.coordinator.limits().max_outputs {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Budget("Boss profiles")));
            }
            let mut sources = Vec::new();
            if sources.try_reserve_exact(ids.len()).is_err() {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Budget("Boss profiles")));
            }
            for id in ids {
                if let Err(e) = self.solid_boss_validate_profile(id) {
                    self.solid_pending = Some(pending);
                    return Some(Err(e));
                }
                match self.document().stamp(id) {
                    Ok(s) => sources.push(s),
                    Err(e) => {
                        self.solid_pending = Some(pending);
                        return Some(Err(e));
                    }
                }
            }
            pending.difference_targets = Some(sources);
            self.clear_selection();
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        let ids = self.selected_ordered();
        if ids.len() != 1 {
            self.solid_pending = Some(pending);
            return Some(Err(Error::Missing("select one planar boundary")));
        }
        let result = self.solid_boss_commit(pending.difference_targets.as_ref().unwrap(), ids[0]);
        if result.is_err() {
            self.clear_selection();
            self.solid_pending = Some(pending);
        }
        return Some(result);
    }

    pub(super) fn submit_solid_pipe_text(
        &mut self,
        mut pending: SolidPending,
        text: &str,
    ) -> Option<Result<()>> {
        let input = text.trim();
        if input.eq_ignore_ascii_case("cancel") || input.eq_ignore_ascii_case("esc") {
            if pending.pipe_chain {
                self.clear_subselection();
            }
            self.cancel();
            return Some(Ok(()));
        }
        if input.eq_ignore_ascii_case("undo") {
            if pending.pipe_multiple_selecting {
                if pending.pipe_chain {
                    match pending.pipe_chain_order.pop() {
                        Some(PipeChainItem::Edge(edge, _)) => {
                            let _ = self.toggle_subobject_ref(edge);
                        }
                        Some(PipeChainItem::Curve(source)) => {
                            let _ = self.remove_objects(&[source.object]);
                        }
                        None => {}
                    }
                } else if let Some(id) = self.selected_ordered().last().copied() {
                    let _ = self.remove_objects(&[id]);
                }
            } else if pending.pipe_station_position.take().is_none()
                && pending.pipe_stations.pop().is_none()
                && pending.values.pop().is_none()
            {
                self.clear_selection();
            }
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if input.eq_ignore_ascii_case("Multiple") || input.eq_ignore_ascii_case("ChainEdges") {
            if !pending.values.is_empty() || !pending.pipe_stations.is_empty() {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Invalid(
                    "Multiple must be selected before entering radii",
                )));
            }
            pending.pipe_multiple = true;
            pending.pipe_multiple_selecting = true;
            pending.pipe_chain = input.eq_ignore_ascii_case("ChainEdges");
            if pending.pipe_chain {
                self.clear_subselection();
                match self.solid_pipe_existing_chain_selection() {
                    Ok(items) => pending.pipe_chain_order = items,
                    Err(error) => {
                        self.solid_pending = Some(pending);
                        return Some(Err(error));
                    }
                }
            } else {
                self.clear_subselection();
                pending.pipe_chain_order.clear();
            }
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if input.eq_ignore_ascii_case("ShapeBlending=Global") {
            pending.pipe_global = true;
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if input.eq_ignore_ascii_case("ShapeBlending=Local") {
            if (pending.pipe_multiple || pending.pipe_chain)
                && (!pending.pipe_stations.is_empty() || pending.pipe_station_position.is_some())
            {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Invalid("Local stations require a single curve")));
            }
            pending.pipe_global = false;
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if input.eq_ignore_ascii_case("Diameter") || input.eq_ignore_ascii_case("Radius") {
            pending.pipe_diameter = input.eq_ignore_ascii_case("Diameter");
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        let thick = if input.eq_ignore_ascii_case("Thick") {
            Some(!pending.pipe_thick)
        } else if input.eq_ignore_ascii_case("Thick=Yes") {
            Some(true)
        } else if input.eq_ignore_ascii_case("Thick=No") {
            Some(false)
        } else {
            None
        };
        if let Some(thick) = thick {
            if thick != pending.pipe_thick
                && (!pending.values.is_empty() || !pending.pipe_stations.is_empty())
            {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Invalid(
                    "undo entered radii before changing Thick",
                )));
            }
            pending.pipe_thick = thick;
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if input.eq_ignore_ascii_case("Cap")
            || input.eq_ignore_ascii_case("Cap=None")
            || input.eq_ignore_ascii_case("Cap=Flat")
            || input.eq_ignore_ascii_case("Cap=Round")
        {
            let cap = if input.eq_ignore_ascii_case("Cap") {
                if pending.pipe_round {
                    "flat"
                } else if pending.pipe_capped {
                    "none"
                } else {
                    "round"
                }
            } else if input.eq_ignore_ascii_case("Cap=Flat") {
                "flat"
            } else if input.eq_ignore_ascii_case("Cap=Round") {
                "round"
            } else {
                "none"
            };
            pending.pipe_capped = cap != "none";
            pending.pipe_round = cap == "round";
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if input.to_ascii_lowercase().starts_with("cap=") {
            self.solid_pending = Some(pending);
            return Some(Err(Error::Invalid("Pipe cap must be None, Flat or Round")));
        }
        if pending.pipe_multiple_selecting {
            if input.is_empty()
                && (!self.selection.is_empty() || !pending.pipe_chain_order.is_empty())
                && self.selection.iter().all(|id| {
                    matches!(
                        self.document().entities.get(id).map(|e| &e.geometry),
                        Some(Geometry::Curve(_))
                    )
                })
            {
                if pending.pipe_chain
                    && let Err(error) = self.solid_pipe_chain_points(&pending)
                {
                    self.solid_pending = Some(pending);
                    return Some(Err(error));
                }
                pending.pipe_multiple_selecting = false;
                self.solid_pending = Some(pending);
                return Some(Ok(()));
            }
            self.solid_pending = Some(pending);
            return Some(Err(Error::Missing("select curves and press Enter")));
        }
        if (self.selection.is_empty() && pending.pipe_chain_order.is_empty())
            || (!pending.pipe_multiple && self.selection.len() != 1)
            || !self.selection.iter().all(|id| {
                matches!(
                    self.document().entities.get(id).map(|e| &e.geometry),
                    Some(Geometry::Curve(_))
                )
            })
        {
            self.solid_pending = Some(pending);
            return Some(Err(Error::Missing("select one curve")));
        }
        if input.contains(',') {
            let point = (|| {
                let plane = self.input_plane();
                let input = om9_core::coords::parse_input(input, self.document().page_unit)?;
                let point =
                    om9_core::coords::resolve_with_base(input, Some(&plane), self.relative_base())?;
                Ok(point.xyz())
            })();
            self.solid_pending = Some(pending);
            return Some(point.and_then(|point| self.solid_pipe_accept_radius_point(point)));
        }
        {
            let count = if pending.pipe_thick { 4 } else { 2 };
            if input.is_empty()
                && pending.values.len() == count
                && pending.pipe_station_position.is_none()
            {
                // Mouse-to-world projection may differ by a few microns for
                // geometrically equal radii. Respect document tolerance.
                if !pending.pipe_global
                    || (pending.pipe_chain
                        && matches!(
                            self.solid_pipe_chain_endpoints(&pending),
                            Err(Error::Unsupported(_))
                        ))
                {
                    let tolerance = self.document().tolerance().map_or_else(
                        || om9_core::history::Tolerance::om9_template().absolute_mm,
                        |t| t.absolute_mm,
                    );
                    if pending.pipe_thick {
                        let start = if pending.values[0] <= pending.values[1] {
                            [0, 1]
                        } else {
                            [1, 0]
                        };
                        let end = if pending.values[2] <= pending.values[3] {
                            [2, 3]
                        } else {
                            [3, 2]
                        };
                        for (first, second) in start.into_iter().zip(end) {
                            if (pending.values[first] - pending.values[second]).abs() <= tolerance {
                                pending.values[second] = pending.values[first];
                            }
                        }
                    } else if (pending.values[0] - pending.values[1]).abs() <= tolerance {
                        pending.values[1] = pending.values[0];
                    }
                }
                if pending.pipe_thick
                    && !pending.pipe_global
                    && (pending.values[0].min(pending.values[1])
                        != pending.values[2].min(pending.values[3])
                        || pending.values[0].max(pending.values[1])
                            != pending.values[2].max(pending.values[3]))
                {
                    if pending.pipe_multiple
                        || pending.pipe_chain
                        || !pending.pipe_stations.is_empty()
                    {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Unsupported(
                            "varying Local Thick Pipe requires a single curve",
                        )));
                    }
                    let result = self.solid_pipe_local(&pending);
                    if result.is_err() {
                        self.solid_pending = Some(pending);
                    }
                    return Some(result);
                }
                if !pending.pipe_thick
                    && !pending.pipe_global
                    && pending.pipe_stations.is_empty()
                    && pending.values[0] != pending.values[1]
                {
                    if pending.pipe_multiple
                        || pending.pipe_chain
                        || !pending.pipe_stations.is_empty()
                    {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Unsupported(
                            "varying Local Pipe requires a single curve",
                        )));
                    }
                    let result = self.solid_pipe_local(&pending);
                    if result.is_err() {
                        self.solid_pending = Some(pending);
                    }
                    return Some(result);
                }
                if !pending.pipe_stations.is_empty() {
                    if pending.pipe_thick
                        || (pending.pipe_round && pending.pipe_chain)
                        || (!pending.pipe_global && (pending.pipe_multiple || pending.pipe_chain))
                    {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Unsupported(
                            "intermediate stations require non-Thick Pipe; Round Chain and Local Multiple are unavailable",
                        )));
                    }
                    let result = self.solid_pipe_commit_stations(&pending);
                    if result.is_ok() && pending.pipe_chain {
                        self.clear_subselection();
                    }
                    if result.is_err() {
                        self.solid_pending = Some(pending);
                    }
                    return Some(result);
                }
                if pending.pipe_multiple {
                    let result = if pending.pipe_chain {
                        self.solid_pipe_commit_chain(&pending)
                    } else {
                        self.solid_pipe_commit_multiple(&pending)
                    };
                    if result.is_ok() && pending.pipe_chain {
                        self.clear_subselection();
                    }
                    if result.is_err() {
                        self.solid_pending = Some(pending);
                    }
                    return Some(result);
                }
                let input_values = if !pending.pipe_thick && pending.values[0] == pending.values[1]
                {
                    &pending.values[..1]
                } else {
                    &pending.values[..]
                };
                let mut args: Vec<String> =
                    input_values.iter().map(|value| value.to_string()).collect();
                args.push(
                    if pending.pipe_round {
                        "Cap=Round"
                    } else if pending.pipe_capped {
                        "Cap=Flat"
                    } else {
                        "Cap=None"
                    }
                    .to_owned(),
                );
                let refs: Vec<&str> = args.iter().map(String::as_str).collect();
                let result = self.dispatch_solid_args(pending.feature, &refs);
                if result.is_err() {
                    self.solid_pending = Some(pending);
                }
                return Some(result);
            }
            if pending.values.len() >= count && pending.pipe_station_position.is_none() {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Unsupported(
                    "Pipe intermediate radii are not yet available",
                )));
            }
            let value = input
                .parse::<f64>()
                .ok()
                .map(|v| if pending.pipe_diameter { v / 2.0 } else { v })
                .filter(|v| v.is_finite() && *v > 0.0);
            if let Some(value) = value {
                if let Some(fraction) = pending.pipe_station_position {
                    if pending.pipe_stations.len() as u64 >= self.coordinator.limits().max_points {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Budget("Pipe station count")));
                    }
                    if pending.pipe_stations.try_reserve(1).is_err() {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Budget("Pipe stations")));
                    }
                    pending.pipe_stations.push((fraction, value));
                    pending.pipe_station_position = None;
                } else {
                    pending.values.push(value);
                }
                self.solid_pending = Some(pending);
                return Some(Ok(()));
            }
            self.solid_pending = Some(pending);
            return Some(Err(Error::Invalid(
                "Pipe radius must be finite and positive",
            )));
        }
    }

    pub(super) fn submit_solid_fillet_text(
        &mut self,
        mut pending: SolidPending,
        text: &str,
    ) -> Option<Result<()>> {
        let input = text.trim();
        if input.eq_ignore_ascii_case("cancel") || input.eq_ignore_ascii_case("esc") {
            self.clear_subselection();
            self.cancel();
            return Some(Ok(()));
        }
        if input.eq_ignore_ascii_case("CopyHandle") {
            if pending.fillet_preview.is_none()
                || pending.fillet_add_mode
                || pending.fillet_remove_mode
                || pending.fillet_copy_phase != 0
            {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Missing("fillet preview")));
            }
            pending.fillet_copy_phase = 1;
            pending.fillet_handle = None;
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if input.eq_ignore_ascii_case("AddHandle") {
            if pending.fillet_preview.is_none()
                || pending.fillet_add_mode
                || pending.fillet_remove_mode
                || pending.fillet_copy_phase != 0
            {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Missing("fillet preview")));
            }
            pending.fillet_add_radius = pending
                .fillet_handle
                .and_then(|handle| {
                    let base = pending.fillet_edges.len().checked_mul(2)?;
                    if handle >= base {
                        pending.fillet_controls.get(handle - base).map(|c| c.radius)
                    } else {
                        pending.fillet_edges.get(handle / 2).map(|e| {
                            if handle % 2 == 0 {
                                e.start_radius
                            } else {
                                e.end_radius
                            }
                        })
                    }
                })
                .or(pending.fillet_radius);
            pending.fillet_add_mode = true;
            pending.fillet_handle = None;
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if input.eq_ignore_ascii_case("RemoveHandle") {
            if pending.fillet_preview.is_none()
                || pending.fillet_controls.is_empty()
                || pending.fillet_add_mode
                || pending.fillet_remove_mode
                || pending.fillet_copy_phase != 0
            {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Missing("added fillet handles")));
            }
            pending.fillet_remove_mode = true;
            pending.fillet_handle = None;
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if input.eq_ignore_ascii_case("SelectEdges") {
            let Some(preview) = &pending.fillet_preview else {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Missing("fillet preview")));
            };
            if self.document().stamp(preview.source.object).ok().as_ref() != Some(&preview.source)
                || !self.selection.contains(&preview.source.object)
            {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Stale("fillet preview source")));
            }
            pending.fillet_preview = None;
            pending.fillet_handle = None;
            pending.fillet_add_mode = false;
            pending.fillet_remove_mode = false;
            pending.fillet_copy_phase = 0;
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if input.eq_ignore_ascii_case("undo") || input.eq_ignore_ascii_case("u") {
            if pending.fillet_add_mode
                || pending.fillet_remove_mode
                || pending.fillet_copy_phase != 0
            {
                pending.fillet_add_mode = false;
                pending.fillet_remove_mode = false;
                pending.fillet_copy_phase = 0;
                self.solid_pending = Some(pending);
                return Some(Ok(()));
            }
            if pending.fillet_preview.take().is_none() {
                if pending.fillet_edges.pop().is_none() {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Missing("no fillet edge to undo")));
                }
                pending
                    .fillet_controls
                    .retain(|c| c.edge < pending.fillet_edges.len());
                self.clear_subselection();
                for edge in &pending.fillet_edges {
                    if let Err(error) = self.select_subobject_ref(edge.reference.clone()) {
                        self.solid_pending = Some(pending);
                        return Some(Err(error));
                    }
                }
            } else {
                pending.fillet_handle = None;
            }
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if let Some((name, value)) = input.split_once('=') {
            if name.eq_ignore_ascii_case("LinkHandles") {
                if pending.fillet_preview.is_none() {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Missing("fillet preview")));
                }
                let next = if value.eq_ignore_ascii_case("yes") {
                    Some(true)
                } else if value.eq_ignore_ascii_case("no") {
                    Some(false)
                } else {
                    None
                };
                let Some(next) = next else {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Invalid("LinkHandles must be Yes or No")));
                };
                pending.fillet_link_handles = next;
                self.solid_pending = Some(pending);
                return Some(Ok(()));
            }
            if name.eq_ignore_ascii_case("SetAll")
                || (name.eq_ignore_ascii_case("CurrentRadius")
                    && pending.fillet_link_handles
                    && pending.fillet_handle.is_some()
                    && pending.fillet_copy_phase == 0
                    && !pending.fillet_add_mode)
            {
                let Some(radius) = value
                    .parse::<f64>()
                    .ok()
                    .filter(|v| v.is_finite() && *v > 0.0)
                else {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Invalid(
                        if name.eq_ignore_ascii_case("CurrentRadius") {
                            "CurrentRadius must be positive"
                        } else {
                            "SetAll radius must be positive"
                        },
                    )));
                };
                let Some(old_preview) = &pending.fillet_preview else {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Missing("fillet preview")));
                };
                if self
                    .document()
                    .stamp(old_preview.source.object)
                    .ok()
                    .as_ref()
                    != Some(&old_preview.source)
                    || !self.selection.contains(&old_preview.source.object)
                {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Stale("fillet preview source")));
                }
                for edge in &pending.fillet_edges {
                    if let Err(error) = self.select_subobject_ref(edge.reference.clone()) {
                        self.solid_pending = Some(pending);
                        return Some(Err(error));
                    }
                }
                let mut backup_usage = match om9_core::budget::Usage::array::<(f64, f64)>(
                    pending.fillet_edges.len(),
                ) {
                    Ok(usage) => usage,
                    Err(error) => {
                        self.solid_pending = Some(pending);
                        return Some(Err(error));
                    }
                };
                if let Err(error) = backup_usage.add(
                    match om9_core::budget::Usage::array::<f64>(pending.fillet_controls.len()) {
                        Ok(usage) => usage,
                        Err(error) => {
                            self.solid_pending = Some(pending);
                            return Some(Err(error));
                        }
                    },
                    self.coordinator.limits(),
                ) {
                    self.solid_pending = Some(pending);
                    return Some(Err(error));
                }
                let checked = self.scene_cache_usage().and_then(|mut held| {
                    held.add(old_preview.usage, self.coordinator.limits())?;
                    held.add(backup_usage, self.coordinator.limits())?;
                    self.coordinator.retained_usage_with_held_usage(held)?;
                    Ok(())
                });
                if let Err(error) = checked {
                    self.solid_pending = Some(pending);
                    return Some(Err(error));
                }
                let mut previous = Vec::new();
                if previous
                    .try_reserve_exact(pending.fillet_edges.len())
                    .is_err()
                {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Budget("fillet radii backup")));
                }
                let mut previous_controls = Vec::new();
                if previous_controls
                    .try_reserve_exact(pending.fillet_controls.len())
                    .is_err()
                {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Budget("fillet control backup")));
                }
                for edge in &mut pending.fillet_edges {
                    previous.push((edge.start_radius, edge.end_radius));
                    edge.start_radius = radius;
                    edge.end_radius = radius;
                }
                for control in &mut pending.fillet_controls {
                    previous_controls.push(control.radius);
                    control.radius = radius;
                }
                let result = self
                    .solid_fillet_build_preview(&pending.fillet_edges, &pending.fillet_controls)
                    .and_then(|preview| {
                        let mut held = self.scene_cache_usage()?;
                        held.add(old_preview.usage, self.coordinator.limits())?;
                        held.add(backup_usage, self.coordinator.limits())?;
                        held.add(preview.usage, self.coordinator.limits())?;
                        self.coordinator.retained_usage_with_held_usage(held)?;
                        Ok(preview)
                    });
                match result {
                    Ok(preview) => {
                        pending.fillet_preview = Some(preview);
                        self.solid_pending = Some(pending);
                        return Some(Ok(()));
                    }
                    Err(error) => {
                        for (edge, (start, end)) in pending.fillet_edges.iter_mut().zip(previous) {
                            edge.start_radius = start;
                            edge.end_radius = end;
                        }
                        for (control, radius) in
                            pending.fillet_controls.iter_mut().zip(previous_controls)
                        {
                            control.radius = radius;
                        }
                        self.solid_pending = Some(pending);
                        return Some(Err(error));
                    }
                }
            }
            if name.eq_ignore_ascii_case("ShowRadius") {
                let next = if value.eq_ignore_ascii_case("yes") {
                    Some(true)
                } else if value.eq_ignore_ascii_case("no") {
                    Some(false)
                } else {
                    None
                };
                let Some(next) = next else {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Invalid("ShowRadius must be Yes or No")));
                };
                pending.fillet_show_radius = next;
                self.solid_pending = Some(pending);
                return Some(Ok(()));
            }
            if name.eq_ignore_ascii_case("CurrentRadius") {
                if pending.fillet_copy_phase != 0 {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Invalid("select CopyHandle target")));
                }
                if pending.fillet_add_mode {
                    let Some(radius) = value
                        .parse::<f64>()
                        .ok()
                        .filter(|v| v.is_finite() && *v > 0.0)
                    else {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Invalid("CurrentRadius must be positive")));
                    };
                    pending.fillet_add_radius = Some(radius);
                    self.solid_pending = Some(pending);
                    return Some(Ok(()));
                }
                let Some(index) = pending.fillet_handle else {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Missing("select a fillet handle")));
                };
                let Some(radius) = value
                    .parse::<f64>()
                    .ok()
                    .filter(|v| v.is_finite() && *v > 0.0)
                else {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Invalid("CurrentRadius must be positive")));
                };
                let Some(old_preview) = &pending.fillet_preview else {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Missing("fillet preview")));
                };
                if self
                    .document()
                    .stamp(old_preview.source.object)
                    .ok()
                    .as_ref()
                    != Some(&old_preview.source)
                    || !self.selection.contains(&old_preview.source.object)
                {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Stale("fillet preview source")));
                }
                for edge in &pending.fillet_edges {
                    if let Err(error) = self.select_subobject_ref(edge.reference.clone()) {
                        self.solid_pending = Some(pending);
                        return Some(Err(error));
                    }
                }
                let base = pending.fillet_edges.len().saturating_mul(2);
                if index >= base {
                    let Some(control) = pending.fillet_controls.get_mut(index - base) else {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Invalid("fillet handle")));
                    };
                    let previous = control.radius;
                    control.radius = radius;
                    let result = self
                        .solid_fillet_build_preview(&pending.fillet_edges, &pending.fillet_controls)
                        .and_then(|preview| {
                            let mut held = self.scene_cache_usage()?;
                            held.add(old_preview.usage, self.coordinator.limits())?;
                            held.add(preview.usage, self.coordinator.limits())?;
                            self.coordinator.retained_usage_with_held_usage(held)?;
                            Ok(preview)
                        });
                    match result {
                        Ok(preview) => {
                            pending.fillet_preview = Some(preview);
                            self.solid_pending = Some(pending);
                            return Some(Ok(()));
                        }
                        Err(error) => {
                            pending.fillet_controls[index - base].radius = previous;
                            self.solid_pending = Some(pending);
                            return Some(Err(error));
                        }
                    }
                }
                let anticipated = self.scene_cache_usage().and_then(|mut held| {
                    held.add(old_preview.usage, self.coordinator.limits())?;
                    held.add(
                        om9_core::budget::Usage::array::<u8>(pending.fillet_edges.len())?,
                        self.coordinator.limits(),
                    )?;
                    held.add(
                        om9_core::budget::Usage::array::<usize>(pending.fillet_edges.len())?,
                        self.coordinator.limits(),
                    )?;
                    held.add(
                        om9_core::budget::Usage::array::<i32>(pending.fillet_edges.len())?,
                        self.coordinator.limits(),
                    )?;
                    self.coordinator.retained_usage_with_held_usage(held)?;
                    Ok(())
                });
                if let Err(error) = anticipated {
                    self.solid_pending = Some(pending);
                    return Some(Err(error));
                }
                let masks =
                    match self.solid_fillet_shared_endpoint_masks(&pending.fillet_edges, index) {
                        Ok(masks) => masks,
                        Err(error) => {
                            self.solid_pending = Some(pending);
                            return Some(Err(error));
                        }
                    };
                let backup_usage = match om9_core::budget::Usage::array::<(f64, f64)>(
                    pending.fillet_edges.len(),
                ) {
                    Ok(usage) => usage,
                    Err(error) => {
                        self.solid_pending = Some(pending);
                        return Some(Err(error));
                    }
                };
                let mask_usage = match om9_core::budget::Usage::array::<u8>(masks.capacity()) {
                    Ok(usage) => usage,
                    Err(error) => {
                        self.solid_pending = Some(pending);
                        return Some(Err(error));
                    }
                };
                let checked = self.scene_cache_usage().and_then(|mut held| {
                    held.add(old_preview.usage, self.coordinator.limits())?;
                    held.add(backup_usage, self.coordinator.limits())?;
                    held.add(mask_usage, self.coordinator.limits())?;
                    self.coordinator.retained_usage_with_held_usage(held)?;
                    Ok(())
                });
                if let Err(error) = checked {
                    self.solid_pending = Some(pending);
                    return Some(Err(error));
                }
                let mut previous = Vec::new();
                if previous
                    .try_reserve_exact(pending.fillet_edges.len())
                    .is_err()
                {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Budget("fillet radii backup")));
                }
                for (edge, mask) in pending.fillet_edges.iter_mut().zip(&masks) {
                    previous.push((edge.start_radius, edge.end_radius));
                    if mask & 1 != 0 {
                        edge.start_radius = radius;
                    }
                    if mask & 2 != 0 {
                        edge.end_radius = radius;
                    }
                }
                let result = self
                    .solid_fillet_build_preview(&pending.fillet_edges, &pending.fillet_controls)
                    .and_then(|preview| {
                        let mut held = self.scene_cache_usage()?;
                        held.add(old_preview.usage, self.coordinator.limits())?;
                        held.add(backup_usage, self.coordinator.limits())?;
                        held.add(mask_usage, self.coordinator.limits())?;
                        held.add(preview.usage, self.coordinator.limits())?;
                        self.coordinator.retained_usage_with_held_usage(held)?;
                        Ok(preview)
                    });
                match result {
                    Ok(preview) => {
                        pending.fillet_preview = Some(preview);
                        self.solid_pending = Some(pending);
                        return Some(Ok(()));
                    }
                    Err(error) => {
                        for (edge, (start, end)) in pending.fillet_edges.iter_mut().zip(previous) {
                            edge.start_radius = start;
                            edge.end_radius = end;
                        }
                        self.solid_pending = Some(pending);
                        return Some(Err(error));
                    }
                }
            }
            if name.eq_ignore_ascii_case("NextRadius") {
                let parsed = value
                    .parse::<f64>()
                    .ok()
                    .filter(|v| v.is_finite() && *v > 0.0);
                let Some(radius) = parsed else {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Invalid("NextRadius must be positive")));
                };
                if pending.fillet_preview.is_some() {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Unsupported(
                        "Undo preview before changing NextRadius",
                    )));
                }
                pending.fillet_radius = Some(radius);
                self.solid_pending = Some(pending);
                return Some(Ok(()));
            }
        }
        if input.is_empty() {
            if pending.fillet_add_mode
                || pending.fillet_remove_mode
                || pending.fillet_copy_phase != 0
            {
                pending.fillet_add_mode = false;
                pending.fillet_remove_mode = false;
                pending.fillet_copy_phase = 0;
                self.solid_pending = Some(pending);
                return Some(Ok(()));
            }
            if let Some(preview) = pending.fillet_preview.take() {
                let revision = self.document().revision;
                let result = self.solid_fillet_commit(&preview);
                if result.is_err() && self.document().revision == revision {
                    pending.fillet_preview = Some(preview);
                    self.solid_pending = Some(pending);
                } else if result.is_ok() {
                    self.clear_subselection();
                }
                return Some(result);
            }
            if pending.fillet_edges.is_empty() || pending.fillet_radius.is_none() {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Missing("select edges and set NextRadius")));
            }
            for edge in &pending.fillet_edges {
                if let Err(error) = self.select_subobject_ref(edge.reference.clone()) {
                    self.solid_pending = Some(pending);
                    return Some(Err(error));
                }
                if !self.selection.contains(&edge.reference.root) {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Stale("fillet selection")));
                }
            }
            let result =
                self.solid_fillet_build_preview(&pending.fillet_edges, &pending.fillet_controls);
            match result {
                Ok(preview) => {
                    pending.fillet_preview = Some(preview);
                    self.solid_pending = Some(pending);
                    return Some(Ok(()));
                }
                Err(error) => {
                    self.solid_pending = Some(pending);
                    return Some(Err(error));
                }
            }
        }
        let words: Vec<_> = input.split_whitespace().collect();
        if words.len() == 2 && pending.fillet_edges.is_empty() {
            let revision = self.document().revision;
            let result = self.dispatch_solid_args(pending.feature, &words);
            if result.is_err() && self.document().revision == revision {
                self.solid_pending = Some(pending);
            }
            return Some(result);
        }
        self.solid_pending = Some(pending);
        return Some(Err(Error::Invalid(
            "set NextRadius, select edges, then Enter",
        )));
    }

    pub(super) fn submit_solid_shell_text(
        &mut self,
        mut pending: SolidPending,
        text: &str,
    ) -> Option<Result<()>> {
        let input = text.trim();
        if input.eq_ignore_ascii_case("cancel") || input.eq_ignore_ascii_case("esc") {
            self.clear_subselection();
            self.cancel();
            return Some(Ok(()));
        }
        if input.eq_ignore_ascii_case("undo") || input.eq_ignore_ascii_case("u") {
            if pending.shell_select_done {
                pending.shell_select_done = false;
            } else if pending.extract_face_picks.pop().is_none() {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Missing("no selected face to undo")));
            }
            self.clear_subselection();
            for reference in &pending.extract_face_picks {
                if let Err(error) = self.select_subobject_ref(reference.clone()) {
                    self.solid_pending = Some(pending);
                    return Some(Err(error));
                }
            }
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        let words: Vec<_> = input.split_whitespace().collect();
        if words.len() == 2 && pending.extract_face_picks.is_empty() {
            let revision = self.document().revision;
            let result = self.dispatch_solid_args(pending.feature, &words);
            if result.is_err() && self.document().revision == revision {
                self.solid_pending = Some(pending);
            }
            return Some(result);
        }
        if let Some((name, value)) = input.split_once('=') {
            if name.eq_ignore_ascii_case("Thickness") {
                let Some(thickness) = value
                    .parse::<f64>()
                    .ok()
                    .filter(|v| v.is_finite() && *v > 0.0)
                else {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Invalid("Thickness must be positive")));
                };
                pending.shell_thickness = Some(thickness);
                self.solid_pending = Some(pending);
                return Some(Ok(()));
            }
        }
        if pending.shell_select_done && !input.is_empty() {
            let Some(thickness) = input
                .parse::<f64>()
                .ok()
                .filter(|v| v.is_finite() && *v > 0.0)
            else {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Invalid("Thickness must be positive")));
            };
            pending.shell_thickness = Some(thickness);
        } else if !input.is_empty() {
            self.solid_pending = Some(pending);
            return Some(Err(Error::Invalid("select faces and press Enter")));
        }
        if pending.extract_face_picks.is_empty() {
            self.solid_pending = Some(pending);
            return Some(Err(Error::Missing("select a B-Rep face")));
        }
        pending.shell_select_done = true;
        let Some(thickness) = pending.shell_thickness else {
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        };
        for reference in &pending.extract_face_picks {
            if let Err(error) = self.select_subobject_ref(reference.clone()) {
                self.solid_pending = Some(pending);
                return Some(Err(error));
            }
        }
        let revision = self.document().revision;
        let result = self.solid_shell_picks(&pending.extract_face_picks, thickness);
        if result.is_err() && self.document().revision == revision {
            self.solid_pending = Some(pending);
        } else if result.is_ok() {
            self.clear_subselection();
        }
        return Some(result);
    }

    pub(super) fn submit_solid_extract_text(
        &mut self,
        mut pending: SolidPending,
        text: &str,
    ) -> Option<Result<()>> {
        let input = text.trim().to_ascii_lowercase();
        match input.as_str() {
            "cancel" | "esc" => {
                self.clear_subselection();
                self.cancel();
                return Some(Ok(()));
            }
            "undo" | "u" => {
                if pending.extract_face_picks.pop().is_none() {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Missing("no selected face to undo")));
                }
                self.clear_subselection();
                for reference in &pending.extract_face_picks {
                    if let Err(error) = self.select_subobject_ref(reference.clone()) {
                        self.solid_pending = Some(pending);
                        return Some(Err(error));
                    }
                }
            }
            "copy" | "c" => pending.extract_copy = !pending.extract_copy,
            "copy=yes" | "copy=true" => pending.extract_copy = true,
            "copy=no" | "copy=false" => pending.extract_copy = false,
            "outputlayer" | "o" => pending.extract_input_layer = !pending.extract_input_layer,
            "outputlayer=input" => pending.extract_input_layer = true,
            "outputlayer=current" => pending.extract_input_layer = false,
            _ => {
                let copy = if pending.extract_copy {
                    "Copy=Yes"
                } else {
                    "Copy=No"
                };
                let layer = if pending.extract_input_layer {
                    "OutputLayer=Input"
                } else {
                    "OutputLayer=Current"
                };
                let revision = self.document().revision;
                if input.is_empty() {
                    if pending.extract_face_picks.is_empty() {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Missing("select a B-Rep face")));
                    }
                    for reference in &pending.extract_face_picks {
                        if let Err(error) = self.select_subobject_ref(reference.clone()) {
                            self.solid_pending = Some(pending);
                            return Some(Err(error));
                        }
                        if !self.selection.contains(&reference.root) {
                            self.solid_pending = Some(pending);
                            return Some(Err(Error::Stale("extract selection")));
                        }
                    }
                }
                let result = if input.is_empty() {
                    self.solid_extract_picks(
                        &pending.extract_face_picks,
                        pending.extract_copy,
                        pending.extract_input_layer,
                    )
                } else {
                    self.dispatch_solid_args(pending.feature, &[text.trim(), copy, layer])
                };
                if result.is_err() && self.document().revision == revision {
                    self.solid_pending = Some(pending);
                } else if result.is_ok() {
                    self.clear_subselection();
                }
                return Some(result);
            }
        }
        self.solid_pending = Some(pending);
        return Some(Ok(()));
    }

    pub(super) fn submit_solid_cap_text(
        &mut self,
        pending: SolidPending,
        text: &str,
    ) -> Option<Result<()>> {
        let input = text.trim();
        if input.eq_ignore_ascii_case("cancel") || input.eq_ignore_ascii_case("esc") {
            self.cancel();
            return Some(Ok(()));
        }
        if input.eq_ignore_ascii_case("undo") || input.eq_ignore_ascii_case("u") {
            let result = match self.selected_ordered().last().copied() {
                Some(id) => self.remove_objects(&[id]),
                None => Err(Error::Missing("no Cap Planar input to undo")),
            };
            self.solid_pending = Some(pending);
            return Some(result);
        }
        if !input.is_empty() {
            self.solid_pending = Some(pending);
            return Some(Err(Error::Invalid("select surfaces and press Enter")));
        }
        let revision = self.document().revision;
        let result = self.solid_cap(pending.feature);
        if result.is_err() && self.document().revision == revision {
            self.solid_pending = Some(pending);
        }
        return Some(result);
    }

    pub(super) fn submit_solid_boolean_text(
        &mut self,
        mut pending: SolidPending,
        text: &str,
    ) -> Option<Result<()>> {
        let input = text.trim().to_ascii_lowercase();
        let delete_value = match input.as_str() {
            "deleteinput" | "d" => Some(!pending.difference_delete_input),
            "deleteinput=yes" | "deleteinput=true" => Some(true),
            "deleteinput=no" | "deleteinput=false" => Some(false),
            _ => None,
        };
        if let Some(value) = delete_value {
            pending.difference_delete_input = value;
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if input.starts_with("deleteinput=") {
            self.solid_pending = Some(pending);
            return Some(Err(Error::Invalid("DeleteInput must be Yes or No")));
        }
        if text.trim().eq_ignore_ascii_case("cancel") {
            self.cancel();
            return Some(Ok(()));
        }
        if matches!(text.trim().to_ascii_lowercase().as_str(), "undo" | "u") {
            if let Some(id) = self.selected_ordered().last().copied() {
                let result = self.remove_objects(&[id]);
                self.solid_pending = Some(pending);
                return Some(result);
            }
            if let Some(targets) = pending.difference_targets.as_ref() {
                let mut ids = Vec::new();
                if ids.try_reserve_exact(targets.len()).is_err() {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Budget("boolean targets")));
                }
                for target in targets {
                    if self.document().stamp(target.object).ok().as_ref() != Some(target) {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Stale("Boolean first set")));
                    }
                    ids.push(target.object);
                }
                let result = self.select_objects(&ids);
                if result.is_ok() {
                    pending.difference_targets = None;
                }
                self.solid_pending = Some(pending);
                return Some(result);
            }
            self.solid_pending = Some(pending);
            return Some(Err(Error::Missing("no Boolean input to undo")));
        }
        if !text.trim().is_empty() {
            self.solid_pending = Some(pending);
            return Some(Err(Error::Invalid("select objects and press Enter")));
        }
        if pending.feature == "SolidCreateSolid" {
            let revision = self.document().revision;
            let result = self.solid_create_selected(pending.difference_delete_input);
            if result.is_err() && self.document().revision == revision {
                self.solid_pending = Some(pending);
            }
            return Some(result);
        }
        if pending.feature == "OM9-SOLID-003" {
            let revision = self.document().revision;
            let result = self.solid_union_selected(pending.difference_delete_input);
            if result.is_err() && self.document().revision == revision {
                self.solid_pending = Some(pending);
            }
            return Some(result);
        }
        if pending.difference_targets.is_none() {
            let ids = self.selected_ordered();
            if ids.is_empty() {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Missing("select the first Boolean set")));
            }
            let result = self.difference_capture_targets(&ids);
            match result {
                Ok(targets) => {
                    pending.difference_targets = Some(targets);
                    self.clear_selection();
                    self.solid_pending = Some(pending);
                    return Some(Ok(()));
                }
                Err(error) => {
                    self.solid_pending = Some(pending);
                    return Some(Err(error));
                }
            }
        }
        let revision = self.document().revision;
        let result = if pending.feature == "SolidBooleanSplit" {
            self.solid_split_groups(
                pending.difference_targets.as_deref().expect("target stage"),
                &self.selected_ordered(),
                pending.difference_delete_input,
            )
        } else {
            self.solid_binary_groups(
                pending.feature,
                pending.difference_targets.as_deref().expect("target stage"),
                &self.selected_ordered(),
                pending.difference_delete_input,
            )
        };
        if result.is_err() && self.document().revision == revision {
            self.solid_pending = Some(pending);
        }
        return Some(result);
    }

    pub(super) fn submit_solid_box_text(
        &mut self,
        mut pending: SolidPending,
        text: &str,
    ) -> Option<Result<()>> {
        let raw = text.trim().to_ascii_lowercase();
        let option = raw.strip_prefix("mode=").unwrap_or(&raw).to_string();
        if option == "corners" {
            self.solid_pending = Some(pending);
            return Some(Err(Error::Invalid(
                "Corners is unavailable. Choose Diagonal, 3Point, Vertical or Center",
            )));
        }
        let selected = match option.as_str() {
            "diagonal" | "d" => Some("Diagonal"),
            "3point" | "3p" => Some("ThreePoint"),
            "3" if pending.points.is_empty() => Some("ThreePoint"),
            "vertical" | "v" => Some("Vertical"),
            "center" | "c" => Some("Center"),
            "cube" => Some("Cube"),
            _ => None,
        };
        if let Some(kind) = selected {
            if !pending.points.is_empty() {
                if kind == "Cube" && pending.points.len() == 1 && pending.box_kind == "Diagonal" {
                    pending.box_kind = "Cube";
                    self.solid_pending = Some(pending);
                    return Some(Ok(()));
                }
                self.solid_pending = Some(pending);
                return Some(Err(Error::Invalid(
                    "select the construction mode before the first point",
                )));
            }
            pending.box_kind = kind;
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if option == "undo" || option == "u" {
            if pending.box_width.take().is_none() {
                if pending.box_length > 0.0 {
                    pending.box_length = 0.0;
                } else {
                    pending.points.pop();
                    if pending.points.is_empty() {
                        pending.box_plane = None;
                    }
                }
            }
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if option == "cancel" || option == "esc" {
            return Some(Ok(()));
        }
        if pending.box_kind == "Center" && pending.points.len() == 1 {
            let parse = |value: &str| {
                value
                    .trim()
                    .parse::<f64>()
                    .ok()
                    .filter(|value| value.is_finite() && *value > 0.0)
            };
            if pending.box_length <= 0.0 {
                let value = text
                    .trim()
                    .strip_prefix("Length=")
                    .or_else(|| text.trim().strip_prefix("L="))
                    .unwrap_or(text.trim());
                let Some(length) = parse(value) else {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Invalid("length must be positive")));
                };
                let length = match self.document().page_unit.length_to_mm(length) {
                    Ok(v) => v,
                    Err(e) => {
                        self.solid_pending = Some(pending);
                        return Some(Err(e));
                    }
                };
                pending.box_length = length;
                self.solid_pending = Some(pending);
                return Some(Ok(()));
            }
            if pending.box_width.is_none() {
                let value = text
                    .trim()
                    .strip_prefix("Width=")
                    .or_else(|| text.trim().strip_prefix("W="))
                    .unwrap_or(text.trim());
                let width = if text.trim().is_empty() {
                    pending.box_length
                } else if let Some(width) = parse(value) {
                    match self.document().page_unit.length_to_mm(width) {
                        Ok(v) => v,
                        Err(e) => {
                            self.solid_pending = Some(pending);
                            return Some(Err(e));
                        }
                    }
                } else {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Invalid("width must be positive")));
                };
                pending.box_width = Some(width);
                self.solid_pending = Some(pending);
                return Some(Ok(()));
            }
            let height_value = text
                .trim()
                .strip_prefix("Height=")
                .or_else(|| text.trim().strip_prefix("H="))
                .unwrap_or(text.trim());
            let height = if text.trim().is_empty() {
                pending.box_width.unwrap_or(pending.box_length)
            } else if let Some(height) = height_value
                .parse::<f64>()
                .ok()
                .filter(|v| v.is_finite() && *v != 0.0)
            {
                match self.document().page_unit.length_to_mm(height) {
                    Ok(v) => v,
                    Err(e) => {
                        self.solid_pending = Some(pending);
                        return Some(Err(e));
                    }
                }
            } else {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Invalid("height must be non-zero")));
            };
            let center = pending.points[0];
            let length = pending.box_length;
            let width = pending.box_width.unwrap_or(length);
            let plane = pending.box_plane.unwrap_or_else(|| self.input_plane());
            let result = self.solid_box_dimensions(center, length, width, height, plane);
            if result.is_err() {
                self.solid_pending = Some(pending);
            }
            return Some(result);
        }
        if let Some((key, value)) = text.split_once('=') {
            let key = match key.trim().to_ascii_lowercase().as_str() {
                "l" => "length".to_owned(),
                "w" => "width".to_owned(),
                "h" => "height".to_owned(),
                other => other.to_owned(),
            };
            let value = match value.trim().parse::<f64>() {
                Ok(value)
                    if value.is_finite()
                        && ((matches!(key.as_str(), "height" | "width") && value != 0.0)
                            || (key == "length" && value > 0.0)) =>
                {
                    value
                }
                _ => {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Invalid("box option value must be non-zero")));
                }
            };
            let value = match self.document().page_unit.length_to_mm(value) {
                Ok(v) => v,
                Err(e) => {
                    self.solid_pending = Some(pending);
                    return Some(Err(e));
                }
            };
            if pending.points.len() < 2 {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Invalid("pick both base points first")));
            }
            match key.as_str() {
                "width" if matches!(pending.box_kind, "ThreePoint" | "Vertical") => {
                    pending.box_width = Some(value);
                    self.solid_pending = Some(pending);
                    return Some(Ok(()));
                }
                "height" if value != 0.0 => {
                    let first = pending.points[0];
                    let second = pending.points[1];
                    let width = pending.box_width;
                    let result = self.solid_box_points(
                        first,
                        second,
                        value,
                        pending.box_kind,
                        width,
                        pending.box_plane.unwrap_or_else(|| self.input_plane()),
                    );
                    if result.is_err() {
                        self.solid_pending = Some(pending);
                    }
                    return Some(result);
                }
                _ => {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Invalid("Box accepts Width= or Height=")));
                }
            }
        }
        if pending.points.len() >= 2 && text.contains(',') {
            let plane = pending.box_plane.unwrap_or_else(|| self.input_plane());
            let base = pending
                .points
                .last()
                .copied()
                .and_then(|p| om9_core::coords::WorldPoint::try_new(p).ok());
            let point = om9_core::coords::parse_input(text.trim(), self.document().page_unit)
                .and_then(|input| om9_core::coords::resolve_with_base(input, Some(&plane), base));
            self.solid_pending = Some(pending);
            return Some(point.and_then(|point| self.accept_world_point(point.xyz())));
        }
        let numbers: Vec<f64> = text
            .split([',', ' ', '\t'])
            .filter(|word| !word.is_empty())
            .filter_map(|word| word.parse::<f64>().ok())
            .filter(|value| value.is_finite())
            .collect();
        if pending.points.len() < 2 {
            if numbers.len() == 3
                || text.contains(',')
                || (pending.points.is_empty() && text.trim() == "0")
            {
                let plane = pending.box_plane.unwrap_or_else(|| self.input_plane());
                let base = pending
                    .points
                    .last()
                    .copied()
                    .and_then(|p| om9_core::coords::WorldPoint::try_new(p).ok());
                let input_text = if text.trim() == "0" {
                    "0,0,0"
                } else {
                    text.trim()
                };
                let point = om9_core::coords::parse_input(input_text, self.document().page_unit)
                    .and_then(|input| {
                        om9_core::coords::resolve_with_base(input, Some(&plane), base)
                    });
                match point {
                    Ok(point) => {
                        if pending.box_kind == "Cube" && pending.points.len() == 1 {
                            self.solid_pending = Some(pending);
                            return Some(self.accept_world_point(point.xyz()));
                        }
                        if pending.points.is_empty() {
                            pending.box_plane = Some(plane);
                        }
                        pending.points.push(point.xyz());
                    }
                    Err(error) => {
                        self.solid_pending = Some(pending);
                        return Some(Err(error));
                    }
                }
            } else {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Invalid(
                    "base corner needs three coordinates or one length",
                )));
            }
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        let first = pending.points[0];
        let second = pending.points[1];
        if pending.box_kind == "Cube" {
            self.solid_pending = None;
            return Some(self.solid_box_points(
                first,
                second,
                1.0,
                "Cube",
                None,
                pending.box_plane.unwrap_or_else(|| self.input_plane()),
            ));
        }
        if matches!(pending.box_kind, "ThreePoint" | "Vertical") && pending.box_width.is_none() {
            let width = if text.trim().is_empty() {
                let plane = pending.box_plane.unwrap_or_else(|| self.input_plane());
                match box_edge_axes(first, second, plane, pending.box_kind == "Vertical") {
                    Ok((length, _)) => length,
                    Err(error) => {
                        self.solid_pending = Some(pending);
                        return Some(Err(error));
                    }
                }
            } else if numbers.len() == 1 && numbers[0] != 0.0 {
                match self.document().page_unit.length_to_mm(numbers[0]) {
                    Ok(v) => v,
                    Err(e) => {
                        self.solid_pending = Some(pending);
                        return Some(Err(e));
                    }
                }
            } else {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Invalid("width must be non-zero")));
            };
            pending.box_width = Some(width);
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        let default_height = if matches!(pending.box_kind, "ThreePoint" | "Vertical") {
            pending.box_width.unwrap_or(
                ((second[0] - first[0]).powi(2)
                    + (second[1] - first[1]).powi(2)
                    + (second[2] - first[2]).powi(2))
                .sqrt(),
            )
        } else {
            let plane = pending.box_plane.unwrap_or_else(|| self.input_plane());
            (0..3)
                .map(|i| (second[i] - first[i]) * plane.y[i])
                .sum::<f64>()
                .abs()
        };
        let height = if text.trim().is_empty() {
            default_height
        } else if numbers.len() == 1 && numbers[0] != 0.0 {
            match self.document().page_unit.length_to_mm(numbers[0]) {
                Ok(v) => v,
                Err(e) => {
                    self.solid_pending = Some(pending);
                    return Some(Err(e));
                }
            }
        } else {
            self.solid_pending = Some(pending);
            return Some(Err(Error::Invalid("height must be non-zero")));
        };
        let width = pending.box_width;
        let result = self.solid_box_points(
            first,
            second,
            height,
            pending.box_kind,
            width,
            pending.box_plane.unwrap_or_else(|| self.input_plane()),
        );
        if result.is_err() {
            self.solid_pending = Some(pending);
        }
        return Some(result);
    }

    pub(super) fn submit_solid_boolean_two_text(
        &mut self,
        mut pending: SolidPending,
        text: &str,
    ) -> Option<Result<()>> {
        let operation = text.trim().to_ascii_lowercase();
        if matches!(operation.as_str(), "cancel" | "esc") {
            self.cancel();
            return Some(Ok(()));
        }
        let delete_value = match operation.as_str() {
            "deleteinput" => Some(!pending.difference_delete_input),
            "deleteinput=yes" | "deleteinput=true" => Some(true),
            "deleteinput=no" | "deleteinput=false" => Some(false),
            _ => None,
        };
        if let Some(value) = delete_value {
            pending.difference_delete_input = value;
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if operation.starts_with("deleteinput=") {
            self.solid_pending = Some(pending);
            return Some(Err(Error::Invalid("DeleteInput must be Yes or No")));
        }
        if operation == "undo" {
            let result = if pending.boolean_two_preview.take().is_some() {
                Ok(())
            } else {
                match self.selected_ordered().last().copied() {
                    Some(id) => self.remove_objects(&[id]),
                    None => Err(Error::Missing("no Boolean input to undo")),
                }
            };
            self.solid_pending = Some(pending);
            return Some(result);
        }
        if operation.is_empty() && pending.boolean_two_preview.is_some() {
            let result = self.boolean_two_commit(
                pending.boolean_two_preview.as_ref().expect("preview"),
                pending.difference_delete_input,
            );
            if result.is_err() {
                self.solid_pending = Some(pending);
            }
            return Some(result);
        }
        let mode = match operation.as_str() {
            "" | "u" | "union" => Some(0),
            "d" | "difference" => Some(1),
            "r" | "reversedifference" => Some(2),
            "i" | "intersection" => Some(3),
            "x" | "xor" | "inversionintersection" => Some(4),
            "next" | "n" => Some(
                pending
                    .boolean_two_preview
                    .as_ref()
                    .map_or(0, |p| (p.mode + 1) % 5),
            ),
            _ => None,
        };
        let Some(mode) = mode else {
            self.solid_pending = Some(pending);
            return Some(Err(Error::Invalid(
                "choose Union, Difference, ReverseDifference, Intersection or InversionIntersection",
            )));
        };
        if let Some(preview) = &pending.boolean_two_preview {
            if preview
                .sources
                .iter()
                .any(|s| self.document().stamp(s.object).ok().as_ref() != Some(s))
                || self.selected_ordered()
                    != preview.sources.iter().map(|s| s.object).collect::<Vec<_>>()
            {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Stale("Boolean input")));
            }
        }
        let result = self.boolean_two_build_preview(mode).and_then(|preview| {
            if let Some(old) = &pending.boolean_two_preview {
                let mut held = self.scene_cache_usage()?;
                held.add(old.usage, self.coordinator.limits())?;
                held.add(preview.usage, self.coordinator.limits())?;
                self.coordinator.retained_usage_with_held_usage(held)?;
            }
            Ok(preview)
        });
        match result {
            Ok(preview) => {
                pending.boolean_two_preview = Some(preview);
                self.solid_pending = Some(pending);
                return Some(Ok(()));
            }
            Err(error) => {
                self.solid_pending = Some(pending);
                return Some(Err(error));
            }
        }
    }

    pub(super) fn submit_solid_text_feature(
        &mut self,
        mut pending: SolidPending,
        text: &str,
    ) -> Option<Result<()>> {
        if pending.text.is_none() {
            if text.trim().is_empty() {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Invalid("text must not be empty")));
            }
            if text.len() > self.coordinator.limits().max_name_bytes {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Budget("text bytes")));
            }
            let mut value = String::new();
            if value.try_reserve_exact(text.len()).is_err() {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Budget("text input")));
            }
            value.push_str(text);
            pending.text = Some(value);
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if !text
            .trim()
            .parse::<f64>()
            .is_ok_and(|v| v.is_finite() && v > 0.0 && v <= f32::MAX as f64)
        {
            self.solid_pending = Some(pending);
            return Some(Err(Error::Invalid(
                "text height must be finite and positive",
            )));
        }
        let revision = self.document().revision;
        let result = self.dispatch_solid_args(
            pending.feature,
            &[pending.text.as_deref().expect("text entered"), text.trim()],
        );
        if result.is_err() && self.document().revision == revision {
            self.solid_pending = Some(pending);
        }
        return Some(result);
    }

    pub(super) fn submit_solid_sphere_text(
        &mut self,
        mut pending: SolidPending,
        text: &str,
    ) -> Option<Result<()>> {
        if pending.sphere_mode
            && pending.sphere_kind == "FitPoints"
            && (text.trim().eq_ignore_ascii_case("selection")
                || (text.trim().is_empty() && pending.points.is_empty()))
        {
            let points = self.solid_fit_selection_points();
            if points.is_empty() {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Missing(
                    "select points, curves or meshes to fit",
                )));
            }
            pending.points.extend(points);
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        #[cfg(not(feature = "occt-brep"))]
        if pending.sphere_mode {
            self.solid_pending = Some(pending);
            return Some(Err(Error::Unsupported("Sphere requires OCCT")));
        }
        #[cfg(feature = "occt-brep")]
        if pending.sphere_mode {
            let raw = text.trim().to_ascii_lowercase();
            if !matches!(raw.as_str(), "undo" | "u" | "cancel" | "esc")
                && pending
                    .sphere_sources
                    .iter()
                    .any(|stamp| self.document().stamp(stamp.object).ok().as_ref() != Some(stamp))
            {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Stale("Sphere curve source")));
            }
            let option = raw.strip_prefix("mode=").unwrap_or(&raw).to_string();
            let selected = match option.as_str() {
                "center" | "c" => Some("Center"),
                "2point" | "2p"
                    if !matches!(
                        pending.sphere_kind,
                        "TangentPoint" | "Tangent2" | "Tangent3" | "TangentCurvePoint"
                    ) =>
                {
                    Some("TwoPoint")
                }
                "2" if pending.points.is_empty()
                    && pending.sphere_center.is_none()
                    && !matches!(
                        pending.sphere_kind,
                        "TangentPoint" | "Tangent2" | "Tangent3" | "TangentCurvePoint"
                    ) =>
                {
                    Some("TwoPoint")
                }
                "3point" | "3p"
                    if !matches!(
                        pending.sphere_kind,
                        "TangentPoint" | "Tangent2" | "Tangent3" | "TangentCurvePoint"
                    ) =>
                {
                    Some("ThreePoint")
                }
                "3" if pending.points.is_empty()
                    && pending.sphere_center.is_none()
                    && !matches!(
                        pending.sphere_kind,
                        "TangentPoint" | "Tangent2" | "Tangent3" | "TangentCurvePoint"
                    ) =>
                {
                    Some("ThreePoint")
                }
                "4point" | "4p"
                    if !matches!(
                        pending.sphere_kind,
                        "TangentPoint" | "Tangent2" | "Tangent3" | "TangentCurvePoint"
                    ) =>
                {
                    Some("FourPoint")
                }
                "4" if pending.points.is_empty()
                    && pending.sphere_center.is_none()
                    && !matches!(
                        pending.sphere_kind,
                        "TangentPoint" | "Tangent2" | "Tangent3" | "TangentCurvePoint"
                    ) =>
                {
                    Some("FourPoint")
                }
                "fitpoints" | "fit" | "f" => Some("FitPoints"),
                "vertical" | "v" => Some("Vertical"),
                "aroundcurve" | "a" => Some("AroundCurve"),
                "tangent" | "t" => Some("Tangent"),
                "point" | "p" if pending.sphere_kind == "Tangent2" => Some("Tangent2Point"),
                "point" | "p" if pending.sphere_kind == "Tangent" => {
                    Some(if pending.sphere_curve.is_empty() {
                        "TangentPoint"
                    } else {
                        "TangentCurvePoint"
                    })
                }
                _ => None,
            };
            if let Some(kind) = selected {
                if !pending.points.is_empty() {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Invalid(
                        "select the construction mode before the first point",
                    )));
                }
                if matches!(
                    kind,
                    "Center" | "TwoPoint" | "ThreePoint" | "FourPoint" | "FitPoints" | "Vertical"
                ) {
                    pending.sphere_sources.clear();
                    pending.sphere_edge = None;
                }
                if kind == "AroundCurve" || kind == "Tangent" {
                    let curves: Vec<_> = self
                        .selection
                        .iter()
                        .filter_map(|id| match &self.document().entities.get(id)?.geometry {
                            Geometry::Curve(_) => self.document().entities.get(id),
                            _ => None,
                        })
                        .collect();
                    if curves.len() > 3 {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Missing("select at most three curves")));
                    }
                    if curves.is_empty() {
                        pending.sphere_kind = kind;
                        self.solid_pending = Some(pending);
                        return Some(Ok(()));
                    }
                    pending.sphere_edge = None;
                    let stamps: Result<Vec<_>> = curves
                        .iter()
                        .map(|entity| self.document().stamp(entity.id))
                        .collect();
                    pending.sphere_sources = match stamps {
                        Ok(stamps) => stamps,
                        Err(error) => {
                            self.solid_pending = Some(pending);
                            return Some(Err(error));
                        }
                    };
                    if curves.len() == 3 {
                        if kind != "Tangent" {
                            self.solid_pending = Some(pending);
                            return Some(Err(Error::Missing("tangent accepts three curves")));
                        }
                        let sample = |entity: &om9_core::model::Entity| {
                            sphere_world_curve_samples(entity, self.coordinator.limits())
                        };
                        let first = match sample(curves[0]) {
                            Ok(samples) => samples,
                            Err(error) => {
                                self.solid_pending = Some(pending);
                                return Some(Err(error));
                            }
                        };
                        let second = match sample(curves[1]) {
                            Ok(samples) => samples,
                            Err(error) => {
                                self.solid_pending = Some(pending);
                                return Some(Err(error));
                            }
                        };
                        let third = match sample(curves[2]) {
                            Ok(samples) => samples,
                            Err(error) => {
                                self.solid_pending = Some(pending);
                                return Some(Err(error));
                            }
                        };
                        pending.sphere_curve = first;
                        pending.sphere_curve2 = second;
                        pending.sphere_curve3 = third;
                        pending.sphere_kind = "Tangent3";
                        self.solid_pending = Some(pending);
                        return Some(Ok(()));
                    }
                    if curves.len() == 2 {
                        if kind != "Tangent" {
                            self.solid_pending = Some(pending);
                            return Some(Err(Error::Missing("AroundCurve accepts one curve")));
                        }
                        let first = match sphere_world_curve_samples(
                            curves[0],
                            self.coordinator.limits(),
                        ) {
                            Ok(samples) => samples,
                            Err(error) => {
                                self.solid_pending = Some(pending);
                                return Some(Err(error));
                            }
                        };
                        let second = match sphere_world_curve_samples(
                            curves[1],
                            self.coordinator.limits(),
                        ) {
                            Ok(samples) => samples,
                            Err(error) => {
                                self.solid_pending = Some(pending);
                                return Some(Err(error));
                            }
                        };
                        pending.sphere_curve = first;
                        pending.sphere_curve2 = second;
                        pending.sphere_kind = "Tangent2";
                        self.solid_pending = Some(pending);
                        return Some(Ok(()));
                    }
                    let samples =
                        match sphere_world_curve_samples(curves[0], self.coordinator.limits()) {
                            Ok(samples) => samples,
                            Err(error) => {
                                self.solid_pending = Some(pending);
                                return Some(Err(error));
                            }
                        };
                    if samples.len() < 2 {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Invalid("curve has no samples")));
                    }
                    pending.sphere_curve = samples;
                }
                pending.sphere_kind = kind;
                self.solid_pending = Some(pending);
                return Some(Ok(()));
            }
            if option == "undo" || option == "u" {
                if pending.sphere_kind == "AroundCurve"
                    && pending.sphere_center.is_none()
                    && !pending.sphere_curve.is_empty()
                {
                    pending.sphere_curve.clear();
                    pending.sphere_sources.clear();
                    pending.sphere_edge = None;
                } else if pending.sphere_kind == "ThreePoint"
                    && pending.points.len() == 2
                    && pending.sphere_radius != 0.0
                {
                    pending.sphere_radius = 0.0;
                } else if pending.sphere_kind == "FourPoint" && pending.points.len() >= 3 {
                    pending.points.pop();
                    pending.sphere_center = None;
                } else if pending.points.pop().is_none() {
                    pending.sphere_center = None;
                }
                if pending.points.is_empty() && pending.sphere_center.is_none() {
                    pending.sphere_plane = None;
                }
                self.solid_pending = Some(pending);
                return Some(Ok(()));
            }
            let coordinate_words: Vec<_> = text.split_whitespace().collect();
            let spaced_point = coordinate_words.len() == 3
                && coordinate_words.iter().all(|v| v.parse::<f64>().is_ok());
            if (text.contains(',') && !text.contains('='))
                || spaced_point
                || (text.trim() == "0"
                    && pending.points.is_empty()
                    && pending.sphere_center.is_none())
            {
                let plane = pending.sphere_plane.unwrap_or_else(|| self.input_plane());
                let base = pending
                    .points
                    .last()
                    .copied()
                    .or(pending.sphere_center)
                    .and_then(|p| om9_core::coords::WorldPoint::try_new(p).ok());
                let input = if text.trim() == "0" {
                    "0,0,0".to_owned()
                } else if spaced_point {
                    coordinate_words.join(",")
                } else {
                    text.trim().to_owned()
                };
                let point = om9_core::coords::parse_input(&input, self.document().page_unit)
                    .and_then(|input| {
                        om9_core::coords::resolve_with_base(input, Some(&plane), base)
                    });
                self.solid_pending = Some(pending);
                return Some(point.and_then(|point| self.accept_sphere_point(point.xyz())));
            }
            if pending.sphere_kind == "ThreePoint"
                && pending.points.len() == 2
                && (option == "radius"
                    || option.starts_with("radius=")
                    || pending.sphere_radius < 0.0)
            {
                if option == "radius" {
                    pending.sphere_radius = -1.0;
                    self.solid_pending = Some(pending);
                    return Some(Ok(()));
                }
                let raw = option.strip_prefix("radius=").unwrap_or(&option);
                let Some(value) = raw
                    .parse::<f64>()
                    .ok()
                    .filter(|value| value.is_finite() && *value > 0.0)
                else {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Invalid("radius must be positive")));
                };
                let value = match self.document().page_unit.length_to_mm(value) {
                    Ok(v) => v,
                    Err(e) => {
                        self.solid_pending = Some(pending);
                        return Some(Err(e));
                    }
                };
                let a = pending.points[0];
                let b = pending.points[1];
                let half = (b[0] - a[0]).hypot(b[1] - a[1]).hypot(b[2] - a[2]) * 0.5;
                if !half.is_finite() || half <= 1e-9 || value < half {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Invalid(
                        "radius must be at least half the chord",
                    )));
                }
                pending.sphere_radius = value;
                self.solid_pending = Some(pending);
                return Some(Ok(()));
            }
            if pending.sphere_kind == "AroundCurve"
                && text.trim().to_ascii_lowercase().starts_with("oncurve=")
            {
                let Some(fraction) = text
                    .trim()
                    .split_once('=')
                    .and_then(|(_, value)| value.parse::<f64>().ok())
                    .filter(|value| value.is_finite() && (0.0..=1.0).contains(value))
                else {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Invalid("OnCurve must be within 0..1")));
                };
                let Some(entity) = pending
                    .sphere_sources
                    .first()
                    .and_then(|stamp| self.document().entities.get(&stamp.object))
                else {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Invalid("AroundCurve has no source curve")));
                };
                let source = match sphere_curve_source(
                    entity,
                    pending.sphere_edge,
                    self.coordinator.limits(),
                ) {
                    Ok(source) => source,
                    Err(error) => {
                        self.solid_pending = Some(pending);
                        return Some(Err(error));
                    }
                };
                let Geometry::Curve(curve) = &source.geometry else {
                    unreachable!()
                };
                let center =
                    om9_core::curve_ops::point_at(curve, fraction, self.coordinator.limits())
                        .and_then(|point| entity.placement.point(point));
                match center {
                    Ok(center) => pending.sphere_center = Some(center),
                    Err(error) => {
                        self.solid_pending = Some(pending);
                        return Some(Err(error));
                    }
                }
                self.solid_pending = Some(pending);
                return Some(Ok(()));
            }
            if pending.sphere_kind == "TwoPoint" && pending.points.len() == 1 {
                let text_value = text.trim();
                let value = text_value
                    .strip_prefix("Diameter=")
                    .unwrap_or(text_value)
                    .parse::<f64>()
                    .ok()
                    .filter(|value| value.is_finite() && *value > 0.0);
                let Some(diameter) = value else {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Invalid("diameter must be positive")));
                };
                let diameter = match self.document().page_unit.length_to_mm(diameter) {
                    Ok(v) => v,
                    Err(e) => {
                        self.solid_pending = Some(pending);
                        return Some(Err(e));
                    }
                };
                let first = pending.points[0];
                let x = pending.sphere_plane.unwrap_or_else(|| self.input_plane()).x;
                let center = std::array::from_fn(|i| first[i] + x[i] * diameter * 0.5);
                self.solid_pending = None;
                return Some(self.solid_sphere_points(center, diameter * 0.5));
            }
            let raw_lower = text.trim().to_ascii_lowercase();
            if matches!(pending.sphere_kind, "AroundCurve" | "Tangent")
                && pending.sphere_curve.is_empty()
                && (raw_lower.starts_with("path=") || raw_lower.starts_with("curve="))
            {
                let value = text
                    .trim()
                    .split_once('=')
                    .map(|(_, name)| name.trim())
                    .unwrap_or_default();
                let (raw_name, pick) = match value.split_once('@') {
                    Some((name, coords)) => {
                        let values: Vec<f64> = coords
                            .split(',')
                            .filter_map(|value| value.trim().parse::<f64>().ok())
                            .collect();
                        if values.len() != 3 || values.iter().any(|value| !value.is_finite()) {
                            self.solid_pending = Some(pending);
                            return Some(Err(Error::Invalid(
                                "Curve pick syntax: Curve=Object.EdgeN@x,y,z",
                            )));
                        }
                        (name, Some([values[0], values[1], values[2]]))
                    }
                    None => (value, None),
                };
                let (name, edge) = match raw_name.split_once('.') {
                    Some((name, suffix)) => {
                        let index = suffix
                            .strip_prefix("Edge")
                            .and_then(|n| n.parse::<usize>().ok())
                            .and_then(|n| n.checked_sub(1));
                        let Some(index) = index else {
                            self.solid_pending = Some(pending);
                            return Some(Err(Error::Invalid("Path edge must be EdgeN")));
                        };
                        (name, Some(index))
                    }
                    None => (raw_name, None),
                };
                let entity = self
                    .document()
                    .entities
                    .values()
                    .find(|entity| entity.label == name);
                let Some(entity) = entity else {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Missing("Path=Object.EdgeN curve")));
                };
                if matches!(entity.geometry, Geometry::Brep(_))
                    && pending.sphere_kind != "AroundCurve"
                {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Unsupported("Sphere Tangent B-Rep edge")));
                }
                if matches!(entity.geometry, Geometry::Curve(_))
                    && edge.is_some_and(|index| index != 0)
                {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Invalid("curve has only Edge1")));
                }
                let source = match sphere_curve_source(entity, edge, self.coordinator.limits()) {
                    Ok(source) => source,
                    Err(error) => {
                        self.solid_pending = Some(pending);
                        return Some(Err(error));
                    }
                };
                let stamp = match self.document().stamp(entity.id) {
                    Ok(stamp) => stamp,
                    Err(error) => {
                        self.solid_pending = Some(pending);
                        return Some(Err(error));
                    }
                };
                let samples = match sphere_world_curve_samples(&source, self.coordinator.limits()) {
                    Ok(samples) => samples,
                    Err(error) => {
                        self.solid_pending = Some(pending);
                        return Some(Err(error));
                    }
                };
                if let Some(pick) = pick {
                    if pending.sphere_kind == "AroundCurve" {
                        match sphere_closest_world_curve_point(
                            &source,
                            pick,
                            self.coordinator.limits(),
                        ) {
                            Ok(center) => pending.sphere_center = Some(center),
                            Err(error) => {
                                self.solid_pending = Some(pending);
                                return Some(Err(error));
                            }
                        }
                    }
                }
                pending.sphere_curve = samples;
                pending.sphere_edge = if matches!(entity.geometry, Geometry::Brep(_)) {
                    Some(edge.unwrap_or(0))
                } else {
                    None
                };
                pending.sphere_sources = vec![stamp];
                self.solid_pending = Some(pending);
                return Some(Ok(()));
            }
            if matches!(pending.sphere_kind, "Tangent" | "TangentPoint")
                && text.trim().to_ascii_lowercase().starts_with("solution=")
            {
                let Some(solution) = text
                    .trim()
                    .split_once('=')
                    .and_then(|(_, value)| value.parse::<u32>().ok())
                else {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Invalid("Solution must be 1 or 2")));
                };
                if !(1..=2).contains(&solution) {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Invalid("Solution must be 1 or 2")));
                }
                pending.sphere_solution_negative = solution == 2;
                self.solid_pending = Some(pending);
                return Some(Ok(()));
            }
            if pending.sphere_kind == "Tangent" && pending.sphere_center.is_some() {
                let Some(radius) = sphere_radius_input(text) else {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Invalid("tangent radius must be positive")));
                };
                let radius = match self.document().page_unit.length_to_mm(radius) {
                    Ok(v) => v,
                    Err(e) => {
                        self.solid_pending = Some(pending);
                        return Some(Err(e));
                    }
                };
                let Some(point) = pending.sphere_center else {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Invalid("tangent point required")));
                };
                let normal = pending.sphere_normal;
                let sign = if pending.sphere_solution_negative {
                    -1.0
                } else {
                    1.0
                };
                let center = [
                    point[0] + normal[0] * radius * sign,
                    point[1] + normal[1] * radius * sign,
                    point[2] + normal[2] * radius * sign,
                ];
                self.solid_pending = None;
                return Some(self.solid_sphere_points(center, radius));
            }
            if pending.sphere_kind == "Tangent2Point" {
                if pending.points.is_empty() {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Invalid("pick a point constraint first")));
                }
                if !text.trim().is_empty() {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Invalid(
                        "press Enter to fit the two curves and point",
                    )));
                }
                let center = match tangent_sphere_two_curves_point(
                    &pending.sphere_curve,
                    &pending.sphere_curve2,
                    pending.points[0],
                ) {
                    Ok(center) => center,
                    Err(error) => {
                        self.solid_pending = Some(pending);
                        return Some(Err(error));
                    }
                };
                let radius = ((center[0] - pending.points[0][0]).powi(2)
                    + (center[1] - pending.points[0][1]).powi(2)
                    + (center[2] - pending.points[0][2]).powi(2))
                .sqrt();
                self.solid_pending = None;
                return Some(self.solid_sphere_points(center, radius));
            }
            if pending.sphere_kind == "Tangent3" {
                let radius = sphere_radius_input(text);
                let Some(radius) = radius else {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Invalid("radius must be positive")));
                };
                let radius = match self.document().page_unit.length_to_mm(radius) {
                    Ok(v) => v,
                    Err(e) => {
                        self.solid_pending = Some(pending);
                        return Some(Err(e));
                    }
                };
                let center = match tangent_sphere_three_curves(
                    &pending.sphere_curve,
                    &pending.sphere_curve2,
                    &pending.sphere_curve3,
                    radius,
                ) {
                    Ok(center) => center,
                    Err(error) => {
                        self.solid_pending = Some(pending);
                        return Some(Err(error));
                    }
                };
                self.solid_pending = None;
                return Some(self.solid_sphere_points(center, radius));
            }
            if pending.sphere_kind == "Tangent2" {
                let radius = sphere_radius_input(text);
                let Some(radius) = radius else {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Invalid("radius must be positive")));
                };
                let radius = match self.document().page_unit.length_to_mm(radius) {
                    Ok(v) => v,
                    Err(e) => {
                        self.solid_pending = Some(pending);
                        return Some(Err(e));
                    }
                };
                let center = match tangent_sphere_two_curves(
                    &pending.sphere_curve,
                    &pending.sphere_curve2,
                    radius,
                ) {
                    Ok(center) => center,
                    Err(error) => {
                        self.solid_pending = Some(pending);
                        return Some(Err(error));
                    }
                };
                self.solid_pending = None;
                return Some(self.solid_sphere_points(center, radius));
            }
            if pending.sphere_kind == "TangentCurvePoint" {
                if pending.points.len() >= 2 {
                    if !text.trim().is_empty() {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Invalid(
                            "press Enter to fit through the two points",
                        )));
                    }
                    let center = match tangent_sphere_curve_two_points(
                        &pending.sphere_curve,
                        pending.points[0],
                        pending.points[1],
                    ) {
                        Ok(center) => center,
                        Err(error) => {
                            self.solid_pending = Some(pending);
                            return Some(Err(error));
                        }
                    };
                    let radius = ((center[0] - pending.points[0][0]).powi(2)
                        + (center[1] - pending.points[0][1]).powi(2)
                        + (center[2] - pending.points[0][2]).powi(2))
                    .sqrt();
                    self.solid_pending = None;
                    return Some(self.solid_sphere_points(center, radius));
                }
                if pending.points.is_empty() {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Invalid("pick a point constraint first")));
                }
                let radius = sphere_radius_input(text);
                let Some(radius) = radius else {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Invalid("radius must be positive")));
                };
                let radius = match self.document().page_unit.length_to_mm(radius) {
                    Ok(v) => v,
                    Err(e) => {
                        self.solid_pending = Some(pending);
                        return Some(Err(e));
                    }
                };
                let center = match tangent_sphere_curve_point(
                    &pending.sphere_curve,
                    pending.points[0],
                    radius,
                ) {
                    Ok(center) => center,
                    Err(error) => {
                        self.solid_pending = Some(pending);
                        return Some(Err(error));
                    }
                };
                self.solid_pending = None;
                return Some(self.solid_sphere_points(center, radius));
            }
            if pending.sphere_kind == "TangentPoint" {
                if pending.points.len() < 2 {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Invalid("pick two point constraints first")));
                }
                let value = sphere_radius_input(text);
                let Some(radius) = value else {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Invalid("radius must be positive")));
                };
                let radius = match self.document().page_unit.length_to_mm(radius) {
                    Ok(v) => v,
                    Err(e) => {
                        self.solid_pending = Some(pending);
                        return Some(Err(e));
                    }
                };
                let a = pending.points[0];
                let b = pending.points[1];
                let mid = [
                    (a[0] + b[0]) * 0.5,
                    (a[1] + b[1]) * 0.5,
                    (a[2] + b[2]) * 0.5,
                ];
                let half = ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2) + (b[2] - a[2]).powi(2))
                    .sqrt()
                    * 0.5;
                if radius < half {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Invalid(
                        "radius must be at least half the point distance",
                    )));
                }
                let offset = (radius * radius - half * half).sqrt();
                let sign = if pending.sphere_solution_negative {
                    -1.0
                } else {
                    1.0
                };
                let center = [mid[0], mid[1] + offset * sign, mid[2]];
                self.solid_pending = None;
                return Some(self.solid_sphere_points(center, radius));
            }
            if let Some((key, value)) = text.split_once('=') {
                let key = match key.trim().to_ascii_lowercase().as_str() {
                    "r" => "radius".to_owned(),
                    "d" => "diameter".to_owned(),
                    other => other.to_owned(),
                };
                let value = match value.trim().parse::<f64>() {
                    Ok(value) if value.is_finite() && value > 0.0 => value,
                    _ => {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Invalid("option value must be positive")));
                    }
                };
                let value = match self.document().page_unit.length_to_mm(value) {
                    Ok(v) => v,
                    Err(e) => {
                        self.solid_pending = Some(pending);
                        return Some(Err(e));
                    }
                };
                if pending.sphere_kind == "Tangent" {
                    let Some(point) = pending.sphere_center else {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Invalid("tangent point required")));
                    };
                    if key != "radius" {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Invalid("Tangent accepts Radius=")));
                    }
                    let normal = pending.sphere_normal;
                    let sign = if pending.sphere_solution_negative {
                        -1.0
                    } else {
                        1.0
                    };
                    let center = [
                        point[0] + normal[0] * value * sign,
                        point[1] + normal[1] * value * sign,
                        point[2] + normal[2] * value * sign,
                    ];
                    self.solid_pending = None;
                    return Some(self.solid_sphere_points(center, value));
                }
                let Some(center) = pending.sphere_center else {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Invalid("sphere center required")));
                };
                let radius = match key.as_str() {
                    "radius" => value,
                    "diameter" => value * 0.5,
                    _ => {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Invalid("Sphere accepts Radius= or Diameter=")));
                    }
                };
                self.solid_pending = None;
                return Some(self.solid_sphere_points(center, radius));
            }
            let numbers: Vec<f64> = text
                .split([',', ' ', '\t'])
                .filter(|word| !word.is_empty())
                .filter_map(|word| word.parse::<f64>().ok())
                .filter(|value| value.is_finite())
                .collect();
            if pending.sphere_kind == "Tangent" {
                let Some(tangent_point) = pending.sphere_center else {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Invalid(
                        "pick the tangent point on the curve first",
                    )));
                };
                if numbers.len() != 1 || numbers[0] <= 0.0 {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Invalid("tangent radius must be positive")));
                }
                let radius = numbers[0];
                let normal = pending.sphere_normal;
                let center = [
                    tangent_point[0] + normal[0] * radius,
                    tangent_point[1] + normal[1] * radius,
                    tangent_point[2] + normal[2] * radius,
                ];
                self.solid_pending = None;
                return Some(self.solid_sphere_points(center, radius));
            }
            if pending.sphere_kind == "FitPoints" {
                if !text.trim().is_empty() {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Invalid(
                        "press Enter to fit the collected points",
                    )));
                }
                if pending.points.len() < 4 {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Invalid("fit sphere needs at least four points")));
                }
                let points = std::mem::take(&mut pending.points);
                self.solid_pending = None;
                return Some(self.fit_sphere_points(&points));
            }
            if pending.sphere_center.is_none() {
                if numbers.len() != 3 {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Invalid("sphere center needs three coordinates")));
                }
                pending
                    .sphere_center
                    .replace([numbers[0], numbers[1], numbers[2]]);
                self.solid_pending = Some(pending);
                return Some(Ok(()));
            }
            if numbers.len() != 1 || numbers[0] <= 0.0 {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Invalid("radius must be positive")));
            }
            let center = pending.sphere_center.expect("center");
            let radius = match self.document().page_unit.length_to_mm(numbers[0]) {
                Ok(v) => v,
                Err(e) => {
                    self.solid_pending = Some(pending);
                    return Some(Err(e));
                }
            };
            let result = self.solid_sphere_points(center, radius);
            if result.is_err() {
                self.solid_pending = Some(pending);
            }
            return Some(result);
        }
        self.submit_solid_generic_tail(pending, text)
    }

    pub(super) fn submit_solid_generic_tail(
        &mut self,
        mut pending: SolidPending,
        text: &str,
    ) -> Option<Result<()>> {
        let previous_len = pending.values.len();
        let mut submitted = Vec::new();
        for word in text.split_whitespace() {
            let value = match word.parse::<f64>() {
                Ok(value) if value.is_finite() => value,
                _ => {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Invalid("solid dimension")));
                }
            };
            if pending.feature == "OM9-SOLID-018"
                && previous_len + submitted.len() == 0
                && (value < 3.0 || value.fract() != 0.0 || value > i32::MAX as f64)
            {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Invalid(
                    "pyramid sides must be an integer of at least three",
                )));
            }
            submitted.push(value);
        }
        pending.values.extend(submitted);
        if pending.values.len() < pending.labels.len() {
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        let args: Vec<String> = pending
            .values
            .iter()
            .map(|value| value.to_string())
            .collect();
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        let revision = self.document().revision;
        let result = self.dispatch_solid_args(pending.feature, &refs);
        if result.is_err() && self.document().revision == revision {
            pending.values.truncate(previous_len);
            self.solid_pending = Some(pending);
        }
        return Some(result);
    }
}
