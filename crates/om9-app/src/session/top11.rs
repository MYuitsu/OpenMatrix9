// SPDX-License-Identifier: LGPL-2.1-or-later
//! Matrix Top 11 command adapters (OM9-L2-03-08).
//!
//! The toolbar, its command names and the `top11` command line share one execution
//! path. Move/Rotate/Mirror collect their points through the same `accept_world_point`
//! entry the viewport uses, then commit one transaction; the other actions are
//! immediate. Ring Rail is deliberately unavailable until the M04 builder exists.
use super::*;
use om9_core::{
    geometry::{Point3, Transform},
    identity::ObjectId,
    top11::{self, GripMode, Top11Action, Top11Availability, Top11Context},
};
use std::collections::BTreeSet;

/// Mirrored copies plus replacement geometry for the delete-input option.
type MirrorBatch = (Vec<Entity>, Vec<(ObjectId, Geometry)>);

/// What a Top 11 toolbar result means for the status line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Top11Result {
    Completed {
        label: &'static str,
        objects: usize,
    },
    Awaiting {
        label: &'static str,
        prompt: &'static str,
    },
    GripMode {
        label: &'static str,
        mode: Option<GripMode>,
    },
}

impl Top11Result {
    pub fn message(&self) -> String {
        match self {
            Self::Completed { label, objects } => format!("{label}: {objects} object(s)"),
            Self::Awaiting { label, prompt } => format!("{label}: {prompt}"),
            Self::GripMode { label, mode } => match mode {
                Some(GripMode::Edit) => format!("{label}: edit points on"),
                Some(GripMode::Control) => format!("{label}: control points on"),
                None => format!("{label}: points off"),
            },
        }
    }
}

/// The in-flight point/object-driven Top 11 command.
#[derive(Debug, Clone, PartialEq)]
pub enum Top11Pending {
    SelectObjects {
        action: Top11Action,
    },
    Duplicate {
        ids: BTreeSet<ObjectId>,
        base: Option<Point3>,
        copies: usize,
        vertical: bool,
        from_last: bool,
        use_last_distance: bool,
        use_last_direction: bool,
        last_delta: Option<Point3>,
        last_destination: Option<Point3>,
    },
    Split {
        target: ObjectId,
        cutters: Vec<ObjectId>,
    },
    SplitMany {
        targets: Vec<ObjectId>,
        cutters: Vec<ObjectId>,
    },
    SplitPoint {
        target: ObjectId,
    },
    Trim {
        target: ObjectId,
        cutters: Vec<ObjectId>,
        fractions: Option<Vec<f64>>,
    },
    Move {
        ids: BTreeSet<ObjectId>,
        base: Option<Point3>,
        vertical: bool,
    },
    Rotate {
        ids: BTreeSet<ObjectId>,
        center: Option<Point3>,
        reference: Option<Point3>,
        copy: bool,
    },
    Mirror {
        ids: BTreeSet<ObjectId>,
        points: Vec<Point3>,
        copy: bool,
        three_point: bool,
    },
}

impl Top11Pending {
    pub fn action(&self) -> Top11Action {
        match self {
            Self::SelectObjects { action } => *action,
            Self::Duplicate { .. } => Top11Action::Duplicate,
            Self::Split { .. } => Top11Action::Split,
            Self::SplitMany { .. } => Top11Action::Split,
            Self::SplitPoint { .. } => Top11Action::Split,
            Self::Trim { .. } => Top11Action::Trim,
            Self::Move { .. } => Top11Action::Move,
            Self::Rotate { .. } => Top11Action::Rotate,
            Self::Mirror { .. } => Top11Action::Mirror,
        }
    }

    pub fn prompt(&self) -> &'static str {
        match self {
            Self::SelectObjects { .. } => "select objects (Enter to continue)",
            Self::Duplicate { base: None, .. } => "pick the base point",
            Self::Duplicate { .. } => "pick a destination point (Enter to finish)",
            Self::Split { .. } => "pick cutting curves (Enter to execute)",
            Self::SplitMany { .. } => "pick cutting curves (Enter to execute)",
            Self::SplitPoint { .. } => "pick a point on the line",
            Self::Trim {
                fractions: None, ..
            } => "pick cutting curves (Enter to continue)",
            Self::Trim { .. } => "pick the portion to remove",
            Self::Move { base: None, .. } => "pick the base point",
            Self::Move { base: Some(_), .. } => "pick the destination point",
            Self::Rotate { center: None, .. } => "pick the center point",
            Self::Rotate {
                reference: None, ..
            } => "pick the reference point",
            Self::Rotate { .. } => "pick the second reference point",
            Self::Mirror {
                three_point: true, ..
            } => "pick three points on the mirror plane",
            Self::Mirror { .. } => {
                "pick two points on the mirror plane (type 3Point for any plane)"
            }
        }
    }

    /// True when the next viewport click must resolve an object, not a point.
    pub fn needs_object(&self) -> bool {
        matches!(
            self,
            Self::SelectObjects { .. }
                | Self::Split { .. }
                | Self::SplitMany { .. }
                | Self::Trim {
                    fractions: None,
                    ..
                }
        )
    }
    /// Current Duplicate flags as command-line options. Other Top11 actions
    /// remain positional object/point input and therefore expose no fake row.
    pub fn option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        if let Self::Split { cutters, .. } = self {
            return if cutters.is_empty() {
                vec![("Point", "Point")]
            } else {
                Vec::new()
            };
        }
        if let Self::Move { vertical, .. } = self {
            return vec![(
                "Vertical",
                if *vertical {
                    "Vertical=No"
                } else {
                    "Vertical=Yes"
                },
            )];
        }
        if let Self::Mirror { copy, .. } = self {
            return vec![
                ("Copy", if *copy { "Copy=No" } else { "Copy=Yes" }),
                ("3Point", "3Point"),
                ("XAxis", "XAxis"),
                ("YAxis", "YAxis"),
            ];
        }
        if let Self::Rotate { copy, .. } = self {
            return vec![("Copy", if *copy { "Copy=No" } else { "Copy=Yes" })];
        }
        let Self::Duplicate {
            base,
            vertical,
            from_last,
            use_last_distance,
            use_last_direction,
            ..
        } = self
        else {
            return Vec::new();
        };
        let mut options = vec![
            (
                "Vertical",
                if *vertical {
                    "Vertical=No"
                } else {
                    "Vertical=Yes"
                },
            ),
            (
                "FromLastPoint",
                if *from_last {
                    "FromLastPoint=No"
                } else {
                    "FromLastPoint=Yes"
                },
            ),
            (
                "UseLastDistance",
                if *use_last_distance {
                    "UseLastDistance=No"
                } else {
                    "UseLastDistance=Yes"
                },
            ),
            (
                "UseLastDirection",
                if *use_last_direction {
                    "UseLastDirection=No"
                } else {
                    "UseLastDirection=Yes"
                },
            ),
        ];
        if base.is_none() {
            options.insert(0, ("InPlace", "InPlace"));
        }
        options
    }
}

fn vector(from: Point3, to: Point3) -> Point3 {
    [to[0] - from[0], to[1] - from[1], to[2] - from[2]]
}

fn add(a: Point3, b: Point3) -> Point3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn scale(a: Point3, factor: f64) -> Point3 {
    [a[0] * factor, a[1] * factor, a[2] * factor]
}

fn norm(a: Point3) -> f64 {
    dot(a, a).sqrt()
}

fn duplicate_option(text: &str) -> Option<(&str, Option<bool>)> {
    let (key, value) = match text.split_once('=') {
        Some((key, value)) => {
            let flag = match value.trim().to_ascii_lowercase().as_str() {
                "yes" | "y" | "on" | "1" | "true" => true,
                "no" | "n" | "off" | "0" | "false" => false,
                _ => return None,
            };
            (key.trim(), Some(flag))
        }
        None => (text.trim(), None),
    };
    top11_duplicate_key(key).then_some((key, value))
}

fn top11_duplicate_key(key: &str) -> bool {
    matches!(
        key.to_ascii_lowercase().as_str(),
        "vertical"
            | "fromlastpoint"
            | "from_last_point"
            | "uselastdistance"
            | "use_last_distance"
            | "uselastdirection"
            | "use_last_direction"
    )
}

fn child_entity(source: &Entity, geometry: Geometry) -> Entity {
    let mut entity = source.clone();
    entity.id = ObjectId::new();
    entity.generation = 1;
    entity.geometry_revision = 1;
    entity.attributes_revision = 1;
    entity.geometry = geometry;
    entity
}

fn dot(a: Point3, b: Point3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: Point3, b: Point3) -> Point3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn unit(a: Point3) -> Result<Point3> {
    let length = dot(a, a).sqrt();
    if !length.is_finite() || length <= 1e-12 {
        return Err(Error::Invalid("Top 11 direction"));
    }
    Ok([a[0] / length, a[1] / length, a[2] / length])
}

fn signed_degrees(axis: Point3, from: Point3, to: Point3) -> Result<f64> {
    let axis = unit(axis)?;
    let from = unit(from)?;
    let to = unit(to)?;
    let angle = dot(axis, cross(from, to)).atan2(dot(from, to));
    Ok(angle.to_degrees())
}

