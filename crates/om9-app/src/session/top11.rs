// SPDX-License-Identifier: LGPL-2.1-or-later
//! Matrix Top 11 command adapters (OM9-L2-03-08).
//!
//! The toolbar, its command names and the `top11` command line share one execution
//! path. Move/Rotate/Mirror collect their points through the same `accept_world_point`
//! entry the viewport uses, then commit one transaction; the other actions are
//! immediate. Ring Rail dispatches to the M04 builder.
use super::*;
use om9_core::{
    geometry::{Point3, Transform},
    identity::ObjectId,
    top11::{self, GripMode, Top11Action, Top11Availability, Top11Context},
};
use std::collections::BTreeSet;

/// Mirrored copies plus replacement geometry for the delete-input option.
type MirrorBatch = (Vec<Entity>, Vec<(ObjectId, Geometry)>, Vec<(ObjectId, Transform)>);

/// What a Top 11 toolbar result means for the status line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Top11Result {
    Completed {
        label: &'static str,
        objects: usize,
    },
    NoChange { label: &'static str },
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
            Self::NoChange { label } => format!("{label}: no changes"),
            Self::Awaiting { label, prompt } => format!("{label}: {prompt}"),
            Self::GripMode { label, mode } => match mode {
                Some(GripMode::Edit) => format!("{label}: edit points on"),
                Some(GripMode::Control) => format!("{label}: control points on"),
                None => format!("{label}: points off"),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitIsoDirection {
    U,
    V,
    Both,
}

/// The in-flight point/object-driven Top 11 command.
#[derive(Debug, Clone, PartialEq)]
pub enum Top11Pending {
    SelectObjects {
        action: Top11Action,
        split_point: bool,
        explode_blocks: bool,
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
        isocurve_allowed: bool,
        shrink: Option<bool>,
    },
    SplitPoint {
        target: ObjectId,
        closed: bool,
        fractions: Vec<f64>,
    },
    SplitIso {
        target: ObjectId,
        direction: SplitIsoDirection,
        shrink: bool,
        shrink_available: bool,
    },
    Trim {
        target: ObjectId,
        cutters: Vec<ObjectId>,
        fractions: Option<Vec<f64>>,
    },
    TrimMany {
        targets: Vec<ObjectId>,
        cutters: Vec<ObjectId>,
        fractions: Option<Vec<(ObjectId, Vec<f64>)>>,
        completed: usize,
    },
    TrimMutual {
        objects: Vec<ObjectId>,
        fractions: Vec<(ObjectId, Vec<f64>)>,
        completed: usize,
    },
    TrimFace {
        target: ObjectId,
        partition: Option<om9_core::brep::Brep>,
        merge_regions: bool,
        original_edges: Option<om9_core::brep::Brep>,
        objects: Vec<ObjectId>,
        fractions: Vec<(ObjectId, Vec<f64>)>,
        completed: usize,
    },
    TrimSurfaces {
        objects: Vec<ObjectId>,
        partitions: Vec<(ObjectId, om9_core::brep::Brep)>,
        original_edges: Vec<(ObjectId, om9_core::brep::Brep)>,
        fractions: Vec<(ObjectId, Vec<f64>)>,
        completed: usize,
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
            Self::SelectObjects { action, .. } => *action,
            Self::Duplicate { .. } => Top11Action::Duplicate,
            Self::Split { .. } => Top11Action::Split,
            Self::SplitMany { .. } => Top11Action::Split,
            Self::SplitPoint { .. } => Top11Action::Split,
            Self::SplitIso { .. } => Top11Action::Split,
            Self::Trim { .. } => Top11Action::Trim,
            Self::TrimMany { .. } => Top11Action::Trim,
            Self::TrimMutual { .. } => Top11Action::Trim,
            Self::TrimFace { .. } => Top11Action::Trim,
            Self::TrimSurfaces { .. } => Top11Action::Trim,
            Self::Move { .. } => Top11Action::Move,
            Self::Rotate { .. } => Top11Action::Rotate,
            Self::Mirror { .. } => Top11Action::Mirror,
        }
    }

    pub fn prompt(&self) -> &'static str {
        match self {
            Self::SelectObjects {
                split_point: true, ..
            } => "select one curve (Enter to continue)",
            Self::SelectObjects { .. } => "select objects (Enter to continue)",
            Self::Duplicate { base: None, .. } => "pick the base point",
            Self::Duplicate { .. } => "pick a destination point (Enter to finish)",
            Self::Split { .. } => "pick cutting curves (Enter to execute)",
            Self::SplitMany { .. } => "pick cutting curves (Enter to execute)",
            Self::SplitPoint { closed: true, .. } => {
                "pick points on the closed curve (Enter to split)"
            }
            Self::SplitPoint { .. } => "pick a point on the curve",
            Self::SplitIso { .. } => "pick an isocurve location on the surface",
            Self::Trim {
                fractions: None, ..
            } => "pick cutting curves (Enter to continue)",
            Self::Trim { .. } => "pick the portion to remove",
            Self::TrimMany {
                fractions: None, ..
            } => "pick cutting curves (Enter to continue)",
            Self::TrimMany { .. } => "pick portions to remove (Enter to finish)",
            Self::TrimMutual { .. } => "pick portions to remove (Enter to finish)",
            Self::TrimFace { .. } => "pick portions to remove (Enter to finish)",
            Self::TrimSurfaces { .. } => "pick portions to remove (Enter to finish)",
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
                | Self::TrimMany {
                    fractions: None,
                    ..
                }
        )
    }
    /// Command-line options available for the current Top11 action and stage.
    pub fn option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        if let Self::SplitMany {
            cutters,
            isocurve_allowed,
            shrink,
            ..
        } = self
        {
            let mut options = Vec::new();
            if cutters.is_empty() && *isocurve_allowed {
                options.push(("Isocurve", "Isocurve"));
            }
            if let Some(value) = shrink {
                options.push(("Shrink", if *value { "Shrink=No" } else { "Shrink=Yes" }));
            }
            return options;
        }
        if let Self::SplitIso {
            shrink,
            shrink_available,
            ..
        } = self
        {
            let mut options = vec![
                ("Direction", "Toggle"),
                ("U", "U"),
                ("V", "V"),
                ("Both", "Both"),
                ("Toggle", "Toggle"),
            ];
            if *shrink_available {
                options.push(("Shrink", if *shrink { "Shrink=No" } else { "Shrink=Yes" }));
            }
            return options;
        }
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
            copies,
            vertical,
            from_last,
            use_last_distance,
            use_last_direction,
            ..
        } = self
        else {
            return Vec::new();
        };
        // Matrix documents the first three options at the base stage and the
        // From/Use-Last group only "after the first copy".
        let mut options = vec![(
            "Vertical",
            if *vertical {
                "Vertical=No"
            } else {
                "Vertical=Yes"
            },
        )];
        if *copies > 0 {
            options.push((
                "FromLastPoint",
                if *from_last {
                    "FromLastPoint=No"
                } else {
                    "FromLastPoint=Yes"
                },
            ));
            options.push((
                "UseLastDistance",
                if *use_last_distance {
                    "UseLastDistance=No"
                } else {
                    "UseLastDistance=Yes"
                },
            ));
            options.push((
                "UseLastDirection",
                if *use_last_direction {
                    "UseLastDirection=No"
                } else {
                    "UseLastDirection=Yes"
                },
            ));
        }
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
    a[0].hypot(a[1]).hypot(a[2])
}

