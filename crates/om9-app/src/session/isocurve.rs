//! Extract Isocurve (`ExtractIsocurve`, OM9-CURVE-019).

use om9_core::geometry::{Curve, Geometry};
use om9_core::surface_kernel::SurfaceIso;

use super::{Controller, Error};

pub const ISOCURVE: &str = "OM9-CURVE-019";

#[cfg(feature="occt-brep")]
#[derive(Clone)]
pub(crate) struct NativeIsocurvePick {
    reference: super::SelectedSubobject,
    revision: u64,
    publication: uuid::Uuid,
    pub(super) ray: om9_kernel::editing::IsocurveFaceRay,
    world_reference: om9_core::geometry::Point3,
    world_limit: f64,
    camera: om9_scene::camera::Camera,
    viewport: om9_scene::camera::Viewport,
    cursor: [f64;2],
    pixels: f64,
}

#[derive(Clone)]
pub struct IsoPending {
    pub source: om9_core::identity::SourceStamp,
    pub(crate) surface: Option<std::sync::Arc<om9_core::geometry::NurbsSurface>>,
    #[cfg(feature="occt-brep")]
    pub(crate) face: Option<(super::SelectedSubobject, om9_kernel::editing::IsocurveFaceWitness, u64)>,
    #[cfg(feature="occt-brep")]
    pub(crate) native_face: Option<NativeIsocurvePick>,
    pub(crate) _surface_reservation: Option<std::sync::Arc<om9_core::transaction::ExternalMemoryReservation>>,
    pub placement: om9_core::geometry::Transform,
    pub iso: SurfaceIso,
    pub value: f64,
}

#[cfg(test)]
mod pending_memory_tests {
    use super::*;
    use om9_core::{budget::{Limits,memory::MemoryClass},model::{Document,Entity}};

    #[test]
    fn copied_pending_session_keeps_reservation_until_last_owner_drops() {
        let limits=Limits::default();
        let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
        let entity=Entity::new("surface",Geometry::Surface(surface),None);let id=entity.id;
        let mut doc=Document::new("pending owner");doc.entities.insert(id,entity);
        let mut controller=Controller::new(doc,limits).unwrap();controller.select_objects(&[id]).unwrap();
        let before=controller.coordinator().external_reserved_bytes(MemoryClass::Kernel);
        controller.dispatch_isocurve().unwrap();let held=controller.coordinator().external_reserved_bytes(MemoryClass::Kernel);
        assert!(held>before);let cloned=controller.isocurve_pending.as_ref().unwrap().clone();
        assert!(controller.submit_isocurve_text("Cancel").unwrap().is_ok());
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel),held);
        drop(cloned);assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel),before);
    }
}

impl std::fmt::Debug for IsoPending {
    fn fmt(&self, f:&mut std::fmt::Formatter<'_>)->std::fmt::Result {
        f.debug_struct("IsoPending").field("source",&self.source).field("surface",&self.surface)
            .field("placement",&self.placement).field("iso",&self.iso).field("value",&self.value).finish_non_exhaustive()
    }
}

