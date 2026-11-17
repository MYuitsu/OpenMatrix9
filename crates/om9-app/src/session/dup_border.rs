//! Duplicate Border (`DupBorder`, OM9-CURVE-039): surface and B-rep boundary curves.

use om9_core::geometry::{Curve, Geometry};
use om9_core::surface_kernel::SurfaceIso;

use super::{Controller, Error};

pub const DUP_BORDER: &str = "OM9-CURVE-039";

#[cfg(feature = "occt-brep")]
fn source_boundaries_with_admission(
    brep: &dyn om9_core::brep::BrepSourceAccess,
    limits: &om9_core::budget::Limits,
    admit: &mut impl FnMut(u64,u64) -> Result<(),Error>,
) -> Result<Vec<Curve>,Error> {
    super::wireframe::source_edges_with_admission(brep,limits,true,admit)
}

pub(super) fn closed_surface_axis(surface: &om9_core::geometry::NurbsSurface, axis: usize,
    limits: &om9_core::budget::Limits) -> Result<bool, Error> {
    let degree = surface.degree[axis]; let count = surface.size[axis]; let knots = &surface.knots[axis];
    if surface.domain[axis] != [knots[degree], knots[count]] { return Ok(false); }
    if surface.periodic[axis] {
        surface.certify_repeated_periodic_basis(limits)?;
        return Ok(true);
    }
    if !knots[..=degree].iter().all(|k| *k == knots[degree])
        || !knots[count..].iter().all(|k| *k == knots[count]) { return Ok(false); }
    Ok((0..surface.size[1-axis]).all(|j| {
        let (a,b)=if axis==0 {(j,(count-1)*surface.size[1]+j)} else {(j*surface.size[1],j*surface.size[1]+count-1)};
        surface.poles[a] == surface.poles[b] && surface.weights[a] == surface.weights[b]
    }))
}

impl Controller {
    pub fn dispatch_dup_border(&mut self) -> Result<(), Error> {
        if self.selection.len() > 1 {
            return self.duplicate_multiple_borders();
        }
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
        #[cfg(feature = "occt-brep")]
        if matches!(entity.geometry, Geometry::NativeBrep(_)) {
            return self.duplicate_multiple_borders();
        }
        if matches!(entity.geometry, Geometry::Surface(_)) && entity.extensions.contains_key(om9_core::model::TRIM_SURFACE_EXTENSION_SLOT) {
            return self.duplicate_trimmed_surface_border(id);
        }
        if matches!(entity.geometry, Geometry::Surface(_)) {
            return self.duplicate_multiple_borders();
        }
        let geometry = entity.geometry.clone();
        let placement = entity.placement;
        match geometry {
            Geometry::Brep(brep) => {
                #[cfg(not(feature = "occt-brep"))]
                {
                    let _ = (brep, placement);
                    Err(Error::Unsupported("exact B-Rep boundary requires OCCT"))
                }
                #[cfg(feature = "occt-brep")]
                {
                use om9_core::budget::{Usage, memory::MemoryClass};
                let limits = *self.coordinator.limits();
                let _source = self.coordinator.capture_tracked_document(self.document)?;
                let held = self.scene_cache_usage()?;
                let physical = || om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
                let mut reservation = self.coordinator.reserve_external_memory_with_held(MemoryClass::Kernel, 1, physical(), held)?;
                let mut admit = |live: u64, pending: u64| -> Result<(), Error> {
                    let needed = live.checked_add(pending).ok_or(Error::Overflow("DupBorder scratch"))?.max(1);
                    if needed > reservation.bytes() {
                        let extra = needed - reservation.bytes();
                        self.coordinator.grow_external_memory_with_held(&mut reservation, extra, physical(), held)?;
                    }
                    reservation.set_allocated_bytes(live)?;
                    reservation.recheck_pending()
                };
                let curves = source_boundaries_with_admission(&brep, &limits, &mut admit)?;
                let curve_bytes = curves.iter().try_fold(Usage::array::<Curve>(curves.capacity())?.bytes,
                    |n, curve| -> Result<u64, Error> { n.checked_add(curve.validate(&limits)?.bytes).ok_or(Error::Overflow("DupBorder curves")) })?;
                let output_bytes = curve_bytes.checked_mul(2).and_then(|n| n.checked_add(
                    Usage::array::<(&str, Geometry)>(curves.len()).ok()?.bytes)).ok_or(Error::Overflow("DupBorder placement"))?;
                admit(0, output_bytes)?;
                let mut items = Vec::new();
                items.try_reserve_exact(curves.len()).map_err(|_| Error::Budget("DupBorder outputs"))?;
                for curve in curves {
                    items.push(("DupBorder", Geometry::Curve(om9_core::top11::bake_curve(&curve, placement)?)));
                }
                admit(output_bytes, 0)?;
                drop(admit);
                self.commit_geometries(DUP_BORDER, "DupBorder", items)?;
                Ok(())
                }
            }
            _ => Err(Error::Invalid("surface or B-Rep selection")),
        }
    }

