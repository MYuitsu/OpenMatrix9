//! Duplicate Edge / Extract Wire Frame (`DupEdge`, `ExtractWire`, OM9-CURVE-040/054).

use om9_core::geometry::{Curve, Geometry};

use super::{Controller, Error};

pub const DUP_EDGE: &str = "OM9-CURVE-040";
pub const EXTRACT_WIRE: &str = "OM9-CURVE-054";

#[cfg(feature = "occt-brep")]
pub(super) fn edge_extraction_error(error:om9_kernel::editing::EditingError,label:&str)->Error {
    match error {
        om9_kernel::editing::EditingError::Budget(message) if message=="edge curve allocation" => Error::Budget("bytes"),
        error=>Error::Io(format!("{label}: {error}")),
    }
}

#[cfg(feature = "occt-brep")]
fn selected_source_edge_with_admission(
    brep: &dyn om9_core::brep::BrepSourceAccess, point:om9_core::geometry::Point3, tolerance:f64,
    segment:[om9_core::geometry::Point3;2], segment_tolerance:f64,
    limits:&om9_core::budget::Limits, temporary:bool, refresh:&mut bool,
    admit:&mut impl FnMut(u64,u64)->Result<(),Error>,
) -> Result<(Curve,om9_core::geometry::Point3),Error> {
    let mut refused=None;
    let mut guarded=|live,pending| match admit(live,pending) {
        Ok(())=>Ok(()),Err(error)=>{refused=Some(error.clone());Err(error)}
    };
    om9_kernel::editing::source_edge_curve_at_segment_admitted(
        brep,point,tolerance,segment,segment_tolerance,limits,&mut guarded,
    ).map_err(|error| {
        if let Some(error)=refused {return error;}
        *refresh=!temporary && matches!(&error,
            om9_kernel::editing::EditingError::Kernel(message) if message=="source edge pick: no edge within tolerance");
        match error {
            om9_kernel::editing::EditingError::Budget(message) if message=="edge curve allocation"=>Error::Budget("bytes"),
            error=>Error::Io(format!("DupEdge cannot verify the displayed edge against the source; no curve was created ({error})")),
        }
    })
}

#[cfg(feature = "occt-brep")]
fn selected_native_source_edge_with_admission(
    source:&str,point:om9_core::geometry::Point3,tolerance:f64,segment:[om9_core::geometry::Point3;2],segment_tolerance:f64,
    limits:&om9_core::budget::Limits,admit:&mut impl FnMut(u64,u64)->Result<(),Error>,
) -> Result<(Curve,om9_core::geometry::Point3),Error> {
    let mut refused=None;
    let mut guarded=|live,pending|match admit(live,pending) {Ok(())=>Ok(()),Err(error)=>{refused=Some(error.clone());Err(error)}};
    om9_kernel::editing::source_edge_from_text_segment_admitted(source,point,tolerance,segment,segment_tolerance,limits,&mut guarded)
        .map_err(|error|refused.unwrap_or_else(||edge_extraction_error(error,"DupEdge cannot verify the displayed edge against native source")))
}

#[cfg(feature = "occt-brep")]
pub(super) fn source_edges_with_admission(
    brep: &dyn om9_core::brep::BrepSourceAccess, limits: &om9_core::budget::Limits, boundary: bool,
    admit: &mut impl FnMut(u64,u64) -> Result<(),Error>,
) -> Result<Vec<Curve>,Error> {
    // Keep the original callback error; the adapter formats it as text.
    let mut refused=None;
    let mut guarded=|live,pending| match admit(live,pending) {
        Ok(())=>Ok(()),Err(error)=>{refused=Some(error.clone());Err(error)}
    };
    let result=if boundary {
        om9_kernel::editing::source_boundary_edges_nurbs_admitted(brep,limits,&mut guarded)
    } else {om9_kernel::editing::all_source_edges_nurbs_admitted(brep,limits,&mut guarded)};
    result.map_err(|error| refused.unwrap_or_else(|| edge_extraction_error(error,if boundary {"DupBorder"} else {"ExtractWire"})))
}

#[cfg(feature = "occt-brep")]
pub(super) fn source_wires_with_admission(source:&str,limits:&om9_core::budget::Limits,
    admit:&mut impl FnMut(u64,u64)->Result<(),Error>) -> Result<Vec<Curve>,Error> {
    use om9_core::budget::Usage;
    let mut refused=None;
    let mut guarded=|live,pending| match admit(live,pending) {
        Ok(())=>Ok(()),Err(error)=>{refused=Some(error.clone());Err(error)}
    };
    let mut edges=om9_kernel::editing::all_edges_from_source_admitted(source,limits,&mut guarded)
        .map_err(|error|refused.take().unwrap_or_else(||edge_extraction_error(error,"ExtractWire edges")))?;
    let bytes=|curves:&Vec<Curve>| -> Result<u64,Error> {
        curves.iter().try_fold(Usage::array::<Curve>(curves.capacity())?.bytes,|sum,curve|
            sum.checked_add(curve.validate(limits)?.bytes).ok_or(Error::Overflow("ExtractWire curves")))
    };
    let edge_bytes=bytes(&edges)?;
    let mut refused=None;
    let mut guarded=|live:u64,pending| {
        let result=edge_bytes.checked_add(live).ok_or(Error::Overflow("ExtractWire curves"))
            .and_then(|total|admit(total,pending));
        if let Err(error)=&result {refused=Some(error.clone());} result
    };
    let mut isos=om9_kernel::editing::default_face_isocurves_from_source_admitted(source,limits,&mut guarded)
        .map_err(|error|refused.take().unwrap_or_else(||edge_extraction_error(error,"ExtractWire face wires")))?;
    let count=edges.len().checked_add(isos.len()).ok_or(Error::Overflow("ExtractWire outputs"))?;
    if u64::try_from(count).map_err(|_|Error::Overflow("ExtractWire outputs"))?>limits.max_outputs {
        return Err(Error::Budget("ExtractWire outputs"));
    }
    let held=edge_bytes.checked_add(bytes(&isos)?).ok_or(Error::Overflow("ExtractWire curves"))?;
    let new_slots=Usage::array::<Curve>(count)?.bytes;
    admit(held,new_slots)?;
    reserve_wire(&mut edges,isos.len())?;
    let held=bytes(&edges)?.checked_add(bytes(&isos)?).ok_or(Error::Overflow("ExtractWire curves"))?;
    admit(held,0)?;
    edges.append(&mut isos);
    drop(isos);
    admit(bytes(&edges)?,0)?;
    Ok(edges)
}

#[cfg(feature = "occt-brep")]
pub(super) fn root_scene_has_interior_wires(scene:&om9_scene::scene::Scene,stamp:&om9_core::identity::SourceStamp) -> Result<bool,Error> {
    let object=scene.objects.iter().find(|object|object.root==stamp.object && object.path.is_empty())
        .ok_or(Error::Missing("ExtractWireframe scene object"))?;
    scene.root_brep_pick_display(stamp)?;
    scene.brep_occurrence_has_interior_wires(object)
}

pub(super) fn reserve_wire<T>(items: &mut Vec<T>, additional: usize) -> Result<(), Error> {
    let count = items.len().checked_add(additional).ok_or(Error::Overflow("ExtractWire allocation"))?;
    std::alloc::Layout::array::<T>(count).map_err(|_| Error::Overflow("ExtractWire allocation"))?;
    items.try_reserve_exact(additional).map_err(|_| Error::Budget("bytes"))
}

fn is_bilinear_trim_surface(surface: &om9_core::geometry::NurbsSurface) -> bool {
    if surface.degree != [1, 1] || surface.size != [2, 2] || surface.periodic != [false, false]
        || surface.poles.len() != 4 || surface.weights.len()!=4 || surface.weights.iter().any(|w|!w.is_finite()||*w<=0.) {
        return false;
    }
    for axis in 0..2 {
        let k = &surface.knots[axis]; let d = surface.domain[axis];
        if k.len() != 4 || !k.iter().all(|v| v.is_finite()) || k[0] != k[1] || k[2] != k[3]
            || k[0] >= k[2] || !(k[2] - k[0]).is_finite()
            || !d.iter().all(|v| v.is_finite()) || d[0] < k[0] || d[1] > k[2] || d[0] >= d[1] {
            return false;
        }
    }
    true
}

pub(super) fn is_affine_trim_plane(surface: &om9_core::geometry::NurbsSurface) -> bool {
    if !is_bilinear_trim_surface(surface) || surface.weights.as_slice()!=[1.,1.,1.,1.] {return false;}
    // TwoSum preserves the rounding residual, so equal rounded sums alone do
    // not accidentally classify a warped bilinear surface as affine.
    let sum = |a: f64, b: f64| {
        let hi = a + b;
        let z = hi - a;
        (hi, (a - (hi - z)) + (b - z))
    };
    (0..3).all(|axis| sum(surface.poles[0][axis], surface.poles[3][axis])
        == sum(surface.poles[1][axis], surface.poles[2][axis]))
}

pub(super) fn validate_trim_closed_seams(surface: &om9_core::geometry::NurbsSurface,
    trim: &om9_core::rhino_semantics::TrimmedSurface, limits: &om9_core::budget::Limits) -> Result<Option<usize>,Error> {
    let mut certified=None;
    for axis in 0..2 {
        if super::dup_border::closed_surface_axis(surface,axis,limits)?
            && trim.outer.iter().any(|p|p[axis]==surface.domain[axis][0])
            && trim.outer.iter().any(|p|p[axis]==surface.domain[axis][1]) {
            let other=1-axis;let low=trim.outer.iter().map(|p|p[other]).fold(f64::INFINITY,f64::min);
            let high=trim.outer.iter().map(|p|p[other]).fold(f64::NEG_INFINITY,f64::max);
            let ends=surface.domain[axis];
            let rectangular=trim.outer.len()==4 && low<high && trim.outer.iter().enumerate().all(|(i,p)| {
                let q=trim.outer[(i+1)%4];
                (p[axis]==ends[0]||p[axis]==ends[1]) && (p[other]==low||p[other]==high)
                    && ((p[axis]==q[axis]) != (p[other]==q[other]))
            });
            let holes_clear=trim.inners.iter().flatten().all(|p|p[axis]>ends[0]&&p[axis]<ends[1]);
            if !rectangular || !holes_clear || certified.is_some() {
                return Err(Error::Unsupported("trim spans a closed surface seam"));
            }
            certified=Some(axis);
        }
    }
    Ok(certified)
}

// Resolve every occurrence from the document, not from its display mesh. A
// selected block is all-or-nothing, including inaccessible or unsupported members.
#[cfg(feature = "occt-brep")]
fn collect_wireframe_occurrences<'a>(
    document: &'a om9_core::model::Document,
    scene: &om9_scene::scene::Scene,
    root: om9_core::identity::ObjectId,
    path: &mut Vec<om9_core::identity::ObjectId>,
    entity: &'a om9_core::model::Entity,
    parent: om9_core::geometry::Transform,
    depth: usize,
    limits: &om9_core::budget::Limits,
    nodes: &mut u64,
    occurrences: &mut Vec<(&'a om9_core::model::Entity, om9_core::geometry::Transform)>,
    retained: &std::cell::Cell<u64>,
    admit: &mut impl FnMut(u64, u64) -> Result<(), Error>,
) -> Result<(), Error> {
    use om9_core::budget::Usage;
    if depth > limits.max_depth { return Err(Error::Budget("ExtractWire instance depth")); }
    *nodes = nodes.checked_add(1).ok_or(Error::Overflow("ExtractWire instances"))?;
    if *nodes > limits.max_nodes { return Err(Error::Budget("ExtractWire instances")); }
    let permissions = document.permissions_for_entity(entity, false)?;
    if !permissions.visible || !permissions.selectable {
        return Err(Error::Invalid("ExtractWireframe block member is not selectable"));
    }
    let placement = parent.compose(entity.placement)?;
    if let Geometry::Instance(id) = entity.geometry {
        let definition = document.definitions.get(&id).ok_or(Error::Missing("ExtractWire definition"))?;
        if definition.members.is_empty() { return Err(Error::Invalid("ExtractWireframe empty block")); }
        for member in &definition.members {
            if path.len() == path.capacity() {
                let old = Usage::array::<om9_core::identity::ObjectId>(path.capacity())?.bytes;
                let target = path.capacity().max(4).checked_mul(2).ok_or(Error::Overflow("ExtractWire occurrence path"))?;
                let requested = Usage::array::<om9_core::identity::ObjectId>(target)?.bytes;
                admit(0, requested.checked_mul(2).ok_or(Error::Overflow("ExtractWire occurrence path"))?)?;
                path.try_reserve_exact(target - path.len()).map_err(|_| Error::Budget("bytes"))?;
                let actual = Usage::array::<om9_core::identity::ObjectId>(path.capacity())?.bytes;
                admit(0, actual)?;
                retained.set(retained.get().checked_add(actual - old).ok_or(Error::Overflow("ExtractWire occurrence path"))?);
                admit(0, 0)?;
            }
            path.push(member.id);
            let result = collect_wireframe_occurrences(document, scene, root, path, member, placement, depth + 1, limits, nodes, occurrences, retained, admit);
            path.pop();
            result?;
        }
        return Ok(());
    }
    if !matches!(entity.geometry, Geometry::Brep(_) | Geometry::NativeBrep(_) | Geometry::Mesh(_) | Geometry::Surface(_)) {
        return Err(Error::Invalid("ExtractWireframe unsupported block member"));
    }
    if occurrences.len() as u64 >= limits.max_outputs { return Err(Error::Budget("ExtractWire occurrences")); }
    if occurrences.len() == occurrences.capacity() {
        let old = Usage::array::<(&om9_core::model::Entity, om9_core::geometry::Transform)>(occurrences.capacity())?.bytes;
        let target = occurrences.capacity().max(4).checked_mul(2).ok_or(Error::Overflow("ExtractWire occurrences"))?;
        let requested = Usage::array::<(&om9_core::model::Entity, om9_core::geometry::Transform)>(target)?.bytes;
        admit(0, requested.checked_mul(2).ok_or(Error::Overflow("ExtractWire occurrences"))?)?;
        occurrences.try_reserve_exact(target - occurrences.len()).map_err(|_| Error::Budget("bytes"))?;
        let actual = Usage::array::<(&om9_core::model::Entity, om9_core::geometry::Transform)>(occurrences.capacity())?.bytes;
        admit(0, actual)?;
        retained.set(retained.get().checked_add(actual - old).ok_or(Error::Overflow("ExtractWire occurrences"))?);
        admit(0, 0)?;
    }
    if matches!(entity.geometry, Geometry::Brep(_)) {
        let mut objects = scene.objects.iter().filter(|object| object.root == root && object.path == *path);
        let object = objects.next().ok_or(Error::Missing("ExtractWireframe BRep occurrence"))?;
        if objects.next().is_some() || object.leaf != entity.id || object.transform != placement {
            return Err(Error::Stale("ExtractWireframe BRep occurrence"));
        }
        scene.brep_occurrence_has_interior_wires(object)?;
    }
    occurrences.push((entity, placement));
    Ok(())
}