impl Controller {
    pub fn top11_context(&self) -> Top11Context {
        Top11Context::capture(self.document(), &self.selection, self.is_running())
    }

    pub fn top11_availability(&self) -> [Top11Availability; 11] {
        let mut items = top11::toolbar(&self.top11_context());
        // M04 owns the Ring Rail builder now: the core model keeps it conservatively
        // disabled, and the application enables it when the builder provider is compiled in.
        for item in &mut items {
            if !self.is_running()
                && self.selection.is_empty()
                && item.action != Top11Action::RingRail
            {
                *item = Top11Availability {
                    action: item.action,
                    enabled: true,
                    reason: "",
                };
            }
            if item.action == Top11Action::Split {
                let context = self.top11_context();
                if !context.running && context.selected > 1 && context.curves == context.selected {
                    *item = Top11Availability {
                        action: Top11Action::Split,
                        enabled: true,
                        reason: "",
                    };
                }
            }
            if item.action == Top11Action::RingRail {
                *item = Top11Availability {
                    action: Top11Action::RingRail,
                    enabled: !self.is_running(),
                    reason: if self.is_running() {
                        "a command is already running"
                    } else {
                        ""
                    },
                };
            }
            #[cfg(feature = "occt-brep")]
            if item.action == Top11Action::Explode {
                let context = self.top11_context();
                if !context.running
                    && context.breps > 0
                    && context.curves + context.breps == context.selected
                {
                    *item = Top11Availability {
                        action: Top11Action::Explode,
                        enabled: true,
                        reason: "",
                    };
                }
            }
            #[cfg(feature = "occt-brep")]
            if item.action == Top11Action::Join {
                let context = self.top11_context();
                if !context.running && context.selected >= 2 && context.breps == context.selected {
                    *item = Top11Availability {
                        action: Top11Action::Join,
                        enabled: true,
                        reason: "",
                    };
                }
            }
        }
        items
    }

    pub fn top11_pending(&self) -> Option<&Top11Pending> {
        self.top11_pending.as_ref()
    }

