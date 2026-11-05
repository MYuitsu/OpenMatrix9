// SPDX-License-Identifier: LGPL-2.1-or-later
//! Matrix Top 11 command service (OM9-L2-03-08).
//!
//! The Top 11 are command adapters, not a second geometry kernel: duplicate, mirror,
//! move/rotate, point display, explode/join/split/trim all call the existing curve,
//! placement and document records. This module owns the action catalogue, the
//! availability rule and the pure entity/geometry builders; the host commits them in
//! one transaction.
use crate::{
    Error, Result,
    budget::Limits,
    curve_ops, curve_surface_ops,
    geometry::{Curve, Geometry, GeometryKind, Mesh, NurbsCurve, Point3, Transform, finite},
    identity::ObjectId,
    model::{Document, Entity},
    surface_kernel,
};
use std::collections::{BTreeMap, BTreeSet};

/// The eleven Matrix Top 11 actions in toolbar order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Top11Action {
    Duplicate,
    Mirror,
    Move,
    Rotate,
    EditPoints,
    ControlPoints,
    Explode,
    Join,
    Split,
    Trim,
    RingRail,
}

impl Top11Action {
    pub const ALL: [Self; 11] = [
        Self::Duplicate,
        Self::Mirror,
        Self::Move,
        Self::Rotate,
        Self::EditPoints,
        Self::ControlPoints,
        Self::Explode,
        Self::Join,
        Self::Split,
        Self::Trim,
        Self::RingRail,
    ];

    /// Original Matrix command name.
    pub fn command(self) -> &'static str {
        match self {
            Self::Duplicate => "Copy",
            Self::Mirror => "Mirror",
            Self::Move => "Move",
            Self::Rotate => "Rotate",
            Self::EditPoints => "EditPtOn",
            Self::ControlPoints => "PointsOn",
            Self::Explode => "Explode",
            Self::Join => "Join",
            Self::Split => "Split",
            Self::Trim => "Trim",
            Self::RingRail => "gvRingRail",
        }
    }

    /// Icon key registered by the application.
    pub fn icon(self) -> &'static str {
        match self {
            Self::Duplicate => "Duplicate",
            Self::Mirror => "Mirror",
            Self::Move => "Move",
            Self::Rotate => "Rotate",
            Self::EditPoints => "EditPoints",
            Self::ControlPoints => "ControlPoints",
            Self::Explode => "Explode",
            Self::Join => "Join",
            Self::Split => "Split",
            Self::Trim => "Trim",
            Self::RingRail => "RingRail",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Duplicate => "Duplicate",
            Self::Mirror => "Mirror",
            Self::Move => "Move",
            Self::Rotate => "Rotate",
            Self::EditPoints => "Edit Points On",
            Self::ControlPoints => "Control Points On",
            Self::Explode => "Explode",
            Self::Join => "Join",
            Self::Split => "Split",
            Self::Trim => "Trim",
            Self::RingRail => "Ring Rail",
        }
    }

    /// Core requirement id from `docs/core-requirements.json`.
    pub fn feature_id(self) -> &'static str {
        match self {
            Self::ControlPoints => "OM9-TOP11-001",
            Self::Duplicate => "OM9-TOP11-002",
            Self::EditPoints => "OM9-TOP11-003",
            Self::Mirror => "OM9-TOP11-004",
            Self::Explode => "OM9-TOP11-005",
            Self::Move => "OM9-TOP11-006",
            Self::Rotate => "OM9-TOP11-007",
            Self::Join => "OM9-TOP11-008",
            Self::Split => "OM9-TOP11-009",
            Self::Trim => "OM9-TOP11-010",
            Self::RingRail => "OM9-TOP11-011",
        }
    }

    /// Only `PointsOn` has a documented Matrix9 shortcut (F10).
    pub fn shortcut(self) -> Option<&'static str> {
        match self {
            Self::ControlPoints => Some("F10"),
            _ => None,
        }
    }

    pub fn from_label(label: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|action| action.label().eq_ignore_ascii_case(label))
    }

    pub fn from_icon(icon: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|action| action.icon().eq_ignore_ascii_case(icon))
    }

    /// Resolve the stable feature id a menu row or CMD alias publishes, so a
    /// toolbar click and a feature dispatch reach the same action.
    pub fn from_feature(feature: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|action| action.feature_id().eq_ignore_ascii_case(feature))
    }
}

/// Point display mode selected by the two `*PtOn` actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GripMode {
    Edit,
    Control,
}

/// What the selection currently offers to the Top 11.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Top11Context {
    pub running: bool,
    pub selected: usize,
    pub curves: usize,
    pub points: usize,
    pub meshes: usize,
    pub surfaces: usize,
    pub breps: usize,
    pub other: usize,
    pub mirrorable_other: usize,
}

impl Top11Context {
    pub fn capture(document: &Document, selection: &BTreeSet<ObjectId>, running: bool) -> Self {
        let mut context = Self {
            running,
            selected: selection.len(),
            curves: 0,
            points: 0,
            meshes: 0,
            surfaces: 0,
            breps: 0,
            other: 0,
            mirrorable_other: 0,
        };
        for id in selection {
            match document
                .entities
                .get(id)
                .map(|entity| entity.geometry.kind())
            {
                Some(GeometryKind::Curve) => context.curves += 1,
                Some(GeometryKind::Point) => context.points += 1,
                Some(GeometryKind::Mesh) => context.meshes += 1,
                Some(GeometryKind::Surface) => context.surfaces += 1,
                Some(GeometryKind::Brep) => context.breps += 1,
                Some(kind) => {
                    context.other += 1;
                    if matches!(kind,GeometryKind::Cloud | GeometryKind::Instance | GeometryKind::Hatch | GeometryKind::Light) {context.mirrorable_other += 1;}
                },
                None => context.selected = context.selected.saturating_sub(1),
            }
        }
        context
    }
}

/// One toolbar button's admitted state and the reason a disabled button exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Top11Availability {
    pub action: Top11Action,
    pub enabled: bool,
    pub reason: &'static str,
}

impl Top11Availability {
    fn enabled(action: Top11Action) -> Self {
        Self {
            action,
            enabled: true,
            reason: "",
        }
    }

    fn disabled(action: Top11Action, reason: &'static str) -> Self {
        Self {
            action,
            enabled: false,
            reason,
        }
    }
}

/// Admission rule for every Top 11 button.
pub fn availability(action: Top11Action, context: &Top11Context) -> Top11Availability {
    if action == Top11Action::RingRail {
        return Top11Availability::disabled(action, "Ring Rail builder belongs to M04");
    }
    if context.running {
        return Top11Availability::disabled(action, "a command is already running");
    }
    if context.selected == 0 {
        return Top11Availability::disabled(action, "select objects first");
    }
    match action {
        Top11Action::Duplicate => Top11Availability::enabled(action),
        Top11Action::Mirror => {
            if context.curves + context.points + context.meshes + context.surfaces + context.breps + context.mirrorable_other
                > 0
            {
                Top11Availability::enabled(action)
            } else {
                Top11Availability::disabled(
                    action,
                    "mirror needs supported geometry or an instance",
                )
            }
        }
        Top11Action::Move | Top11Action::Rotate => Top11Availability::enabled(action),
        Top11Action::EditPoints => {
            if context.curves + context.surfaces > 0 {
                Top11Availability::enabled(action)
            } else {
                Top11Availability::disabled(action, "select a curve or surface first")
            }
        }
        Top11Action::ControlPoints => {
            if context.curves + context.surfaces > 0 {
                Top11Availability::enabled(action)
            } else {
                Top11Availability::disabled(action, "select a curve or surface first")
            }
        }
        Top11Action::Explode => {
            if context.curves + context.meshes > 0 && context.curves + context.meshes == context.selected {
                Top11Availability::enabled(action)
            } else {
                Top11Availability::disabled(action, "explode needs curves or meshes")
            }
        }
        Top11Action::Join => {
            if context.curves >= 2 && context.curves == context.selected {
                Top11Availability::enabled(action)
            } else {
                Top11Availability::disabled(action, "join needs two or more curves")
            }
        }
        Top11Action::Split | Top11Action::Trim => {
            if context.curves == 1 && context.selected == 1 {
                Top11Availability::enabled(action)
            } else {
                Top11Availability::disabled(action, "split/trim need exactly one curve")
            }
        }
        Top11Action::RingRail => unreachable!("handled first"),
    }
}