#[cfg(all(test, not(feature = "occt-brep")))]
mod curved_trim_no_occt_tests {
    use super::*;
    use om9_core::{budget::Limits,model::{Document,Entity},rhino_semantics::{set_trimmed_surface,TrimmedSurface}};
    #[test]
    fn full_seam_trimmed_cylinder_without_occt() {
        let limits=Limits::default();let surface=om9_core::surface_kernel::cylinder([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,5.,&limits).unwrap();
        let trim=TrimmedSurface {underlying:surface.clone(),outer:vec![[0.,0.2],[1.,0.2],[1.,0.8],[0.,0.8]],inners:vec![],reversed:false};
        let mut entity=Entity::new("cylinder",Geometry::Surface(surface),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();let id=entity.id;
        let mut doc=Document::new("seam");doc.entities.insert(id,entity);
        for (command,label,count) in [("DupBorder","DupBorder",2),("ExtractIsocurve","ExtractIsocurve",1)] {
            let mut c=Controller::new(doc.clone(),limits).unwrap();c.select_objects(&[id]).unwrap();let before=c.document().clone();c.submit(command).unwrap();
            if command=="ExtractIsocurve" {c.submit("U=0.5").unwrap();c.submit("").unwrap();}
            assert_eq!(c.document().entities.values().filter(|e|e.label==label).count(),count);let after=c.document().clone();
            c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
        }
    }

    #[test]
    fn trimmed_cylinder_isocurve_without_occt() {
        let limits=Limits::default();let surface=om9_core::surface_kernel::cylinder([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,5.,&limits).unwrap();
        let trim=TrimmedSurface {underlying:surface.clone(),outer:vec![[0.1,0.2],[0.8,0.2],[0.8,0.8],[0.1,0.8]],
            inners:vec![vec![[0.3,0.4],[0.6,0.4],[0.6,0.6],[0.3,0.6]]],reversed:false};
        let mut entity=Entity::new("cylinder",Geometry::Surface(surface),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();let id=entity.id;
        let mut doc=Document::new("cylinder isocurve");doc.entities.insert(id,entity);let mut c=Controller::new(doc,limits).unwrap();c.select_objects(&[id]).unwrap();
        let before=c.document().clone();c.submit("ExtractIsocurve").unwrap();c.submit("U=0.5").unwrap();c.submit("").unwrap();
        assert_eq!(c.document().entities.values().filter(|e|e.label=="ExtractIsocurve").count(),2);
        let after=c.document().clone();c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }
}

impl IsoPending {
    pub fn prompt(&self) -> &'static str {
        "U= or V= parameter, or Enter to extract"
    }
}

impl Controller {
    /// Preserve a viewport ray for a root NativeBrep or editable B-rep face pick.
    /// Source resolution occurs later, after command admission.
    #[cfg(feature="occt-brep")]
    pub fn toggle_native_isocurve_face_hit(&mut self, hit:&om9_scene::scene::PickHit,
        camera:&om9_scene::camera::Camera,viewport:om9_scene::camera::Viewport,cursor:[f64;2],pixels:f64) -> Result<(),Error> {
        use om9_scene::scene::Subelement;
        let scene=self.scene().ok_or(Error::Missing("NativeBrep face scene"))?;
        hit.revalidate_in_scene(self.coordinator(),Some(scene),camera,viewport)?;
        if !hit.path.is_empty() || !matches!(hit.subset,Subelement::MeshFace(_) | Subelement::BrepFace(_)) || !pixels.is_finite() || pixels<=0. {
            return Err(Error::Unsupported("B-rep face pick"));
        }
        let displayed=camera.project(hit.world_point,viewport)?.ok_or(Error::Invalid("NativeBrep face hit outside view"))?;
        let pixel_distance=(displayed.screen[0]-cursor[0]).hypot(displayed.screen[1]-cursor[1]);
        let depth_error=(displayed.depth-hit.depth).abs();
        if !pixel_distance.is_finite() || pixel_distance>pixels+1e-6 || !depth_error.is_finite() ||
            depth_error>1e-8*displayed.depth.abs().max(1.) {return Err(Error::Stale("NativeBrep viewport hit"));}
        let reference=super::SelectedSubobject::from_hit(hit);
        let object=reference.validate(scene,self.coordinator.epoch())?;
        let entity=scene.entity_for(self.document(), object)?;
        let witness=match (&entity.geometry,&reference.subset) {
            (Geometry::NativeBrep(_),Subelement::MeshFace(_))=>None,
            (Geometry::Brep(_),Subelement::BrepFace(key))=>{
                let display=scene.root_brep_pick_display(&reference.source)?;
                let index=display.topology.faces.iter().position(|face|face==key).ok_or(Error::Stale("B-rep face"))?;
                let triangle=super::face_verification::choose_witness(display,index,None)?;
                let tolerance=self.document().effective_tolerance()?.absolute_mm.min(1e-6);
                Some(om9_kernel::editing::IsocurveFaceWitness {triangle,deflection:display.deflection_mm,tolerance})
            },
            _=>return Err(Error::Unsupported("B-rep root face pick")),
        };
        if object.display_transform()?!=entity.placement {return Err(Error::Unsupported("B-rep root face placement"));}
        let inverse=om9_core::top11::inverse_affine(entity.placement)?;
        let norm=|m:om9_core::geometry::Transform| (0..3).flat_map(|i|(0..3).map(move |j|m.0[i][j]*m.0[i][j])).sum::<f64>().sqrt();
        let inverse_bound=norm(inverse);let condition=norm(entity.placement)*inverse_bound;
        if !inverse_bound.is_finite() || !condition.is_finite() || condition>1e8 {return Err(Error::Unsupported("NativeBrep face placement"));}
        let world_ray=camera.visible_ray(cursor,viewport)?;
        let direction_squared=world_ray.direction.iter().map(|v|v*v).sum::<f64>();
        let along=(0..3).map(|i|(hit.world_point[i]-world_ray.origin[i])*world_ray.direction[i]).sum::<f64>()/direction_squared;
        if !along.is_finite() || along<0. {return Err(Error::Invalid("NativeBrep face ray reference"));}
        // Compare source intersections by position along the world-space ray.
        // This preserves their ordering after a non-uniform affine transform.
        let on_ray=std::array::from_fn(|i|world_ray.origin[i]+along*world_ray.direction[i]);
        let origin=inverse.point(world_ray.origin)?;let local_reference=inverse.point(on_ray)?;
        let direction=std::array::from_fn(|i|(0..3).map(|j|inverse.0[i][j]*world_ray.direction[j]).sum::<f64>());
        let world_per_pixel=match camera.projection {
            om9_scene::camera::Projection::Orthographic {vertical_span_mm}=>vertical_span_mm/viewport.size[1],
            om9_scene::camera::Projection::Perspective {vertical_fov_rad}=>2.*(vertical_fov_rad*0.5).tan()*hit.depth/viewport.size[1],
        };
        let world_tolerance=self.document().effective_tolerance()?.absolute_mm;
        let world_limit=(pixels*world_per_pixel).max(world_tolerance);
        let tolerance=(world_tolerance*inverse_bound).min(1e-6*inverse_bound);
        let max_distance=world_limit*inverse_bound;
        if !world_limit.is_finite() || world_limit<=0. || !tolerance.is_finite() || tolerance<=0. ||
            !max_distance.is_finite() || max_distance<=0. || !direction.iter().all(|v|v.is_finite()) {
            return Err(Error::Invalid("NativeBrep face ray"));
        }
        let publication=scene.publication_id();
        self.toggle_subobject_ref(reference.clone())?;
        self.native_isocurve_pick=if self.subselection().contains(&reference) {
            Some(NativeIsocurvePick {reference,revision:self.sub_revision(),publication,
                ray:om9_kernel::editing::IsocurveFaceRay {origin,direction,reference:local_reference,tolerance,max_distance,witness},
                world_reference:hit.world_point,world_limit,camera:camera.clone(),viewport,cursor,pixels})
        } else {None};
        Ok(())
    }

    #[cfg(feature="occt-brep")]
    pub(super) fn extract_picked_native_boundary(
        &self,
        source: &om9_core::brep::Brep,
        owner: &om9_core::model::Entity,
    ) -> om9_core::Result<om9_core::brep::Brep> {
        let pick = self
            .native_isocurve_pick
            .as_ref()
            .filter(|p| p.reference.root == owner.id)
            .ok_or(Error::Missing("Pick the boundary face in a viewport"))?;
        self.validate_native_isocurve_pick(pick)?;
        let (face, point) = om9_kernel::editing::extract_ray_face(
            source,
            pick.ray,
            om9_kernel::creation::DEFAULT_DEFLECTION_MM,
            self.coordinator.limits(),
        )
        .map_err(|e| Error::Io(format!("Boundary face: {e}")))?;
        let hit = owner.placement.point(point)?;
        let distance = (0..3)
            .map(|i| (hit[i] - pick.world_reference[i]).powi(2))
            .sum::<f64>()
            .sqrt();
        if !distance.is_finite() || distance > pick.world_limit {
            return Err(Error::Invalid(
                "Boundary face is outside the pick tolerance",
            ));
        }
        Ok(face)
    }

    #[cfg(feature="occt-brep")]
    fn validate_native_isocurve_pick(&self,pick:&NativeIsocurvePick) -> Result<(),Error> {
        let scene=self.scene().ok_or(Error::Missing("NativeBrep face scene"))?;
        if scene.publication_id()!=pick.publication || self.sub_revision()!=pick.revision ||
            self.subselection().len()!=1 || !self.subselection().contains(&pick.reference) ||
            (!self.selection.is_empty() && !self.selection.contains(&pick.reference.root)) {
            return Err(Error::Stale("NativeBrep face selection"));
        }
        let object=pick.reference.validate(scene,self.coordinator.epoch())?;
        let entity=scene.entity_for(self.document(), object)?;
        if !matches!((&entity.geometry,&pick.reference.subset),
            (Geometry::NativeBrep(_),om9_scene::scene::Subelement::MeshFace(_)) |
            (Geometry::Brep(_),om9_scene::scene::Subelement::BrepFace(_))) || object.display_transform()?!=entity.placement {
            return Err(Error::Stale("NativeBrep face source"));
        }
        Ok(())
    }

    pub(crate) fn isocurve_recovery_key(&self) -> Option<(u64, om9_core::identity::SourceStamp, SurfaceIso, u64)> {
        let pending = self.isocurve_pending.as_ref()?;
        #[cfg(feature="occt-brep")]
        if let Some((reference,_,revision))=&pending.face {
            if *revision!=self.sub_revision() || !self.subselection().contains(reference) {return None;}
            reference.validate(self.scene.as_ref()?,self.coordinator.epoch()).ok()?;
        }
        #[cfg(feature="occt-brep")]
        if let Some(pick)=&pending.native_face {self.validate_native_isocurve_pick(pick).ok()?;}
        Some((self.isocurve_recovery_run, pending.source, pending.iso, pending.value.to_bits()))
    }

    pub fn dispatch_isocurve(&mut self) -> Result<(), Error> {
        if self.selection.len() > 1 || self.subselection().len()>1 {
            return Err(Error::Invalid("ExtractIsocurve requires exactly one selected surface"));
        }
        let selected_face=self.subselection().first();
        if let Some(reference)=selected_face {
            if !reference.path.is_empty() || !matches!(reference.subset,om9_scene::scene::Subelement::BrepFace(_) | om9_scene::scene::Subelement::MeshFace(_)) {
                return Err(Error::Unsupported("ExtractIsocurve requires a root B-rep face"));
            }
            if !self.selection.is_empty() && !self.selection.contains(&reference.root) {return Err(Error::Invalid("surface selection"));}
        }
        let id=selected_face.map(|reference|reference.root).or_else(||self.selection.iter().next().copied())
            .ok_or(Error::Missing("surface selection"))?;
        let entity = self
            .document()
            .entities
            .get(&id)
            .ok_or(Error::Missing("selected surface"))?;
        if selected_face.is_some_and(|reference|matches!(reference.subset,om9_scene::scene::Subelement::MeshFace(_))) &&
            !matches!(entity.geometry,Geometry::NativeBrep(_)) {return Err(Error::Invalid("NativeBrep face selection"));}
        #[cfg(feature="occt-brep")]
        let native_pick=if selected_face.is_some_and(|reference|matches!(reference.subset,om9_scene::scene::Subelement::MeshFace(_))) {
            let pick=self.native_isocurve_pick.as_ref().ok_or(Error::Missing("NativeBrep viewport face pick"))?;
            self.validate_native_isocurve_pick(pick)?;
            Some(pick.clone())
        } else if let Some(pick)=self.native_isocurve_pick.as_ref().filter(|pick|selected_face==Some(&pick.reference)) {
            self.validate_native_isocurve_pick(pick)?;Some(pick.clone())
        } else {None};
        #[cfg(feature="occt-brep")]
        if self.background_isocurve_build_enabled && let Some(pick)=&native_pick {
            let reference=selected_face.ok_or(Error::Stale("isocurve ray selection"))?;
            if let Some(receipt)=&self.face_verification_receipt {
                let (bounds,local)=receipt.isocurve_ray_hit_for(self,reference)?;
                let world=entity.placement.point(local)?;
                let distance=(0..3).map(|i|(world[i]-pick.world_reference[i]).powi(2)).sum::<f64>().sqrt();
                let projected=pick.camera.project(world,pick.viewport)?.ok_or(Error::Invalid("NativeBrep source hit outside view"))?;
                let screen=(projected.screen[0]-pick.cursor[0]).hypot(projected.screen[1]-pick.cursor[1]);
                if !distance.is_finite() || distance>pick.world_limit || !screen.is_finite() || screen>pick.pixels {
                    return Err(Error::Invalid("NativeBrep display does not match source face"));
                }
                self.isocurve_pending=Some(IsoPending {source:self.document().stamp(id)?,surface:None,face:None,native_face:native_pick,
                    _surface_reservation:None,placement:entity.placement,iso:SurfaceIso::U,value:bounds[1][0]*0.5+bounds[1][1]*0.5});
                self.isocurve_recovery_run=self.isocurve_recovery_run.wrapping_add(1);return Ok(());
            }
            return self.request_isocurve_ray_verification(reference.clone(),pick.ray);
        }
        #[cfg(feature="occt-brep")]
        if self.background_isocurve_build_enabled && selected_face.is_none()
            && matches!(entity.geometry,Geometry::Brep(_) | Geometry::NativeBrep(_)) {
            let stamp=self.document().stamp(id)?;
            if let Some(receipt)=&self.face_verification_receipt {
                let bounds=receipt.isocurve_root_bounds_for(self,stamp)?;
                self.isocurve_pending=Some(IsoPending {source:stamp,surface:None,face:None,native_face:None,
                    _surface_reservation:None,placement:entity.placement,iso:SurfaceIso::U,value:bounds[1][0]*0.5+bounds[1][1]*0.5});
                self.isocurve_recovery_run=self.isocurve_recovery_run.wrapping_add(1);return Ok(());
            }
            return self.request_isocurve_root_verification(stamp);
        }
        #[cfg(feature="occt-brep")]
        if native_pick.is_none() && let Some(reference)=selected_face && self.face_verification_receipt.is_none()
            && matches!(entity.geometry,Geometry::Brep(_)) {
            let scene=self.scene.as_ref().ok_or(Error::Missing("face selection scene"))?;
            reference.validate(scene,self.coordinator.epoch())?;
            scene.root_brep_pick_display(&reference.source)?;
            let reference=reference.clone();
            return self.request_face_verification(std::slice::from_ref(&reference),ISOCURVE,true,ISOCURVE,None,None);
        }
        let surface = match &entity.geometry {
            Geometry::Surface(surface) => surface,
            #[cfg(feature="occt-brep")]
            Geometry::Brep(_) | Geometry::NativeBrep(_) => {
                use om9_core::budget::memory::MemoryClass;
                let input_len=match &entity.geometry {Geometry::Brep(brep)=>brep.source().len(),Geometry::NativeBrep(native)=>native.payload.len(),_=>unreachable!()};
                let bytes=(input_len as u64).checked_mul(4).and_then(|n|n.checked_add(4*1024*1024))
                    .ok_or(Error::Overflow("isocurve source"))?;
                let held=self.scene_cache_usage()?;
                let mut reservation=self.coordinator.reserve_external_memory_with_held(MemoryClass::Kernel,bytes,
                    om9_native_owner::physical_memory().map(|s|(s.total_bytes,s.available_bytes)),held)?;
                reservation.recheck_pending()?;
                let limits=*self.coordinator.limits();
                let converted=if let Geometry::NativeBrep(native)=&entity.geometry {
                    let tolerance=self.document().tolerance().ok_or(Error::Missing("document tolerance"))?.absolute_mm;
                    let mut admit=|live:u64,extra:u64| {
                        let needed=bytes.checked_add(live).and_then(|n|n.checked_add(extra)).ok_or(Error::Overflow("isocurve source"))?;
                        if needed>reservation.bytes() {let grow=needed-reservation.bytes();
                            self.coordinator.grow_external_memory_with_held(&mut reservation,grow,
                                om9_native_owner::physical_memory().map(|s|(s.total_bytes,s.available_bytes)),held)?;}
                        Ok(())
                    };
                    Some(om9_kernel::brep::from_3dm_source_typed_admitted(&native.payload,tolerance,&limits,&mut admit)?)
                } else {None};
                let source=match (&entity.geometry,&converted) {
                    (Geometry::Brep(brep),_)=>{
                        if self.face_verification_receipt.is_none() { brep.validate_source(&limits)?; }
                        brep.source()
                    },
                    (Geometry::NativeBrep(_),Some(source))=>source.as_str(),_=>unreachable!()};
                if let Some(source)=&converted {
                    let needed=(source.capacity() as u64).checked_add((source.len() as u64).checked_mul(4).ok_or(Error::Overflow("isocurve source"))?)
                        .and_then(|n|n.checked_add(4*1024*1024)).ok_or(Error::Overflow("isocurve source"))?;
                    if needed>reservation.bytes() {let grow=needed-reservation.bytes();self.coordinator.grow_external_memory_with_held(
                        &mut reservation,grow,om9_native_owner::physical_memory().map(|s|(s.total_bytes,s.available_bytes)),held)?;}
                }
                reservation.recheck_pending()?;
                let face=if let Some(reference)=selected_face.filter(|_|native_pick.is_none()) {
                    if !matches!(entity.geometry,Geometry::Brep(_)) {return Err(Error::Unsupported("ExtractIsocurve face selection requires B-rep topology"));}
                    let triangle=self.solid_face_display_triangle(reference)?;
                    let scene=self.scene.as_ref().ok_or(Error::Missing("face selection scene"))?;
                    let deflection=scene.root_brep_pick_display(&reference.source)?.deflection_mm;
                    let tolerance=self.document().tolerance().map_or(1e-6,|value|value.absolute_mm.min(1e-6));
                    Some((reference.clone(),om9_kernel::editing::IsocurveFaceWitness {triangle,deflection,tolerance},self.sub_revision()))
                } else {None};
                let bounds=if let Some(pick)=&native_pick {
                    let ray=pick.ray;
                    let (bounds,local)=om9_kernel::editing::ray_face_isocurve_hit_with_witness_from_source(source,ray,&limits)
                        .map_err(|e|Error::Io(format!("ExtractIsocurve: {e}")))?;
                    let world=entity.placement.point(local)?;
                    let distance=(0..3).map(|i|(world[i]-pick.world_reference[i]).powi(2)).sum::<f64>().sqrt();
                    let projected=pick.camera.project(world,pick.viewport)?.ok_or(Error::Invalid("NativeBrep source hit outside view"))?;
                    let screen=(projected.screen[0]-pick.cursor[0]).hypot(projected.screen[1]-pick.cursor[1]);
                    if !distance.is_finite() || distance>pick.world_limit || !screen.is_finite() || screen>pick.pixels {
                        return Err(Error::Invalid("NativeBrep display does not match source face"));
                    }
                    bounds
                } else if let Some((reference,_,_)) = &face {
                    self.face_verification_receipt.as_ref().ok_or(Error::Missing("isocurve bounds worker"))?
                        .isocurve_bounds_for(self, reference)?
                } else {om9_kernel::editing::face_isocurve_bounds_from_source(source,None,&limits)
                    .map_err(|e|Error::Io(format!("ExtractIsocurve: {e}")))?};
                self.isocurve_pending=Some(IsoPending {source:self.document().stamp(id)?,surface:None,face,native_face:native_pick,
                    _surface_reservation:Some(std::sync::Arc::new(reservation)),placement:entity.placement,
                    iso:SurfaceIso::U,value:bounds[1][0]*0.5+bounds[1][1]*0.5});
                self.isocurve_recovery_run=self.isocurve_recovery_run.wrapping_add(1);
                return Ok(());
            }
            #[cfg(not(feature="occt-brep"))]
            Geometry::Brep(_) => {
                return Err(Error::Unsupported("ExtractIsocurve does not yet support B-rep faces"));
            }
            #[cfg(not(feature="occt-brep"))]
            Geometry::NativeBrep(_) => {
                return Err(Error::Unsupported("ExtractIsocurve does not yet support B-rep faces"));
            }
            _ => return Err(Error::Invalid("surface selection")),
        };
        if let Some(extension) = entity.extensions.get(om9_core::model::TRIM_SURFACE_EXTENSION_SLOT) {
            surface.validate(self.coordinator.limits())?;
            if extension.payload.len() > om9_core::model::trim_surface_payload_limit(self.coordinator.limits()) {
                return Err(Error::Unsupported("ExtractIsocurve of this trimmed surface is not supported"));
            }
        }
        // The pending session owns a surface copy until commit or cancellation.
        // Admit it alongside the live scene before any of the vector copies.
        let usage=surface.validate(self.coordinator.limits())?;
        let bytes=usage.bytes.checked_mul(2).and_then(|n|n.checked_add(1024)).ok_or(Error::Overflow("isocurve pending surface"))?;
        let held=self.scene_cache_usage()?;
        let mut reservation=self.coordinator.reserve_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::Kernel,bytes.max(1),
            om9_native_owner::physical_memory().map(|s|(s.total_bytes,s.available_bytes)),held)?;
        reservation.recheck_pending()?;
        fn copy_values<T:Copy>(source:&[T])->Result<Vec<T>,Error> {
            let mut result=Vec::new();result.try_reserve_exact(source.len()).map_err(|_|Error::Budget("bytes"))?;
            result.extend_from_slice(source);Ok(result)
        }
        let snapshot=om9_core::geometry::NurbsSurface {degree:surface.degree,size:surface.size,
            poles:copy_values(&surface.poles)?,weights:copy_values(&surface.weights)?,
            knots:[copy_values(&surface.knots[0])?,copy_values(&surface.knots[1])?],
            domain:surface.domain,periodic:surface.periodic,reversed:surface.reversed};
        let live=snapshot.validate(self.coordinator.limits())?.bytes;
        reservation.set_allocated_bytes(live)?;
        // U isocurves vary along U, so their fixed parameter is in the V domain.
        let [v0, v1] = surface.domain[1];
        self.isocurve_pending = Some(IsoPending {
            source: self.document().stamp(id)?,
            surface: Some(std::sync::Arc::new(snapshot)),
            #[cfg(feature="occt-brep")]
            face: None,
            #[cfg(feature="occt-brep")]
            native_face: None,
            _surface_reservation: Some(std::sync::Arc::new(reservation)),
            placement: entity.placement,
            iso: SurfaceIso::U,
            value: v0 * 0.5 + v1 * 0.5,
        });
        self.isocurve_recovery_run = self.isocurve_recovery_run.wrapping_add(1);
        Ok(())
    }

    pub fn submit_isocurve_text(&mut self, text: &str) -> Option<Result<(), Error>> {
        #[cfg(feature="occt-brep")]
        if self.isocurve_build_worker_pending() {
            if matches!(text.trim().to_ascii_lowercase().as_str(),"cancel" | "esc") {
                self.cancel_isocurve_build_worker();
                self.isocurve_pending.take();
                return Some(Ok(()));
            }
            return Some(Err(Error::Busy));
        }
        let mut pending = self.isocurve_pending.take()?;
        let option = text.trim().to_ascii_lowercase();
        if option == "cancel" || option == "esc" {
            return Some(Ok(()));
        }
        if let Some(value) = option.strip_prefix("u=") {
            let Some(value) = value.parse::<f64>().ok().filter(|value| value.is_finite()) else {
                self.isocurve_pending = Some(pending);
                return Some(Err(Error::Invalid("isocurve parameter")));
            };
            pending.iso = SurfaceIso::U;
            pending.value = value;
            self.isocurve_pending = Some(pending);
            return Some(Ok(()));
        }
        if let Some(value) = option.strip_prefix("v=") {
            let Some(value) = value.parse::<f64>().ok().filter(|value| value.is_finite()) else {
                self.isocurve_pending = Some(pending);
                return Some(Err(Error::Invalid("isocurve parameter")));
            };
            pending.iso = SurfaceIso::V;
            pending.value = value;
            self.isocurve_pending = Some(pending);
            return Some(Ok(()));
        }
        if !text.trim().is_empty() {
            self.isocurve_pending = Some(pending);
            return Some(Err(Error::Invalid(
                "ExtractIsocurve accepts U=, V= or Enter",
            )));
        }
        let source = match self.document().resolve(&pending.source) {
            Ok(source) => source,
            Err(_) => {
            self.isocurve_pending = Some(pending);
            return Some(Err(Error::Stale("ExtractIsocurve source")));
            }
        };
        #[cfg(feature="occt-brep")]
        if pending.surface.is_none() {
            if self.background_isocurve_build_enabled && matches!(source.geometry,Geometry::Brep(_) | Geometry::NativeBrep(_)) {
                self.isocurve_pending=Some(pending);
                return Some(self.request_isocurve_build_worker());
            }
            let result=self.commit_brep_isocurve(&pending);
            if result.is_err() {self.isocurve_pending=Some(pending);}
            return Some(result);
        }
        if source.extensions.contains_key(om9_core::model::TRIM_SURFACE_EXTENSION_SLOT) {
            let result = self.commit_trimmed_surface_isocurve(&pending);
            if result.is_err() { self.isocurve_pending = Some(pending); }
            return Some(result);
        }
        let curve = match om9_core::surface_kernel::isocurve(
            pending.surface.as_deref().expect("standalone isocurve surface"),
            pending.iso,
            pending.value,
            self.coordinator.limits(),
        ) {
            Ok(curve) => curve,
            Err(error) => {
                self.isocurve_pending = Some(pending);
                return Some(Err(error));
            }
        };
        let result = self.commit_placed_geometries(
            ISOCURVE,
            "ExtractIsocurve",
            [("ExtractIsocurve", Geometry::Curve(Curve::Nurbs(curve)), pending.placement)].into_iter(),
        );
        if result.is_err() { self.isocurve_pending = Some(pending); }
        Some(result)
    }

    pub fn isocurve_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        if self.isocurve_pending.is_some() {
            vec![("U=", "U="), ("V=", "V="), ("Cancel", "Cancel")]
        } else {
            Vec::new()
        }
    }

    #[cfg(feature="occt-brep")]
    fn commit_brep_isocurve(&mut self,pending:&IsoPending)->Result<(),Error> {
        use om9_core::budget::memory::MemoryClass;
        let entity=self.document().resolve(&pending.source)?;
        if let Some((reference,_,revision))=&pending.face {
            if *revision!=self.sub_revision() || !self.subselection().contains(reference) {return Err(Error::Stale("isocurve face selection"));}
            reference.validate(self.scene.as_ref().ok_or(Error::Missing("face selection scene"))?,self.coordinator.epoch())?;
        }
        if let Some(pick)=&pending.native_face {self.validate_native_isocurve_pick(pick)?;}
        let limits=*self.coordinator.limits();let held=self.scene_cache_usage()?;
        let source_lease=if matches!(entity.geometry,Geometry::Brep(_)) && pending.native_face.is_none() {
            Some(self.coordinator.capture_brep_source(&pending.source)?)
        } else {None};
        let physical=||om9_native_owner::physical_memory().map(|s|(s.total_bytes,s.available_bytes));
        let mut reservation=self.coordinator.reserve_external_memory_with_held(MemoryClass::Kernel,1,physical(),held)?;
        if let Some(pending_reservation)=&pending._surface_reservation {pending_reservation.recheck_pending()?;}
        reservation.recheck_pending()?;
        let converted=if let Geometry::NativeBrep(native)=&entity.geometry {
            let tolerance=self.document().tolerance().ok_or(Error::Missing("document tolerance"))?.absolute_mm;
            let mut admit=|live:u64,extra:u64| {
                let needed=live.checked_add(extra).ok_or(Error::Overflow("isocurve source"))?;
                if needed>reservation.bytes() {let grow=needed-reservation.bytes();
                    self.coordinator.grow_external_memory_with_held(&mut reservation,grow,physical(),held)?;}
                Ok(())
            };
            Some(om9_kernel::brep::from_3dm_source_typed_admitted(&native.payload,tolerance,&limits,&mut admit)?)
        } else {None};
        let source=match (&entity.geometry,&converted) {
            (Geometry::Brep(shape),_)=>{if source_lease.is_none(){shape.validate_source(&limits)?;}shape.source()},
            (Geometry::NativeBrep(_),Some(source))=>source.as_str(),
            _=>return Err(Error::Stale("ExtractIsocurve source")),
        };
        let source_bytes=converted.as_ref().map_or(0,|source|source.capacity() as u64);
        reservation.recheck_pending()?;
        let mut refused=None;
        let mut admit=|live:u64,extra:u64| {
            let needed=source_bytes.checked_add(live).and_then(|n|n.checked_add(extra)).ok_or(Error::Overflow("isocurve outputs"))?;
            let result=if needed>reservation.bytes() {
                let grow=needed-reservation.bytes();
                self.coordinator.grow_external_memory_with_held(&mut reservation,grow,physical(),held)
            } else {Ok(())};
            if let Err(error)=&result {refused=Some(error.clone());}
            result
        };
        let curves=if let Some(lease)=&source_lease {
            om9_kernel::editing::face_isocurve_from_captured_source_cancellable(lease,pending.iso==SurfaceIso::U,pending.value,
                pending.face.as_ref().map(|(_,w,_)|*w),&limits,&om9_core::transaction::Cancellation::default(),&mut admit)?
        } else {
            let result=if let Some(pick)=&pending.native_face {
                om9_kernel::editing::ray_face_isocurve_from_source_admitted(source,pending.iso==SurfaceIso::U,pending.value,pick.ray,&limits,&mut admit)
            } else {om9_kernel::editing::face_isocurve_from_source_admitted(source,pending.iso==SurfaceIso::U,pending.value,
                pending.face.as_ref().map(|(_,w,_)|*w),&limits,&mut admit)};
            result.map_err(|e|refused.unwrap_or_else(||match e {
                om9_kernel::editing::EditingError::Budget(message) if message=="edge curve allocation"=>Error::Budget("bytes"),
                error=>Error::Io(format!("ExtractIsocurve: {error}")),
            }))?
        };
        drop(source_lease);
        self.commit_placed_geometries(ISOCURVE,"ExtractIsocurve",
            curves.into_iter().map(|curve|("ExtractIsocurve",Geometry::Curve(curve),pending.placement)))
    }

    fn commit_trimmed_surface_isocurve(&mut self, pending: &IsoPending) -> Result<(), Error> {
        use om9_core::budget::{memory::MemoryClass, Usage};
        let surface=pending.surface.as_deref().ok_or(Error::Stale("isocurve surface"))?;
        let source = self.coordinator.capture_tracked_document(self.document)?;
        let entity = source.resolve(&pending.source)?;
        let extension = entity.extensions.get(om9_core::model::TRIM_SURFACE_EXTENSION_SLOT)
            .ok_or(Error::Stale("isocurve trim"))?;
        if extension.payload.len() > om9_core::model::trim_surface_payload_limit(self.coordinator.limits()) {
            return Err(Error::Unsupported("ExtractIsocurve of this trimmed surface is not supported"));
        }
        let limits = *self.coordinator.limits();
        let held = self.scene_cache_usage()?;
        let physical = || om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
        let mut reservation = self.coordinator.reserve_external_memory_with_held(MemoryClass::Kernel, 1, physical(), held)?;
        let scratch = u64::try_from(extension.payload.len()).map_err(|_| Error::Overflow("isocurve trim metadata"))?
            .checked_mul(64).and_then(|n| n.checked_add(4096)).ok_or(Error::Overflow("isocurve trim metadata"))?;
        if scratch > reservation.bytes() {
            let extra = scratch - reservation.bytes();
            self.coordinator.grow_external_memory_with_held(&mut reservation, extra, physical(), held)?;
        }
        reservation.recheck_pending()?;
        let trim = om9_core::rhino_semantics::TrimmedSurface::from_extension(extension, &limits)?;
        if trim.underlying != *surface { return Err(Error::Stale("isocurve trim surface")); }
        om9_core::rhino_semantics::validate_trim_loop_topology_for_display(&trim)?;
        let affine=super::wireframe::is_affine_trim_plane(surface);
        if !affine {super::wireframe::validate_trim_closed_seams(surface,&trim,&limits)?;}
        let fixed = if pending.iso == SurfaceIso::U { 1 } else { 0 };
        let free = 1 - fixed;
        let fixed_domain = surface.domain[fixed];
        let free_domain = surface.domain[free];
        if !pending.value.is_finite() || !(fixed_domain[0]..=fixed_domain[1]).contains(&pending.value) {
            return Err(Error::Invalid("isocurve parameter outside surface domain"));
        }
        let count = trim.inners.iter().try_fold(trim.outer.len(), |n, loop_| n.checked_add(loop_.len()))
            .ok_or(Error::Overflow("isocurve trim edges"))?;
        let slots = Usage::array::<f64>(count)?.bytes.checked_add(
            Usage::array::<(&str, Geometry, om9_core::geometry::Transform)>(count)?.bytes.checked_mul(2)
                .ok_or(Error::Overflow("isocurve trim outputs"))?).ok_or(Error::Overflow("isocurve trim outputs"))?;
        let curved_scratch=if affine {0} else {
            let points=surface.size[free];let knots=surface.knots[free].len();
            let curve=Usage::array::<om9_core::geometry::Point3>(points)?.bytes
                .checked_add(Usage::array::<f64>(points)?.bytes).and_then(|n|n.checked_add(Usage::array::<f64>(knots).ok()?.bytes))
                .ok_or(Error::Overflow("isocurve trim curves"))?;
            let workspace=Usage::array::<f64>(surface.size[0].checked_add(surface.size[1]).ok_or(Error::Overflow("isocurve basis"))?)?.bytes
                .checked_mul(8).ok_or(Error::Overflow("isocurve basis"))?;
            curve.checked_mul(u64::try_from(count).map_err(|_|Error::Overflow("isocurve trim curves"))?)
                .and_then(|n|n.checked_mul(2)).and_then(|n|n.checked_add(workspace)).ok_or(Error::Overflow("isocurve trim curves"))?
        };
        let needed = scratch.checked_add(slots).and_then(|n|n.checked_add(curved_scratch)).ok_or(Error::Overflow("isocurve trim outputs"))?;
        if needed > reservation.bytes() {
            let extra = needed - reservation.bytes();
            self.coordinator.grow_external_memory_with_held(&mut reservation, extra, physical(), held)?;
        }
        reservation.recheck_pending()?;
        let fixed_epsilon = 1e-12 * fixed_domain[0].abs().max(fixed_domain[1].abs()).max(1.);
        let free_epsilon = 1e-12 * free_domain[0].abs().max(free_domain[1].abs()).max(1.);
        let origin = pending.placement.point(surface.poles[0])?;
        // Include the magnitudes before placement as well as after it: a large
        // translation can cancel large local coordinates in the final result.
        let local_scale = surface.poles.iter().flatten().map(|v| v.abs()).fold(1_f64, f64::max);
        let coordinate_scale = pending.placement.0[..3].iter().map(|row| {
            row[3].abs() + (0..3).map(|i| row[i].abs() * local_scale).sum::<f64>()
        }).chain(origin.iter().map(|v|v.abs())).fold(1_f64, f64::max);
        let (world_per_uv,roundoff_factor)=if affine {
            let u_end=pending.placement.point(surface.poles[2])?;let v_end=pending.placement.point(surface.poles[1])?;
            let extent=(0..3).map(|i|(u_end[i]-origin[i]).abs()+(v_end[i]-origin[i]).abs()).fold(0_f64,f64::max);
            (extent/(surface.knots[free][2]-surface.knots[free][0]),1.)
        } else {
            let mut low=[f64::INFINITY;3];let mut high=[f64::NEG_INFINITY;3];
            for pole in &surface.poles {let p=pending.placement.point(*pole)?;for i in 0..3 {low[i]=low[i].min(p[i]);high[i]=high[i].max(p[i]);}}
            let extent=(0..3).map(|i|high[i]-low[i]).fold(0_f64,f64::max);
            let gap=surface.knots[free].windows(2).filter_map(|w| {let d=w[1]-w[0];(d>0.).then_some(d)}).fold(f64::INFINITY,f64::min);
            let min_weight=surface.weights.iter().copied().fold(f64::INFINITY,f64::min);
            let max_weight=surface.weights.iter().copied().fold(0_f64,f64::max);
            let ratio=max_weight/min_weight;
            (2.*surface.degree[free] as f64*ratio*extent/gap,ratio*surface.poles.len() as f64)
        };
        if !fixed_epsilon.is_finite() || !free_epsilon.is_finite() || !world_per_uv.is_finite() {
            return Err(Error::Unsupported("isocurve parameter scale is numerically uncertain"));
        }
        let tolerance = source.tolerance().map_or(1e-6, |value| value.absolute_mm.min(1e-6));
        let world_roundoff = 256. * f64::EPSILON * coordinate_scale * roundoff_factor;
        if !world_roundoff.is_finite() || world_roundoff>=tolerance {
            return Err(Error::Unsupported("isocurve placement is numerically uncertain"));
        }
        let mut max_uv_roundoff = 0_f64;
        let mut crossings = Vec::new();
        crossings.try_reserve_exact(count).map_err(|_| Error::Budget("bytes"))?;
        for loop_ in std::iter::once(&trim.outer).chain(&trim.inners) {
            for i in 0..loop_.len() {
                let a = loop_[i]; let b = loop_[(i + 1) % loop_.len()];
                let da = a[fixed] - pending.value;
                let db = b[fixed] - pending.value;
                if (da != 0. && da.abs() <= fixed_epsilon) || (db != 0. && db.abs() <= fixed_epsilon) {
                    return Err(Error::Unsupported("isocurve is too close to a trim vertex"));
                }
                if da == 0. {
                    let previous = loop_[(i + loop_.len() - 1) % loop_.len()][fixed] - pending.value;
                    if previous.abs() <= fixed_epsilon || db.abs() <= fixed_epsilon {
                        return Err(Error::Unsupported("isocurve coincides with a trim edge"));
                    }
                    if (previous < 0.) == (db < 0.) {
                        return Err(Error::Unsupported("isocurve touches a trim vertex tangentially"));
                    }
                    if !world_roundoff.is_finite() || world_roundoff >= tolerance {
                        return Err(Error::Unsupported("isocurve trim vertex is numerically uncertain"));
                    }
                    crossings.push(a[free]);
                    continue;
                }
                // The next iteration handles an exact crossing at b once.
                if db == 0. { continue; }
                if (a[fixed] < pending.value) != (b[fixed] < pending.value) {
                    let denominator = b[fixed] - a[fixed];
                    let parameter_roundoff = 256. * f64::EPSILON
                        * (1. + a[fixed].abs() + b[fixed].abs() + pending.value.abs()) / denominator.abs();
                    let uv_roundoff = 256. * f64::EPSILON * (1. + a[free].abs() + b[free].abs())
                        + parameter_roundoff * (b[free] - a[free]).abs();
                    if !uv_roundoff.is_finite() || uv_roundoff * world_per_uv + world_roundoff >= tolerance {
                        return Err(Error::Unsupported("isocurve trim crossing is numerically uncertain"));
                    }
                    max_uv_roundoff = max_uv_roundoff.max(uv_roundoff);
                    let t = (pending.value - a[fixed]) / denominator;
                    let value = a[free] + t * (b[free] - a[free]);
                    if !value.is_finite() { return Err(Error::Invalid("isocurve trim crossing")); }
                    crossings.push(value);
                }
            }
        }
        crossings.sort_by(f64::total_cmp);
        if crossings.windows(2).any(|pair| pair[1] - pair[0] <= free_epsilon + 4. * max_uv_roundoff) {
            return Err(Error::Unsupported("ambiguous isocurve trim crossings"));
        }
        let inside = |p: [f64; 2], polygon: &[[f64; 2]]| -> Result<bool, Error> {
            let mut result = false;
            let mut j = polygon.len() - 1;
            for i in 0..polygon.len() {
                let (a, b) = (polygon[i], polygon[j]);
                if (a[1] > p[1]) != (b[1] > p[1]) {
                    let crossing = a[0] + ((p[1] - a[1]) / (b[1] - a[1])) * (b[0] - a[0]);
                    if !crossing.is_finite() { return Err(Error::Invalid("isocurve trim classification")); }
                    if p[0] < crossing { result = !result; }
                }
                j = i;
            }
            Ok(result)
        };
        let mut items = Vec::new();
        items.try_reserve_exact(crossings.len()).map_err(|_| Error::Budget("bytes"))?;
        for pair in crossings.windows(2) {
            let middle = pair[0] * 0.5 + pair[1] * 0.5;
            if middle <= pair[0] || middle >= pair[1] { return Err(Error::Invalid("isocurve trim precision")); }
            let uv = if fixed == 1 { [middle, pending.value] } else { [pending.value, middle] };
            if !inside(uv, &trim.outer)? { continue; }
            let mut in_hole = false;
            for hole in &trim.inners { if inside(uv, hole)? { in_hole = true; break; } }
            if in_hole { continue; }
            let point = |v: f64| {
                let uv = if fixed == 1 { [v, pending.value] } else { [pending.value, v] };
                om9_core::surface_kernel::evaluate(surface, uv[0], uv[1], &limits)
            };
            let start = point(pair[0])?; let end = point(pair[1])?;
            let curve=if affine {
                if start==end {return Err(Error::Invalid("isocurve trim has zero length"));}
                Curve::Line {start,end,domain:[0.,1.]}
            } else {
                if start==end && point(middle)?==start {return Err(Error::Unsupported("isocurve trim may be degenerate"));}
                let mut curve=om9_core::surface_kernel::isocurve(surface,pending.iso,pending.value,&limits)?;
                curve.domain=[pair[0],pair[1]];curve.validate(&limits)?;
                Curve::Nurbs(curve)
            };
            items.push(("ExtractIsocurve",Geometry::Curve(curve),pending.placement));
            if items.len() as u64 > limits.max_outputs { return Err(Error::Budget("isocurve outputs")); }
        }
        if items.is_empty() { return Err(Error::Invalid("isocurve does not cross trimmed surface")); }
        drop(trim); drop(crossings);
        self.commit_placed_geometries(ISOCURVE, "ExtractIsocurve", items.into_iter())
    }
}