    fn duplicate_multiple_borders(&mut self) -> Result<(), Error> {
        use om9_core::budget::{Usage, memory::MemoryClass};
        let source = self.coordinator.capture_tracked_document(self.document)?;
        let scene = self.scene.as_ref().ok_or(Error::Missing("DupBorder scene"))?;
        if scene.epoch != self.coordinator.epoch() { return Err(Error::Stale("DupBorder selection")); }
        let limits = *self.coordinator.limits();
        for id in &self.selection {
            let entity = source.entities.get(id).ok_or(Error::Stale("DupBorder source"))?;
            let permissions = source.permissions(*id, false)?;
            if !permissions.visible || !permissions.selectable { return Err(Error::Invalid("DupBorder source is not selectable")); }
            match &entity.geometry {
                Geometry::Surface(surface) => { surface.validate(&limits)?; }
                #[cfg(feature = "occt-brep")]
                Geometry::Brep(_) | Geometry::NativeBrep(_) => {}
                _ => return Err(Error::Unsupported("DupBorder multiple selection requires surfaces or B-Reps")),
            }
            entity.placement.validate()?;
        }
        let held = self.scene_cache_usage()?;
        let physical = || om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
        let mut reservation = self.coordinator.reserve_external_memory_with_held(MemoryClass::Kernel, 1, physical(), held)?;
        let retained = std::cell::Cell::new(0_u64);
        let mut admit = |live: u64, pending: u64| -> Result<(), Error> {
            let live = retained.get().checked_add(live).ok_or(Error::Overflow("DupBorder outputs"))?;
            let needed = live.checked_add(pending).ok_or(Error::Overflow("DupBorder scratch"))?.max(1);
            if needed > reservation.bytes() {
                let extra = needed - reservation.bytes();
                self.coordinator.grow_external_memory_with_held(&mut reservation, extra, physical(), held)?;
            }
            reservation.set_allocated_bytes(live)?; reservation.recheck_pending()
        };
        let mut items: Vec<(&str, Geometry, om9_core::geometry::Transform)> = Vec::new();
        for id in &self.selection {
            let entity = &source.entities[id];
            #[cfg(feature = "occt-brep")]
            if matches!(&entity.geometry, Geometry::Brep(_) | Geometry::NativeBrep(_)) {
                let curves = match &entity.geometry {
                    Geometry::Brep(brep) => source_boundaries_with_admission(brep,&limits,&mut admit)?,
                    Geometry::NativeBrep(native) => {
                        let tolerance = source.tolerance().map_or(1e-6, |t| t.absolute_mm.min(1e-6));
                        let native_source = om9_kernel::brep::from_3dm_source_typed_admitted(&native.payload,tolerance,&limits,&mut admit)?;
                        let bytes = u64::try_from(native_source.capacity()).map_err(|_|Error::Overflow("DupBorder native source"))?;
                        let mut refused = None;
                        let mut guarded = |live:u64,pending:u64| {
                            let live = bytes.checked_add(live).ok_or(Error::Overflow("DupBorder native source"))?;
                            match admit(live,pending) { Ok(()) => Ok(()), Err(error) => { refused=Some(error.clone()); Err(error) } }
                        };
                        om9_kernel::editing::boundary_edges_from_source_admitted(&native_source,&limits,&mut guarded)
                            .map_err(|error|refused.unwrap_or_else(||super::wireframe::edge_extraction_error(error,"DupBorder native boundary")))?
                    }
                    _ => unreachable!(),
                };
                if curves.is_empty() {return Err(Error::Invalid("B-Rep has no boundary"));}
                let total=items.len().checked_add(curves.len()).ok_or(Error::Overflow("DupBorder outputs"))?;
                if total as u64 > limits.max_outputs {return Err(Error::Budget("DupBorder outputs"));}
                let curve_bytes=curves.iter().try_fold(Usage::array::<Curve>(curves.capacity())?.bytes,
                    |n,curve| -> Result<u64,Error> {n.checked_add(curve.validate(&limits)?.bytes).ok_or(Error::Overflow("DupBorder curves"))})?;
                let slots=Usage::array::<(&str,Geometry,om9_core::geometry::Transform)>(total)?.bytes;
                let pending=curve_bytes.checked_mul(2).and_then(|n|n.checked_add(slots.checked_mul(2)?)).ok_or(Error::Overflow("DupBorder outputs"))?;
                admit(0,pending)?;
                let old=Usage::array::<(&str,Geometry,om9_core::geometry::Transform)>(items.capacity())?.bytes;
                super::wireframe::reserve_wire(&mut items,curves.len())?;
                let new=Usage::array::<(&str,Geometry,om9_core::geometry::Transform)>(items.capacity())?.bytes;
                admit(0,pending.checked_add(new).ok_or(Error::Overflow("DupBorder outputs"))?)?;
                for curve in curves {items.push(("DupBorder",Geometry::Curve(curve),entity.placement));}
                let additional=curve_bytes.checked_mul(2).and_then(|n|n.checked_add(new-old)).ok_or(Error::Overflow("DupBorder outputs"))?;
                retained.set(retained.get().checked_add(additional).ok_or(Error::Overflow("DupBorder outputs"))?);admit(0,0)?;
                continue;
            }
            let Geometry::Surface(surface) = &entity.geometry else { unreachable!() };
            if entity.extensions.contains_key(om9_core::model::TRIM_SURFACE_EXTENSION_SLOT) {
                super::wireframe::append_trimmed_surface_edges(entity,&mut items,"DupBorder",&limits,source.tolerance().map_or(1e-6,|t|t.absolute_mm.min(1e-6)),&retained,&mut admit)?;
                continue;
            }
            let axis_points=surface.size[0].checked_add(surface.size[1]).ok_or(Error::Overflow("DupBorder surface axes"))?;
            let axis_knots=surface.knots[0].len().checked_add(surface.knots[1].len()).ok_or(Error::Overflow("DupBorder surface knots"))?;
            let slots=items.len().checked_add(4).ok_or(Error::Overflow("DupBorder outputs"))?;
            let scratch=Usage::array::<om9_core::geometry::Point3>(axis_points)?.bytes.checked_mul(4)
                .and_then(|n|n.checked_add(Usage::array::<f64>(axis_points).ok()?.bytes.checked_mul(6)?))
                .and_then(|n|n.checked_add(Usage::array::<f64>(axis_knots).ok()?.bytes.checked_mul(4)?))
                .and_then(|n|n.checked_add(Usage::array::<(&str,Geometry,om9_core::geometry::Transform)>(slots).ok()?.bytes.checked_mul(2)?))
                .and_then(|n|n.checked_add(4096)).ok_or(Error::Overflow("DupBorder surface outputs"))?;
            admit(0,scratch)?;
            let closed_u=closed_surface_axis(surface,0,&limits)?; let closed_v=closed_surface_axis(surface,1,&limits)?;
            let [[u0,u1],[v0,v1]]=surface.domain;
            let old=Usage::array::<(&str,Geometry,om9_core::geometry::Transform)>(items.capacity())?.bytes;
            super::wireframe::reserve_wire(&mut items,4)?;
            let new=Usage::array::<(&str,Geometry,om9_core::geometry::Transform)>(items.capacity())?.bytes;
            admit(0,scratch.checked_add(new).ok_or(Error::Overflow("DupBorder outputs"))?)?;
            let first=items.len(); let mut curve_bytes=0_u64;
            for (iso,value,closed) in [(SurfaceIso::U,v0,closed_v),(SurfaceIso::U,v1,closed_v),
                (SurfaceIso::V,u0,closed_u),(SurfaceIso::V,u1,closed_u)] {
                if closed || super::wireframe::collapsed_clamped_surface_boundary(surface,iso,value) {continue;}
                let curve=Curve::Nurbs(om9_core::surface_kernel::isocurve(surface,iso,value,&limits)?);
                curve_bytes=curve_bytes.checked_add(curve.validate(&limits)?.bytes).ok_or(Error::Overflow("DupBorder curves"))?;
                items.push(("DupBorder",Geometry::Curve(curve),entity.placement));
                if items.len() as u64 > limits.max_outputs {return Err(Error::Budget("DupBorder outputs"));}
            }
            if items.len()==first {return Err(Error::Invalid("closed surface has no boundary"));}
            let additional=curve_bytes.checked_mul(2).and_then(|n|n.checked_add(new-old)).ok_or(Error::Overflow("DupBorder outputs"))?;
            retained.set(retained.get().checked_add(additional).ok_or(Error::Overflow("DupBorder outputs"))?); admit(0,0)?;
        }
        drop(admit);
        self.commit_placed_geometries(DUP_BORDER,"DupBorder",items.into_iter())
    }