pub fn toolbar(context: &Top11Context) -> [Top11Availability; 11] {
    std::array::from_fn(|index| availability(Top11Action::ALL[index], context))
}

/* ------------------------------ entity helpers ------------------------------ */

fn clone_entity(source: &Entity, geometry: Geometry, placement: Transform) -> Entity {
    let mut entity = source.clone();
    entity.id = ObjectId::new();
    entity.generation = 1;
    entity.geometry_revision = 1;
    entity.attributes_revision = 1;
    entity.geometry = geometry;
    entity.placement = placement;
    entity
}

fn curve_of(document: &Document, id: ObjectId) -> Result<&Curve> {
    match &document
        .entities
        .get(&id)
        .ok_or(Error::Missing("object"))?
        .geometry
    {
        Geometry::Curve(curve) => Ok(curve),
        _ => Err(Error::Unsupported("curve operation needs a curve")),
    }
}

/// Duplicate selected objects in place (the command's `In Place` result).
pub fn duplicate_entities(
    document: &Document,
    ids: &BTreeSet<ObjectId>,
    limits: &Limits,
) -> Result<Vec<Entity>> {
    if ids.is_empty() {
        return Err(Error::Invalid("duplicate selection"));
    }
    let mut copies = Vec::new();
    copies
        .try_reserve_exact(ids.len())
        .map_err(|_| Error::Budget("duplicate objects"))?;
    for id in ids {
        let source = document.entities.get(id).ok_or(Error::Missing("object"))?;
        source.geometry.validate(limits)?;
        source.placement.validate()?;
        copies.push(clone_entity(
            source,
            source.geometry.clone(),
            source.placement,
        ));
    }
    Ok(copies)
}

/// Duplicate selected objects with one world translation (the base-point flow).
pub fn duplicate_entities_offset(
    document: &Document,
    ids: &BTreeSet<ObjectId>,
    delta: Point3,
    limits: &Limits,
) -> Result<Vec<Entity>> {
    finite(delta)?;
    let mut copies = duplicate_entities(document, ids, limits)?;
    let translation = Transform::translation(delta);
    for copy in &mut copies {
        copy.placement = translation.compose(copy.placement)?;
    }
    Ok(copies)
}

/// Explode every selected polycurve into its members.
pub fn explode_entities(
    document: &Document,
    ids: &BTreeSet<ObjectId>,
    limits: &Limits,
) -> Result<Vec<Entity>> {
    let mut pieces = Vec::new();
    for id in ids {
        let source = document.entities.get(id).ok_or(Error::Missing("object"))?;
        match &source.geometry {
            Geometry::Curve(curve) => {
                let members = curve_ops::explode_curve(curve, limits)?;
                if members.len() < 2 { return Err(Error::Unsupported("explode needs a polycurve")); }
                for member in members {
                    member.validate(limits)?;
                    pieces.push(clone_entity(source, Geometry::Curve(member), source.placement));
                }
            }
            Geometry::Mesh(mesh) => {
                for member in explode_mesh(mesh, limits)? {
                    pieces.push(clone_entity(source, Geometry::Mesh(member), source.placement));
                }
            }
            _ => return Err(Error::Unsupported("explode needs curves or meshes")),
        }
    }
    if pieces.is_empty() {
        return Err(Error::Invalid("explode selection"));
    }
    Ok(pieces)
}

/// Separate face components connected by welded (shared-index) mesh edges.
fn explode_mesh(mesh: &Mesh, limits: &Limits) -> Result<Vec<Mesh>> {
    mesh.validate(limits)?;
    let count=mesh.faces.len();
    let corners=mesh.faces.iter().try_fold(0usize, |n,f| n.checked_add(f.len()).ok_or(Error::Overflow("mesh corners")))?;
    let mut scratch=crate::budget::Usage::array::<usize>(count)?;
    scratch.add(crate::budget::Usage::array::<u8>(count)?,limits)?;
    scratch.add(crate::budget::Usage::array::<bool>(mesh.vertices.len())?,limits)?;
    scratch.add(crate::budget::Usage::array::<(Point3,Point3,[u8;4])>(mesh.vertices.len())?,limits)?;
    // Account conservatively for edge maps, remapping maps and transient output
    // storage before populating them. Output meshes still receive normal validation.
    scratch.add(crate::budget::Usage::array::<[u8;128]>(corners)?,limits)?;
    let mut parent:Vec<usize>=(0..count).collect();
    let mut rank=vec![0u8;count];
    let mut used=vec![false;mesh.vertices.len()];
    let mut edges=BTreeMap::<(u32,u32),usize>::new();
    fn root(parent:&mut [usize],mut i:usize)->usize {
        while parent[i]!=i { parent[i]=parent[parent[i]];i=parent[i]; } i
    }
    for (index,face) in mesh.faces.iter().enumerate() {
        for i in 0..face.len() {
            let a=face[i];let b=face[(i+1)%face.len()];let edge=(a.min(b),a.max(b));
            used[a as usize]=true;
            if let Some(other)=edges.insert(edge,index) {
                let a=root(&mut parent,index);let b=root(&mut parent,other);
                if a!=b {
                    if rank[a]<rank[b] {parent[a]=b;} else {
                        parent[b]=a;
                        if rank[a]==rank[b] {rank[a]=rank[a].saturating_add(1);}
                    }
                }
            }
        }
    }
    let components:BTreeSet<usize>=(0..count).map(|i|root(&mut parent,i)).collect();
    if components.len()<2 {return Err(Error::Unsupported("mesh has no unwelded components"));}
    if u64::try_from(components.len()).map_err(|_| Error::Overflow("mesh outputs"))?>limits.max_outputs {
        return Err(Error::Budget("outputs"));
    }
    let mut groups=BTreeMap::<usize,(Mesh,BTreeMap<u32,u32>)>::new();
    let mut uv=0;
    for (index,face) in mesh.faces.iter().enumerate() {
        let key=root(&mut parent,index);
        let (part,map)=groups.entry(key).or_insert_with(||(Mesh{vertices:Vec::new(),faces:Vec::new(),normals:Vec::new(),colors:Vec::new(),corner_uvs:Vec::new()},BTreeMap::new()));
        let mut output=Vec::with_capacity(face.len());
        for old in face {
            let new=if let Some(new)=map.get(old) {*new} else {
                let new=u32::try_from(part.vertices.len()).map_err(|_| Error::Overflow("mesh vertex index"))?;
                let i=*old as usize;part.vertices.push(mesh.vertices[i]);
                if !mesh.normals.is_empty(){part.normals.push(mesh.normals[i]);}
                if !mesh.colors.is_empty(){part.colors.push(mesh.colors[i]);}
                map.insert(*old,new);new
            };
            output.push(new);
        }
        part.faces.push(output);
        if !mesh.corner_uvs.is_empty(){part.corner_uvs.extend_from_slice(&mesh.corner_uvs[uv..uv+face.len()]);}
        uv+=face.len();
    }
    // Isolated vertices are not assigned to any face component. Keep them once,
    // with their channels, on the deterministic first component.
    if let Some((part,_))=groups.values_mut().next() {
        for (i,referenced) in used.iter().enumerate() {
            if !referenced {
                part.vertices.push(mesh.vertices[i]);
                if !mesh.normals.is_empty(){part.normals.push(mesh.normals[i]);}
                if !mesh.colors.is_empty(){part.colors.push(mesh.colors[i]);}
            }
        }
    }
    let mut result=Vec::new();
    result.try_reserve_exact(groups.len()).map_err(|_| Error::Budget("exploded meshes"))?;
    for (_, (part,_)) in groups {part.validate(limits)?;result.push(part);}
    Ok(result)
}

