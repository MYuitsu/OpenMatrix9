// SPDX-License-Identifier: LGPL-2.1-or-later
//! Surface and curve-pair command dispatch, surface text input and option tokens.
//!
//! Extracted unchanged from `session.rs`: each method is still `impl Controller`, and the parent
//! module keeps the command-line pipeline and calls these through `pub(super)`.
use super::*;

impl Controller {
    /// Start a fillet command that waits for one or two radii.
    pub(super) fn dispatch_fillet(&mut self, feature: &str) -> Result<()> {
        let feature = surface_edit::static_feature(feature);
        let (first, second) = self.selected_curve_pair()?;
        self.fillet_pending = Some(FilletPending {
            feature,
            first,
            second,
            start_radius: None,
            end_radius: None,
        });
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "fillet strip",
            None,
        );
        Ok(())
    }
    /// Start a variable-chamfer command that waits for two setbacks.
    pub(super) fn dispatch_variable_chamfer(&mut self, feature: &str) -> Result<()> {
        let feature = surface_edit::static_feature(feature);
        let (first, second) = self.selected_curve_pair()?;
        self.variable_chamfer_pending = Some(VarChamferPending {
            feature,
            first,
            second,
            start_setback: None,
            end_setback: None,
        });
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "variable chamfer strip",
            None,
        );
        Ok(())
    }
    pub(super) fn selected_curve_pair(&self) -> Result<(Curve, Curve)> {
        let ordered = self.selected_ordered();
        if ordered.len() != 2 {
            return Err(Error::Invalid("two curve selection"));
        }
        let mut curves = Vec::with_capacity(2);
        for id in ordered {
            let entity = self
                .document()
                .entities
                .get(&id)
                .ok_or(Error::Missing("selected curve"))?;
            let Geometry::Curve(curve) = &entity.geometry else {
                return Err(Error::Invalid("curve selection"));
            };
            curves.push(curve.clone());
        }
        Ok((curves[0].clone(), curves[1].clone()))
    }
    /// Start a chamfer command that waits for a setback.
    pub(super) fn dispatch_chamfer(&mut self, feature: &str) -> Result<()> {
        let feature = surface_edit::static_feature(feature);
        let ordered = self.selected_ordered();
        if ordered.len() != 2 {
            return Err(Error::Invalid("two curve selection"));
        }
        let mut curves = Vec::with_capacity(2);
        for id in ordered {
            let entity = self
                .document()
                .entities
                .get(&id)
                .ok_or(Error::Missing("selected curve"))?;
            let Geometry::Curve(curve) = &entity.geometry else {
                return Err(Error::Invalid("curve selection"));
            };
            curves.push(curve.clone());
        }
        self.chamfer_pending = Some(ChamferPending {
            feature,
            first: curves[0].clone(),
            second: curves[1].clone(),
        });
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "chamfer strip",
            None,
        );
        Ok(())
    }
    /// Remove one trim extension and commit the untrimmed copy.
    pub(super) fn dispatch_untrim(&mut self, feature: &str) -> Result<()> {
        let feature = surface_edit::static_feature(feature);
        let id = self
            .selection
            .iter()
            .next()
            .copied()
            .ok_or(Error::Missing("surface selection"))?;
        let entity = self
            .document()
            .entities
            .get(&id)
            .ok_or(Error::Missing("selected surface"))?;
        if !matches!(entity.geometry, Geometry::Surface(_)) {
            return Err(Error::Invalid("surface selection"));
        }
        let untrimmed = surface_edit::untrim_entity(entity, self.coordinator.limits())?;
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "untrim surface",
            None,
        );
        self.commit_entity(feature, surface_edit::label(feature), untrimmed)
    }
    /// Start a variable-blend command that waits for two tangent scales.
    pub(super) fn dispatch_variable_blend(&mut self, feature: &str) -> Result<()> {
        let feature = surface_edit::static_feature(feature);
        let ordered = self.selected_ordered();
        if ordered.len() != 2 {
            return Err(Error::Invalid("two curve selection"));
        }
        let mut curves = Vec::with_capacity(2);
        for id in ordered {
            let entity = self
                .document()
                .entities
                .get(&id)
                .ok_or(Error::Missing("selected curve"))?;
            let Geometry::Curve(curve) = &entity.geometry else {
                return Err(Error::Invalid("curve selection"));
            };
            curves.push(curve.clone());
        }
        self.variable_blend_pending = Some(VarBlendPending {
            feature,
            first: curves[0].clone(),
            second: curves[1].clone(),
            start_scale: None,
            end_scale: None,
        });
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "variable blend",
            None,
        );
        Ok(())
    }
    /// Start a shrink command that waits for a UV sub-domain.
    pub(super) fn dispatch_shrink(&mut self, feature: &str) -> Result<()> {
        let feature = surface_edit::static_feature(feature);
        let id = self
            .selection
            .iter()
            .next()
            .copied()
            .ok_or(Error::Missing("surface selection"))?;
        let entity = self
            .document()
            .entities
            .get(&id)
            .ok_or(Error::Missing("selected surface"))?;
        let Geometry::Surface(surface) = &entity.geometry else {
            return Err(Error::Invalid("surface selection"));
        };
        self.shrink_pending = Some(ShrinkPending {
            feature,
            surface: surface.clone(),
            u0: None,
            u1: None,
            v0: None,
            v1: None,
        });
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "shrink surface",
            None,
        );
        Ok(())
    }
    /// Start a variable-offset command that waits for two distances and an axis.
    pub(super) fn dispatch_variable_offset(&mut self, feature: &str) -> Result<()> {
        let feature = surface_edit::static_feature(feature);
        let id = self
            .selection
            .iter()
            .next()
            .copied()
            .ok_or(Error::Missing("surface selection"))?;
        let entity = self
            .document()
            .entities
            .get(&id)
            .ok_or(Error::Missing("selected surface"))?;
        let Geometry::Surface(surface) = &entity.geometry else {
            return Err(Error::Invalid("surface selection"));
        };
        self.variable_offset_pending = Some(VarOffsetPending {
            feature,
            surface: surface.clone(),
            start: None,
            end: None,
            axis: None,
        });
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "variable offset",
            None,
        );
        Ok(())
    }
    /// Build a planar surface from selected coplanar curves.
    pub(super) fn dispatch_planar_curves(&mut self, feature: &str) -> Result<()> {
        let feature = surface_edit::static_feature(feature);
        let ordered = self.selected_ordered();
        let mut curves = Vec::with_capacity(ordered.len());
        for id in ordered {
            let entity = self
                .document()
                .entities
                .get(&id)
                .ok_or(Error::Missing("selected curve"))?;
            let Geometry::Curve(curve) = &entity.geometry else {
                return Err(Error::Invalid("curve selection"));
            };
            curves.push(curve.clone());
        }
        if curves.is_empty() {
            return Err(Error::Invalid("planar curve count"));
        }
        let geometry = surface_edit::apply_planar_curves(&curves, self.coordinator.limits())?;
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "planar curves",
            None,
        );
        self.commit_geometry(feature, surface_edit::label(feature), geometry)
    }
    /// Start an extrude-all command that waits for a height.
    pub(super) fn dispatch_extrude_all(&mut self, feature: &str) -> Result<()> {
        let feature = surface_edit::static_feature(feature);
        let ordered = self.selected_ordered();
        let mut curves = Vec::with_capacity(ordered.len());
        for id in ordered {
            let entity = self
                .document()
                .entities
                .get(&id)
                .ok_or(Error::Missing("selected curve"))?;
            let Geometry::Curve(curve) = &entity.geometry else {
                return Err(Error::Invalid("curve selection"));
            };
            curves.push(curve.clone());
        }
        if curves.is_empty() {
            return Err(Error::Invalid("curve selection"));
        }
        self.extrude_all_pending = Some(ExtrudeAllPending { feature, curves });
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "extrude all curves",
            None,
        );
        Ok(())
    }
    /// Start a tangent command that waits for an edge and direction.
    pub(super) fn dispatch_tangent(&mut self, feature: &str) -> Result<()> {
        let feature = surface_edit::static_feature(feature);
        let id = self
            .selection
            .iter()
            .next()
            .copied()
            .ok_or(Error::Missing("surface selection"))?;
        let entity = self
            .document()
            .entities
            .get(&id)
            .ok_or(Error::Missing("selected surface"))?;
        let Geometry::Surface(surface) = &entity.geometry else {
            return Err(Error::Invalid("surface selection"));
        };
        self.tangent_pending = Some(TangentPending {
            feature,
            surface: surface.clone(),
            edge: None,
            direction: None,
        });
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "surface tangent",
            None,
        );
        Ok(())
    }
    /// Drape a surface over selected point entities.
    pub(super) fn dispatch_drape(&mut self, feature: &str) -> Result<()> {
        let feature = surface_edit::static_feature(feature);
        let ordered = self.selected_ordered();
        let mut points = Vec::with_capacity(ordered.len());
        for id in ordered {
            let entity = self
                .document()
                .entities
                .get(&id)
                .ok_or(Error::Missing("selected point"))?;
            let Geometry::Point(point) = entity.geometry else {
                return Err(Error::Invalid("point selection"));
            };
            points.push(point);
        }
        if points.len() < 3 {
            return Err(Error::Invalid("drape point count"));
        }
        let geometry = surface_edit::apply_drape(&points, self.coordinator.limits())?;
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "drape points",
            None,
        );
        self.commit_geometry(feature, surface_edit::label(feature), geometry)
    }
    /// Split a surface at an interior isocurve.
    pub(super) fn dispatch_split(&mut self, feature: &str) -> Result<()> {
        let feature = surface_edit::static_feature(feature);
        let id = self
            .selection
            .iter()
            .next()
            .copied()
            .ok_or(Error::Missing("surface selection"))?;
        let entity = self
            .document()
            .entities
            .get(&id)
            .ok_or(Error::Missing("selected surface"))?;
        let Geometry::Surface(surface) = &entity.geometry else {
            return Err(Error::Invalid("surface selection"));
        };
        self.split_pending = Some(SplitPending {
            feature,
            surface: surface.clone(),
            axis: None,
            parameter: None,
        });
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "surface split",
            None,
        );
        Ok(())
    }
    /// Start an extend command that waits for an edge and distance.
    pub(super) fn dispatch_extend(&mut self, feature: &str) -> Result<()> {
        let feature = surface_edit::static_feature(feature);
        let id = self
            .selection
            .iter()
            .next()
            .copied()
            .ok_or(Error::Missing("surface selection"))?;
        let entity = self
            .document()
            .entities
            .get(&id)
            .ok_or(Error::Missing("selected surface"))?;
        let Geometry::Surface(surface) = &entity.geometry else {
            return Err(Error::Invalid("surface selection"));
        };
        self.extend_pending = Some(ExtendPending {
            feature,
            surface: surface.clone(),
            edge: None,
            distance_mm: None,
        });
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "surface extend",
            None,
        );
        Ok(())
    }
    /// Start a tween command that waits for a fraction.
    pub(super) fn dispatch_tween(&mut self, feature: &str) -> Result<()> {
        let feature = surface_edit::static_feature(feature);
        let ordered = self.selected_ordered();
        if ordered.len() != 2 {
            return Err(Error::Invalid("two surface selection"));
        }
        let mut surfaces = Vec::with_capacity(2);
        for id in ordered {
            let entity = self
                .document()
                .entities
                .get(&id)
                .ok_or(Error::Missing("selected surface"))?;
            let Geometry::Surface(surface) = &entity.geometry else {
                return Err(Error::Invalid("surface selection"));
            };
            surfaces.push(surface.clone());
        }
        self.tween_pending = Some(TweenPending {
            feature,
            first: surfaces[0].clone(),
            second: surfaces[1].clone(),
        });
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "surface tween",
            None,
        );
        Ok(())
    }
    /// Start a soft-edit command that waits for a base point, move and options.
    pub(super) fn dispatch_soft_edit(&mut self, feature: &str) -> Result<()> {
        let feature = surface_edit::static_feature(feature);
        let id = self
            .selection
            .iter()
            .next()
            .copied()
            .ok_or(Error::Missing("surface selection"))?;
        let entity = self
            .document()
            .entities
            .get(&id)
            .ok_or(Error::Missing("selected surface"))?;
        let Geometry::Surface(surface) = &entity.geometry else {
            return Err(Error::Invalid("surface selection"));
        };
        self.soft_edit_pending = Some(SoftEditPending {
            feature,
            id,
            surface: surface.clone(),
            base: [None; 2],
            delta: [None; 3],
            // Matrix Book 1 p216: U/V distance default to 1 (the whole parameter domain).
            u_distance: 1.0,
            v_distance: 1.0,
            // Copy=Yes keeps the original and adds the edited surface.
            copy: true,
            fix_edges: false,
            constraint: surface_edit::SoftEditConstraint::None,
        });
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "surface soft edit",
            None,
        );
        Ok(())
    }
    /// Parse one Soft Edit input line: `U=`/`V=` base point, `Dx=`/`Dy=`/`Dz=` move,
    /// the documented `UDistance`/`VDistance`/`Copy`/`FixEdges`/`DirConstraint`
    /// options, or a bare tuple of the values still missing. Enter commits.
    pub(super) fn submit_soft_edit_text(&mut self, text: &str) -> Result<()> {
        let mut pending = self.soft_edit_pending.take().expect("soft edit pendings");
        let trimmed = text.trim();
        if trimmed.eq_ignore_ascii_case("cancel") || trimmed.eq_ignore_ascii_case("esc") {
            return Ok(());
        }
        let mut bare = Vec::new();
        if !trimmed.is_empty() {
            for token in trimmed.split([',', ' ']).filter(|token| !token.is_empty()) {
                let Some((name, value)) = token.split_once('=') else {
                    match parse_soft_value(token) {
                        Ok(value) => bare.push(value),
                        Err(error) => {
                            return Err(Self::keep_soft_edit(
                                &mut self.soft_edit_pending,
                                pending,
                                error,
                            ));
                        }
                    }
                    continue;
                };
                let parsed = match name.trim().to_ascii_lowercase().as_str() {
                    "u" => parse_soft_value(value).map(|value| pending.base[0] = Some(value)),
                    "v" => parse_soft_value(value).map(|value| pending.base[1] = Some(value)),
                    "dx" => parse_soft_value(value).map(|value| pending.delta[0] = Some(value)),
                    "dy" => parse_soft_value(value).map(|value| pending.delta[1] = Some(value)),
                    "dz" => parse_soft_value(value).map(|value| pending.delta[2] = Some(value)),
                    "udistance" | "u distance" => {
                        parse_soft_distance(value).map(|value| pending.u_distance = value)
                    }
                    "vdistance" | "v distance" => {
                        parse_soft_distance(value).map(|value| pending.v_distance = value)
                    }
                    "copy" => parse_soft_flag(value).map(|value| pending.copy = value),
                    "fixedges" | "fix edges" => {
                        parse_soft_flag(value).map(|value| pending.fix_edges = value)
                    }
                    "dirconstraint" | "dir constraint" => {
                        surface_edit::SoftEditConstraint::parse(value)
                            .map(|value| pending.constraint = value)
                    }
                    _ => Err(Error::Invalid("soft edit option")),
                };
                if let Err(error) = parsed {
                    return Err(Self::keep_soft_edit(
                        &mut self.soft_edit_pending,
                        pending,
                        error,
                    ));
                }
            }
        }
        for value in bare {
            let slot = if let Some(index) = pending.base.iter().position(Option::is_none) {
                &mut pending.base[index]
            } else if let Some(index) = pending.delta.iter().position(Option::is_none) {
                &mut pending.delta[index]
            } else {
                return Err(Self::keep_soft_edit(
                    &mut self.soft_edit_pending,
                    pending,
                    Error::Invalid("soft edit input"),
                ));
            };
            *slot = Some(value);
        }
        if trimmed.is_empty() {
            if !pending.base_complete() || !pending.delta_complete() {
                return Err(Self::keep_soft_edit(
                    &mut self.soft_edit_pending,
                    pending,
                    Error::Invalid("soft edit needs a base point and a move"),
                ));
            }
            return self.commit_surface_soft_edit(pending);
        }
        self.soft_edit_pending = Some(pending);
        Ok(())
    }

    pub(super) fn keep_soft_edit(
        slot: &mut Option<SoftEditPending>,
        pending: SoftEditPending,
        error: Error,
    ) -> Error {
        *slot = Some(pending);
        error
    }

    pub(super) fn commit_surface_soft_edit(&mut self, pending: SoftEditPending) -> Result<()> {
        let geometry = match surface_edit::apply_soft_edit(
            &pending.surface,
            pending.base(),
            [pending.u_distance, pending.v_distance],
            pending.delta(),
            pending.constraint,
            pending.fix_edges,
            self.coordinator.limits(),
        ) {
            Ok(geometry) => geometry,
            Err(error) => {
                self.soft_edit_pending = Some(pending);
                return Err(error);
            }
        };
        if pending.copy {
            return self.commit_geometry(
                surface_edit::static_feature(pending.feature),
                surface_edit::label(pending.feature),
                geometry,
            );
        }
        // Copy=No edits the selected surface in place: one Undo entry, same object id.
        let source = self.document().stamp(pending.id)?;
        self.commit_utility_mutations(
            surface_edit::label(pending.feature),
            vec![Mutation::Geometry { source, geometry }],
        )
    }
    /// Start a seam command that waits for its axis and tolerance.
    pub(super) fn dispatch_seam(&mut self, feature: &str) -> Result<()> {
        let feature = surface_edit::static_feature(feature);
        let id = self
            .selection
            .iter()
            .next()
            .copied()
            .ok_or(Error::Missing("surface selection"))?;
        let entity = self
            .document()
            .entities
            .get(&id)
            .ok_or(Error::Missing("selected surface"))?;
        let Geometry::Surface(surface) = &entity.geometry else {
            return Err(Error::Invalid("surface selection"));
        };
        self.seam_pending = Some(SeamPending {
            feature,
            surface: surface.clone(),
            axis: None,
            tolerance_mm: None,
        });
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "surface seam",
            None,
        );
        Ok(())
    }
    /// Start a two-surface command that waits for edge names.
    pub(super) fn dispatch_surface_pair(&mut self, feature: &str) -> Result<()> {
        let feature = surface_edit::static_feature(feature);
        let ordered = self.selected_ordered();
        if ordered.len() != 2 {
            return Err(Error::Invalid("two surface selection"));
        }
        let mut surfaces = Vec::with_capacity(2);
        for id in ordered {
            let entity = self
                .document()
                .entities
                .get(&id)
                .ok_or(Error::Missing("selected surface"))?;
            let Geometry::Surface(surface) = &entity.geometry else {
                return Err(Error::Invalid("surface selection"));
            };
            surfaces.push(surface.clone());
        }
        self.surface_pair_pending = Some(SurfacePairPending {
            feature,
            source: surfaces[0].clone(),
            target: surfaces[1].clone(),
            source_edge: None,
            target_edge: None,
            continuity: om9_core::surface_ops::SurfaceContinuity::G0,
        });
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "surface match edges",
            None,
        );
        Ok(())
    }
    /// Attach FourRailProfile metadata to selected profile curves.
    pub(super) fn dispatch_four_rail_profile(&mut self, feature: &str) -> Result<()> {
        let feature = surface_edit::static_feature(feature);
        let ordered = self.selected_ordered();
        if ordered.len() < 5 {
            return Err(Error::Invalid("four rails and a profile"));
        }
        let rails: [om9_core::identity::ObjectId; 4] = ordered[..4]
            .try_into()
            .map_err(|_| Error::Invalid("four rails"))?;
        let mut rail_points = Vec::with_capacity(4);
        for id in &rails {
            let entity = self
                .document()
                .entities
                .get(id)
                .ok_or(Error::Missing("four rail"))?;
            let Geometry::Curve(curve) = &entity.geometry else {
                return Err(Error::Invalid("four rail curve"));
            };
            rail_points.push(
                om9_core::surface_kernel::sample_curve(curve, 33, self.coordinator.limits())
                    .map_err(|_| Error::Unsupported("four rail curve"))?,
            );
        }
        let first_profile = self
            .document()
            .entities
            .get(&ordered[4])
            .ok_or(Error::Missing("four rail profile"))?;
        let Geometry::Curve(first_curve) = &first_profile.geometry else {
            return Err(Error::Invalid("profile curve"));
        };
        let first_points =
            om9_core::surface_kernel::sample_curve(first_curve, 64, self.coordinator.limits())
                .map_err(|_| Error::Unsupported("four rail profile curve"))?;
        let (_, rail_params) = rail_intersections(
            first_curve,
            &first_points,
            &rail_points,
            self.coordinator.limits(),
        )
        .ok_or(Error::Invalid("four rail profile intersections"))?;
        let mut rail_order = [0usize, 1, 2, 3];
        rail_order.sort_by(|left, right| rail_params[*left].total_cmp(&rail_params[*right]));
        let rails = [
            rails[rail_order[0]],
            rails[rail_order[1]],
            rails[rail_order[2]],
            rails[rail_order[3]],
        ];
        let mut mutations = Vec::with_capacity(ordered.len() - 4);
        for id in &ordered[4..] {
            let entity = self
                .document()
                .entities
                .get(id)
                .ok_or(Error::Missing("selected profile"))?;
            let Geometry::Curve(curve) = &entity.geometry else {
                return Err(Error::Invalid("profile curve"));
            };
            let points =
                om9_core::surface_kernel::sample_curve(curve, 16, self.coordinator.limits())
                    .map_err(|_| Error::Unsupported("four rail profile curve"))?;
            let mut chord = None;
            let mut normal = None;
            'outer: for i in 1..points.len() {
                let first = [
                    points[i][0] - points[0][0],
                    points[i][1] - points[0][1],
                    points[i][2] - points[0][2],
                ];
                for j in (i + 1)..points.len() {
                    let second = [
                        points[j][0] - points[0][0],
                        points[j][1] - points[0][1],
                        points[j][2] - points[0][2],
                    ];
                    let candidate = [
                        first[1] * second[2] - first[2] * second[1],
                        first[2] * second[0] - first[0] * second[2],
                        first[0] * second[1] - first[1] * second[0],
                    ];
                    let length = candidate[0].hypot(candidate[1]).hypot(candidate[2]);
                    if length > 1.0e-9 {
                        chord = Some(first);
                        normal = Some([
                            candidate[0] / length,
                            candidate[1] / length,
                            candidate[2] / length,
                        ]);
                        break 'outer;
                    }
                }
            }
            let (chord, normal) = match (chord, normal) {
                (Some(chord), Some(normal)) => (chord, normal),
                _ => return Err(Error::Invalid("four rail profile plane")),
            };
            let chord_len = chord[0].hypot(chord[1]).hypot(chord[2]);
            let metadata = om9_core::rhino_semantics::FourRailProfile {
                rails,
                origin: points[0],
                normal,
                x_axis: [
                    chord[0] / chord_len,
                    chord[1] / chord_len,
                    chord[2] / chord_len,
                ],
            };
            let mut extensions = entity.extensions.clone();
            extensions.insert(
                om9_core::model::FOUR_RAIL_PROFILE_EXTENSION_SLOT.to_string(),
                metadata.to_extension()?,
            );
            mutations.push(Mutation::Extensions {
                source: self.document().stamp(*id)?,
                extensions,
            });
        }
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "four rail profile",
            None,
        );
        let prepared = self
            .coordinator
            .prepare(TransactionRequest::new("FourRailProfile", mutations))?;
        let receipt = self.coordinator.commit(prepared)?;
        if receipt.published {
            self.refresh_scene()?;
            self.sync_published(om9_core::changes::ChangeCause::Edit);
        }
        self.events.emit(
            crate::command_events::EventKind::Finished,
            feature,
            "four rail profile",
            Some(if receipt.published {
                crate::command_events::EventOutcome::Committed
            } else {
                crate::command_events::EventOutcome::Refused
            }),
        );
        Ok(())
    }
    #[cfg(feature = "occt-brep")]
    pub(super) fn build_sweep4(&self, curves: &[Curve]) -> Result<om9_core::brep::Brep> {
        use om9_core::surface_kernel;
        if curves.len() < 6 {
            return Err(Error::Invalid("four rails and two profiles"));
        }
        let limits = self.coordinator.limits();
        let mut rails = Vec::with_capacity(4);
        for curve in &curves[..4] {
            rails.push(
                surface_kernel::sample_curve(curve, 33, limits)
                    .map_err(|_| Error::Unsupported("Sweep 4 rail"))?,
            );
        }
        let mut profiles = Vec::with_capacity(curves.len() - 4);
        for curve in &curves[4..] {
            profiles.push(
                surface_kernel::sample_curve(curve, 64, limits)
                    .map_err(|_| Error::Unsupported("Sweep 4 profile"))?,
            );
        }
        let (_, first_params) = rail_intersections(&curves[4], &profiles[0], &rails, limits)
            .ok_or(Error::Invalid("four rail profile intersections"))?;
        let mut order = [0usize, 1, 2, 3];
        order.sort_by(|left, right| first_params[*left].total_cmp(&first_params[*right]));
        let ordered_rails: Vec<Vec<Point3>> =
            order.iter().map(|index| rails[*index].clone()).collect();
        let mut entries: Vec<(f64, [f64; 4], Vec<Point3>)> = Vec::with_capacity(profiles.len());
        for (index, profile) in profiles.into_iter().enumerate() {
            let (indices, _) = rail_intersections(&curves[4 + index], &profile, &rails, limits)
                .ok_or(Error::Invalid("four rail profile intersections"))?;
            let mut ordered = [0.0f64; 4];
            for (slot, rail) in order.iter().enumerate() {
                ordered[slot] = indices[*rail];
            }
            for left in 0..4 {
                for right in (left + 1)..4 {
                    if (ordered[left] - ordered[right]).abs() <= 1.0e-6 {
                        return Err(Error::Invalid("four rail profile intersections"));
                    }
                }
            }
            entries.push((ordered[0], ordered, profile));
        }
        entries.sort_by(|left, right| left.0.total_cmp(&right.0));
        let mut sections: [Vec<Vec<Point3>>; 4] = Default::default();
        for (_, indices, profile) in entries {
            for quadrant in 0..4 {
                let start = indices[quadrant];
                let end = indices[(quadrant + 1) % 4];
                sections[quadrant].push(profile_segment(&profile, start, end));
            }
        }
        om9_kernel::creation::four_rail_sweep(
            &ordered_rails
                .try_into()
                .map_err(|_| Error::Invalid("four rails"))?,
            &sections,
            1.0e-3,
            om9_kernel::creation::DEFAULT_DEFLECTION_MM,
            limits,
        )
        .map_err(|_| Error::Unsupported("Sweep 4 builder"))
    }

    #[cfg(feature = "occt-brep")]
    pub(super) fn dispatch_sweep4(&mut self, feature: &str) -> Result<()> {
        let feature = surface_edit::static_feature(feature);
        let mut ordered = self.selected_ordered();
        if ordered.len() < 6 {
            // Profiles created by Four Rail Profile carry their four rail ids;
            // selecting only those profiles is enough to resolve the rails.
            let mut rails: Option<[om9_core::identity::ObjectId; 4]> = None;
            let mut profiles = Vec::new();
            for id in &ordered {
                let entity = self
                    .document()
                    .entities
                    .get(id)
                    .ok_or(Error::Missing("selected profile"))?;
                let metadata = om9_core::rhino_semantics::four_rail_profile_of(
                    entity,
                    self.coordinator.limits(),
                )?
                .ok_or(Error::Invalid("profile has no FourRailProfile metadata"))?;
                if let Some(existing) = rails {
                    if existing != metadata.rails {
                        return Err(Error::Invalid("profiles use different four rails"));
                    }
                } else {
                    rails = Some(metadata.rails);
                }
                profiles.push(*id);
            }
            let Some(rails) = rails else {
                return Err(Error::Invalid("four rails and two profiles"));
            };
            ordered = rails.to_vec();
            ordered.extend(profiles);
        }
        if ordered.len() < 6 {
            return Err(Error::Invalid("four rails and two profiles"));
        }
        let mut curves = Vec::with_capacity(ordered.len());
        for id in &ordered {
            let entity = self
                .document()
                .entities
                .get(id)
                .ok_or(Error::Missing("selected curve"))?;
            let Geometry::Curve(curve) = &entity.geometry else {
                return Err(Error::Invalid("curve selection"));
            };
            curves.push(curve.clone());
        }
        let brep = self.build_sweep4(&curves)?;
        let entity = Entity::new(
            surface_edit::label(feature),
            Geometry::Brep(brep),
            self.document().current_layer,
        );
        let mut inputs = Vec::with_capacity(ordered.len());
        for (index, id) in ordered.iter().enumerate() {
            inputs.push(SourceBinding {
                role: if index < 4 { "rail" } else { "profile" }.into(),
                source: self.document().stamp(*id)?,
                selection: Selection::Whole,
            });
        }
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "four rail sweep",
            None,
        );
        self.commit_entity_with_history(
            feature,
            surface_edit::label(feature),
            entity,
            "Sweep4",
            inputs,
        )
    }

    #[cfg(not(feature = "occt-brep"))]
    pub(super) fn dispatch_sweep4(&mut self, _feature: &str) -> Result<()> {
        Err(Error::Unsupported("Sweep 4 requires OCCT"))
    }
    /// Whether Sweep 1/2 History may offer `Closed`: every rail must be a closed curve
    /// and at least two profile curves must be selected (Matrix Book 1 p183/p186).
    /// `Closed=Yes` is the documented default only when this is true.
    pub(super) fn closed_sweep_available(&self, curves: &[Curve], rail_count: usize) -> bool {
        if curves.len() < rail_count + 2 {
            return false;
        }
        let tolerance = self
            .document()
            .tolerance()
            .map_or(1.0e-6, |tolerance| tolerance.absolute_mm);
        let limits = self.coordinator.limits();
        curves[..rail_count]
            .iter()
            .all(|rail| om9_core::curve_ops::is_closed(rail, tolerance, limits).unwrap_or(false))
    }
    /// Multi-profile Sweep 1/2 History through the OCCT pipe-shell adapter.
    #[cfg(feature = "occt-brep")]
    pub(super) fn dispatch_sweep_history(&mut self, feature: &str) -> Result<()> {
        let feature = surface_edit::static_feature(feature);
        let rail_count = if feature == surface_edit::SWEEP2_HISTORY {
            2
        } else {
            1
        };
        let ordered = self.selected_ordered();
        if ordered.len() <= rail_count {
            return Err(Error::Invalid("rail and profile selection"));
        }
        let mut curves = Vec::with_capacity(ordered.len());
        for id in &ordered {
            let entity = self
                .document()
                .entities
                .get(id)
                .ok_or(Error::Missing("selected curve"))?;
            let Geometry::Curve(curve) = &entity.geometry else {
                return Err(Error::Invalid("curve selection"));
            };
            curves.push(curve.clone());
        }
        let closed_available = self.closed_sweep_available(&curves, rail_count);
        self.history_options_pending = Some(HistoryOptionsPending {
            feature,
            curves,
            ids: ordered,
            rail_count,
            closed: closed_available,
            flip: false,
            closed_available,
        });
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "multi-profile sweep options",
            None,
        );
        Ok(())
    }

    #[cfg(feature = "occt-brep")]
    pub(super) fn build_sweep_history(
        &self,
        _feature: &'static str,
        curves: &[Curve],
        rail_count: usize,
        closed: bool,
        flip: bool,
        slash: Option<[Point3; 2]>,
    ) -> Result<om9_core::brep::Brep> {
        use om9_core::surface_kernel;
        let limits = self.coordinator.limits();
        let mut rails: Vec<Vec<Point3>> = Vec::with_capacity(rail_count);
        for curve in &curves[..rail_count] {
            rails.push(
                surface_kernel::sample_curve(curve, 33, limits)
                    .map_err(|_| Error::Unsupported("Sweep history builder"))?,
            );
        }
        let mut profiles: Vec<Vec<Point3>> = Vec::with_capacity(curves.len() - rail_count);
        for curve in &curves[rail_count..] {
            profiles.push(
                surface_kernel::sample_curve(curve, 24, limits)
                    .map_err(|_| Error::Unsupported("Sweep history builder"))?,
            );
        }
        if flip {
            profiles.reverse();
            for profile in &mut profiles {
                profile.reverse();
            }
        }
        if let Some([a, b]) = slash {
            let section = slash_alignment_section(&profiles, &rails, a, b)?;
            profiles.push(section);
        }
        if closed {
            let first = rails[0][0];
            let last = rails[0][rails[0].len() - 1];
            let dx = first[0] - last[0];
            let dy = first[1] - last[1];
            let dz = first[2] - last[2];
            if dx * dx + dy * dy + dz * dz > 1.0e-12 {
                return Err(Error::Invalid("closed sweep needs a closed rail"));
            }
            if profiles.len() > 1 {
                let first_profile = profiles[0].clone();
                profiles.push(first_profile);
            }
        }
        om9_kernel::creation::pipe_shell(
            &rails[0],
            rails.get(1).map(Vec::as_slice),
            &profiles,
            false,
            om9_kernel::creation::DEFAULT_DEFLECTION_MM,
            limits,
        )
        .map_err(|_| Error::Unsupported("Sweep history builder"))
    }

    #[cfg(not(feature = "occt-brep"))]
    pub(super) fn build_sweep_history(
        &self,
        _feature: &'static str,
        _curves: &[Curve],
        _rail_count: usize,
        _closed: bool,
        _flip: bool,
        _slash: Option<[Point3; 2]>,
    ) -> Result<om9_core::brep::Brep> {
        Err(Error::Unsupported("Sweep History requires OCCT"))
    }

    #[cfg(not(feature = "occt-brep"))]
    pub(super) fn dispatch_sweep_history(&mut self, _feature: &str) -> Result<()> {
        Err(Error::Unsupported("Sweep History requires OCCT"))
    }
    /// Merge two selected surfaces sharing a boundary.
    pub(super) fn dispatch_merge(&mut self, feature: &str) -> Result<()> {
        let feature = surface_edit::static_feature(feature);
        let ordered = self.selected_ordered();
        if ordered.len() != 2 {
            return Err(Error::Invalid("two surface selection"));
        }
        let mut surfaces = Vec::with_capacity(2);
        for id in ordered {
            let entity = self
                .document()
                .entities
                .get(&id)
                .ok_or(Error::Missing("selected surface"))?;
            let Geometry::Surface(surface) = &entity.geometry else {
                return Err(Error::Invalid("surface selection"));
            };
            surfaces.push(surface.clone());
        }
        let geometry =
            surface_edit::apply_merge(&surfaces[0], &surfaces[1], self.coordinator.limits())?;
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "merge surfaces",
            None,
        );
        self.commit_geometry(feature, surface_edit::label(feature), geometry)
    }
    /// Sweep all profiles after the first selected curve along that rail.
    pub(super) fn dispatch_sweep_multi(&mut self, feature: &str) -> Result<()> {
        let feature = surface_edit::static_feature(feature);
        let ordered = self.selected_ordered();
        if ordered.len() < 2 {
            return Err(Error::Invalid("rail and profile selection"));
        }
        let mut curves = Vec::with_capacity(ordered.len());
        for id in ordered {
            let entity = self
                .document()
                .entities
                .get(&id)
                .ok_or(Error::Missing("selected curve"))?;
            let Geometry::Curve(curve) = &entity.geometry else {
                return Err(Error::Invalid("curve selection"));
            };
            curves.push(curve.clone());
        }
        let rail = curves.remove(0);
        let surfaces = surface_edit::apply_sweep_multi(&rail, &curves, self.coordinator.limits())?;
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "sweep multi",
            None,
        );
        let items = surfaces
            .into_iter()
            .map(|surface| (surface_edit::label(feature), surface))
            .collect();
        self.commit_geometries(feature, surface_edit::label(feature), items)
    }
    /// Start a profile/rail command that waits for its axis.
    pub(super) fn dispatch_curve_pair(&mut self, feature: &str) -> Result<()> {
        let feature = surface_edit::static_feature(feature);
        let ordered = self.selected_ordered();
        if ordered.len() != 2 {
            return Err(Error::Invalid("profile and rail selection"));
        }
        let mut curves = Vec::with_capacity(2);
        for id in ordered {
            let entity = self
                .document()
                .entities
                .get(&id)
                .ok_or(Error::Missing("selected curve"))?;
            let Geometry::Curve(curve) = &entity.geometry else {
                return Err(Error::Invalid("curve selection"));
            };
            curves.push(curve.clone());
        }
        self.curve_pair_pending = Some(CurvePairPending {
            feature,
            profile: curves[0].clone(),
            rail: curves[1].clone(),
        });
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "rail revolve axis",
            None,
        );
        Ok(())
    }
    /// Selected-surface distance option. The numeric value itself remains owned
    /// by the surface command; this descriptor only opens the shared value entry.
    pub fn surface_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        if self.surface_pending.is_some() {
            vec![("Distance=", "Distance=")]
        } else {
            Vec::new()
        }
    }
    /// Soft-edit parameters, one stage at a time.
    pub fn soft_edit_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        let Some(pending) = &self.soft_edit_pending else {
            return Vec::new();
        };
        vec![
            ("U", "U="),
            ("V", "V="),
            ("Dx", "Dx="),
            ("Dy", "Dy="),
            ("Dz", "Dz="),
            ("UDistance", "UDistance="),
            ("VDistance", "VDistance="),
            ("Copy", if pending.copy { "Copy=No" } else { "Copy=Yes" }),
            (
                "FixEdges",
                if pending.fix_edges {
                    "FixEdges=No"
                } else {
                    "FixEdges=Yes"
                },
            ),
            ("DirConstraint", pending.constraint.next_command()),
        ]
    }
    /// Slash point entry. The two points remain viewport/point input; the
    /// command line can open either stage by name.
    pub fn slash_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        match &self.slash_pending {
            Some(pending) if pending.points.is_empty() => vec![("First=", "First=")],
            Some(_) => vec![("Second=", "Second=")],
            None => Vec::new(),
        }
    }
    /// Top11 Duplicate flags. Other Top11 actions are positional and expose no row.
    pub fn top11_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        self.top11_pending
            .as_ref()
            .map(|pending| pending.option_tokens())
            .unwrap_or_default()
    }
    /// Sweep-history flags and the Slash transition.
    pub fn history_options_tokens(&self) -> Vec<(&'static str, &'static str)> {
        let Some(pending) = &self.history_options_pending else {
            return Vec::new();
        };
        let mut tokens = vec![("Flip", if pending.flip { "Flip=No" } else { "Flip=Yes" })];
        if pending.closed_available {
            tokens.insert(
                0,
                (
                    "Closed",
                    if pending.closed {
                        "Closed=No"
                    } else {
                        "Closed=Yes"
                    },
                ),
            );
        }
        if pending.rail_count == 2 {
            tokens.push(("Slash", "Slash"));
        }
        tokens
    }
    /// Shrink options: u0, u1, v0, then v1.
    pub fn shrink_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        let Some(pending) = &self.shrink_pending else {
            return Vec::new();
        };
        if pending.u0.is_none() {
            vec![("U0=", "U0=")]
        } else if pending.u1.is_none() {
            vec![("U1=", "U1=")]
        } else if pending.v0.is_none() {
            vec![("V0=", "V0=")]
        } else if pending.v1.is_none() {
            vec![("V1=", "V1=")]
        } else {
            Vec::new()
        }
    }
    /// Variable offset options: start distance, end distance, then axis.
    pub fn variable_offset_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        let Some(pending) = &self.variable_offset_pending else {
            return Vec::new();
        };
        if pending.start.is_none() {
            vec![("Start=", "Start=")]
        } else if pending.end.is_none() {
            vec![("End=", "End=")]
        } else if pending.axis.is_none() {
            vec![("Axis=", "Axis=")]
        } else {
            Vec::new()
        }
    }
    /// Variable blend options: start scale, then end scale.
    pub fn variable_blend_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        match &self.variable_blend_pending {
            Some(pending) if pending.start_scale.is_none() => vec![("Start=", "Start=")],
            Some(pending) if pending.end_scale.is_none() => vec![("End=", "End=")],
            _ => Vec::new(),
        }
    }
    /// Variable chamfer options: start setback, then end setback.
    pub fn variable_chamfer_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        match &self.variable_chamfer_pending {
            Some(pending) if pending.start_setback.is_none() => vec![("Start=", "Start=")],
            Some(pending) if pending.end_setback.is_none() => vec![("End=", "End=")],
            _ => Vec::new(),
        }
    }
    /// Surface fillet radius (constant) or start/end radii (variable).
    pub fn fillet_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        let Some(pending) = &self.fillet_pending else {
            return Vec::new();
        };
        if pending.feature == surface_edit::VARIABLE_FILLET {
            if pending.start_radius.is_none() {
                vec![("Start=", "Start=")]
            } else if pending.end_radius.is_none() {
                vec![("End=", "End=")]
            } else {
                Vec::new()
            }
        } else {
            vec![("Radius=", "Radius=")]
        }
    }
    /// Surface chamfer setback option.
    pub fn chamfer_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        if self.chamfer_pending.is_some() {
            vec![("Setback=", "Setback=")]
        } else {
            Vec::new()
        }
    }
    /// Surface tangent options: one edge, then one direction vector.
    pub fn tangent_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        match &self.tangent_pending {
            Some(pending) if pending.edge.is_none() => vec![("Edge=", "Edge=")],
            Some(pending) if pending.direction.is_none() => vec![("Direction=", "Direction=")],
            _ => Vec::new(),
        }
    }
    /// Surface match options: one source edge, then one target edge.
    pub fn surface_pair_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        match &self.surface_pair_pending {
            Some(pending) if pending.source_edge.is_none() => {
                vec![("Source=", "Source="), ("Continuity=", "Continuity=")]
            }
            Some(pending) if pending.target_edge.is_none() => {
                vec![("Target=", "Target="), ("Continuity=", "Continuity=")]
            }
            Some(_) => vec![("Continuity=", "Continuity=")],
            None => Vec::new(),
        }
    }
    /// Surface split options: one axis, then one parameter.
    pub fn split_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        match &self.split_pending {
            Some(pending) if pending.axis.is_none() => vec![("Axis=", "Axis=")],
            Some(pending) if pending.parameter.is_none() => vec![("Parameter=", "Parameter=")],
            _ => Vec::new(),
        }
    }
    /// Surface extend options: one edge, then one distance.
    pub fn extend_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        match &self.extend_pending {
            Some(pending) if pending.edge.is_none() => vec![("Edge=", "Edge=")],
            Some(pending) if pending.distance_mm.is_none() => vec![("Distance=", "Distance=")],
            _ => Vec::new(),
        }
    }
    /// Extrude-all height option.
    pub fn extrude_all_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        if self.extrude_all_pending.is_some() {
            vec![("Height=", "Height=")]
        } else {
            Vec::new()
        }
    }
    /// Surface seam options: one axis, then one tolerance.
    pub fn seam_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        match &self.seam_pending {
            Some(pending) if pending.axis.is_none() => vec![("Axis=", "Axis=")],
            Some(pending) if pending.tolerance_mm.is_none() => vec![("Tolerance=", "Tolerance=")],
            _ => Vec::new(),
        }
    }
    /// Surface tween fraction option.
    pub fn tween_surface_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        if self.tween_pending.is_some() {
            vec![("Fraction=", "Fraction=")]
        } else {
            Vec::new()
        }
    }
    /// Rail-revolve axis option; the six values stay owner-validated.
    pub fn curve_pair_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        if self.curve_pair_pending.is_some() {
            vec![("Axis=", "Axis=")]
        } else {
            Vec::new()
        }
    }
    /// Start a selected-surface command that waits for one numeric distance.
    pub(super) fn dispatch_surface_distance(&mut self, feature: &str) -> Result<()> {
        let feature = surface_edit::static_feature(feature);
        let id = self
            .selection
            .iter()
            .next()
            .copied()
            .ok_or(Error::Missing("surface selection"))?;
        let entity = self
            .document()
            .entities
            .get(&id)
            .ok_or(Error::Missing("selected surface"))?;
        let Geometry::Surface(surface) = &entity.geometry else {
            return Err(Error::Invalid("surface selection"));
        };
        self.surface_pending = Some(SurfacePending {
            feature,
            surface: surface.clone(),
        });
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "surface distance",
            None,
        );
        Ok(())
    }
    /// Flatten the selected surface with `Smash` and report the measured stretch.
    ///
    /// `Smash` is the flattening that accepts a surface `Unroll Surface` can only
    /// approximate: it runs the same arc-length map and reports the sampling-level
    /// stretch as a progress event instead of refusing the input.
    pub(super) fn dispatch_smash(&mut self) -> Result<()> {
        let id = self
            .selection
            .iter()
            .next()
            .copied()
            .ok_or(Error::Missing("surface selection"))?;
        let entity = self
            .document()
            .entities
            .get(&id)
            .ok_or(Error::Missing("selected surface"))?;
        let Geometry::Surface(surface) = &entity.geometry else {
            return Err(Error::Invalid("surface selection"));
        };
        let smash = surface_edit::apply_smash(surface, self.coordinator.limits())?;
        self.events.emit(
            crate::command_events::EventKind::Started,
            surface_edit::SMASH,
            "selected surface",
            None,
        );
        self.events.emit(
            crate::command_events::EventKind::Progress,
            surface_edit::SMASH,
            &format!("stretch {:.3}", smash.stretch),
            None,
        );
        self.commit_geometry(
            surface_edit::SMASH,
            surface_edit::label(surface_edit::SMASH),
            Geometry::Surface(smash.surface),
        )
    }
}