    fn duplicate_trimmed_surface_border(&mut self, id: om9_core::identity::ObjectId) -> Result<(), Error> {
        use om9_core::budget::memory::MemoryClass;
        let source = self.coordinator.capture_tracked_document(self.document)?;
        let entity = source.entities.get(&id).ok_or(Error::Stale("DupBorder source"))?;
        let scene = self.scene.as_ref().ok_or(Error::Missing("DupBorder scene"))?;
        if scene.epoch != self.coordinator.epoch() { return Err(Error::Stale("DupBorder selection")); }
        let permissions = source.permissions(id, false)?;
        if !permissions.visible || !permissions.selectable { return Err(Error::Invalid("DupBorder source is not selectable")); }
        entity.placement.validate()?;
        let limits = *self.coordinator.limits();
        let held = self.scene_cache_usage()?;
        let physical = || om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
        let mut reservation = self.coordinator.reserve_external_memory_with_held(MemoryClass::Kernel, 1, physical(), held)?;
        let retained = std::cell::Cell::new(0_u64);
        let mut admit = |live: u64, pending: u64| -> Result<(), Error> {
            let live = retained.get().checked_add(live).ok_or(Error::Overflow("DupBorder outputs"))?;
            let needed = live.checked_add(pending).ok_or(Error::Overflow("DupBorder scratch"))?.max(1);
            if needed > reservation.bytes() {
                let extra = needed - reservation.bytes();
                self.coordinator.grow_external_memory_with_held(&mut reservation, extra, physical(), held)?;
            }
            reservation.set_allocated_bytes(live)?;
            reservation.recheck_pending()
        };
        let mut items = Vec::new();
        super::wireframe::append_trimmed_surface_edges(entity, &mut items, "DupBorder", &limits, source.tolerance().map_or(1e-6,|t|t.absolute_mm.min(1e-6)), &retained, &mut admit)?;
        drop(admit);
        self.commit_placed_geometries(DUP_BORDER, "DupBorder", items.into_iter())
    }
}