/// Join selected curves into one curve in the first object's layer/attributes.
pub fn join_entities(
    document: &Document,
    ids: &BTreeSet<ObjectId>,
    tolerance: f64,
    limits: &Limits,
) -> Result<Entity> {
    if ids.len() < 2 {
        return Err(Error::Invalid("join selection"));
    }
    let mut source: Option<&Entity> = None;
    let mut curves = Vec::new();
    for id in ids {
        let entity = document.entities.get(id).ok_or(Error::Missing("object"))?;
        let curve = curve_of(document, *id)?;
        if source.is_none() { source = Some(entity); }
        curves.push(if entity.placement == Transform::identity() {
            curve.clone()
        } else {
            bake_curve(curve, entity.placement)?
        });
    }
    let curves = chain_curves(curves, tolerance, limits)?;
    let joined = curve_ops::join_curves(&curves, tolerance, limits)?;
    let joined = heal_joined_curve(joined, tolerance, limits)?;
    joined.validate(limits)?;
    let source = source.ok_or(Error::Missing("join source"))?;
    Ok(clone_entity(source, Geometry::Curve(joined), Transform::identity()))
}

/// Replace a curve endpoint while retaining the other poles and parameters where possible.
fn set_join_endpoint(curve: &Curve, start: bool, target: Point3, tolerance: f64, limits: &Limits) -> Result<Curve> {
    let old = curve_ops::curve_endpoints(curve, limits)?[usize::from(!start)];
    if old == target { return Ok(curve.clone()); }
    let result = match curve {
        Curve::Line { start: a, end: b, domain } => Curve::Line { start: if start { target } else { *a }, end: if start { *b } else { target }, domain: *domain },
        Curve::Circle { center, x, y, radius, domain, angle_domain } => {
            let n = affine_circle(*center,*x,*y,*radius,angle_domain.unwrap_or(*domain),Transform::identity())?;
            return set_join_endpoint(&n,start,target,tolerance,limits);
        }
        Curve::Nurbs(n) => {
            let mut changed=curve_ops::clamped_active_nurbs(n,start,limits)?;
            let index=if start != changed.reversed {0} else {changed.poles.len()-1};
            changed.poles[index]=target; Curve::Nurbs(changed)
        }
        Curve::Polycurve { .. } => {
            let mut segments=curve_ops::explode_curve(curve,limits)?;
            let index=if start {0} else {segments.len()-1};
            segments[index]=set_join_endpoint(&segments[index],start,target,tolerance,limits)?;
            Curve::Polycurve {segments,reversed:false}
        }
    };
    result.validate(limits)?;
    let actual=curve_ops::curve_endpoints(&result,limits)?[usize::from(!start)];
    if distance(actual,target)>tolerance*0.01 { return Err(Error::Invalid("join endpoint adjustment")); }
    Ok(result)
}

/// Replace each pair of tolerated endpoints by their common world-space midpoint.
fn heal_joined_curve(curve: Curve, tolerance: f64, limits: &Limits) -> Result<Curve> {
    let Curve::Polycurve { mut segments, reversed: false } = curve else { return Ok(curve) };
    for i in 1..segments.len() {
        let a=curve_ops::curve_endpoints(&segments[i-1],limits)?[1];
        let b=curve_ops::curve_endpoints(&segments[i],limits)?[0];
        if distance(a,b)>tolerance { return Err(Error::Invalid("join continuity")); }
        if a != b {
            let midpoint=std::array::from_fn(|axis| a[axis]*0.5+b[axis]*0.5);
            segments[i-1]=set_join_endpoint(&segments[i-1],false,midpoint,tolerance,limits)?;
            segments[i]=set_join_endpoint(&segments[i],true,midpoint,tolerance,limits)?;
        }
    }
    if segments.len()>1 {
        let first=curve_ops::curve_endpoints(&segments[0],limits)?[0];
        let last=curve_ops::curve_endpoints(segments.last().unwrap(),limits)?[1];
        if first != last && distance(first,last)<=tolerance {
            let midpoint=std::array::from_fn(|axis| first[axis]*0.5+last[axis]*0.5);
            segments[0]=set_join_endpoint(&segments[0],true,midpoint,tolerance,limits)?;
            let end=segments.len()-1;
            segments[end]=set_join_endpoint(&segments[end],false,midpoint,tolerance,limits)?;
        }
    }
    Ok(Curve::Polycurve {segments,reversed:false})
}

fn distance(a: Point3, b: Point3) -> f64 {
    let delta = sub(a, b);
    delta[0].hypot(delta[1]).hypot(delta[2])
}

/// Order curves as a continuous trail, including loops encountered along the way.
fn chain_curves(curves: Vec<Curve>, tolerance: f64, limits: &Limits) -> Result<Vec<Curve>> {
    if curves.len() < 2 { return Err(Error::Invalid("join selection")); }
    if !tolerance.is_finite() || tolerance < 0.0 { return Err(Error::Invalid("join tolerance")); }
    let endpoint_count = curves.len().checked_mul(2).ok_or(Error::Overflow("join endpoints"))?;
    let mut scratch = crate::budget::Usage::array::<Point3>(endpoint_count)?;
    scratch.nodes = u64::try_from(endpoint_count).map_err(|_| Error::Overflow("join nodes"))?;
    scratch.add(crate::budget::Usage::array::<usize>(endpoint_count)?, limits)?;
    scratch.add(crate::budget::Usage::array::<usize>(endpoint_count)?, limits)?;
    scratch.add(crate::budget::Usage::array::<Vec<(usize, usize, bool)>>(endpoint_count)?, limits)?;
    scratch.add(crate::budget::Usage::array::<(usize, usize, bool)>(endpoint_count)?, limits)?;
    let mut endpoints = Vec::new();
    endpoints.try_reserve_exact(endpoint_count).map_err(|_| Error::Budget("join endpoints"))?;
    for curve in &curves { endpoints.extend(curve_ops::curve_endpoints(curve, limits)?); }
    let mut parent = Vec::new();
    parent.try_reserve_exact(endpoint_count).map_err(|_| Error::Budget("join nodes"))?;
    parent.extend(0..endpoint_count);
    fn root(parent: &mut [usize], mut index: usize) -> usize {
        while parent[index] != index { parent[index] = parent[parent[index]]; index = parent[index]; }
        index
    }
    // Sweep along the widest axis; only candidates within tolerance on that axis
    // need the exact 3D distance test. Dense coincident points remain quadratic.
    let axis = (0..3).max_by(|&a, &b| {
        let extent = |axis: usize| {
            let (min, max) = endpoints.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), p| (min.min(p[axis]), max.max(p[axis])));
            max - min
        };
        extent(a).total_cmp(&extent(b))
    }).expect("three axes");
    let mut order = Vec::new();
    order.try_reserve_exact(endpoint_count).map_err(|_| Error::Budget("join sweep"))?;
    order.extend(0..endpoint_count);
    order.sort_unstable_by(|&a, &b| endpoints[a][axis].total_cmp(&endpoints[b][axis]).then(a.cmp(&b)));
    for position in 0..endpoint_count {
        let i = order[position];
        for &j in order[..position].iter().rev() {
            if endpoints[i][axis] - endpoints[j][axis] > tolerance { break; }
            if distance(endpoints[i], endpoints[j]) <= tolerance {
                let a = root(&mut parent, i); let b = root(&mut parent, j);
                if a != b { let low = a.min(b); let high = a.max(b); parent[high] = low; }
            }
        }
    }
    let mut adjacency: Vec<Vec<(usize, usize, bool)>> = Vec::new();
    adjacency.try_reserve_exact(endpoint_count).map_err(|_| Error::Budget("join adjacency"))?;
    adjacency.resize_with(endpoint_count, Vec::new);
    for index in 0..curves.len() {
        let a = root(&mut parent, index * 2); let b = root(&mut parent, index * 2 + 1);
        adjacency[a].try_reserve(1).map_err(|_| Error::Budget("join adjacency"))?;
        adjacency[a].push((index, b, false));
        adjacency[b].try_reserve(1).map_err(|_| Error::Budget("join adjacency"))?;
        adjacency[b].push((index, a, true));
    }
    let odd: Vec<_> = adjacency.iter().enumerate().filter_map(|(i, edges)| (edges.len() % 2 == 1).then_some(i)).collect();
    if !matches!(odd.len(), 0 | 2) { return Err(Error::Invalid("join has no continuous trail")); }
    let start = odd.first().copied().unwrap_or_else(|| adjacency.iter().position(|edges| !edges.is_empty()).expect("nonempty curves"));
    let mut used = vec![false; curves.len()];
    let mut stack = vec![(start, None)];
    let mut trail = Vec::new();
    while let Some(&(vertex, _)) = stack.last() {
        let next = loop {
            match adjacency[vertex].pop() {
                Some((edge, other, reverse)) if !used[edge] => break Some((edge, other, reverse)),
                Some(_) => continue,
                None => break None,
            }
        };
        if let Some((edge, other, reverse)) = next {
            used[edge] = true; stack.push((other, Some((edge, reverse))));
        } else if let Some((_, Some(edge))) = stack.pop() { trail.push(edge); }
    }
    if trail.len() != curves.len() { return Err(Error::Invalid("join continuity")); }
    trail.reverse();
    let mut ordered = Vec::new();
    ordered.try_reserve_exact(curves.len()).map_err(|_| Error::Budget("join trail"))?;
    for (edge, reverse) in trail {
        ordered.push(if reverse { curve_ops::reverse_curve(&curves[edge], limits)? } else { curves[edge].clone() });
    }
    // Tolerance clusters can be non-transitive: reject any consecutive endpoints
    // farther apart than the requested tolerance instead of inventing a connection.
    for pair in ordered.windows(2) {
        let end = curve_ops::curve_endpoints(&pair[0], limits)?[1];
        let start = curve_ops::curve_endpoints(&pair[1], limits)?[0];
        if distance(end, start) > tolerance { return Err(Error::Invalid("join ambiguous tolerance cluster")); }
    }
    Ok(ordered)
}