/// Map a baked curve parameter back when affine circle baking changed its parameterization.
fn source_curve_fraction(
    source: &Curve,
    world: &Curve,
    placement: Transform,
    fraction: f64,
    limits: &Limits,
) -> Result<f64> {
    match (source, world) {
        (Curve::Circle { .. }, Curve::Nurbs(_)) => {
            let point = om9_core::curve_ops::point_at(world, fraction, limits)?;
            Ok(om9_core::curve_surface_ops::closest_point_on_placed_curve(
                source, placement, point, limits,
            )?
            .0)
        }
        (
            Curve::Polycurve { segments, reversed },
            Curve::Polycurve {
                segments: baked, ..
            },
        ) => {
            let count = segments.len();
            let forward = if *reversed { 1.0 - fraction } else { fraction };
            let scaled = forward * count as f64;
            let index = (scaled.floor() as usize).min(count - 1);
            let local = scaled - index as f64;
            let mapped =
                source_curve_fraction(&segments[index], &baked[index], placement, local, limits)?;
            let result = (index as f64 + mapped) / count as f64;
            Ok(if *reversed { 1.0 - result } else { result })
        }
        _ => Ok(fraction),
    }
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

fn unit_direction(v: Point3) -> Result<Point3> {
    finite(v)?;
    let largest = v.into_iter().map(f64::abs).fold(0., f64::max);
    if largest == 0. { return Err(Error::Invalid("Top 11 direction")); }
    unit(v.map(|component| component / largest))
}

fn signed_degrees(axis: Point3, from: Point3, to: Point3) -> Result<f64> {
    let axis = unit(axis)?;
    let from = unit_direction(from)?;
    let to = unit_direction(to)?;
    let from = unit(vector(scale(axis, dot(from, axis)), from))?;
    let to = unit(vector(scale(axis, dot(to, axis)), to))?;
    let angle = dot(axis, cross(from, to)).atan2(dot(from, to));
    Ok(angle.to_degrees())
}

// Offset applied to the original, the last step, and the last destination.
// Both interactive preview and commit use this candidate.
fn translation_candidate(pending: &Top11Pending, point: Point3, plane: CPlane) -> Result<Option<(Point3, Point3, Point3)>> {
    match pending {
        Top11Pending::Move { base: Some(base), vertical, .. } => {
            let mut delta=vector(*base,point);
            if *vertical {let n=unit(cross(plane.x,plane.y))?;delta=scale(n,dot(delta,n));}
            finite(delta)?;Ok(Some((delta,delta,point)))
        }
        Top11Pending::Duplicate {base:Some(base),vertical,from_last,use_last_distance,use_last_direction,last_delta,last_destination,..} => {
            let start=if *from_last {last_destination.unwrap_or(*base)}else{*base};
            let mut delta=vector(start,point);
            finite(delta)?;
            if *vertical {let n=unit(cross(plane.x,plane.y))?;delta=scale(n,dot(delta,n));}
            if *use_last_direction && *use_last_distance && let Some(last)=last_delta {delta=*last;}
            else {
                if *use_last_direction && let Some(last)=last_delta {delta=scale(unit_direction(*last)?,norm(delta));}
                if *use_last_distance && let Some(last)=last_delta {delta=scale(unit_direction(delta)?,norm(*last));}
            }
            finite(delta)?;
            if delta == [0.; 3] {return Err(Error::Invalid("duplicate destination equals the base point"));}
            let offset=add(vector(*base,start),delta);let destination=add(start,delta);finite(offset)?;finite(destination)?;
            Ok(Some((offset,delta,destination)))
        }
        _ => Ok(None),
    }
}

fn mirror_plane(points: &[Point3], point: Point3, three_point: bool, plane: CPlane) -> Result<(Point3, Point3)> {
    let origin = points[0];
    let first = if three_point { vector(origin, points[1]) } else { vector(origin, point) };
    let second = if three_point { vector(origin, point) } else { cross(plane.x, plane.y) };
    Ok((origin, unit(cross(unit_direction(first)?, unit_direction(second)?))?))
}

impl Controller {
    /// Preview using the transform and plane calculations used by point acceptance.
    pub(super) fn top11_transform_preview(&self, point: Point3, draft: &str) -> Result<Vec<[Point3; 2]>> {
        use om9_scene::scene::RenderBuffers;
        let Some(pending)=self.top11_pending.as_ref() else{return Ok(Vec::new())};
        let plane = self.input_plane();
        let (ids, transform) = match pending {
            Top11Pending::Move { ids, base: Some(_), .. } | Top11Pending::Duplicate { ids, base: Some(_), .. } => {
                let Some((offset, _, _)) = translation_candidate(pending, point, plane)? else { return Ok(Vec::new()) };
                (ids, Transform::translation(offset))
            }
            Top11Pending::Rotate { ids, center: Some(center), reference, .. } => {
                let axis = unit(cross(plane.x, plane.y))?;
                let degrees = if let Some(reference) = reference {
                    signed_degrees(axis, vector(*center, *reference), vector(*center, point))?
                } else if let Ok(value) = draft.trim().parse::<f64>() {
                    value
                } else { return Ok(Vec::new()) };
                (ids, top11::rotation(*center, axis, degrees)?)
            }
            Top11Pending::Mirror { ids, points, three_point, .. } if points.len() == if *three_point { 2 } else { 1 } => {
                let (origin, normal) = mirror_plane(points, point, *three_point, plane)?;
                let mut transform = Transform::identity();
                for row in 0..3 {
                    for column in 0..3 { transform.0[row][column] -= 2. * normal[row] * normal[column]; }
                    transform.0[row][3] = 2. * normal[row] * dot(origin, normal);
                }
                transform.validate()?;
                (ids, transform)
            }
            _ => return Ok(Vec::new()),
        };
        let scene=self.scene().ok_or(Error::Missing("preview scene"))?;let limits=self.coordinator.limits();
        let max=limits.max_points.min(limits.max_bytes/(std::mem::size_of::<[Point3;2]>() as u64));
        let mut count=0usize;
        for object in scene.objects.iter().filter(|object| object.visible && ids.contains(&object.root)) {
            let n=match object.buffers.as_ref() {
                RenderBuffers::Points{points,..}=>points.len(),
                RenderBuffers::Curve{points,..}=>points.len().saturating_sub(1),
                RenderBuffers::Mesh{edges,triangles,..}=>if edges.is_empty(){triangles.len().checked_mul(3).ok_or(Error::Overflow("top11 preview"))?}else{edges.len()},
            };
            count=count.checked_add(n).ok_or(Error::Overflow("top11 preview"))?;
            if count as u64>max{return Err(Error::Budget("top11 preview"));}
        }
        let mut wire=Vec::new();wire.try_reserve_exact(count).map_err(|_|Error::Budget("top11 preview"))?;
        for object in scene.objects.iter().filter(|object| object.visible && ids.contains(&object.root)) {
            let t=transform.compose(object.transform)?;
            let mut push=|a:Point3,b:Point3|->Result<()> {
                wire.push([t.point(a)?,t.point(b)?]);Ok(())
            };
            match object.buffers.as_ref() {
                RenderBuffers::Points{points,..}=>for p in points {push(*p,*p)?;},
                RenderBuffers::Curve{points,..}=>for pair in points.windows(2) {push(pair[0],pair[1])?;},
                RenderBuffers::Mesh{vertices,edges,triangles}=>{
                    if edges.is_empty(){for (face,_) in triangles {for i in 0..3 {let a=vertices.get(face[i] as usize).ok_or(Error::Invalid("preview index"))?;let b=vertices.get(face[(i+1)%3] as usize).ok_or(Error::Invalid("preview index"))?;push(*a,*b)?;}}}
                    else {for [a,b] in edges {let a=vertices.get(*a as usize).ok_or(Error::Invalid("preview index"))?;let b=vertices.get(*b as usize).ok_or(Error::Invalid("preview index"))?;push(*a,*b)?;}}
                }
            }
        }
        Ok(wire)
    }
    pub fn top11_context(&self) -> Top11Context {
        Top11Context::capture(self.document(), &self.selection, self.is_running())
    }

    pub fn top11_availability(&self) -> [Top11Availability; 11] {
        let mut items = top11::toolbar(&self.top11_context());
        let has_native_brep = self.selection.iter().any(|id| {
            self.document().entities.get(id).is_some_and(|entity| matches!(entity.geometry, Geometry::NativeBrep(_)))
        });
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
            if matches!(item.action, Top11Action::Split | Top11Action::Trim) {
                let context = self.top11_context();
                if !context.running && context.selected > 1 && context.curves == context.selected {
                    *item = Top11Availability {
                        action: item.action,
                        enabled: true,
                        reason: "",
                    };
                }
            }
            #[cfg(feature = "occt-brep")]
            if item.action == Top11Action::Split {
                let context = self.top11_context();
                if !context.running
                    && context.selected > 0
                    && context.curves + context.surfaces + context.breps == context.selected
                {
                    *item = Top11Availability {
                        action: item.action,
                        enabled: true,
                        reason: "",
                    };
                }
            }
            #[cfg(feature = "occt-brep")]
            if item.action == Top11Action::Trim {
                let context = self.top11_context();
                if !context.running
                    && context.selected >= 2
                    && context.curves + context.surfaces + context.breps == context.selected
                    && context.surfaces + context.breps >= 1
                {
                    *item = Top11Availability {
                        action: item.action,
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
                    && context.curves + context.meshes + context.breps == context.selected
                {
                    *item = Top11Availability {
                        action: Top11Action::Explode,
                        enabled: true,
                        reason: "",
                    };
                }
            }
            if item.action == Top11Action::Explode && !self.is_running() && !self.selection.is_empty() {
                let mut actionable=false;
                let all_supported=self.selection.iter().all(|id| {
                    self.document().entities.get(id).is_some_and(|e| match &e.geometry {
                        Geometry::Curve(_) | Geometry::Mesh(_) => {actionable=true;true}
                        Geometry::Brep(_) if cfg!(feature="occt-brep") => {actionable=true;true}
                        Geometry::NativeBrep(_) if cfg!(feature="occt-brep") => {actionable=true;true}
                        Geometry::Instance(_) => true,
                        _ => false,
                    })
                });
                if all_supported && actionable {
                    *item=Top11Availability{action:item.action,enabled:true,reason:""};
                }
            }
            #[cfg(feature = "occt-brep")]
            if item.action == Top11Action::Join {
                let context = self.top11_context();
                if !context.running && context.selected >= 2 && context.breps + context.surfaces == context.selected {
                    *item = Top11Availability {
                        action: Top11Action::Join,
                        enabled: true,
                        reason: "",
                    };
                }
            }
            if !self.is_running() && has_native_brep && (matches!(item.action, Top11Action::Join | Top11Action::Split | Top11Action::Trim) || (item.action == Top11Action::Explode && !cfg!(feature="occt-brep"))) {
                *item = Top11Availability {
                    action: item.action,
                    enabled: false,
                    reason: "retained native BRep topology editing is unavailable",
                };
            }
        }
        items
    }

    pub fn top11_pending(&self) -> Option<&Top11Pending> {
        self.top11_pending.as_ref()
    }

    pub(crate) fn top11_transform_is_pending(&self) -> bool {
        matches!(self.top11_pending, Some(Top11Pending::Move { .. } | Top11Pending::Duplicate { .. } | Top11Pending::Mirror { .. } | Top11Pending::Rotate { .. }))
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

    /// Delete selected control grips in one undoable geometry transaction.
    pub fn top11_delete_grip(&mut self, handle: top11::GripHandle) -> Result<Top11Result> {
        self.top11_delete_grips(&[handle])
    }

    pub fn top11_delete_grips(&mut self, handles: &[top11::GripHandle]) -> Result<Top11Result> {
        if self.top11_grip_mode() != Some(GripMode::Control) || handles.is_empty() {
            return Err(Error::Invalid("stale or inactive control grip"));
        }
        let current = self.top11_grip_handles()?;
        let mut groups: std::collections::BTreeMap<ObjectId, BTreeSet<usize>> =
            std::collections::BTreeMap::new();
        for handle in handles {
            if !current.contains(handle)
                || !groups
                    .entry(handle.object)
                    .or_default()
                    .insert(handle.index)
            {
                return Err(Error::Invalid("stale or duplicate control grip"));
            }
        }
        let mut mutations = Vec::new();
        for (id, indices) in &groups {
            let entity = self
                .document()
                .entities
                .get(id)
                .ok_or(Error::Missing("object"))?;
            let geometry = match &entity.geometry {
                Geometry::Curve(curve) => {
                    let mut curve = curve.clone();
                    for index in indices.iter().rev() {
                        curve = om9_core::curve_ops::delete_control_point(
                            &curve,
                            *index,
                            self.coordinator.limits(),
                        )?;
                    }
                    Geometry::Curve(curve)
                }
                Geometry::Surface(surface) => {
                    Geometry::Surface(om9_core::surface_kernel::delete_control_points(
                        surface,
                        indices,
                        self.coordinator.limits(),
                    )?)
                }
                _ => {
                    return Err(Error::Unsupported(
                        "control point deletion needs a NURBS curve or surface",
                    ));
                }
            };
            mutations.push(Mutation::Geometry {
                source: self.document().stamp(*id)?,
                geometry,
            });
        }
        let selection = self.selection.iter().copied().collect();
        self.commit_top11(Top11Action::ControlPoints, mutations, Some(selection))?;
        Ok(Top11Result::Completed {
            label: "Control points deleted",
            objects: groups.len(),
        })
    }
    /// The grips picked while grips are on (click, Shift+click, window), as live handles.
    /// Picks whose object left the selection or whose index no longer exists drop out.
    pub fn selected_grips(&self) -> Vec<top11::GripHandle> {
        if self.grip_selection.is_empty() {
            return Vec::new();
        }
        let Ok(handles) = self.top11_grip_handles() else {
            return Vec::new();
        };
        handles
            .into_iter()
            .filter(|handle| self.grip_selection.contains(&(handle.object, handle.index)))
            .collect()
    }

    /// Whether at least one grip is picked (the gumball then acts on grips).
    pub fn grips_selected(&self) -> bool {
        self.top11_grip.is_some() && !self.grip_selection.is_empty()
    }

    /// Whether this grip is picked.
    pub fn grip_is_selected(&self, handle: &top11::GripHandle) -> bool {
        self.grip_selection.contains(&(handle.object, handle.index))
    }

    /// Pick grips: replace the picked set, add to it (Shift) or remove from it (Ctrl).
    pub fn select_grips(&mut self, grips: &[top11::GripHandle], change: GripPick) {
        let keys = grips.iter().map(|handle| (handle.object, handle.index));
        match change {
            GripPick::Replace => {
                self.grip_selection = keys.collect();
            }
            GripPick::Add => {
                for key in keys {
                    if !self.grip_selection.contains(&key) {
                        self.grip_selection.push(key);
                    }
                }
            }
            GripPick::Remove => {
                let removed: Vec<_> = keys.collect();
                self.grip_selection.retain(|key| !removed.contains(key));
            }
        }
        self.gumball_origin = None;
    }

    /// The gumball transform being previewed on the picked grips, if any.
    pub fn grip_preview(&self) -> Option<om9_core::geometry::Transform> {
        self.grip_preview
    }

    /// The picked grips at their previewed positions plus the previewed edited curves, so the
    /// viewport can show a gumball drag on grips before it is committed.
    pub fn grip_preview_curves(&self) -> Vec<Vec<Point3>> {
        let Some(delta) = self.grip_preview else {
            return Vec::new();
        };
        let moves: Vec<_> = self
            .selected_grips()
            .into_iter()
            .filter_map(|handle| Some((handle, delta.point(handle.point).ok()?)))
            .collect();
        self.grip_edit_preview(&moves)
    }

    /// Sample the curves produced by moving `moves`, in world space, for a drag preview.
    pub fn grip_edit_preview(&self, moves: &[(top11::GripHandle, Point3)]) -> Vec<Vec<Point3>> {
        let limits = self.coordinator.limits();
        let Ok(edited) = top11::move_grips_geometry(self.document(), moves, limits) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for (id, geometry) in edited {
            let (Some(entity), om9_core::geometry::Geometry::Curve(curve)) =
                (self.document().entities.get(&id), &geometry)
            else {
                continue;
            };
            let Ok(points) = om9_core::curve_ops::sample_curve(curve, 200, limits) else {
                continue;
            };
            out.push(
                points
                    .into_iter()
                    .filter_map(|point| entity.placement.point(point).ok())
                    .collect(),
            );
        }
        out
    }

    /// Commit several grip moves as one geometry edit (one Undo entry).
    pub fn top11_move_grips(
        &mut self,
        moves: &[(top11::GripHandle, Point3)],
    ) -> Result<Top11Result> {
        let Some(first) = moves.first() else {
            return Err(Error::Missing("grips to move"));
        };
        let current = self.top11_grip_handles()?;
        let mut seen = BTreeSet::new();
        for (handle, _) in moves {
            if !current.contains(handle) || !seen.insert((handle.object, handle.index)) {
                return Err(Error::Invalid("stale or duplicate grip"));
            }
        }
        let edited = top11::move_grips_geometry(self.document(), moves, self.coordinator.limits())?;
        let mut mutations = Vec::with_capacity(edited.len());
        let mut ids = Vec::with_capacity(edited.len());
        for (id, geometry) in edited {
            let source = self.document().stamp(id)?;
            mutations.push(Mutation::Geometry { source, geometry });
            ids.push(id);
        }
        let objects = ids.len();
        let selection: Vec<ObjectId> = self.selection.iter().copied().collect();
        self.commit_top11(
            if first.0.mode == GripMode::Edit {
                Top11Action::EditPoints
            } else {
                Top11Action::ControlPoints
            },
            mutations,
            Some(selection),
        )?;
        Ok(Top11Result::Completed {
            label: "Grip edit",
            objects,
        })
    }

    pub fn top11_needs_object(&self) -> bool {
        self.top11_pending
            .as_ref()
            .is_some_and(Top11Pending::needs_object)
    }

    pub fn top11_split_iso_target(&self) -> Option<ObjectId> {
        match self.top11_pending {
            Some(Top11Pending::SplitIso { target, .. }) => Some(target),
            _ => None,
        }
    }

    #[cfg(feature = "occt-brep")]
    pub fn top11_split_iso_accept_viewport_pick(
        &mut self,
        target: ObjectId,
        point: Point3,
        ray: om9_scene::camera::Ray,
    ) -> Result<()> {
        finite(point)?;
        let Some(Top11Pending::SplitIso {
            target: expected,
            direction,
            shrink,
            shrink_available,
        }) = self.top11_pending.clone()
        else {
            return Err(Error::Missing("split isocurve"));
        };
        if target != expected {
            return Err(Error::Invalid("pick the split surface"));
        }
        let exact = {
            let entity = self
                .document()
                .entities
                .get(&target)
                .ok_or(Error::Missing("split surface"))?;
            om9_kernel::measurement::surface_ray_hit(
                &entity.geometry,
                entity.placement,
                ray.origin,
                ray.direction,
                point,
                self.document().effective_tolerance()?.absolute_mm,
                self.coordinator.limits(),
            )
            .map_err(|error| Error::Io(format!("Split isocurve viewport: {error}")))?
        };
        self.top11_pending = None;
        let result = self.split_isocurve_at_point(target, direction, exact, true, shrink);
        if result.is_err() {
            self.top11_pending = Some(Top11Pending::SplitIso {
                target: expected,
                direction,
                shrink,
                shrink_available,
            });
        }
        result.map(|_| ())
    }

    #[cfg(not(feature = "occt-brep"))]
    pub fn top11_split_iso_accept_viewport_pick(
        &mut self,
        _target: ObjectId,
        _point: Point3,
        _ray: om9_scene::camera::Ray,
    ) -> Result<()> {
        Err(Error::Unsupported(
            "isocurve viewport needs the OCCT kernel feature",
        ))
    }

    /// Whether a viewport hit belongs to an as-yet untrimmed multi-target curve.
    pub fn top11_trim_many_has_target(&self, id: ObjectId) -> bool {
        match &self.top11_pending {
            Some(Top11Pending::TrimMany {
                fractions: Some(items),
                ..
            })
            | Some(Top11Pending::TrimMutual {
                fractions: items, ..
            }) => items.iter().any(|(target, _)| *target == id),
            Some(Top11Pending::TrimFace {
                target,
                partition,
                fractions,
                ..
            }) => {
                (*target == id && partition.is_some())
                    || fractions.iter().any(|(curve, _)| *curve == id)
            }
            Some(Top11Pending::TrimSurfaces {
                partitions,
                fractions,
                ..
            }) => {
                partitions.iter().any(|(target, _)| *target == id)
                    || fractions.iter().any(|(curve, _)| *curve == id)
            }
            _ => false,
        }
    }

    pub fn top11_trim_many_ready(&self) -> bool {
        matches!(
            self.top11_pending,
            Some(
                Top11Pending::TrimMany {
                    fractions: Some(_),
                    ..
                } | Top11Pending::TrimMutual { .. }
                    | Top11Pending::TrimFace { .. }
                    | Top11Pending::TrimSurfaces { .. }
            )
        )
    }

    /// Use the viewport hit's explicit target and world point in the shared trim owner.
    /// Surface hits are refined against the exact partition along the camera ray;
    /// display triangles are only used to identify the object and approximate depth.
    pub fn top11_trim_many_accept_viewport_pick(
        &mut self,
        id: ObjectId,
        world: Point3,
        ray: om9_scene::camera::Ray,
    ) -> Result<Top11Result> {
        finite(world)?;
        finite(ray.origin)?;
        finite(ray.direction)?;
        #[cfg(feature = "occt-brep")]
        {
            let partition = match &self.top11_pending {
                Some(Top11Pending::TrimFace {
                    target,
                    partition: Some(partition),
                    ..
                }) if *target == id => Some(partition),
                Some(Top11Pending::TrimSurfaces { partitions, .. }) => partitions
                    .iter()
                    .find(|(target, _)| *target == id)
                    .map(|(_, partition)| partition),
                _ => None,
            };
            if let Some(partition) = partition {
                let direction = unit(ray.direction)?;
                let depth = dot(vector(ray.origin, world), direction);
                if !depth.is_finite() || depth <= 0.0 {
                    return Err(Error::Invalid("trim viewport ray"));
                }
                let end = add(ray.origin, scale(direction, depth * 2.0 + 2.0));
                finite(end)?;
                let curve = Curve::Line {
                    start: ray.origin,
                    end,
                    domain: [0., 1.],
                };
                let tolerance = self.document().effective_tolerance()?.absolute_mm;
                let fractions = om9_kernel::intersect::curve_brep_fractions_including_endpoints(
                    partition,
                    &curve,
                    tolerance,
                    self.coordinator.limits(),
                )
                .map_err(|error| Error::Io(format!("Trim viewport intersection: {error}")))?;
                let exact = fractions
                    .into_iter()
                    .map(|fraction| add(ray.origin, scale(vector(ray.origin, end), fraction)))
                    .min_by(|a, b| norm(vector(*a, world)).total_cmp(&norm(vector(*b, world))))
                    .ok_or(Error::Invalid("viewport ray misses exact trim surface"))?;
                return self.top11_trim_many_accept_pick(id, exact);
            }
        }
        self.top11_trim_many_accept_pick(id, world)
    }

    /// Use an explicit object and world point in the shared trim owner.
    pub fn top11_trim_many_accept_pick(
        &mut self,
        id: ObjectId,
        world: Point3,
    ) -> Result<Top11Result> {
        finite(world)?;
        let pending = self
            .top11_pending
            .take()
            .ok_or(Error::Missing("trim command"))?;
        match pending {
            Top11Pending::TrimMany {
                targets,
                cutters,
                fractions: Some(fractions),
                completed,
            } => self.apply_trim_many(targets, cutters, fractions, completed, world, Some(id)),
            Top11Pending::TrimMutual {
                objects,
                fractions,
                completed,
            } => self.apply_trim_mutual(objects, fractions, completed, world, Some(id)),
            Top11Pending::TrimFace {
                target,
                partition,
                merge_regions,
                original_edges,
                objects,
                fractions,
                completed,
            } => self.apply_trim_face(
                target,
                partition,
                merge_regions,
                original_edges,
                objects,
                fractions,
                completed,
                world,
                Some(id),
            ),
            Top11Pending::TrimSurfaces {
                objects,
                partitions,
                original_edges,
                fractions,
                completed,
            } => self.apply_trim_surfaces(
                objects,
                partitions,
                original_edges,
                fractions,
                completed,
                world,
                Some(id),
            ),
            other => {
                self.top11_pending = Some(other);
                Err(Error::Invalid("trim is not picking portions"))
            }
        }
    }

    fn apply_trim_many(
        &mut self,
        targets: Vec<ObjectId>,
        cutters: Vec<ObjectId>,
        mut fractions: Vec<(ObjectId, Vec<f64>)>,
        mut completed: usize,
        point: Point3,
        picked: Option<ObjectId>,
    ) -> Result<Top11Result> {
        let result = (|| {
            let index = if let Some(id) = picked {
                fractions
                    .iter()
                    .position(|(target, _)| *target == id)
                    .ok_or(Error::Invalid("pick a remaining trim target"))?
            } else {
                let tolerance = self.document().effective_tolerance()?.absolute_mm;
                let mut nearest: Option<(usize, f64)> = None;
                let mut within_tolerance = 0usize;
                for (index, (id, _)) in fractions.iter().enumerate() {
                    let entity = self
                        .document()
                        .entities
                        .get(id)
                        .ok_or(Error::Missing("trim curve"))?;
                    let Geometry::Curve(curve) = &entity.geometry else {
                        return Err(Error::Invalid("trim target must be a curve"));
                    };
                    let (_, _, distance) =
                        om9_core::curve_surface_ops::closest_point_on_placed_curve(
                            curve,
                            entity.placement,
                            point,
                            self.coordinator.limits(),
                        )?;
                    if distance <= tolerance {
                        within_tolerance += 1;
                    }
                    if nearest.is_none_or(|(_, best)| distance < best) {
                        nearest = Some((index, distance));
                    }
                }
                let (index, distance) = nearest.ok_or(Error::Missing("remaining trim targets"))?;
                if distance > tolerance {
                    return Err(Error::Invalid(
                        "pick a target curve within document tolerance",
                    ));
                }
                if within_tolerance > 1 {
                    return Err(Error::Invalid(
                        "ambiguous trim point; pick a target in the viewport",
                    ));
                }
                index
            };
            let (target, cuts) = &fractions[index];
            let entity = self
                .document()
                .entities
                .get(target)
                .ok_or(Error::Missing("trim curve"))?;
            let Geometry::Curve(curve) = &entity.geometry else {
                return Err(Error::Missing("trim curve"));
            };
            let pieces = top11::trim_by_fractions_in_world(
                curve,
                cuts,
                point,
                entity.placement,
                self.coordinator.limits(),
            )?;
            // Prepare every surviving piece's future cuts before publishing anything.
            let mut remaining = Vec::with_capacity(pieces.len());
            for piece in &pieces {
                let mut candidate = entity.clone();
                candidate.geometry = Geometry::Curve(piece.clone());
                let mut next = Vec::new();
                for cutter in &cutters {
                    match self.cutter_fractions_for_entity(&candidate, *cutter) {
                        Ok(hit) => next.extend(hit),
                        Err(Error::Invalid("cutting object has no intersection")) => {}
                        Err(error) => return Err(error),
                    }
                }
                remaining.push(next);
            }
            Ok((index, *target, pieces, remaining))
        })();
        let result = match result {
            Ok((index, target, pieces, remaining)) => self
                .commit_cut(Top11Action::Trim, target, pieces)
                .map(|ids| {
                    fractions.remove(index);
                    fractions.extend(
                        ids.into_iter()
                            .zip(remaining)
                            .filter(|(_, cuts)| !cuts.is_empty()),
                    );
                    completed += 1;
                    Top11Result::Awaiting {
                        label: Top11Action::Trim.label(),
                        prompt: "pick portions to remove (Enter to finish)",
                    }
                }),
            Err(error) => Err(error),
        };
        self.top11_pending = Some(Top11Pending::TrimMany {
            targets,
            cutters,
            fractions: Some(fractions),
            completed,
        });
        result
    }

    fn mutual_trim_fractions(&self, objects: &[&Entity]) -> Result<Vec<(ObjectId, Vec<f64>)>> {
        let mut out = Vec::new();
        for target in objects {
            if !matches!(target.geometry, Geometry::Curve(_)) {
                continue;
            }
            let mut cuts = Vec::new();
            for cutter in objects {
                if cutter.id == target.id {
                    continue;
                }
                match self.cutter_fractions_between(target, cutter, true) {
                    Ok(hit) => cuts.extend(hit),
                    Err(Error::Invalid("cutting object has no intersection")) => {}
                    Err(Error::Unsupported("curve may overlap surface")) => {}
                    Err(error) => return Err(error),
                }
            }
            let Geometry::Curve(curve) = &target.geometry else { return Err(Error::Missing("trim curve")); };
            let cuts = top11::normalize_cut_fractions(curve, target.placement, cuts, self.document().effective_tolerance()?.absolute_mm, true, self.coordinator.limits())?;
            let has_interior = cuts.iter().any(|f| *f > 0.0 && *f < 1.0);
            if !has_interior && !cuts.is_empty() {
                let Geometry::Curve(curve) = &target.geometry else {
                    return Err(Error::Missing("trim curve"));
                };
                if top11::curve_closed_for_cut(curve, self.coordinator.limits())? {
                    continue;
                }
            }
            if !cuts.is_empty() {
                out.push((target.id, cuts));
            }
        }
        Ok(out)
    }

    fn start_trim_mutual(&mut self) -> Result<Top11Result> {
        let objects: Vec<_> = self
            .selection
            .iter()
            .map(|id| {
                self.document()
                    .entities
                    .get(id)
                    .ok_or(Error::Missing("trim object"))
            })
            .collect::<Result<_>>()?;
        #[cfg(feature = "occt-brep")]
        let mut curve_targets_only = false;
        #[cfg(not(feature = "occt-brep"))]
        let curve_targets_only = false;
        #[cfg(feature = "occt-brep")]
        if objects.len() >= 2 {
            let faces: Vec<_> = objects
                .iter()
                .filter(|e| matches!(e.geometry, Geometry::Brep(_) | Geometry::Surface(_)))
                .collect();
            if faces.len() >= 2
                && objects.iter().all(|e| {
                    matches!(
                        e.geometry,
                        Geometry::Curve(_) | Geometry::Surface(_) | Geometry::Brep(_)
                    )
                })
            {
                let mut original_edges = Vec::new();
                for face in &faces {
                    if let Geometry::Brep(brep) = &face.geometry {
                        let measure =
                            om9_kernel::creation::measure(brep, self.coordinator.limits())
                                .map_err(|error| Error::Io(format!("Trim face: {error}")))?;
                        if measure.faces > 1 || measure.solids > 0 {
                            original_edges.push((face.id, self.trim_original_world(face, brep)?));
                        }
                    }
                }
                let partitions = self.trim_surface_partitions(&objects)?;
                let fractions = self.mutual_trim_fractions(&objects)?;
                if partitions.is_empty() && fractions.is_empty() {
                    return Err(Error::Invalid("selected objects do not intersect"));
                }
                self.top11_pending = Some(Top11Pending::TrimSurfaces {
                    objects: objects.iter().map(|entity| entity.id).collect(),
                    partitions,
                    original_edges,
                    fractions,
                    completed: 0,
                });
                return Ok(Top11Result::Awaiting {
                    label: Top11Action::Trim.label(),
                    prompt: "pick portions to remove (Enter to finish)",
                });
            }
            if faces.len() == 1
                && objects.iter().all(|e| {
                    matches!(
                        e.geometry,
                        Geometry::Curve(_) | Geometry::Surface(_) | Geometry::Brep(_)
                    )
                })
            {
                let face = faces[0];
                let cutters: Vec<_> = objects
                    .iter()
                    .filter(|e| e.id != face.id)
                    .map(|e| (&e.geometry, e.placement))
                    .collect();
                let deflection = match &face.geometry {
                    Geometry::Brep(brep) => brep.display.deflection_mm,
                    _ => om9_kernel::creation::DEFAULT_DEFLECTION_MM,
                };
                let merge_regions = match &face.geometry {
                    Geometry::Brep(brep) => {
                        let measure =
                            om9_kernel::creation::measure(brep, self.coordinator.limits())
                                .map_err(|error| Error::Io(format!("Trim face: {error}")))?;
                        measure.faces == 1 && measure.solids == 0
                    }
                    Geometry::Surface(_) => true,
                    _ => false,
                };
                let original_edges = if !merge_regions {
                    let Geometry::Brep(brep) = &face.geometry else {
                        return Err(Error::Missing("trim BRep"));
                    };
                    Some(self.trim_original_world(face, brep)?)
                } else {
                    None
                };
                let split = om9_kernel::measurement::split_surface_by_geometry(
                    &face.geometry,
                    face.placement,
                    &cutters,
                    deflection,
                    self.coordinator.limits(),
                );
                match split {
                    Ok(split) => {
                        let fractions = self.mutual_trim_fractions(&objects)?;
                        self.top11_pending = Some(Top11Pending::TrimFace {
                            target: face.id,
                            partition: Some(split.brep),
                            merge_regions,
                            original_edges,
                            objects: objects.iter().map(|e| e.id).collect(),
                            fractions,
                            completed: 0,
                        });
                        return Ok(Top11Result::Awaiting {
                            label: Top11Action::Trim.label(),
                            prompt: "pick portions to remove (Enter to finish)",
                        });
                    }
                    Err(om9_kernel::KernelError::InvalidInput(message))
                        if message == "cutters do not split surface"
                            || message == "split surface: target must be an open face or shell" =>
                    {
                        curve_targets_only = true;
                    }
                    Err(error) => return Err(Error::Io(format!("Trim face: {error}"))),
                }
            }
        }
        if objects.len() < 2
            || (!curve_targets_only
                && objects
                    .iter()
                    .any(|e| !matches!(e.geometry, Geometry::Curve(_))))
        {
            return Err(Error::Invalid("select at least two curves for Trim"));
        }
        let fractions = self.mutual_trim_fractions(&objects)?;
        if fractions.is_empty() {
            return Err(Error::Invalid("selected curves have no intersection"));
        }
        self.top11_pending = Some(Top11Pending::TrimMutual {
            objects: objects.iter().map(|e| e.id).collect(),
            fractions,
            completed: 0,
        });
        Ok(Top11Result::Awaiting {
            label: Top11Action::Trim.label(),
            prompt: "pick portions to remove (Enter to finish)",
        })
    }

    #[cfg(feature = "occt-brep")]
    fn trim_original_world(
        &self,
        entity: &Entity,
        brep: &om9_core::brep::Brep,
    ) -> Result<om9_core::brep::Brep> {
        let world = if entity.placement == Transform::identity() {
            brep.clone()
        } else {
            om9_kernel::transform_ops::transform_brep(
                brep,
                &entity.placement,
                brep.display.deflection_mm,
                self.coordinator.limits(),
            )
            .map_err(|error| Error::Io(format!("Trim original edges: {error}")))?
        };
        let before = om9_kernel::creation::measure(brep, self.coordinator.limits())
            .map_err(|e| Error::Io(format!("Trim face: {e}")))?;
        let after = om9_kernel::creation::measure(&world, self.coordinator.limits())
            .map_err(|e| Error::Io(format!("Trim face: {e}")))?;
        if before.faces != after.faces {
            return Err(Error::Invalid("trim transform changed face count"));
        }
        Ok(world)
    }

    #[cfg(feature = "occt-brep")]
    fn trim_surface_partitions(
        &self,
        objects: &[&Entity],
    ) -> Result<Vec<(ObjectId, om9_core::brep::Brep)>> {
        let mut partitions = Vec::new();
        for face in objects {
            if matches!(face.geometry, Geometry::Curve(_)) {
                continue;
            }
            let cutters: Vec<_> = objects
                .iter()
                .filter(|cutter| cutter.id != face.id)
                .map(|cutter| (&cutter.geometry, cutter.placement))
                .collect();
            if cutters.is_empty() {
                continue;
            }
            let deflection = match &face.geometry {
                Geometry::Brep(brep) => brep.display.deflection_mm,
                Geometry::Surface(_) => om9_kernel::creation::DEFAULT_DEFLECTION_MM,
                _ => return Err(Error::Invalid("mutual surface Trim expects surfaces")),
            };
            match om9_kernel::measurement::split_surface_by_geometry(
                &face.geometry,
                face.placement,
                &cutters,
                deflection,
                self.coordinator.limits(),
            ) {
                Ok(split) => partitions.push((face.id, split.brep)),
                Err(om9_kernel::KernelError::InvalidInput(message))
                    if message == "cutters do not split surface" => {}
                Err(error) => return Err(Error::Io(format!("Trim surface partition: {error}"))),
            }
        }
        Ok(partitions)
    }

    fn apply_trim_surfaces(
        &mut self,
        mut objects: Vec<ObjectId>,
        mut partitions: Vec<(ObjectId, om9_core::brep::Brep)>,
        original_edges: Vec<(ObjectId, om9_core::brep::Brep)>,
        mut fractions: Vec<(ObjectId, Vec<f64>)>,
        mut completed: usize,
        point: Point3,
        picked: Option<ObjectId>,
    ) -> Result<Top11Result> {
        #[cfg(feature = "occt-brep")]
        let mut picked = picked;
        #[cfg(feature = "occt-brep")]
        if picked.is_none() {
            let resolved = (|| {
                let tolerance = self.document().effective_tolerance()?.absolute_mm;
                let mut hits = Vec::new();
                for (id, surface) in &partitions {
                    let distance = om9_kernel::measurement::distance(
                        &Geometry::Brep(surface.clone()),
                        Transform::identity(),
                        &Geometry::Point(point),
                        Transform::identity(),
                        self.coordinator.limits(),
                    )
                    .map_err(|error| Error::Io(format!("Trim pick: {error}")))?
                    .millimetres;
                    if distance <= tolerance {
                        hits.push(*id);
                    }
                }
                for (id, _) in &fractions {
                    let entity = self
                        .document()
                        .entities
                        .get(id)
                        .ok_or(Error::Missing("trim curve"))?;
                    let Geometry::Curve(curve) = &entity.geometry else {
                        return Err(Error::Missing("trim curve"));
                    };
                    let (_, _, distance) =
                        om9_core::curve_surface_ops::closest_point_on_placed_curve(
                            curve,
                            entity.placement,
                            point,
                            self.coordinator.limits(),
                        )?;
                    if distance <= tolerance {
                        hits.push(*id);
                    }
                }
                if hits.len() != 1 {
                    return Err(Error::Invalid("pick one unambiguous trim object"));
                }
                Ok(hits[0])
            })();
            match resolved {
                Ok(id) => picked = Some(id),
                Err(error) => {
                    self.top11_pending = Some(Top11Pending::TrimSurfaces {
                        objects,
                        partitions,
                        original_edges,
                        fractions,
                        completed,
                    });
                    return Err(error);
                }
            }
        }
        if picked.is_some_and(|id| fractions.iter().any(|(curve, _)| *curve == id)) {
            return self.apply_trim_surfaces_curve(
                objects,
                partitions,
                original_edges,
                fractions,
                completed,
                point,
                picked.expect("curve pick"),
            );
        }
        let result: Result<_> = (|| {
            #[cfg(not(feature = "occt-brep"))]
            {
                let _ = (point, picked);
                Err(Error::Unsupported(
                    "surface Trim needs the OCCT kernel feature",
                ))
            }
            #[cfg(feature = "occt-brep")]
            {
                let index = if let Some(id) = picked {
                    partitions
                        .iter()
                        .position(|(target, _)| *target == id)
                        .ok_or(Error::Invalid("pick an intersecting trim surface"))?
                } else {
                    let tolerance = self.document().effective_tolerance()?.absolute_mm;
                    let mut matches = Vec::new();
                    for (index, (_, surface)) in partitions.iter().enumerate() {
                        let distance = om9_kernel::measurement::distance(
                            &Geometry::Brep(surface.clone()),
                            Transform::identity(),
                            &Geometry::Point(point),
                            Transform::identity(),
                            self.coordinator.limits(),
                        )
                        .map_err(|error| Error::Io(format!("Trim pick: {error}")))?
                        .millimetres;
                        if distance <= tolerance {
                            matches.push(index);
                        }
                    }
                    if matches.len() != 1 {
                        return Err(Error::Invalid("pick one unambiguous trim surface"));
                    }
                    matches[0]
                };
                let (target, partition) = &partitions[index];
                let limits = self.coordinator.limits();
                let tolerance = self.document().effective_tolerance()?.absolute_mm;
                let remaining = om9_kernel::editing::remove_picked_face(
                    partition,
                    point,
                    tolerance,
                    partition.display.deflection_mm,
                    limits,
                )
                .map_err(|error| Error::Io(format!("Trim surface: {error}")))?;
                let original = self
                    .document()
                    .entities
                    .get(target)
                    .ok_or(Error::Missing("trim surface"))?;
                let local = if let Some(remaining) = &remaining {
                    let unified = if let Some((_, original)) =
                        original_edges.iter().find(|(id, _)| id == target)
                    {
                        om9_kernel::editing::unify_surface_regions_preserving_edges(
                            remaining,
                            original,
                            remaining.display.deflection_mm,
                            limits,
                        )
                    } else {
                        om9_kernel::editing::unify_surface_regions(
                            remaining,
                            remaining.display.deflection_mm,
                            limits,
                        )
                    }
                    .map_err(|error| Error::Io(format!("Trim unify regions: {error}")))?;
                    if original.placement == Transform::identity() {
                        Some(unified)
                    } else {
                        let inverse = om9_kernel::transform_ops::inverse(&original.placement)
                            .map_err(|error| Error::Io(format!("Trim face inverse: {error}")))?;
                        Some(
                            om9_kernel::transform_ops::transform_brep(
                                &unified,
                                &inverse,
                                unified.display.deflection_mm,
                                limits,
                            )
                            .map_err(|error| Error::Io(format!("Trim face transform: {error}")))?,
                        )
                    }
                } else {
                    None
                };
                let mut next: Vec<Entity> = objects
                    .iter()
                    .filter(|id| **id != *target)
                    .map(|id| {
                        self.document()
                            .entities
                            .get(id)
                            .cloned()
                            .ok_or(Error::Missing("trim surface"))
                    })
                    .collect::<Result<_>>()?;
                if let Some(local) = &local {
                    let mut entity = original.clone();
                    entity.geometry = Geometry::Brep(local.clone());
                    next.push(entity);
                }
                let refs: Vec<_> = next.iter().collect();
                let mut next_partitions = self.trim_surface_partitions(&refs)?;
                let next_fractions = self.mutual_trim_fractions(&refs)?;
                next_partitions.retain(|(id, _)| id != target);
                if let Some(remaining) = remaining {
                    next_partitions.push((*target, remaining));
                }
                let ids = next.iter().map(|entity| entity.id).collect();
                let source = self.document().stamp(*target)?;
                let mutation = match local {
                    Some(local) => Mutation::Geometry {
                        source,
                        geometry: Geometry::Brep(local),
                    },
                    None => Mutation::Delete { source },
                };
                let selected = if next.iter().any(|e| e.id == *target) {
                    vec![*target]
                } else {
                    vec![]
                };
                Ok((mutation, selected, ids, next_partitions, next_fractions))
            }
        })();
        let result = match result {
            Ok((mutation, selected, next_ids, next_partitions, next_fractions)) => self
                .commit_top11(Top11Action::Trim, vec![mutation], Some(selected))
                .map(|_| {
                    objects = next_ids;
                    partitions = next_partitions;
                    fractions = next_fractions;
                    completed += 1;
                }),
            Err(error) => Err(error),
        };
        self.top11_pending = Some(Top11Pending::TrimSurfaces {
            objects,
            partitions,
            original_edges,
            fractions,
            completed,
        });
        result.map(|_| Top11Result::Awaiting {
            label: Top11Action::Trim.label(),
            prompt: "pick portions to remove (Enter to finish)",
        })
    }

    fn apply_trim_surfaces_curve(
        &mut self,
        mut objects: Vec<ObjectId>,
        mut partitions: Vec<(ObjectId, om9_core::brep::Brep)>,
        original_edges: Vec<(ObjectId, om9_core::brep::Brep)>,
        mut fractions: Vec<(ObjectId, Vec<f64>)>,
        mut completed: usize,
        point: Point3,
        picked: ObjectId,
    ) -> Result<Top11Result> {
        let result: Result<_> = (|| {
            let (_, cuts) = fractions
                .iter()
                .find(|(id, _)| *id == picked)
                .ok_or(Error::Invalid("pick an intersecting trim curve"))?;
            let original = self
                .document()
                .entities
                .get(&picked)
                .ok_or(Error::Missing("trim curve"))?;
            let Geometry::Curve(curve) = &original.geometry else {
                return Err(Error::Missing("trim curve"));
            };
            let pieces = if cuts.iter().any(|f| *f > 0.0 && *f < 1.0) {
                top11::trim_by_fractions_in_world(
                    curve,
                    cuts,
                    point,
                    original.placement,
                    self.coordinator.limits(),
                )?
            } else {
                Vec::new()
            };
            let mut replacements = Vec::new();
            let mut mutations = Vec::new();
            if pieces.len() == 1 {
                let geometry = Geometry::Curve(pieces[0].clone());
                let mut entity = original.clone();
                entity.geometry = geometry.clone();
                replacements.push(entity);
                mutations.push(Mutation::Geometry {
                    source: self.document().stamp(picked)?,
                    geometry,
                });
            } else {
                mutations.push(Mutation::Delete {
                    source: self.document().stamp(picked)?,
                });
                for piece in pieces {
                    let entity = child_entity(original, Geometry::Curve(piece));
                    mutations.push(Mutation::Insert {
                        document: self.document,
                        entity: entity.clone(),
                    });
                    replacements.push(entity);
                }
            }
            let mut next: Vec<Entity> = objects
                .iter()
                .filter(|id| **id != picked)
                .map(|id| {
                    self.document()
                        .entities
                        .get(id)
                        .cloned()
                        .ok_or(Error::Missing("trim object"))
                })
                .collect::<Result<_>>()?;
            next.extend(replacements.iter().cloned());
            let refs: Vec<_> = next.iter().collect();
            let next_fractions = self.mutual_trim_fractions(&refs)?;
            #[cfg(feature = "occt-brep")]
            let next_partitions = self.trim_surface_partitions(&refs)?;
            #[cfg(not(feature = "occt-brep"))]
            let next_partitions = Vec::new();
            let ids = next.iter().map(|entity| entity.id).collect();
            let selected = replacements.iter().map(|entity| entity.id).collect();
            Ok((mutations, selected, ids, next_partitions, next_fractions))
        })();
        let result = match result {
            Ok((mutations, selected, next_ids, next_partitions, next_fractions)) => self
                .commit_top11(Top11Action::Trim, mutations, Some(selected))
                .map(|_| {
                    objects = next_ids;
                    partitions = next_partitions;
                    fractions = next_fractions;
                    completed += 1;
                }),
            Err(error) => Err(error),
        };
        self.top11_pending = Some(Top11Pending::TrimSurfaces {
            objects,
            partitions,
            original_edges,
            fractions,
            completed,
        });
        result.map(|_| Top11Result::Awaiting {
            label: Top11Action::Trim.label(),
            prompt: "pick portions to remove (Enter to finish)",
        })
    }

    fn apply_trim_face(
        &mut self,
        target: ObjectId,
        mut partition: Option<om9_core::brep::Brep>,
        merge_regions: bool,
        original_edges: Option<om9_core::brep::Brep>,
        mut objects: Vec<ObjectId>,
        mut fractions: Vec<(ObjectId, Vec<f64>)>,
        mut completed: usize,
        point: Point3,
        picked: Option<ObjectId>,
    ) -> Result<Top11Result> {
        if picked.is_some_and(|id| id != target) || (picked.is_none() && partition.is_none()) {
            return self.apply_trim_face_curve(
                target,
                partition,
                merge_regions,
                original_edges,
                objects,
                fractions,
                completed,
                point,
                picked,
            );
        }
        let result: Result<(
            Option<om9_core::brep::Brep>,
            Vec<ObjectId>,
            Vec<(ObjectId, Vec<f64>)>,
        )> = (|| {
            if partition.is_none() || picked.is_some_and(|id| id != target) {
                return Err(Error::Invalid("pick an intersecting trim face"));
            }
            if picked.is_none() {
                let tolerance = self.document().effective_tolerance()?.absolute_mm;
                for (id, _) in &fractions {
                    let entity = self
                        .document()
                        .entities
                        .get(id)
                        .ok_or(Error::Missing("trim curve"))?;
                    let Geometry::Curve(curve) = &entity.geometry else {
                        return Err(Error::Missing("trim curve"));
                    };
                    let (_, _, distance) =
                        om9_core::curve_surface_ops::closest_point_on_placed_curve(
                            curve,
                            entity.placement,
                            point,
                            self.coordinator.limits(),
                        )?;
                    if distance <= tolerance {
                        return Err(Error::Invalid(
                            "ambiguous trim point; pick an object in the viewport",
                        ));
                    }
                }
            }
            #[cfg(not(feature = "occt-brep"))]
            {
                let _ = (target, point, merge_regions);
                Err(Error::Unsupported(
                    "face Trim needs the OCCT kernel feature",
                ))
            }
            #[cfg(feature = "occt-brep")]
            {
                let world = partition
                    .as_ref()
                    .ok_or(Error::Missing("trim face partition"))?;
                let limits = self.coordinator.limits();
                let tolerance = self.document().effective_tolerance()?.absolute_mm;
                let trimmed = om9_kernel::editing::remove_picked_face(
                    &world,
                    point,
                    tolerance,
                    world.display.deflection_mm,
                    limits,
                )
                .map_err(|error| Error::Io(format!("Trim face: {error}")))?;
                let original = self
                    .document()
                    .entities
                    .get(&target)
                    .ok_or(Error::Missing("trim face"))?;
                let committed = if let Some(trimmed) = &trimmed {
                    if merge_regions {
                        Some(
                            om9_kernel::editing::unify_surface_regions(
                                trimmed,
                                trimmed.display.deflection_mm,
                                limits,
                            )
                            .map_err(|error| Error::Io(format!("Trim unify regions: {error}")))?,
                        )
                    } else if let Some(original) = &original_edges {
                        Some(
                            om9_kernel::editing::unify_surface_regions_preserving_edges(
                                trimmed,
                                original,
                                trimmed.display.deflection_mm,
                                limits,
                            )
                            .map_err(|error| Error::Io(format!("Trim preserve edges: {error}")))?,
                        )
                    } else {
                        return Err(Error::Missing("original trim edges"));
                    }
                } else {
                    None
                };
                let local = if let Some(trimmed) = &committed {
                    if original.placement == Transform::identity() {
                        Some(trimmed.clone())
                    } else {
                        let inverse = om9_kernel::transform_ops::inverse(&original.placement)
                            .map_err(|error| Error::Io(format!("Trim face inverse: {error}")))?;
                        Some(
                            om9_kernel::transform_ops::transform_brep(
                                trimmed,
                                &inverse,
                                trimmed.display.deflection_mm,
                                limits,
                            )
                            .map_err(|error| Error::Io(format!("Trim face transform: {error}")))?,
                        )
                    }
                } else {
                    None
                };
                let source = self.document().stamp(target)?;
                let mut next_entities: Vec<Entity> = objects
                    .iter()
                    .filter(|id| **id != target)
                    .map(|id| {
                        self.document()
                            .entities
                            .get(id)
                            .cloned()
                            .ok_or(Error::Missing("trim object"))
                    })
                    .collect::<Result<_>>()?;
                if let Some(local) = &local {
                    let mut replacement = original.clone();
                    replacement.geometry = Geometry::Brep(local.clone());
                    next_entities.push(replacement);
                }
                let next_refs: Vec<_> = next_entities.iter().collect();
                let next_fractions = self.mutual_trim_fractions(&next_refs)?;
                let next_ids = next_entities.iter().map(|entity| entity.id).collect();
                let mutation = match local {
                    Some(local) => Mutation::Geometry {
                        source,
                        geometry: Geometry::Brep(local),
                    },
                    None => Mutation::Delete { source },
                };
                self.commit_top11(
                    Top11Action::Trim,
                    vec![mutation],
                    Some(if trimmed.is_some() {
                        vec![target]
                    } else {
                        vec![]
                    }),
                )?;
                Ok((trimmed, next_ids, next_fractions))
            }
        })();
        let result = result.map(|(remaining, next_ids, next_fractions)| {
            partition = remaining;
            objects = next_ids;
            fractions = next_fractions;
            completed += 1;
        });
        self.top11_pending = Some(Top11Pending::TrimFace {
            target,
            partition,
            merge_regions,
            original_edges,
            objects,
            fractions,
            completed,
        });
        result.map(|_| Top11Result::Awaiting {
            label: Top11Action::Trim.label(),
            prompt: "pick portions to remove (Enter to finish)",
        })
    }

    fn apply_trim_face_curve(
        &mut self,
        face_id: ObjectId,
        mut partition: Option<om9_core::brep::Brep>,
        merge_regions: bool,
        original_edges: Option<om9_core::brep::Brep>,
        mut objects: Vec<ObjectId>,
        mut fractions: Vec<(ObjectId, Vec<f64>)>,
        mut completed: usize,
        point: Point3,
        picked: Option<ObjectId>,
    ) -> Result<Top11Result> {
        let result: Result<_> = (|| {
            if !merge_regions
                && original_edges.is_none()
                && self.document().entities.contains_key(&face_id)
            {
                return Err(Error::Unsupported(
                    "curve edits with a multi-face trim target are not supported",
                ));
            }
            let index = if let Some(id) = picked {
                fractions
                    .iter()
                    .position(|(curve, _)| *curve == id)
                    .ok_or(Error::Invalid("pick an intersecting trim curve"))?
            } else {
                let tolerance = self.document().effective_tolerance()?.absolute_mm;
                let mut hits = Vec::new();
                for (index, (id, _)) in fractions.iter().enumerate() {
                    let entity = self
                        .document()
                        .entities
                        .get(id)
                        .ok_or(Error::Missing("trim curve"))?;
                    let Geometry::Curve(curve) = &entity.geometry else {
                        return Err(Error::Missing("trim curve"));
                    };
                    let (_, _, distance) =
                        om9_core::curve_surface_ops::closest_point_on_placed_curve(
                            curve,
                            entity.placement,
                            point,
                            self.coordinator.limits(),
                        )?;
                    if distance <= tolerance {
                        hits.push(index);
                    }
                }
                if hits.len() != 1 {
                    return Err(Error::Invalid("pick one unambiguous trim curve"));
                }
                hits[0]
            };
            let (curve_id, cuts) = &fractions[index];
            let original = self
                .document()
                .entities
                .get(curve_id)
                .ok_or(Error::Missing("trim curve"))?;
            let Geometry::Curve(curve) = &original.geometry else {
                return Err(Error::Missing("trim curve"));
            };
            let pieces = if cuts.iter().any(|f| *f > 0.0 && *f < 1.0) {
                top11::trim_by_fractions_in_world(
                    curve,
                    cuts,
                    point,
                    original.placement,
                    self.coordinator.limits(),
                )?
            } else {
                Vec::new()
            };
            let mut replacements = Vec::new();
            let mut mutations = Vec::new();
            if pieces.len() == 1 {
                let geometry = Geometry::Curve(pieces[0].clone());
                let mut entity = original.clone();
                entity.geometry = geometry.clone();
                replacements.push(entity);
                mutations.push(Mutation::Geometry {
                    source: self.document().stamp(*curve_id)?,
                    geometry,
                });
            } else {
                mutations.push(Mutation::Delete {
                    source: self.document().stamp(*curve_id)?,
                });
                for piece in pieces {
                    let entity = child_entity(original, Geometry::Curve(piece));
                    mutations.push(Mutation::Insert {
                        document: self.document,
                        entity: entity.clone(),
                    });
                    replacements.push(entity);
                }
            }
            let mut next: Vec<Entity> = objects
                .iter()
                .filter(|id| **id != *curve_id)
                .map(|id| {
                    self.document()
                        .entities
                        .get(id)
                        .cloned()
                        .ok_or(Error::Missing("trim object"))
                })
                .collect::<Result<_>>()?;
            next.extend(replacements.iter().cloned());
            let refs: Vec<_> = next.iter().collect();
            let next_fractions = self.mutual_trim_fractions(&refs)?;
            #[cfg(feature = "occt-brep")]
            let next_partition = if let Some(face) = next.iter().find(|e| e.id == face_id) {
                let cutters: Vec<_> = next
                    .iter()
                    .filter(|e| e.id != face_id)
                    .map(|e| (&e.geometry, e.placement))
                    .collect();
                if cutters.is_empty() {
                    None
                } else {
                    let deflection = match &face.geometry {
                        Geometry::Brep(b) => b.display.deflection_mm,
                        _ => om9_kernel::creation::DEFAULT_DEFLECTION_MM,
                    };
                    match om9_kernel::measurement::split_surface_by_geometry(
                        &face.geometry,
                        face.placement,
                        &cutters,
                        deflection,
                        self.coordinator.limits(),
                    ) {
                        Ok(split) => Some(split.brep),
                        Err(om9_kernel::KernelError::InvalidInput(message))
                            if message == "cutters do not split surface" =>
                        {
                            None
                        }
                        Err(error) => {
                            return Err(Error::Io(format!("Trim face repartition: {error}")));
                        }
                    }
                }
            } else {
                None
            };
            #[cfg(not(feature = "occt-brep"))]
            let next_partition = None;
            let ids = next.iter().map(|e| e.id).collect();
            let selected = replacements.iter().map(|e| e.id).collect();
            Ok((mutations, selected, ids, next_fractions, next_partition))
        })();
        let result = match result {
            Ok((mutations, selected, next_ids, next_fractions, next_partition)) => self
                .commit_top11(Top11Action::Trim, mutations, Some(selected))
                .map(|_| {
                    objects = next_ids;
                    fractions = next_fractions;
                    partition = next_partition;
                    completed += 1;
                }),
            Err(error) => Err(error),
        };
        self.top11_pending = Some(Top11Pending::TrimFace {
            target: face_id,
            partition,
            merge_regions,
            original_edges,
            objects,
            fractions,
            completed,
        });
        result.map(|_| Top11Result::Awaiting {
            label: Top11Action::Trim.label(),
            prompt: "pick portions to remove (Enter to finish)",
        })
    }

    fn apply_trim_mutual(
        &mut self,
        objects: Vec<ObjectId>,
        fractions: Vec<(ObjectId, Vec<f64>)>,
        mut completed: usize,
        point: Point3,
        picked: Option<ObjectId>,
    ) -> Result<Top11Result> {
        let result = (|| {
            let index = if let Some(id) = picked {
                fractions
                    .iter()
                    .position(|(target, _)| *target == id)
                    .ok_or(Error::Invalid("pick an intersecting trim object"))?
            } else {
                let tolerance = self.document().effective_tolerance()?.absolute_mm;
                let mut nearest: Option<(usize, f64)> = None;
                let mut within = 0;
                for (index, (id, _)) in fractions.iter().enumerate() {
                    let entity = self
                        .document()
                        .entities
                        .get(id)
                        .ok_or(Error::Missing("trim curve"))?;
                    let Geometry::Curve(curve) = &entity.geometry else {
                        return Err(Error::Missing("trim curve"));
                    };
                    let (_, _, distance) =
                        om9_core::curve_surface_ops::closest_point_on_placed_curve(
                            curve,
                            entity.placement,
                            point,
                            self.coordinator.limits(),
                        )?;
                    if distance <= tolerance {
                        within += 1;
                    }
                    if nearest.is_none_or(|(_, best)| distance < best) {
                        nearest = Some((index, distance));
                    }
                }
                let (index, distance) =
                    nearest.ok_or(Error::Missing("intersecting trim objects"))?;
                if distance > tolerance {
                    return Err(Error::Invalid(
                        "pick a trim curve within document tolerance",
                    ));
                }
                if within > 1 {
                    return Err(Error::Invalid(
                        "ambiguous trim point; pick an object in the viewport",
                    ));
                }
                index
            };
            let (target, cuts) = &fractions[index];
            let original = self
                .document()
                .entities
                .get(target)
                .ok_or(Error::Missing("trim curve"))?;
            let Geometry::Curve(curve) = &original.geometry else {
                return Err(Error::Missing("trim curve"));
            };
            let has_interior = cuts.iter().any(|f| *f > 0.0 && *f < 1.0);
            let pieces = if has_interior {
                top11::trim_by_fractions_in_world(
                    curve,
                    cuts,
                    point,
                    original.placement,
                    self.coordinator.limits(),
                )?
            } else {
                Vec::new()
            };
            let mut replacements = Vec::new();
            let mut mutations = Vec::new();
            if pieces.len() == 1 {
                let geometry = Geometry::Curve(pieces[0].clone());
                let mut replacement = original.clone();
                replacement.geometry = geometry.clone();
                replacements.push(replacement);
                mutations.push(Mutation::Geometry {
                    source: self.document().stamp(*target)?,
                    geometry,
                });
            } else {
                mutations.push(Mutation::Delete {
                    source: self.document().stamp(*target)?,
                });
                for piece in pieces {
                    let entity = child_entity(original, Geometry::Curve(piece));
                    mutations.push(Mutation::Insert {
                        document: self.document,
                        entity: entity.clone(),
                    });
                    replacements.push(entity);
                }
            }
            let mut next = Vec::new();
            for id in &objects {
                if *id == *target {
                    next.extend(replacements.iter());
                } else {
                    next.push(
                        self.document()
                            .entities
                            .get(id)
                            .ok_or(Error::Missing("trim object"))?,
                    );
                }
            }
            let next_fractions = self.mutual_trim_fractions(&next)?;
            Ok((
                mutations,
                replacements.iter().map(|e| e.id).collect::<Vec<_>>(),
                next.iter().map(|e| e.id).collect::<Vec<_>>(),
                next_fractions,
            ))
        })();
        let mut next_objects = objects;
        let mut next_fractions = fractions;
        let result = match result {
            Ok((mutations, selected, prospective_objects, prospective_fractions)) => self
                .commit_top11(Top11Action::Trim, mutations, Some(selected))
                .map(|_| {
                    next_objects = prospective_objects;
                    next_fractions = prospective_fractions;
                    completed += 1;
                    Top11Result::Awaiting {
                        label: Top11Action::Trim.label(),
                        prompt: "pick portions to remove (Enter to finish)",
                    }
                }),
            Err(error) => Err(error),
        };
        self.top11_pending = Some(Top11Pending::TrimMutual {
            objects: next_objects,
            fractions: next_fractions,
            completed,
        });
        result
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
            Top11Pending::SelectObjects {
                action,
                split_point,
                explode_blocks,
            } => {
                if self.selection.is_empty() {
                    return Some(Err(Error::Missing("selected objects")));
                }
                if action == Top11Action::Trim && self.selection.len() < 2 {
                    return Some(Err(Error::Invalid("select at least two curves for Trim")));
                }
                self.top11_pending = None;
                let result = self.top11_invoke_route(action,explode_blocks).and_then(|result| {
                    if split_point {
                        self.top11_split_point_mode()
                    } else {
                        Ok(result)
                    }
                });
                if result.is_err() {
                    self.top11_pending = Some(Top11Pending::SelectObjects {
                        action,
                        split_point,
                        explode_blocks,
                    });
                }
                Some(result)
            }
            Top11Pending::Duplicate { copies, .. } => {
                self.top11_pending = None;
                if copies == 0 { return Some(Ok(Top11Result::NoChange { label: "Duplicate" })); }
                Some(Ok(Top11Result::Completed {
                    label: "Duplicate",
                    objects: copies,
                }))
            }
            Top11Pending::Split { target, cutters } => Some(self.finish_split(target, cutters)),
            Top11Pending::SplitMany {
                targets,
                cutters,
                shrink,
                ..
            } => Some(self.finish_split_many(targets, cutters, shrink.unwrap_or(false))),
            Top11Pending::SplitPoint {
                target,
                closed: true,
                fractions,
            } => Some(self.finish_split_points(target, &fractions)),
            Top11Pending::Trim {
                target,
                cutters,
                fractions: None,
            } => Some(self.finish_trim_cutters(target, cutters)),
            Top11Pending::TrimMany {
                targets,
                cutters,
                fractions: None,
                completed,
            } => Some(self.finish_trim_many_cutters(targets, cutters, completed)),
            Top11Pending::TrimMany {
                fractions: Some(_),
                completed,
                ..
            } => {
                self.top11_pending = None;
                if completed == 0 { return Some(Ok(Top11Result::NoChange { label: "Trim" })); }
                Some(Ok(Top11Result::Completed {
                    label: Top11Action::Trim.label(),
                    objects: completed,
                }))
            }
            Top11Pending::TrimMutual { completed, .. } => {
                self.top11_pending = None;
                if completed == 0 { return Some(Ok(Top11Result::NoChange { label: "Trim" })); }
                Some(Ok(Top11Result::Completed {
                    label: Top11Action::Trim.label(),
                    objects: completed,
                }))
            }
            Top11Pending::TrimFace { completed, .. } => {
                self.top11_pending = None;
                if completed == 0 { return Some(Ok(Top11Result::NoChange { label: "Trim" })); }
                Some(Ok(Top11Result::Completed {
                    label: Top11Action::Trim.label(),
                    objects: completed,
                }))
            }
            Top11Pending::TrimSurfaces { completed, .. } => {
                self.top11_pending = None;
                if completed == 0 { return Some(Ok(Top11Result::NoChange { label: "Trim" })); }
                Some(Ok(Top11Result::Completed {
                    label: Top11Action::Trim.label(),
                    objects: completed,
                }))
            }
            _ => None,
        }
    }

    fn finish_split(&mut self, target: ObjectId, cutters: Vec<ObjectId>) -> Result<Top11Result> {
        if cutters.is_empty() {
            return Err(Error::Missing("cutting curve"));
        }
        let curve = match &self.document().entities[&target].geometry {
            Geometry::Curve(curve) => curve.clone(),
            _ => return Err(Error::Missing("split curve")),
        };
        let closed = top11::curve_closed_for_cut(&curve, self.coordinator.limits())?;
        let fractions = self.split_cutter_fractions(target, &cutters, closed)?;
        let pieces = if closed { top11::split_closed_at_fractions(&curve, &fractions, self.coordinator.limits())? } else { top11::split_at_fractions(&curve, &fractions, self.coordinator.limits())? };
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
        shrink: bool,
    ) -> Result<Top11Result> {
        #[cfg(not(feature = "occt-brep"))]
        let _ = shrink;
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
            #[cfg(feature = "occt-brep")]
            if matches!(entity.geometry, Geometry::Surface(_) | Geometry::Brep(_)) {
                let limits = self.coordinator.limits();
                let deflection = match &entity.geometry {
                    Geometry::Brep(brep) => brep.display.deflection_mm,
                    _ => om9_kernel::creation::DEFAULT_DEFLECTION_MM,
                };
                let tools: Vec<_> = cutters
                    .iter()
                    .map(|id| {
                        let cutter = self
                            .document()
                            .entities
                            .get(id)
                            .ok_or(Error::Missing("cutting object"))?;
                        Ok((&cutter.geometry, cutter.placement))
                    })
                    .collect::<Result<_>>()?;
                let result = match &entity.geometry {
                    Geometry::Brep(_) => om9_kernel::measurement::split_boundary_by_geometry(
                        &entity.geometry,
                        entity.placement,
                        &tools,
                        deflection,
                        limits,
                    ),
                    _ => om9_kernel::measurement::split_surface_by_geometry(
                        &entity.geometry,
                        entity.placement,
                        &tools,
                        deflection,
                        limits,
                    )
                    .and_then(|split| {
                        (0..split.pieces)
                            .map(|index| {
                                om9_kernel::editing::extract_face(
                                    &split.brep,
                                    index,
                                    deflection,
                                    limits,
                                )
                                .map_err(|error| {
                                    om9_kernel::KernelError::Failure(error.to_string())
                                })
                            })
                            .collect()
                    }),
                };
                let pieces = match result {
                    Ok(pieces) => pieces,
                    Err(om9_kernel::KernelError::InvalidInput(message))
                        if message == "cutters do not split surface" =>
                    {
                        selected.push(target);
                        continue;
                    }
                    Err(error) => return Err(Error::Io(format!("Split surface: {error}"))),
                };
                let inverse = om9_kernel::transform_ops::inverse(&entity.placement)
                    .map_err(|error| Error::Io(format!("Split surface inverse: {error}")))?;
                mutations.push(Mutation::Delete {
                    source: self.document().stamp(target)?,
                });
                for face in pieces {
                    let face = if shrink {
                        om9_kernel::measurement::shrink_face(&face, deflection, limits)
                            .map_err(|error| Error::Io(format!("Split surface shrink: {error}")))?
                    } else {
                        face
                    };
                    let local = if entity.placement == Transform::identity() {
                        face
                    } else {
                        om9_kernel::transform_ops::transform_brep(
                            &face, &inverse, deflection, limits,
                        )
                        .map_err(|error| Error::Io(format!("Split surface transform: {error}")))?
                    };
                    let child = child_entity(entity, Geometry::Brep(local));
                    selected.push(child.id);
                    mutations.push(Mutation::Insert {
                        document: self.document,
                        entity: child,
                    });
                }
                changed = true;
                continue;
            }
            let Geometry::Curve(curve) = &entity.geometry else {
                return Err(Error::Invalid("split target must be a curve or surface"));
            };
            let closed = top11::curve_closed_for_cut(curve, self.coordinator.limits())?;
            let fractions = self.split_cutter_fractions(target, &cutters, closed)?;
            if fractions.is_empty() {
                selected.push(target);
                continue;
            }
            let pieces = if closed { top11::split_closed_at_fractions(curve, &fractions, self.coordinator.limits())? } else { top11::split_at_fractions(curve, &fractions, self.coordinator.limits())? };
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
        let Geometry::Curve(curve) = &entity.geometry else {
            return Err(Error::Unsupported("split by point needs a curve"));
        };
        if let Curve::Nurbs(n) = curve && n.seam.is_some() {
            let mut base=n.clone();base.seam=None;
            if !om9_core::curve_ops::is_closed(&Curve::Nurbs(base),1e-9,self.coordinator.limits())? {
                return Err(Error::Invalid("NURBS seam needs a closed curve"));
            }
        }
        if let Curve::Nurbs(n) = curve && n.periodic {
            n.certify_repeated_periodic_basis(self.coordinator.limits())?;
        }
        let closed = top11::curve_closed_for_cut(curve, self.coordinator.limits())?;
        self.top11_pending = Some(Top11Pending::SplitPoint {
            target,
            closed,
            fractions: Vec::new(),
        });
        Ok(Top11Result::Awaiting {
            label: Top11Action::Split.label(),
            prompt: self.top11_prompt().unwrap(),
        })
    }

    fn finish_split_points(&mut self, target: ObjectId, fractions: &[f64]) -> Result<Top11Result> {
        if fractions.len() < 2 {
            return Err(Error::Invalid(
                "closed curve needs at least two distinct points",
            ));
        }
        let Geometry::Curve(curve) = &self
            .document()
            .entities
            .get(&target)
            .ok_or(Error::Missing("split curve"))?
            .geometry
        else {
            return Err(Error::Missing("split curve"));
        };
        let pieces = top11::split_closed_at_fractions(curve, fractions, self.coordinator.limits())?;
        let count = pieces.len();
        self.commit_cut(Top11Action::Split, target, pieces)?;
        self.top11_pending = None;
        Ok(Top11Result::Completed {
            label: Top11Action::Split.label(),
            objects: count,
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
        let entity = self.document().entities.get(&target).ok_or(Error::Missing("target curve"))?;
        let Geometry::Curve(curve) = &entity.geometry else { return Err(Error::Missing("target curve")); };
        let closed = top11::curve_closed_for_cut(curve, self.coordinator.limits())?;
        let fractions = self.split_cutter_fractions(target, &cutters, closed)?;
        if !fractions.iter().any(|f| *f > 0.0 && *f < 1.0) { return Err(Error::Invalid("cutting objects have no interior intersection")); }
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

    fn finish_trim_many_cutters(
        &mut self,
        targets: Vec<ObjectId>,
        cutters: Vec<ObjectId>,
        completed: usize,
    ) -> Result<Top11Result> {
        if cutters.is_empty() {
            return Err(Error::Missing("cutting curve"));
        }
        let mut results = Vec::new();
        for target in &targets {
            let entity = self.document().entities.get(target).ok_or(Error::Missing("target curve"))?;
            let Geometry::Curve(curve) = &entity.geometry else { return Err(Error::Missing("target curve")); };
            let closed = top11::curve_closed_for_cut(curve, self.coordinator.limits())?;
            let fractions = self.split_cutter_fractions(*target, &cutters, closed)?;
            if fractions.iter().any(|f| *f > 0.0 && *f < 1.0) {
                results.push((*target, fractions));
            }
        }
        if results.is_empty() {
            return Err(Error::Invalid("cutting objects have no intersection"));
        }
        self.top11_pending = Some(Top11Pending::TrimMany {
            targets,
            cutters,
            fractions: Some(results),
            completed,
        });
        Ok(Top11Result::Awaiting {
            label: Top11Action::Trim.label(),
            prompt: "pick portions to remove (Enter to finish)",
        })
    }

    /// Run an immediate Top 11 action or start its point prompt.
    pub fn top11_invoke(&mut self, action: Top11Action) -> Result<Top11Result> {
        self.top11_invoke_route(action,false)
    }

    fn top11_invoke_route(&mut self, action: Top11Action, explode_blocks: bool) -> Result<Top11Result> {
        if !self.is_running()
            && action != Top11Action::RingRail
            && (self.selection.is_empty()
                || (action == Top11Action::Trim && self.selection.len() < 2))
        {
            self.top11_pending = Some(Top11Pending::SelectObjects {
                action,
                split_point: false,
                explode_blocks,
            });
            return Ok(Top11Result::Awaiting {
                label: action.label(),
                prompt: "select objects (Enter to continue)",
            });
        }
        let context = self.top11_context();
        let availability = if matches!(
            action,
            Top11Action::RingRail
                | Top11Action::Explode
                | Top11Action::Join
                | Top11Action::Split
                | Top11Action::Trim
        ) {
            self.top11_availability()
                .into_iter()
                .find(|item| item.action == action)
                .ok_or(Error::Missing("top11 ring rail"))?
        } else {
            top11::availability(action, &context)
        };
        let command_blocks = explode_blocks && action==Top11Action::Explode && !self.is_running()
            && self.selection.iter().all(|id| self.document().entities.get(id).is_some_and(|e| matches!(e.geometry,Geometry::Curve(_) | Geometry::Mesh(_) | Geometry::Brep(_) | Geometry::Instance(_))));
        if !availability.enabled && !command_blocks {
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
                let originals: Vec<ObjectId> = self.selection.iter().copied().filter(|id| {
                    explode_blocks || !matches!(self.document().entities.get(id).map(|e| &e.geometry),Some(Geometry::Instance(_)))
                }).collect();
                let mut pieces = Vec::new();
                let mut replacements = std::collections::BTreeMap::new();
                for id in &originals {
                    let first=pieces.len();
                    let entity = self
                        .document()
                        .entities
                        .get(id)
                        .ok_or(Error::Missing("object"))?;
                    match &entity.geometry {
                        Geometry::Curve(_) | Geometry::Mesh(_) => pieces.extend(top11::explode_entities(
                            self.document(),
                            &BTreeSet::from([*id]),
                            self.coordinator.limits(),
                        )?),
                        Geometry::Brep(_) | Geometry::NativeBrep(_) => {
                            #[cfg(feature = "occt-brep")]
                            {
                                let limits = self.coordinator.limits();
                                let converted;
                                let brep = match &entity.geometry {
                                    Geometry::Brep(brep) => brep,
                                    Geometry::NativeBrep(native) => {
                                        converted = om9_kernel::brep::from_3dm(
                                            &native.payload,
                                            self.document().effective_tolerance()?.absolute_mm,
                                            om9_kernel::creation::DEFAULT_DEFLECTION_MM,
                                            limits,
                                        ).map_err(|error| Error::Io(format!("retained BRep explode conversion: {error}")))?;
                                        &converted
                                    }
                                    _ => unreachable!(),
                                };
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
                                return Err(Error::Unsupported(
                                    "BRep explode needs the OCCT kernel feature",
                                ));
                            }
                        }
                        Geometry::Instance(definition) if explode_blocks => {
                            let members=&self.document().definitions.get(definition).ok_or(Error::Missing("instance definition"))?.members;
                            if members.is_empty(){return Err(Error::Unsupported("empty instance definition"));}
                            let count=pieces.len().checked_add(members.len()).ok_or(Error::Overflow("exploded block outputs"))?;
                            if u64::try_from(count).map_err(|_| Error::Overflow("exploded block outputs"))?>self.coordinator.limits().max_outputs {
                                return Err(Error::Budget("outputs"));
                            }
                            pieces.try_reserve(members.len()).map_err(|_| Error::Budget("exploded block outputs"))?;
                            for member in members {
                                let mut child=child_entity(member,member.geometry.clone());
                                child.placement=entity.placement.compose(member.placement)?;
                                pieces.push(child);
                            }
                        }
                        _ => return Err(Error::Unsupported("explode needs curves, meshes or BReps")),
                    }
                    replacements.insert(*id,pieces[first..].iter().map(|piece|piece.id).collect::<Vec<_>>());
                }
                let ids: Vec<ObjectId> = pieces.iter().map(|entity| entity.id).collect();
                let output_count=ids.len();
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
                for group in self.document().groups.values() {
                    if group.members.iter().any(|id| replacements.contains_key(id)) {
                        let mut updated=group.clone();
                        updated.members=group.members.iter().flat_map(|id| {
                            replacements.get(id).cloned().unwrap_or_else(||vec![*id])
                        }).collect();
                        updated.revision=updated.revision.checked_add(1).ok_or(Error::Overflow("group revision"))?;
                        mutations.push(Mutation::GroupReplace {
                            document:self.document,
                            expected_revision:self.document().revision,
                            group:Box::new(updated),
                        });
                    }
                }
                self.commit_top11(action, mutations, Some(ids))?;
                Ok(Top11Result::Completed {
                    label: action.label(),
                    objects: output_count,
                })
            }
            Top11Action::Join => {
                let originals: Vec<ObjectId> = self.selection.iter().copied().collect();
                let joined = if context.selected >= 2 && context.breps + context.surfaces == context.selected
                    && (context.surfaces > 0 || context.breps > 2) {
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
                        let result = om9_kernel::editing::sew_preserving_surfaces(
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
                        self.document().effective_tolerance()?.absolute_mm,
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
                if self.selection.len() > 1
                    || self.selection.iter().any(|id| {
                        !matches!(self.document().entities[id].geometry, Geometry::Curve(_))
                    })
                {
                    let isocurve_allowed = if self.selection.len() == 1 {
                        let entity =
                            &self.document().entities[self.selection.iter().next().unwrap()];
                        match &entity.geometry {
                            Geometry::Surface(_) => true,
                            #[cfg(feature = "occt-brep")]
                            Geometry::Brep(brep) => {
                                om9_kernel::creation::measure(brep, self.coordinator.limits())
                                    .is_ok_and(|m| m.faces == 1 && m.solids == 0)
                            }
                            _ => false,
                        }
                    } else {
                        false
                    };
                    let mut has_surface = false;
                    let mut shrink_supported = true;
                    for id in &self.selection {
                        match &self.document().entities[id].geometry {
                            Geometry::Curve(_) => {}
                            Geometry::Surface(surface) => {
                                has_surface = true;
                                shrink_supported &= !surface.periodic.iter().any(|p| *p);
                            }
                            #[cfg(feature = "occt-brep")]
                            Geometry::Brep(brep) => {
                                has_surface = true;
                                if shrink_supported {
                                    shrink_supported &=
                                        om9_kernel::measurement::face_shrink_supported(
                                            brep,
                                            self.coordinator.limits(),
                                        )
                                        .map_err(
                                            |error| {
                                                Error::Io(format!(
                                                    "Split shrink capability: {error}"
                                                ))
                                            },
                                        )?;
                                }
                            }
                            _ => shrink_supported = false,
                        }
                    }
                    self.top11_pending = Some(Top11Pending::SplitMany {
                        targets: self.selection.iter().copied().collect(),
                        cutters: Vec::new(),
                        isocurve_allowed,
                        shrink: (has_surface && shrink_supported).then_some(false),
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
            Top11Action::Trim => self.start_trim_mutual(),
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
                self.grip_selection.clear();
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

    fn split_isocurve_at_point(
        &mut self,
        target: ObjectId,
        direction: SplitIsoDirection,
        point: Point3,
        enforce_tolerance: bool,
        shrink: bool,
    ) -> Result<Top11Result> {
        let entity = self
            .document()
            .entities
            .get(&target)
            .ok_or(Error::Missing("split surface"))?;
        #[cfg(feature = "occt-brep")]
        if let Geometry::Brep(brep) = &entity.geometry {
            let limits = self.coordinator.limits();
            let deflection = brep.display.deflection_mm;
            let tolerance = if enforce_tolerance {
                Some(self.document().effective_tolerance()?.absolute_mm)
            } else {
                None
            };
            let axis = match direction {
                SplitIsoDirection::U => 0,
                SplitIsoDirection::V => 1,
                SplitIsoDirection::Both => 2,
            };
            let partition = om9_kernel::measurement::split_face_isocurve(
                &entity.geometry,
                entity.placement,
                point,
                axis,
                tolerance,
                deflection,
                limits,
            )
            .map_err(|error| Error::Io(format!("Split isocurve: {error}")))?;
            let inverse = om9_kernel::transform_ops::inverse(&entity.placement)
                .map_err(|error| Error::Io(format!("Split isocurve inverse: {error}")))?;
            let mut mutations = vec![Mutation::Delete {
                source: self.document().stamp(target)?,
            }];
            let mut ids = Vec::new();
            for index in 0..partition.pieces {
                let face =
                    om9_kernel::editing::extract_face(&partition.brep, index, deflection, limits)
                        .map_err(|error| Error::Io(format!("Split isocurve face: {error}")))?;
                let face = if shrink {
                    om9_kernel::measurement::shrink_face(&face, deflection, limits)
                        .map_err(|error| Error::Io(format!("Split isocurve shrink: {error}")))?
                } else {
                    face
                };
                let local = if entity.placement == Transform::identity() {
                    face
                } else {
                    om9_kernel::transform_ops::transform_brep(&face, &inverse, deflection, limits)
                        .map_err(|error| Error::Io(format!("Split isocurve transform: {error}")))?
                };
                let child = child_entity(entity, Geometry::Brep(local));
                ids.push(child.id);
                mutations.push(Mutation::Insert {
                    document: self.document,
                    entity: child,
                });
            }
            let count = ids.len();
            self.commit_top11(Top11Action::Split, mutations, Some(ids))?;
            return Ok(Top11Result::Completed {
                label: Top11Action::Split.label(),
                objects: count,
            });
        }
        let Geometry::Surface(surface) = &entity.geometry else {
            return Err(Error::Invalid("split isocurve requires a surface"));
        };
        let limits = self.coordinator.limits();
        let mut world = surface.clone();
        for pole in &mut world.poles {
            *pole = entity.placement.point(*pole)?;
        }
        let projection =
            om9_core::curve_surface_ops::closest_point_on_surface(&world, point, limits)?;
        if enforce_tolerance
            && projection.distance > self.document().effective_tolerance()?.absolute_mm
        {
            return Err(Error::Invalid(
                "point must lie on the surface within document tolerance",
            ));
        }
        let uv = projection.parameters;
        let split_u = matches!(direction, SplitIsoDirection::V | SplitIsoDirection::Both);
        let split_v = matches!(direction, SplitIsoDirection::U | SplitIsoDirection::Both);
        for (axis, active) in [(0, split_u), (1, split_v)] {
            let [low, high] = surface.domain[axis];
            let epsilon = (high - low).abs() * 1e-9;
            if active && (uv[axis] <= low + epsilon || uv[axis] >= high - epsilon) {
                return Err(Error::Invalid("isocurve must be inside the surface domain"));
            }
        }
        let mut domains = vec![surface.domain];
        for (axis, active) in [(0, split_u), (1, split_v)] {
            if active {
                domains = domains
                    .into_iter()
                    .flat_map(|domain| {
                        let mut first = domain;
                        let mut second = domain;
                        first[axis][1] = uv[axis];
                        second[axis][0] = uv[axis];
                        [first, second]
                    })
                    .collect();
            }
        }
        let mut mutations = vec![Mutation::Delete {
            source: self.document().stamp(target)?,
        }];
        let mut ids = Vec::new();
        for domain in domains {
            let mut piece = if shrink {
                om9_core::surface_ops::shrink_surface_domain_exact(surface, domain, limits)?
            } else {
                surface.clone()
            };
            piece.domain = domain;
            piece.validate(limits)?;
            let child = child_entity(entity, Geometry::Surface(piece));
            ids.push(child.id);
            mutations.push(Mutation::Insert {
                document: self.document,
                entity: child,
            });
        }
        let count = ids.len();
        self.commit_top11(Top11Action::Split, mutations, Some(ids))?;
        Ok(Top11Result::Completed {
            label: Top11Action::Split.label(),
            objects: count,
        })
    }

    /// Feed one world point to the pending Top 11 command.
    pub fn top11_accept_point(&mut self, point: Point3) -> Result<Top11Result> {
        let retryable = self
            .top11_pending
            .as_ref()
            .filter(|pending| {
                matches!(
                    pending,
                    Top11Pending::Duplicate { .. }
                        | Top11Pending::Move { .. }
                        | Top11Pending::Mirror { .. }
                        | Top11Pending::Rotate { .. }
                )
            })
            .cloned();
        let revision = self.document().revision;
        let result = self.top11_accept_point_inner(point);
        if result.is_err() && self.top11_pending.is_none() && self.document().revision == revision {
            self.top11_pending = retryable;
        }
        result
    }

    fn top11_accept_point_inner(&mut self, point: Point3) -> Result<Top11Result> {
        finite(point)?;
        let pending = self
            .top11_pending
            .take()
            .ok_or(Error::Missing("top 11 point command"))?;
        let translation = translation_candidate(&pending, point, self.input_plane())?;
        match pending {
            Top11Pending::SplitIso {
                target,
                direction,
                shrink,
                shrink_available,
            } => {
                let result = self.split_isocurve_at_point(target, direction, point, true, shrink);
                if result.is_err() {
                    self.top11_pending = Some(Top11Pending::SplitIso {
                        target,
                        direction,
                        shrink,
                        shrink_available,
                    });
                }
                result
            }
            Top11Pending::SelectObjects {
                action,
                split_point,
                explode_blocks,
            } => {
                self.top11_pending = Some(Top11Pending::SelectObjects {
                    action,
                    split_point,
                    explode_blocks,
                });
                Err(Error::Invalid("select an object"))
            }
            Top11Pending::SplitPoint {
                target,
                closed,
                mut fractions,
            } => {
                let result = (|| {
                    let entity = self
                        .document()
                        .entities
                        .get(&target)
                        .ok_or(Error::Missing("split curve"))?;
                    let Geometry::Curve(curve) = &entity.geometry else {
                        return Err(Error::Missing("split curve"));
                    };
                    let (mut fraction, closest, measured) =
                        om9_core::curve_surface_ops::closest_point_on_placed_curve(
                            curve,
                            entity.placement,
                            point,
                            self.coordinator.limits(),
                        )?;
                    let tolerance = self.document().effective_tolerance()?.absolute_mm;
                    if measured > tolerance {
                        return Err(Error::Invalid(
                            "point must lie on the curve within document tolerance",
                        ));
                    }
                    if closed {
                        if fraction >= 1.0 {
                            fraction = 0.0;
                        }
                        for previous in &fractions {
                            let p = om9_core::curve_ops::point_at(
                                curve,
                                *previous,
                                self.coordinator.limits(),
                            )?;
                            if norm(vector(entity.placement.point(p)?, closest)) <= tolerance {
                                return Err(Error::Invalid("split point already selected"));
                            }
                        }
                        fractions.push(fraction);
                        return Ok(Top11Result::Awaiting {
                            label: Top11Action::Split.label(),
                            prompt: "pick points on the closed curve (Enter to split)",
                        });
                    }
                    if !(0.0 < fraction && fraction < 1.0) {
                        return Err(Error::Invalid("point must lie inside the curve"));
                    }
                    let pieces =
                        top11::split_at_fractions(curve, &[fraction], self.coordinator.limits())?;
                    let count = pieces.len();
                    self.commit_cut(Top11Action::Split, target, pieces)?;
                    Ok(Top11Result::Completed {
                        label: Top11Action::Split.label(),
                        objects: count,
                    })
                })();
                if result.is_err() || closed {
                    self.top11_pending = Some(Top11Pending::SplitPoint {
                        target,
                        closed,
                        fractions,
                    });
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
                last_delta: _,
                last_destination: _,
            } => {
                let (placement_delta,delta,destination)=translation.ok_or(Error::Invalid("duplicate candidate"))?;
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
                    last_destination: Some(destination),
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
            Top11Pending::SplitMany {
                targets,
                cutters,
                isocurve_allowed,
                shrink,
            } => {
                self.top11_pending = Some(Top11Pending::SplitMany {
                    targets,
                    cutters,
                    isocurve_allowed,
                    shrink,
                });
                Err(Error::Invalid("select a cutting object"))
            }
            Top11Pending::TrimMany {
                targets,
                cutters,
                fractions: None,
                completed,
            } => {
                self.top11_pending = Some(Top11Pending::TrimMany {
                    targets,
                    cutters,
                    fractions: None,
                    completed,
                });
                Err(Error::Invalid("trim expects Enter to continue"))
            }
            Top11Pending::TrimMany {
                targets,
                cutters,
                fractions: Some(fractions),
                completed,
            } => self.apply_trim_many(targets, cutters, fractions, completed, point, None),
            Top11Pending::TrimMutual {
                objects,
                fractions,
                completed,
            } => self.apply_trim_mutual(objects, fractions, completed, point, None),
            Top11Pending::TrimFace {
                target,
                partition,
                merge_regions,
                original_edges,
                objects,
                fractions,
                completed,
            } => self.apply_trim_face(
                target,
                partition,
                merge_regions,
                original_edges,
                objects,
                fractions,
                completed,
                point,
                None,
            ),
            Top11Pending::TrimSurfaces {
                objects,
                partitions,
                original_edges,
                fractions,
                completed,
            } => self.apply_trim_surfaces(
                objects,
                partitions,
                original_edges,
                fractions,
                completed,
                point,
                None,
            ),
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
                let pieces = match top11::trim_by_fractions_in_world(
                    &match &self.document().entities[&target].geometry {
                        Geometry::Curve(curve) => curve.clone(),
                        _ => return Err(Error::Missing("trim curve")),
                    },
                    &fractions,
                    point,
                    self.document().entities[&target].placement,
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
                base: Some(_),
                vertical: _,
            } => {
                let (delta,_,_)=translation.ok_or(Error::Invalid("move candidate"))?;
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
                let plane = self.input_plane();
                let axis = unit(cross(plane.x, plane.y))?;
                let reference_vector = vector(center, point);
                signed_degrees(axis, reference_vector, reference_vector)?;
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
                let (origin, normal) = mirror_plane(&points[..points.len()-1], point, three_point, self.input_plane())?;
                let (copies, replacements, placements) = self.mirror_entities(&ids, origin, normal, copy)?;
                let mut mutations = Vec::with_capacity(replacements.len() + copies.len() + placements.len());
                for (id, placement) in placements { mutations.push(Mutation::Placement { source:self.document().stamp(id)?,placement }); }
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
            Top11Pending::SelectObjects {
                action,
                split_point,
                explode_blocks,
            } => {
                let mut ids: Vec<ObjectId> = self.selection.iter().copied().collect();
                if !ids.contains(&object) {
                    ids.push(object);
                }
                let result = self.select_objects(&ids);
                self.top11_pending = Some(Top11Pending::SelectObjects {
                    action,
                    split_point,
                    explode_blocks,
                });
                result?;
                Ok(Top11Result::Awaiting {
                    label: action.label(),
                    prompt: self.top11_prompt().unwrap(),
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
                isocurve_allowed,
                shrink,
            } => {
                let validation = (|| {
                    if targets.contains(&object) {
                        return Err(Error::Invalid("cutting curve is a target"));
                    }
                    if cutters.contains(&object) {
                        return Err(Error::Invalid("cutting curve already picked"));
                    }
                    #[cfg(feature = "occt-brep")]
                    if targets.iter().any(|target| {
                        matches!(
                            self.document().entities[target].geometry,
                            Geometry::Surface(_) | Geometry::Brep(_)
                        )
                    }) {
                        let cutter = self
                            .document()
                            .entities
                            .get(&object)
                            .ok_or(Error::Missing("cutting object"))?;
                        return if matches!(
                            cutter.geometry,
                            Geometry::Curve(_) | Geometry::Surface(_) | Geometry::Brep(_)
                        ) {
                            Ok(())
                        } else {
                            Err(Error::Unsupported(
                                "cutting object must be a curve, surface or B-Rep",
                            ))
                        };
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
                self.top11_pending = Some(Top11Pending::SplitMany {
                    targets,
                    cutters,
                    isocurve_allowed,
                    shrink,
                });
                validation?;
                Ok(Top11Result::Awaiting {
                    label: Top11Action::Split.label(),
                    prompt: "pick more cutting curves (Enter to execute)",
                })
            }
            Top11Pending::TrimMany {
                targets,
                mut cutters,
                fractions: None,
                completed,
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
                        match self.trim_cutter_fractions(*target, object) {
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
                self.top11_pending = Some(Top11Pending::TrimMany {
                    targets,
                    cutters,
                    fractions: None,
                    completed,
                });
                validation?;
                Ok(Top11Result::Awaiting {
                    label: Top11Action::Trim.label(),
                    prompt: "pick more cutting curves (Enter to continue)",
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
                if let Err(error) = self.trim_cutter_fractions(target, object) {
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
                isocurve_allowed,
                shrink,
            } => {
                let Some(index) = cutters.iter().position(|id| *id == object) else {
                    return Err(Error::Missing("cutting object"));
                };
                cutters.remove(index);
                self.top11_pending = Some(Top11Pending::SplitMany {
                    targets,
                    cutters,
                    isocurve_allowed,
                    shrink,
                });
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
            Top11Pending::TrimMany {
                targets,
                mut cutters,
                fractions: None,
                completed,
            } => {
                let Some(index) = cutters.iter().position(|id| *id == object) else {
                    return Err(Error::Missing("cutting object"));
                };
                cutters.remove(index);
                self.top11_pending = Some(Top11Pending::TrimMany {
                    targets,
                    cutters,
                    fractions: None,
                    completed,
                });
            }
            _ => return Err(Error::Invalid("top 11 command does not select objects")),
        }
        Ok(Top11Result::Awaiting {
            label: action.label(),
            prompt: self.top11_prompt().unwrap_or("select objects"),
        })
    }

    /// Interior source-curve fractions from world-space cutter intersections.
    fn cutter_fractions(&self, target: ObjectId, cutter: ObjectId) -> Result<Vec<f64>> {
        let target_entity = self
            .document()
            .entities
            .get(&target)
            .ok_or(Error::Missing("target curve"))?;
        self.cutter_fractions_for_entity(target_entity, cutter)
    }

    fn trim_cutter_fractions(&self, target: ObjectId, cutter: ObjectId) -> Result<Vec<f64>> {
        let target = self.document().entities.get(&target).ok_or(Error::Missing("target curve"))?;
        let cutter = self.document().entities.get(&cutter).ok_or(Error::Missing("cutting object"))?;
        let Geometry::Curve(curve) = &target.geometry else { return Err(Error::Missing("target curve")); };
        let closed = top11::curve_closed_for_cut(curve, self.coordinator.limits())?;
        self.cutter_fractions_between(target, cutter, closed)
    }

    fn cutter_fractions_for_entity(
        &self,
        target_entity: &Entity,
        cutter: ObjectId,
    ) -> Result<Vec<f64>> {
        let cutter_entity = self
            .document()
            .entities
            .get(&cutter)
            .ok_or(Error::Missing("cutting object"))?;
        let Geometry::Curve(curve) = &target_entity.geometry else {
            return Err(Error::Missing("target curve"));
        };
        let closed = top11::curve_closed_for_cut(curve, self.coordinator.limits())?;
        self.cutter_fractions_between(target_entity, cutter_entity, closed)
    }

    fn cutter_fractions_between(
        &self,
        target_entity: &Entity,
        cutter_entity: &Entity,
        include_endpoints: bool,
    ) -> Result<Vec<f64>> {
        let Geometry::Curve(target_curve) = &target_entity.geometry else {
            return Err(Error::Missing("target curve"));
        };
        let tolerance = self.document().effective_tolerance()?.absolute_mm;
        let world_target = top11::bake_curve(target_curve, target_entity.placement)?;
        let mut fractions: Vec<f64> = match &cutter_entity.geometry {
            Geometry::Curve(cutter_curve) => {
                let cutter_curve = top11::bake_curve(cutter_curve, cutter_entity.placement)?;
                om9_core::curve_surface_ops::curve_curve_intersections(
                    &world_target,
                    &cutter_curve,
                    tolerance,
                    self.coordinator.limits(),
                )?
                .into_iter()
                .map(|hit| hit.first_fraction)
                .collect()
            }
            Geometry::Surface(surface) => {
                let Geometry::Surface(surface) = top11::bake_geometry(
                    &Geometry::Surface(surface.clone()),
                    cutter_entity.placement,
                    self.coordinator.limits(),
                )?
                else {
                    unreachable!()
                };
                om9_core::curve_surface_ops::curve_surface_intersections(
                    &world_target,
                    &surface,
                    tolerance,
                    self.coordinator.limits(),
                )?
                .into_iter()
                .map(|hit| hit.curve_fraction)
                .collect()
            }
            Geometry::Brep(brep) => {
                #[cfg(feature = "occt-brep")]
                {
                    let transformed;
                    let brep = if cutter_entity.placement == Transform::identity() {
                        brep
                    } else {
                        transformed = om9_kernel::transform_ops::transform_brep(
                            brep,
                            &cutter_entity.placement,
                            brep.display.deflection_mm,
                            self.coordinator.limits(),
                        )
                        .map_err(|error| Error::Io(format!("BRep cutter transform: {error}")))?;
                        &transformed
                    };
                    let intersection = if include_endpoints {
                        om9_kernel::intersect::curve_brep_fractions_including_endpoints
                    } else {
                        om9_kernel::intersect::curve_brep_fractions
                    };
                    intersection(brep, &world_target, tolerance, self.coordinator.limits())
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
        for fraction in &mut fractions {
            *fraction = source_curve_fraction(
                target_curve,
                &world_target,
                target_entity.placement,
                *fraction,
                self.coordinator.limits(),
            )?;
        }
        let fractions=top11::normalize_cut_fractions(target_curve,target_entity.placement,fractions,tolerance,include_endpoints,self.coordinator.limits())?;
        if fractions.is_empty() {
            return Err(Error::Invalid("cutting object has no intersection"));
        }
        Ok(fractions)
    }

    fn split_cutter_fractions(&self, target: ObjectId, cutters: &[ObjectId], closed: bool) -> Result<Vec<f64>> {
        let entity = self.document().entities.get(&target).ok_or(Error::Missing("target curve"))?;
        let Geometry::Curve(curve) = &entity.geometry else { return Err(Error::Missing("target curve")); };
        let mut fractions = Vec::new();
        for cutter in cutters {
            let cutter = self.document().entities.get(cutter).ok_or(Error::Missing("cutting object"))?;
            match self.cutter_fractions_between(entity, cutter, closed) {
                Ok(hit) => fractions.extend(hit),
                Err(Error::Invalid("cutting object has no intersection")) => {},
                Err(error) => return Err(error),
            }
        }
        let mut fractions = top11::normalize_cut_fractions(curve, entity.placement, fractions, self.document().effective_tolerance()?.absolute_mm, closed, self.coordinator.limits())?;
        if closed {
            for fraction in &mut fractions { if *fraction == 1.0 { *fraction = 0.0; } }
            fractions.sort_by(f64::total_cmp);
            fractions.dedup();
        }
        Ok(fractions)
    }

    fn commit_cut(
        &mut self,
        action: Top11Action,
        target: ObjectId,
        pieces: Vec<Curve>,
    ) -> Result<Vec<ObjectId>> {
        if pieces.is_empty() {
            return Err(Error::Invalid("cut produced no curve"));
        }
        if pieces.len() == 1 {
            let geometry = Geometry::Curve(pieces.into_iter().next().expect("one piece"));
            let source = self.document().stamp(target)?;
            self.commit_top11(
                action,
                vec![Mutation::Geometry { source, geometry }],
                Some(vec![target]),
            )?;
            return Ok(vec![target]);
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
        self.commit_top11(action, mutations, Some(ids.clone()))?;
        Ok(ids)
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
        Ok((copies, replacements, outcome.placements))
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
            let world = match &entity.geometry {
                Geometry::Surface(surface) => om9_kernel::measurement::surface_to_face(
                    surface,entity.placement,om9_kernel::creation::DEFAULT_DEFLECTION_MM,limits,
                ).map_err(|e| Error::Io(format!("surface join: {e}")))?,
                Geometry::Brep(brep) if entity.placement == Transform::identity() => brep.clone(),
                Geometry::Brep(brep) => om9_kernel::transform_ops::transform_brep(
                    brep,&entity.placement,brep.display.deflection_mm,limits,
                ).map_err(|e| Error::Io(format!("BRep join: {e}")))?,
                _ => return Err(Error::Invalid("BRep join selection")),
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
                let candidate = om9_kernel::editing::sew_preserving_surfaces(
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
        let (copies, replacements, placements) = self.mirror_entities(ids, origin, normal, copy)?;
        let mut mutations = Vec::with_capacity(replacements.len() + copies.len() + placements.len());
        for (id,placement) in placements {mutations.push(Mutation::Placement {source:self.document().stamp(id)?,placement});}
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
        let old_scene=self.scene.as_ref().ok_or(Error::Missing("top11 scene"))?;
        if old_scene.epoch!=self.coordinator.epoch() {return Err(Error::Stale("top11 scene"));}
        let held = self.scene_cache_usage()?;
        let prepared = self
            .coordinator
            .prepare_with_held_usage(TransactionRequest::new(action.command(), mutations), held)?;
        let candidate=prepared.candidate_world().documents.get(&self.document)
            .ok_or(Error::Missing("top11 candidate document"))?;
        let unchanged = std::sync::Arc::ptr_eq(candidate,&self.coordinator.world().documents[&self.document]);
        let receipt = if unchanged {
            self.coordinator.commit(prepared)?
        } else {
            self.commit_with_candidate_scene(prepared, held)?
        };
        if receipt.published {
            if unchanged { self.clear_scene(); }
            self.sync_published(om9_core::changes::ChangeCause::Edit);
            if let Some(ids) = selection {
                // An edit may produce hidden, locked or non-renderable members.
                // The document is already committed: do not report its success as
                // a failure merely because those members cannot be selected.
                let selectable:Vec<_>=ids.into_iter().filter(|id| self.scene.as_ref().is_some_and(|scene|
                    scene.objects.iter().any(|object| object.root==*id && object.selectable && object.has_pick_geometry())
                )).collect();
                if let Err(error)=self.select_objects(&selectable) {
                    self.clear_selection();
                    self.events.emit(crate::command_events::EventKind::Warn,action.command(),
                        &format!("Committed; result selection was cleared: {error}"),None);
                }
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
        let shrink_state = match self.top11_pending.as_mut() {
            Some(Top11Pending::SplitMany {
                shrink: Some(shrink),
                ..
            }) => Some((shrink, "pick cutting curves (Enter to execute)")),
            Some(Top11Pending::SplitIso {
                shrink,
                shrink_available: true,
                ..
            }) => Some((shrink, "pick an isocurve location on the surface")),
            _ => None,
        };
        if let Some((shrink, prompt)) = shrink_state {
            let lower = text.to_ascii_lowercase();
            if lower == "shrink" || lower.starts_with("shrink=") {
                *shrink = match lower.strip_prefix("shrink=") {
                    None => !*shrink,
                    Some("yes" | "y" | "on" | "1" | "true") => true,
                    Some("no" | "n" | "off" | "0" | "false") => false,
                    _ => return Some(Err(Error::Invalid("Shrink expects Yes or No"))),
                };
                return Some(Ok(Top11Result::Awaiting {
                    label: Top11Action::Split.label(),
                    prompt,
                }));
            }
        }
        if matches!(text.to_ascii_lowercase().as_str(), "isocurve" | "i") {
            if let Some(Top11Pending::SplitMany {
                targets,
                cutters,
                isocurve_allowed,
                shrink,
            }) = self.top11_pending.as_ref()
            {
                if targets.len() == 1 && cutters.is_empty() && *isocurve_allowed {
                    let target = targets[0];
                    let shrink_available = shrink.is_some();
                    let shrink = shrink.unwrap_or(false);
                    self.top11_pending = Some(Top11Pending::SplitIso {
                        target,
                        direction: SplitIsoDirection::U,
                        shrink,
                        shrink_available,
                    });
                    return Some(Ok(Top11Result::Awaiting {
                        label: Top11Action::Split.label(),
                        prompt: self.top11_prompt().unwrap(),
                    }));
                }
            }
        }
        if let Some(Top11Pending::SplitIso { direction, .. }) = self.top11_pending.as_mut() {
            let lower = text.to_ascii_lowercase();
            let value = lower.strip_prefix("direction=").unwrap_or(&lower);
            *direction = match value {
                "u" => SplitIsoDirection::U,
                "v" => SplitIsoDirection::V,
                "both" | "b" => SplitIsoDirection::Both,
                "toggle" | "t" => {
                    if *direction == SplitIsoDirection::U {
                        SplitIsoDirection::V
                    } else {
                        SplitIsoDirection::U
                    }
                }
                _ => return None,
            };
            return Some(Ok(Top11Result::Awaiting {
                label: Top11Action::Split.label(),
                prompt: "pick an isocurve location on the surface",
            }));
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
                if let Err(error) = self.top11_invoke(action) {
                    return Some(Err(error));
                }
                if let Some(Top11Pending::SelectObjects { split_point, .. }) =
                    self.top11_pending.as_mut()
                {
                    *split_point = true;
                    return Some(Ok(Top11Result::Awaiting {
                        label: action.label(),
                        prompt: "select one curve (Enter to continue)",
                    }));
                }
                self.top11_split_point_mode()
            }
            Top11Action::Trim if values.len() >= 2 => self.top11_trim(values[0], values[1]),
            _ => self.top11_invoke_route(action,action==Top11Action::Explode),
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
        self.grip_selection.clear();
    }
}

/// How a grip pick changes the picked set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GripPick {
    Replace,
    Add,
    Remove,
}