#[cfg(all(test, feature = "occt-brep"))]
mod admission_tests {
    use super::*;
    #[test]
    fn source_boundary_preserves_callback_refusal_but_not_structural_limit() {
        let limits=om9_core::budget::Limits::default();
        let circle=om9_core::curve_build::circle_on_frame([0.;3],[1.,0.,0.],[0.,1.,0.],2.).unwrap();
        let shape=om9_kernel::extrusion::shell(&circle,[0.,0.,5.],0.1,&limits).unwrap();
        let mut calls=0;let result=source_boundaries_with_admission(&shape,&limits,&mut |_,pending| {
            calls+=1;if pending>0 {Err(Error::Budget("managed memory"))} else {Ok(())}
        });
        assert!(calls>1);assert!(matches!(result,Err(Error::Budget("managed memory"))));
        let limited=om9_core::budget::Limits {max_outputs:1,..limits};
        let result=source_boundaries_with_admission(&shape,&limited,&mut |_,_| Ok(()));
        assert!(matches!(result,Err(Error::Io(_))));
    }
}

#[cfg(all(test, not(feature = "occt-brep")))]
mod trimmed_plane_tests {
    use super::*;
    use om9_core::{budget::Limits, model::{Document, Entity}, rhino_semantics::{set_trimmed_surface, TrimmedSurface}};