/// Split one curve at a fraction into two objects.
pub fn split_entity(
    document: &Document,
    id: ObjectId,
    fraction: f64,
    limits: &Limits,
) -> Result<Vec<Entity>> {
    let source = document.entities.get(&id).ok_or(Error::Missing("object"))?;
    let curve = curve_of(document, id)?;
    let (left, right) = curve_ops::split_curve(curve, fraction, limits)?;
    left.validate(limits)?;
    right.validate(limits)?;
    Ok(vec![
        clone_entity(source, Geometry::Curve(left), source.placement),
        clone_entity(source, Geometry::Curve(right), source.placement),
    ])
}

/// Trim one curve to the fraction range; the same object identity is kept.
pub fn trim_entity(
    document: &Document,
    id: ObjectId,
    from: f64,
    to: f64,
    limits: &Limits,
) -> Result<Geometry> {
    let curve = curve_of(document, id)?;
    let trimmed = curve_ops::trim_curve(curve, from, to, limits)?;
    trimmed.validate(limits)?;
    Ok(Geometry::Curve(trimmed))
}

/* ------------------------------- transforms ------------------------------- */

fn unit(value: Point3) -> Result<Point3> {
    let length = (value[0] * value[0] + value[1] * value[1] + value[2] * value[2]).sqrt();
    if !length.is_finite() || length <= 1e-12 {
        return Err(Error::Invalid("direction"));
    }
    Ok([value[0] / length, value[1] / length, value[2] / length])
}