pub(super) fn append_trimmed_surface_edges(
    entity: &om9_core::model::Entity,
    items: &mut Vec<(&'static str, Geometry, om9_core::geometry::Transform)>,
    label: &'static str,
    limits: &om9_core::budget::Limits,
    tolerance: f64,
    retained: &std::cell::Cell<u64>,
    admit: &mut impl FnMut(u64, u64) -> Result<(), Error>,
) -> Result<(), Error> {
    append_trimmed_surface_edges_at(entity, entity.placement, items, label, limits, tolerance, retained, admit)
}

fn append_trimmed_surface_edges_at(
    entity: &om9_core::model::Entity,
    placement: om9_core::geometry::Transform,
    items: &mut Vec<(&'static str, Geometry, om9_core::geometry::Transform)>,
    label: &'static str,
    limits: &om9_core::budget::Limits,
    tolerance: f64,
    retained: &std::cell::Cell<u64>,
    admit: &mut impl FnMut(u64, u64) -> Result<(), Error>,
) -> Result<(), Error> {
    use om9_core::{budget::Usage, surface_kernel};
    let Geometry::Surface(surface) = &entity.geometry else { return Err(Error::Invalid("trimmed surface")); };
    surface.validate(limits)?;
    let extension = entity.extensions.get(om9_core::model::TRIM_SURFACE_EXTENSION_SLOT)
        .ok_or(Error::Missing("trim metadata"))?;
    let affine = is_affine_trim_plane(surface);
    let bilinear = is_bilinear_trim_surface(surface);
    // Bound decoding and output slots before parsing; large metadata stays
    // unsupported until streaming trim decoding is available.
    if extension.payload.len() > om9_core::model::trim_surface_payload_limit(limits) { return Err(Error::Unsupported("trim metadata exceeds supported size")); }
    let scratch = u64::try_from(extension.payload.len()).map_err(|_| Error::Overflow("trim metadata"))?
        .checked_mul(64).and_then(|n| n.checked_add(4096)).ok_or(Error::Overflow("trim metadata"))?;
    admit(0, scratch)?;
    let trim = om9_core::rhino_semantics::TrimmedSurface::from_extension(extension, limits)?;
    if trim.underlying != *surface { return Err(Error::Invalid("trimmed surface underlying mismatch")); }
    om9_core::rhino_semantics::validate_trim_loop_topology_for_display(&trim)?;
    let seam_axis=if affine {None}else{validate_trim_closed_seams(surface,&trim,limits)?};
    let count = trim.inners.iter().try_fold(trim.outer.len(), |n, loop_| n.checked_add(loop_.len()))
        .ok_or(Error::Overflow("trim edges"))?;
    if count > om9_core::rhino_semantics::TRIM_DISPLAY_MAX_EDGES {
        return Err(Error::Unsupported("trim boundary count exceeds supported size"));
    }
    let has_diagonal=std::iter::once(&trim.outer).chain(&trim.inners).any(|loop_| (0..loop_.len()).any(|i| {
        let a=loop_[i];let b=loop_[(i+1)%loop_.len()];a[0]!=b[0] && a[1]!=b[1]
    }));
    if !bilinear && has_diagonal {return Err(Error::Unsupported("curved trim edges must follow a source parameter"));}
    let precision_bound=if bilinear && !affine && has_diagonal {
        let scale=surface.poles.iter().flatten().map(|x|x.abs()).fold(1_f64,f64::max);
        let world=placement.0[..3].iter().map(|row|row[3].abs()+(0..3).map(|i|row[i].abs()*scale).sum::<f64>()).fold(1_f64,f64::max);
        let uv=(0..2).map(|axis| {
            let k=&surface.knots[axis];(1.+k[0].abs().max(k[2].abs())/(k[2]-k[0])).max(1.)
        }).fold(1_f64,f64::max);
        let min_weight=surface.weights.iter().copied().fold(f64::INFINITY,f64::min);
        let max_weight=surface.weights.iter().copied().fold(0_f64,f64::max);
        let ratio=max_weight/min_weight;
        let bound=4096.*f64::EPSILON*world*uv*uv*ratio*ratio;
        if !bound.is_finite() || !tolerance.is_finite() || bound>=tolerance {
            return Err(Error::Unsupported("bilinear trim precision is uncertain"));
        }
        bound
    } else {0.};
    let omitted=if seam_axis.is_some() {if label=="DupBorder" {2} else if label=="ExtractWire" {1} else {return Err(Error::Invalid("trim output command"));}} else {0};
    let emitted=count.checked_sub(omitted).ok_or(Error::Overflow("trim edges"))?;
    let total = items.len().checked_add(emitted).ok_or(Error::Overflow("trim edges"))?;
    if total as u64 > limits.max_outputs { return Err(Error::Budget("trim edges")); }
    let old_slots = Usage::array::<(&str, Geometry, om9_core::geometry::Transform)>(items.capacity())?.bytes;
    let slots = Usage::array::<(&str, Geometry, om9_core::geometry::Transform)>(total)?.bytes;
    let curve_reserve = if affine {0} else {
        let points=surface.size[0].max(surface.size[1]).max(3);let knots=surface.knots[0].len().max(surface.knots[1].len()).max(6);
        let curve=Usage::array::<om9_core::geometry::Point3>(points)?.bytes
            .checked_add(Usage::array::<f64>(points)?.bytes).and_then(|n|n.checked_add(Usage::array::<f64>(knots).ok()?.bytes))
            .ok_or(Error::Overflow("trim curves"))?;
        let workspace=Usage::array::<f64>(surface.size[0].checked_add(surface.size[1]).ok_or(Error::Overflow("trim basis"))?)?.bytes
            .checked_mul(8).ok_or(Error::Overflow("trim basis"))?;
        curve.checked_mul(u64::try_from(count).map_err(|_|Error::Overflow("trim curves"))?)
            .and_then(|n|n.checked_mul(2)).and_then(|n|n.checked_add(workspace)).ok_or(Error::Overflow("trim curves"))?
    };
    let scratch=scratch.checked_add(curve_reserve).ok_or(Error::Overflow("trim curves"))?;
    admit(0, scratch.checked_add(slots.checked_mul(2).ok_or(Error::Overflow("trim edges"))?)
        .ok_or(Error::Overflow("trim edges"))?)?;
    reserve_wire(items, count)?;
    let new_slots = Usage::array::<(&str, Geometry, om9_core::geometry::Transform)>(items.capacity())?.bytes;
    admit(0, scratch.checked_add(new_slots).ok_or(Error::Overflow("trim edges"))?)?;
    let mut curve_bytes=0_u64;
    for (loop_index,loop_) in std::iter::once(&trim.outer).chain(&trim.inners).enumerate() {
        for i in 0..loop_.len() {
            let a = loop_[i]; let b = loop_[(i + 1) % loop_.len()];
            if let Some(axis)=seam_axis && loop_index==0 && a[axis]==b[axis]
                && (a[axis]==surface.domain[axis][0] || a[axis]==surface.domain[axis][1])
                && (label=="DupBorder" || a[axis]==surface.domain[axis][1]) {continue;}
            let curve=if affine {
                let start=surface_kernel::evaluate(surface,a[0],a[1],limits)?;
                let end=surface_kernel::evaluate(surface,b[0],b[1],limits)?;
                if start==end {return Err(Error::Invalid("trim edge has zero length"));}
                Curve::Line {start,end,domain:[0.,1.]}
            } else if bilinear && a[0]!=b[0] && a[1]!=b[1] {
                let max_weight=surface.weights.iter().copied().fold(0_f64,f64::max);
                let homogeneous=|uv:[f64;2]| -> Result<[f64;4],Error> {
                    let u=(uv[0]-surface.knots[0][0])/(surface.knots[0][2]-surface.knots[0][0]);
                    let v=(uv[1]-surface.knots[1][0])/(surface.knots[1][2]-surface.knots[1][0]);
                    let basis=[(1.-u)*(1.-v),(1.-u)*v,u*(1.-v),u*v];let mut h=[0.;4];
                    for i in 0..4 {let w=basis[i]*(surface.weights[i]/max_weight);for j in 0..3 {h[j]+=w*surface.poles[i][j];}h[3]+=w;}
                    if h.iter().all(|x|x.is_finite()) {Ok(h)}else{Err(Error::Unsupported("bilinear trim precision is uncertain"))}
                };
                let first=homogeneous(a)?;let last=homogeneous(b)?;
                let mid=homogeneous([a[0]*0.5+b[0]*0.5,a[1]*0.5+b[1]*0.5])?;
                let middle=std::array::from_fn::<_,4,_>(|i|2.*mid[i]-0.5*first[i]-0.5*last[i]);
                let h=[first,middle,last];let min_weight=h.iter().map(|p|p[3]).fold(f64::INFINITY,f64::min);
                let max_control_weight=h.iter().map(|p|p[3]).fold(0_f64,f64::max);
                if min_weight<=0. || !min_weight.is_finite() || !max_control_weight.is_finite()
                    || !((max_control_weight/min_weight)*precision_bound).is_finite()
                    || (max_control_weight/min_weight)*precision_bound>=tolerance {
                    return Err(Error::Unsupported("bilinear trim precision is uncertain"));
                }
                let mut poles=Vec::new();poles.try_reserve_exact(3).map_err(|_|Error::Budget("bytes"))?;
                let mut weights=Vec::new();weights.try_reserve_exact(3).map_err(|_|Error::Budget("bytes"))?;
                for p in h {let point=[p[0]/p[3],p[1]/p[3],p[2]/p[3]];if point.iter().any(|v|!v.is_finite()) {return Err(Error::Unsupported("bilinear trim precision is uncertain"));}poles.push(point);weights.push(p[3]);}
                if poles[0]==poles[1] && poles[1]==poles[2] {return Err(Error::Unsupported("trim edge may be degenerate"));}
                let mut knots=Vec::new();knots.try_reserve_exact(6).map_err(|_|Error::Budget("bytes"))?;knots.extend_from_slice(&[0.,0.,0.,1.,1.,1.]);
                Curve::Nurbs(om9_core::geometry::NurbsCurve {degree:2,poles,weights,knots,domain:[0.,1.],periodic:false,reversed:false,seam:None})
            } else {
                let (iso,fixed,varying)=if a[1]==b[1] {(surface_kernel::SurfaceIso::U,a[1],0)}
                    else {(surface_kernel::SurfaceIso::V,a[0],1)};
                let mut curve=surface_kernel::isocurve(surface,iso,fixed,limits)?;
                curve.domain=[a[varying].min(b[varying]),a[varying].max(b[varying])];
                curve.reversed=a[varying]>b[varying];curve.validate(limits)?;
                if curve.poles.iter().all(|p|*p==curve.poles[0]) {return Err(Error::Invalid("trim edge has zero length"));}
                let start=surface_kernel::evaluate(surface,a[0],a[1],limits)?;
                let end=surface_kernel::evaluate(surface,b[0],b[1],limits)?;
                let mid=surface_kernel::evaluate(surface,a[0]*0.5+b[0]*0.5,a[1]*0.5+b[1]*0.5,limits)?;
                if start==mid && mid==end {return Err(Error::Unsupported("trim edge may be degenerate"));}
                Curve::Nurbs(curve)
            };
            curve_bytes=curve_bytes.checked_add(curve.validate(limits)?.bytes).ok_or(Error::Overflow("trim curves"))?;
            items.push((label,Geometry::Curve(curve),placement));
        }
    }
    drop(trim);
    let curve_live=if affine {0} else {curve_bytes.checked_mul(2).ok_or(Error::Overflow("trim curves"))?};
    retained.set(retained.get().checked_add(new_slots - old_slots).and_then(|n|n.checked_add(curve_live)).ok_or(Error::Overflow("trim edges"))?);
    admit(0, 0)?;
    Ok(())
}

pub(super) fn collapsed_clamped_surface_boundary(surface: &om9_core::geometry::NurbsSurface,
    iso: om9_core::surface_kernel::SurfaceIso, value: f64) -> bool {
    use om9_core::surface_kernel::SurfaceIso;
    let (fixed, varying) = match iso { SurfaceIso::U => (1, 0), SurfaceIso::V => (0, 1) };
    let knots = &surface.knots[fixed];
    let count = surface.degree[fixed] + 1;
    let row = if knots[..count].iter().all(|k| *k == value) { 0 }
        else if knots[knots.len()-count..].iter().all(|k| *k == value) { surface.size[fixed] - 1 }
        else { return false; };
    let pole = |i: usize| {
        let (u, v) = if varying == 0 { (i, row) } else { (row, i) };
        surface.poles[u * surface.size[1] + v]
    };
    let first = pole(0);
    (1..surface.size[varying]).all(|i| pole(i) == first)
}

#[cfg(feature = "occt-brep")]
fn surface_wire_sides(surface: &om9_core::geometry::NurbsSurface, limits: &om9_core::budget::Limits)
    -> Result<[(om9_core::surface_kernel::SurfaceIso, f64, bool); 4], Error> {
    use om9_core::surface_kernel::{self, SurfaceIso};
    let closed_u = super::dup_border::closed_surface_axis(surface, 0, limits)?;
    let closed_v = super::dup_border::closed_surface_axis(surface, 1, limits)?;
    if (surface.periodic[0] && !closed_u) || (surface.periodic[1] && !closed_v) {
        return Err(Error::Unsupported("ExtractWireframe requires verified periodic surface closure"));
    }
    let [[u0, u1], [v0, v1]] = surface.domain;
    let mut corners = [[0.; 3]; 4];
    for (i, (u, v)) in [(u0,v0),(u0,v1),(u1,v0),(u1,v1)].into_iter().enumerate() {
        corners[i] = surface_kernel::evaluate(surface, u, v, limits)?;
        for j in 0..i {
            let same_u = i / 2 == j / 2;
            let same_v = i % 2 == j % 2;
            let certified_equal = (closed_u && same_v) || (closed_v && same_u) || (closed_u && closed_v);
            if corners[i] == corners[j] && !certified_equal {
                return Err(Error::Unsupported("ExtractWireframe surface has ambiguous boundary corners"));
            }
        }
    }
    // The scene draws all four domain edges; a certified closed axis draws the
    // same seam twice, so retain one copy instead of discarding the seam.
    Ok([(SurfaceIso::U, v0, true), (SurfaceIso::U, v1, !closed_v),
        (SurfaceIso::V, u0, true), (SurfaceIso::V, u1, !closed_u)])
}

#[derive(Clone)]
pub(super) struct DupEdgeSourcePick {
    reference: super::SelectedSubobject,
    scene_publication: uuid::Uuid,
    temporary: bool,
    world: om9_core::geometry::Point3,
    camera: om9_scene::camera::Camera,
    viewport: om9_scene::camera::Viewport,
    screen: [f64; 2],
    pixel_tolerance: f64,
    depth: f64,
}

struct ReservedDupEdgePicks {
    slots: Vec<DupEdgeSourcePick>,
    reservation: Option<om9_core::transaction::ExternalMemoryReservation>,
    extra: Option<om9_core::transaction::ExternalMemoryReservation>,
}

#[cfg(feature = "occt-brep")]
pub(super) fn source_view_memory_refusal(error:&Error)->bool {
    matches!(error,Error::Budget("bytes" | "managed memory" | "managed RAM" | "available physical RAM" | "available system commit" | "external memory headroom"))
}

#[cfg(feature = "occt-brep")]
fn source_view_refusal(error: Error, reason: Error) -> Error {
    if source_view_memory_refusal(&reason) {
        return reason;
    }
    let reason = if matches!(reason, Error::Busy) {
        "another source-view worker is still finishing".to_owned()
    } else { reason.to_string() };
    Error::Io(format!("{error}; DupEdge source view unavailable: {reason}; try again"))
}

impl Controller {
    pub fn dispatch_wireframe(&mut self, feature: &'static str) -> Result<(), Error> {
        if feature == DUP_EDGE {
            if self.subselection.is_empty() {
                #[cfg(not(feature = "occt-brep"))]
                return Err(Error::Unsupported("exact DupEdge requires OCCT"));
                #[cfg(feature = "occt-brep")]
                {
                    self.dupedge_session_id = uuid::Uuid::new_v4();
                    self.dupedge_pending = Some(self.context());
                    if self.selection.len()==1 {
                        let root=*self.selection.iter().next().expect("one selection");
                        if self.document().entities.get(&root).is_some_and(|entity|matches!(entity.geometry,Geometry::NativeBrep(_))) {
                            if let Err(error)=self.request_dupedge_source_view(root) {
                                if !source_view_memory_refusal(&error) {self.dupedge_pending=None;}
                                return Err(error);
                            }
                        }
                    }
                    return Ok(());
                }
            }
            #[cfg(feature = "occt-brep")]
            if self.topology_proof_for(Some(self.subselection[0].root)).is_none() {
                let mut refresh = false;
                if let Err(error) = self.duplicate_source_picks(&mut refresh) {
                    if refresh {
                        let root = self.dupedge_source_picks[0].reference.root;
                        let old_session = self.dupedge_session_id;
                        self.dupedge_session_id = uuid::Uuid::new_v4();
                        self.dupedge_pending = Some(self.context());
                        match self.request_dupedge_source_view(root) {
                            Ok(()) => return Ok(()),
                            Err(reason) => {
                                self.dupedge_pending = None;
                                if source_view_memory_refusal(&reason) {
                                    self.dupedge_session_id=old_session;
                                }
                                return Err(source_view_refusal(error, reason));
                            }
                        }
                    }
                    return Err(error);
                }
                self.clear_subselection();
                return Ok(());
            }
            return self.extract_brep_edges(DUP_EDGE);
        }
        if feature == EXTRACT_WIRE { return self.extract_brep_edges(EXTRACT_WIRE); }
        Err(Error::Unsupported("wireframe command"))
    }

    pub fn dupedge_needs_pick(&self) -> bool {
        self.dupedge_pending.is_some() && self.is_running()
    }

    pub(crate) fn dupedge_recovery_key(&self) -> Option<(uuid::Uuid,u64,u64,uuid::Uuid)> {
        #[cfg(feature="occt-brep")]
        {
            if (self.is_running() && !self.dupedge_needs_pick())
                || (self.subselection.is_empty() && self.dupedge_source_picks.is_empty()) {return None;}
            Some((self.dupedge_session_id,self.sub_revision,self.selection_revision,self.scene.as_ref()?.publication_id()))
        }
        #[cfg(not(feature="occt-brep"))]
        {None}
    }

    #[cfg(feature="occt-brep")]
    pub(crate) fn dupedge_source_view_context_key(&self) -> Option<(uuid::Uuid,u64,u64,uuid::Uuid)> {
        if !self.dupedge_needs_pick() {return None;}
        Some((self.dupedge_session_id,self.sub_revision,self.selection_revision,self.scene.as_ref()?.publication_id()))
    }

    pub(crate) fn dupedge_temporary_picks(&self) -> impl Iterator<Item = &super::SelectedSubobject> {
        let publication = self.scene.as_ref().map(|scene| scene.publication_id());
        let active = self.dupedge_needs_pick();
        self.dupedge_source_picks.iter().filter(move |pick|
            active && pick.temporary && Some(pick.scene_publication) == publication)
            .map(|pick| &pick.reference)
    }

    pub(super) fn has_dupedge_temporary_picks(&self) -> bool {
        self.dupedge_source_picks.iter().any(|pick| pick.temporary)
    }

    pub fn dupedge_pick(&mut self, reference: super::SelectedSubobject) -> Result<(), Error> {
        if !self.dupedge_needs_pick() { return Err(Error::Missing("DupEdge selection session")); }
        if !matches!(reference.subset, om9_scene::scene::Subelement::BrepEdge(_)) || !reference.path.is_empty() {
            return Err(Error::Invalid("DupEdge requires a root B-Rep edge"));
        }
        if self.subselection.first().is_some_and(|old| old.root != reference.root) {
            return Err(Error::Unsupported("DupEdge requires edges of one root B-Rep"));
        }
        self.toggle_subobject_ref(reference)?;
        self.dupedge_source_picks = Vec::new();
        self.dupedge_picks_reservation = None;
        Ok(())
    }

    /// Retain the validated viewport hit for source-based selection after open.
    pub(crate) fn dupedge_pick_with_point(&mut self, reference: super::SelectedSubobject,
        point: om9_core::geometry::Point3, tolerance: f64, camera: &om9_scene::camera::Camera,
        viewport: om9_scene::camera::Viewport, screen: [f64; 2], pixel_tolerance: f64, depth: f64) -> Result<(), Error> {
        if !point.iter().chain(screen.iter()).all(|value| value.is_finite()) || !tolerance.is_finite()
            || tolerance <= 0.0 || !pixel_tolerance.is_finite() || pixel_tolerance <= 0.0 || !depth.is_finite() {
            return Err(Error::Invalid("DupEdge pick point or tolerance"));
        }
        viewport.validate()?;
        if self.is_running() && !self.dupedge_needs_pick() { return Err(Error::Busy); }
        if !matches!(reference.subset, om9_scene::scene::Subelement::BrepEdge(_)) || !reference.path.is_empty() {
            return Err(Error::Invalid("DupEdge requires a root B-Rep edge"));
        }
        if self.subselection.first().is_some_and(|old| old.root != reference.root) {
            return Err(Error::Unsupported("DupEdge requires edges of one root B-Rep"));
        }
        let scene_publication = self.scene.as_ref().ok_or(Error::Missing("DupEdge scene"))?.publication_id();
        let scene = self.scene.as_ref().ok_or(Error::Missing("DupEdge scene"))?;
        let entity = scene.document.resolve(&reference.source)?;
        let om9_scene::scene::Subelement::BrepEdge(edge_id) = reference.subset else { return Err(Error::Invalid("DupEdge edge")); };
        let temporary = match &entity.geometry {
            Geometry::Brep(shape) => !shape.topology().edges.contains(&edge_id),
            Geometry::NativeBrep(_) => true,
            _ => return Err(Error::Invalid("DupEdge B-Rep")),
        };
        if temporary {
            if !self.dupedge_needs_pick() || !self.subselection.is_empty()
                || !scene.root_brep_pick_display(&reference.source)?.topology.edges.contains(&edge_id) {
                return Err(Error::Stale("DupEdge temporary edge"));
            }
            let object = scene.objects.iter().find(|object| object.root == reference.root && object.path.is_empty())
                .ok_or(Error::Missing("DupEdge root"))?;
            if object.source != reference.source || !object.visible || !object.selectable || !object.has_pick_geometry()
                || reference.world_epoch != scene.epoch || scene.epoch != self.coordinator.epoch() {
                return Err(Error::Stale("DupEdge temporary edge"));
            }
        } else if self.dupedge_source_picks.iter().any(|pick| pick.temporary && pick.scene_publication == scene_publication) {
            return Err(Error::Invalid("DupEdge mixed edge identities"));
        }
        if self.dupedge_source_picks.iter().find(|pick| pick.scene_publication == scene_publication)
            .is_some_and(|pick| pick.reference.root != reference.root || pick.temporary != temporary) {
            return Err(Error::Unsupported("DupEdge requires edges of one root B-Rep"));
        }
        let removing = if temporary { self.dupedge_source_picks.iter().any(|pick| pick.scene_publication == scene_publication && pick.reference == reference) }
            else { self.subselection.contains(&reference) };
        let keep = self.dupedge_source_picks.iter().filter(|pick| pick.scene_publication == scene_publication && pick.reference != reference).count();
        let count = keep.checked_add(usize::from(!removing)).ok_or(Error::Overflow("DupEdge pick count"))?;
        let reservation = if count == 0 { None } else {
            let bytes = om9_core::budget::Usage::array::<DupEdgeSourcePick>(count)?.bytes;
            let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
            Some(self.coordinator.reserve_external_memory_with_held(
                om9_core::budget::memory::MemoryClass::CpuCache, bytes, physical, self.scene_cache_usage()?,
            )?)
        };
        let mut next = ReservedDupEdgePicks { slots: Vec::new(), reservation, extra: None };
        next.slots.try_reserve_exact(count).map_err(|_| Error::Budget("DupEdge picks"))?;
        if let Some(owner) = next.reservation.as_mut() {
            let actual = om9_core::budget::Usage::array::<DupEdgeSourcePick>(next.slots.capacity())?.bytes;
            owner.set_allocated_bytes(actual.min(owner.bytes()))?;
            if actual > owner.bytes() {
                let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
                let extra = actual - owner.bytes();
                next.extra = Some(self.coordinator.reserve_external_memory_with_allocated(
                    om9_core::budget::memory::MemoryClass::CpuCache, extra, extra, physical, self.scene_cache_usage()?,
                )?);
                owner.transfer_allocated_from(next.extra.as_mut().expect("extra pick capacity"), extra)?;
                next.extra = None;
            }
        }
        next.slots.extend(self.dupedge_source_picks.iter().filter(|pick|
            pick.scene_publication == scene_publication && pick.reference != reference).cloned());
        if !removing {
            next.slots.push(DupEdgeSourcePick { reference: reference.clone(), scene_publication, temporary, world: point,
                camera: camera.clone(), viewport, screen, pixel_tolerance, depth });
        }
        if !temporary { self.toggle_subobject_ref(reference)?; }
        let old_picks = std::mem::replace(&mut self.dupedge_source_picks, next.slots);
        let old_reservation = std::mem::replace(&mut self.dupedge_picks_reservation, next.reservation);
        drop(old_picks);
        drop(old_reservation);
        if temporary { self.sub_revision = self.sub_revision.saturating_add(1); }
        Ok(())
    }

    pub(super) fn finish_dupedge_selection(&mut self) -> Result<(), Error> {
        #[cfg(feature = "occt-brep")]
        if self.dupedge_source_picks.first().is_some_and(|pick| pick.temporary)
            || (!self.subselection.is_empty() && self.topology_proof_for(Some(self.subselection[0].root)).is_none()) {
            if self.dupedge_source_view_pending() { return Err(Error::Busy); }
            let mut refresh = false;
            if let Err(error) = self.duplicate_source_picks(&mut refresh) {
                if refresh && self.dupedge_scene_backup.is_none() {
                    let root = self.dupedge_source_picks[0].reference.root;
                    match self.request_dupedge_source_view(root) {
                        Ok(()) => return Ok(()),
                        Err(reason) => return Err(source_view_refusal(error, reason)),
                    }
                }
                return Err(error);
            }
        } else {
            self.extract_brep_edges(DUP_EDGE)?;
        }
        #[cfg(not(feature = "occt-brep"))]
        self.extract_brep_edges(DUP_EDGE)?;
        self.dupedge_pending = None;
        self.clear_subselection();
        Ok(())
    }

    #[cfg(feature = "occt-brep")]
    fn duplicate_source_picks(&mut self, refresh: &mut bool) -> Result<(), Error> {
        use om9_core::budget::{Usage, memory::MemoryClass};
        if self.dupedge_source_picks.is_empty() {
            return Err(Error::Unsupported("select the reopened edge in the viewport"));
        }
        let temporary = self.dupedge_source_picks[0].temporary;
        if (temporary && (!self.subselection.is_empty() || self.dupedge_source_picks.iter().any(|pick| !pick.temporary)))
            || (!temporary && (self.dupedge_source_picks.len() != self.subselection.len() || self.subselection.is_empty()
                || !self.subselection.iter().all(|reference| self.dupedge_source_picks.iter().any(|pick| &pick.reference == reference)))) {
            return Err(Error::Stale("DupEdge source pick"));
        }
        let reference = &self.dupedge_source_picks[0].reference;
        let scene = self.scene.as_ref().ok_or(Error::Missing("DupEdge scene"))?;
        for pick in &self.dupedge_source_picks {
            if pick.reference.root != reference.root { return Err(Error::Unsupported("DupEdge requires edges of one root B-Rep")); }
            if pick.scene_publication != scene.publication_id() { return Err(Error::Stale("DupEdge scene publication")); }
            if pick.temporary {
                let display = scene.root_brep_pick_display(&pick.reference.source)?;
                let om9_scene::scene::Subelement::BrepEdge(id) = pick.reference.subset else { return Err(Error::Invalid("DupEdge edge")); };
                if !display.topology.edges.contains(&id) || pick.reference.world_epoch != self.coordinator.epoch() {
                    return Err(Error::Stale("DupEdge temporary edge"));
                }
            } else { pick.reference.validate(scene, self.coordinator.epoch())?; }
        }
        let source = self.coordinator.capture_tracked_document(self.document)?;
        let entity = source.entities.get(&reference.root).ok_or(Error::Missing("DupEdge source"))?;
        if !matches!(entity.geometry,Geometry::Brep(_) | Geometry::NativeBrep(_)) {return Err(Error::Invalid("DupEdge B-Rep"));}
        let display = scene.root_brep_pick_display(&reference.source)?;
        entity.placement.validate()?;
        let mut rigid = true;
        let mut norm = 0_f64;
        for a in 0..3 {
            for b in 0..3 {
                norm = norm.hypot(entity.placement.0[a][b]);
                let dot: f64 = (0..3).map(|i| entity.placement.0[i][a] * entity.placement.0[i][b]).sum();
                if !dot.is_finite() || (dot - if a == b { 1.0 } else { 0.0 }).abs() > 1e-10 { rigid = false; }
            }
        }
        let inverse = om9_core::top11::inverse_affine(entity.placement)?;
        // Source-to-display correspondence must not exceed an explicit document tolerance.
        let world_tolerance = source.tolerance().map_or(1e-6, |value| value.absolute_mm.min(1e-6));
        let segment_tolerance = if rigid { world_tolerance } else { world_tolerance / norm.max(1.) };
        if !segment_tolerance.is_finite() || segment_tolerance <= 0. || (!rigid && segment_tolerance < 1e-12) {
            return Err(Error::Unsupported("DupEdge placement requires a smaller verification tolerance"));
        }
        let limits = *self.coordinator.limits();
        let held = self.scene_cache_usage()?;
        let physical = || om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
        let mut reservation = self.coordinator.reserve_external_memory_with_held(MemoryClass::Kernel, 1, physical(), held)?;
        let mut admit = |live: u64, pending: u64| -> Result<(), Error> {
            let needed = live.checked_add(pending).ok_or(Error::Overflow("DupEdge scratch"))?.max(1);
            if needed > reservation.bytes() {
                let extra = needed - reservation.bytes();
                self.coordinator.grow_external_memory_with_held(&mut reservation, extra, physical(), held)?;
            }
            reservation.set_allocated_bytes(live)?;
            reservation.recheck_pending()
        };
        let native_source=if let Geometry::NativeBrep(native)=&entity.geometry {
            if !temporary {return Err(Error::Stale("DupEdge native source view"));}
            Some(om9_kernel::brep::from_3dm_source_typed_admitted(&native.payload,world_tolerance,&limits,&mut admit)?)
        } else {None};
        let native_bytes=native_source.as_ref().map_or(0,|text|text.capacity() as u64);
        let mut items = Vec::new();
        let slots = Usage::array::<(&str, Geometry, om9_core::geometry::Transform)>(self.dupedge_source_picks.len())?.bytes;
        admit(native_bytes, slots)?;
        reserve_wire(&mut items,self.dupedge_source_picks.len())?;
        let mut live = native_bytes.checked_add(Usage::array::<(&str, Geometry, om9_core::geometry::Transform)>(items.capacity())?.bytes)
            .ok_or(Error::Overflow("DupEdge output"))?;
        admit(live, 0)?;
        for pick in &self.dupedge_source_picks {
            let local = inverse.point(pick.world)?;
            if !rigid {
                let roundtrip = entity.placement.point(local)?;
                let error = (0..3).map(|i| (roundtrip[i] - pick.world[i]).powi(2)).sum::<f64>().sqrt();
                if !error.is_finite() || error > world_tolerance { return Err(Error::Invalid("DupEdge placement loses pick precision")); }
            }
            // The hit is already on the picked display segment. Cursor radius
            // cannot justify a larger gap between that segment and the source.
            let source_tolerance = display.deflection_mm + segment_tolerance;
            if !source_tolerance.is_finite() { return Err(Error::Invalid("DupEdge pick tolerance")); }
            let om9_scene::scene::Subelement::BrepEdge(id) = pick.reference.subset else { return Err(Error::Invalid("DupEdge edge")); };
            let edge = display.topology.edges.iter().position(|candidate| *candidate == id)
                .ok_or(Error::Stale("DupEdge display edge"))?;
            let mut nearest: Option<(f64, [om9_core::geometry::Point3; 2])> = None;
            for segment in display.segments.iter().filter(|s| s[2] as usize == edge) {
                let a = *display.points.get(segment[0] as usize).ok_or(Error::Stale("DupEdge display segment"))?;
                let b = *display.points.get(segment[1] as usize).ok_or(Error::Stale("DupEdge display segment"))?;
                let direction = std::array::from_fn::<_, 3, _>(|i| b[i] - a[i]);
                let length2 = direction.iter().map(|v| v * v).sum::<f64>();
                if !length2.is_finite() || length2 <= 1e-18 { continue; }
                let t = (0..3).map(|i| (local[i] - a[i]) * direction[i]).sum::<f64>() / length2;
                let t = t.clamp(0.0, 1.0);
                let d2 = (0..3).map(|i| (local[i] - a[i] - t * direction[i]).powi(2)).sum::<f64>();
                if nearest.as_ref().is_none_or(|(best, _)| d2 < *best) { nearest = Some((d2, [a, b])); }
            }
            let (distance2, segment) = nearest.ok_or(Error::Stale("DupEdge display segment"))?;
            if distance2 > 1e-12 { return Err(Error::Stale("DupEdge display pick")); }
            let (curve, resolved) = match &entity.geometry {
                Geometry::Brep(brep) => selected_source_edge_with_admission(
                    brep,local,source_tolerance,segment,segment_tolerance,&limits,temporary,refresh,
                    &mut |current,pending| admit(live.checked_add(current).ok_or(Error::Overflow("DupEdge output"))?,pending))?,
                Geometry::NativeBrep(_) => selected_native_source_edge_with_admission(
                    native_source.as_deref().ok_or(Error::Stale("DupEdge native source"))?,local,source_tolerance,segment,segment_tolerance,&limits,
                    &mut |current,pending| admit(live.checked_add(current).ok_or(Error::Overflow("DupEdge output"))?,pending))?,
                _ => unreachable!(),
            };
            let projected = pick.camera.project(pick.world, pick.viewport)?
                .ok_or(Error::Invalid("DupEdge source edge outside viewport"))?;
            let pixel_distance = ((projected.screen[0] - pick.screen[0]).powi(2)
                + (projected.screen[1] - pick.screen[1]).powi(2)).sqrt();
            if pixel_distance > pick.pixel_tolerance || (projected.depth - pick.depth).abs() > source_tolerance {
                return Err(Error::Invalid("DupEdge source edge does not match viewport pick"));
            }
            let source_world = entity.placement.point(resolved)?;
            let source_screen = pick.camera.project(source_world, pick.viewport)?;
            if source_screen.as_ref().is_none_or(|projected| {
                let pixels = ((projected.screen[0]-pick.screen[0]).powi(2) + (projected.screen[1]-pick.screen[1]).powi(2)).sqrt();
                pixels > pick.pixel_tolerance || (projected.depth - pick.depth).abs() > source_tolerance
            }) {
                *refresh = !temporary;
                return Err(Error::Invalid("DupEdge source edge does not match viewport pick"));
            }
            let curve_bytes = curve.validate(&limits)?.bytes;
            let peak = curve_bytes.checked_mul(2).ok_or(Error::Overflow("DupEdge output"))?;
            admit(live, peak)?;
            let (placed, placement) = if rigid {
                (om9_core::top11::bake_curve(&curve, entity.placement)?, om9_core::geometry::Transform::identity())
            } else { (curve, entity.placement) };
            if items.iter().any(|(_, geometry, _)| matches!(geometry, Geometry::Curve(existing) if existing == &placed)) {
                return Err(Error::Invalid("DupEdge picks resolve to the same curve"));
            }
            live = live.checked_add(placed.validate(&limits)?.bytes).ok_or(Error::Overflow("DupEdge output"))?;
            items.push(("DupEdge", Geometry::Curve(placed), placement));
            admit(live, 0)?;
        }
        drop(admit);
        self.commit_dupedge_source_geometries(items)
    }

    #[cfg(all(test,feature="occt-brep"))]
    pub(crate) fn probe_dupedge_source_picks_for_test(&mut self) -> (Result<(),Error>,bool) {
        let mut refresh=false;let result=self.duplicate_source_picks(&mut refresh);(result,refresh)
    }

    #[cfg(not(feature = "occt-brep"))]
    fn extract_brep_edges(&mut self, _feature: &'static str) -> Result<(), Error> {
        Err(Error::Unsupported("exact B-Rep edge extraction requires OCCT"))
    }

    #[cfg(feature = "occt-brep")]
    fn extract_root_brep_wires(&mut self, root:om9_core::identity::ObjectId) -> Result<(),Error> {
        use om9_core::budget::{Usage,memory::MemoryClass};
        let document=self.document();
        let entity=document.entities.get(&root).ok_or(Error::Missing("edge extraction source"))?;
        let permissions=document.permissions(root,false)?;
        if !permissions.visible || !permissions.selectable {return Err(Error::Invalid("ExtractWireframe source is not selectable"));}
        if !matches!(entity.geometry,Geometry::Brep(_)) {return Err(Error::Invalid("edge extraction B-Rep"));}
        let placement=entity.placement;
        placement.validate()?;
        let stamp=document.stamp(root)?;
        let scene=self.scene.as_ref().ok_or(Error::Missing("ExtractWireframe scene"))?;
        if scene.epoch!=self.coordinator.epoch() {return Err(Error::Stale("ExtractWireframe selection"));}
        root_scene_has_interior_wires(scene,&stamp)?;
        let source=self.coordinator.capture_brep_source(&stamp)?;
        let limits=*self.coordinator.limits();
        let held=self.scene_cache_usage()?;
        let physical=||om9_native_owner::physical_memory().map(|s|(s.total_bytes,s.available_bytes));
        let mut reservation=self.coordinator.reserve_external_memory_with_held(MemoryClass::Kernel,1,physical(),held)?;
        let mut admit=|live:u64,pending:u64| -> Result<(),Error> {
            let needed=live.checked_add(pending).ok_or(Error::Overflow("ExtractWire scratch"))?.max(1);
            if needed>reservation.bytes() {
                let extra=needed-reservation.bytes();
                self.coordinator.grow_external_memory_with_held(&mut reservation,extra,physical(),held)?;
            }
            reservation.set_allocated_bytes(live)?;
            reservation.recheck_pending()
        };
        // The tracked source remains independently accounted; including it in
        // this validation reservation is conservative during transition.
        source.validate_canonical_admitted(&limits,&mut admit)?;
        let curves=source_wires_with_admission(source.source(),&limits,&mut admit)?;
        let curve_bytes=curves.iter().try_fold(Usage::array::<Curve>(curves.capacity())?.bytes,|sum,curve|
            sum.checked_add(curve.validate(&limits)?.bytes).ok_or(Error::Overflow("ExtractWire curves")))?;
        let output_bytes=curve_bytes.checked_mul(2).and_then(|n|n.checked_add(
            Usage::array::<(&str,Geometry,om9_core::geometry::Transform)>(curves.len()).ok()?.bytes))
            .ok_or(Error::Overflow("ExtractWire placement"))?;
        admit(0,output_bytes)?;
        let mut items=Vec::new();reserve_wire(&mut items,curves.len())?;
        let rigid=(0..3).all(|a|(0..3).all(|b| {
            let dot:f64=(0..3).map(|i|placement.0[i][a]*placement.0[i][b]).sum();
            dot.is_finite() && (dot-if a==b {1.} else {0.}).abs()<=1e-10
        }));
        for curve in curves {
            let (curve,placement)=if rigid {(om9_core::top11::bake_curve(&curve,placement)?,om9_core::geometry::Transform::identity())}
                else {(curve,placement)};
            items.push(("ExtractWire",Geometry::Curve(curve),placement));
        }
        admit(output_bytes,0)?;
        drop(admit);drop(source);
        self.commit_placed_geometries(EXTRACT_WIRE,"ExtractWire",items.into_iter())
    }

    #[cfg(feature = "occt-brep")]
    fn extract_brep_edges(&mut self, feature: &'static str) -> Result<(), Error> {
        use om9_core::budget::{Usage, memory::MemoryClass};
        use om9_scene::scene::Subelement;
        let all_edges = feature == EXTRACT_WIRE;
        if all_edges && (self.selection.len() > 1 || self.selection.iter().any(|id| {
            self.document().entities.get(id).is_some_and(|entity| matches!(entity.geometry, Geometry::NativeBrep(_) | Geometry::Instance(_) | Geometry::Surface(_)))
        })) {
            return self.extract_wireframe_roots();
        }
        if all_edges && self.selection.len()==1 {
            let root=*self.selection.iter().next().unwrap();
            if self.document().entities.get(&root).is_some_and(|entity|matches!(entity.geometry,Geometry::Brep(_))) {
                if self.background_wireframe_enabled {return self.request_brep_wireframe_worker();}
                return self.extract_root_brep_wires(root);
            }
        }
        let label = if all_edges { "ExtractWire" } else { "DupEdge" };
        let root = if all_edges {
            if self.selection.len() != 1 { return Err(Error::Invalid("ExtractWireframe requires one selected B-Rep")); }
            *self.selection.iter().next().unwrap()
        } else {
            let root = self.subselection.first().ok_or(Error::Missing("select B-Rep edges before DupEdge"))?.root;
            let scene = self.scene.as_ref().ok_or(Error::Missing("DupEdge scene"))?;
            for selected in &self.subselection {
                selected.validate(scene, self.coordinator.epoch())?;
                if selected.root != root || !selected.path.is_empty() {
                    return Err(Error::Unsupported("DupEdge requires edges of one root B-Rep"));
                }
                if !matches!(selected.subset, Subelement::BrepEdge(_)) {
                    return Err(Error::Invalid("DupEdge requires edge selection"));
                }
            }
            root
        };
        let source = self.coordinator.capture_tracked_document(self.document)?;
        let entity = source.entities.get(&root).ok_or(Error::Missing("edge extraction source"))?;
        if all_edges {
            let scene = self.scene.as_ref().ok_or(Error::Missing("ExtractWireframe scene"))?;
            if scene.epoch != self.coordinator.epoch() { return Err(Error::Stale("ExtractWireframe selection")); }
            let permissions = source.permissions(root, false)?;
            if !permissions.visible || !permissions.selectable { return Err(Error::Invalid("ExtractWireframe source is not selectable")); }
            if matches!(entity.geometry, Geometry::Surface(_)) && entity.extensions.contains_key(om9_core::model::TRIM_SURFACE_EXTENSION_SLOT) {
                return Err(Error::Unsupported("ExtractWireframe of trimmed surfaces requires source trim boundaries"));
            }
        }
        if all_edges {
            if let Geometry::Mesh(mesh) = &entity.geometry {
                let limits = *self.coordinator.limits();
                entity.placement.validate()?;
                mesh.validate(&limits)?;
                let count = mesh.faces.iter().try_fold(0usize, |sum, face| sum.checked_add(face.len()))
                    .ok_or(Error::Overflow("ExtractWire mesh edges"))?;
                if count == 0 { return Err(Error::Invalid("ExtractWireframe mesh has no edges")); }
                let reserve = Usage::array::<(u32, u32)>(count)?.bytes
                    .checked_add(Usage::array::<(&str, Geometry, om9_core::geometry::Transform)>(count)?.bytes.checked_mul(2)
                        .ok_or(Error::Overflow("ExtractWire mesh outputs"))?)
                    .and_then(|n| n.checked_add(4096))
                    .ok_or(Error::Overflow("ExtractWire mesh outputs"))?;
                let held = self.scene_cache_usage()?;
                let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
                let mut reservation = self.coordinator.reserve_external_memory_with_held(MemoryClass::Kernel, reserve, physical, held)?;
                reservation.recheck_pending()?;
                let mut edges = Vec::new();
                reserve_wire(&mut edges, count)?;
                for face in &mesh.faces {
                    for i in 0..face.len() {
                        let (a, b) = (face[i], face[(i + 1) % face.len()]);
                        edges.push((a.min(b), a.max(b)));
                    }
                }
                edges.sort_unstable();
                edges.dedup();
                let mut items = Vec::new();
                reserve_wire(&mut items, edges.len())?;
                for (a, b) in edges {
                    let (start, end) = (mesh.vertices[a as usize], mesh.vertices[b as usize]);
                    if start == end { return Err(Error::Invalid("ExtractWireframe mesh edge has zero length")); }
                    items.push(("ExtractWire", Geometry::Curve(Curve::Line { start, end, domain: [0., 1.] }), entity.placement));
                }
                reservation.set_allocated_bytes(reserve)?;
                return self.commit_placed_geometries(EXTRACT_WIRE, "ExtractWire", items.into_iter());
            }
        }
        let Geometry::Brep(_) = &entity.geometry else { return Err(Error::Invalid("edge extraction B-Rep")); };
        entity.placement.validate()?;
        let mut rigid = true;
        for a in 0..3 {
            for b in 0..3 {
                let dot: f64 = (0..3).map(|i| entity.placement.0[i][a] * entity.placement.0[i][b]).sum();
                if !dot.is_finite() || (dot - if a == b { 1.0 } else { 0.0 }).abs() > 1e-10 {
                    rigid = false;
                }
            }
        }
        let proof = self.topology_proof_for(Some(root));
        if !all_edges && proof.is_none() {
            return Err(Error::Unsupported("edge extraction requires attested source topology"));
        }
        let limits = *self.coordinator.limits();
        let held = self.scene_cache_usage()?;
        let physical = || om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
        // Declare the lease before every allocation it covers, and retain it
        // through commit. Accounting at commit is conservative (output overlap).
        let mut reservation = self.coordinator.reserve_external_memory_with_held(MemoryClass::Kernel, 1, physical(), held)?;
        let mut admit = |live: u64, pending: u64| -> Result<(), Error> {
            let needed = live.checked_add(pending).ok_or(Error::Overflow("DupEdge scratch"))?.max(1);
            if needed > reservation.bytes() {
                let extra = needed - reservation.bytes();
                self.coordinator.grow_external_memory_with_held(&mut reservation, extra, physical(), held)?;
            }
            reservation.set_allocated_bytes(live)?;
            reservation.recheck_pending()
        };
        let proof = proof.expect("proof required outside all-edge source extraction");
        proof.rebind_unchanged(&self.coordinator, &mut admit)?;
        let _permit = proof.acquire_worker()?;
        let registered_source = proof.capture_source(&self.coordinator)?;
        let deflection = self.scene.as_ref().ok_or(Error::Missing("DupEdge scene"))?
            .root_brep_pick_display(&self.document().stamp(root)?)?.deflection_mm;
        // Reuse native descriptor matching. A UUID's old array offset is not
        // assumed to survive source serialization. This temporary capture is
        // never published and never becomes the extracted curve geometry.
        let candidate = om9_kernel::brep::capture_source_topology(registered_source.source(), deflection,
            0.3, &limits, &mut admit).map_err(|e| Error::Io(format!("{label} topology: {e}")))?
            .into_rebound_for_source(&registered_source, &limits, &mut admit)?;
        let base = candidate.manifest_bytes()?.checked_add(candidate.brep().storage_usage(&limits)?.bytes)
            .ok_or(Error::Overflow("DupEdge candidate"))?;
        let count = self.subselection.len();
        if count == 0 { return Err(Error::Invalid("B-Rep has no edges")); }
        let index_bytes = Usage::array::<usize>(count)?.bytes;
        admit(base, index_bytes)?;
        let mut indices = Vec::new();
        indices.try_reserve_exact(count).map_err(|_| Error::Budget("edge indices"))?;
        for selected in &self.subselection {
            let Subelement::BrepEdge(id) = selected.subset else { unreachable!() };
            indices.push(candidate.brep().topology().edges.iter().position(|key| *key == id)
                .ok_or(Error::Stale("DupEdge edge mapping"))?);
        }
        let base = base.checked_add(Usage::array::<usize>(indices.capacity())?.bytes)
            .ok_or(Error::Overflow("DupEdge indices"))?;
        let curve_limits = limits.remaining_after(Usage { bytes: base, ..Usage::default() })?;
        let mut refused=None;
        let curves = om9_kernel::editing::registered_edges_nurbs_admitted(&registered_source, &indices, &curve_limits, |live, pending| {
            let result=base.checked_add(live).ok_or(Error::Overflow("DupEdge output"))
                .and_then(|bytes|admit(bytes,pending));
            if let Err(error)=&result {refused=Some(error.clone());}result
        }).map_err(|error|refused.unwrap_or_else(||edge_extraction_error(error,label)))?;
        let curve_bytes = curves.iter().try_fold(Usage::array::<Curve>(curves.capacity())?.bytes,
            |n, curve| -> Result<u64, Error> { n.checked_add(curve.validate(&limits)?.bytes).ok_or(Error::Overflow("DupEdge curves")) })?;
        let output_bytes = curve_bytes.checked_mul(2).and_then(|n| n.checked_add(
            Usage::array::<(&str, Geometry, om9_core::geometry::Transform)>(curves.len()).ok()?.bytes)).ok_or(Error::Overflow("DupEdge placement"))?;
        admit(base, output_bytes)?;
        let mut items = Vec::new();
        reserve_wire(&mut items,curves.len())?;
        for curve in curves {
            let (curve, placement) = if rigid {
                (om9_core::top11::bake_curve(&curve, entity.placement)?, om9_core::geometry::Transform::identity())
            } else { (curve, entity.placement) };
            items.push((label, Geometry::Curve(curve), placement));
        }
        admit(base.checked_add(output_bytes).ok_or(Error::Overflow("DupEdge outputs"))?, 0)?;
        drop(admit);
        // All curves must be ready before a single document transaction.
        self.commit_placed_geometries(feature, label, items.into_iter())
    }

    #[cfg(feature = "occt-brep")]
    fn extract_wireframe_roots(&mut self) -> Result<(), Error> {
        use om9_core::budget::{Usage, memory::MemoryClass};
        let source = self.coordinator.capture_tracked_document(self.document)?;
        let scene = self.scene.as_ref().ok_or(Error::Missing("ExtractWireframe scene"))?;
        if scene.epoch != self.coordinator.epoch() { return Err(Error::Stale("ExtractWireframe selection")); }
        for root in &self.selection {
            let entity = source.entities.get(root).ok_or(Error::Missing("edge extraction source"))?;
            let permissions = source.permissions(*root, false)?;
            if !permissions.visible || !permissions.selectable { return Err(Error::Invalid("ExtractWireframe source is not selectable")); }
            if !matches!(entity.geometry, Geometry::Brep(_) | Geometry::NativeBrep(_) | Geometry::Mesh(_) | Geometry::Surface(_) | Geometry::Instance(_)) {
                return Err(Error::Invalid("ExtractWireframe requires B-Rep, mesh or surface"));
            }
            entity.placement.validate()?;
        }
        let limits = *self.coordinator.limits();
        let held = self.scene_cache_usage()?;
        let physical = || om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
        let mut reservation = self.coordinator.reserve_external_memory_with_held(MemoryClass::Kernel, 1, physical(), held)?;
        let retained = std::cell::Cell::new(0_u64);
        let mut admit = |live: u64, pending: u64| -> Result<(), Error> {
            let live = retained.get().checked_add(live).ok_or(Error::Overflow("ExtractWire outputs"))?;
            let needed = live.checked_add(pending).ok_or(Error::Overflow("ExtractWire scratch"))?.max(1);
            if needed > reservation.bytes() {
                let extra = needed - reservation.bytes();
                self.coordinator.grow_external_memory_with_held(&mut reservation, extra, physical(), held)?;
            }
            reservation.set_allocated_bytes(live)?;
            reservation.recheck_pending()
        };
        let mut occurrences = Vec::new();
        let mut path = Vec::new();
        let mut nodes = 0;
        for root in &self.selection {
            collect_wireframe_occurrences(&source, scene, *root, &mut path, &source.entities[root], om9_core::geometry::Transform::identity(), 0,
                &limits, &mut nodes, &mut occurrences, &retained, &mut admit)?;
        }
        let mut items: Vec<(&str, Geometry, om9_core::geometry::Transform)> = Vec::new();
        for (entity, placement) in occurrences {
            if let Geometry::Mesh(mesh) = &entity.geometry {
                mesh.validate(&limits)?;
                let count = mesh.faces.iter().try_fold(0usize, |sum, face| sum.checked_add(face.len()))
                    .ok_or(Error::Overflow("ExtractWire mesh edges"))?;
                if count == 0 { return Err(Error::Invalid("ExtractWireframe mesh has no edges")); }
                let slots = items.len().checked_add(count).ok_or(Error::Overflow("ExtractWire mesh outputs"))?;
                let reserve = Usage::array::<(u32, u32)>(count)?.bytes
                    .checked_add(Usage::array::<(&str, Geometry, om9_core::geometry::Transform)>(slots)?.bytes.checked_mul(2)
                        .ok_or(Error::Overflow("ExtractWire mesh outputs"))?)
                    .and_then(|n| n.checked_add(4096)).ok_or(Error::Overflow("ExtractWire mesh outputs"))?;
                admit(0, reserve)?;
                let mut edges = Vec::new();
                reserve_wire(&mut edges, count)?;
                for face in &mesh.faces {
                    for i in 0..face.len() {
                        let (a, b) = (face[i], face[(i + 1) % face.len()]);
                        edges.push((a.min(b), a.max(b)));
                    }
                }
                edges.sort_unstable(); edges.dedup();
                let old_slots = Usage::array::<(&str, Geometry, om9_core::geometry::Transform)>(items.capacity())?.bytes;
                reserve_wire(&mut items, edges.len())?;
                let new_slots = Usage::array::<(&str, Geometry, om9_core::geometry::Transform)>(items.capacity())?.bytes;
                admit(0, reserve.checked_add(new_slots).ok_or(Error::Overflow("ExtractWire outputs"))?)?;
                for (a, b) in edges {
                    let (start, end) = (mesh.vertices[a as usize], mesh.vertices[b as usize]);
                    if start == end { return Err(Error::Invalid("ExtractWireframe mesh edge has zero length")); }
                    items.push(("ExtractWire", Geometry::Curve(Curve::Line { start, end, domain: [0., 1.] }), placement));
                }
                retained.set(retained.get().checked_add(new_slots - old_slots).ok_or(Error::Overflow("ExtractWire outputs"))?);
                admit(0, 0)?;
                continue;
            }
            if let Geometry::Surface(surface) = &entity.geometry {
                use om9_core::surface_kernel;
                surface.validate(&limits)?;
                if entity.extensions.contains_key(om9_core::model::TRIM_SURFACE_EXTENSION_SLOT) {
                    append_trimmed_surface_edges_at(entity, placement, &mut items, "ExtractWire", &limits, source.tolerance().map_or(1e-6,|t|t.absolute_mm.min(1e-6)), &retained, &mut admit)?;
                    continue;
                }
                let axis_points = surface.size[0].checked_add(surface.size[1]).ok_or(Error::Overflow("ExtractWire surface axes"))?;
                let axis_knots = surface.knots[0].len().checked_add(surface.knots[1].len()).ok_or(Error::Overflow("ExtractWire surface knots"))?;
                let wire_count = 4usize.checked_add(surface_kernel::default_wire_parameters(surface, 0)?.count())
                    .and_then(|n| n.checked_add(surface_kernel::default_wire_parameters(surface, 1).ok()?.count()))
                    .ok_or(Error::Overflow("ExtractWire surface outputs"))?;
                let slots = items.len().checked_add(wire_count).ok_or(Error::Overflow("ExtractWire surface outputs"))?;
                if u64::try_from(slots - 4).map_err(|_| Error::Overflow("ExtractWire surface outputs"))? > limits.max_outputs {
                    return Err(Error::Budget("ExtractWire surface outputs"));
                }
                let multiplier = u64::try_from(wire_count).map_err(|_| Error::Overflow("ExtractWire surface outputs"))?;
                let reserve = Usage::array::<om9_core::geometry::Point3>(axis_points)?.bytes.checked_mul(multiplier)
                    .and_then(|n| n.checked_add(Usage::array::<f64>(axis_points).ok()?.bytes.checked_mul(multiplier.checked_mul(2)?)?))
                    .and_then(|n| n.checked_add(Usage::array::<f64>(axis_knots).ok()?.bytes.checked_mul(multiplier)?))
                    .and_then(|n| n.checked_add(Usage::array::<(&str, Geometry, om9_core::geometry::Transform)>(slots).ok()?.bytes.checked_mul(2)?))
                    .and_then(|n| n.checked_add(4096)).ok_or(Error::Overflow("ExtractWire surface outputs"))?;
                admit(0, reserve)?;
                let sides = surface_wire_sides(surface, &limits)?;
                let old_slots = Usage::array::<(&str, Geometry, om9_core::geometry::Transform)>(items.capacity())?.bytes;
                reserve_wire(&mut items, wire_count)?;
                let new_slots = Usage::array::<(&str, Geometry, om9_core::geometry::Transform)>(items.capacity())?.bytes;
                admit(0, reserve.checked_add(new_slots).ok_or(Error::Overflow("ExtractWire outputs"))?)?;
                let mut curve_bytes = 0_u64;
                let first_surface_output = items.len();
                for (iso, value, include) in sides {
                    if !include { continue; }
                    if collapsed_clamped_surface_boundary(surface, iso, value) { continue; }
                    let curve = Curve::Nurbs(surface_kernel::isocurve(surface, iso, value, &limits)?);
                    curve_bytes = curve_bytes.checked_add(curve.validate(&limits)?.bytes)
                        .ok_or(Error::Overflow("ExtractWire surface curves"))?;
                    items.push(("ExtractWire", Geometry::Curve(curve), placement));
                }
                for axis in 0..2 {
                    let iso = if axis == 0 { surface_kernel::SurfaceIso::V } else { surface_kernel::SurfaceIso::U };
                    for value in surface_kernel::default_wire_parameters(surface, axis)? {
                        let curve = Curve::Nurbs(surface_kernel::isocurve(surface, iso, value, &limits)?);
                        curve_bytes = curve_bytes.checked_add(curve.validate(&limits)?.bytes)
                            .ok_or(Error::Overflow("ExtractWire surface curves"))?;
                        items.push(("ExtractWire", Geometry::Curve(curve), placement));
                    }
                }
                if items.len() == first_surface_output { return Err(Error::Invalid("ExtractWireframe surface has no curve boundaries")); }
                let additional = curve_bytes.checked_mul(2).and_then(|n| n.checked_add(new_slots - old_slots))
                    .ok_or(Error::Overflow("ExtractWire outputs"))?;
                retained.set(retained.get().checked_add(additional).ok_or(Error::Overflow("ExtractWire outputs"))?);
                admit(0, 0)?;
                continue;
            }
            let rigid = (0..3).all(|a| (0..3).all(|b| {
                let dot: f64 = (0..3).map(|i| placement.0[i][a] * placement.0[i][b]).sum();
                (dot - if a == b { 1.0 } else { 0.0 }).abs() <= 1e-10
            }));
            let curves = match &entity.geometry {
                Geometry::Brep(brep) => {
                    brep.validate_source(&limits)?;
                    source_wires_with_admission(brep.source(),&limits,&mut admit)?
                },
                Geometry::NativeBrep(_) => return Err(Error::Unsupported(
                    "ExtractWireframe native BRep face and trim binding is not verified")),
                _ => unreachable!(),
            };
            let curve_bytes = curves.iter().try_fold(Usage::array::<Curve>(curves.capacity())?.bytes,
                |n, curve| -> Result<u64, Error> { n.checked_add(curve.validate(&limits)?.bytes).ok_or(Error::Overflow("ExtractWire curves")) })?;
            let new_count = items.len().checked_add(curves.len()).ok_or(Error::Overflow("ExtractWire outputs"))?;
            if u64::try_from(new_count).map_err(|_|Error::Overflow("ExtractWire outputs"))?>limits.max_outputs {
                return Err(Error::Budget("ExtractWire outputs"));
            }
            let output_bytes = curve_bytes.checked_mul(2).and_then(|n| n.checked_add(
                Usage::array::<(&str, Geometry, om9_core::geometry::Transform)>(new_count).ok()?.bytes)).ok_or(Error::Overflow("ExtractWire placement"))?;
            admit(0, output_bytes)?;
            let old_slots = Usage::array::<(&str, Geometry, om9_core::geometry::Transform)>(items.capacity())?.bytes;
            reserve_wire(&mut items, curves.len())?;
            let actual_slots = Usage::array::<(&str, Geometry, om9_core::geometry::Transform)>(items.capacity())?.bytes;
            let held_bytes = output_bytes.checked_add(actual_slots).ok_or(Error::Overflow("ExtractWire outputs"))?;
            admit(0, held_bytes)?;
            for curve in curves {
                let (curve, placement) = if rigid {
                    (om9_core::top11::bake_curve(&curve, placement)?, om9_core::geometry::Transform::identity())
                } else { (curve, placement) };
                items.push(("ExtractWire", Geometry::Curve(curve), placement));
            }
            let additional = curve_bytes.checked_mul(2).and_then(|n| n.checked_add(actual_slots - old_slots))
                .ok_or(Error::Overflow("ExtractWire outputs"))?;
            retained.set(retained.get().checked_add(additional).ok_or(Error::Overflow("ExtractWire outputs"))?);
            admit(0, 0)?;
        }
        drop(admit);
        self.commit_placed_geometries(EXTRACT_WIRE, "ExtractWire", items.into_iter())
    }
}

#[cfg(all(test, feature = "occt-brep"))]
mod source_pick_tests {
    use super::*;
    use om9_core::{budget::Limits, model::{Document, Entity}};
    use om9_scene::{camera::{Camera, Viewport}, scene::Subelement};

    #[test]
    fn source_wire_extraction_checks_combined_count_and_preserves_refusal() {
        let limits=Limits::default();
        let shape=om9_kernel::brep::box_brep([4.,5.,6.],0.1,&limits).unwrap();
        let exact=Limits {max_outputs:24,..limits};
        assert_eq!(source_wires_with_admission(shape.source(),&exact,&mut |_,_|Ok(())).unwrap().len(),24);
        let tight=Limits {max_outputs:23,..limits};
        assert!(matches!(source_wires_with_admission(shape.source(),&tight,&mut |_,_|Ok(())),Err(Error::Budget("ExtractWire outputs"))));
        let mut edge_calls=0;
        om9_kernel::editing::all_edges_from_source_admitted(shape.source(),&limits,|_,_| {edge_calls+=1;Ok(())}).unwrap();
        let stop=edge_calls+4;
        let mut calls=0;
        let result=source_wires_with_admission(shape.source(),&limits,&mut |_,_| {
            calls+=1;if calls==stop {Err(Error::Canceled)} else {Ok(())}
        });
        assert!(matches!(result,Err(Error::Canceled)));
        assert_eq!(calls,stop);
    }

    #[test]
    fn edge_allocation_failure_is_recoverable_but_structural_limit_is_not() {
        use om9_kernel::editing::EditingError;
        assert!(matches!(edge_extraction_error(EditingError::Budget("edge curve allocation".into()),"ExtractWire"),Error::Budget("bytes")));
        assert!(matches!(edge_extraction_error(EditingError::Budget("edge curve count".into()),"ExtractWire"),Error::Io(_)));
        assert!(matches!(source_view_refusal(Error::Invalid("old view"),Error::Budget("managed memory")),Error::Budget("managed memory")));
        assert!(matches!(source_view_refusal(Error::Invalid("old view"),Error::Budget("DupEdge source-view budget")),Error::Io(_)));
    }

    #[test]
    fn all_source_edges_preserve_admission_error_not_structural_limit() {
        let limits=Limits::default();
        let shape=om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Box {x_mm:4.,y_mm:5.,z_mm:6.},0.1,&limits).unwrap();
        let mut calls=0;let result=source_edges_with_admission(&shape,&limits,false,&mut |_,pending| {
            calls+=1;if pending>0 {Err(Error::Budget("managed memory"))}else{Ok(())}
        });
        assert!(calls>1);assert!(matches!(result,Err(Error::Budget("managed memory"))));
        let limited=Limits {max_outputs:1,..limits};
        let result=source_edges_with_admission(&shape,&limited,false,&mut |_,_| Ok(()));
        assert!(matches!(result,Err(Error::Io(_))));
    }

    #[test]
    fn selected_source_edge_preserves_admission_failure_and_does_not_refresh() {
        let limits=Limits::default();
        let shape=om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Box {x_mm:4.,y_mm:5.,z_mm:6.},0.1,&limits).unwrap();
        let mut refresh=false;let mut calls=0;
        let result=selected_source_edge_with_admission(&shape,[0.,0.,3.],0.1,[[0.,0.,0.],[0.,0.,6.]],1e-6,
            &limits,false,&mut refresh,&mut |_,_|{calls+=1;Err(Error::Budget("managed memory"))});
        assert!(calls>0);assert!(matches!(result,Err(Error::Budget("managed memory"))));assert!(!refresh);
        let result=selected_source_edge_with_admission(&shape,[0.,0.,3.],0.1,[[0.,0.,0.],[0.,0.,6.]],1e-6,
            &limits,false,&mut refresh,&mut |_,_|Err(Error::Overflow("test admission")));
        assert!(matches!(result,Err(Error::Overflow("test admission"))));assert!(!refresh);
    }

    #[test]
    fn affine_trim_plane_requires_exact_control_net_parallelogram() {
        let limits = Limits::default();
        let mut surface = om9_core::surface_kernel::plane([1e6, 0., 0.], [1., 0., 0.], [0., 1., 0.], 10., 10., &limits).unwrap();
        assert!(is_affine_trim_plane(&surface));
        surface.poles[3][0] = f64::from_bits(surface.poles[3][0].to_bits() + 1);
        assert_eq!(surface.poles[0][0] + surface.poles[3][0], surface.poles[1][0] + surface.poles[2][0], "rounded sums alone miss the warp");
        assert!(!is_affine_trim_plane(&surface));
    }

    #[test]
    fn wire_reservation_rejects_capacity_overflow_before_allocation() {
        let mut edges = vec![(1_u32, 2_u32)];
        let old = edges.clone();
        assert!(matches!(reserve_wire(&mut edges, usize::MAX), Err(Error::Overflow("ExtractWire allocation"))));
        assert_eq!(edges, old);
        let beyond_layout = (isize::MAX as usize / std::mem::size_of::<(u32, u32)>()) + 1;
        assert!(matches!(reserve_wire(&mut edges, beyond_layout), Err(Error::Overflow("ExtractWire allocation"))));
        assert_eq!(edges, old);
        reserve_wire(&mut edges, 2).unwrap();
        assert!(edges.capacity() >= 3);
    }

    #[cfg(all(windows, target_pointer_width = "64"))]
    #[test]
    fn wire_reservation_reports_allocator_refusal_as_memory_pressure() {
        // One EiB is a valid Rust allocation layout but exceeds Windows x64's
        // user address space, so the fallible allocation must return an error.
        let mut bytes = Vec::<u8>::new();
        assert!(std::alloc::Layout::array::<u8>(1_usize << 60).is_ok());
        assert!(matches!(reserve_wire(&mut bytes, 1_usize << 60), Err(Error::Budget("bytes"))));
        assert!(bytes.is_empty());
    }

    fn box_x_edge(shape: &dyn om9_core::brep::BrepSourceAccess) -> uuid::Uuid {
        let display = shape.display();
        let segment = display.segments.iter().find(|s| {
            let a = display.points[s[0] as usize];
            let b = display.points[s[1] as usize];
            (a[1].abs() + a[2].abs() + b[1].abs() + b[2].abs()) < 1e-9
                && (a[0] - b[0]).abs() > 1e-9
        }).expect("Box x edge display");
        shape.topology().edges[segment[2] as usize]
    }

    #[test]
    fn reopened_affine_box_source_picks_keep_selected_edges_and_undo() {
        use om9_core::{evaluation::{evaluate_curve, Parameter}, geometry::Transform};
        let limits = Limits::default();
        let shape = om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Box { x_mm: 4., y_mm: 5., z_mm: 6. }, 0.01, &limits).unwrap();
        for placement in [
            Transform([[2.,0.,0.,3.],[0.,1.,0.,-2.],[0.,0.,0.5,1.],[0.,0.,0.,1.]]),
            Transform([[1.,0.5,0.,3.],[0.,1.,0.,-2.],[0.,0.,1.,1.],[0.,0.,0.,1.]]),
            Transform([[-1.,0.,0.,3.],[0.,2.,0.,-2.],[0.,0.,1.,1.],[0.,0.,0.,1.]]),
        ] {
          for profile in [0.5,0.01] {
            let body = om9_kernel::brep::read_brep(shape.source(),profile,&limits).unwrap();
            let mut entity = Entity::new("Box",Geometry::Brep(body),None); entity.placement = placement;
            let id = entity.id; let mut document = Document::new("reopened affine picks"); document.entities.insert(id,entity);
            let path = std::env::temp_dir().join(format!("om9-dupedge-affine-{}.om9",uuid::Uuid::new_v4()));
            om9_archive::Archive::save(&path,&document,&limits,&om9_core::transaction::Cancellation::default()).unwrap();
            let mut c = Controller::from_file(&path,limits).unwrap(); std::fs::remove_file(path).unwrap();
            c.submit("DupEdge").unwrap();
            let Geometry::Brep(body) = &c.document().entities[&id].geometry else { panic!("B-rep") };
            let mut picks = Vec::new();
            for segment in &body.display().segments {
                let a = body.display().points[segment[0] as usize]; let b = body.display().points[segment[1] as usize];
                if a[2].abs() > 1e-9 || b[2].abs() > 1e-9 { continue; }
                let key = body.topology().edges[segment[2] as usize];
                if picks.iter().any(|(edge, _, _)| *edge == key) { continue; }
                picks.push((key,a,b)); if picks.len() == 2 { break; }
            }
            assert_eq!(picks.len(),2);
            let camera = Camera::orthographic([12.,-15.,20.],[3.,2.,2.],[0.,0.,1.],40.).unwrap();
            let viewport = Viewport { origin: [0.,0.], size: [800.,800.] };
            let before = c.document().entities.clone();
            for (edge,a,b) in &picks {
                let local = std::array::from_fn(|i| (a[i]+b[i])*0.5); let world = placement.point(local).unwrap();
                let screen = camera.project(world,viewport).unwrap().unwrap();
                let reference = super::super::SelectedSubobject { root:id,path:vec![],subset:Subelement::BrepEdge(*edge),
                    source:c.document().stamp(id).unwrap(),world_epoch:c.coordinator().epoch() };
                c.dupedge_pick_with_point(reference,world,0.01,&camera,viewport,screen.screen,6.,screen.depth).unwrap();
            }
            c.submit("").unwrap();
            let output: Vec<_> = c.document().entities.values().filter(|e| e.label=="DupEdge").collect(); assert_eq!(output.len(),2);
            for (_,a,b) in &picks {
                let expected = [placement.point(*a).unwrap(),placement.point(*b).unwrap()];
                assert!(output.iter().any(|entity| {
                    let Geometry::Curve(curve) = &entity.geometry else { return false };
                    let points: Vec<_> = [0.,1.].into_iter().map(|t| evaluate_curve(curve,Parameter::Fraction(t),0,entity.placement,&limits).unwrap()[0]).collect();
                    expected.iter().all(|p| points.iter().any(|q| (0..3).all(|i| (p[i]-q[i]).abs()<1e-8)))
                }));
            }
            let after = c.document().entities.clone(); c.undo().unwrap(); assert_eq!(c.document().entities,before);
            c.redo().unwrap(); assert_eq!(c.document().entities,after);
          }
        }
    }

    #[test]
    fn rigid_cylinder_coarse_chord_offscreen_source_requires_view_refresh() {
        let limits=Limits::default();let shape=om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Cylinder {radius_mm:2.,height_mm:5.},0.5,&limits).unwrap();
        let segment=shape.display().segments.iter().find(|s| {
            let a=shape.display().points[s[0] as usize];let b=shape.display().points[s[1] as usize];
            (a[2]-5.).abs()<1e-8&&(b[2]-5.).abs()<1e-8&&(a[0]-b[0]).hypot(a[1]-b[1])>0.1
        }).unwrap();
        let a=shape.display().points[segment[0] as usize];let b=shape.display().points[segment[1] as usize];
        let edge=shape.topology().edges[segment[2] as usize];let picked=std::array::from_fn(|i|(a[i]+b[i])*0.5);
        let mut doc=Document::with_template_tolerance("rigid cylinder zoom");let entity=Entity::new("cylinder",Geometry::Brep(shape),None);
        let id=entity.id;doc.entities.insert(id,entity);let mut c=Controller::new(doc,limits).unwrap();
        c.submit("Point").unwrap();c.submit("20,20,20").unwrap();c.undo().unwrap();c.submit("DupEdge").unwrap();
        let camera=Camera::orthographic([picked[0],picked[1],50.],[picked[0],picked[1],5.],[0.,1.,0.],0.1).unwrap();
        let viewport=Viewport {origin:[0.,0.],size:[800.,800.]};let screen=camera.project(picked,viewport).unwrap().unwrap();
        let reference=super::super::SelectedSubobject {root:id,path:vec![],subset:Subelement::BrepEdge(edge),source:c.document().stamp(id).unwrap(),world_epoch:c.coordinator().epoch()};
        c.dupedge_pick_with_point(reference,picked,0.01,&camera,viewport,screen.screen,6.,screen.depth).unwrap();
        let before=c.coordinator().snapshot();let (result,refresh)=c.probe_dupedge_source_picks_for_test();
        assert!(result.is_err()&&refresh,"source outside the clicked screen area must request a fresh view");
        assert_eq!(c.coordinator().snapshot(),before);assert_eq!(c.coordinator().redo_len(),1);assert!(c.dupedge_needs_pick());
        c.submit("").unwrap();assert!(c.dupedge_source_view_pending());
        assert_eq!(c.coordinator().snapshot(),before);assert_eq!(c.coordinator().redo_len(),1);
        c.cancel();assert!(!c.dupedge_source_view_pending());assert_eq!(c.coordinator().snapshot(),before);assert_eq!(c.coordinator().redo_len(),1);
    }

    #[test]
    fn reopened_affine_cylinder_pick_preserves_exact_circular_source() {
        use om9_core::{evaluation::{evaluate_curve, Parameter}, geometry::Transform};
        let limits = Limits::default();
        let shape = om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Cylinder { radius_mm: 2., height_mm: 5. },0.01,&limits).unwrap();
        let placement = Transform([[2.,0.5,0.,10.],[0.,1.,0.,-2.],[0.,0.,1.,1.],[0.,0.,0.,1.]]);
        for profile in [0.5,0.01] {
            let body = om9_kernel::brep::read_brep(shape.source(),profile,&limits).unwrap();
            let mut entity = Entity::new("cylinder",Geometry::Brep(body),None); entity.placement=placement;
            let id=entity.id; let mut document=Document::new("reopened affine circle"); document.entities.insert(id,entity);
            let path=std::env::temp_dir().join(format!("om9-dupedge-affine-circle-{}.om9",uuid::Uuid::new_v4()));
            om9_archive::Archive::save(&path,&document,&limits,&om9_core::transaction::Cancellation::default()).unwrap();
            let mut c=Controller::from_file(&path,limits).unwrap(); std::fs::remove_file(path).unwrap();
            c.submit("Point").unwrap(); c.submit("20,20,20").unwrap(); c.undo().unwrap(); c.submit("DupEdge").unwrap();
            let Geometry::Brep(body)=&c.document().entities[&id].geometry else { panic!("B-rep") };
            let segment=body.display().segments.iter().find(|s| {
                let a=body.display().points[s[0] as usize]; let b=body.display().points[s[1] as usize];
                (a[2]-5.).abs()<1e-8 && (b[2]-5.).abs()<1e-8
            }).unwrap();
            let a=body.display().points[segment[0] as usize]; let b=body.display().points[segment[1] as usize];
            let edge=body.topology().edges[segment[2] as usize];
            let mut world=placement.point(std::array::from_fn(|i|a[i]*0.98+b[i]*0.02)).unwrap();
            let camera=Camera::orthographic([20.,-15.,20.],[10.,-2.,3.],[0.,0.,1.],40.).unwrap();
            let viewport=Viewport {origin:[0.,0.],size:[800.,800.]}; let mut screen=camera.project(world,viewport).unwrap().unwrap();
            let mut reference=super::super::SelectedSubobject {root:id,path:vec![],subset:Subelement::BrepEdge(edge),
                source:c.document().stamp(id).unwrap(),world_epoch:c.coordinator().epoch()};
            let before=c.document().entities.clone();
            if profile == 0.5 {
                let midpoint=placement.point(std::array::from_fn(|i|(a[i]+b[i])*0.5)).unwrap();
                let middle_screen=camera.project(midpoint,viewport).unwrap().unwrap();
                c.dupedge_pick_with_point(reference.clone(),midpoint,0.01,&camera,viewport,middle_screen.screen,6.,middle_screen.depth).unwrap();
                let snapshot=c.coordinator().snapshot();
                c.dupedge_source_view_active.store(true,std::sync::atomic::Ordering::Release);
                let error=c.submit("").unwrap_err().to_string();
                assert!(error.contains("source view unavailable") && error.contains("worker is still finishing"),"{error}");
                assert_eq!(c.coordinator().snapshot(),snapshot); assert_eq!(c.coordinator().redo_len(),1); assert!(c.dupedge_needs_pick());
                c.dupedge_source_view_active.store(false,std::sync::atomic::Ordering::Release);
                c.submit("").expect("coarse affine pick starts source-view recovery");
                assert_eq!(c.coordinator().snapshot(),snapshot); assert_eq!(c.coordinator().redo_len(),1);
                assert!(c.dupedge_source_view_pending());
                c.cancel(); assert!(!c.dupedge_needs_pick()); assert!(!c.dupedge_source_view_pending());
                assert_eq!(c.coordinator().snapshot(),snapshot); assert_eq!(c.coordinator().redo_len(),1);
                let deadline=std::time::Instant::now()+std::time::Duration::from_secs(20);
                while c.dupedge_source_view_worker_active() {
                    assert!(std::time::Instant::now()<deadline); std::thread::yield_now();
                }
                c.submit("DupEdge").unwrap();
                c.dupedge_pick_with_point(reference.clone(),midpoint,0.01,&camera,viewport,middle_screen.screen,6.,middle_screen.depth).unwrap();
                c.submit("").expect("recovery can be retried after cancel");
                loop {
                    if let Some(result)=c.poll_dupedge_source_view() { result.unwrap(); break; }
                    assert!(std::time::Instant::now()<deadline); std::thread::yield_now();
                }
                assert!(c.dupedge_needs_pick()); assert!(c.subselection().is_empty());
                let stamp=c.document().stamp(id).unwrap();
                let display=c.scene().unwrap().root_brep_pick_display(&stamp).unwrap();
                assert!((display.deflection_mm-0.01).abs()<1e-12);
                let s=display.segments.iter().find(|s| {
                    let p=display.points[s[0] as usize]; let q=display.points[s[1] as usize];
                    (p[2]-5.).abs()<1e-8 && (q[2]-5.).abs()<1e-8
                }).unwrap();
                let p=display.points[s[0] as usize]; let q=display.points[s[1] as usize];
                let edge=display.topology.edges[s[2] as usize];
                world=placement.point(std::array::from_fn(|i|(p[i]+q[i])*0.5)).unwrap();
                screen=camera.project(world,viewport).unwrap().unwrap();
                reference=super::super::SelectedSubobject {root:id,path:vec![],subset:Subelement::BrepEdge(edge),
                    source:stamp,world_epoch:c.coordinator().epoch()};
            }
            c.dupedge_pick_with_point(reference,world,0.01,&camera,viewport,screen.screen,6.,screen.depth).unwrap(); c.submit("").unwrap();
            let output=c.document().entities.values().find(|e|e.label=="DupEdge").unwrap(); assert_eq!(output.placement,placement);
            let Geometry::Curve(curve)=&output.geometry else {panic!("curve")};
            for t in [0.,0.13,0.37,0.61,0.91,1.] {
                let p=evaluate_curve(curve,Parameter::Fraction(t),0,output.placement,&limits).unwrap()[0];
                let y=p[1]+2.; let x=(p[0]-10.-0.5*y)/2.;
                assert!((x*x+y*y-4.).abs()<1e-7 && (p[2]-6.).abs()<1e-8);
            }
            let after=c.document().entities.clone(); c.undo().unwrap(); assert_eq!(c.document().entities,before);
            c.redo().unwrap(); assert_eq!(c.document().entities,after);
        }
    }

    #[test]
    fn affine_pick_refuses_unresolvable_verification_tolerance_without_losing_redo() {
        use om9_core::geometry::Transform;
        let limits=Limits::default();
        let shape=om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Box{x_mm:4.,y_mm:5.,z_mm:6.},0.01,&limits).unwrap();
        let edge=box_x_edge(&shape);
        let mut entity=Entity::new("large scale",Geometry::Brep(shape),None);
        entity.placement=Transform::scale([2e6,1.,1.]).unwrap(); let id=entity.id;
        let mut doc=Document::new("affine tolerance refusal"); doc.entities.insert(id,entity);
        let mut c=Controller::new(doc,limits).unwrap(); c.submit("Point").unwrap(); c.submit("10,10,10").unwrap(); c.undo().unwrap(); c.submit("DupEdge").unwrap();
        let camera=Camera::orthographic([4e6,0.,10.],[4e6,0.,0.],[0.,1.,0.],10.).unwrap();
        let viewport=Viewport {origin:[0.,0.],size:[800.,800.]}; let world=[4e6,0.,0.];
        let screen=camera.project(world,viewport).unwrap().unwrap();
        let reference=super::super::SelectedSubobject {root:id,path:vec![],subset:Subelement::BrepEdge(edge),
            source:c.document().stamp(id).unwrap(),world_epoch:c.coordinator().epoch()};
        c.dupedge_pick_with_point(reference,world,0.01,&camera,viewport,screen.screen,6.,screen.depth).unwrap();
        let before=c.coordinator().snapshot();
        assert!(matches!(c.submit(""),Err(Error::Unsupported("DupEdge placement requires a smaller verification tolerance"))));
        assert_eq!(c.coordinator().snapshot(),before); assert_eq!(c.coordinator().redo_len(),1); assert!(c.dupedge_needs_pick());
    }

    #[test]
    fn temporary_scene_edges_duplicate_together_and_preserve_history() {
        let limits = Limits::default();
        let shape = om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Box { x_mm: 4., y_mm: 5., z_mm: 6. }, 0.01, &limits,
        ).unwrap();
        let mut document = Document::new("temporary two-edge selection");
        let entity = Entity::new("box", Geometry::Brep(shape.clone()), document.current_layer);
        let id = entity.id;
        document.entities.insert(id, entity);
        let mut controller = Controller::new(document, limits).unwrap();
        let mut display = shape.display().clone();
        for id in &mut display.topology.edges { *id = uuid::Uuid::new_v4(); }
        let fresh = om9_core::brep::Brep::from_native(shape.source().to_owned(), display, &limits).unwrap();
        let stamp = controller.document().stamp(id).unwrap();
        let (buffers, _) = om9_scene::scene::Scene::prepare_brep_buffers(&fresh, &limits, |_, _| Ok(())).unwrap();
        let replacement = controller.scene().unwrap().try_replace_root_brep_display(
            &stamp, &fresh, buffers, [0, 0], std::sync::Arc::new(()), std::sync::Arc::new(()),
        ).unwrap();
        controller.publish_owned_scene(replacement, None);
        controller.submit("DupEdge").unwrap();
        let camera = Camera::orthographic([10., -10., 14.], [2., 2., 3.], [0., 0., 1.], 30.).unwrap();
        let viewport = Viewport { origin: [0., 0.], size: [400., 400.] };
        let mut expected = Vec::new();
        for s in fresh.display().segments.iter().filter(|s| {
            let a = fresh.display().points[s[0] as usize]; let b = fresh.display().points[s[1] as usize];
            (a[2] - 6.).abs() < 1e-8 && (b[2] - 6.).abs() < 1e-8
        }).take(2) {
            let a = fresh.display().points[s[0] as usize]; let b = fresh.display().points[s[1] as usize];
            let point = std::array::from_fn(|i| (a[i] + b[i]) * 0.5);
            let projected = camera.project(point, viewport).unwrap().unwrap();
            let reference = super::super::SelectedSubobject { root: id, path: vec![],
                subset: Subelement::BrepEdge(fresh.topology().edges[s[2] as usize]),
                source: controller.document().stamp(id).unwrap(), world_epoch: controller.coordinator().epoch() };
            controller.dupedge_pick_with_point(reference, point, 0.01, &camera, viewport,
                projected.screen, 6., projected.depth).unwrap();
            expected.push([a, b]);
        }
        assert_eq!(expected.len(), 2);
        assert_eq!(controller.dupedge_temporary_picks().count(), 2);
        let before = controller.document().entities.len();
        controller.submit("").unwrap();
        let curves: Vec<_> = controller.document().entities.values().filter(|e| e.id != id).collect();
        assert_eq!(curves.len(), 2);
        for pair in expected {
            assert!(curves.iter().any(|entity| {
                let Geometry::Curve(curve) = &entity.geometry else { return false };
                let endpoints: Vec<_> = [0., 1.].into_iter().map(|t| om9_core::evaluation::evaluate_curve(curve,
                    om9_core::evaluation::Parameter::Fraction(t), 0, om9_core::geometry::Transform::identity(), &limits).unwrap()[0]).collect();
                pair.iter().all(|point| endpoints.iter().any(|end| (0..3).all(|i| (point[i] - end[i]).abs() < 1e-8)))
            }));
        }
        controller.undo().unwrap(); assert_eq!(controller.document().entities.len(), before);
        controller.redo().unwrap(); assert_eq!(controller.document().entities.len(), before + 2);
    }

    #[test]
    fn source_pick_refuses_changed_scene_publication_and_preserves_redo() {
        let limits = Limits::default();
        let shape = om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Box { x_mm: 4., y_mm: 5., z_mm: 6. }, 0.01, &limits,
        ).unwrap();
        let edge = box_x_edge(&shape);
        let mut document = Document::new("stale DupEdge scene");
        let entity = Entity::new("Box", Geometry::Brep(shape), document.current_layer);
        let id = entity.id;
        document.entities.insert(id, entity);
        let mut controller = Controller::new(document, limits).unwrap();
        controller.submit("Point").unwrap(); controller.submit("10,10,10").unwrap(); controller.undo().unwrap();
        controller.submit("DupEdge").unwrap();
        let reference = super::super::SelectedSubobject {
            root: id, path: vec![], subset: Subelement::BrepEdge(edge),
            source: controller.document().stamp(id).unwrap(), world_epoch: controller.coordinator().epoch(),
        };
        let camera = Camera::orthographic([0., 0., 10.], [0., 0., 0.], [0., 1., 0.], 10.).unwrap();
        let viewport = Viewport { origin: [0., 0.], size: [400., 400.] };
        let point = [2., 0., 0.];
        let projected = camera.project(point, viewport).unwrap().unwrap();
        controller.dupedge_pick_with_point(reference, point, 0.01, &camera, viewport,
            projected.screen, 6., projected.depth).unwrap();
        controller.replace_scene_same_epoch_for_test(0.).unwrap();
        let before = controller.coordinator().snapshot();
        assert!(controller.submit("").is_err(), "a pick from a replaced scene must be refused");
        assert_eq!(controller.coordinator().snapshot(), before);
        assert_eq!(controller.coordinator().redo_len(), 1);
    }

    #[test]
    fn preselection_point_is_discarded_after_external_selection_changes() {
        let limits = Limits::default();
        let shape = om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Box { x_mm: 4., y_mm: 5., z_mm: 6. }, 0.01, &limits,
        ).unwrap();
        let edge = shape.topology().edges[0];
        let mut document = Document::new("stale preselection");
        let entity = Entity::new("Box", Geometry::Brep(shape), document.current_layer);
        let id = entity.id;
        document.entities.insert(id, entity);
        let mut controller = Controller::new(document, limits).unwrap();
        let reference = super::super::SelectedSubobject {
            root: id, path: vec![], subset: Subelement::BrepEdge(edge),
            source: controller.document().stamp(id).unwrap(), world_epoch: controller.coordinator().epoch(),
        };
        let camera = Camera::orthographic([0., 0., 10.], [0., 0., 0.], [0., 1., 0.], 10.).unwrap();
        let viewport = Viewport { origin: [0., 0.], size: [400., 400.] };
        let world = [2., 0., 0.];
        let projected = camera.project(world, viewport).unwrap().unwrap();
        controller.dupedge_pick_with_point(reference.clone(), world, 0.01, &camera, viewport,
            projected.screen, 6., projected.depth).unwrap();
        controller.toggle_subobject_ref(reference.clone()).unwrap();
        controller.toggle_subobject_ref(reference).unwrap();
        let before = controller.coordinator().snapshot();
        assert!(controller.submit("DupEdge").is_err());
        assert_eq!(controller.coordinator().snapshot(), before);
    }

    #[test]
    fn deceptive_nurbs_segment_refuses_without_losing_redo() {
        use om9_core::geometry::{Curve, NurbsCurve};
        let limits = Limits::default();
        let curve = Curve::Nurbs(NurbsCurve { degree: 5,
            poles: [0., 3.75, -8.125, 8.125, -3.75, 0.].into_iter().enumerate()
                .map(|(i, y)| [i as f64 * 0.8, y, 0.]).collect(),
            weights: vec![1.; 6], knots: vec![0., 0., 0., 0., 0., 0., 1., 1., 1., 1., 1., 1.],
            domain: [0., 1.], periodic: false, reversed: false, seam: None });
        let profile = Curve::Polycurve { segments: vec![curve,
            Curve::Line { start: [4., 0., 0.], end: [4., -5., 0.], domain: [0., 1.] },
            Curve::Line { start: [4., -5., 0.], end: [0., -5., 0.], domain: [0., 1.] },
            Curve::Line { start: [0., -5., 0.], end: [0., 0., 0.], domain: [0., 1.] },
        ], reversed: false };
        let shape = om9_kernel::extrusion::shell(&profile, [0., 0., 5.], 0.01, &limits).unwrap();
        let mut display = shape.display().clone();
        let selected_segment = *display.segments.iter().find(|s| {
            let a = display.points[s[0] as usize]; let b = display.points[s[1] as usize];
            (a[2] - 5.).abs() < 1e-8 && (b[2] - 5.).abs() < 1e-8
                && (0.5..1.5).contains(&a[0]) && (0.5..1.5).contains(&b[0])
        }).expect("top NURBS segment");
        let edge = selected_segment[2];
        let edge_id = display.topology.edges[edge as usize];
        let a = display.points[selected_segment[0] as usize];
        let b = display.points[selected_segment[1] as usize];
        let picked = std::array::from_fn(|i| (a[i] + b[i]) * 0.5);
        assert!(picked[1].abs() > 1e-4, "replacement pick must differ from the old straight chord");
        let mut original = Document::new("reopened NURBS");
        let entity = Entity::new("shell", Geometry::Brep(shape.clone()), original.current_layer);
        let original_id = entity.id;
        original.entities.insert(original_id, entity);
        let path = std::env::temp_dir().join(format!("om9-dupedge-nurbs-{}.om9", uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&path, &original, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
        let mut app = crate::ui::MatrixApp::from_file(&path).unwrap();
        std::fs::remove_file(path).unwrap();
        app.controller.submit("DupEdge").unwrap();
        let reference = super::super::SelectedSubobject {
            root: original_id, path: vec![], subset: Subelement::BrepEdge(edge_id),
            source: app.controller.document().stamp(original_id).unwrap(), world_epoch: app.controller.coordinator().epoch(),
        };
        let camera = Camera::orthographic([8., -10., 14.], [1., 1., 2.5], [0., 0., 1.], 40.).unwrap();
        let viewport = Viewport { origin: [0., 0.], size: [400., 400.] };
        let projected = camera.project(picked, viewport).unwrap().unwrap();
        app.controller.dupedge_pick_with_point(reference, picked, 0.7, &camera, viewport,
            projected.screen, 7., projected.depth).unwrap();
        let before = app.controller.document().entities.len();
        app.controller.submit("").expect("valid displayed NURBS segment after reopen");
        assert_eq!(app.controller.document().entities.len(), before + 1);
        let output = app.controller.document().entities.values().find(|e| e.id != original_id).unwrap();
        let Geometry::Curve(output_curve) = &output.geometry else { panic!("DupEdge must produce a curve") };
        for fraction in [0.125, 0.375, 0.625, 0.875] {
            let p = om9_core::evaluation::evaluate_curve(output_curve, om9_core::evaluation::Parameter::Fraction(fraction),
                0, om9_core::geometry::Transform::identity(), &limits).unwrap()[0];
            let t = p[0] / 4.;
            let expected_y = 200. * t * (t - 0.25) * (t - 0.5) * (t - 0.75) * (t - 1.);
            assert!((p[1] - expected_y).abs() < 1e-7 && (p[2] - 5.).abs() < 1e-7);
        }
        app.controller.undo().unwrap();
        assert_eq!(app.controller.document().entities.len(), before);
        app.controller.redo().unwrap();
        assert_eq!(app.controller.document().entities.len(), before + 1);
        let first = u32::try_from(display.points.len()).unwrap();
        display.points.extend([[0., 0., 5.], [4., 0., 5.]]);
        display.segments.retain(|segment| segment[2] != edge);
        display.segments.insert(0, [first, first + 1, edge]);
        let altered = om9_core::brep::Brep::from_native(shape.source().to_owned(), display, &limits).unwrap();
        let mut document = Document::new("deceptive NURBS segment");
        let entity = Entity::new("shell", Geometry::Brep(altered), document.current_layer);
        let id = entity.id;
        document.entities.insert(id, entity);
        let path = std::env::temp_dir().join(format!("om9-dupedge-old-display-{}.om9", uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&path, &document, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
        let mut reopened = crate::ui::MatrixApp::from_file(&path).unwrap();
        std::fs::remove_file(path).unwrap();
        let controller = &mut reopened.controller;
        controller.submit("Point").unwrap(); controller.submit("10,10,10").unwrap(); controller.undo().unwrap();
        controller.submit("DupEdge").unwrap();
        let reference = super::super::SelectedSubobject {
            root: id, path: vec![], subset: Subelement::BrepEdge(edge_id),
            source: controller.document().stamp(id).unwrap(), world_epoch: controller.coordinator().epoch(),
        };
        let point = [1., 0., 5.];
        let camera = Camera::orthographic([8., -10., 14.], [1., 1., 2.5], [0., 0., 1.], 40.).unwrap();
        let viewport = Viewport { origin: [0., 0.], size: [400., 400.] };
        let projected = camera.project(point, viewport).unwrap().unwrap();
        controller.dupedge_pick_with_point(reference, point, 0.7, &camera, viewport,
            projected.screen, 7., projected.depth).unwrap();
        let before = controller.coordinator().snapshot();
        controller.dupedge_source_view_active.store(true, std::sync::atomic::Ordering::Release);
        let busy = controller.submit("").unwrap_err().to_string();
        assert!(busy.contains("source view unavailable") && busy.contains("worker is still finishing"), "{busy}");
        assert_eq!(controller.coordinator().snapshot(), before);
        assert_eq!(controller.coordinator().redo_len(), 1);
        assert!(controller.dupedge_needs_pick());
        controller.dupedge_source_view_active.store(false, std::sync::atomic::Ordering::Release);
        controller.submit("").expect("a mismatched segment should start source-view recovery");
        assert_eq!(controller.coordinator().snapshot(), before);
        assert_eq!(controller.coordinator().redo_len(), 1);
        assert!(controller.dupedge_needs_pick());

        assert!(controller.dupedge_source_view_pending(), "old display must schedule source regeneration");
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
        loop {
            if let Some(result) = controller.poll_dupedge_source_view() { result.unwrap(); break; }
            assert!(std::time::Instant::now() < deadline, "source view timed out");
            std::thread::yield_now();
        }
        assert_eq!(controller.coordinator().snapshot(), before);
        assert_eq!(controller.coordinator().redo_len(), 1);
        let stamp = controller.document().stamp(id).unwrap();
        let view = controller.scene().unwrap().root_brep_pick_display(&stamp).unwrap();
        let segment = view.segments.iter().find(|s| {
            let a = view.points[s[0] as usize]; let b = view.points[s[1] as usize];
            (a[2] - 5.).abs() < 1e-8 && (b[2] - 5.).abs() < 1e-8
                && (a[1].abs() + b[1].abs()) > 0.05 && (a[0] - b[0]).abs() > 1e-5
        }).expect("regenerated curved segment");
        let a = view.points[segment[0] as usize]; let b = view.points[segment[1] as usize];
        let point = std::array::from_fn(|i| (a[i] + b[i]) * 0.5);
        let new_edge = view.topology.edges[segment[2] as usize];
        let reference = super::super::SelectedSubobject { root: id, path: vec![],
            subset: Subelement::BrepEdge(new_edge), source: stamp, world_epoch: controller.coordinator().epoch() };
        let projected = camera.project(point, viewport).unwrap().unwrap();
        controller.dupedge_pick_with_point(reference, point, 0.7, &camera, viewport,
            projected.screen, 7., projected.depth).unwrap();
        let count = controller.document().entities.len();
        controller.submit("").expect("regenerated source edge duplicates");
        let result = controller.document().entities.values().find(|entity| entity.id != id).unwrap();
        let Geometry::Curve(curve) = &result.geometry else { panic!("expected curve") };
        for fraction in [0.125, 0.375, 0.625, 0.875] {
            let p = om9_core::evaluation::evaluate_curve(curve, om9_core::evaluation::Parameter::Fraction(fraction),
                0, om9_core::geometry::Transform::identity(), &limits).unwrap()[0];
            let t = p[0] / 4.;
            let expected_y = 200. * t * (t - 0.25) * (t - 0.5) * (t - 0.75) * (t - 1.);
            assert!((p[1] - expected_y).abs() < 1e-7 && (p[2] - 5.).abs() < 1e-7);
        }
        controller.undo().unwrap(); assert_eq!(controller.document().entities.len(), count);
        controller.redo().unwrap(); assert_eq!(controller.document().entities.len(), count + 1);

        let mut preselected = Controller::new(document.clone(), limits).unwrap();
        let reference = super::super::SelectedSubobject { root: id, path: vec![],
            subset: Subelement::BrepEdge(edge_id), source: preselected.document().stamp(id).unwrap(),
            world_epoch: preselected.coordinator().epoch() };
        let point = [1., 0., 5.];
        let projected = camera.project(point, viewport).unwrap().unwrap();
        preselected.dupedge_pick_with_point(reference, point, 0.7, &camera, viewport,
            projected.screen, 7., projected.depth).unwrap();
        let before = preselected.coordinator().snapshot();
        preselected.dupedge_source_view_active.store(true, std::sync::atomic::Ordering::Release);
        let busy = preselected.submit("DupEdge").unwrap_err().to_string();
        assert!(busy.contains("source view unavailable") && busy.contains("worker is still finishing"), "{busy}");
        assert_eq!(preselected.coordinator().snapshot(), before);
        assert_eq!(preselected.subselection().len(), 1);
        assert!(!preselected.dupedge_needs_pick());
        preselected.dupedge_source_view_active.store(false, std::sync::atomic::Ordering::Release);
        preselected.submit("DupEdge").unwrap();
        assert!(preselected.dupedge_source_view_pending(), "preselection must also regenerate the view");
        assert!(preselected.prompt().contains("Preparing"));
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
        loop {
            if let Some(result) = preselected.poll_dupedge_source_view() { result.unwrap(); break; }
            assert!(std::time::Instant::now() < deadline);
            std::thread::yield_now();
        }
        assert!(preselected.dupedge_needs_pick());
        assert!(preselected.subselection().is_empty(), "old display selection must be replaced");
        let stamp = preselected.document().stamp(id).unwrap();
        let view = preselected.scene().unwrap().root_brep_pick_display(&stamp).unwrap();
        let segment = view.segments.iter().find(|s| {
            let a = view.points[s[0] as usize]; let b = view.points[s[1] as usize];
            (a[2] - 5.).abs() < 1e-8 && (b[2] - 5.).abs() < 1e-8
                && (a[1].abs() + b[1].abs()) > 0.05 && (a[0] - b[0]).abs() > 1e-5
        }).unwrap();
        let a = view.points[segment[0] as usize]; let b = view.points[segment[1] as usize];
        let point = std::array::from_fn(|i| (a[i] + b[i]) * 0.5);
        let reference = super::super::SelectedSubobject { root: id, path: vec![],
            subset: Subelement::BrepEdge(view.topology.edges[segment[2] as usize]), source: stamp,
            world_epoch: preselected.coordinator().epoch() };
        let projected = camera.project(point, viewport).unwrap().unwrap();
        preselected.dupedge_pick_with_point(reference, point, 0.7, &camera, viewport,
            projected.screen, 7., projected.depth).unwrap();
        let count = preselected.document().entities.len();
        preselected.submit("").unwrap();
        preselected.undo().unwrap(); assert_eq!(preselected.document().entities.len(), count);
        preselected.redo().unwrap(); assert_eq!(preselected.document().entities.len(), count + 1);

        // Test-only publication of an already paired display; this does not
        // establish a mapping for arbitrary v4 files.
        let mut controller = Controller::new(document.clone(), limits).unwrap();
        let stamp = controller.document().stamp(id).unwrap();
        let (buffers, _) = om9_scene::scene::Scene::prepare_brep_buffers(&shape, &limits, |_, _| Ok(())).unwrap();
        let replacement = controller.scene().unwrap().try_replace_root_brep_display(
            &stamp, &shape, buffers, [0, 0], std::sync::Arc::new(()), std::sync::Arc::new(()),
        ).unwrap();
        controller.publish_owned_scene(replacement, None);
        controller.submit("DupEdge").unwrap();
        let reference = super::super::SelectedSubobject {
            root: id, path: vec![], subset: Subelement::BrepEdge(edge_id),
            source: controller.document().stamp(id).unwrap(), world_epoch: controller.coordinator().epoch(),
        };
        let projected = camera.project(picked, viewport).unwrap().unwrap();
        controller.dupedge_pick_with_point(reference, picked, 0.7, &camera, viewport,
            projected.screen, 7., projected.depth).unwrap();
        let before = controller.document().entities.len();
        controller.submit("").expect("DupEdge must use the published replacement display");
        assert_eq!(controller.document().entities.len(), before + 1);
        controller.undo().unwrap();
        assert_eq!(controller.document().entities.len(), before);
        controller.redo().unwrap();
        assert_eq!(controller.document().entities.len(), before + 1);

        let mut fresh = shape.display().clone();
        let fresh_topology = &mut *fresh.topology;
        for id in &mut fresh_topology.edges { *id = uuid::Uuid::new_v4(); }
        let temporary_edge = fresh_topology.edges[edge as usize];
        let fresh = om9_core::brep::Brep::from_native(shape.source().to_owned(), fresh, &limits).unwrap();
        let mut controller = Controller::new(document, limits).unwrap();
        let stamp = controller.document().stamp(id).unwrap();
        let (buffers, _) = om9_scene::scene::Scene::prepare_brep_buffers(&fresh, &limits, |_, _| Ok(())).unwrap();
        let replacement = controller.scene().unwrap().try_replace_root_brep_display(
            &stamp, &fresh, buffers, [0, 0], std::sync::Arc::new(()), std::sync::Arc::new(()),
        ).unwrap();
        controller.publish_owned_scene(replacement, None);
        assert!(controller.select_subobject(id, Subelement::BrepEdge(temporary_edge)).is_err(),
            "temporary IDs must remain unavailable to general selection");
        controller.submit("DupEdge").unwrap();
        let reference = super::super::SelectedSubobject {
            root: id, path: vec![], subset: Subelement::BrepEdge(temporary_edge),
            source: controller.document().stamp(id).unwrap(), world_epoch: controller.coordinator().epoch(),
        };
        let before_revision = controller.sub_revision();
        controller.dupedge_pick_with_point(reference.clone(), picked, 0.7, &camera, viewport,
            projected.screen, 7., projected.depth).unwrap();
        assert!(controller.subselection().is_empty(), "temporary IDs must not enter persistent selection");
        assert_eq!(controller.dupedge_temporary_picks().count(), 1);
        assert!(controller.sub_revision() > before_revision);
        controller.cancel();
        assert!(controller.dupedge_source_picks.is_empty(), "cancel must release temporary picks");
        controller.submit("DupEdge").unwrap();
        controller.dupedge_pick_with_point(reference.clone(), picked, 0.7, &camera, viewport,
            projected.screen, 7., projected.depth).unwrap();
        controller.replace_scene_same_epoch_for_test(0.).unwrap();
        assert_eq!(controller.dupedge_temporary_picks().count(), 0, "stale picks must not be highlighted");
        let before_stale = controller.coordinator().snapshot();
        assert!(controller.submit("").is_err(), "temporary pick from replaced scene must be refused");
        assert_eq!(controller.coordinator().snapshot(), before_stale);
        let (buffers, _) = om9_scene::scene::Scene::prepare_brep_buffers(&fresh, &limits, |_, _| Ok(())).unwrap();
        let replacement = controller.scene().unwrap().try_replace_root_brep_display(
            &stamp, &fresh, buffers, [0, 0], std::sync::Arc::new(()), std::sync::Arc::new(()),
        ).unwrap();
        controller.publish_owned_scene(replacement, None);
        controller.dupedge_pick_with_point(reference, picked, 0.7, &camera, viewport,
            projected.screen, 7., projected.depth).unwrap();
        assert_eq!(controller.dupedge_temporary_picks().count(), 1, "new pick replaces the stale publication");
        let before = controller.document().entities.len();
        controller.submit("").expect("temporary source-scene edge must duplicate");
        assert_eq!(controller.document().entities.len(), before + 1);
        controller.undo().unwrap(); assert_eq!(controller.document().entities.len(), before);
        controller.redo().unwrap(); assert_eq!(controller.document().entities.len(), before + 1);
    }

    #[test]
    fn display_chord_farther_than_deflection_cannot_pick_source_curve() {
        use om9_core::geometry::Transform;
        let limits = Limits::default();
        for point in [[1., 1., 5.], [1.999, 0.001, 5.]] {
        for placement in [Transform::identity(), Transform([[2.,0.5,0.,3.],[0.,1.,0.,-2.],[0.,0.,0.5,1.],[0.,0.,0.,1.]])] {
        let shape = om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Cylinder { radius_mm: 2., height_mm: 5. }, 0.01, &limits,
        ).unwrap();
        let mut display = shape.display().clone();
        let edge = display.segments.iter().find(|s| {
            let a = display.points[s[0] as usize]; let b = display.points[s[1] as usize];
            (a[2] - 5.).abs() < 1e-8 && (b[2] - 5.).abs() < 1e-8
        }).unwrap()[2];
        let first = u32::try_from(display.points.len()).unwrap();
        display.points.push([2., 0., 5.]); display.points.push([0., 2., 5.]);
        display.segments.push([first, first + 1, edge]);
        let edge_id = display.topology.edges[edge as usize];
        let altered = om9_core::brep::Brep::from_native(shape.source().to_owned(), display, &limits).unwrap();
        let mut document = Document::new("misleading display chord");
        let mut entity = Entity::new("cylinder", Geometry::Brep(altered), document.current_layer);
        entity.placement = placement;
        let id = entity.id;
        document.entities.insert(id, entity);
        let mut controller = Controller::new(document, limits).unwrap();
        controller.submit("Point").unwrap(); controller.submit("10,10,10").unwrap(); controller.undo().unwrap();
        controller.submit("DupEdge").unwrap();
        let reference = super::super::SelectedSubobject {
            root: id, path: vec![], subset: Subelement::BrepEdge(edge_id),
            source: controller.document().stamp(id).unwrap(), world_epoch: controller.coordinator().epoch(),
        };
        let camera = Camera::orthographic([8., -10., 14.], [1., 1., 2.5], [0., 0., 1.], 40.).unwrap();
        let viewport = Viewport { origin: [0., 0.], size: [400., 400.] };
        let world = placement.point(point).unwrap();
        let projected = camera.project(world, viewport).unwrap().unwrap();
        controller.dupedge_pick_with_point(reference, world, 0.7, &camera, viewport,
            projected.screen, 7., projected.depth).unwrap();
        let before = controller.coordinator().snapshot();
        let result = controller.submit("");
        if result.is_ok() { assert!(controller.dupedge_source_view_pending()); }
        assert_eq!(controller.coordinator().snapshot(), before);
        assert_eq!(controller.coordinator().redo_len(), 1);
        assert!(controller.dupedge_needs_pick());
        }
        }
    }

    #[test]
    fn changed_source_after_pick_refuses_without_losing_new_redo() {
        use om9_core::{geometry::Transform, transaction::{Mutation, TransactionRequest}};
        let limits = Limits::default();
        for replace_geometry in [false, true] {
            let shape = om9_kernel::creation::primitive(
                om9_kernel::creation::Primitive::Box { x_mm: 4., y_mm: 5., z_mm: 6. }, 0.01, &limits,
            ).unwrap();
            let edge = box_x_edge(&shape);
            let mut document = Document::new("stale DupEdge source");
            let entity = Entity::new("Box", Geometry::Brep(shape), document.current_layer);
            let id = entity.id;
            document.entities.insert(id, entity);
            let mut controller = Controller::new(document, limits).unwrap();
            controller.submit("DupEdge").unwrap();
            let reference = super::super::SelectedSubobject {
                root: id, path: vec![], subset: Subelement::BrepEdge(edge),
                source: controller.document().stamp(id).unwrap(), world_epoch: controller.coordinator().epoch(),
            };
            let camera = Camera::orthographic([0., 0., 10.], [0., 0., 0.], [0., 1., 0.], 10.).unwrap();
            let viewport = Viewport { origin: [0., 0.], size: [400., 400.] };
            let world = [2., 0., 0.];
            let projected = camera.project(world, viewport).unwrap().unwrap();
            controller.dupedge_pick_with_point(reference, world, 0.01, &camera, viewport,
                projected.screen, 6., projected.depth).unwrap();
            let source = controller.document().stamp(id).unwrap();
            let mutation = if replace_geometry {
                let replacement = om9_kernel::creation::primitive(
                    om9_kernel::creation::Primitive::Box { x_mm: 8., y_mm: 5., z_mm: 6. }, 0.01, &limits,
                ).unwrap();
                Mutation::Geometry { source, geometry: Geometry::Brep(replacement) }
            } else {
                Mutation::Placement { source, placement: Transform::translation([1., 0., 0.]) }
            };
            let prepared = controller.coordinator.prepare(TransactionRequest::new("external source edit", vec![mutation])).unwrap();
            controller.coordinator.commit(prepared).unwrap();
            let point = Entity::new("redo point", Geometry::Point([20., 20., 20.]), controller.document().current_layer);
            let prepared = controller.coordinator.prepare(TransactionRequest::new("external point", vec![
                Mutation::Insert { document: controller.document, entity: point },
            ])).unwrap();
            controller.coordinator.commit(prepared).unwrap();
            controller.coordinator.undo().unwrap();
            controller.refresh_scene().unwrap();
            let before = controller.coordinator().snapshot();
            assert_eq!(controller.coordinator().redo_len(), 1);
            assert!(matches!(controller.submit(""), Err(Error::Stale(_))));
            assert_eq!(controller.coordinator().snapshot(), before);
            assert_eq!(controller.coordinator().redo_len(), 1);
            assert!(!controller.dupedge_needs_pick(), "a changed command context must be restarted");
        }
    }

    #[test]
    fn later_unmatched_source_pick_keeps_all_inputs_and_redo() {
        let limits = Limits::default();
        let shape = om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Box { x_mm: 4., y_mm: 5., z_mm: 6. }, 0.01, &limits,
        ).unwrap();
        let first = box_x_edge(&shape);
        let edges = [first, *shape.topology().edges.iter().find(|edge| **edge != first).unwrap()];
        let mut document = Document::new("source pick late refusal");
        let entity = Entity::new("Box", Geometry::Brep(shape), document.current_layer);
        let id = entity.id;
        document.entities.insert(id, entity);
        let mut controller = Controller::new(document, limits).unwrap();
        controller.submit("Point").unwrap(); controller.submit("1,2,3").unwrap(); controller.undo().unwrap();
        controller.submit("DupEdge").unwrap();
        let camera = Camera::orthographic([0., 0., 10.], [0., 0., 0.], [0., 1., 0.], 10.).unwrap();
        let viewport = Viewport { origin: [0., 0.], size: [400., 400.] };
        for (edge, point) in edges.into_iter().zip([[2., 0., 0.], [1., 1., 1.]]) {
            let reference = super::super::SelectedSubobject {
                root: id, path: vec![], subset: Subelement::BrepEdge(edge),
                source: controller.document().stamp(id).unwrap(), world_epoch: controller.coordinator().epoch(),
            };
            let projected = camera.project(point, viewport).unwrap().unwrap();
            controller.dupedge_pick_with_point(reference, point, 0.01, &camera, viewport,
                projected.screen, 6., projected.depth).unwrap();
        }
        let before = controller.coordinator().snapshot();
        assert!(matches!(controller.submit(""), Err(Error::Stale("DupEdge display pick"))));
        assert_eq!(controller.coordinator().snapshot(), before);
        assert_eq!(controller.coordinator().redo_len(), 1);
        assert_eq!(controller.subselection().len(), 2);
        assert!(controller.dupedge_needs_pick());
    }

    #[test]
    fn source_point_outside_pick_screen_preserves_redo_and_pending_input() {
        let limits = Limits::default();
        let shape = om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Box { x_mm: 4., y_mm: 5., z_mm: 6. }, 0.01, &limits,
        ).unwrap();
        let edge = box_x_edge(&shape);
        let mut document = Document::new("source pick viewport guard");
        let entity = Entity::new("Box", Geometry::Brep(shape), document.current_layer);
        let id = entity.id;
        document.entities.insert(id, entity);
        let mut controller = Controller::new(document, limits).unwrap();
        controller.submit("Point").unwrap(); controller.submit("1,2,3").unwrap(); controller.undo().unwrap();
        controller.submit("DupEdge").unwrap();
        let reference = super::super::SelectedSubobject {
            root: id, path: vec![], subset: Subelement::BrepEdge(edge),
            source: controller.document().stamp(id).unwrap(), world_epoch: controller.coordinator().epoch(),
        };
        let camera = Camera::orthographic([0., 0., 10.], [0., 0., 0.], [0., 1., 0.], 10.).unwrap();
        let viewport = Viewport { origin: [0., 0.], size: [400., 400.] };
        let world = [2., 0., 0.];
        let projected = camera.project(world, viewport).unwrap().unwrap();
        controller.dupedge_pick_with_point(reference.clone(), world, 0.01, &camera, viewport,
            [projected.screen[0] + 50., projected.screen[1]], 6., projected.depth).unwrap();
        let before = controller.coordinator().snapshot();
        assert!(matches!(controller.submit(""), Err(Error::Invalid("DupEdge source edge does not match viewport pick"))));
        assert_eq!(controller.coordinator().snapshot(), before);
        assert_eq!(controller.coordinator().redo_len(), 1);
        assert_eq!(controller.subselection().len(), 1);
        assert!(controller.dupedge_needs_pick());
        controller.dupedge_pick(reference.clone()).unwrap();
        controller.dupedge_pick_with_point(reference, world, 0.01, &camera, viewport,
            projected.screen, 6., projected.depth + 1.).unwrap();
        assert!(matches!(controller.submit(""), Err(Error::Invalid("DupEdge source edge does not match viewport pick"))));
        assert_eq!(controller.coordinator().snapshot(), before);
        assert_eq!(controller.coordinator().redo_len(), 1);
        assert_eq!(controller.subselection().len(), 1);
    }
}