    #[test]
    fn dupborder_bilinear_diagonal_trim_without_occt() {
        let limits=Limits::default();for rational in [false,true] {
        let mut surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();surface.poles[3][2]=4.;
        if rational {surface.weights=vec![1.,2.,3.,4.];}
        let trim=TrimmedSurface {underlying:surface.clone(),outer:vec![[0.1,0.1],[0.8,0.2],[0.7,0.9],[0.2,0.8]],inners:vec![],reversed:false};
        let mut entity=Entity::new("warped",Geometry::Surface(surface),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();let id=entity.id;
        let mut doc=Document::new("bilinear trim");doc.entities.insert(id,entity);let mut c=Controller::new(doc,limits).unwrap();c.select_objects(&[id]).unwrap();let before=c.document().clone();
        c.submit("DupBorder").unwrap();assert_eq!(c.document().entities.values().filter(|e|e.label=="DupBorder").count(),4);
        let after=c.document().clone();c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
        }
    }

    #[test]
    fn dupborder_trimmed_cylinder_works_without_occt() {
        let limits=Limits::default();let surface=om9_core::surface_kernel::cylinder([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,5.,&limits).unwrap();
        let trim=TrimmedSurface {underlying:surface.clone(),outer:vec![[0.1,0.2],[0.8,0.2],[0.8,0.8],[0.1,0.8]],inners:vec![],reversed:false};
        let mut entity=Entity::new("trim cylinder",Geometry::Surface(surface),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();let id=entity.id;
        let mut doc=Document::new("trim cylinder");doc.entities.insert(id,entity);let mut c=Controller::new(doc,limits).unwrap();c.select_objects(&[id]).unwrap();
        let before=c.document().clone();c.submit("DupBorder").unwrap();assert_eq!(c.document().entities.values().filter(|e|e.label=="DupBorder").count(),4);
        let after=c.document().clone();c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }

    #[test]
    fn dupborder_two_surfaces_work_without_occt() {
        let limits=Limits::default();
        let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
        let a=Entity::new("a",Geometry::Surface(surface.clone()),None);let b=Entity::new("b",Geometry::Surface(surface),None);
        let (a_id,b_id)=(a.id,b.id);let mut doc=Document::new("two surfaces");doc.entities.insert(a_id,a);doc.entities.insert(b_id,b);
        let mut c=Controller::new(doc,limits).unwrap();c.select_objects(&[a_id,b_id]).unwrap();let before=c.document().clone();
        c.submit("DupBorder").unwrap();assert_eq!(c.document().entities.values().filter(|e|e.label=="DupBorder").count(),8);
        let after=c.document().clone();c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }

    #[test]
    fn dupborder_conical_surface_works_without_occt() {
        let limits = Limits::default();
        let mut surface = om9_core::surface_kernel::cylinder([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,5.,&limits).unwrap();
        for u in 0..surface.size[0] { surface.poles[u*surface.size[1]+1] = [0.,0.,5.]; }
        let entity = Entity::new("cone",Geometry::Surface(surface),None); let id = entity.id;
        let mut doc = Document::new("cone"); doc.entities.insert(id,entity);
        let mut c = Controller::new(doc,limits).unwrap(); c.select_objects(&[id]).unwrap(); let before = c.document().clone();
        c.submit("DupBorder").unwrap(); assert_eq!(c.document().entities.values().filter(|e| e.label == "DupBorder").count(),1);
        let after = c.document().clone(); c.undo().unwrap(); assert_eq!(c.document(),&before); c.redo().unwrap(); assert_eq!(c.document(),&after);
    }

    #[test]
    fn dupborder_trimmed_plane_works_without_occt() {
        let limits = Limits::default();
        let surface = om9_core::surface_kernel::plane([0.,0.,0.], [1.,0.,0.], [0.,1.,0.], 10., 10., &limits).unwrap();
        let trim = TrimmedSurface { underlying: surface.clone(), outer: vec![[0.2,0.2],[0.8,0.2],[0.8,0.8],[0.2,0.8]],
            inners: vec![vec![[0.4,0.4],[0.6,0.4],[0.5,0.6]]], reversed: false };
        let mut entity = Entity::new("trimmed", Geometry::Surface(surface), None);
        set_trimmed_surface(&mut entity, &trim, &limits).unwrap();
        let id = entity.id; let mut doc = Document::new("trimmed"); doc.entities.insert(id, entity);
        let mut c = Controller::new(doc, limits).unwrap(); c.select_objects(&[id]).unwrap();
        let before = c.document().clone(); c.submit("DupBorder").unwrap();
        assert_eq!(c.document().entities.values().filter(|e| e.label == "DupBorder").count(), 7);
        c.undo().unwrap(); assert_eq!(c.document(), &before); c.redo().unwrap();
        assert_eq!(c.document().entities.values().filter(|e| e.label == "DupBorder").count(), 7);
    }
}