fn dot(a: Point3, b: Point3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn sub(a: Point3, b: Point3) -> Point3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale(a: Point3, factor: f64) -> Point3 {
    [a[0] * factor, a[1] * factor, a[2] * factor]
}

fn translation(offset: Point3) -> Transform {
    Transform::translation(offset)
}

/// Rotation about an arbitrary axis through `center`, right-handed, degrees.
pub fn rotation(center: Point3, axis: Point3, degrees: f64) -> Result<Transform> {
    finite(center)?;
    if !degrees.is_finite() {
        return Err(Error::Invalid("rotation angle"));
    }
    let axis = unit(axis)?;
    let radians = degrees.to_radians();
    let (sine, cosine) = radians.sin_cos();
    let one = 1.0 - cosine;
    let (x, y, z) = (axis[0], axis[1], axis[2]);
    let linear = [
        [
            cosine + x * x * one,
            x * y * one - z * sine,
            x * z * one + y * sine,
        ],
        [
            y * x * one + z * sine,
            cosine + y * y * one,
            y * z * one - x * sine,
        ],
        [
            z * x * one - y * sine,
            z * y * one + x * sine,
            cosine + z * z * one,
        ],
    ];
    let mut matrix = Transform::identity();
    for (row, values) in linear.iter().enumerate() {
        matrix.0[row][..3].copy_from_slice(values);
        matrix.0[row][3] = center[row]
            - (0..3)
                .map(|column| values[column] * center[column])
                .sum::<f64>();
    }
    matrix.validate()?;
    Ok(matrix)
}

/// Move every selected object by one world translation.
pub fn translate_placements(
    document: &Document,
    ids: &BTreeSet<ObjectId>,
    delta: Point3,
) -> Result<Vec<(ObjectId, Transform)>> {
    finite(delta)?;
    let mut moved = Vec::new();
    moved
        .try_reserve_exact(ids.len())
        .map_err(|_| Error::Budget("move objects"))?;
    for id in ids {
        let entity = document.entities.get(id).ok_or(Error::Missing("object"))?;
        moved.push((*id, translation(delta).compose(entity.placement)?));
    }
    Ok(moved)
}

/// Rotate every selected object about a world axis through `center`.
pub fn rotate_placements(
    document: &Document,
    ids: &BTreeSet<ObjectId>,
    center: Point3,
    axis: Point3,
    degrees: f64,
) -> Result<Vec<(ObjectId, Transform)>> {
    let rotation = rotation(center, axis, degrees)?;
    let mut moved = Vec::new();
    moved
        .try_reserve_exact(ids.len())
        .map_err(|_| Error::Budget("rotate objects"))?;
    for id in ids {
        let entity = document.entities.get(id).ok_or(Error::Missing("object"))?;
        moved.push((*id, rotation.compose(entity.placement)?));
    }
    Ok(moved)
}

/// Inverse of a rigid placement (orthonormal rotation/reflection plus translation).
pub fn inverse_affine(placement: Transform) -> Result<Transform> {
    placement.validate()?;
    let m = placement.0;
    let determinant = m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
    if !determinant.is_finite() || determinant.abs() <= 1e-300 {
        return Err(Error::Invalid("singular placement"));
    }
    let inverse_det = 1.0 / determinant;
    let mut inverse = Transform::identity();
    inverse.0[0][0] = (m[1][1] * m[2][2] - m[1][2] * m[2][1]) * inverse_det;
    inverse.0[0][1] = (m[0][2] * m[2][1] - m[0][1] * m[2][2]) * inverse_det;
    inverse.0[0][2] = (m[0][1] * m[1][2] - m[0][2] * m[1][1]) * inverse_det;
    inverse.0[1][0] = (m[1][2] * m[2][0] - m[1][0] * m[2][2]) * inverse_det;
    inverse.0[1][1] = (m[0][0] * m[2][2] - m[0][2] * m[2][0]) * inverse_det;
    inverse.0[1][2] = (m[0][2] * m[1][0] - m[0][0] * m[1][2]) * inverse_det;
    inverse.0[2][0] = (m[1][0] * m[2][1] - m[1][1] * m[2][0]) * inverse_det;
    inverse.0[2][1] = (m[0][1] * m[2][0] - m[0][0] * m[2][1]) * inverse_det;
    inverse.0[2][2] = (m[0][0] * m[1][1] - m[0][1] * m[1][0]) * inverse_det;
    for row in 0..3 {
        inverse.0[row][3] = -(0..3)
            .map(|column| inverse.0[row][column] * m[column][3])
            .sum::<f64>();
    }
    inverse.validate()?;
    Ok(inverse)
}

/// Exact affine image of a circle: a rational quadratic NURBS ellipse. The weights and
/// knots are unchanged by an affine map, so only the Cartesian poles are transformed.
pub(crate) fn affine_circle(
    center: Point3,
    x: Point3,
    y: Point3,
    radius: f64,
    angles: [f64; 2],
    placement: Transform,
) -> Result<Curve> {
    let span = angles[1] - angles[0];
    if !span.is_finite() || span <= 0. || span > std::f64::consts::TAU + 1e-10 {
        return Err(Error::Invalid("affine arc angular domain"));
    }
    let segments = (span / std::f64::consts::FRAC_PI_2).ceil() as usize;
    let mut poles = Vec::with_capacity(segments * 2 + 1);
    let mut weights = Vec::with_capacity(segments * 2 + 1);
    let mut knots = vec![0., 0., 0.];
    for i in 0..segments {
        let a = angles[0] + span * i as f64 / segments as f64;
        let b = angles[0] + span * (i + 1) as f64 / segments as f64;
        let weight = ((b - a) * 0.5).cos();
        for (angle, w) in [(a, 1.), ((a + b) * 0.5, weight), (b, 1.)].into_iter().skip(usize::from(i > 0)) {
            let point = std::array::from_fn(|axis| center[axis] + radius * (angle.cos() * x[axis] + angle.sin() * y[axis]) / w);
            poles.push(placement.point(point)?);
            weights.push(w);
        }
        if i + 1 < segments {
            knots.extend([ (i + 1) as f64 / segments as f64; 2]);
        }
    }
    knots.extend([1., 1., 1.]);
    let curve = NurbsCurve {
        degree: 2,
        poles,
        weights,
        knots,
        domain: [0.0, 1.0],
        periodic: false,
        reversed: false,
        seam: None,
    };
    curve.validate(&Limits::default())?;
    Ok(Curve::Nurbs(curve))
}

fn transform_normal(placement: Transform, normal: Point3) -> Result<Point3> {
    let inverse = inverse_affine(placement)?;
    let mut out = [0.0; 3];
    for (row, value) in out.iter_mut().enumerate() {
        *value = (0..3)
            .map(|column| inverse.0[column][row] * normal[column])
            .sum::<f64>();
    }
    finite(out)?;
    let determinant = placement.0[0][0]
        * (placement.0[1][1] * placement.0[2][2] - placement.0[1][2] * placement.0[2][1])
        - placement.0[0][1]
            * (placement.0[1][0] * placement.0[2][2] - placement.0[1][2] * placement.0[2][0])
        + placement.0[0][2]
            * (placement.0[1][0] * placement.0[2][1] - placement.0[1][1] * placement.0[2][0]);
    if determinant < 0.0 {
        out = out.map(|value| -value);
    }
    unit(out)
}

pub fn bake_curve(curve: &Curve, placement: Transform) -> Result<Curve> {
    let vector = |v: Point3| -> Result<Point3> {
        let mut out = [0.0; 3];
        for (row, value) in out.iter_mut().enumerate() {
            *value = (0..3)
                .map(|column| placement.0[row][column] * v[column])
                .sum::<f64>();
        }
        finite(out)?;
        Ok(out)
    };
    Ok(match curve {
        Curve::Line { start, end, domain } => {
            let start = placement.point(*start)?;
            let end = placement.point(*end)?;
            Curve::Line {
                start,
                end,
                domain: *domain,
            }
        }
        Curve::Circle {
            center,
            x,
            y,
            radius,
            domain,
            angle_domain,
        } => {
            let transformed_x = vector(*x)?;
            let transformed_y = vector(*y)?;
            let x_scale = transformed_x[0].hypot(transformed_x[1]).hypot(transformed_x[2]);
            let y_scale = transformed_y[0].hypot(transformed_y[1]).hypot(transformed_y[2]);
            let same_scale = x_scale.is_finite() && y_scale.is_finite()
                && x_scale > 1e-12 && y_scale > 1e-12
                && (x_scale-y_scale).abs() <= 8.0*f64::EPSILON*x_scale.max(y_scale);
            let axes = if same_scale {
                let a = transformed_x.map(|v| v/x_scale);
                let b = transformed_y.map(|v| v/y_scale);
                (dot(a,b).abs() <= 8.0*f64::EPSILON).then_some((a,b))
            } else { None };
            if let Some((x,y)) = axes {
                Curve::Circle {
                    center: placement.point(*center)?,
                    x,
                    y,
                    radius: radius*x_scale,
                    domain: *domain,
                    angle_domain: *angle_domain,
                }
            } else {
                affine_circle(*center, *x, *y, *radius, angle_domain.unwrap_or(*domain), placement)?
            }
        }
        Curve::Nurbs(nurbs) => {
            let mut baked = nurbs.clone();
            for pole in &mut baked.poles {
                *pole = placement.point(*pole)?;
            }
            Curve::Nurbs(baked)
        }
        Curve::Polycurve { segments, reversed } => {
            let mut baked = Vec::new();
            baked
                .try_reserve_exact(segments.len())
                .map_err(|_| Error::Budget("baked polycurve"))?;
            for segment in segments {
                baked.push(bake_curve(segment, placement)?);
            }
            Curve::Polycurve {
                segments: baked,
                reversed: *reversed,
            }
        }
    })
}

/// The geometry with the placement applied, so callers that work in world space (mirror,
/// deformers) can write one Geometry mutation back into the object's own frame.
pub fn bake_geometry(
    geometry: &Geometry,
    placement: Transform,
    limits: &Limits,
) -> Result<Geometry> {
    geometry.validate(limits)?;
    Ok(match geometry {
        Geometry::Point(point) => Geometry::Point(placement.point(*point)?),
        Geometry::Curve(curve) => Geometry::Curve(bake_curve(curve, placement)?),
        Geometry::Surface(surface) => {
            let mut baked = surface.clone();
            for pole in &mut baked.poles {
                *pole = placement.point(*pole)?;
            }
            Geometry::Surface(baked)
        }
        Geometry::Mesh(mesh) => {
            let mut baked = mesh.clone();
            for vertex in &mut baked.vertices {
                *vertex = placement.point(*vertex)?;
            }
            for normal in &mut baked.normals {
                *normal = transform_normal(placement, *normal)?;
            }
            Geometry::Mesh(baked)
        }
        Geometry::Cloud(cloud) => {
            let mut baked = cloud.clone();
            for point in &mut baked.points {
                *point = placement.point(*point)?;
            }
            for normal in &mut baked.normals {
                *normal = transform_normal(placement, *normal)?;
            }
            Geometry::Cloud(baked)
        }
        _ => return Err(Error::Unsupported("mirror for this geometry kind")),
    })
}

fn reflect_point(point: Point3, origin: Point3, normal: Point3) -> Point3 {
    sub(point, scale(normal, 2.0 * dot(sub(point, origin), normal)))
}

fn reflect_direction(direction: Point3, normal: Point3) -> Point3 {
    sub(direction, scale(normal, 2.0 * dot(direction, normal)))
}

fn reflect_geometry(geometry: Geometry, origin: Point3, normal: Point3) -> Result<Geometry> {
    Ok(match geometry {
        Geometry::Point(point) => Geometry::Point(reflect_point(point, origin, normal)),
        Geometry::Curve(curve) => Geometry::Curve(match curve {
            Curve::Line { start, end, domain } => {
                let start = reflect_point(start, origin, normal);
                let end = reflect_point(end, origin, normal);
                Curve::Line {
                    start,
                    end,
                    domain,
                }
            }
            Curve::Circle {
                center,
                x,
                y,
                radius,
                domain,
                angle_domain,
            } => Curve::Circle {
                center: reflect_point(center, origin, normal),
                x: sub(x, scale(normal, 2.0 * dot(x, normal))),
                y: sub(y, scale(normal, 2.0 * dot(y, normal))),
                radius,
                domain,
                angle_domain,
            },
            Curve::Nurbs(mut nurbs) => {
                for pole in &mut nurbs.poles {
                    *pole = reflect_point(*pole, origin, normal);
                }
                Curve::Nurbs(nurbs)
            }
            Curve::Polycurve { segments, reversed } => {
                let mut reflected = Vec::new();
                for segment in segments {
                    reflected.push(
                        match reflect_geometry(Geometry::Curve(segment), origin, normal)? {
                            Geometry::Curve(curve) => curve,
                            _ => return Err(Error::Invalid("mirror polycurve segment")),
                        },
                    );
                }
                Curve::Polycurve {
                    segments: reflected,
                    reversed,
                }
            }
        }),
        Geometry::Surface(mut surface) => {
            for pole in &mut surface.poles {
                *pole = reflect_point(*pole, origin, normal);
            }
            Geometry::Surface(surface)
        }
        Geometry::Mesh(mut mesh) => {
            for vertex in &mut mesh.vertices {
                *vertex = reflect_point(*vertex, origin, normal);
            }
            for direction in &mut mesh.normals {
                *direction = reflect_direction(*direction, normal);
            }
            // Reverse each boundary and its corner attributes together. A quad
            // needs the entire tail reversed to avoid crossing its edges.
            let mut uv_offset = 0;
            for face in &mut mesh.faces {
                let end = uv_offset + face.len();
                if face.len() >= 3 {
                    face[1..].reverse();
                    if !mesh.corner_uvs.is_empty() {
                        mesh.corner_uvs[uv_offset + 1..end].reverse();
                    }
                }
                uv_offset = end;
            }
            Geometry::Mesh(mesh)
        }
        Geometry::Cloud(mut cloud) => {
            for point in &mut cloud.points {
                *point = reflect_point(*point, origin, normal);
            }
            for direction in &mut cloud.normals {
                *direction = reflect_direction(*direction, normal);
            }
            Geometry::Cloud(cloud)
        }
        _ => return Err(Error::Unsupported("mirror for this geometry kind")),
    })
}

/// Result of a mirror: either fresh copies (default) or replacement geometry on the
/// same identities (`copy = false`).
#[derive(Debug, Clone, PartialEq)]
pub struct MirrorOutcome {
    pub copies: Vec<Entity>,
    pub replacements: Vec<(ObjectId, Geometry)>,
    pub placements: Vec<(ObjectId, Transform)>,
}

/// Mirror selected objects about a world plane.
pub fn mirror_entities(
    document: &Document,
    ids: &BTreeSet<ObjectId>,
    origin: Point3,
    normal: Point3,
    copy: bool,
    limits: &Limits,
) -> Result<MirrorOutcome> {
    finite(origin)?;
    let normal = unit(normal)?;
    let mut outcome = MirrorOutcome {
        copies: Vec::new(),
        replacements: Vec::new(),
        placements: Vec::new(),
    };
    for id in ids {
        let entity = document.entities.get(id).ok_or(Error::Missing("object"))?;
        if matches!(entity.geometry, Geometry::Instance(_) | Geometry::NativeBrep(_) | Geometry::Hatch(_) | Geometry::Light(_)) {
            let mut matrix=Transform::identity();
            let distance=dot(origin,normal);
            for i in 0..3 {
                for j in 0..3 {matrix.0[i][j]-=2.*normal[i]*normal[j];}
                matrix.0[i][3]=2.*distance*normal[i];
            }
            let placement=matrix.compose(entity.placement)?;
            if copy {outcome.copies.push(clone_entity(entity,entity.geometry.clone(),placement));}
            else {outcome.placements.push((*id,placement));}
            continue;
        }
        let baked = bake_geometry(&entity.geometry, entity.placement, limits)?;
        let reflected = reflect_geometry(baked, origin, normal)?;
        reflected.validate(limits)?;
        if copy {
            outcome
                .copies
                .push(clone_entity(entity, reflected, Transform::identity()));
        } else {
            // Keep the object's placement: express the reflected world geometry back in
            // its local frame so one Geometry write is enough.
            let inverse = inverse_affine(entity.placement)?;
            let local = bake_geometry(&reflected, inverse, limits)?;
            outcome.replacements.push((*id, local));
        }
    }
    Ok(outcome)
}

/* -------------------------------- grips -------------------------------- */

/// Surface edit points: the surface point at each Greville abscissa pair. Periodic and
/// reversed surfaces are included because the grip moves the corresponding pole, not a
/// re-interpolated replacement record.
const MAX_SURFACE_EDIT_GRIPS: usize = 10_000;

fn surface_edit_points(
    surface: &crate::geometry::NurbsSurface,
    limits: &Limits,
) -> Result<Vec<Point3>> {
    let count = surface.size[0]
        .checked_mul(surface.size[1])
        .ok_or(Error::Overflow("surface edit grips"))?;
    if count > MAX_SURFACE_EDIT_GRIPS {
        return Err(Error::Budget(
            "surface edit grips are limited to 10,000 points; use control points",
        ));
    }
    surface_kernel::edit_points(surface, limits)
}

/// The rational basis coefficient of pole `(i, j)` at its own Greville point:
/// `N_i(u_i) N_j(v_j) w_ij / W(u_i, v_j)`. Moving only that pole by `delta / alpha`
/// moves the edit point by exactly `delta` because the weights and denominator do not
/// change.
fn surface_edit_influence(
    surface: &crate::geometry::NurbsSurface,
    i: usize,
    j: usize,
    limits: &Limits,
) -> Result<f64> {
    surface_kernel::edit_influence(surface, i, j, limits)
}

/// One addressable grip: the object, its index in the selected mode and the world point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GripHandle {
    pub object: ObjectId,
    pub index: usize,
    pub mode: GripMode,
    pub point: Point3,
}