    pub fn top11_prompt(&self) -> Option<&'static str> {
        self.top11_pending.as_ref().map(Top11Pending::prompt)
    }

    pub fn top11_grip_mode(&self) -> Option<GripMode> {
        self.top11_grip
    }

    pub fn top11_grip_points(&self) -> Result<Vec<Point3>> {
        match self.top11_grip {
            Some(mode) => top11::grip_points(
                self.document(),
                &self.selection,
                mode,
                self.coordinator.limits(),
            ),
            None => Ok(Vec::new()),
        }
    }

    pub fn top11_grip_handles(&self) -> Result<Vec<top11::GripHandle>> {
        match self.top11_grip {
            Some(mode) => top11::grip_handles(
                self.document(),
                &self.selection,
                mode,
                self.coordinator.limits(),
            ),
            None => Ok(Vec::new()),
        }
    }

    /// Commit one grip drag as a geometry edit.
    pub fn top11_move_grip(
        &mut self,
        handle: top11::GripHandle,
        world: Point3,
    ) -> Result<Top11Result> {
        let geometry =
            top11::move_grip_geometry(self.document(), handle, world, self.coordinator.limits())?;
        let source = self.document().stamp(handle.object)?;
        self.commit_top11(
            if handle.mode == GripMode::Edit {
                Top11Action::EditPoints
            } else {
                Top11Action::ControlPoints
            },
            vec![Mutation::Geometry { source, geometry }],
            Some(vec![handle.object]),
        )?;
        Ok(Top11Result::Completed {
            label: "Grip edit",
            objects: 1,
        })
    }

    pub fn top11_needs_object(&self) -> bool {
        self.top11_pending
            .as_ref()
            .is_some_and(Top11Pending::needs_object)
    }

    /// Duplicate in place without a point prompt (the command's `In Place` option).
    pub fn top11_duplicate_in_place(&mut self) -> Result<Top11Result> {
        if self.is_running() {
            return Err(Error::Busy);
        }
        self.duplicate_in_place_ids(&self.selection.clone())
    }

    fn duplicate_in_place_ids(&mut self, sources: &BTreeSet<ObjectId>) -> Result<Top11Result> {
        let copies =
            top11::duplicate_entities(self.document(), sources, self.coordinator.limits())?;
        let ids: Vec<ObjectId> = copies.iter().map(|entity| entity.id).collect();
        let mutations = copies
            .into_iter()
            .map(|entity| Mutation::Insert {
                document: self.document,
                entity,
            })
            .collect();
        self.commit_top11(Top11Action::Duplicate, mutations, Some(ids))?;
        Ok(Top11Result::Completed {
            label: "Duplicate (In Place)",
            objects: self.selection.len(),
        })
    }

    /// Start a Mirror prompt. `copy = false` is the command's Delete-Input option.
    pub fn top11_mirror(&mut self, copy: bool) -> Result<Top11Result> {
        self.top11_mirror_mode(copy, false)
    }

    pub fn top11_mirror_three_point(&mut self, copy: bool) -> Result<Top11Result> {
        self.top11_mirror_mode(copy, true)
    }

    /// Switch an active Mirror prompt between the two-point vertical plane and the
    /// three-point arbitrary plane without restarting the command.
    pub fn top11_mirror_use_three_point(&mut self, three_point: bool) -> Result<bool> {
        let Some(Top11Pending::Mirror {
            three_point: current,
            points,
            ..
        }) = self.top11_pending.as_mut()
        else {
            return Ok(false);
        };
        *current = three_point;
        points.clear();
        Ok(true)
    }

    fn top11_mirror_mode(&mut self, copy: bool, three_point: bool) -> Result<Top11Result> {
        let context = self.top11_context();
        let availability = top11::availability(Top11Action::Mirror, &context);
        if !availability.enabled {
            return Err(Error::Unsupported(availability.reason));
        }
        self.top11_pending = Some(Top11Pending::Mirror {
            ids: self.selection.clone(),
            points: Vec::new(),
            copy,
            three_point,
        });
        Ok(Top11Result::Awaiting {
            label: Top11Action::Mirror.label(),
            prompt: if three_point {
                "pick three points on the mirror plane"
            } else {
                "pick two points on the mirror plane"
            },
        })
    }

    /// Set the Mirror copy/delete-input flag without re-entering the command.
    pub fn top11_set_mirror_copy(&mut self, copy: bool) -> bool {
        let Some(Top11Pending::Mirror { copy: current, .. }) = self.top11_pending.as_mut() else {
            return false;
        };
        *current = copy;
        true
    }
    /// Set one Duplicate option while the base-point prompt is active.
    pub fn top11_set_duplicate_option(&mut self, key: &str, value: Option<bool>) -> Result<bool> {
        let Some(Top11Pending::Duplicate {
            vertical,
            from_last,
            use_last_distance,
            use_last_direction,
            ..
        }) = self.top11_pending.as_mut()
        else {
            return Ok(false);
        };
        let target = match key.to_ascii_lowercase().as_str() {
            "vertical" => vertical,
            "fromlastpoint" | "from_last_point" => from_last,
            "uselastdistance" | "use_last_distance" => use_last_distance,
            "uselastdirection" | "use_last_direction" => use_last_direction,
            _ => return Ok(false),
        };
        *target = value.unwrap_or(!*target);
        Ok(true)
    }

    /// Finish a repeating Duplicate prompt. Returns `None` when the pending action
    /// is not a repeatable one.
    pub fn top11_finish(&mut self) -> Option<Result<Top11Result>> {
        let pending = self.top11_pending.clone()?;
        match pending {
            Top11Pending::SelectObjects { action } => {
                if self.selection.is_empty() {
                    return Some(Err(Error::Missing("selected objects")));
                }
                self.top11_pending = None;
                let result = self.top11_invoke(action);
                if result.is_err() {
                    self.top11_pending = Some(Top11Pending::SelectObjects { action });
                }
                Some(result)
            }
            Top11Pending::Duplicate { copies, .. } => {
                self.top11_pending = None;
                Some(Ok(Top11Result::Completed {
                    label: "Duplicate",
                    objects: copies,
                }))
            }
            Top11Pending::Split { target, cutters } => Some(self.finish_split(target, cutters)),
            Top11Pending::SplitMany { targets, cutters } => {
                Some(self.finish_split_many(targets, cutters))
            }
            Top11Pending::Trim {
                target,
                cutters,
                fractions: None,
            } => Some(self.finish_trim_cutters(target, cutters)),
            _ => None,
        }
    }

    fn finish_split(&mut self, target: ObjectId, cutters: Vec<ObjectId>) -> Result<Top11Result> {
        if cutters.is_empty() {
            return Err(Error::Missing("cutting curve"));
        }
        let fractions = self.cutter_fractions_all(target, &cutters)?;
        let curve = match &self.document().entities[&target].geometry {
            Geometry::Curve(curve) => curve.clone(),
            _ => return Err(Error::Missing("split curve")),
        };
        let pieces = top11::split_at_fractions(&curve, &fractions, self.coordinator.limits())?;
        let count = pieces.len();
        self.commit_cut(Top11Action::Split, target, pieces)?;
        self.top11_pending = None;
        Ok(Top11Result::Completed {
            label: Top11Action::Split.label(),
            objects: count,
        })
    }

    fn finish_split_many(
        &mut self,
        targets: Vec<ObjectId>,
        cutters: Vec<ObjectId>,
    ) -> Result<Top11Result> {
        if cutters.is_empty() {
            return Err(Error::Missing("cutting curve"));
        }
        let mut mutations = Vec::new();
        let mut selected = Vec::new();
        let mut changed = false;
        for target in targets {
            let entity = self
                .document()
                .entities
                .get(&target)
                .ok_or(Error::Missing("split curve"))?;
            let Geometry::Curve(curve) = &entity.geometry else {
                return Err(Error::Invalid("split target must be a curve"));
            };
            let mut fractions = Vec::new();
            for cutter in &cutters {
                match self.cutter_fractions(target, *cutter) {
                    Ok(hit) => fractions.extend(hit),
                    Err(Error::Invalid("cutting object has no intersection")) => {}
                    Err(error) => return Err(error),
                }
            }
            if fractions.is_empty() {
                selected.push(target);
                continue;
            }
            let pieces = top11::split_at_fractions(curve, &fractions, self.coordinator.limits())?;
            if pieces.len() < 2 {
                return Err(Error::Invalid("split produced no pieces"));
            }
            mutations.push(Mutation::Delete {
                source: self.document().stamp(target)?,
            });
            for piece in pieces {
                let child = child_entity(entity, Geometry::Curve(piece));
                selected.push(child.id);
                mutations.push(Mutation::Insert {
                    document: self.document,
                    entity: child,
                });
            }
            changed = true;
        }
        if !changed {
            return Err(Error::Invalid("cutting objects have no intersection"));
        }
        let count = selected.len();
        self.commit_top11(Top11Action::Split, mutations, Some(selected))?;
        self.top11_pending = None;
        Ok(Top11Result::Completed {
            label: Top11Action::Split.label(),
            objects: count,
        })
    }

    fn top11_split_point_mode(&mut self) -> Result<Top11Result> {
        let Some(Top11Pending::Split { target, cutters }) = self.top11_pending.as_ref() else {
            return Err(Error::Invalid("split point option"));
        };
        if !cutters.is_empty() {
            return Err(Error::Invalid("remove cutting objects before Point"));
        }
        let target = *target;
        let entity = self
            .document()
            .entities
            .get(&target)
            .ok_or(Error::Missing("split curve"))?;
        if !matches!(entity.geometry, Geometry::Curve(Curve::Line { .. })) {
            return Err(Error::Unsupported("split by point currently needs a line"));
        }
        self.top11_pending = Some(Top11Pending::SplitPoint { target });
        Ok(Top11Result::Awaiting {
            label: Top11Action::Split.label(),
            prompt: "pick a point on the line",
        })
    }

    fn finish_trim_cutters(
        &mut self,
        target: ObjectId,
        cutters: Vec<ObjectId>,
    ) -> Result<Top11Result> {
        if cutters.is_empty() {
            return Err(Error::Missing("cutting curve"));
        }
        let fractions = self.cutter_fractions_all(target, &cutters)?;
        self.top11_pending = Some(Top11Pending::Trim {
            target,
            cutters,
            fractions: Some(fractions),
        });
        Ok(Top11Result::Awaiting {
            label: Top11Action::Trim.label(),
            prompt: "pick the portion to remove",
        })
    }

    /// Run an immediate Top 11 action or start its point prompt.
    pub fn top11_invoke(&mut self, action: Top11Action) -> Result<Top11Result> {
        if !self.is_running() && self.selection.is_empty() && action != Top11Action::RingRail {
            self.top11_pending = Some(Top11Pending::SelectObjects { action });
            return Ok(Top11Result::Awaiting {
                label: action.label(),
                prompt: "select objects (Enter to continue)",
            });
        }
        let context = self.top11_context();
        let availability = if matches!(
            action,
            Top11Action::RingRail | Top11Action::Explode | Top11Action::Join | Top11Action::Split
        ) {
            self.top11_availability()
                .into_iter()
                .find(|item| item.action == action)
                .ok_or(Error::Missing("top11 ring rail"))?
        } else {
            top11::availability(action, &context)
        };
        if !availability.enabled {
            return Err(Error::Unsupported(availability.reason));
        }
        match action {
            Top11Action::Duplicate => {
                let ids = self.selection.clone();
                self.top11_pending = Some(Top11Pending::Duplicate {
                    ids,
                    base: None,
                    copies: 0,
                    vertical: false,
                    from_last: false,
                    use_last_distance: false,
                    use_last_direction: false,
                    last_delta: None,
                    last_destination: None,
                });
                self.events.emit(
                    crate::command_events::EventKind::Started,
                    action.command(),
                    "duplicate base point",
                    None,
                );
                Ok(Top11Result::Awaiting {
                    label: action.label(),
                    prompt: "pick the base point",
                })
            }
            Top11Action::Explode => {
                let originals: Vec<ObjectId> = self.selection.iter().copied().collect();
                let mut pieces = Vec::new();
                for id in &self.selection {
                    let entity = self
                        .document()
                        .entities
                        .get(id)
                        .ok_or(Error::Missing("object"))?;
                    match &entity.geometry {
                        Geometry::Curve(_) => pieces.extend(top11::explode_entities(
                            self.document(),
                            &BTreeSet::from([*id]),
                            self.coordinator.limits(),
                        )?),
                        Geometry::Brep(brep) => {
                            #[cfg(feature = "occt-brep")]
                            {
                                let limits = self.coordinator.limits();
                                let count = om9_kernel::creation::measure(brep, limits)
                                    .map_err(|error| Error::Io(format!("BRep explode: {error}")))?
                                    .faces;
                                if count < 2 {
                                    return Err(Error::Unsupported("explode needs a polysurface"));
                                }
                                pieces
                                    .try_reserve(count)
                                    .map_err(|_| Error::Budget("exploded faces"))?;
                                for index in 0..count {
                                    let face = om9_kernel::editing::extract_face(
                                        brep,
                                        index,
                                        brep.display.deflection_mm,
                                        limits,
                                    )
                                    .map_err(|error| Error::Io(format!("BRep explode: {error}")))?;
                                    pieces.push(child_entity(entity, Geometry::Brep(face)));
                                }
                            }
                            #[cfg(not(feature = "occt-brep"))]
                            {
                                let _ = brep;
                                return Err(Error::Unsupported(
                                    "BRep explode needs the OCCT kernel feature",
                                ));
                            }
                        }
                        _ => return Err(Error::Unsupported("explode needs curves or BReps")),
                    }
                }
                let ids: Vec<ObjectId> = pieces.iter().map(|entity| entity.id).collect();
                let mut mutations: Vec<Mutation> = originals
                    .iter()
                    .map(|id| {
                        Ok(Mutation::Delete {
                            source: self.document().stamp(*id)?,
                        })
                    })
                    .collect::<Result<_>>()?;
                mutations.extend(pieces.into_iter().map(|entity| Mutation::Insert {
                    document: self.document,
                    entity,
                }));
                self.commit_top11(action, mutations, Some(ids))?;
                Ok(Top11Result::Completed {
                    label: action.label(),
                    objects: self.selection.len(),
                })
            }
            Top11Action::Join => {
                let originals: Vec<ObjectId> = self.selection.iter().copied().collect();
                let joined = if context.breps > 2 && context.breps == context.selected {
                    #[cfg(feature = "occt-brep")]
                    {
                        self.join_many_breps(&originals)?
                    }
                    #[cfg(not(feature = "occt-brep"))]
                    {
                        return Err(Error::Unsupported(
                            "BRep join needs the OCCT kernel feature",
                        ));
                    }
                } else if context.breps == 2 && context.selected == 2 {
                    #[cfg(feature = "occt-brep")]
                    {
                        let first = &self.document().entities[&originals[0]];
                        let second = &self.document().entities[&originals[1]];
                        let (Geometry::Brep(a), Geometry::Brep(b)) =
                            (&first.geometry, &second.geometry)
                        else {
                            return Err(Error::Invalid("BRep join selection"));
                        };
                        let limits = self.coordinator.limits();
                        let tolerance = self.document().effective_tolerance()?.absolute_mm;
                        let world_a = if first.placement == Transform::identity() {
                            a.clone()
                        } else {
                            om9_kernel::transform_ops::transform_brep(
                                a,
                                &first.placement,
                                a.display.deflection_mm,
                                limits,
                            )
                            .map_err(|e| Error::Io(format!("BRep join: {e}")))?
                        };
                        let world_b = if second.placement == Transform::identity() {
                            b.clone()
                        } else {
                            om9_kernel::transform_ops::transform_brep(
                                b,
                                &second.placement,
                                b.display.deflection_mm,
                                limits,
                            )
                            .map_err(|e| Error::Io(format!("BRep join: {e}")))?
                        };
                        let before_a = om9_kernel::creation::measure(&world_a, limits)
                            .map_err(|e| Error::Io(format!("BRep join: {e}")))?;
                        let before_b = om9_kernel::creation::measure(&world_b, limits)
                            .map_err(|e| Error::Io(format!("BRep join: {e}")))?;
                        let result = om9_kernel::editing::sew(
                            &world_a,
                            &world_b,
                            tolerance,
                            a.display.deflection_mm.min(b.display.deflection_mm),
                            limits,
                        )
                        .map_err(|e| Error::Io(format!("BRep join: {e}")))?;
                        let after = om9_kernel::creation::measure(&result, limits)
                            .map_err(|e| Error::Io(format!("BRep join: {e}")))?;
                        if after.faces != before_a.faces + before_b.faces
                            || after.shells != 1
                            || after.naked_edges >= before_a.naked_edges + before_b.naked_edges
                        {
                            return Err(Error::Invalid("BRep faces do not share a joinable edge"));
                        }
                        let local = if first.placement == Transform::identity() {
                            result
                        } else {
                            let inverse = om9_kernel::transform_ops::inverse(&first.placement)
                                .map_err(|e| Error::Io(format!("BRep join: {e}")))?;
                            om9_kernel::transform_ops::transform_brep(
                                &result,
                                &inverse,
                                result.display.deflection_mm,
                                limits,
                            )
                            .map_err(|e| Error::Io(format!("BRep join: {e}")))?
                        };
                        child_entity(first, Geometry::Brep(local))
                    }
                    #[cfg(not(feature = "occt-brep"))]
                    {
                        return Err(Error::Unsupported(
                            "BRep join needs the OCCT kernel feature",
                        ));
                    }
                } else {
                    top11::join_entities(
                        self.document(),
                        &self.selection,
                        1e-7,
                        self.coordinator.limits(),
                    )?
                };
                let id = joined.id;
                let mut mutations: Vec<Mutation> = originals
                    .iter()
                    .map(|original| {
                        Ok(Mutation::Delete {
                            source: self.document().stamp(*original)?,
                        })
                    })
                    .collect::<Result<_>>()?;
                mutations.push(Mutation::Insert {
                    document: self.document,
                    entity: joined,
                });
                self.commit_top11(action, mutations, Some(vec![id]))?;
                Ok(Top11Result::Completed {
                    label: action.label(),
                    objects: 1,
                })
            }
            Top11Action::Split => {
                if self.selection.len() > 1 {
                    self.top11_pending = Some(Top11Pending::SplitMany {
                        targets: self.selection.iter().copied().collect(),
                        cutters: Vec::new(),
                    });
                    return Ok(Top11Result::Awaiting {
                        label: action.label(),
                        prompt: "pick cutting curves (Enter to execute)",
                    });
                }
                let target = self
                    .selection
                    .iter()
                    .next()
                    .copied()
                    .ok_or(Error::Missing("split curve"))?;
                self.top11_pending = Some(Top11Pending::Split {
                    target,
                    cutters: Vec::new(),
                });
                Ok(Top11Result::Awaiting {
                    label: action.label(),
                    prompt: "pick cutting curves (Enter to execute)",
                })
            }
            Top11Action::Trim => {
                let target = self
                    .selection
                    .iter()
                    .next()
                    .copied()
                    .ok_or(Error::Missing("trim curve"))?;
                self.top11_pending = Some(Top11Pending::Trim {
                    target,
                    cutters: Vec::new(),
                    fractions: None,
                });
                Ok(Top11Result::Awaiting {
                    label: action.label(),
                    prompt: "pick cutting curves (Enter to continue)",
                })
            }
            Top11Action::EditPoints | Top11Action::ControlPoints => {
                let mode = if action == Top11Action::EditPoints {
                    GripMode::Edit
                } else {
                    GripMode::Control
                };
                let next = if self.top11_grip == Some(mode) {
                    None
                } else {
                    Some(mode)
                };
                self.top11_grip = next;
                Ok(Top11Result::GripMode {
                    label: action.label(),
                    mode: next,
                })
            }
            Top11Action::Move | Top11Action::Rotate | Top11Action::Mirror => {
                let ids = self.selection.clone();
                self.top11_pending = Some(match action {
                    Top11Action::Move => Top11Pending::Move {
                        ids,
                        base: None,
                        vertical: false,
                    },
                    Top11Action::Rotate => Top11Pending::Rotate {
                        ids,
                        center: None,
                        reference: None,
                        copy: false,
                    },
                    Top11Action::Mirror => Top11Pending::Mirror {
                        ids,
                        points: Vec::new(),
                        copy: true,
                        three_point: false,
                    },
                    _ => unreachable!(),
                });
                self.events.emit(
                    crate::command_events::EventKind::Started,
                    action.command(),
                    "top 11",
                    None,
                );
                Ok(Top11Result::Awaiting {
                    label: action.label(),
                    prompt: self.top11_prompt().unwrap_or("pick a point"),
                })
            }
            Top11Action::RingRail => {
                self.begin_builder(om9_core::builder::BuilderKind::RingRail)?;
                Ok(Top11Result::Awaiting {
                    label: "Ring Rail",
                    prompt: "builder panel",
                })
            }
        }
    }

    /// Feed one world point to the pending Top 11 command.
    pub fn top11_accept_point(&mut self, point: Point3) -> Result<Top11Result> {
        finite(point)?;
        let pending = self
            .top11_pending
            .take()
            .ok_or(Error::Missing("top 11 point command"))?;
        match pending {
            Top11Pending::SelectObjects { action } => {
                self.top11_pending = Some(Top11Pending::SelectObjects { action });
                Err(Error::Invalid("select an object"))
            }
            Top11Pending::SplitPoint { target } => {
                let result = (|| {
                    let entity = self
                        .document()
                        .entities
                        .get(&target)
                        .ok_or(Error::Missing("split curve"))?;
                    let Geometry::Curve(Curve::Line { start, end, .. }) = &entity.geometry else {
                        return Err(Error::Unsupported("split by point currently needs a line"));
                    };
                    let start = entity.placement.point(*start)?;
                    let end = entity.placement.point(*end)?;
                    let direction = vector(start, end);
                    let squared = dot(direction, direction);
                    if squared <= 1e-24 {
                        return Err(Error::Invalid("degenerate split line"));
                    }
                    let fraction = dot(vector(start, point), direction) / squared;
                    let closest = add(start, scale(direction, fraction));
                    let tolerance = self.document().effective_tolerance()?.absolute_mm;
                    if !(0.0 < fraction && fraction < 1.0)
                        || norm(vector(closest, point)) > tolerance
                    {
                        return Err(Error::Invalid(
                            "point must lie inside the line within document tolerance",
                        ));
                    }
                    let Geometry::Curve(curve) = &entity.geometry else {
                        unreachable!()
                    };
                    let pieces =
                        top11::split_at_fractions(curve, &[fraction], self.coordinator.limits())?;
                    let count = pieces.len();
                    self.commit_cut(Top11Action::Split, target, pieces)?;
                    Ok(Top11Result::Completed {
                        label: Top11Action::Split.label(),
                        objects: count,
                    })
                })();
                if result.is_err() {
                    self.top11_pending = Some(Top11Pending::SplitPoint { target });
                }
                result
            }
            Top11Pending::Duplicate {
                ids,
                base: None,
                copies,
                vertical,
                from_last,
                use_last_distance,
                use_last_direction,
                last_delta,
                last_destination,
            } => {
                self.top11_pending = Some(Top11Pending::Duplicate {
                    ids,
                    base: Some(point),
                    copies,
                    vertical,
                    from_last,
                    use_last_distance,
                    use_last_direction,
                    last_delta,
                    last_destination,
                });
                Ok(Top11Result::Awaiting {
                    label: Top11Action::Duplicate.label(),
                    prompt: "pick a destination point (Enter to finish)",
                })
            }
            Top11Pending::Duplicate {
                ids,
                base: Some(base),
                copies,
                vertical,
                from_last,
                use_last_distance,
                use_last_direction,
                last_delta,
                last_destination,
            } => {
                let start = if from_last {
                    last_destination.unwrap_or(base)
                } else {
                    base
                };
                let mut delta = vector(start, point);
                if vertical {
                    let plane = self.input_plane();
                    let z = unit(cross(plane.x, plane.y))?;
                    delta = scale(z, dot(delta, z));
                }
                if use_last_direction
                    && use_last_distance
                    && let Some(last) = last_delta
                {
                    delta = last;
                } else {
                    if use_last_direction && let Some(last) = last_delta {
                        delta = scale(unit(last)?, norm(delta));
                    }
                    if use_last_distance && let Some(last) = last_delta {
                        delta = scale(unit(delta)?, norm(last));
                    }
                }
                if norm(delta) <= 1e-12 {
                    self.top11_pending = Some(Top11Pending::Duplicate {
                        ids,
                        base: Some(base),
                        copies,
                        vertical,
                        from_last,
                        use_last_distance,
                        use_last_direction,
                        last_delta,
                        last_destination,
                    });
                    return Err(Error::Invalid(
                        "duplicate destination equals the base point",
                    ));
                }
                let placement_delta = add(vector(base, start), delta);
                let clones = top11::duplicate_entities_offset(
                    self.document(),
                    &ids,
                    placement_delta,
                    self.coordinator.limits(),
                )?;
                let mutations = clones
                    .into_iter()
                    .map(|entity| Mutation::Insert {
                        document: self.document,
                        entity,
                    })
                    .collect();
                self.commit_top11(Top11Action::Duplicate, mutations, None)?;
                self.top11_pending = Some(Top11Pending::Duplicate {
                    ids,
                    base: Some(base),
                    copies: copies + 1,
                    vertical,
                    from_last,
                    use_last_distance,
                    use_last_direction,
                    last_delta: Some(delta),
                    last_destination: Some(add(start, delta)),
                });
                Ok(Top11Result::Awaiting {
                    label: Top11Action::Duplicate.label(),
                    prompt: "pick another destination point (Enter to finish)",
                })
            }
            Top11Pending::Split { target, cutters } => {
                self.top11_pending = Some(Top11Pending::Split { target, cutters });
                Err(Error::Invalid("split expects Enter to execute"))
            }
            Top11Pending::SplitMany { targets, cutters } => {
                self.top11_pending = Some(Top11Pending::SplitMany { targets, cutters });
                Err(Error::Invalid("select a cutting object"))
            }
            Top11Pending::Trim {
                target,
                cutters,
                fractions: None,
            } => {
                self.top11_pending = Some(Top11Pending::Trim {
                    target,
                    cutters,
                    fractions: None,
                });
                Err(Error::Invalid("trim expects Enter to continue"))
            }
            Top11Pending::Trim {
                target,
                cutters,
                fractions: Some(fractions),
            } => {
                let pieces = match top11::trim_by_fractions(
                    &match &self.document().entities[&target].geometry {
                        Geometry::Curve(curve) => curve.clone(),
                        _ => return Err(Error::Missing("trim curve")),
                    },
                    &fractions,
                    point,
                    self.coordinator.limits(),
                ) {
                    Ok(pieces) => pieces,
                    Err(error) => {
                        self.top11_pending = Some(Top11Pending::Trim {
                            target,
                            cutters,
                            fractions: Some(fractions),
                        });
                        return Err(error);
                    }
                };
                self.commit_cut(Top11Action::Trim, target, pieces)?;
                self.top11_pending = None;
                Ok(Top11Result::Completed {
                    label: Top11Action::Trim.label(),
                    objects: self.selection.len(),
                })
            }
            Top11Pending::Move {
                ids,
                base: None,
                vertical,
            } => {
                self.top11_pending = Some(Top11Pending::Move {
                    ids,
                    base: Some(point),
                    vertical,
                });
                Ok(Top11Result::Awaiting {
                    label: Top11Action::Move.label(),
                    prompt: "pick the destination point",
                })
            }
            Top11Pending::Move {
                ids,
                base: Some(base),
                vertical,
            } => {
                let mut delta = vector(base, point);
                if vertical {
                    let plane = self.input_plane();
                    let normal = unit(cross(plane.x, plane.y))?;
                    delta = scale(normal, dot(delta, normal));
                }
                let moves = top11::translate_placements(self.document(), &ids, delta)?;
                let mutations = self.placement_mutations(moves)?;
                self.commit_top11(Top11Action::Move, mutations, None)?;
                Ok(Top11Result::Completed {
                    label: Top11Action::Move.label(),
                    objects: ids.len(),
                })
            }
            Top11Pending::Rotate {
                ids,
                center: None,
                reference,
                copy,
            } => {
                self.top11_pending = Some(Top11Pending::Rotate {
                    ids,
                    center: Some(point),
                    reference,
                    copy,
                });
                Ok(Top11Result::Awaiting {
                    label: Top11Action::Rotate.label(),
                    prompt: "pick the reference point",
                })
            }
            Top11Pending::Rotate {
                ids,
                center: Some(center),
                reference: None,
                copy,
            } => {
                self.top11_pending = Some(Top11Pending::Rotate {
                    ids,
                    center: Some(center),
                    reference: Some(point),
                    copy,
                });
                Ok(Top11Result::Awaiting {
                    label: Top11Action::Rotate.label(),
                    prompt: "pick the second reference point",
                })
            }
            Top11Pending::Rotate {
                ids,
                center: Some(center),
                reference: Some(reference),
                copy,
            } => {
                let axis = {
                    let plane = self.input_plane();
                    unit(cross(plane.x, plane.y))?
                };
                let degrees =
                    signed_degrees(axis, vector(center, reference), vector(center, point))?;
                self.commit_top11_rotation(&ids, center, axis, degrees, copy)?;
                Ok(Top11Result::Completed {
                    label: Top11Action::Rotate.label(),
                    objects: ids.len(),
                })
            }
            Top11Pending::Mirror {
                ids,
                mut points,
                copy,
                three_point,
            } => {
                points.push(point);
                let needed = if three_point { 3 } else { 2 };
                if points.len() < needed {
                    self.top11_pending = Some(Top11Pending::Mirror {
                        ids,
                        points,
                        copy,
                        three_point,
                    });
                    return Ok(Top11Result::Awaiting {
                        label: Top11Action::Mirror.label(),
                        prompt: if three_point {
                            "pick the third point on the mirror plane"
                        } else {
                            "pick the second point on the mirror plane"
                        },
                    });
                }
                let (origin, normal) = if three_point {
                    let first = points[0];
                    let normal = unit(cross(vector(first, points[1]), vector(first, points[2])))?;
                    (first, normal)
                } else {
                    let first = points[0];
                    let plane = self.input_plane();
                    let normal = unit(cross(vector(first, points[1]), cross(plane.x, plane.y)))?;
                    (first, normal)
                };
                let (copies, replacements) = self.mirror_entities(&ids, origin, normal, copy)?;
                let mut mutations = Vec::with_capacity(replacements.len() + copies.len());
                for (id, geometry) in replacements {
                    let source = self.document().stamp(id)?;
                    mutations.push(Mutation::Geometry { source, geometry });
                }
                let ids_new: Vec<ObjectId> = copies.iter().map(|entity| entity.id).collect();
                mutations.extend(copies.into_iter().map(|entity| Mutation::Insert {
                    document: self.document,
                    entity,
                }));
                let selection = if copy {
                    Some(ids_new)
                } else {
                    Some(ids.iter().copied().collect())
                };
                self.commit_top11(Top11Action::Mirror, mutations, selection)?;
                Ok(Top11Result::Completed {
                    label: Top11Action::Mirror.label(),
                    objects: ids.len(),
                })
            }
        }
    }

    /// Feed one picked object to a pending Split/Trim cutting-curve prompt. Cutters
    /// accumulate until Enter; each pick is validated before it is kept.
    pub fn top11_accept_object(&mut self, object: ObjectId) -> Result<Top11Result> {
        let pending = self
            .top11_pending
            .take()
            .ok_or(Error::Missing("top 11 object command"))?;
        match pending {
            Top11Pending::SelectObjects { action } => {
                let mut ids: Vec<ObjectId> = self.selection.iter().copied().collect();
                if !ids.contains(&object) {
                    ids.push(object);
                }
                let result = self.select_objects(&ids);
                self.top11_pending = Some(Top11Pending::SelectObjects { action });
                result?;
                Ok(Top11Result::Awaiting {
                    label: action.label(),
                    prompt: "select objects (Enter to continue)",
                })
            }
            Top11Pending::Split {
                target,
                mut cutters,
            } => {
                if object == target {
                    self.top11_pending = Some(Top11Pending::Split { target, cutters });
                    return Err(Error::Invalid("cutting curve is the target"));
                }
                if cutters.contains(&object) {
                    self.top11_pending = Some(Top11Pending::Split { target, cutters });
                    return Err(Error::Invalid("cutting curve already picked"));
                }
                if let Err(error) = self.cutter_fractions(target, object) {
                    self.top11_pending = Some(Top11Pending::Split { target, cutters });
                    return Err(error);
                }
                cutters.push(object);
                self.top11_pending = Some(Top11Pending::Split { target, cutters });
                Ok(Top11Result::Awaiting {
                    label: Top11Action::Split.label(),
                    prompt: "pick more cutting curves (Enter to execute)",
                })
            }
            Top11Pending::SplitMany {
                targets,
                mut cutters,
            } => {
                let validation = (|| {
                    if targets.contains(&object) {
                        return Err(Error::Invalid("cutting curve is a target"));
                    }
                    if cutters.contains(&object) {
                        return Err(Error::Invalid("cutting curve already picked"));
                    }
                    let mut intersects = false;
                    for target in &targets {
                        match self.cutter_fractions(*target, object) {
                            Ok(_) => intersects = true,
                            Err(Error::Invalid("cutting object has no intersection")) => {}
                            Err(error) => return Err(error),
                        }
                    }
                    if !intersects {
                        return Err(Error::Invalid("cutting object has no intersection"));
                    }
                    Ok(())
                })();
                if validation.is_ok() {
                    cutters.push(object);
                }
                self.top11_pending = Some(Top11Pending::SplitMany { targets, cutters });
                validation?;
                Ok(Top11Result::Awaiting {
                    label: Top11Action::Split.label(),
                    prompt: "pick more cutting curves (Enter to execute)",
                })
            }
            Top11Pending::Trim {
                target,
                mut cutters,
                fractions: None,
            } => {
                if object == target {
                    self.top11_pending = Some(Top11Pending::Trim {
                        target,
                        cutters,
                        fractions: None,
                    });
                    return Err(Error::Invalid("cutting curve is the target"));
                }
                if cutters.contains(&object) {
                    self.top11_pending = Some(Top11Pending::Trim {
                        target,
                        cutters,
                        fractions: None,
                    });
                    return Err(Error::Invalid("cutting curve already picked"));
                }
                if let Err(error) = self.cutter_fractions(target, object) {
                    self.top11_pending = Some(Top11Pending::Trim {
                        target,
                        cutters,
                        fractions: None,
                    });
                    return Err(error);
                }
                cutters.push(object);
                self.top11_pending = Some(Top11Pending::Trim {
                    target,
                    cutters,
                    fractions: None,
                });
                Ok(Top11Result::Awaiting {
                    label: Top11Action::Trim.label(),
                    prompt: "pick more cutting curves (Enter to continue)",
                })
            }
            other => {
                self.top11_pending = Some(other);
                Err(Error::Invalid("top 11 command does not take an object"))
            }
        }
    }

    /// Remove a previously picked object from a selection or cutting-object stage.
    pub fn top11_remove_object(&mut self, object: ObjectId) -> Result<Top11Result> {
        let pending = self
            .top11_pending
            .clone()
            .ok_or(Error::Missing("top 11 object command"))?;
        let action = pending.action();
        match pending {
            Top11Pending::SelectObjects { .. } => {
                if !self.selection.contains(&object) {
                    return Err(Error::Missing("selected object"));
                }
                let ids: Vec<_> = self
                    .selection
                    .iter()
                    .copied()
                    .filter(|id| *id != object)
                    .collect();
                self.select_objects(&ids)?;
            }
            Top11Pending::Split {
                target,
                mut cutters,
            } => {
                let Some(index) = cutters.iter().position(|id| *id == object) else {
                    return Err(Error::Missing("cutting object"));
                };
                cutters.remove(index);
                self.top11_pending = Some(Top11Pending::Split { target, cutters });
            }
            Top11Pending::SplitMany {
                targets,
                mut cutters,
            } => {
                let Some(index) = cutters.iter().position(|id| *id == object) else {
                    return Err(Error::Missing("cutting object"));
                };
                cutters.remove(index);
                self.top11_pending = Some(Top11Pending::SplitMany { targets, cutters });
            }
            Top11Pending::Trim {
                target,
                mut cutters,
                fractions: None,
            } => {
                let Some(index) = cutters.iter().position(|id| *id == object) else {
                    return Err(Error::Missing("cutting object"));
                };
                cutters.remove(index);
                self.top11_pending = Some(Top11Pending::Trim {
                    target,
                    cutters,
                    fractions: None,
                });
            }
            _ => return Err(Error::Invalid("top 11 command does not select objects")),
        }
        Ok(Top11Result::Awaiting {
            label: action.label(),
            prompt: self.top11_prompt().unwrap_or("select objects"),
        })
    }

    /// Interior fractions produced by one cutting object: a curve, a NURBS surface or a
    /// B-Rep display snapshot. The target and cutter must share a placement.
    fn cutter_fractions(&self, target: ObjectId, cutter: ObjectId) -> Result<Vec<f64>> {
        let target_entity = self
            .document()
            .entities
            .get(&target)
            .ok_or(Error::Missing("target curve"))?;
        let cutter_entity = self
            .document()
            .entities
            .get(&cutter)
            .ok_or(Error::Missing("cutting object"))?;
        if target_entity.placement != cutter_entity.placement {
            return Err(Error::Unsupported(
                "split/trim needs the target and cutter in the same placement",
            ));
        }
        let Geometry::Curve(target_curve) = &target_entity.geometry else {
            return Err(Error::Missing("target curve"));
        };
        let tolerance = 1e-6;
        let mut fractions: Vec<f64> = match &cutter_entity.geometry {
            Geometry::Curve(cutter_curve) => {
                om9_core::curve_surface_ops::curve_curve_intersections(
                    target_curve,
                    cutter_curve,
                    tolerance,
                    self.coordinator.limits(),
                )?
                .into_iter()
                .map(|hit| hit.first_fraction)
                .collect()
            }
            Geometry::Surface(surface) => om9_core::curve_surface_ops::curve_surface_intersections(
                target_curve,
                surface,
                tolerance,
                self.coordinator.limits(),
            )?
            .into_iter()
            .map(|hit| hit.curve_fraction)
            .collect(),
            Geometry::Brep(brep) => {
                #[cfg(feature = "occt-brep")]
                {
                    om9_kernel::intersect::curve_brep_fractions(
                        brep,
                        target_curve,
                        tolerance,
                        self.coordinator.limits(),
                    )
                    .map_err(|error| Error::Io(format!("BRep cutter: {error}")))?
                }
                #[cfg(not(feature = "occt-brep"))]
                {
                    let _ = (brep, target_curve);
                    return Err(Error::Unsupported(
                        "BRep cutter needs the OCCT kernel feature",
                    ));
                }
            }
            _ => {
                return Err(Error::Unsupported(
                    "cutting object must be a curve, surface or B-Rep",
                ));
            }
        };
        fractions.retain(|fraction| *fraction > 1e-7 && *fraction < 1.0 - 1e-7);
        fractions.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        fractions.dedup_by(|a, b| (*a - *b).abs() <= 1e-6);
        if fractions.is_empty() {
            return Err(Error::Invalid("cutting object has no intersection"));
        }
        Ok(fractions)
    }

    fn cutter_fractions_all(&self, target: ObjectId, cutters: &[ObjectId]) -> Result<Vec<f64>> {
        let mut fractions = Vec::new();
        for cutter in cutters {
            fractions.extend(self.cutter_fractions(target, *cutter)?);
        }
        fractions.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        fractions.dedup_by(|a, b| (*a - *b).abs() <= 1e-6);
        Ok(fractions)
    }

    fn commit_cut(
        &mut self,
        action: Top11Action,
        target: ObjectId,
        pieces: Vec<Curve>,
    ) -> Result<()> {
        if pieces.is_empty() {
            return Err(Error::Invalid("cut produced no curve"));
        }
        if pieces.len() == 1 {
            let geometry = Geometry::Curve(pieces.into_iter().next().expect("one piece"));
            let source = self.document().stamp(target)?;
            return self.commit_top11(
                action,
                vec![Mutation::Geometry { source, geometry }],
                Some(vec![target]),
            );
        }
        let source_entity = self
            .document()
            .entities
            .get(&target)
            .ok_or(Error::Missing("target curve"))?
            .clone();
        let mut mutations = vec![Mutation::Delete {
            source: self.document().stamp(target)?,
        }];
        let mut ids = Vec::with_capacity(pieces.len());
        for piece in pieces {
            let entity = child_entity(&source_entity, Geometry::Curve(piece));
            ids.push(entity.id);
            mutations.push(Mutation::Insert {
                document: self.document,
                entity,
            });
        }
        self.commit_top11(action, mutations, Some(ids))
    }

    /// Mirror selected entities, dispatching BRep bodies to the OCCT adapter.
    fn mirror_entities(
        &self,
        ids: &BTreeSet<ObjectId>,
        origin: Point3,
        normal: Point3,
        copy: bool,
    ) -> Result<MirrorBatch> {
        let mut brep_ids = BTreeSet::new();
        let mut other_ids = BTreeSet::new();
        for id in ids {
            let entity = self
                .document()
                .entities
                .get(id)
                .ok_or(Error::Missing("object"))?;
            if matches!(entity.geometry, Geometry::Brep(_)) {
                brep_ids.insert(*id);
            } else {
                other_ids.insert(*id);
            }
        }
        let outcome = top11::mirror_entities(
            self.document(),
            &other_ids,
            origin,
            normal,
            copy,
            self.coordinator.limits(),
        )?;
        #[cfg_attr(not(feature = "occt-brep"), allow(unused_mut))]
        let mut copies = outcome.copies;
        #[cfg_attr(not(feature = "occt-brep"), allow(unused_mut))]
        let mut replacements = outcome.replacements;
        for id in brep_ids {
            let entity = self
                .document()
                .entities
                .get(&id)
                .ok_or(Error::Missing("BRep"))?;
            let Geometry::Brep(brep) = &entity.geometry else {
                continue;
            };
            #[cfg(feature = "occt-brep")]
            {
                let matrix = om9_kernel::transform_ops::mirror(origin, normal)
                    .map_err(|error| Error::Io(format!("BRep mirror: {error}")))?;
                let world = if entity.placement == Transform::identity() {
                    brep.clone()
                } else {
                    om9_kernel::transform_ops::transform_brep(
                        brep,
                        &entity.placement,
                        brep.display.deflection_mm,
                        self.coordinator.limits(),
                    )
                    .map_err(|error| Error::Io(format!("BRep mirror: {error}")))?
                };
                let reflected = om9_kernel::transform_ops::transform_brep(
                    &world,
                    &matrix,
                    world.display.deflection_mm,
                    self.coordinator.limits(),
                )
                .map_err(|error| Error::Io(format!("BRep mirror: {error}")))?;
                if copy {
                    copies.push(child_entity(entity, Geometry::Brep(reflected)));
                } else {
                    // The edit keeps the object placement: bring the reflected world
                    // body back into the object's local frame.
                    let local = if entity.placement == Transform::identity() {
                        reflected
                    } else {
                        let inverse = om9_kernel::transform_ops::inverse(&entity.placement)
                            .map_err(|error| Error::Io(format!("BRep mirror: {error}")))?;
                        om9_kernel::transform_ops::transform_brep(
                            &reflected,
                            &inverse,
                            reflected.display.deflection_mm,
                            self.coordinator.limits(),
                        )
                        .map_err(|error| Error::Io(format!("BRep mirror: {error}")))?
                    };
                    replacements.push((id, Geometry::Brep(local)));
                }
            }
            #[cfg(not(feature = "occt-brep"))]
            {
                let _ = (entity, brep, copy);
                return Err(Error::Unsupported(
                    "BRep mirror needs the OCCT kernel feature",
                ));
            }
        }
        Ok((copies, replacements))
    }

    #[cfg(feature = "occt-brep")]
    fn join_many_breps(&self, ids: &[ObjectId]) -> Result<Entity> {
        let limits = self.coordinator.limits();
        let tolerance = self.document().effective_tolerance()?.absolute_mm;
        let first = self
            .document()
            .entities
            .get(&ids[0])
            .ok_or(Error::Missing("BRep"))?;
        let mut parts = Vec::new();
        parts
            .try_reserve_exact(ids.len())
            .map_err(|_| Error::Budget("BRep join inputs"))?;
        for id in ids {
            let entity = self
                .document()
                .entities
                .get(id)
                .ok_or(Error::Missing("BRep"))?;
            let Geometry::Brep(brep) = &entity.geometry else {
                return Err(Error::Invalid("BRep join selection"));
            };
            let world = if entity.placement == Transform::identity() {
                brep.clone()
            } else {
                om9_kernel::transform_ops::transform_brep(
                    brep,
                    &entity.placement,
                    brep.display.deflection_mm,
                    limits,
                )
                .map_err(|e| Error::Io(format!("BRep join: {e}")))?
            };
            let measure = om9_kernel::creation::measure(&world, limits)
                .map_err(|e| Error::Io(format!("BRep join: {e}")))?;
            parts.push((world, measure));
        }
        let (mut current, mut current_measure) = parts.remove(0);
        let mut attempts = 0u64;
        while !parts.is_empty() {
            let mut joined = None;
            for (index, (part, measure)) in parts.iter().enumerate() {
                attempts = attempts
                    .checked_add(1)
                    .ok_or(Error::Overflow("BRep join attempts"))?;
                if attempts > limits.max_outputs {
                    return Err(Error::Budget("BRep join attempts"));
                }
                let candidate = om9_kernel::editing::sew(
                    &current,
                    part,
                    tolerance,
                    current
                        .display
                        .deflection_mm
                        .min(part.display.deflection_mm),
                    limits,
                )
                .map_err(|e| Error::Io(format!("BRep join: {e}")))?;
                let after = om9_kernel::creation::measure(&candidate, limits)
                    .map_err(|e| Error::Io(format!("BRep join: {e}")))?;
                let faces = current_measure
                    .faces
                    .checked_add(measure.faces)
                    .ok_or(Error::Overflow("BRep join faces"))?;
                let naked = current_measure
                    .naked_edges
                    .checked_add(measure.naked_edges)
                    .ok_or(Error::Overflow("BRep join edges"))?;
                if after.faces == faces && after.shells == 1 && after.naked_edges < naked {
                    joined = Some((index, candidate, after));
                    break;
                }
            }
            let Some((index, candidate, measure)) = joined else {
                return Err(Error::Invalid("BRep faces do not form one connected shell"));
            };
            parts.remove(index);
            current = candidate;
            current_measure = measure;
        }
        let local = if first.placement == Transform::identity() {
            current
        } else {
            let inverse = om9_kernel::transform_ops::inverse(&first.placement)
                .map_err(|e| Error::Io(format!("BRep join: {e}")))?;
            om9_kernel::transform_ops::transform_brep(
                &current,
                &inverse,
                current.display.deflection_mm,
                limits,
            )
            .map_err(|e| Error::Io(format!("BRep join: {e}")))?
        };
        Ok(child_entity(first, Geometry::Brep(local)))
    }

    fn placement_mutations(&self, placements: Vec<(ObjectId, Transform)>) -> Result<Vec<Mutation>> {
        placements
            .into_iter()
            .map(|(id, placement)| {
                Ok(Mutation::Placement {
                    source: self.document().stamp(id)?,
                    placement,
                })
            })
            .collect()
    }

    fn commit_top11_rotation(
        &mut self,
        ids: &BTreeSet<ObjectId>,
        center: Point3,
        axis: Point3,
        degrees: f64,
        copy: bool,
    ) -> Result<()> {
        let placements = top11::rotate_placements(self.document(), ids, center, axis, degrees)?;
        if !copy {
            let mutations = self.placement_mutations(placements)?;
            return self.commit_top11(Top11Action::Rotate, mutations, None);
        }
        let mut mutations = Vec::with_capacity(placements.len());
        let mut selected = Vec::with_capacity(placements.len());
        for (id, placement) in placements {
            let source = self
                .document()
                .entities
                .get(&id)
                .ok_or(Error::Missing("object"))?;
            let mut entity = child_entity(source, source.geometry.clone());
            entity.placement = placement;
            selected.push(entity.id);
            mutations.push(Mutation::Insert {
                document: self.document,
                entity,
            });
        }
        self.commit_top11(Top11Action::Rotate, mutations, Some(selected))
    }

    fn commit_top11_mirror_axis(
        &mut self,
        ids: &BTreeSet<ObjectId>,
        origin: Point3,
        normal: Point3,
        copy: bool,
    ) -> Result<()> {
        let (copies, replacements) = self.mirror_entities(ids, origin, normal, copy)?;
        let mut mutations = Vec::with_capacity(replacements.len() + copies.len());
        for (id, geometry) in replacements {
            mutations.push(Mutation::Geometry {
                source: self.document().stamp(id)?,
                geometry,
            });
        }
        let new_ids: Vec<ObjectId> = copies.iter().map(|entity| entity.id).collect();
        mutations.extend(copies.into_iter().map(|entity| Mutation::Insert {
            document: self.document,
            entity,
        }));
        let selection = if copy {
            new_ids
        } else {
            ids.iter().copied().collect()
        };
        self.commit_top11(Top11Action::Mirror, mutations, Some(selection))
    }

    fn commit_top11(
        &mut self,
        action: Top11Action,
        mutations: Vec<Mutation>,
        selection: Option<Vec<ObjectId>>,
    ) -> Result<()> {
        let held = self.auxiliary_usage()?;
        let prepared = self
            .coordinator
            .prepare_with_held_usage(TransactionRequest::new(action.command(), mutations), held)?;
        let receipt = self.coordinator.commit(prepared)?;
        if receipt.published {
            self.refresh_scene()?;
            self.sync_published(om9_core::changes::ChangeCause::Edit);
            if let Some(ids) = selection {
                self.select_objects(&ids)?;
            }
            self.events.emit(
                crate::command_events::EventKind::Finished,
                action.command(),
                "committed",
                Some(crate::command_events::EventOutcome::Committed),
            );
        }
        Ok(())
    }

    /// Parse a Matrix command name or `top11 <action> [values]`.
    pub fn top11_submit(&mut self, text: &str) -> Option<Result<Top11Result>> {
        let text = text.trim();
        if text.is_empty() {
            return None;
        }
        if matches!(text.to_ascii_lowercase().as_str(), "point" | "p")
            && matches!(self.top11_pending, Some(Top11Pending::Split { .. }))
        {
            return Some(self.top11_split_point_mode());
        }
        if matches!(
            text.to_ascii_lowercase().as_str(),
            "inplace" | "in place" | "in-place"
        ) && let Some(Top11Pending::Duplicate {
            ids, base: None, ..
        }) = self.top11_pending.clone()
        {
            let pending = self.top11_pending.take();
            let result = self.duplicate_in_place_ids(&ids);
            if result.is_err() {
                self.top11_pending = pending;
            }
            return Some(result);
        }
        if let Some(Top11Pending::Rotate { copy, .. }) = self.top11_pending.as_mut() {
            let lower = text.to_ascii_lowercase();
            if lower == "copy" || lower.starts_with("copy=") {
                let next = match lower.strip_prefix("copy=") {
                    None => !*copy,
                    Some("yes" | "y" | "on" | "1" | "true") => true,
                    Some("no" | "n" | "off" | "0" | "false") => false,
                    Some(_) => return Some(Err(Error::Invalid("rotate copy option"))),
                };
                *copy = next;
                return Some(Ok(Top11Result::Awaiting {
                    label: Top11Action::Rotate.label(),
                    prompt: "pick a point or enter an angle",
                }));
            }
        }
        if let Some(Top11Pending::Rotate {
            ids,
            center: Some(center),
            reference: None,
            copy,
        }) = self.top11_pending.clone()
            && let Ok(degrees) = text.parse::<f64>()
        {
            if !degrees.is_finite() {
                return Some(Err(Error::Invalid("rotation angle")));
            }
            let plane = self.input_plane();
            let axis = match unit(cross(plane.x, plane.y)) {
                Ok(axis) => axis,
                Err(error) => return Some(Err(error)),
            };
            let pending = self.top11_pending.take();
            let result = self.commit_top11_rotation(&ids, center, axis, degrees, copy);
            if result.is_err() {
                self.top11_pending = pending;
            }
            return Some(result.map(|_| Top11Result::Completed {
                label: Top11Action::Rotate.label(),
                objects: ids.len(),
            }));
        }
        if let Some(Top11Pending::Move { vertical, .. }) = self.top11_pending.as_mut()
            && let Some((key, value)) = duplicate_option(text)
            && key.eq_ignore_ascii_case("vertical")
        {
            *vertical = value.unwrap_or(!*vertical);
            return Some(Ok(Top11Result::Awaiting {
                label: Top11Action::Move.label(),
                prompt: "pick a point",
            }));
        }
        if matches!(self.top11_pending, Some(Top11Pending::Duplicate { .. }))
            && let Some((key, value)) = duplicate_option(text)
        {
            return Some(self.top11_set_duplicate_option(key, value).map(|_| {
                Top11Result::Awaiting {
                    label: Top11Action::Duplicate.label(),
                    prompt: "pick a destination point (Enter to finish)",
                }
            }));
        }
        if matches!(self.top11_pending, Some(Top11Pending::Mirror { .. })) {
            let axis = match text.to_ascii_lowercase().as_str() {
                "xaxis" | "x axis" => Some(true),
                "yaxis" | "y axis" => Some(false),
                _ => None,
            };
            if let Some(x_axis) = axis {
                let Some(Top11Pending::Mirror { ids, copy, .. }) = self.top11_pending.clone()
                else {
                    unreachable!()
                };
                let plane = self.input_plane();
                let normal = if x_axis { plane.y } else { plane.x };
                let pending = self.top11_pending.take();
                let result = self.commit_top11_mirror_axis(&ids, plane.origin, normal, copy);
                if result.is_err() {
                    self.top11_pending = pending;
                }
                return Some(result.map(|_| Top11Result::Completed {
                    label: Top11Action::Mirror.label(),
                    objects: ids.len(),
                }));
            }
            if let Some(value) = text
                .strip_prefix("Copy=")
                .or_else(|| text.strip_prefix("copy="))
            {
                let copy = match value.to_ascii_lowercase().as_str() {
                    "yes" | "y" | "on" | "1" | "true" => true,
                    "no" | "n" | "off" | "0" | "false" => false,
                    _ => return Some(Err(Error::Invalid("mirror copy option"))),
                };
                return Some(self.top11_set_mirror_copy(copy).then_some(()).ok_or(
                    Error::Invalid("mirror copy option"),
                ).map(|_| Top11Result::Awaiting {
                    label: Top11Action::Mirror.label(),
                    prompt: "pick two points on the mirror plane (type 3Point for any plane)",
                }));
            }
            let three = matches!(
                text.to_ascii_lowercase().as_str(),
                "3point" | "3-point" | "plane3" | "three"
            );
            let two = matches!(
                text.to_ascii_lowercase().as_str(),
                "2point" | "2-point" | "plane2" | "vertical"
            );
            if three || two {
                return Some(self.top11_mirror_use_three_point(three).map(|_| {
                    Top11Result::Awaiting {
                        label: Top11Action::Mirror.label(),
                        prompt: if three {
                            "pick three points on the mirror plane"
                        } else {
                            "pick two points on the mirror plane"
                        },
                    }
                }));
            }
        }
        let mut words = text.split_whitespace();
        let first = words.next()?;
        let rest: Vec<&str> = words.collect();
        let top11_prefix = first.eq_ignore_ascii_case("top11");
        let command = if top11_prefix {
            rest.first().copied()?
        } else {
            first
        };
        let action = match command.to_ascii_lowercase().as_str() {
            "copy" | "duplicate" => Top11Action::Duplicate,
            "mirror" => Top11Action::Mirror,
            "move" => Top11Action::Move,
            "rotate" => Top11Action::Rotate,
            "editpton" => Top11Action::EditPoints,
            "pointson" | "controlpointson" => Top11Action::ControlPoints,
            "explode" => Top11Action::Explode,
            "join" => Top11Action::Join,
            "split" => Top11Action::Split,
            "trim" => Top11Action::Trim,
            "gvringrail" | "ringrail" => Top11Action::RingRail,
            _ => return None,
        };
        if !top11_prefix {
            // A bare command name is the Matrix spelling; every other text remains a
            // point or an unrelated command.
            if !action.command().eq_ignore_ascii_case(first)
                && !action.label().eq_ignore_ascii_case(first)
            {
                return None;
            }
        }
        if !top11_prefix
            && matches!(action, Top11Action::Move | Top11Action::Rotate)
            && !rest.is_empty()
            && !(action == Top11Action::Rotate
                && rest
                    .iter()
                    .all(|word| word.to_ascii_lowercase().starts_with("copy=")))
        {
            // `Move 1,2,3` / `Rotate 90` belong to the gumball numeric grammar, not to the
            // bare top11 action. Defer so a malformed numeric form is refused instead of
            // silently invoking the command.
            return None;
        }
        let option_words: Vec<String> = if top11_prefix {
            rest.iter()
                .skip(1)
                .map(|word| word.to_ascii_lowercase())
                .collect()
        } else {
            rest.iter().map(|word| word.to_ascii_lowercase()).collect()
        };
        let values: Vec<f64> = option_words
            .iter()
            .filter_map(|value| value.parse().ok())
            .collect();
        Some(match action {
            Top11Action::Duplicate
                if !option_words
                    .iter()
                    .any(|word| word == "inplace" || word == "in-place") =>
            {
                let result = match self.top11_invoke(action) {
                    Ok(result) => result,
                    Err(error) => return Some(Err(error)),
                };
                for word in &option_words {
                    if let Some((key, value)) = duplicate_option(word)
                        && let Err(error) = self.top11_set_duplicate_option(key, value)
                    {
                        return Some(Err(error));
                    }
                }
                Ok(result)
            }
            Top11Action::Duplicate => self.top11_duplicate_in_place(),
            Top11Action::Mirror => {
                if self.selection.is_empty() && option_words.is_empty() {
                    return Some(self.top11_invoke(action));
                }
                let copy = !option_words
                    .iter()
                    .any(|word| word == "delete" || word == "deleteinput" || word == "nocopy");
                let three_point = option_words.iter().any(|word| {
                    word == "3point" || word == "three" || word == "plane3" || word == "3-point"
                });
                self.top11_mirror_mode(copy, three_point)
            }
            Top11Action::Rotate if !option_words.is_empty() => {
                if option_words.len() != 1 {
                    return Some(Err(Error::Invalid("rotate options")));
                }
                let value = match option_words[0].strip_prefix("copy=") {
                    Some("yes" | "y" | "on" | "1" | "true") => true,
                    Some("no" | "n" | "off" | "0" | "false") => false,
                    _ => return Some(Err(Error::Invalid("rotate copy option"))),
                };
                let result = match self.top11_invoke(action) {
                    Ok(result) => result,
                    Err(error) => return Some(Err(error)),
                };
                if let Some(Top11Pending::Rotate { copy, .. }) = self.top11_pending.as_mut() {
                    *copy = value;
                }
                Ok(result)
            }
            Top11Action::Split if !values.is_empty() => self.top11_split(values[0]),
            Top11Action::Split
                if option_words.len() == 1 && matches!(option_words[0].as_str(), "point" | "p") =>
            {
                if self.selection.is_empty() {
                    return Some(Err(Error::Missing("select a line before Split Point")));
                }
                if let Err(error) = self.top11_invoke(action) {
                    return Some(Err(error));
                }
                self.top11_split_point_mode()
            }
            Top11Action::Trim if values.len() >= 2 => self.top11_trim(values[0], values[1]),
            _ => self.top11_invoke(action),
        })
    }

    fn top11_split(&mut self, fraction: f64) -> Result<Top11Result> {
        if self.is_running() {
            return Err(Error::Busy);
        }
        let id = self
            .selection
            .iter()
            .next()
            .copied()
            .ok_or(Error::Missing("split curve"))?;
        let halves = top11::split_entity(self.document(), id, fraction, self.coordinator.limits())?;
        let ids: Vec<ObjectId> = halves.iter().map(|entity| entity.id).collect();
        let mut mutations = vec![Mutation::Delete {
            source: self.document().stamp(id)?,
        }];
        mutations.extend(halves.into_iter().map(|entity| Mutation::Insert {
            document: self.document,
            entity,
        }));
        self.commit_top11(Top11Action::Split, mutations, Some(ids))?;
        Ok(Top11Result::Completed {
            label: Top11Action::Split.label(),
            objects: 2,
        })
    }

    fn top11_trim(&mut self, from: f64, to: f64) -> Result<Top11Result> {
        if self.is_running() {
            return Err(Error::Busy);
        }
        let id = self
            .selection
            .iter()
            .next()
            .copied()
            .ok_or(Error::Missing("trim curve"))?;
        let geometry =
            top11::trim_entity(self.document(), id, from, to, self.coordinator.limits())?;
        let source = self.document().stamp(id)?;
        self.commit_top11(
            Top11Action::Trim,
            vec![Mutation::Geometry { source, geometry }],
            Some(vec![id]),
        )?;
        Ok(Top11Result::Completed {
            label: Top11Action::Trim.label(),
            objects: 1,
        })
    }

    pub fn clear_top11(&mut self) {
        self.top11_pending = None;
        self.top11_grip = None;
    }
}