/// World-space grips for the selected curves.
pub fn grip_handles(
    document: &Document,
    ids: &BTreeSet<ObjectId>,
    mode: GripMode,
    limits: &Limits,
) -> Result<Vec<GripHandle>> {
    let mut handles = Vec::new();
    let mut compatible = false;
    for id in ids {
        let entity = document.entities.get(id).ok_or(Error::Missing("object"))?;
        let source: Vec<Point3> = match &entity.geometry {
            Geometry::Curve(curve) => {
                compatible = true;
                let grips = curve_ops::curve_grips(curve, 1e-9, limits)?;
                match mode {
                    GripMode::Edit => grips.edit_points,
                    GripMode::Control => grips.poles,
                }
            }
            Geometry::Surface(surface) => {
                compatible = true;
                match mode {
                    GripMode::Edit => surface_edit_points(surface, limits)?,
                    GripMode::Control => surface.poles.clone(),
                }
            },
            _ => continue,
        };
        handles
            .try_reserve(source.len())
            .map_err(|_| Error::Budget("grip points"))?;
        for (index, point) in source.iter().enumerate() {
            handles.push(GripHandle {
                object: *id,
                index,
                mode,
                point: entity.placement.point(*point)?,
            });
        }
    }
    if !ids.is_empty() && !compatible { return Err(Error::Unsupported("grips need a curve or surface")); }
    Ok(handles)
}

/// World-space grip points for the selected curves.
pub fn grip_points(
    document: &Document,
    ids: &BTreeSet<ObjectId>,
    mode: GripMode,
    limits: &Limits,
) -> Result<Vec<Point3>> {
    Ok(grip_handles(document, ids, mode, limits)?
        .into_iter()
        .map(|handle| handle.point)
        .collect())
}

/// Return geometry after deleting a control grip; callers publish it atomically.
pub fn delete_grip_geometry(document: &Document, handle: GripHandle, limits: &Limits) -> Result<Geometry> {
    if handle.mode != GripMode::Control { return Err(Error::Invalid("control grip required")); }
    let entity = document.entities.get(&handle.object).ok_or(Error::Missing("object"))?;
    let Geometry::Curve(curve) = &entity.geometry else { return Err(Error::Unsupported("control point deletion needs a NURBS curve")); };
    Ok(Geometry::Curve(curve_ops::delete_control_point(curve, handle.index, limits)?))
}

/// Move one grip to a world point and return the edited geometry.
pub fn move_grip_geometry(
    document: &Document,
    handle: GripHandle,
    world: Point3,
    limits: &Limits,
) -> Result<Geometry> {
    finite(world)?;
    let entity = document
        .entities
        .get(&handle.object)
        .ok_or(Error::Missing("object"))?;
    let local = inverse_affine(entity.placement)?.point(world)?;
    match &entity.geometry {
        Geometry::Curve(curve) => {
            let edited = if matches!(curve, Curve::Polycurve { .. }) {
                let tolerance = document.effective_tolerance()?.absolute_mm;
                match handle.mode {
                    GripMode::Control => curve_ops::move_grip_with_joints(curve, handle.index, local, entity.placement, tolerance, limits)?,
                    GripMode::Edit => curve_ops::move_edit_point_with_joints(curve, handle.index, local, entity.placement, tolerance, limits)?,
                }
            } else {
                match handle.mode {
                    GripMode::Control => curve_ops::move_grip(curve, handle.index, local, limits)?,
                    GripMode::Edit => curve_ops::move_edit_point(curve, handle.index, local, limits)?,
                }
            };
            let edited = if handle.mode == GripMode::Control && matches!(curve, Curve::Polycurve { .. }) {
                let tolerance = document.effective_tolerance()?;
                curve_ops::fuse_smooth_control_polycurve(curve, &edited, entity.placement, tolerance.absolute_mm, tolerance.angular_rad, limits)?.unwrap_or(edited)
            } else { edited };
            edited.validate(limits)?;
            Ok(Geometry::Curve(edited))
        }
        Geometry::Surface(surface) => {
            if handle.index >= surface.poles.len() {
                return Err(Error::Invalid("surface grip index"));
            }
            let mut edited = surface.clone();
            match handle.mode {
                GripMode::Control => {
                    edited.poles[handle.index] = local;
                }
                GripMode::Edit => {
                    let row = handle.index / surface.size[1];
                    let column = handle.index % surface.size[1];
                    let influence = surface_edit_influence(surface, row, column, limits)?;
                    let current = surface_kernel::edit_point(surface, row, column, limits)?;
                    let delta = [
                        (local[0] - current[0]) / influence,
                        (local[1] - current[1]) / influence,
                        (local[2] - current[2]) / influence,
                    ];
                    edited.poles[handle.index] = [
                        surface.poles[handle.index][0] + delta[0],
                        surface.poles[handle.index][1] + delta[1],
                        surface.poles[handle.index][2] + delta[2],
                    ];
                }
            }
            edited.validate(limits)?;
            Ok(Geometry::Surface(edited))
        }
        _ => Err(Error::Unsupported("grips need a curve or surface")),
    }
}

/* ------------------------- split / trim by cutter ------------------------- */

/// Closure for cutting: a short line or a small open circular arc is not a loop.
pub fn curve_closed_for_cut(curve: &Curve, limits: &Limits) -> Result<bool> {
    curve.validate(limits)?;
    match curve {
        Curve::Line { .. } => Ok(false),
        Curve::Circle { radius, domain, angle_domain, .. } => {
            let angles = angle_domain.unwrap_or(*domain);
            let span = angles[1] - angles[0];
            let tau = std::f64::consts::TAU;
            let tolerance = 1e-9_f64.min(1e-9 / radius);
            if span < tau - tolerance { return Ok(false); }
            let residual = span.rem_euclid(tau);
            Ok(residual.min(tau-residual) <= tolerance)
        }
        Curve::Nurbs(n) if n.seam.is_some() => {
            let mut base = n.clone(); base.seam = None;
            curve_closed_for_cut(&Curve::Nurbs(base), limits)
        }
        _ => {
            let start = curve_ops::point_at(curve, 0.0, limits)?;
            let end = curve_ops::point_at(curve, 1.0, limits)?;
            let separation = (start[0]-end[0]).hypot(start[1]-end[1]).hypot(start[2]-end[2]);
            if separation == 0.0 { return Ok(true); }
            if separation > 1e-9 { return Ok(false); }
            // Do not let a fixed absolute threshold classify an entire small
            // open curve as closed. Sampling gives a conservative size bound.
            let mut extent = separation;
            for fraction in [0.25, 0.5, 0.75] {
                let p = curve_ops::point_at(curve, fraction, limits)?;
                extent = extent.max((start[0]-p[0]).hypot(start[1]-p[1]).hypot(start[2]-p[2]));
            }
            Ok(separation <= 1e-9_f64.min(extent * 1e-9))
        }
    }
}

fn intersection_fractions(
    target: &Curve,
    cutter: &Curve,
    tolerance: f64,
    include_endpoints: bool,
    limits: &Limits,
) -> Result<Vec<f64>> {
    if !tolerance.is_finite() || tolerance <= 0.0 {
        return Err(Error::Invalid("cutting tolerance"));
    }
    let hits = curve_surface_ops::curve_curve_intersections(target, cutter, tolerance, limits)?;
    let fractions: Vec<f64> = hits
        .into_iter()
        .map(|hit| hit.first_fraction)
        .collect();
    let fractions = normalize_cut_fractions(target, Transform::identity(), fractions, tolerance, include_endpoints, limits)?;
    if fractions.is_empty() {
        return Err(Error::Invalid("cutting curve has no intersection"));
    }
    Ok(fractions)
}

/// Sort intersection fractions and merge only cuts within tolerance along the placed curve.
pub fn normalize_cut_fractions(
    curve: &Curve,
    placement: Transform,
    mut fractions: Vec<f64>,
    tolerance: f64,
    include_endpoints: bool,
    limits: &Limits,
) -> Result<Vec<f64>> {
    if !tolerance.is_finite() || tolerance <= 0.0 { return Err(Error::Invalid("cutting tolerance")); }
    if fractions.iter().any(|f| !f.is_finite() || *f < 0.0 || *f > 1.0) { return Err(Error::Invalid("cutting fraction")); }
    fractions.sort_by(f64::total_cmp);
    let mut unique: Vec<f64> = Vec::new();
    unique.try_reserve_exact(fractions.len()).map_err(|_| Error::Budget("cut fractions"))?;
    let within_tolerance = |a: f64, b: f64| -> Result<bool> {
        if a == b { return Ok(true); }
        let pa=placement.point(curve_ops::point_at(curve,a,limits)?)?;
        let pb=placement.point(curve_ops::point_at(curve,b,limits)?)?;
        let distance=(pa[0]-pb[0]).hypot(pa[1]-pb[1]).hypot(pa[2]-pb[2]);
        if distance > tolerance { return Ok(false); }
        if matches!(curve, Curve::Line { .. }) { return Ok(true); }
        if let Curve::Nurbs(n)=curve && let Some(seam)=n.seam {
            let offset=if n.reversed {(n.domain[1]-seam)/(n.domain[1]-n.domain[0])} else {(seam-n.domain[0])/(n.domain[1]-n.domain[0])};
            // The two fractions may round to exactly the same unwrapped native
            // parameter; trimming that zero-representable interval is invalid.
            if offset+a == offset+b { return Ok(true); }
        }
        let interval=curve_ops::trim_curve_by_parameter(curve,a,b,limits)?;
        let world=bake_curve(&interval,placement)?;
        Ok(curve_ops::arc_length(&world,limits)? <= tolerance)
    };
    for mut fraction in fractions {
        let endpoint=if within_tolerance(0.0,fraction)? { Some(0.0) }
            else if within_tolerance(fraction,1.0)? { Some(1.0) } else { None };
        if let Some(end)=endpoint {
            if !include_endpoints { continue; }
            fraction=end;
        }
        if let Some(&previous) = unique.last() {
            if within_tolerance(previous,fraction)? { continue; }
        }
        unique.push(fraction);
    }
    Ok(unique)
}

fn cut_pieces(target: &Curve, fractions: &[f64], limits: &Limits) -> Result<Vec<Curve>> {
    let mut boundaries = Vec::with_capacity(fractions.len() + 2);
    boundaries.push(0.0);
    boundaries.extend_from_slice(fractions);
    boundaries.push(1.0);
    let mut pieces = Vec::new();
    for pair in boundaries.windows(2) {
        if pair[1] <= pair[0] {
            continue;
        }
        let piece = curve_ops::trim_curve_by_parameter(target, pair[0], pair[1], limits)?;
        if curve_ops::arc_length(&piece, limits)? > 0.0 {
            pieces.push(piece);
        }
    }
    if pieces.len() < 2 {
        return Err(Error::Invalid("cut did not split the curve"));
    }
    Ok(pieces)
}

/// Split a curve at explicit interior fractions.
pub fn split_at_fractions(
    target: &Curve,
    fractions: &[f64],
    limits: &Limits,
) -> Result<Vec<Curve>> {
    target.validate(limits)?;
    let mut clean: Vec<f64> = fractions
        .iter()
        .copied()
        .filter(|fraction| fraction.is_finite() && *fraction > 0.0 && *fraction < 1.0)
        .collect();
    clean.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    clean.dedup_by(|a, b| *a == *b);
    if clean.is_empty() {
        return Err(Error::Invalid("no cutting fraction"));
    }
    cut_pieces(target, &clean, limits)
}

/// Split a closed curve at distinct fractions, including the interval across its seam.
pub fn split_closed_at_fractions(
    target: &Curve,
    fractions: &[f64],
    limits: &Limits,
) -> Result<Vec<Curve>> {
    if !curve_closed_for_cut(target, limits)? {
        return Err(Error::Invalid("closed split needs a closed curve"));
    }
    let mut cuts = fractions.to_vec();
    if cuts.iter().any(|f| !f.is_finite() || *f < 0.0 || *f >= 1.0) {
        return Err(Error::Invalid("closed split fraction"));
    }
    cuts.sort_by(f64::total_cmp);
    cuts.dedup_by(|a, b| *a == *b);
    if cuts.len() < 2 {
        return Err(Error::Invalid("closed curve needs two distinct points"));
    }
    let mut pieces = Vec::with_capacity(cuts.len());
    for pair in cuts.windows(2) {
        pieces.push(curve_ops::trim_curve_by_parameter(target, pair[0], pair[1], limits)?);
    }
    let tail = curve_ops::trim_curve_by_parameter(target, *cuts.last().unwrap(), 1.0, limits)?;
    if cuts[0] == 0.0 {
        pieces.push(tail);
    } else {
        let head = curve_ops::trim_curve_by_parameter(target, 0.0, cuts[0], limits)?;
        pieces.push(curve_ops::join_curves(&[tail, head], 1e-7, limits)?);
    }
    Ok(pieces)
}

/// Split a target curve at every intersection with a cutting curve.
pub fn split_by_cutter(
    target: &Curve,
    cutter: &Curve,
    tolerance: f64,
    limits: &Limits,
) -> Result<Vec<Curve>> {
    let fractions = intersection_fractions(target, cutter, tolerance, false, limits)?;
    split_at_fractions(target, &fractions, limits)
}

/// Remove the interval nearest `pick` and return the surviving pieces.
pub fn trim_by_fractions(
    target: &Curve,
    fractions: &[f64],
    pick: Point3,
    limits: &Limits,
) -> Result<Vec<Curve>> {
    trim_by_fractions_in_world(target, fractions, pick, Transform::identity(), limits)
}

/// Choose the picked interval in world space while retaining each piece in its source frame.
pub fn trim_by_fractions_in_world(
    target: &Curve,
    fractions: &[f64],
    pick: Point3,
    placement: Transform,
    limits: &Limits,
) -> Result<Vec<Curve>> {
    finite(pick)?;
    target.validate(limits)?;
    let closed = curve_closed_for_cut(target, limits)?;
    let seam_cut = closed && fractions.iter().any(|f| *f == 0.0 || *f == 1.0);
    let mut fractions: Vec<f64> = fractions
        .iter()
        .copied()
        .filter(|fraction| fraction.is_finite() && *fraction > 0.0 && *fraction < 1.0)
        .collect();
    fractions.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    fractions.dedup_by(|a, b| *a == *b);
    if fractions.is_empty() {
        return Err(Error::Invalid("no cutting fraction"));
    }
    let mut boundaries = Vec::with_capacity(fractions.len() + 2);
    boundaries.push(0.0);
    boundaries.extend_from_slice(&fractions);
    boundaries.push(1.0);
    let mut remove = None;
    let mut best = f64::INFINITY;
    for (index, pair) in boundaries.windows(2).enumerate() {
        let piece = curve_ops::trim_curve_by_parameter(target, pair[0], pair[1], limits)?;
        let world_piece = bake_curve(&piece, placement)?;
        let (_, _, measured) = curve_surface_ops::closest_point_on_curve(&world_piece, pick, limits)?;
        if measured < best {
            best = measured;
            remove = Some(index);
        }
    }
    let remove = remove.ok_or(Error::Invalid("trim interval"))?;
    // On a closed curve the first and last parameter intervals are the same
    // cyclic interval, separated only by the representation seam.
    if closed && !seam_cut && fractions.len() >= 2 && (remove == 0 || remove == boundaries.len() - 2) {
        let piece = curve_ops::trim_curve_by_parameter(target, fractions[0], *fractions.last().unwrap(), limits)?;
        if curve_ops::arc_length(&piece, limits)? <= 0.0 { return Err(Error::Invalid("trim removed the whole curve")); }
        return Ok(vec![piece]);
    }
    let mut pieces = Vec::new();
    for (from, to) in [(0.0, boundaries[remove]), (boundaries[remove + 1], 1.0)] {
        if from < to {
            let piece = curve_ops::trim_curve_by_parameter(target, from, to, limits)?;
            if curve_ops::arc_length(&piece, limits)? > 0.0 {
                pieces.push(piece);
            }
        }
    }
    if pieces.is_empty() {
        return Err(Error::Invalid("trim removed the whole curve"));
    }
    if pieces.len() == 2 && closed {
        let tail = pieces.pop().expect("two pieces");
        let head = pieces.pop().expect("two pieces");
        pieces.push(curve_ops::join_curves(&[tail, head], 1e-7, limits)?);
    }
    Ok(pieces)
}

/// Remove the interval nearest `pick` after intersecting with a cutting curve.
pub fn trim_by_cutter(
    target: &Curve,
    cutter: &Curve,
    pick: Point3,
    tolerance: f64,
    limits: &Limits,
) -> Result<Vec<Curve>> {
    let closed = curve_closed_for_cut(target, limits)?;
    let fractions = intersection_fractions(target, cutter, tolerance, closed, limits)?;
    trim_by_fractions(target, &fractions, pick, limits)
}

/* ------------------------------- defaults ------------------------------- */

/// The duration unit used by the quick trim variant (10% off each end).
pub const QUICK_TRIM: [f64; 2] = [0.1, 0.9];

/// Split fraction used by the quick Split button.
pub const QUICK_SPLIT: f64 = 0.5;
