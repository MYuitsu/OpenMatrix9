// SPDX-License-Identifier: LGPL-2.1-or-later
use super::*;
use om9_core::transaction::UndoPolicy;

impl MatrixApp {
    #[cfg(feature="occt-brep")]
    pub(super) fn isocurve_build_recovery_waiting(&self)->bool {
        self.memory_recovery_open && self.memory_recovery_running
            && matches!(self.memory_recovery_target,Some(MemoryRecoveryTarget::Isocurve {waiting_worker:true,..}))
    }
    #[cfg(feature="occt-brep")]
    pub(super) fn isocurve_build_recovery_context_valid(&self)->bool {
        match &self.memory_recovery_target {
            Some(MemoryRecoveryTarget::Isocurve {authority,epoch,key,root_revision,sub_revision,draft,waiting_worker:true,..})=>
                self.controller.coordinator().authority()==*authority && self.controller.coordinator().epoch()==*epoch
                && self.controller.isocurve_recovery_key()==Some(*key)
                && self.controller.selection_state().revision==*root_revision && self.controller.sub_revision()==*sub_revision
                && self.transcript.draft()==draft,
            _=>false,
        }
    }
    #[cfg(feature="occt-brep")]
    pub(super) fn stop_isocurve_build_recovery(&mut self,error:Error) {
        self.controller.cancel_isocurve_build_worker();
        let message=format!("ExtractIsocurve did not complete after removing {} Undo step(s): {error}",self.memory_recovery_removed);
        self.transcript.append_history(&message);self.status.error(&message);
        self.memory_recovery_open=false;self.memory_recovery_running=false;self.memory_recovery_defer=false;self.memory_recovery_target=None;self.focus_cmd=true;
    }
    #[cfg(feature="occt-brep")]
    pub(super) fn finish_isocurve_build_recovery(&mut self,result:Result<()>) {
        match result {
            Ok(())=>{
                let message=format!("ExtractIsocurve completed after removing {} oldest Undo step(s).",self.memory_recovery_removed);
                self.transcript.append_history(&message);self.status.info(&message);
                self.memory_recovery_open=false;self.memory_recovery_running=false;self.memory_recovery_defer=false;self.memory_recovery_target=None;self.focus_cmd=true;
            }
            Err(error) if Self::is_memory_refusal(&error) && self.isocurve_build_recovery_context_valid()=>{
                let retried=matches!(self.memory_recovery_target,Some(MemoryRecoveryTarget::Isocurve {retried_after_release:true,..}));
                if !retried {
                    if let Some(MemoryRecoveryTarget::Isocurve {waiting_worker,retried_after_release,..})=&mut self.memory_recovery_target {
                        *waiting_worker=false;*retried_after_release=true;
                    }
                    self.memory_recovery_defer=true;return;
                }
                match self.controller.discard_oldest_undo() {
                    Ok(true)=>{self.memory_recovery_removed+=1;
                        if let Some(MemoryRecoveryTarget::Isocurve {waiting_worker,retried_after_release,..})=&mut self.memory_recovery_target {
                            *waiting_worker=false;*retried_after_release=false;
                        }
                        self.memory_recovery_defer=true;
                    }
                    Ok(false)=>self.stop_isocurve_build_recovery(error),
                    Err(reason)=>self.stop_isocurve_build_recovery(reason),
                }
            }
            Err(error)=>self.stop_isocurve_build_recovery(error),
        }
    }
    #[cfg(feature="occt-brep")]
    pub(super) fn wire_recovery_waiting(&self)->bool {
        self.memory_recovery_open && self.memory_recovery_running
            && matches!(self.memory_recovery_target,Some(MemoryRecoveryTarget::Wire {waiting_worker:true,..}))
    }
    #[cfg(feature="occt-brep")]
    pub(super) fn wire_recovery_context_valid(&self)->bool {
        match &self.memory_recovery_target {
            Some(MemoryRecoveryTarget::Wire {authority,epoch,selected,root_revision,draft,waiting_worker:true,..})=>
                self.controller.coordinator().authority()==*authority && self.controller.coordinator().epoch()==*epoch
                && self.controller.selected()==selected && self.controller.selection_state().revision==*root_revision
                && self.transcript.draft()==draft,
            _=>false,
        }
    }
    #[cfg(feature="occt-brep")]
    pub(super) fn stop_wire_worker_recovery(&mut self,error:Error) {
        if self.controller.brep_wireframe_worker_pending() {self.controller.cancel();}
        let message=format!("ExtractWireframe did not complete after removing {} Undo step(s): {error}",self.memory_recovery_removed);
        self.transcript.append_history(&message);self.status.error(&message);
        self.memory_recovery_open=false;self.memory_recovery_running=false;self.memory_recovery_defer=false;self.memory_recovery_target=None;self.focus_cmd=true;
    }
    #[cfg(feature="occt-brep")]
    pub(super) fn finish_wire_worker_recovery(&mut self,result:Result<()>) {
        match result {
            Ok(())=>{
                let message=format!("ExtractWireframe completed after removing {} oldest Undo step(s).",self.memory_recovery_removed);
                self.transcript.append_history(&message);self.status.info(&message);
                self.memory_recovery_open=false;self.memory_recovery_running=false;self.memory_recovery_defer=false;self.memory_recovery_target=None;self.focus_cmd=true;
            }
            Err(error) if Self::is_memory_refusal(&error) && self.wire_recovery_context_valid()=>{
                let retried=matches!(self.memory_recovery_target,Some(MemoryRecoveryTarget::Wire {retried_after_release:true,..}));
                if !retried {
                    if let Some(MemoryRecoveryTarget::Wire {waiting_worker,retried_after_release,..})=&mut self.memory_recovery_target {
                        *waiting_worker=false;*retried_after_release=true;
                    }
                    self.memory_recovery_defer=true;return;
                }
                match self.controller.discard_oldest_undo() {
                    Ok(true)=>{self.memory_recovery_removed+=1;
                        if let Some(MemoryRecoveryTarget::Wire {waiting_worker,retried_after_release,..})=&mut self.memory_recovery_target {
                            *waiting_worker=false;*retried_after_release=false;
                        }
                        self.memory_recovery_defer=true;
                    },
                    Ok(false)=>self.stop_wire_worker_recovery(error),
                    Err(reason)=>self.stop_wire_worker_recovery(reason),
                }
            }
            Err(error)=>self.stop_wire_worker_recovery(error),
        }
    }
    #[cfg(feature="occt-brep")]
    pub(super) fn isocurve_recovery_waiting(&self)->bool {
        self.memory_recovery_open && self.memory_recovery_running
            && matches!(self.memory_recovery_target,Some(MemoryRecoveryTarget::IsocurveStart {waiting_face:true,..}))
    }

    #[cfg(feature="occt-brep")]
    pub(super) fn isocurve_recovery_context_valid(&self)->bool {
        match &self.memory_recovery_target {
            Some(MemoryRecoveryTarget::IsocurveStart {authority,epoch,selected,root_revision,sub_revision,draft,waiting_face:true,..})=>
                self.controller.coordinator().authority()==*authority && self.controller.coordinator().epoch()==*epoch
                && self.controller.selected()==selected && self.controller.selection_state().revision==*root_revision && self.controller.sub_revision()==*sub_revision && self.transcript.draft()==draft,
            _=>false,
        }
    }

    #[cfg(feature="occt-brep")]
    pub(super) fn finish_isocurve_face_recovery(&mut self,result:Result<()>) {
        match result {
            Ok(())=>{
                let clear=matches!(&self.memory_recovery_target,Some(MemoryRecoveryTarget::IsocurveStart {draft,..})
                    if self.transcript.draft()==draft && (draft.trim().eq_ignore_ascii_case("ExtractIsocurve") || draft.trim().eq_ignore_ascii_case("OM9-CURVE-019")));
                let message=format!("ExtractIsocurve started after removing {} oldest Undo step(s).",self.memory_recovery_removed);
                self.transcript.append_history(&message);self.status.info(&message);self.append_running_prompt();
                if clear {self.transcript.set_draft("");}
                self.memory_recovery_open=false;self.memory_recovery_running=false;self.memory_recovery_defer=false;self.memory_recovery_target=None;self.focus_cmd=true;
            }
            Err(error) if Self::is_memory_refusal(&error) && self.isocurve_recovery_context_valid()=>{
                let retried=matches!(self.memory_recovery_target,Some(MemoryRecoveryTarget::IsocurveStart {retried_after_release:true,..}));
                if !retried {
                    if let Some(MemoryRecoveryTarget::IsocurveStart {waiting_face,retried_after_release,..})=&mut self.memory_recovery_target {
                        *waiting_face=false;*retried_after_release=true;
                    }
                    self.memory_recovery_defer=true;return;
                }
                match self.controller.discard_oldest_undo() {
                    Ok(true)=>{self.memory_recovery_removed+=1;
                        if let Some(MemoryRecoveryTarget::IsocurveStart {waiting_face,retried_after_release,..})=&mut self.memory_recovery_target {
                            *waiting_face=false;*retried_after_release=false;
                        }
                        self.memory_recovery_defer=true;
                    }
                    Ok(false)=>self.stop_isocurve_face_recovery(error),
                    Err(reason)=>self.stop_isocurve_face_recovery(reason),
                }
            }
            Err(error)=>self.stop_isocurve_face_recovery(error),
        }
    }

    #[cfg(feature="occt-brep")]
    pub(super) fn stop_isocurve_face_recovery(&mut self,error:Error) {
        if self.controller.isocurve_face_verification_pending() {self.controller.cancel();}
        let message=format!("ExtractIsocurve did not start after removing {} Undo step(s): {error}",self.memory_recovery_removed);
        self.transcript.append_history(&message);self.status.error(&message);
        self.memory_recovery_open=false;self.memory_recovery_running=false;self.memory_recovery_defer=false;self.memory_recovery_target=None;self.focus_cmd=true;
    }
    #[cfg(feature="occt-brep")]
    pub(super) fn dupedge_recovery_waiting(&self)->bool {
        self.memory_recovery_open && self.memory_recovery_running
            && matches!(self.memory_recovery_target,Some(MemoryRecoveryTarget::DupEdge {waiting_view:true,..}))
    }

    #[cfg(feature="occt-brep")]
    pub(super) fn dupedge_recovery_wait_context_valid(&self)->bool {
        match &self.memory_recovery_target {
            Some(MemoryRecoveryTarget::DupEdge {authority,epoch,key,draft,view_root,waiting_view:true,..})=>
                self.controller.coordinator().authority()==*authority && self.controller.coordinator().epoch()==*epoch
                && (if view_root.is_some() {self.controller.dupedge_source_view_context_key()} else {self.controller.dupedge_recovery_key()})==Some(*key)
                && self.transcript.draft()==draft,
            _=>false,
        }
    }

    #[cfg(feature="occt-brep")]
    pub(super) fn finish_dupedge_view_recovery(&mut self,result:Result<()>) {
        match result {
            Ok(())=>{
                let clear=matches!(&self.memory_recovery_target,Some(MemoryRecoveryTarget::DupEdge {draft,..})
                    if self.transcript.draft()==draft && (draft.trim().eq_ignore_ascii_case("DupEdge") || draft.trim().eq_ignore_ascii_case("OM9-CURVE-040")));
                let message=format!("DupEdge source view ready; select the edge again (after removing {} oldest Undo step(s))",self.memory_recovery_removed);
                let prior=self.status.message().strip_prefix("DupEdge source view unavailable: ").unwrap_or(self.status.message());
                if matches!(prior,"budget exceeded: bytes" | "budget exceeded: managed memory" | "budget exceeded: managed RAM"
                    | "budget exceeded: available physical RAM" | "budget exceeded: available system commit" | "budget exceeded: external memory headroom") {self.status.acknowledge();}
                self.transcript.append_history(&message);self.status.info(&message);
                if clear {self.transcript.set_draft("");}
                self.memory_recovery_open=false;self.memory_recovery_running=false;self.memory_recovery_defer=false;self.memory_recovery_target=None;self.focus_cmd=true;
            }
            Err(error) if Self::is_memory_refusal(&error) && self.dupedge_recovery_wait_context_valid()=>{
                let retried=matches!(self.memory_recovery_target,Some(MemoryRecoveryTarget::DupEdge {retried_after_release:true,..}));
                if !retried {
                    if let Some(MemoryRecoveryTarget::DupEdge {waiting_view,retried_after_release,..})=&mut self.memory_recovery_target {
                        *waiting_view=false;*retried_after_release=true;
                    }
                    self.memory_recovery_defer=true;return;
                }
                match self.controller.discard_oldest_undo() {
                    Ok(true)=>{
                        self.memory_recovery_removed+=1;
                        if let Some(MemoryRecoveryTarget::DupEdge {waiting_view,retried_after_release,..})=&mut self.memory_recovery_target {
                            *waiting_view=false;*retried_after_release=false;
                        }
                        self.memory_recovery_defer=true;
                    }
                    Ok(false)=>self.stop_dupedge_view_recovery(error),
                    Err(reason)=>self.stop_dupedge_view_recovery(reason),
                }
            }
            Err(error)=>self.stop_dupedge_view_recovery(error),
        }
    }

    #[cfg(feature="occt-brep")]
    pub(super) fn stop_dupedge_view_recovery(&mut self,error:Error) {
        self.controller.cancel_dupedge_source_view();
        let message=format!("DupEdge source view did not complete after removing {} Undo step(s): {error}",self.memory_recovery_removed);
        self.transcript.append_history(&message);self.status.error(&message);
        self.memory_recovery_open=false;self.memory_recovery_running=false;self.memory_recovery_defer=false;self.memory_recovery_target=None;self.focus_cmd=true;
    }
    pub(super) fn offer_dupedge_memory_recovery(&mut self,key:(uuid::Uuid,u64,u64,uuid::Uuid),draft:String,input:&'static str) {
        self.memory_recovery_running=false;self.memory_recovery_defer=false;self.memory_recovery_removed=0;
        self.memory_recovery_target=Some(MemoryRecoveryTarget::DupEdge {authority:self.controller.coordinator().authority(),
            epoch:self.controller.coordinator().epoch(),key,draft,input,view_root:None,waiting_view:false,retried_after_release:false});self.memory_recovery_open=true;
    }
    #[cfg(feature="occt-brep")]
    pub(super) fn offer_dupedge_view_memory_recovery(&mut self,root:om9_core::identity::ObjectId,key:(uuid::Uuid,u64,u64,uuid::Uuid),draft:String) {
        self.memory_recovery_running=false;self.memory_recovery_defer=false;self.memory_recovery_removed=0;
        self.memory_recovery_target=Some(MemoryRecoveryTarget::DupEdge {authority:self.controller.coordinator().authority(),
            epoch:self.controller.coordinator().epoch(),key,draft,input:"",view_root:Some(root),waiting_view:false,retried_after_release:false});
        self.memory_recovery_open=true;
    }
    #[cfg(feature="occt-brep")]
    pub(super) fn offer_selected_native_dupedge_view_recovery(&mut self,error:&Error,draft:String) {
        if !Self::is_memory_refusal(error) || self.controller.selected().len()!=1 {return;}
        let root=*self.controller.selected().iter().next().expect("one selection");
        if self.controller.document().entities.get(&root).is_some_and(|entity|matches!(entity.geometry,om9_core::geometry::Geometry::NativeBrep(_)))
            && let Some(key)=self.controller.dupedge_source_view_context_key() {
            self.offer_dupedge_view_memory_recovery(root,key,draft);
        }
    }
    pub(super) fn offer_direct_solid_memory_recovery(&mut self, command: &'static str, input: String) {
        self.memory_recovery_running = false;
        self.memory_recovery_removed = 0;
        self.memory_recovery_target = Some(MemoryRecoveryTarget::DirectSolid { authority: self.controller.coordinator().authority(), epoch: self.controller.coordinator().epoch(), input, command });
        self.memory_recovery_open = true;
    }
    pub(super) fn is_memory_refusal(error: &Error) -> bool {
        matches!(error, Error::Budget("bytes" | "managed memory" | "managed RAM" | "available physical RAM" | "available system commit" | "external memory headroom"))
    }

    pub(super) fn offer_wire_memory_recovery(&mut self, command: &'static str) {
        self.memory_recovery_running = false;
        self.memory_recovery_removed = 0;
        self.memory_recovery_target = Some(MemoryRecoveryTarget::Wire { authority: self.controller.coordinator().authority(), epoch: self.controller.coordinator().epoch(), selected: self.controller.selected().clone(), root_revision:self.controller.selection_state().revision, draft: self.transcript.draft().to_owned(), command, waiting_worker:false, retried_after_release:false });
        self.memory_recovery_open = true;
    }

    pub(super) fn offer_isocurve_memory_recovery(&mut self, key: (u64, om9_core::identity::SourceStamp, om9_core::surface_kernel::SurfaceIso, u64)) {
        self.memory_recovery_running = false;
        self.memory_recovery_removed = 0;
        self.memory_recovery_target = Some(MemoryRecoveryTarget::Isocurve { authority: self.controller.coordinator().authority(), epoch: self.controller.coordinator().epoch(), key,
            root_revision:self.controller.selection_state().revision,sub_revision:self.controller.sub_revision(),draft:self.transcript.draft().to_owned(),waiting_worker:false,retried_after_release:false });
        self.memory_recovery_open = true;
    }

    pub(super) fn offer_isocurve_start_memory_recovery(&mut self) {
        self.memory_recovery_running = false;
        self.memory_recovery_removed = 0;
        self.memory_recovery_target = Some(MemoryRecoveryTarget::IsocurveStart {
            authority: self.controller.coordinator().authority(), epoch: self.controller.coordinator().epoch(),
            selected: self.controller.selected().clone(), root_revision:self.controller.selection_state().revision, sub_revision:self.controller.sub_revision(), draft: self.transcript.draft().to_owned(), waiting_face:false,retried_after_release:false,
        });
        self.memory_recovery_open = true;
    }

    pub(super) fn offer_sphere_memory_recovery(&mut self, key: crate::session::SphereRecoveryKey, input: String) {
        self.memory_recovery_running = false;
        self.memory_recovery_removed = 0;
        self.memory_recovery_target = Some(MemoryRecoveryTarget::Sphere { authority: self.controller.coordinator().authority(), epoch: self.controller.coordinator().epoch(), key, input });
        self.memory_recovery_open = true;
    }

    pub(super) fn offer_sphere_selection_memory_recovery(&mut self, key: crate::session::SphereRecoveryKey, input: String) {
        self.memory_recovery_running = false;
        self.memory_recovery_removed = 0;
        self.memory_recovery_target = Some(MemoryRecoveryTarget::SphereSelection {
            authority: self.controller.coordinator().authority(), epoch: self.controller.coordinator().epoch(),
            key, root_revision: self.controller.selection_state().revision, input,
        });
        self.memory_recovery_open = true;
    }

    pub(super) fn offer_box_memory_recovery(&mut self, key: crate::session::BoxRecoveryKey, input: String) {
        self.memory_recovery_running = false;
        self.memory_recovery_removed = 0;
        self.memory_recovery_target = Some(MemoryRecoveryTarget::Box { authority: self.controller.coordinator().authority(), epoch: self.controller.coordinator().epoch(), key, input });
        self.memory_recovery_open = true;
    }

    pub(super) fn validate_snap_anchor(&self, anchor: &Option<om9_scene::osnap::SnapCandidate>) -> Result<()> {
        if let Some(candidate) = anchor {
            let scene = self.controller.scene().ok_or(Error::Missing("snap scene"))?;
            if candidate.root.is_some() && candidate.source.is_none() {
                return Err(Error::Stale("snap source binding"));
            }
            if candidate.root.is_some() {
                let (view_id, camera, viewport) = self.snap_view.as_ref().ok_or(Error::Stale("snap view"))?;
                let view = self.controller.workspace.views.get(view_id).ok_or(Error::Stale("snap view"))?;
                let rect = self.view_rects.get(view_id).ok_or(Error::Stale("snap viewport"))?;
                let current = Viewport { origin: [rect.min.x as f64, rect.min.y as f64], size: [rect.width() as f64, rect.height() as f64] };
                if &view.camera != camera || current != *viewport || self.hover_view != Some(*view_id) {
                    return Err(Error::Stale("snap camera or viewport"));
                }
            }
            if let Some(source) = &candidate.source { source.validate(scene)?; }
            if let Some(source) = &candidate.second_source { source.validate(scene)?; }
        }
        Ok(())
    }

    pub(super) fn accept_viewport_command_point(&mut self, point: [f64; 3], anchor: &Option<om9_scene::osnap::SnapCandidate>) -> Result<()> {
        self.validate_snap_anchor(anchor)?;
        let key = self.controller.sphere_point_recovery_key();
        let box_key = self.controller.box_recovery_key();
        let plane = self.controller.input_plane();
        let result = self.controller.accept_world_point(point);
        if let Some(key) = key && self.controller.sphere_point_recovery_key() == Some(key)
            && matches!(&result, Err(error) if Self::is_memory_refusal(error)) {
            self.memory_recovery_running = false;
            self.memory_recovery_removed = 0;
            self.memory_recovery_target = Some(MemoryRecoveryTarget::SpherePoint {
                authority: self.controller.coordinator().authority(), epoch: self.controller.coordinator().epoch(),
                key, point, anchor: anchor.clone(), plane, one_shot: self.snap_state.settings.one_shot, draft: self.transcript.draft().to_owned(),
            });
            self.memory_recovery_open = true;
        }
        if let Some(key) = box_key && self.controller.box_recovery_key() == Some(key)
            && matches!(&result, Err(error) if Self::is_memory_refusal(error)) {
            self.memory_recovery_running = false;
            self.memory_recovery_removed = 0;
            self.memory_recovery_target = Some(MemoryRecoveryTarget::BoxPoint {
                authority: self.controller.coordinator().authority(), epoch: self.controller.coordinator().epoch(),
                key, point, anchor: anchor.clone(), plane, one_shot: self.snap_state.settings.one_shot, draft: self.transcript.draft().to_owned(),
            });
            self.memory_recovery_open = true;
        }
        result
    }

    /// Perform at most one retry and one explicit history removal per UI frame.
    fn retry_undo_recovery_step(&mut self) -> Result<bool> {
        #[allow(unused_mut)] // The OCCT source-view retry updates its captured session.
        let mut target = self.memory_recovery_target.take().ok_or(Error::Stale("memory recovery"))?;
        match &target {
            MemoryRecoveryTarget::DirectSolid { authority, epoch, input, .. } => {
                if self.controller.coordinator().authority() != *authority || self.controller.coordinator().epoch() != *epoch || self.controller.is_running() || self.transcript.draft() != input.as_str() {
                    return Err(Error::Stale("memory recovery solid input"));
                }
            }
            MemoryRecoveryTarget::Wire { authority, epoch, selected, root_revision, draft, .. } => {
                if self.controller.coordinator().authority() != *authority || self.controller.coordinator().epoch() != *epoch || self.controller.selected() != selected || self.controller.selection_state().revision != *root_revision || self.controller.is_running() || self.transcript.draft() != draft {
                    return Err(Error::Stale("memory recovery selection"));
                }
            }
            MemoryRecoveryTarget::Isocurve { authority, epoch, key, root_revision, sub_revision, draft, waiting_worker,.. } => {
                if self.controller.coordinator().authority() != *authority || self.controller.coordinator().epoch() != *epoch || self.controller.isocurve_recovery_key() != Some(*key)
                    || self.controller.selection_state().revision!=*root_revision || self.controller.sub_revision()!=*sub_revision || *waiting_worker || self.transcript.draft() != draft {
                    return Err(Error::Stale("memory recovery Isocurve input"));
                }
            }
            MemoryRecoveryTarget::DupEdge {authority,epoch,key,draft,input,view_root,waiting_view,..} => {
                if self.controller.coordinator().authority()!=*authority || self.controller.coordinator().epoch()!=*epoch
                    || (if view_root.is_some() {
                        #[cfg(feature="occt-brep")] {self.controller.dupedge_source_view_context_key()}
                        #[cfg(not(feature="occt-brep"))] {None}
                    } else {self.controller.dupedge_recovery_key()})!=Some(*key) || self.transcript.draft()!=draft
                    || *waiting_view || (input.is_empty()!=self.controller.dupedge_needs_pick()) {
                    return Err(Error::Stale("memory recovery DupEdge selection"));
                }
            }
            MemoryRecoveryTarget::IsocurveStart { authority, epoch, selected, root_revision, sub_revision, draft, waiting_face,.. } => {
                if self.controller.coordinator().authority() != *authority || self.controller.coordinator().epoch() != *epoch
                    || self.controller.selected() != selected || self.controller.selection_state().revision!=*root_revision || self.controller.sub_revision()!=*sub_revision || self.controller.is_running() || *waiting_face || self.transcript.draft() != draft {
                    return Err(Error::Stale("memory recovery Isocurve start"));
                }
            }
            MemoryRecoveryTarget::Sphere { authority, epoch, key, input } => {
                let mut coordinates = input.trim().split(',');
                let point_input = (0..3).all(|_| coordinates.next().is_some_and(|value| value.trim().parse::<f64>().is_ok_and(f64::is_finite)))
                    && coordinates.next().is_none();
                let current = if point_input { self.controller.sphere_point_recovery_key() } else { self.controller.sphere_recovery_key() };
                if self.controller.coordinator().authority() != *authority || self.controller.coordinator().epoch() != *epoch || current != Some(*key) || self.transcript.draft() != input.as_str() {
                    return Err(Error::Stale("memory recovery Sphere input"));
                }
            }
            MemoryRecoveryTarget::SphereSelection { authority, epoch, key, root_revision, input } => {
                if self.controller.coordinator().authority() != *authority || self.controller.coordinator().epoch() != *epoch
                    || self.controller.sphere_fit_selection_recovery_key() != Some(*key)
                    || self.controller.selection_state().revision != *root_revision || self.transcript.draft() != input {
                    return Err(Error::Stale("memory recovery Sphere selection"));
                }
            }
            MemoryRecoveryTarget::SpherePoint { authority, epoch, key, draft, anchor, .. } => {
                if self.controller.coordinator().authority() != *authority || self.controller.coordinator().epoch() != *epoch || self.controller.sphere_point_recovery_key() != Some(*key) || self.transcript.draft() != draft {
                    return Err(Error::Stale("memory recovery Sphere input"));
                }
                self.validate_snap_anchor(anchor)?;
            }
            MemoryRecoveryTarget::Box { authority, epoch, key, input } => {
                if self.controller.coordinator().authority() != *authority || self.controller.coordinator().epoch() != *epoch || self.controller.box_recovery_key() != Some(*key) || self.transcript.draft() != input.as_str() {
                    return Err(Error::Stale("memory recovery Box input"));
                }
            }
            MemoryRecoveryTarget::BoxPoint { authority, epoch, key, draft, anchor, .. } => {
                if self.controller.coordinator().authority() != *authority || self.controller.coordinator().epoch() != *epoch || self.controller.box_recovery_key() != Some(*key) || self.transcript.draft() != draft {
                    return Err(Error::Stale("memory recovery Box input"));
                }
                self.validate_snap_anchor(anchor)?;
            }
        }
        let result = match &target {
            MemoryRecoveryTarget::DirectSolid { input, .. } => self.controller.submit(input),
            MemoryRecoveryTarget::Wire { command, .. } => self.controller.submit(command),
            MemoryRecoveryTarget::Isocurve { .. } => self.controller.submit(""),
            MemoryRecoveryTarget::DupEdge {input,view_root,..} => {
                #[cfg(feature="occt-brep")]
                {if let Some(root)=view_root {self.controller.request_dupedge_source_view(*root)} else {self.controller.submit(input)}}
                #[cfg(not(feature="occt-brep"))]
                {let _=view_root;self.controller.submit(input)}
            },
            MemoryRecoveryTarget::IsocurveStart { .. } => self.controller.submit("ExtractIsocurve"),
            MemoryRecoveryTarget::Sphere { input, .. } => self.controller.submit(input),
            MemoryRecoveryTarget::SphereSelection { input, .. } => self.controller.submit(input),
            MemoryRecoveryTarget::SpherePoint { point, .. } => self.controller.accept_world_point(*point),
            MemoryRecoveryTarget::Box { input, .. } => self.controller.submit(input),
            MemoryRecoveryTarget::BoxPoint { point, .. } => self.controller.accept_world_point(*point),
        };
        match result {
            Ok(()) => {
                #[cfg(feature="occt-brep")]
                if self.controller.brep_wireframe_worker_pending()
                    && let MemoryRecoveryTarget::Wire {waiting_worker,..}=&mut target {
                    *waiting_worker=true;self.memory_recovery_target=Some(target);return Ok(true);
                }
                #[cfg(feature="occt-brep")]
                if self.controller.isocurve_face_verification_pending()
                    && let MemoryRecoveryTarget::IsocurveStart {waiting_face,..}=&mut target {
                    *waiting_face=true;self.isocurve_face_worker_draft=Some(self.transcript.draft().to_owned());
                    self.memory_recovery_target=Some(target);return Ok(true);
                }
                #[cfg(feature="occt-brep")]
                if self.controller.isocurve_build_worker_pending()
                    && let MemoryRecoveryTarget::Isocurve {waiting_worker,..}=&mut target {
                    *waiting_worker=true;self.isocurve_build_worker_draft=Some(self.transcript.draft().to_owned());
                    self.memory_recovery_target=Some(target);return Ok(true);
                }
                #[cfg(feature="occt-brep")]
                if self.controller.dupedge_source_view_pending()
                    && let MemoryRecoveryTarget::DupEdge {key,input,view_root,waiting_view,..}=&mut target {
                    let current=if view_root.is_some() {self.controller.dupedge_source_view_context_key()} else {self.controller.dupedge_recovery_key()};
                    let Some(current)=current else {
                        self.controller.cancel_dupedge_source_view();return Err(Error::Stale("memory recovery DupEdge selection"));
                    };
                    *key=current;
                    *input="";*waiting_view=true;self.memory_recovery_target=Some(target);
                    return Ok(true);
                }
                if let MemoryRecoveryTarget::IsocurveStart { draft, .. } = &target {
                    if draft.trim().eq_ignore_ascii_case("ExtractIsocurve") || draft.trim().eq_ignore_ascii_case("OM9-CURVE-019") {
                        self.transcript.record_command(draft);
                    } else { self.transcript.append_history("Command: ExtractIsocurve"); }
                    self.append_running_prompt();
                }
                if let MemoryRecoveryTarget::SphereSelection { input, .. } = &target {
                    if !input.trim().is_empty() { self.transcript.record_command(input); }
                    self.append_running_prompt();
                }
                if let MemoryRecoveryTarget::SpherePoint { anchor, plane, one_shot, .. } | MemoryRecoveryTarget::BoxPoint { anchor, plane, one_shot, .. } = &target {
                    if self.snap_state.settings.one_shot == *one_shot { self.snap_state.accept(); }
                    if self.smarttrack_enabled && let Some(candidate) = anchor {
                        let _ = self.smarttrack.push(candidate, *plane);
                    }
                    self.append_running_prompt();
                }
                Ok(true)
            }
            Err(error) if Self::is_memory_refusal(&error) => {
                if !self.controller.discard_oldest_undo()? { return Err(error); }
                if let MemoryRecoveryTarget::DupEdge {retried_after_release,..}=&mut target {*retried_after_release=false;}
                if let MemoryRecoveryTarget::IsocurveStart {retried_after_release,..}=&mut target {*retried_after_release=false;}
                if let MemoryRecoveryTarget::Isocurve {retried_after_release,..}=&mut target {*retried_after_release=false;}
                self.memory_recovery_target = Some(target);
                Ok(false)
            }
            Err(error) => Err(error),
        }
    }

    #[cfg(test)]
    fn retry_after_undo_recovery(&mut self) -> Result<usize> {
        let mut removed = 0;
        while !self.retry_undo_recovery_step()? { removed += 1; }
        Ok(removed)
    }

    pub(super) fn memory_recovery_ui(&mut self, ctx: &egui::Context) {
        if !self.memory_recovery_open { return; }
        let mut open = true;
        let mut discard = false;
        let mut done = false;
        let command = match &self.memory_recovery_target { Some(MemoryRecoveryTarget::DirectSolid { command, .. } | MemoryRecoveryTarget::Wire { command, .. }) => *command, Some(MemoryRecoveryTarget::Isocurve { .. } | MemoryRecoveryTarget::IsocurveStart { .. }) => "ExtractIsocurve", Some(MemoryRecoveryTarget::DupEdge { .. })=>"DupEdge", Some(MemoryRecoveryTarget::Sphere { .. } | MemoryRecoveryTarget::SphereSelection { .. } | MemoryRecoveryTarget::SpherePoint { .. }) => "Sphere", Some(MemoryRecoveryTarget::Box { .. } | MemoryRecoveryTarget::BoxPoint { .. }) => "Box", _ => "ExtractWireframe" };
        let starting_isocurve = matches!(self.memory_recovery_target, Some(MemoryRecoveryTarget::IsocurveStart { .. }));
        let loading_sphere_selection = matches!(self.memory_recovery_target, Some(MemoryRecoveryTarget::SphereSelection { .. }));
        let clear_isocurve_draft = matches!(&self.memory_recovery_target, Some(MemoryRecoveryTarget::IsocurveStart { draft, .. })
            if self.transcript.draft() == draft && (draft.trim().eq_ignore_ascii_case("ExtractIsocurve") || draft.trim().eq_ignore_ascii_case("OM9-CURVE-019")));
        let clear_solid_draft = matches!(&self.memory_recovery_target, Some(MemoryRecoveryTarget::DirectSolid { input, .. } | MemoryRecoveryTarget::Sphere { input, .. } | MemoryRecoveryTarget::SphereSelection { input, .. } | MemoryRecoveryTarget::Box { input, .. }) if self.transcript.draft() == input.as_str());
        let clear_dupedge_draft = matches!(&self.memory_recovery_target, Some(MemoryRecoveryTarget::DupEdge { draft, input, .. })
            if !input.is_empty() && self.transcript.draft()==draft && (draft.trim().eq_ignore_ascii_case("DupEdge") || draft.trim().eq_ignore_ascii_case("OM9-CURVE-040")));
        let modal = super::theme::modal(ctx, egui::Id::new("memory-recovery")).show(ctx, |ui| {
                ui.heading("Not enough memory");
                ui.label("The operation could not be completed because memory is limited.");
                ui.label(format!("Undo steps available: {}", self.controller.coordinator().undo_len()));
                ui.label(format!("Allow removing oldest Undo steps until {command} can continue?"));
                ui.label("Steps are removed permanently. All may be removed without freeing enough memory.");
                ui.label("The drawing and Redo remain unchanged if the operation fails.");
                if self.memory_recovery_running {
                    ui.label(format!("Retrying; {} Undo step(s) removed.", self.memory_recovery_removed));
                }
                ui.horizontal(|ui| {
                    let response = ui.add_enabled(!self.memory_recovery_running,
                        egui::Button::new("Remove as needed and retry"));
                    ctx.data_mut(|data| data.insert_temp(egui::Id::new("memory-recovery-remove"), response.rect));
                    discard = response.clicked();
                    let cancel = ui.button("Cancel");
                    ctx.data_mut(|data| data.insert_temp(egui::Id::new("memory-recovery-cancel"), cancel.rect));
                    done = cancel.clicked() || (self.memory_recovery_running && cancel.is_pointer_button_down_on());
                });
            });
        done |= modal.is_top_modal && ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
        if discard {
            self.memory_recovery_running = true;
            self.memory_recovery_removed = 0;
        }
        if !open || done {
            #[cfg(feature="occt-brep")]
            if self.dupedge_recovery_waiting() {self.controller.cancel_dupedge_source_view();}
            #[cfg(feature="occt-brep")]
            if self.isocurve_recovery_waiting() && self.controller.isocurve_face_verification_pending() {self.controller.cancel();}
            #[cfg(feature="occt-brep")]
            if self.isocurve_build_recovery_waiting() {self.controller.cancel_isocurve_build_worker();}
            #[cfg(feature="occt-brep")]
            if self.wire_recovery_waiting() && self.controller.brep_wireframe_worker_pending() {self.controller.cancel();}
            if self.memory_recovery_running {
                let message = format!("Memory recovery stopped after removing {} Undo step(s).", self.memory_recovery_removed);
                self.transcript.append_history(&message); self.status.info(&message);
            }
            open = false;
        } else if self.memory_recovery_running && (self.memory_recovery_defer
            || (matches!(self.memory_recovery_target,Some(MemoryRecoveryTarget::Wire {waiting_worker:false,..}))
                && {#[cfg(feature="occt-brep")] {self.controller.brep_wireframe_worker_active()} #[cfg(not(feature="occt-brep"))] {false}})
            || (matches!(self.memory_recovery_target,Some(MemoryRecoveryTarget::Isocurve {waiting_worker:false,..}))
                && {#[cfg(feature="occt-brep")] {self.controller.isocurve_build_worker_active()} #[cfg(not(feature="occt-brep"))] {false}})
            || matches!(self.memory_recovery_target,Some(MemoryRecoveryTarget::DupEdge {waiting_view:true,..} | MemoryRecoveryTarget::IsocurveStart {waiting_face:true,..} | MemoryRecoveryTarget::Isocurve {waiting_worker:true,..} | MemoryRecoveryTarget::Wire {waiting_worker:true,..}))) {
            self.memory_recovery_defer=false;ctx.request_repaint_after(std::time::Duration::from_millis(16));
        } else if self.memory_recovery_running {
            match self.retry_undo_recovery_step() {
                Ok(true) => {
                    #[cfg(feature="occt-brep")]
                    let dupedge_view_started=command=="DupEdge" && self.controller.dupedge_source_view_pending();
                    #[cfg(not(feature="occt-brep"))]
                    let dupedge_view_started=false;
                    #[cfg(feature="occt-brep")]
                    let isocurve_face_started=starting_isocurve && self.isocurve_recovery_waiting();
                    #[cfg(not(feature="occt-brep"))]
                    let isocurve_face_started=false;
                    #[cfg(feature="occt-brep")]
                    let isocurve_build_started=self.isocurve_build_recovery_waiting();
                    #[cfg(not(feature="occt-brep"))]
                    let isocurve_build_started=false;
                    #[cfg(feature="occt-brep")]
                    let wire_worker_started=command=="ExtractWireframe" && self.wire_recovery_waiting();
                    #[cfg(not(feature="occt-brep"))]
                    let wire_worker_started=false;
                    let verb = if loading_sphere_selection { "selection loaded" }
                        else if isocurve_face_started { "verification started" }
                        else if isocurve_build_started { "worker started" }
                        else if starting_isocurve { "started" }
                        else if dupedge_view_started {"source view started"}
                        else if wire_worker_started {"worker started"}
                        else { "completed" };
                    let message = format!("{command} {verb} after removing {} oldest Undo step(s).", self.memory_recovery_removed);
                    // This successful retry follows the user's explicit consent
                    // to resolve the very error currently shown.
                    let prior_status=self.status.message().strip_prefix("DupEdge source view unavailable: ").unwrap_or(self.status.message());
                    if dupedge_view_started && matches!(prior_status,
                        "budget exceeded: bytes" | "budget exceeded: managed memory" | "budget exceeded: managed RAM"
                        | "budget exceeded: available physical RAM" | "budget exceeded: available system commit"
                        | "budget exceeded: external memory headroom") {self.status.acknowledge();}
                    self.transcript.append_history(&message); self.status.info(&message);
                    if !dupedge_view_started && !isocurve_face_started && !isocurve_build_started && !wire_worker_started {
                        if clear_solid_draft || clear_isocurve_draft || clear_dupedge_draft || self.transcript.draft().trim().eq_ignore_ascii_case(command) { self.transcript.set_draft(""); }
                        open = false;
                    }
                }
                Ok(false) => {
                    self.memory_recovery_removed += 1;
                    ctx.request_repaint();
                }
                Err(error) => {
                    let message = format!("{command} did not complete after removing {} Undo step(s): {error}", self.memory_recovery_removed);
                    self.transcript.append_history(&message); self.status.error(&message);
                    open = false;
                }
            }
        }
        self.memory_recovery_open = open && !done;
        if !self.memory_recovery_open { self.memory_recovery_running = false; self.memory_recovery_defer=false; self.memory_recovery_target = None; self.focus_cmd = true; }
    }

    pub(super) fn open_undo_settings_command(&mut self, text: &str) -> bool {
        if !["Options", "RhinoOptions", "UndoSettings", "DocumentProperties"]
            .iter()
            .any(|s| text.trim().eq_ignore_ascii_case(s))
        {
            return false;
        }
        self.options.page = if text.trim().eq_ignore_ascii_case("UndoSettings") {
            Some(options::general())
        } else if text.trim().eq_ignore_ascii_case("DocumentProperties") {
            Some(options::document_properties())
        } else {
            None
        };
        self.options.draft = Some(self.options_draft());
        self.undo_settings_draft = Some(self.controller.coordinator().undo_policy());
        self.viewport_menu_owns_frame = true;
        self.focus_cmd = false;
        true
    }

    pub fn load_undo_settings(&mut self, path: &std::path::Path) -> Result<()> {
        self.undo_settings_path = Some(path.to_path_buf());
        if !path.exists() {
            return Ok(());
        }
        if std::fs::metadata(path)
            .map_err(|e| Error::Io(e.to_string()))?
            .len()
            > 4096
        {
            return Err(Error::Budget("Undo preferences file"));
        }
        let bytes = std::fs::read(path).map_err(|e| Error::Io(e.to_string()))?;
        let policy: UndoPolicy =
            serde_json::from_slice(&bytes).map_err(|_| Error::Invalid("Undo preferences"))?;
        self.controller.set_undo_policy(policy)
    }

    fn save_undo_settings(&self, policy: UndoPolicy) -> Result<()> {
        let Some(path) = &self.undo_settings_path else {
            return Ok(());
        };
        let bytes = serde_json::to_vec(&policy).map_err(|_| Error::Invalid("Undo preferences"))?;
        let parent = path
            .parent()
            .ok_or(Error::Invalid("Undo preferences path"))?;
        std::fs::create_dir_all(parent).map_err(|e| Error::Io(e.to_string()))?;
        let staging = parent.join(format!(".undo-{}.tmp", uuid::Uuid::new_v4()));
        std::fs::write(&staging, bytes).map_err(|e| Error::Io(e.to_string()))?;
        if let Err(error) = std::fs::rename(&staging, path) {
            let _ = std::fs::remove_file(&staging);
            return Err(Error::Io(error.to_string()));
        }
        Ok(())
    }

    pub(super) fn undo_settings_ui(&mut self, ctx: &egui::Context) {
        let Some(mut draft) = self.undo_settings_draft else { return; };
        let mut options_draft = self.options.draft.take().unwrap_or_else(|| self.options_draft());
        let mut page = self.options.page;
        let mut open = true;
        let mut apply = false;
        let mut cancel = false;
        let available = ctx.content_rect().size();
        let height = (available.y - 135.).clamp(220., 530.);
        let width = (available.x - 85.).clamp(460., 790.);
        let tree_width = (width * 0.34).clamp(175., 260.);
        let detail_width = width - tree_width - 32.;
        let id = egui::Id::new("undo-settings");
        let top = ((available.y - height - 90.) * 0.5).max(0.);
        super::theme::modal(ctx, id)
            .area(egui::Modal::default_area(id).anchor(egui::Align2::CENTER_TOP, egui::vec2(0., top)))
            .backdrop_color(Color32::from_black_alpha(68))
            .show(ctx, |ui| {
                options::style_dialog(ui);
                ui.set_width(width);
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("OpenMatrix9 Options").font(super::typography::semibold(ctx, 19.)));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        open = !ui.button("×").on_hover_text("Close Options").clicked();
                    });
                });
                ui.separator();
                ui.horizontal_top(|ui| {
                    egui::Frame::group(ui.style()).show(ui, |ui| {
                        ui.set_width(tree_width);
                        egui::ScrollArea::both().id_salt("options-tree").max_width(tree_width)
                            .max_height(height).min_scrolled_height(height).auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui.vertical(|ui| options::tree(ui, &mut page));
                            });
                    });
                    ui.vertical(|ui| {
                        ui.set_width(detail_width);
                        let selected = page.unwrap_or_else(options::view);
                        egui::ScrollArea::vertical().id_salt(selected.id()).max_height(height).min_scrolled_height(height)
                            .auto_shrink([false, false]).show(ui, |ui| {
                                if selected.is_general() {
                                    self.general_options(ui, &mut draft);
                                } else {
                                    self.options_page(ui, selected, &mut options_draft);
                                }
                            });
                    });
                });
                ui.separator();
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let cancel_button = ui.button("Cancel");
                    cancel = cancel_button.clicked();
                    ctx.data_mut(|data| data.insert_temp(egui::Id::new("options-cancel-rect"), cancel_button.rect));
                    let response = ui.add_enabled(!self.controller.is_running(), egui::Button::new("OK"));
                    apply = response.clicked();
                    ctx.data_mut(|data| data.insert_temp(egui::Id::new("undo-apply-rect"), response.rect));
                });
            });
        cancel |= ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
        if apply && !cancel {
            let result = (|| {
                if draft != self.controller.coordinator().undo_policy() {
                    self.controller.set_undo_policy(draft)?;
                    self.save_undo_settings(draft)?;
                }
                self.apply_options_draft(options_draft.clone())
            })();
            if result.is_ok() {
                open = false;
            }
            self.feedback(result);
        }
        let active = open && !cancel;
        self.undo_settings_draft = active.then_some(draft);
        self.options.draft = active.then_some(options_draft);
        self.options.page = page;
        if !active {
            self.focus_cmd = true;
        }
    }

    fn general_options(&mut self, ui: &mut egui::Ui, draft: &mut UndoPolicy) {
        let ctx = ui.ctx().clone();
        let max_mb = self.controller.coordinator().undo_ram_limit_mb();
        ui.heading("General");
        ui.add_enabled_ui(!self.controller.is_running(), |ui| self.matrix_language_control(ui));
        ui.heading("Undo");
        egui::Grid::new("undo-policy-fields").show(ui, |ui| {
            ui.label("Min number of undos:");
            let r = ui.add(egui::DragValue::new(&mut draft.minimum_steps).range(1..=1_000_000));
            ctx.data_mut(|m| m.insert_temp(egui::Id::new("undo-minimum-rect"), r.rect));
            ui.end_row();
            ui.label("Max memory used (MB):");
            let r = ui.add(
                egui::DragValue::new(&mut draft.max_memory_mb)
                    .range(1..=max_mb.unwrap_or(1024)),
            );
            ctx.data_mut(|m| m.insert_temp(egui::Id::new("undo-memory-rect"), r.rect));
            ui.end_row();
        });
        ui.label("Oldest steps are removed over the memory budget; the minimum is kept.");
        if let Some(max) = max_mb {
            ui.label(format!(
                "Maximum configurable: {max} MB (50% of physical RAM)."
            ));
        } else {
            ui.colored_label(
                Color32::YELLOW,
                "Physical RAM capacity unavailable; the 50% limit cannot be verified.",
            );
        }
        let c = self.controller.coordinator();
        {
            let stats = c.undo_memory_stats();
            let bytes = stats.estimated_bytes;
            ui.label(format!(
                "Undo: {}   Redo: {}   Estimated history memory: {:.1} MB",
                c.undo_len(),
                c.redo_len(),
                bytes as f64 / 1_048_576.
            ));
            if stats.capture_change_pending {
                ui.label("History memory estimate awaits capture-owner reconciliation.");
            }
        }
        ui.label(
            "Full memory inventory every 100 MB of growth, or earlier near the budget.",
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[cfg(feature="occt-brep")]
    fn background_wireframe_memory_recovery_preserves_context() {
        use om9_core::{budget::{Limits,memory::MemoryClass},geometry::Geometry,model::Entity};
        for mode in 0..3 {
            let limits=Limits::default();let shape=om9_kernel::brep::box_brep([4.,5.,6.],0.1,&limits).unwrap();
            let mut doc=Document::new("background wire memory");let entity=Entity::new("box",Geometry::Brep(shape),None);
            let id=entity.id;doc.entities.insert(id,entity);let mut app=MatrixApp::new(doc).unwrap();
            app.controller.submit("Point").unwrap();app.controller.submit("20,20,20").unwrap();app.controller.undo().unwrap();
            app.controller.select_objects(&[id]).unwrap();let before=app.controller.document().clone();
            let undo=app.controller.coordinator().undo_len();let redo=app.controller.coordinator().redo_len();
            app.transcript.set_draft("ExtractWireframe");app.submit_draft();
            assert!(app.controller.brep_wireframe_worker_pending(),"{} recovery={}",app.status_line(),app.memory_recovery_open);
            let pressure=app.controller.coordinator().reserve_remaining_external_memory_with_held(
                MemoryClass::Kernel,None,app.controller.scene_cache_usage().unwrap()).unwrap();
            if mode==1 {app.transcript.set_draft("new command");}
            let ctx=egui::Context::default();let rect=Rect::from_min_size(egui::Pos2::ZERO,Vec2::new(1200.,800.));
            let frame=|app:&mut MatrixApp| {let mut out=ctx.run_ui(egui::RawInput {screen_rect:Some(rect),focused:true,..Default::default()},|ui|app.show(ui));out.textures_delta.clear();};
            let deadline=std::time::Instant::now()+std::time::Duration::from_secs(30);
            while app.controller.brep_wireframe_worker_pending() {frame(&mut app);assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(5));}
            assert_eq!(app.controller.document(),&before);assert_eq!(app.controller.coordinator().undo_len(),undo);assert_eq!(app.controller.coordinator().redo_len(),redo);
            if mode==1 {assert!(!app.memory_recovery_open);assert_eq!(app.transcript.draft(),"new command");drop(pressure);continue;}
            assert!(app.memory_recovery_open);drop(pressure);
            let deadline=std::time::Instant::now()+std::time::Duration::from_secs(30);
            while app.controller.brep_wireframe_worker_active() {assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(5));}
            app.memory_recovery_running=true;assert!(app.retry_undo_recovery_step().unwrap());assert!(app.wire_recovery_waiting());
            if mode==2 {
                app.transcript.set_draft("changed while waiting");frame(&mut app);
                assert!(!app.memory_recovery_open);assert!(!app.controller.brep_wireframe_worker_pending());
                assert_eq!(app.controller.document(),&before);assert_eq!(app.controller.coordinator().undo_len(),undo);assert_eq!(app.controller.coordinator().redo_len(),redo);
            } else {
                let deadline=std::time::Instant::now()+std::time::Duration::from_secs(30);
                while app.memory_recovery_open {frame(&mut app);assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(5));}
                assert_eq!(app.controller.coordinator().undo_len(),undo+1);
                assert_eq!(app.controller.document().entities.values().filter(|e|e.label=="ExtractWire").count(),12);
                app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
            }
        }
    }
    #[test]
    #[cfg(feature="occt-brep")]
    fn native_dupedge_view_memory_prompt_retries_after_consent_and_checks_selection() {
        use om9_core::{budget::Limits,geometry::Geometry,model::Entity};
        let limits=Limits::default();let shape=om9_kernel::brep::box_brep([4.,5.,6.],0.1,&limits).unwrap();
        let native=om9_kernel::brep::to_3dm(&shape,1e-6,&limits).unwrap();
        for stale in [false,true] {
            let mut doc=Document::with_template_tolerance("native view recovery");
            let entity=Entity::new("box",Geometry::NativeBrep(native.clone()),None);let id=entity.id;doc.entities.insert(id,entity);
            let mut app=MatrixApp::new(doc).unwrap();
            app.controller.submit("Point").unwrap();app.controller.submit("10,10,10").unwrap();
            app.controller.submit("Point").unwrap();app.controller.submit("20,20,20").unwrap();app.controller.undo().unwrap();
            let before=app.controller.document().clone();let undo=app.controller.coordinator().undo_len();
            app.controller.submit("DupEdge").unwrap();app.controller.request_dupedge_source_view(id).unwrap();
            app.controller.inject_dupedge_source_view_error_for_test(Error::Budget("bytes")).unwrap();
            let ctx=egui::Context::default();
            let frame=|app:&mut MatrixApp,events| {let mut out=ctx.run_ui(egui::RawInput {
                screen_rect:Some(Rect::from_min_size(egui::Pos2::ZERO,Vec2::new(900.,600.))),focused:true,events,..Default::default()},|ui|app.show(ui));out.textures_delta.clear();};
            frame(&mut app,vec![]);assert!(app.memory_recovery_open,"{}",app.status_line());
            assert_eq!(app.controller.coordinator().undo_len(),undo);assert_eq!(app.controller.coordinator().redo_len(),1);
            if stale {
                app.controller.select_objects(&[id]).unwrap();
                assert!(matches!(app.retry_undo_recovery_step(),Err(Error::Stale(_))));
                assert_eq!(app.controller.coordinator().undo_len(),undo);assert_eq!(app.controller.document(),&before);
                continue;
            }
            for _ in 0..2 {frame(&mut app,vec![]);}
            let pos=ctx.data(|d|d.get_temp::<Rect>(egui::Id::new("memory-recovery-remove"))).unwrap().center();
            for pressed in [true,false] {frame(&mut app,vec![egui::Event::PointerMoved(pos),egui::Event::PointerButton {pos,button:egui::PointerButton::Primary,pressed,modifiers:Modifiers::NONE}]);}
            let deadline=std::time::Instant::now()+std::time::Duration::from_secs(20);
            while app.memory_recovery_open {assert!(std::time::Instant::now()<deadline,"{}",app.status_line());frame(&mut app,vec![]);std::thread::yield_now();}
            assert_eq!(app.memory_recovery_removed,0);assert_eq!(app.controller.coordinator().undo_len(),undo);
            assert_eq!(app.controller.coordinator().redo_len(),1);assert_eq!(app.controller.document(),&before);
            let stamp=app.controller.document().stamp(id).unwrap();assert_eq!(app.controller.scene().unwrap().root_brep_pick_display(&stamp).unwrap().topology.edges.len(),12);
        }
    }
    #[test]
    #[cfg(feature="occt-brep")]
    fn preselected_native_dupedge_view_admission_failure_offers_retry() {
        use om9_core::{budget::{Limits,memory::MemoryClass},geometry::Geometry,model::Entity};
        let limits=Limits::default();let shape=om9_kernel::brep::box_brep([4.,5.,6.],0.1,&limits).unwrap();
        let native=om9_kernel::brep::to_3dm(&shape,1e-6,&limits).unwrap();
        for menu in [false,true] {
            let mut doc=Document::with_template_tolerance("native initial admission");
            let entity=Entity::new("box",Geometry::NativeBrep(native.clone()),None);let id=entity.id;doc.entities.insert(id,entity);
            let mut app=MatrixApp::new(doc).unwrap();app.controller.submit("Point").unwrap();app.controller.submit("1,2,3").unwrap();app.controller.undo().unwrap();
            app.controller.select_objects(&[id]).unwrap();let before=app.controller.document().clone();
            let pressure=app.controller.coordinator().reserve_remaining_external_memory_with_held(MemoryClass::Kernel,None,app.controller.scene_cache_usage().unwrap()).unwrap();
            if menu {app.invoke("OM9-CURVE-040",InvocationOrigin::Menu);} else {app.transcript.set_draft("DupEdge");app.submit_draft();}
            assert!(app.memory_recovery_open,"{}",app.status_line());assert!(app.controller.dupedge_needs_pick());
            assert_eq!(app.controller.coordinator().redo_len(),1);assert_eq!(app.controller.document(),&before);drop(pressure);
            assert_eq!(app.retry_after_undo_recovery().unwrap(),0);assert!(app.controller.dupedge_source_view_pending());
            let deadline=std::time::Instant::now()+std::time::Duration::from_secs(20);
            loop {if let Some(result)=app.controller.poll_dupedge_source_view() {app.finish_dupedge_view_recovery(result);break;}assert!(std::time::Instant::now()<deadline);std::thread::yield_now();}
            assert!(!app.memory_recovery_open);assert_eq!(app.controller.coordinator().redo_len(),1);assert_eq!(app.controller.document(),&before);
        }
    }
    #[test]
    #[cfg(feature="occt-brep")]
    fn native_dupedge_view_reclaims_oldest_undo_only_after_consent() {
        use om9_core::{budget::{Limits,memory::MemoryClass},geometry::{Geometry,Mesh},model::Entity};
        let limits=Limits {max_bytes:64*1024*1024,..Limits::default()};
        let shape=om9_kernel::brep::box_brep([4.,5.,6.],0.1,&limits).unwrap();
        let native=om9_kernel::brep::to_3dm(&shape,1e-6,&limits).unwrap();
        let entity=Entity::new("box",Geometry::NativeBrep(native),None);let id=entity.id;
        let old=Entity::new("old mesh",Geometry::Mesh(Mesh {vertices:vec![[1.,1.,1.];400000],faces:vec![],normals:vec![],colors:vec![],corner_uvs:vec![]}),None);let old_id=old.id;
        let mut doc=Document::with_template_tolerance("native view reclaim");doc.entities.insert(id,entity);doc.entities.insert(old_id,old);
        let mut app=MatrixApp::new(doc.clone()).unwrap();app.controller=crate::session::Controller::new(doc,limits).unwrap();
        app.controller.delete_objects(&[old_id]).unwrap();app.controller.submit("Point").unwrap();app.controller.submit("10,10,10").unwrap();app.controller.undo().unwrap();
        let before=app.controller.document().clone();assert_eq!((app.controller.coordinator().undo_len(),app.controller.coordinator().redo_len()),(1,1));
        app.controller.submit("DupEdge").unwrap();app.controller.request_dupedge_source_view(id).unwrap();
        app.controller.inject_dupedge_source_view_error_for_test(Error::Budget("bytes")).unwrap();
        let deadline=std::time::Instant::now()+std::time::Duration::from_secs(20);
        while app.controller.dupedge_source_view_worker_active() {assert!(std::time::Instant::now()<deadline);std::thread::yield_now();}
        let mut pressure=app.controller.coordinator().reserve_remaining_external_memory_with_held(MemoryClass::Kernel,None,app.controller.scene_cache_usage().unwrap()).unwrap();
        pressure.shrink_to(pressure.bytes()-1024).unwrap();
        let ctx=egui::Context::default();let frame=|app:&mut MatrixApp,events| {let mut out=ctx.run_ui(egui::RawInput {
            screen_rect:Some(Rect::from_min_size(egui::Pos2::ZERO,Vec2::new(900.,600.))),focused:true,events,..Default::default()},|ui|app.show(ui));out.textures_delta.clear();};
        for _ in 0..3 {frame(&mut app,vec![]);}assert!(app.memory_recovery_open);assert_eq!(app.controller.coordinator().undo_len(),1);
        let pos=ctx.data(|d|d.get_temp::<Rect>(egui::Id::new("memory-recovery-remove"))).unwrap().center();
        for pressed in [true,false] {frame(&mut app,vec![egui::Event::PointerMoved(pos),egui::Event::PointerButton {pos,button:egui::PointerButton::Primary,pressed,modifiers:Modifiers::NONE}]);}
        while app.memory_recovery_open {assert!(std::time::Instant::now()<deadline,"{}",app.status_line());frame(&mut app,vec![]);std::thread::yield_now();}
        assert_eq!(app.memory_recovery_removed,1,"{}",app.status_line());assert_eq!(app.controller.coordinator().undo_len(),0);
        assert_eq!(app.controller.coordinator().redo_len(),1);assert_eq!(app.controller.document(),&before);
        let stamp=app.controller.document().stamp(id).unwrap();assert_eq!(app.controller.scene().unwrap().root_brep_pick_display(&stamp).unwrap().topology.edges.len(),12);
        drop(pressure);
    }
    #[test]
    fn trim_allocation_error_enters_memory_recovery_but_overflow_does_not() {
        assert!(MatrixApp::is_memory_refusal(&Error::Budget("bytes")));
        assert!(!MatrixApp::is_memory_refusal(&Error::Overflow("trim decomposition allocation")));
    }
    #[test]
    fn memory_recovery_requires_click_and_preserves_document_and_redo() {
        let ctx = egui::Context::default();
        use om9_core::{geometry::{Geometry, Mesh}, model::Entity};
        let mut doc = Document::new("memory recovery");
        let mesh = Mesh { vertices: vec![[0.,0.,0.], [2.,0.,0.], [0.,2.,0.]], faces: vec![vec![0,1,2]], normals: vec![], colors: vec![], corner_uvs: vec![] };
        let entity = Entity::new("mesh", Geometry::Mesh(mesh), None); let id = entity.id; doc.entities.insert(id, entity);
        let mut app = MatrixApp::new(doc).unwrap();
        for p in ["1,0,0", "2,0,0", "3,0,0"] {
            app.controller.submit("Point").unwrap(); app.controller.submit(p).unwrap();
        }
        app.controller.undo().unwrap();
        app.controller.select_objects(&[id]).unwrap();
        let before = app.controller.document().clone();
        assert_eq!((app.controller.coordinator().undo_len(), app.controller.coordinator().redo_len()), (2, 1));
        let pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::Kernel, None, app.controller.scene_cache_usage().unwrap()).unwrap();
        app.transcript.set_draft("ExtractWireframe"); app.submit_draft();
        assert!(app.memory_recovery_open);
        let frame = |app: &mut MatrixApp, events| {
            let mut output = ctx.run_ui(egui::RawInput { screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, Vec2::new(900.,600.))), focused: true, events, ..Default::default() },
                |ui| app.memory_recovery_ui(ui.ctx()));
            output.textures_delta.clear();
        };
        for _ in 0..3 { frame(&mut app, vec![]); }
        assert_eq!(app.controller.coordinator().undo_len(), 2);
        assert_eq!(app.controller.document(), &before);
        drop(pressure);
        let pos = ctx.data(|d| d.get_temp::<Rect>(egui::Id::new("memory-recovery-remove"))).unwrap().center();
        for pressed in [true, false] {
            frame(&mut app, vec![egui::Event::PointerMoved(pos), egui::Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed, modifiers: Modifiers::NONE }]);
        }
        assert!(!app.memory_recovery_open);
        assert_eq!(app.controller.document().entities.len(), before.entities.len() + 3);
        assert_eq!((app.controller.coordinator().undo_len(), app.controller.coordinator().redo_len()), (3, 0));
        app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
        app.controller.undo().unwrap(); assert_eq!(app.controller.document().entities.len(), 2);
        app.controller.undo().unwrap(); assert_eq!(app.controller.document().entities.len(), 1);
        assert!(app.controller.undo().is_err());
    }

    #[test]
    fn memory_recovery_retries_one_step_per_frame_and_can_be_stopped() {
        use om9_core::{budget::memory::MemoryClass, geometry::{Geometry, Mesh}, model::Entity};
        let ctx = egui::Context::default(); let mut doc = Document::new("cancel recovery");
        let mesh = Mesh { vertices: vec![[0.,0.,0.],[2.,0.,0.],[0.,2.,0.]], faces: vec![vec![0,1,2]], normals: vec![], colors: vec![], corner_uvs: vec![] };
        let entity = Entity::new("mesh", Geometry::Mesh(mesh), None); let id = entity.id; doc.entities.insert(id, entity);
        let mut app = MatrixApp::new(doc).unwrap();
        for p in ["1,0,0", "2,0,0", "3,0,0", "4,0,0"] { app.controller.submit("Point").unwrap(); app.controller.submit(p).unwrap(); }
        app.controller.undo().unwrap(); app.controller.select_objects(&[id]).unwrap(); let before = app.controller.document().clone();
        let pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
            MemoryClass::Kernel, None, app.controller.scene_cache_usage().unwrap()).unwrap();
        app.transcript.set_draft("ExtractWireframe"); app.submit_draft(); assert!(app.memory_recovery_open);
        let frame = |app: &mut MatrixApp, events| {
            let mut output = ctx.run_ui(egui::RawInput { screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, Vec2::new(900.,600.))), focused: true, events, ..Default::default() },
                |ui| app.memory_recovery_ui(ui.ctx())); output.textures_delta.clear();
        };
        for _ in 0..3 { frame(&mut app, vec![]); }
        let confirm = ctx.data(|d| d.get_temp::<Rect>(egui::Id::new("memory-recovery-remove"))).unwrap().center();
        for pressed in [true, false] { frame(&mut app, vec![egui::Event::PointerMoved(confirm), egui::Event::PointerButton { pos: confirm, button: egui::PointerButton::Primary, pressed, modifiers: Modifiers::NONE }]); }
        assert_eq!(app.controller.coordinator().undo_len(), 2);
        assert!(app.memory_recovery_running);
        frame(&mut app, vec![]);
        assert_eq!(app.controller.coordinator().undo_len(), 1);
        let cancel = ctx.data(|d| d.get_temp::<Rect>(egui::Id::new("memory-recovery-cancel"))).unwrap().center();
        frame(&mut app, vec![egui::Event::PointerMoved(cancel), egui::Event::PointerButton { pos: cancel, button: egui::PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE }]);
        assert!(!app.memory_recovery_open); assert!(!app.memory_recovery_running);
        assert_eq!((app.controller.coordinator().undo_len(), app.controller.coordinator().redo_len()), (1, 1));
        assert_eq!(app.controller.document(), &before);
        drop(pressure); app.controller.redo().unwrap();
    }

    #[test]
    fn memory_recovery_rechecks_selection_between_steps() {
        use om9_core::{budget::memory::MemoryClass, geometry::{Geometry, Mesh}, model::Entity};
        let mut doc = Document::new("changed between retries");
        let mesh = Mesh { vertices: vec![[0.,0.,0.],[2.,0.,0.],[0.,2.,0.]], faces: vec![vec![0,1,2]], normals: vec![], colors: vec![], corner_uvs: vec![] };
        let entity = Entity::new("mesh", Geometry::Mesh(mesh), None); let id = entity.id; doc.entities.insert(id, entity);
        let mut app = MatrixApp::new(doc).unwrap();
        for p in ["1,0,0", "2,0,0"] { app.controller.submit("Point").unwrap(); app.controller.submit(p).unwrap(); }
        app.controller.select_objects(&[id]).unwrap();
        let pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
            MemoryClass::Kernel, None, app.controller.scene_cache_usage().unwrap()).unwrap();
        app.transcript.set_draft("ExtractWireframe"); app.submit_draft(); assert!(app.memory_recovery_open);
        assert!(!app.retry_undo_recovery_step().unwrap()); assert_eq!(app.controller.coordinator().undo_len(), 1);
        drop(pressure);
        app.controller.select_objects(&[]).unwrap(); let before = app.controller.coordinator().snapshot();
        assert!(matches!(app.retry_undo_recovery_step(), Err(Error::Stale(_))));
        assert_eq!(app.controller.coordinator().snapshot(), before);
    }

    #[test]
    fn unrelated_budget_errors_do_not_offer_history_removal() {
        let mut app = MatrixApp::new(Document::new("structural limit")).unwrap();
        app.feedback(Err(Error::Budget("command input")));
        assert!(!app.memory_recovery_open);
    }
    #[test]
    fn dupborder_single_surface_memory_refusal_can_retry_after_consent() {
        use om9_core::{budget::{Limits,memory::MemoryClass},geometry::Geometry,model::Entity};
        let limits=Limits::default();let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,5.,&limits).unwrap();
        for menu in [false,true] {
            let mut doc=Document::with_template_tolerance("single surface admission");
            let mut entity=Entity::new("plane",Geometry::Surface(surface.clone()),None);
            entity.placement=om9_core::geometry::Transform::translation([4.,-2.,3.]);let id=entity.id;doc.entities.insert(id,entity);
            let mut app=MatrixApp::new(doc).unwrap();app.controller.submit("Point").unwrap();app.controller.submit("30,30,30").unwrap();app.controller.undo().unwrap();
            app.controller.select_objects(&[id]).unwrap();let before=app.controller.document().clone();
            let pressure=app.controller.coordinator().reserve_remaining_external_memory_with_held(MemoryClass::Kernel,None,app.controller.scene_cache_usage().unwrap()).unwrap();
            if menu {app.invoke("OM9-CURVE-039",InvocationOrigin::Menu);} else {app.transcript.set_draft("DupBorder");app.submit_draft();}
            assert!(app.memory_recovery_open,"{}",app.status_line());assert_eq!(app.controller.document(),&before);assert_eq!(app.controller.coordinator().redo_len(),1);
            let ctx=egui::Context::default();let frame=|app:&mut MatrixApp,events| {let mut out=ctx.run_ui(egui::RawInput {
                screen_rect:Some(Rect::from_min_size(egui::Pos2::ZERO,Vec2::new(900.,600.))),focused:true,events,..Default::default()},|ui|app.memory_recovery_ui(ui.ctx()));out.textures_delta.clear();};
            for _ in 0..3 {frame(&mut app,vec![]);}assert_eq!(app.controller.document(),&before);drop(pressure);
            let pos=ctx.data(|d|d.get_temp::<Rect>(egui::Id::new("memory-recovery-remove"))).unwrap().center();
            for pressed in [true,false] {frame(&mut app,vec![egui::Event::PointerMoved(pos),egui::Event::PointerButton {pos,button:egui::PointerButton::Primary,pressed,modifiers:Modifiers::NONE}]);}
            for _ in 0..4 {if app.memory_recovery_open {frame(&mut app,vec![]);}}
            assert!(!app.memory_recovery_open,"{}",app.status_line());assert_eq!(app.memory_recovery_removed,0);
            let outputs:Vec<_>=app.controller.document().entities.values().filter(|e|e.label=="DupBorder").collect();assert_eq!(outputs.len(),4);
            assert!(outputs.iter().all(|entity|entity.placement==om9_core::geometry::Transform::translation([4.,-2.,3.])));
            let after=app.controller.document().clone();app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);app.controller.redo().unwrap();assert_eq!(app.controller.document(),&after);
        }
    }

    #[test]
    fn dupborder_memory_recovery_retries_multiple_surfaces_and_rejects_stale_selection() {
        use om9_core::{budget::memory::MemoryClass, geometry::Geometry, model::Entity};
        for menu in [false,true] {
            let limits=om9_core::budget::Limits::default();
            let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
            let a=Entity::new("a",Geometry::Surface(surface.clone()),None);let b=Entity::new("b",Geometry::Surface(surface),None);
            let ids=[a.id,b.id];let mut doc=Document::new("border recovery");doc.entities.insert(a.id,a);doc.entities.insert(b.id,b);
            let mut app=MatrixApp::new(doc).unwrap();app.controller.submit("Point").unwrap();app.controller.submit("1,2,3").unwrap();
            app.controller.undo().unwrap();app.controller.select_objects(&ids).unwrap();let before=app.controller.document().clone();
            let pressure=app.controller.coordinator().reserve_remaining_external_memory_with_held(MemoryClass::Kernel,None,app.controller.scene_cache_usage().unwrap()).unwrap();
            if menu {app.invoke("OM9-CURVE-039",InvocationOrigin::Menu);} else {app.transcript.set_draft("DupBorder");app.submit_draft();}
            assert!(app.memory_recovery_open);assert_eq!(app.controller.document(),&before);assert_eq!(app.controller.coordinator().redo_len(),1);
            drop(pressure);assert_eq!(app.retry_after_undo_recovery().unwrap(),0);
            assert_eq!(app.controller.document().entities.values().filter(|e|e.label=="DupBorder").count(),8);
            app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
        }
        let limits=om9_core::budget::Limits::default();
        let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
        let a=Entity::new("a",Geometry::Surface(surface.clone()),None);let b=Entity::new("b",Geometry::Surface(surface),None);
        let ids=[a.id,b.id];let mut doc=Document::new("stale border");doc.entities.insert(a.id,a);doc.entities.insert(b.id,b);
        let mut app=MatrixApp::new(doc).unwrap();app.controller.submit("Point").unwrap();app.controller.submit("1,2,3").unwrap();
        app.controller.select_objects(&ids).unwrap();let pressure=app.controller.coordinator().reserve_remaining_external_memory_with_held(MemoryClass::Kernel,None,app.controller.scene_cache_usage().unwrap()).unwrap();
        app.transcript.set_draft("DupBorder");app.submit_draft();assert!(app.memory_recovery_open);
        drop(pressure);app.controller.select_objects(&[ids[0]]).unwrap();let before=app.controller.coordinator().snapshot();
        assert!(matches!(app.retry_after_undo_recovery(),Err(Error::Stale(_))));assert_eq!(app.controller.coordinator().snapshot(),before);
    }

    #[test]
    fn dupborder_memory_recovery_reclaims_oldest_history_when_needed() {
        use om9_core::{budget::{Limits,memory::MemoryClass},geometry::{Geometry,Mesh},model::Entity};
        for count in [1,2] {
        let limits=Limits {max_bytes:64*1024*1024,..Default::default()};
        let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
        let a=Entity::new("a",Geometry::Surface(surface.clone()),None);let b=Entity::new("b",Geometry::Surface(surface),None);let ids=[a.id,b.id];
        let old=Entity::new("old mesh",Geometry::Mesh(Mesh {vertices:vec![[1.,1.,1.];400000],faces:vec![],normals:vec![],colors:vec![],corner_uvs:vec![]}),None);
        let old_id=old.id;let mut doc=Document::new("border history");doc.entities.insert(a.id,a);doc.entities.insert(b.id,b);doc.entities.insert(old.id,old);
        let mut app=MatrixApp::new(doc.clone()).unwrap();app.controller=crate::session::Controller::new(doc,limits).unwrap();
        app.controller.delete_objects(&[old_id]).unwrap();app.controller.submit("Point").unwrap();app.controller.submit("9,0,0").unwrap();
        app.controller.select_objects(&ids[..count]).unwrap();let before=app.controller.document().clone();
        let mut pressure=app.controller.coordinator().reserve_remaining_external_memory_with_held(MemoryClass::Kernel,None,app.controller.scene_cache_usage().unwrap()).unwrap();
        pressure.shrink_to(pressure.bytes()-2048).unwrap();app.transcript.set_draft("DupBorder");app.submit_draft();assert!(app.memory_recovery_open);
        assert_eq!(app.retry_after_undo_recovery().unwrap(),1);assert_eq!(app.controller.document().entities.values().filter(|e|e.label=="DupBorder").count(),4*count);
        drop(pressure);app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
        }
    }

    #[test]
    fn dupborder_memory_recovery_requires_consent_in_dialog() {
        use om9_core::{budget::memory::MemoryClass,geometry::Geometry,model::Entity};
        let limits=om9_core::budget::Limits::default();let ctx=egui::Context::default();
        let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
        let a=Entity::new("a",Geometry::Surface(surface.clone()),None);let b=Entity::new("b",Geometry::Surface(surface),None);let ids=[a.id,b.id];
        let mut doc=Document::new("border consent");doc.entities.insert(a.id,a);doc.entities.insert(b.id,b);let mut app=MatrixApp::new(doc).unwrap();
        app.controller.submit("Point").unwrap();app.controller.submit("1,2,3").unwrap();app.controller.select_objects(&ids).unwrap();let before=app.controller.document().clone();
        let pressure=app.controller.coordinator().reserve_remaining_external_memory_with_held(MemoryClass::Kernel,None,app.controller.scene_cache_usage().unwrap()).unwrap();
        app.transcript.set_draft("DupBorder");app.submit_draft();assert!(app.memory_recovery_open);
        let frame=|app:&mut MatrixApp,events| {let mut output=ctx.run_ui(egui::RawInput {screen_rect:Some(Rect::from_min_size(egui::Pos2::ZERO,Vec2::new(900.,600.))),focused:true,events,..Default::default()},|ui|app.memory_recovery_ui(ui.ctx()));output.textures_delta.clear();};
        for _ in 0..3 {frame(&mut app,vec![]);}assert_eq!(app.controller.coordinator().undo_len(),1);assert_eq!(app.controller.document(),&before);
        drop(pressure);let pos=ctx.data(|d|d.get_temp::<Rect>(egui::Id::new("memory-recovery-remove"))).unwrap().center();
        for pressed in [true,false] {frame(&mut app,vec![egui::Event::PointerMoved(pos),egui::Event::PointerButton {pos,button:egui::PointerButton::Primary,pressed,modifiers:Modifiers::NONE}]);}
        assert!(!app.memory_recovery_open);assert_eq!(app.controller.document().entities.values().filter(|e|e.label=="DupBorder").count(),8);
        assert_eq!(app.transcript.draft(),"");app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
    }

    #[test]
    #[cfg(feature = "occt-brep")]
    fn dupborder_brep_retry_reclaims_history_owned_mesh() {
        use om9_core::{budget::{Limits,memory::MemoryClass},geometry::{Geometry,Mesh},model::Entity};
        let limits=Limits {max_bytes:64*1024*1024,..Limits::default()};
        let circle=om9_core::curve_build::circle_on_frame([0.;3],[1.,0.,0.],[0.,1.,0.],2.).unwrap();
        let shell=om9_kernel::extrusion::shell(&circle,[0.,0.,5.],0.1,&limits).unwrap();
        let entity=Entity::new("shell",Geometry::Brep(shell),None);let id=entity.id;
        let old=Entity::new("old mesh",Geometry::Mesh(Mesh {vertices:vec![[1.,1.,1.];400000],faces:vec![],normals:vec![],colors:vec![],corner_uvs:vec![]}),None);let old_id=old.id;
        let mut doc=Document::new("brep history retry");doc.entities.insert(id,entity);doc.entities.insert(old_id,old);
        let mut app=MatrixApp::new(doc.clone()).unwrap();app.controller=crate::session::Controller::new(doc,limits).unwrap();
        app.controller.delete_objects(&[old_id]).unwrap();app.controller.submit("Point").unwrap();app.controller.submit("9,0,0").unwrap();
        app.controller.select_objects(&[id]).unwrap();let before=app.controller.document().clone();
        let mut pressure=app.controller.coordinator().reserve_remaining_external_memory_with_held(MemoryClass::Kernel,None,app.controller.scene_cache_usage().unwrap()).unwrap();
        pressure.shrink_to(pressure.bytes()-1024).unwrap();app.transcript.set_draft("DupBorder");app.submit_draft();assert!(app.memory_recovery_open);
        assert_eq!(app.retry_after_undo_recovery().unwrap(),1);assert_eq!(app.controller.document().entities.values().filter(|e|e.label=="DupBorder").count(),2);
        drop(pressure);app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
    }

    #[test]
    #[cfg(feature = "occt-brep")]
    fn dupedge_pending_pick_memory_refusal_retries_without_losing_redo() {
        use om9_core::{budget::{Limits,memory::MemoryClass},geometry::Geometry,model::Entity};
        use om9_scene::{camera::{Camera,Viewport},scene::Subelement};
        let limits=Limits::default();let shape=om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Box {x_mm:4.,y_mm:5.,z_mm:6.},0.1,&limits).unwrap();
        let segment=shape.display().segments[0];let edge=shape.display().edges[segment[2] as usize];
        let a=shape.display().points[segment[0] as usize];let b=shape.display().points[segment[1] as usize];
        let point=std::array::from_fn(|i|(a[i]+b[i])*0.5);
        let entity=Entity::new("box",Geometry::Brep(shape),None);let id=entity.id;
        let mut doc=Document::with_template_tolerance("dupedge retry");doc.entities.insert(id,entity);
        let mut app=MatrixApp::new(doc).unwrap();app.controller.submit("Point").unwrap();app.controller.submit("100,100,100").unwrap();app.controller.undo().unwrap();
        app.controller.submit("DupEdge").unwrap();
        let reference=crate::session::SelectedSubobject {root:id,path:vec![],subset:Subelement::BrepEdge(edge),
            source:app.controller.document().stamp(id).unwrap(),world_epoch:app.controller.coordinator().epoch()};
        let camera=Camera::orthographic([10.,-10.,14.],[2.,2.,3.],[0.,0.,1.],30.).unwrap();let viewport=Viewport {origin:[0.,0.],size:[400.,400.]};
        let projected=camera.project(point,viewport).unwrap().unwrap();
        app.controller.dupedge_pick_with_point(reference.clone(),point,0.1,&camera,viewport,projected.screen,7.,projected.depth).unwrap();
        let before=app.controller.document().clone();let key=app.controller.dupedge_recovery_key().unwrap();
        let pressure=app.controller.coordinator().reserve_remaining_external_memory_with_held(MemoryClass::Kernel,None,app.controller.scene_cache_usage().unwrap()).unwrap();
        app.transcript.set_draft("");app.submit_draft();
        assert!(app.memory_recovery_open,"{}",app.status_line());assert_eq!(app.controller.dupedge_recovery_key(),Some(key));
        assert_eq!(app.controller.document(),&before);assert_eq!(app.controller.coordinator().redo_len(),1);
        let before_undo=app.controller.coordinator().undo_len();app.transcript.set_draft("changed");
        assert!(matches!(app.retry_after_undo_recovery(),Err(Error::Stale(_))));
        assert_eq!(app.controller.coordinator().undo_len(),before_undo);assert_eq!(app.controller.coordinator().redo_len(),1);
        app.transcript.set_draft("");app.submit_draft();assert!(app.memory_recovery_open);
        drop(pressure);app.controller.dupedge_pick_with_point(reference.clone(),point,0.1,&camera,viewport,projected.screen,7.,projected.depth).unwrap();
        assert!(matches!(app.retry_after_undo_recovery(),Err(Error::Stale(_))));
        assert_eq!(app.controller.coordinator().undo_len(),before_undo);assert_eq!(app.controller.coordinator().redo_len(),1);
        app.controller.dupedge_pick_with_point(reference,point,0.1,&camera,viewport,projected.screen,7.,projected.depth).unwrap();
        let pressure=app.controller.coordinator().reserve_remaining_external_memory_with_held(MemoryClass::Kernel,None,app.controller.scene_cache_usage().unwrap()).unwrap();
        app.submit_draft();assert!(app.memory_recovery_open);drop(pressure);assert_eq!(app.retry_after_undo_recovery().unwrap(),0);
        assert_eq!(app.controller.document().entities.len(),2);app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
    }

    #[test]
    #[cfg(feature = "occt-brep")]
    fn dupedge_preselected_edge_memory_retry_supports_menu_and_command() {
        use om9_core::{budget::{Limits,memory::MemoryClass},geometry::Geometry,model::Entity};
        use om9_scene::{camera::{Camera,Viewport},scene::Subelement};
        let limits=Limits::default();let shape=om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Box {x_mm:4.,y_mm:5.,z_mm:6.},0.1,&limits).unwrap();
        let segment=shape.display().segments[0];let edge=shape.display().edges[segment[2] as usize];
        let a=shape.display().points[segment[0] as usize];let b=shape.display().points[segment[1] as usize];
        let point=std::array::from_fn(|i|(a[i]+b[i])*0.5);
        for input in ["DupEdge","OM9-CURVE-040","menu"] {
            let entity=Entity::new("box",Geometry::Brep(shape.clone()),None);let id=entity.id;
            let mut doc=Document::with_template_tolerance("dupedge preselection retry");doc.entities.insert(id,entity);
            let mut app=MatrixApp::new(doc).unwrap();app.controller.submit("Point").unwrap();app.controller.submit("100,100,100").unwrap();app.controller.undo().unwrap();
            let reference=crate::session::SelectedSubobject {root:id,path:vec![],subset:Subelement::BrepEdge(edge),
                source:app.controller.document().stamp(id).unwrap(),world_epoch:app.controller.coordinator().epoch()};
            let camera=Camera::orthographic([10.,-10.,14.],[2.,2.,3.],[0.,0.,1.],30.).unwrap();let viewport=Viewport {origin:[0.,0.],size:[400.,400.]};
            let projected=camera.project(point,viewport).unwrap().unwrap();
            app.controller.dupedge_pick_with_point(reference,point,0.1,&camera,viewport,projected.screen,7.,projected.depth).unwrap();
            let before=app.controller.document().clone();let pressure=app.controller.coordinator().reserve_remaining_external_memory_with_held(
                MemoryClass::Kernel,None,app.controller.scene_cache_usage().unwrap()).unwrap();
            if input=="menu" {app.invoke("OM9-CURVE-040",InvocationOrigin::Menu);} else {app.transcript.set_draft(input);app.submit_draft();}
            assert!(app.memory_recovery_open,"input={input}: {}",app.status_line());assert_eq!(app.controller.document(),&before);
            assert_eq!(app.controller.coordinator().redo_len(),1);drop(pressure);
            if input=="OM9-CURVE-040" {
                let ctx=egui::Context::default();let frame=|app:&mut MatrixApp,events| {
                    let mut output=ctx.run_ui(egui::RawInput {screen_rect:Some(Rect::from_min_size(egui::Pos2::ZERO,Vec2::new(900.,600.))),focused:true,events,..Default::default()},
                        |ui|app.memory_recovery_ui(ui.ctx()));output.textures_delta.clear();};
                for _ in 0..3 {frame(&mut app,vec![]);}
                let pos=ctx.data(|d|d.get_temp::<Rect>(egui::Id::new("memory-recovery-remove"))).unwrap().center();
                for pressed in [true,false] {frame(&mut app,vec![egui::Event::PointerMoved(pos),egui::Event::PointerButton {pos,button:egui::PointerButton::Primary,pressed,modifiers:Modifiers::NONE}]);}
                assert!(!app.memory_recovery_open);assert_eq!(app.transcript.draft(),"");
            } else {assert_eq!(app.retry_after_undo_recovery().unwrap(),0);}
            assert_eq!(app.controller.document().entities.len(),2);
            app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
        }
    }

    #[test]
    #[cfg(feature = "occt-brep")]
    fn dupedge_temporary_source_edge_memory_retry_keeps_pick() {
        use om9_core::{budget::{Limits,memory::MemoryClass},geometry::Geometry,model::Entity};
        use om9_scene::{camera::{Camera,Viewport},scene::Subelement};
        let limits=Limits::default();let shape=om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Box {x_mm:4.,y_mm:5.,z_mm:6.},0.1,&limits).unwrap();
        let entity=Entity::new("box",Geometry::Brep(shape.clone()),None);let id=entity.id;
        let mut doc=Document::with_template_tolerance("temporary edge retry");doc.entities.insert(id,entity);
        let mut app=MatrixApp::new(doc).unwrap();app.controller.submit("Point").unwrap();app.controller.submit("100,100,100").unwrap();app.controller.undo().unwrap();
        let mut display=shape.display().clone();for id in &mut display.topology.edges {*id=uuid::Uuid::new_v4();}
        let segment=display.segments[0];let edge=display.edges[segment[2] as usize];
        let a=display.points[segment[0] as usize];let b=display.points[segment[1] as usize];let point=std::array::from_fn(|i|(a[i]+b[i])*0.5);
        let fresh=om9_core::brep::Brep::from_native(shape.source().to_owned(),display,&limits).unwrap();
        app.controller.publish_paired_brep_for_test(id,&fresh).unwrap();app.controller.submit("DupEdge").unwrap();
        let reference=crate::session::SelectedSubobject {root:id,path:vec![],subset:Subelement::BrepEdge(edge),
            source:app.controller.document().stamp(id).unwrap(),world_epoch:app.controller.coordinator().epoch()};
        let camera=Camera::orthographic([10.,-10.,14.],[2.,2.,3.],[0.,0.,1.],30.).unwrap();let viewport=Viewport {origin:[0.,0.],size:[400.,400.]};
        let projected=camera.project(point,viewport).unwrap().unwrap();
        app.controller.dupedge_pick_with_point(reference,point,0.1,&camera,viewport,projected.screen,7.,projected.depth).unwrap();
        let before=app.controller.document().clone();let pressure=app.controller.coordinator().reserve_remaining_external_memory_with_held(
            MemoryClass::Kernel,None,app.controller.scene_cache_usage().unwrap()).unwrap();
        app.transcript.set_draft("");app.submit_draft();assert!(app.memory_recovery_open,"{}",app.status_line());
        assert_eq!(app.controller.document(),&before);assert_eq!(app.controller.coordinator().redo_len(),1);
        assert_eq!(app.controller.dupedge_temporary_picks().count(),1);drop(pressure);
        assert_eq!(app.retry_after_undo_recovery().unwrap(),0);assert_eq!(app.controller.document().entities.len(),2);
        app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
    }

    #[test]
    #[cfg(feature = "occt-brep")]
    fn reopened_dupedge_source_view_reclaims_undo_and_retries_after_consent() {
        use om9_core::{budget::{Limits,memory::MemoryClass},geometry::{Geometry,Mesh},model::Entity,transaction::Cancellation};
        use om9_scene::{camera::{Camera,Viewport},scene::Subelement};
        let limits=Limits {max_bytes:64*1024*1024,..Limits::default()};
        let shape=om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Box {x_mm:4.,y_mm:5.,z_mm:6.},0.1,&limits).unwrap();
        let entity=Entity::new("box",Geometry::Brep(shape),None);let id=entity.id;
        let old=Entity::new("old mesh",Geometry::Mesh(Mesh {vertices:vec![[1.,1.,1.];400000],faces:vec![],normals:vec![],colors:vec![],corner_uvs:vec![]}),None);let old_id=old.id;
        let mut doc=Document::with_template_tolerance("reopened source recovery");doc.entities.insert(id,entity);
        let path=std::env::temp_dir().join(format!("om9-dupedge-recovery-{}.om9",uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&path,&doc,&limits,&Cancellation::default()).unwrap();
        let reopened=crate::session::Controller::from_file(&path,limits).unwrap();std::fs::remove_file(path).unwrap();
        let mut resumed=reopened.document().clone();drop(reopened);resumed.entities.insert(old_id,old);
        let mut app=MatrixApp::new(resumed.clone()).unwrap();app.controller=crate::session::Controller::new(resumed,limits).unwrap();
        app.controller.delete_objects(&[old_id]).unwrap();app.controller.submit("Point").unwrap();app.controller.submit("100,100,100").unwrap();app.controller.undo().unwrap();
        app.controller.submit("DupEdge").unwrap();let original=app.controller.scene().unwrap().publication_id();
        app.controller.request_dupedge_source_view(id).unwrap();let deadline=std::time::Instant::now()+std::time::Duration::from_secs(20);
        loop {if let Some(result)=app.controller.poll_dupedge_source_view() {result.unwrap();break;}
            assert!(std::time::Instant::now()<deadline,"source view timeout");std::thread::yield_now();}
        assert_ne!(app.controller.scene().unwrap().publication_id(),original);
        let stamp=app.controller.document().stamp(id).unwrap();
        let display=app.controller.scene().unwrap().root_brep_pick_display(&stamp).unwrap();
        let segment=display.segments[0];let edge=display.edges[segment[2] as usize];
        let a=display.points[segment[0] as usize];let b=display.points[segment[1] as usize];let point=std::array::from_fn(|i|(a[i]+b[i])*0.5);
        let reference=crate::session::SelectedSubobject {root:id,path:vec![],subset:Subelement::BrepEdge(edge),source:stamp,world_epoch:app.controller.coordinator().epoch()};
        let camera=Camera::orthographic([10.,-10.,14.],[2.,2.,3.],[0.,0.,1.],30.).unwrap();let viewport=Viewport {origin:[0.,0.],size:[400.,400.]};
        let projected=camera.project(point,viewport).unwrap().unwrap();
        app.controller.dupedge_pick_with_point(reference,point,0.1,&camera,viewport,projected.screen,7.,projected.depth).unwrap();
        assert_eq!(app.controller.dupedge_temporary_picks().count(),1);let before=app.controller.document().clone();
        let mut pressure=app.controller.coordinator().reserve_remaining_external_memory_with_held(MemoryClass::Kernel,None,app.controller.scene_cache_usage().unwrap()).unwrap();
        pressure.shrink_to(pressure.bytes()-1024).unwrap();app.transcript.set_draft("");app.submit_draft();
        assert!(app.memory_recovery_open,"{}",app.status_line());assert_eq!(app.controller.document(),&before);
        assert_eq!(app.controller.coordinator().redo_len(),1);assert_eq!(app.controller.dupedge_temporary_picks().count(),1);
        let ctx=egui::Context::default();let frame=|app:&mut MatrixApp,events| {let mut output=ctx.run_ui(egui::RawInput {
            screen_rect:Some(Rect::from_min_size(egui::Pos2::ZERO,Vec2::new(900.,600.))),focused:true,events,..Default::default()},
            |ui|app.memory_recovery_ui(ui.ctx()));output.textures_delta.clear();};
        for _ in 0..3 {frame(&mut app,vec![]);}assert_eq!(app.controller.coordinator().undo_len(),1);
        let pos=ctx.data(|d|d.get_temp::<Rect>(egui::Id::new("memory-recovery-remove"))).unwrap().center();
        for pressed in [true,false] {frame(&mut app,vec![egui::Event::PointerMoved(pos),egui::Event::PointerButton {pos,button:egui::PointerButton::Primary,pressed,modifiers:Modifiers::NONE}]);}
        for _ in 0..4 {if app.memory_recovery_open {frame(&mut app,vec![]);}}
        assert!(!app.memory_recovery_open,"{}",app.status_line());assert_eq!(app.memory_recovery_removed,1,"{}",app.status_line());
        assert_eq!(app.controller.document().entities.len(),2,"{}",app.status_line());
        let output=app.controller.document().entities.values().find(|entity|entity.label=="DupEdge").unwrap();
        let Geometry::Curve(curve)=&output.geometry else {panic!("DupEdge must create a curve")};
        let ends=[0.,1.].map(|t|om9_core::evaluation::evaluate_curve(curve,om9_core::evaluation::Parameter::Fraction(t),0,output.placement,&limits).unwrap()[0]);
        let direction=std::array::from_fn::<_,3,_>(|i|ends[1][i]-ends[0][i]);let length2=direction.iter().map(|v|v*v).sum::<f64>();
        let t=(0..3).map(|i|(point[i]-ends[0][i])*direction[i]).sum::<f64>()/length2;
        assert!((0.0..=1.0).contains(&t));assert!((0..3).map(|i|(point[i]-ends[0][i]-t*direction[i]).powi(2)).sum::<f64>().sqrt()<1e-6);
        drop(pressure);
        app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);app.controller.redo().unwrap();assert_eq!(app.controller.document().entities.len(),2);
    }

    #[test]
    #[cfg(feature = "occt-brep")]
    fn dupedge_reclaims_oldest_undo_after_consent_and_retries() {
        use om9_core::{budget::{Limits,memory::MemoryClass},geometry::{Geometry,Mesh},model::Entity};
        use om9_scene::{camera::{Camera,Viewport},scene::Subelement};
        let limits=Limits {max_bytes:64*1024*1024,..Limits::default()};
        let shape=om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Box {x_mm:4.,y_mm:5.,z_mm:6.},0.1,&limits).unwrap();
        let segment=shape.display().segments[0];let edge=shape.display().edges[segment[2] as usize];
        let a=shape.display().points[segment[0] as usize];let b=shape.display().points[segment[1] as usize];let point=std::array::from_fn(|i|(a[i]+b[i])*0.5);
        let entity=Entity::new("box",Geometry::Brep(shape),None);let id=entity.id;
        let old=Entity::new("old mesh",Geometry::Mesh(Mesh {vertices:vec![[1.,1.,1.];400000],faces:vec![],normals:vec![],colors:vec![],corner_uvs:vec![]}),None);let old_id=old.id;
        let mut doc=Document::with_template_tolerance("dupedge history retry");doc.entities.insert(id,entity);doc.entities.insert(old_id,old);
        let mut app=MatrixApp::new(doc.clone()).unwrap();app.controller=crate::session::Controller::new(doc,limits).unwrap();
        app.controller.delete_objects(&[old_id]).unwrap();app.controller.submit("Point").unwrap();app.controller.submit("100,100,100").unwrap();app.controller.undo().unwrap();
        app.controller.submit("DupEdge").unwrap();let reference=crate::session::SelectedSubobject {root:id,path:vec![],subset:Subelement::BrepEdge(edge),
            source:app.controller.document().stamp(id).unwrap(),world_epoch:app.controller.coordinator().epoch()};
        let camera=Camera::orthographic([10.,-10.,14.],[2.,2.,3.],[0.,0.,1.],30.).unwrap();let viewport=Viewport {origin:[0.,0.],size:[400.,400.]};
        let projected=camera.project(point,viewport).unwrap().unwrap();
        app.controller.dupedge_pick_with_point(reference,point,0.1,&camera,viewport,projected.screen,7.,projected.depth).unwrap();
        let before=app.controller.document().clone();let mut pressure=app.controller.coordinator().reserve_remaining_external_memory_with_held(
            MemoryClass::Kernel,None,app.controller.scene_cache_usage().unwrap()).unwrap();
        pressure.shrink_to(pressure.bytes()-1024).unwrap();app.transcript.set_draft("");app.submit_draft();
        assert!(app.memory_recovery_open,"{}",app.status_line());assert_eq!(app.controller.document(),&before);assert_eq!(app.controller.coordinator().redo_len(),1);
        let ctx=egui::Context::default();let frame=|app:&mut MatrixApp,events| {
            let mut output=ctx.run_ui(egui::RawInput {screen_rect:Some(Rect::from_min_size(egui::Pos2::ZERO,Vec2::new(900.,600.))),focused:true,events,..Default::default()},
                |ui|app.memory_recovery_ui(ui.ctx()));output.textures_delta.clear();};
        for _ in 0..3 {frame(&mut app,vec![]);}assert_eq!(app.controller.coordinator().undo_len(),1);
        let pos=ctx.data(|d|d.get_temp::<Rect>(egui::Id::new("memory-recovery-remove"))).unwrap().center();
        for pressed in [true,false] {frame(&mut app,vec![egui::Event::PointerMoved(pos),egui::Event::PointerButton {pos,button:egui::PointerButton::Primary,pressed,modifiers:Modifiers::NONE}]);}
        for _ in 0..3 {if app.memory_recovery_open {frame(&mut app,vec![]);}}
        assert!(!app.memory_recovery_open);assert_eq!(app.memory_recovery_removed,1);assert_eq!(app.controller.document().entities.len(),2);
        drop(pressure);app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
    }

    #[test]
    #[cfg(feature = "occt-brep")]
    fn dupborder_brep_memory_refusal_offers_retry() {
        use om9_core::{budget::{Limits,memory::MemoryClass},geometry::Geometry,model::Entity};
        let limits=Limits {max_bytes:64*1024*1024,..Limits::default()};
        let circle=om9_core::curve_build::circle_on_frame([0.;3],[1.,0.,0.],[0.,1.,0.],2.).unwrap();
        let shell=om9_kernel::extrusion::shell(&circle,[0.,0.,5.],0.1,&limits).unwrap();
        let entity=Entity::new("shell",Geometry::Brep(shell),None);let id=entity.id;
        let mut doc=Document::new("brep retry");doc.entities.insert(id,entity);
        for menu in [false,true] {
            let mut app=MatrixApp::new(doc.clone()).unwrap();app.controller=crate::session::Controller::new(doc.clone(),limits).unwrap();
            app.controller.submit("Point").unwrap();app.controller.submit("1,2,3").unwrap();app.controller.undo().unwrap();app.controller.select_objects(&[id]).unwrap();let before=app.controller.document().clone();
            let mut pressure=app.controller.coordinator().reserve_remaining_external_memory_with_held(MemoryClass::Kernel,None,app.controller.scene_cache_usage().unwrap()).unwrap();
            pressure.shrink_to(pressure.bytes()-1024).unwrap();
            if menu {app.invoke("OM9-CURVE-039",InvocationOrigin::Menu);} else {app.transcript.set_draft("DupBorder");app.submit_draft();}
            assert!(app.memory_recovery_open);assert_eq!(app.controller.document(),&before);assert_eq!(app.controller.coordinator().redo_len(),1);
            drop(pressure);assert_eq!(app.retry_after_undo_recovery().unwrap(),0);
            assert_eq!(app.controller.document().entities.values().filter(|e|e.label=="DupBorder").count(),2);
            app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
        }
    }

    #[test]
    fn memory_recovery_refuses_changed_selection_before_removing_undo() {
        let mut app = MatrixApp::new(Document::new("stale recovery")).unwrap();
        app.controller.submit("Point").unwrap(); app.controller.submit("1,0,0").unwrap();
        let id = *app.controller.document().entities.keys().next().unwrap();
        app.controller.select_objects(&[id]).unwrap(); app.offer_wire_memory_recovery("ExtractWireframe");
        app.controller.select_objects(&[]).unwrap();
        let before = app.controller.coordinator().snapshot();
        assert!(matches!(app.retry_after_undo_recovery(), Err(Error::Stale(_))));
        assert_eq!(app.controller.coordinator().snapshot(), before);
    }
    #[test]
    fn recovery_refuses_selection_changed_away_and_back_before_pruning() {
        use om9_core::{budget::memory::MemoryClass,geometry::Geometry,model::Entity};
        for isocurve in [false,true] {
            let mut doc=Document::new("selection changed and restored");
            let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&Default::default()).unwrap();
            let entity=Entity::new("surface",Geometry::Surface(surface),None);let id=entity.id;doc.entities.insert(id,entity);
            let mut app=MatrixApp::new(doc).unwrap();
            app.controller.submit("Point").unwrap();app.controller.submit("1,2,3").unwrap();
            app.controller.submit("Point").unwrap();app.controller.submit("4,5,6").unwrap();app.controller.undo().unwrap();
            app.controller.select_objects(&[id]).unwrap();let revision=app.controller.selection_state().revision;
            if isocurve {app.offer_isocurve_start_memory_recovery();} else {app.offer_wire_memory_recovery("ExtractWireframe");}
            app.controller.select_objects(&[]).unwrap();app.controller.select_objects(&[id]).unwrap();
            assert_ne!(app.controller.selection_state().revision,revision);
            let before=app.controller.coordinator().snapshot();let undo=app.controller.coordinator().undo_len();let redo=app.controller.coordinator().redo_len();assert_eq!(redo,1);
            let _pressure=app.controller.coordinator().reserve_remaining_external_memory_with_held(
                MemoryClass::Kernel,None,app.controller.scene_cache_usage().unwrap()).unwrap();
            assert!(matches!(app.retry_undo_recovery_step(),Err(Error::Stale(_))),"isocurve={isocurve}");
            assert_eq!(app.controller.coordinator().snapshot(),before);assert_eq!(app.controller.coordinator().undo_len(),undo);assert_eq!(app.controller.coordinator().redo_len(),redo);
        }
    }
    #[test]
    fn wire_recovery_refuses_changed_draft_before_removing_undo() {
        use om9_core::{budget::memory::MemoryClass,geometry::{Geometry,Mesh},model::Entity};
        let mut doc=Document::new("changed wire draft");
        let mesh=Mesh {vertices:vec![[0.,0.,0.],[2.,0.,0.],[0.,2.,0.]],faces:vec![vec![0,1,2]],normals:vec![],colors:vec![],corner_uvs:vec![]};
        let entity=Entity::new("mesh",Geometry::Mesh(mesh),None);let id=entity.id;doc.entities.insert(id,entity);
        let mut app=MatrixApp::new(doc).unwrap();
        app.controller.submit("Point").unwrap();app.controller.submit("1,0,0").unwrap();
        app.controller.select_objects(&[id]).unwrap();
        app.transcript.set_draft("ExtractWireframe");app.offer_wire_memory_recovery("ExtractWireframe");
        app.transcript.set_draft("Box");
        let before=app.controller.coordinator().snapshot();let undo=app.controller.coordinator().undo_len();
        let _pressure=app.controller.coordinator().reserve_remaining_external_memory_with_held(
            MemoryClass::Kernel,None,app.controller.scene_cache_usage().unwrap()).unwrap();
        assert!(matches!(app.retry_undo_recovery_step(),Err(Error::Stale(_))));
        assert_eq!(app.controller.coordinator().snapshot(),before);
        assert_eq!(app.controller.coordinator().undo_len(),undo);
        assert_eq!(app.transcript.draft(),"Box");
    }
    #[test]
    fn extractwire_menu_offers_recovery_and_retries_without_unneeded_deletion() {
        use om9_core::{budget::memory::MemoryClass, geometry::{Geometry, Mesh}, model::Entity};
        let mut doc = Document::new("menu recovery");
        let mesh = Mesh { vertices: vec![[0.,0.,0.], [2.,0.,0.], [0.,2.,0.]], faces: vec![vec![0,1,2]], normals: vec![], colors: vec![], corner_uvs: vec![] };
        let entity = Entity::new("mesh", Geometry::Mesh(mesh), None); let id = entity.id; doc.entities.insert(id, entity);
        let mut app = MatrixApp::new(doc).unwrap();
        app.controller.submit("Point").unwrap(); app.controller.submit("1,0,0").unwrap();
        app.controller.select_objects(&[id]).unwrap(); let before = app.controller.document().clone();
        let pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
            MemoryClass::Kernel, None, app.controller.scene_cache_usage().unwrap()).unwrap();
        app.invoke("OM9-CURVE-054", InvocationOrigin::Menu);
        assert!(app.memory_recovery_open);
        assert_eq!(app.controller.document(), &before);
        drop(pressure);
        assert_eq!(app.retry_after_undo_recovery().unwrap(), 0);
        assert_eq!(app.controller.coordinator().undo_len(), 2);
        app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
    }
    #[cfg(feature = "occt-brep")]
    #[test]
    fn extractwire_brep_retry_reclaims_history_owned_mesh() {
        use om9_core::{budget::{Limits,memory::MemoryClass},geometry::{Geometry,Mesh},model::Entity};
        let limits=Limits {max_bytes:64*1024*1024,..Limits::default()};
        let shape=om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Box {x_mm:4.,y_mm:5.,z_mm:6.},0.1,&limits).unwrap();
        let entity=Entity::new("box",Geometry::Brep(shape),None);let id=entity.id;
        let old=Entity::new("old mesh",Geometry::Mesh(Mesh {vertices:vec![[1.,1.,1.];400000],faces:vec![],normals:vec![],colors:vec![],corner_uvs:vec![]}),None);let old_id=old.id;
        let mut doc=Document::new("wire history retry");doc.entities.insert(id,entity);doc.entities.insert(old_id,old);
        let mut app=MatrixApp::new(doc.clone()).unwrap();app.controller=crate::session::Controller::new(doc,limits).unwrap();
        app.controller.delete_objects(&[old_id]).unwrap();app.controller.submit("Point").unwrap();app.controller.submit("9,0,0").unwrap();
        app.controller.select_objects(&[id]).unwrap();let before=app.controller.document().clone();
        let mut pressure=app.controller.coordinator().reserve_remaining_external_memory_with_held(MemoryClass::Kernel,None,app.controller.scene_cache_usage().unwrap()).unwrap();
        pressure.shrink_to(pressure.bytes()-1024).unwrap();app.transcript.set_draft("ExtractWireframe");app.submit_draft();assert!(app.memory_recovery_open);
        assert_eq!(app.retry_after_undo_recovery().unwrap(),1);assert_eq!(app.controller.document().entities.values().filter(|e|e.label=="ExtractWire").count(),12);
        drop(pressure);app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn mixed_extractwire_memory_recovery_keeps_selection_and_undo() {
        use om9_core::{budget::{Limits, memory::MemoryClass}, geometry::{Geometry, Mesh}, model::Entity};
        let limits = Limits::default();
        let mut doc = Document::new("mixed recovery");
        let mesh = Entity::new("mesh", Geometry::Mesh(Mesh { vertices: vec![[0.,0.,0.], [2.,0.,0.], [0.,2.,0.]],
            faces: vec![vec![0,1,2]], normals: vec![], colors: vec![], corner_uvs: vec![] }), None);
        let brep = Entity::new("box", Geometry::Brep(om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Box { x_mm: 4., y_mm: 5., z_mm: 6. }, 0.1, &limits).unwrap()), None);
        let ids = [mesh.id, brep.id];
        for entity in [mesh, brep] { doc.entities.insert(entity.id, entity); }
        let mut app = MatrixApp::new(doc).unwrap();
        app.controller.submit("Point").unwrap(); app.controller.submit("1,0,0").unwrap();
        app.controller.select_objects(&ids).unwrap(); let before = app.controller.document().clone();
        let pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
            MemoryClass::Kernel, None, app.controller.scene_cache_usage().unwrap()).unwrap();
        app.invoke("OM9-CURVE-054", InvocationOrigin::Menu);
        assert!(app.memory_recovery_open); assert_eq!(app.controller.document(), &before);
        drop(pressure); assert_eq!(app.retry_after_undo_recovery().unwrap(), 0);
        assert_eq!(app.controller.document().entities.values().filter(|e| e.label == "ExtractWire").count(), 15);
        app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
    }

    #[test]
    fn trimmed_isocurve_memory_recovery_preserves_input_and_retries() {
        use om9_core::{budget::memory::MemoryClass, geometry::Geometry, model::Entity,
            rhino_semantics::{set_trimmed_surface, TrimmedSurface}};
        let limits = Default::default();
        let surface = om9_core::surface_kernel::plane([0.,0.,0.], [1.,0.,0.], [0.,1.,0.], 10., 10., &limits).unwrap();
        let trim = TrimmedSurface { underlying: surface.clone(), outer: vec![[0.2,0.2],[0.8,0.2],[0.8,0.8],[0.2,0.8]],
            inners: vec![vec![[0.4,0.4],[0.6,0.4],[0.6,0.6],[0.4,0.6]]], reversed: false };
        let mut entity = Entity::new("trimmed", Geometry::Surface(surface), None);
        set_trimmed_surface(&mut entity, &trim, &limits).unwrap(); let id = entity.id;
        let mut doc = Document::new("trimmed isocurve memory"); doc.entities.insert(id, entity);
        let mut app = MatrixApp::new(doc).unwrap();
        app.controller.submit("Point").unwrap(); app.controller.submit("1,2,3").unwrap();
        app.controller.select_objects(&[id]).unwrap(); app.controller.submit("ExtractIsocurve").unwrap(); app.controller.submit("U=0.5").unwrap();
        let key = app.controller.isocurve_recovery_key().unwrap(); let before = app.controller.document().clone();
        let pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
            MemoryClass::Kernel, None, app.controller.scene_cache_usage().unwrap()).unwrap();
        app.transcript.set_draft(""); app.submit_draft();
        assert!(app.memory_recovery_open); assert_eq!(app.controller.isocurve_recovery_key(), Some(key));
        assert_eq!(app.controller.document(), &before); drop(pressure);
        assert_eq!(app.retry_after_undo_recovery().unwrap(), 0);
        assert_eq!(app.controller.document().entities.values().filter(|e| e.label == "ExtractIsocurve").count(), 2);
        app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
    }

    #[test]
    fn isocurve_start_recovery_covers_command_and_menu_and_preserves_redo() {
        use om9_core::{budget::memory::MemoryClass, geometry::Geometry, model::Entity};
        for menu in [false,true] {
            let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&Default::default()).unwrap();
            let entity=Entity::new("surface",Geometry::Surface(surface),None);let id=entity.id;
            let mut doc=Document::new("start recovery");doc.entities.insert(id,entity);let mut app=MatrixApp::new(doc).unwrap();
            for point in ["1,0,0","2,0,0"] {app.controller.submit("Point").unwrap();app.controller.submit(point).unwrap();}
            app.controller.undo().unwrap();app.controller.select_objects(&[id]).unwrap();let before=app.controller.document().clone();
            let pressure=app.controller.coordinator().reserve_remaining_external_memory_with_held(MemoryClass::Kernel,None,app.controller.scene_cache_usage().unwrap()).unwrap();
            if menu {app.invoke("OM9-CURVE-019",InvocationOrigin::Menu);} else {app.transcript.set_draft("ExtractIsocurve");app.submit_draft();}
            assert!(app.memory_recovery_open);assert!(app.controller.isocurve_recovery_key().is_none());
            assert_eq!(app.controller.document(),&before);assert_eq!(app.controller.coordinator().redo_len(),1);
            drop(pressure);assert_eq!(app.retry_after_undo_recovery().unwrap(),0);
            assert!(app.controller.isocurve_recovery_key().is_some());assert_eq!(app.controller.document(),&before);
            assert_eq!(app.controller.coordinator().redo_len(),1);app.controller.submit("").unwrap();
            app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
        }
    }

    #[test]
    fn isocurve_start_recovery_waits_for_click_and_opens_pending_prompt() {
        use om9_core::{budget::memory::MemoryClass,geometry::Geometry,model::Entity};
        let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&Default::default()).unwrap();
        let entity=Entity::new("surface",Geometry::Surface(surface),None);let id=entity.id;
        let mut doc=Document::new("start click");doc.entities.insert(id,entity);let mut app=MatrixApp::new(doc).unwrap();
        for p in ["1,0,0","2,0,0"] {app.controller.submit("Point").unwrap();app.controller.submit(p).unwrap();}
        app.controller.undo().unwrap();app.controller.select_objects(&[id]).unwrap();let before=app.controller.document().clone();
        let pressure=app.controller.coordinator().reserve_remaining_external_memory_with_held(MemoryClass::Kernel,None,app.controller.scene_cache_usage().unwrap()).unwrap();
        app.transcript.set_draft("ExtractIsocurve");app.submit_draft();assert!(app.memory_recovery_open);
        let ctx=egui::Context::default();let frame=|app:&mut MatrixApp,events| {
            let mut out=ctx.run_ui(egui::RawInput {screen_rect:Some(Rect::from_min_size(egui::Pos2::ZERO,Vec2::new(900.,600.))),focused:true,events,..Default::default()},|ui|app.memory_recovery_ui(ui.ctx()));out.textures_delta.clear();
        };
        for _ in 0..3 {frame(&mut app,vec![]);}assert_eq!((app.controller.coordinator().undo_len(),app.controller.coordinator().redo_len()),(1,1));
        assert!(app.controller.isocurve_recovery_key().is_none());drop(pressure);
        let pos=ctx.data(|d|d.get_temp::<Rect>(egui::Id::new("memory-recovery-remove"))).unwrap().center();
        for pressed in [true,false] {frame(&mut app,vec![egui::Event::PointerMoved(pos),egui::Event::PointerButton {pos,button:egui::PointerButton::Primary,pressed,modifiers:Modifiers::NONE}]);}
        assert!(!app.memory_recovery_open);assert!(app.controller.isocurve_recovery_key().is_some());assert_eq!(app.transcript.draft(),"");
        assert_eq!(app.controller.document(),&before);assert_eq!((app.controller.coordinator().undo_len(),app.controller.coordinator().redo_len()),(1,1));
    }

    #[test]
    fn isocurve_start_recovery_refuses_changed_draft_or_selection() {
        use om9_core::{geometry::Geometry,model::Entity};
        for change_draft in [false,true] {
            let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&Default::default()).unwrap();
            let entity=Entity::new("surface",Geometry::Surface(surface),None);let id=entity.id;
            let mut doc=Document::new("start stale");doc.entities.insert(id,entity);let mut app=MatrixApp::new(doc).unwrap();
            app.controller.submit("Point").unwrap();app.controller.submit("1,2,3").unwrap();app.controller.select_objects(&[id]).unwrap();
            app.transcript.set_draft("ExtractIsocurve");app.offer_isocurve_start_memory_recovery();
            if change_draft {app.transcript.set_draft("other");} else {app.controller.select_objects(&[]).unwrap();}
            let before=app.controller.coordinator().snapshot();
            assert!(matches!(app.retry_after_undo_recovery(),Err(Error::Stale(_))));assert_eq!(app.controller.coordinator().snapshot(),before);
        }
    }

    #[cfg(feature="occt-brep")]
    #[test]
    fn isocurve_recovery_refuses_changed_face_before_removing_undo() {
        use om9_core::{budget::Limits,geometry::Geometry,model::Entity};
        use om9_scene::scene::Subelement;
        for pending in [false,true] {
            let limits=Limits::default();let shape=om9_kernel::brep::box_brep([4.,6.,8.],0.1,&limits).unwrap();
            let faces=shape.topology().faces.clone();let entity=Entity::new("box",Geometry::Brep(shape),None);let id=entity.id;
            let mut doc=Document::with_template_tolerance("face recovery");doc.entities.insert(id,entity);let mut app=MatrixApp::new(doc).unwrap();
            for p in ["1,0,0","2,0,0"] {app.controller.submit("Point").unwrap();app.controller.submit(p).unwrap();}
            app.controller.undo().unwrap();app.controller.select_objects(&[id]).unwrap();
            app.controller.select_subobject(id,Subelement::BrepFace(faces[0])).unwrap();app.transcript.set_draft("ExtractIsocurve");
            if pending {
                app.controller.submit("ExtractIsocurve").unwrap();
                let deadline=std::time::Instant::now()+std::time::Duration::from_secs(30);
                loop {if let Some(result)=app.controller.poll_face_verification(){result.unwrap();break;}
                    assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(2));}
                let key=app.controller.isocurve_recovery_key().unwrap();app.offer_isocurve_memory_recovery(key);
            }
            else {app.offer_isocurve_start_memory_recovery();}
            app.controller.clear_subselection();app.controller.select_subobject(id,Subelement::BrepFace(faces[1])).unwrap();
            let before=app.controller.coordinator().snapshot();let undo=app.controller.coordinator().undo_len();let redo=app.controller.coordinator().redo_len();
            assert!(matches!(app.retry_after_undo_recovery(),Err(Error::Stale(_))));
            assert_eq!(app.controller.coordinator().snapshot(),before);assert_eq!(app.controller.coordinator().undo_len(),undo);assert_eq!(app.controller.coordinator().redo_len(),redo);
        }
    }

    #[cfg(feature="occt-brep")]
    #[test]
    fn isocurve_build_recovery_waits_retries_and_preserves_redo() {
        use om9_core::{budget::{Limits,memory::MemoryClass},geometry::Geometry,model::Entity};
        use om9_scene::scene::Subelement;
        let limits=Limits::default();let shape=om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Sphere {radius_mm:3.},0.1,&limits).unwrap();
        let face=shape.topology().faces[0];let entity=Entity::new("sphere",Geometry::Brep(shape),None);let id=entity.id;
        let mut doc=Document::with_template_tolerance("isocurve build recovery");doc.entities.insert(id,entity);let mut app=MatrixApp::new(doc).unwrap();
        for p in ["1,0,0","2,0,0"] {app.controller.submit("Point").unwrap();app.controller.submit(p).unwrap();}
        app.controller.undo().unwrap();app.controller.select_objects(&[id]).unwrap();app.controller.select_subobject(id,Subelement::BrepFace(face)).unwrap();
        app.controller.submit("ExtractIsocurve").unwrap();let deadline=std::time::Instant::now()+std::time::Duration::from_secs(30);
        loop {if let Some(result)=app.controller.poll_face_verification(){result.unwrap();break;}
            assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(2));}
        let before=app.controller.document().clone();let key=app.controller.isocurve_recovery_key().unwrap();
        app.transcript.set_draft("");app.offer_isocurve_memory_recovery(key);app.memory_recovery_running=true;
        assert!(app.retry_undo_recovery_step().unwrap());
        app.transcript.set_draft("changed");
        assert!(!app.isocurve_build_recovery_context_valid());
        app.finish_isocurve_build_recovery(Err(Error::Budget("bytes")));
        assert!(!app.memory_recovery_open);
        assert_eq!(app.controller.document(),&before);
        assert_eq!(app.controller.coordinator().undo_len(),1);
        assert_eq!(app.controller.coordinator().redo_len(),1);
        while app.controller.isocurve_build_worker_active() {
            assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(2));
        }
        app.transcript.set_draft("");app.offer_isocurve_memory_recovery(key);app.memory_recovery_running=true;
        for attempt in 0..2 {
            assert!(app.retry_undo_recovery_step().unwrap());assert!(app.isocurve_build_recovery_waiting());
            let pressure=app.controller.coordinator().reserve_remaining_external_memory_with_held(MemoryClass::Kernel,None,app.controller.scene_cache_usage().unwrap()).unwrap();
            let result=loop {if let Some(result)=app.controller.poll_isocurve_build_worker(){break result;}
                assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(2));};
            assert!(matches!(&result,Err(Error::Budget(_))),"{result:?}");drop(pressure);
            app.finish_isocurve_build_recovery(result);
            assert!(app.memory_recovery_open && app.memory_recovery_running);assert_eq!(app.controller.document(),&before);
            assert_eq!(app.controller.coordinator().redo_len(),1);assert_eq!(app.controller.coordinator().undo_len(),if attempt==0 {1}else{0});
        }
        assert!(app.retry_undo_recovery_step().unwrap());assert!(app.isocurve_build_recovery_waiting());
        let result=loop {if let Some(result)=app.controller.poll_isocurve_build_worker(){break result;}
            assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(2));};
        result.as_ref().unwrap();app.finish_isocurve_build_recovery(result);
        assert!(!app.memory_recovery_open);
        assert_eq!(app.controller.document().entities.values().filter(|entity|entity.label=="ExtractIsocurve").count(),1);
    }

    #[cfg(feature="occt-brep")]
    #[test]
    fn isocurve_worker_recovery_uses_one_consent_and_retries_before_pruning() {
        use om9_core::{budget::{Limits,memory::MemoryClass},geometry::Geometry,model::Entity};
        use om9_scene::scene::Subelement;
        let limits=Limits::default();let shape=om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Sphere {radius_mm:3.},0.01,&limits).unwrap();
        assert!(shape.display().triangles.len()>512);let face=shape.topology().faces[0];
        let entity=Entity::new("sphere",Geometry::Brep(shape),None);let id=entity.id;
        let mut doc=Document::with_template_tolerance("isocurve worker recovery");doc.entities.insert(id,entity);let mut app=MatrixApp::new(doc).unwrap();
        for p in ["1,0,0","2,0,0"] {app.controller.submit("Point").unwrap();app.controller.submit(p).unwrap();}
        app.controller.undo().unwrap();app.controller.select_objects(&[id]).unwrap();
        app.controller.select_subobject(id,Subelement::BrepFace(face)).unwrap();let before=app.controller.document().clone();
        app.transcript.set_draft("ExtractIsocurve");app.offer_isocurve_start_memory_recovery();app.memory_recovery_running=true;
        for attempt in 0..2 {
            assert!(app.retry_undo_recovery_step().unwrap());assert!(app.isocurve_recovery_waiting());
            let pressure=app.controller.coordinator().reserve_remaining_external_memory_with_held(
                MemoryClass::Kernel,None,app.controller.scene_cache_usage().unwrap()).unwrap();
            let deadline=std::time::Instant::now()+std::time::Duration::from_secs(30);
            let result=loop {if let Some(result)=app.controller.poll_face_verification() {break result;}
                assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(2));};
            assert!(matches!(&result,Err(Error::Budget(_))),"{result:?}");drop(pressure);
            app.finish_isocurve_face_recovery(result);assert!(app.memory_recovery_open && app.memory_recovery_running);
            assert_eq!(app.controller.document(),&before);assert_eq!(app.controller.coordinator().redo_len(),1);
            assert_eq!(app.controller.coordinator().undo_len(),if attempt==0 {1} else {0});
        }
        assert!(app.retry_undo_recovery_step().unwrap());assert!(app.isocurve_recovery_waiting());
        let deadline=std::time::Instant::now()+std::time::Duration::from_secs(30);
        let result=loop {if let Some(result)=app.controller.poll_face_verification() {break result;}
            assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(2));};
        result.as_ref().unwrap();app.finish_isocurve_face_recovery(result);
        assert!(!app.memory_recovery_open);assert!(!app.controller.isocurve_option_tokens().is_empty());
        assert_eq!(app.controller.document(),&before);assert_eq!(app.controller.coordinator().redo_len(),1);
    }

    #[cfg(feature="occt-brep")]
    #[test]
    fn isocurve_worker_recovery_ui_waits_for_completed_verification() {
        use om9_core::{budget::Limits,geometry::Geometry,model::Entity};use om9_scene::scene::Subelement;
        let limits=Limits::default();let shape=om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Sphere {radius_mm:3.},0.01,&limits).unwrap();let face=shape.topology().faces[0];
        let entity=Entity::new("sphere",Geometry::Brep(shape),None);let id=entity.id;
        let mut doc=Document::with_template_tolerance("worker recovery UI");doc.entities.insert(id,entity);let mut app=MatrixApp::new(doc).unwrap();
        for p in ["1,0,0","2,0,0"] {app.controller.submit("Point").unwrap();app.controller.submit(p).unwrap();}
        app.controller.undo().unwrap();app.controller.select_objects(&[id]).unwrap();app.controller.select_subobject(id,Subelement::BrepFace(face)).unwrap();
        let before=app.controller.document().clone();app.transcript.set_draft("ExtractIsocurve");app.offer_isocurve_start_memory_recovery();app.memory_recovery_running=true;
        let ctx=egui::Context::default();let frame=|app:&mut MatrixApp| {
            let mut output=ctx.run_ui(egui::RawInput {screen_rect:Some(Rect::from_min_size(egui::Pos2::ZERO,Vec2::new(900.,600.))),focused:true,..Default::default()},|ui|app.show(ui));
            output.textures_delta.clear();
        };
        frame(&mut app);assert!(app.isocurve_recovery_waiting());assert!(app.controller.isocurve_face_verification_pending());
        assert!(app.memory_recovery_open);assert_eq!(app.controller.coordinator().undo_len(),1);
        let deadline=std::time::Instant::now()+std::time::Duration::from_secs(30);
        while app.memory_recovery_open {frame(&mut app);assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(2));}
        assert!(!app.controller.isocurve_option_tokens().is_empty());assert_eq!(app.controller.document(),&before);
        assert_eq!((app.controller.coordinator().undo_len(),app.controller.coordinator().redo_len()),(1,1));
    }

    #[cfg(feature="occt-brep")]
    #[test]
    fn isocurve_worker_recovery_cancels_if_root_selection_changes_and_returns() {
        use om9_core::{budget::Limits,geometry::Geometry,model::Entity};use om9_scene::scene::Subelement;
        let limits=Limits::default();let shape=om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Sphere {radius_mm:3.},0.001,&limits).unwrap();
        assert!(shape.display().triangles.len()>512);let face=shape.topology().faces[0];
        let entity=Entity::new("sphere",Geometry::Brep(shape),None);let id=entity.id;
        let mut doc=Document::with_template_tolerance("stale worker root selection");doc.entities.insert(id,entity);
        let mut app=MatrixApp::new(doc).unwrap();
        for p in ["1,0,0","2,0,0"] {app.controller.submit("Point").unwrap();app.controller.submit(p).unwrap();}
        app.controller.undo().unwrap();app.controller.select_objects(&[id]).unwrap();
        app.controller.select_subobject(id,Subelement::BrepFace(face)).unwrap();
        let before=app.controller.document().clone();let sub_revision=app.controller.sub_revision();
        let reserved_before=app.controller.coordinator().external_reserved_bytes(om9_core::budget::memory::MemoryClass::Kernel);
        app.transcript.set_draft("ExtractIsocurve");app.offer_isocurve_start_memory_recovery();app.memory_recovery_running=true;
        assert!(app.retry_undo_recovery_step().unwrap());assert!(app.isocurve_recovery_waiting());assert!(app.controller.face_verification_worker_active_for_test());
        app.controller.select_objects(&[]).unwrap();app.controller.select_objects(&[id]).unwrap();
        assert_eq!(app.controller.sub_revision(),sub_revision,"isolate root-selection changes");
        assert!(!app.isocurve_recovery_context_valid());
        let ctx=egui::Context::default();
        let mut output=ctx.run_ui(egui::RawInput {screen_rect:Some(Rect::from_min_size(egui::Pos2::ZERO,Vec2::new(900.,600.))),focused:true,..Default::default()},|ui|app.show(ui));
        output.textures_delta.clear();
        assert!(!app.memory_recovery_open);assert!(!app.controller.face_verification_pending());
        assert!(app.controller.isocurve_option_tokens().is_empty());assert_eq!(app.controller.document(),&before);
        assert_eq!((app.controller.coordinator().undo_len(),app.controller.coordinator().redo_len()),(1,1));
        let deadline=std::time::Instant::now()+std::time::Duration::from_secs(30);
        while app.controller.face_verification_worker_active_for_test() ||
            app.controller.coordinator().external_reserved_bytes(om9_core::budget::memory::MemoryClass::Kernel)>reserved_before {
            assert!(std::time::Instant::now()<deadline,"canceled worker or its reservation remained active");
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        let mut output=ctx.run_ui(egui::RawInput {screen_rect:Some(Rect::from_min_size(egui::Pos2::ZERO,Vec2::new(900.,600.))),focused:true,..Default::default()},|ui|app.show(ui));
        output.textures_delta.clear();
        assert!(app.controller.isocurve_option_tokens().is_empty());assert_eq!(app.controller.document(),&before);
        assert_eq!((app.controller.coordinator().undo_len(),app.controller.coordinator().redo_len()),(1,1));
    }

    #[test]
    fn isocurve_start_recovery_reclaims_oldest_undo_as_needed() {
        use om9_core::{budget::{Limits,memory::MemoryClass},geometry::{Geometry,Mesh},model::Entity};
        let limits=Limits {max_bytes:32*1024*1024,..Limits::default()};
        let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
        let entity=Entity::new("surface",Geometry::Surface(surface),None);let id=entity.id;
        let old=Entity::new("old mesh",Geometry::Mesh(Mesh {vertices:vec![[1.,1.,1.];50000],faces:vec![],normals:vec![],colors:vec![],corner_uvs:vec![]}),None);let old_id=old.id;
        let mut doc=Document::new("start history");doc.entities.insert(id,entity);doc.entities.insert(old_id,old);
        let mut app=MatrixApp::new(doc.clone()).unwrap();app.controller=crate::session::Controller::new(doc,limits).unwrap();
        app.controller.delete_objects(&[old_id]).unwrap();app.controller.submit("Point").unwrap();app.controller.submit("9,0,0").unwrap();
        app.controller.select_objects(&[id]).unwrap();let before=app.controller.document().clone();
        let pressure=app.controller.coordinator().reserve_remaining_external_memory_with_held(MemoryClass::Kernel,None,app.controller.scene_cache_usage().unwrap()).unwrap();
        app.transcript.set_draft("ExtractIsocurve");app.submit_draft();assert!(app.memory_recovery_open);
        assert_eq!(app.retry_after_undo_recovery().unwrap(),1);assert!(app.controller.isocurve_recovery_key().is_some());
        assert_eq!(app.controller.document(),&before);drop(pressure);app.controller.submit("").unwrap();app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
    }

    #[test]
    fn isocurve_start_recovery_exhaustion_keeps_document_and_redo() {
        use om9_core::{budget::memory::MemoryClass,geometry::Geometry,model::Entity};
        let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&Default::default()).unwrap();
        let entity=Entity::new("surface",Geometry::Surface(surface),None);let id=entity.id;
        let mut doc=Document::new("start exhausted");doc.entities.insert(id,entity);let mut app=MatrixApp::new(doc).unwrap();
        for p in ["1,0,0","2,0,0"] {app.controller.submit("Point").unwrap();app.controller.submit(p).unwrap();}
        app.controller.undo().unwrap();app.controller.select_objects(&[id]).unwrap();let before=app.controller.document().clone();
        let pressure=app.controller.coordinator().reserve_remaining_external_memory_with_held(MemoryClass::Kernel,None,app.controller.scene_cache_usage().unwrap()).unwrap();
        app.transcript.set_draft("ExtractIsocurve");app.submit_draft();assert!(app.memory_recovery_open);
        assert!(app.retry_after_undo_recovery().is_err());assert!(app.controller.isocurve_recovery_key().is_none());
        assert_eq!(app.controller.document(),&before);assert_eq!((app.controller.coordinator().undo_len(),app.controller.coordinator().redo_len()),(0,1));
        drop(pressure);app.controller.submit("ExtractIsocurve").unwrap();assert!(app.controller.isocurve_recovery_key().is_some());
    }

    #[cfg(feature="occt-brep")]
    #[test]
    fn native_brep_wireframe_memory_recovery_retries_from_menu() {
        use om9_core::{budget::{Limits,memory::MemoryClass},geometry::Geometry,model::Entity};
        let limits=Limits::default();let shape=om9_kernel::brep::box_brep([4.,5.,6.],0.1,&limits).unwrap();
        let native=om9_kernel::brep::to_3dm(&shape,0.001,&limits).unwrap();let entity=Entity::new("native",Geometry::NativeBrep(native),None);let id=entity.id;
        let mut doc=Document::with_template_tolerance("native wire recovery");doc.entities.insert(id,entity);let mut app=MatrixApp::new(doc).unwrap();
        app.controller.submit("Point").unwrap();app.controller.submit("1,2,3").unwrap();app.controller.select_objects(&[id]).unwrap();
        let before=app.controller.document().clone();let pressure=app.controller.coordinator().reserve_remaining_external_memory_with_held(MemoryClass::Kernel,None,app.controller.scene_cache_usage().unwrap()).unwrap();
        app.invoke("OM9-CURVE-054",InvocationOrigin::Menu);assert!(app.memory_recovery_open);assert_eq!(app.controller.document(),&before);
        drop(pressure);assert_eq!(app.retry_after_undo_recovery().unwrap(),0);
        assert_eq!(app.controller.document().entities.values().filter(|e|e.label=="ExtractWire").count(),12);
        app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
    }

    #[test]
    fn isocurve_memory_recovery_preserves_input_and_retries() {
        use om9_core::{budget::memory::MemoryClass, geometry::Geometry, model::Entity};
        let mut doc = Document::new("isocurve memory");
        let surface = om9_core::surface_kernel::plane([0.,0.,0.], [1.,0.,0.], [0.,1.,0.], 10., 10., &Default::default()).unwrap();
        let entity = Entity::new("surface", Geometry::Surface(surface), None); let id = entity.id; doc.entities.insert(id, entity);
        let mut app = MatrixApp::new(doc).unwrap();
        app.controller.submit("Point").unwrap(); app.controller.submit("1,2,3").unwrap();
        app.controller.select_objects(&[id]).unwrap();
        app.controller.submit("ExtractIsocurve").unwrap(); app.controller.submit("V=0.25").unwrap();
        let key = app.controller.isocurve_recovery_key().unwrap(); let before = app.controller.document().clone();
        let pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
            MemoryClass::Kernel, None, app.controller.scene_cache_usage().unwrap()).unwrap();
        app.transcript.set_draft(""); app.submit_draft();
        assert!(app.memory_recovery_open); assert_eq!(app.controller.isocurve_recovery_key(), Some(key));
        assert_eq!(app.controller.document(), &before);
        drop(pressure);
        assert_eq!(app.retry_after_undo_recovery().unwrap(), 0);
        assert!(app.controller.isocurve_recovery_key().is_none());
        assert_eq!(app.controller.document().entities.len(), before.entities.len() + 1);
        app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
    }

    #[test]
    fn isocurve_recovery_refuses_changed_parameter_without_removing_undo() {
        use om9_core::{geometry::Geometry, model::Entity};
        let mut doc = Document::new("isocurve changed");
        let surface = om9_core::surface_kernel::plane([0.,0.,0.], [1.,0.,0.], [0.,1.,0.], 10., 10., &Default::default()).unwrap();
        let entity = Entity::new("surface", Geometry::Surface(surface), None); let id = entity.id; doc.entities.insert(id, entity);
        let mut app = MatrixApp::new(doc).unwrap();
        app.controller.submit("Point").unwrap(); app.controller.submit("1,2,3").unwrap();
        app.controller.select_objects(&[id]).unwrap(); app.controller.submit("ExtractIsocurve").unwrap();
        let key = app.controller.isocurve_recovery_key().unwrap(); app.offer_isocurve_memory_recovery(key);
        app.controller.submit("V=0.25").unwrap(); let before = app.controller.coordinator().snapshot();
        assert!(matches!(app.retry_after_undo_recovery(), Err(Error::Stale(_))));
        assert_eq!(app.controller.coordinator().snapshot(), before);
    }

    #[test]
    fn isocurve_recovery_refuses_edited_draft_before_removing_undo() {
        use om9_core::{budget::memory::MemoryClass, geometry::Geometry, model::Entity};
        let mut doc=Document::new("isocurve draft changed");
        let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&Default::default()).unwrap();
        let entity=Entity::new("surface",Geometry::Surface(surface),None);let id=entity.id;doc.entities.insert(id,entity);
        let mut app=MatrixApp::new(doc).unwrap();
        app.controller.submit("Point").unwrap();app.controller.submit("1,2,3").unwrap();
        app.controller.select_objects(&[id]).unwrap();app.controller.submit("ExtractIsocurve").unwrap();
        let pressure=app.controller.coordinator().reserve_remaining_external_memory_with_held(
            MemoryClass::Kernel,None,app.controller.scene_cache_usage().unwrap()).unwrap();
        app.transcript.set_draft("");app.submit_draft();assert!(app.memory_recovery_open);
        app.transcript.set_draft("V=0.25");let before=app.controller.coordinator().snapshot();
        let undo=app.controller.coordinator().undo_len();let key=app.controller.isocurve_recovery_key();
        assert!(matches!(app.retry_undo_recovery_step(),Err(Error::Stale(_))));
        assert_eq!(app.controller.coordinator().snapshot(),before);assert_eq!(app.controller.coordinator().undo_len(),undo);
        assert_eq!(app.controller.isocurve_recovery_key(),key);assert_eq!(app.transcript.draft(),"V=0.25");
        drop(pressure);
    }
    #[test]
    fn isocurve_recovery_refuses_replaced_session_without_removing_undo() {
        use om9_core::{geometry::Geometry, model::Entity};
        let mut doc = Document::new("isocurve replaced");
        let surface = om9_core::surface_kernel::plane([0.,0.,0.], [1.,0.,0.], [0.,1.,0.], 10., 10., &Default::default()).unwrap();
        let entity = Entity::new("surface", Geometry::Surface(surface), None); let id = entity.id; doc.entities.insert(id, entity);
        let mut app = MatrixApp::new(doc).unwrap();
        app.controller.submit("Point").unwrap(); app.controller.submit("1,2,3").unwrap();
        app.controller.select_objects(&[id]).unwrap(); app.controller.submit("ExtractIsocurve").unwrap();
        let key = app.controller.isocurve_recovery_key().unwrap(); app.offer_isocurve_memory_recovery(key);
        app.controller.cancel(); app.controller.submit("ExtractIsocurve").unwrap();
        let before = app.controller.coordinator().snapshot();
        assert!(matches!(app.retry_after_undo_recovery(), Err(Error::Stale(_))));
        assert_eq!(app.controller.coordinator().snapshot(), before);
    }
    #[test]
    fn trimmed_isocurve_retry_can_release_history_owned_memory() {
        use om9_core::{budget::{Limits, memory::MemoryClass}, geometry::{Geometry, Mesh}, model::Entity,
            rhino_semantics::{set_trimmed_surface, TrimmedSurface}};
        let limits = Limits { max_bytes: 32 * 1024 * 1024, ..Default::default() };
        let surface = om9_core::surface_kernel::plane([0.,0.,0.], [1.,0.,0.], [0.,1.,0.], 10., 10., &limits).unwrap();
        let trim = TrimmedSurface { underlying: surface.clone(), outer: vec![[0.2,0.2],[0.8,0.2],[0.8,0.8],[0.2,0.8]],
            inners: vec![vec![[0.4,0.4],[0.6,0.4],[0.6,0.6],[0.4,0.6]]], reversed: false };
        let mut entity = Entity::new("trimmed", Geometry::Surface(surface), None);
        set_trimmed_surface(&mut entity, &trim, &limits).unwrap(); let id = entity.id;
        let old = Entity::new("old mesh", Geometry::Mesh(Mesh { vertices: vec![[1.,1.,1.]; 50000], faces: vec![], normals: vec![], colors: vec![], corner_uvs: vec![] }), None);
        let old_id = old.id; let mut doc = Document::new("trimmed isocurve history");
        doc.entities.insert(id, entity); doc.entities.insert(old_id, old);
        let mut app = MatrixApp::new(doc.clone()).unwrap();
        app.controller = crate::session::Controller::new(doc, limits).unwrap();
        app.controller.delete_objects(&[old_id]).unwrap(); app.controller.submit("Point").unwrap(); app.controller.submit("9,0,0").unwrap();
        app.controller.select_objects(&[id]).unwrap(); app.controller.submit("ExtractIsocurve").unwrap(); app.controller.submit("U=0.5").unwrap();
        let before = app.controller.document().clone();
        let pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
            MemoryClass::Kernel, None, app.controller.scene_cache_usage().unwrap()).unwrap();
        app.transcript.set_draft(""); app.submit_draft(); assert!(app.memory_recovery_open);
        assert_eq!(app.retry_after_undo_recovery().unwrap(), 1);
        assert_eq!(app.controller.document().entities.values().filter(|e| e.label == "ExtractIsocurve").count(), 2);
        drop(pressure); app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
    }

    #[test]
    fn isocurve_retry_can_release_history_owned_memory() {
        use om9_core::{budget::{Limits, memory::MemoryClass}, geometry::{Geometry, Mesh}, model::Entity};
        let mut doc = Document::new("isocurve history recovery");
        let surface = om9_core::surface_kernel::plane([0.,0.,0.], [1.,0.,0.], [0.,1.,0.], 10., 10., &Limits::default()).unwrap();
        let entity = Entity::new("surface", Geometry::Surface(surface), None); let id = entity.id; doc.entities.insert(id, entity);
        let old = Entity::new("old mesh", Geometry::Mesh(Mesh { vertices: vec![[1.,1.,1.]; 50000], faces: vec![], normals: vec![], colors: vec![], corner_uvs: vec![] }), None);
        let old_id = old.id; doc.entities.insert(old_id, old);
        let mut app = MatrixApp::new(doc.clone()).unwrap();
        app.controller = crate::session::Controller::new(doc, Limits { max_bytes: 32 * 1024 * 1024, ..Default::default() }).unwrap();
        app.controller.delete_objects(&[old_id]).unwrap();
        app.controller.submit("Point").unwrap(); app.controller.submit("9,0,0").unwrap();
        app.controller.select_objects(&[id]).unwrap();
        app.controller.submit("ExtractIsocurve").unwrap(); app.controller.submit("V=0.25").unwrap();
        let before = app.controller.document().clone();
        let pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
            MemoryClass::Kernel, None, app.controller.scene_cache_usage().unwrap()).unwrap();
        app.transcript.set_draft(""); app.submit_draft(); assert!(app.memory_recovery_open);
        assert_eq!(app.retry_after_undo_recovery().unwrap(), 1);
        assert_eq!(app.controller.document().entities.len(), before.entities.len() + 1);
        drop(pressure); app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
    }
    #[test]
    fn isocurve_recovery_exhaustion_keeps_pending_and_redo() {
        use om9_core::{budget::memory::MemoryClass, geometry::Geometry, model::Entity};
        let mut doc = Document::new("isocurve retry fails");
        let surface = om9_core::surface_kernel::plane([0.,0.,0.], [1.,0.,0.], [0.,1.,0.], 10., 10., &Default::default()).unwrap();
        let entity = Entity::new("surface", Geometry::Surface(surface), None); let id = entity.id; doc.entities.insert(id, entity);
        let mut app = MatrixApp::new(doc).unwrap();
        for p in ["1,0,0", "2,0,0"] { app.controller.submit("Point").unwrap(); app.controller.submit(p).unwrap(); }
        app.controller.undo().unwrap(); app.controller.select_objects(&[id]).unwrap();
        app.controller.submit("ExtractIsocurve").unwrap(); app.controller.submit("V=0.25").unwrap();
        let before = app.controller.document().clone(); let key = app.controller.isocurve_recovery_key();
        let pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
            MemoryClass::Kernel, None, app.controller.scene_cache_usage().unwrap()).unwrap();
        app.transcript.set_draft(""); app.submit_draft(); assert!(app.memory_recovery_open);
        assert!(app.retry_after_undo_recovery().is_err());
        assert_eq!(app.controller.document(), &before); assert_eq!(app.controller.isocurve_recovery_key(), key);
        assert_eq!((app.controller.coordinator().undo_len(), app.controller.coordinator().redo_len()), (0, 1));
        drop(pressure); app.controller.submit("").unwrap();
        assert_eq!(app.controller.document().entities.len(), before.entities.len() + 1);
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn sphere_numeric_radius_memory_recovery_keeps_center_and_retries() {
        use om9_core::budget::memory::MemoryClass;
        let mut app = MatrixApp::new(Document::with_template_tolerance("sphere recovery")).unwrap();
        app.controller.submit("Point").unwrap(); app.controller.submit("1,2,3").unwrap();
        app.controller.submit("Sphere").unwrap(); app.controller.submit("10,20,30").unwrap();
        let key = app.controller.sphere_recovery_key().unwrap();
        let before = app.controller.document().clone();
        let mut pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
            MemoryClass::Kernel, None, app.controller.scene_cache_usage().unwrap()).unwrap();
        pressure.shrink_to(pressure.bytes() - 2048).unwrap();
        app.transcript.set_draft("5"); app.submit_draft();
        assert!(app.memory_recovery_open); assert_eq!(app.controller.sphere_recovery_key(), Some(key));
        assert_eq!(app.controller.document(), &before);
        drop(pressure); assert_eq!(app.retry_after_undo_recovery().unwrap(), 0);
        assert!(app.controller.sphere_recovery_key().is_none());
        assert_eq!(app.controller.document().entities.len(), before.entities.len() + 1);
        let sphere = app.controller.document().entities.iter().find(|(id, _)| !before.entities.contains_key(id)).unwrap().1;
        let om9_core::geometry::Geometry::Brep(brep) = &sphere.geometry else { panic!("sphere BRep") };
        let measure = om9_kernel::creation::measure(brep, app.controller.coordinator().limits()).unwrap();
        assert!((measure.volume_mm3 - 4. * std::f64::consts::PI * 125. / 3.).abs() < 1e-6);
        assert_eq!(sphere.placement, om9_core::geometry::Transform::translation([10., 20., 30.]));
        app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn sphere_viewport_point_memory_recovery_reuses_world_point() {
        use om9_core::budget::memory::MemoryClass;
        use om9_scene::osnap::{SnapCandidate, SnapKind};
        let mut app = MatrixApp::new(Document::with_template_tolerance("sphere point recovery")).unwrap();
        app.controller.submit("Point").unwrap(); app.controller.submit("1,2,3").unwrap();
        app.controller.submit("Sphere").unwrap(); app.controller.accept_world_point([10.,20.,30.]).unwrap();
        let before = app.controller.document().clone();
        let mut pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
            MemoryClass::Kernel, None, app.controller.scene_cache_usage().unwrap()).unwrap();
        pressure.shrink_to(pressure.bytes() - 2048).unwrap();
        app.snap_state.settings.one_shot = Some(SnapKind::End);
        let anchor = Some(SnapCandidate { kind: SnapKind::End, root: None, world: [15.,20.,30.], screen: [0.,0.], distance_px: 0., normal: None, parameter: None, tracking_tangent: None, source: None, second_source: None, surface_source: None, apparent: false });
        assert!(app.accept_viewport_command_point([15.,20.,30.], &anchor).is_err());
        assert!(app.memory_recovery_open); assert_eq!(app.controller.document(), &before);
        app.snap_state.settings.one_shot = Some(SnapKind::Mid);
        drop(pressure); assert_eq!(app.retry_after_undo_recovery().unwrap(), 0);
        assert_eq!(app.snap_state.settings.one_shot, Some(SnapKind::Mid));
        let sphere = app.controller.document().entities.iter().find(|(id, _)| !before.entities.contains_key(id)).unwrap().1;
        let om9_core::geometry::Geometry::Brep(brep) = &sphere.geometry else { panic!("sphere BRep") };
        let measure = om9_kernel::creation::measure(brep, app.controller.coordinator().limits()).unwrap();
        assert!((measure.volume_mm3 - 4. * std::f64::consts::PI * 125. / 3.).abs() < 1e-6);
        assert_eq!(sphere.placement, om9_core::geometry::Transform::translation([10.,20.,30.]));
        app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn viewport_point_recovery_refuses_an_edited_draft_before_pruning() {
        use om9_core::budget::memory::MemoryClass;
        for sphere in [true,false] {
            let mut app=MatrixApp::new(Document::with_template_tolerance("point recovery edited draft")).unwrap();
            app.controller.submit("Point").unwrap();app.controller.submit("1,2,3").unwrap();
            app.controller.submit("Point").unwrap();app.controller.submit("4,5,6").unwrap();app.controller.undo().unwrap();
            if sphere {app.controller.submit("Sphere").unwrap();}
            else {app.controller.submit("Box").unwrap();app.controller.submit("Cube").unwrap();}
            app.controller.accept_world_point([0.,0.,0.]).unwrap();
            let mut pressure=app.controller.coordinator().reserve_remaining_external_memory_with_held(
                MemoryClass::Kernel,None,app.controller.scene_cache_usage().unwrap()).unwrap();
            pressure.shrink_to(pressure.bytes()-2048).unwrap();
            assert!(app.accept_viewport_command_point([10.,10.,10.],&None).is_err());assert!(app.memory_recovery_open);
            app.transcript.set_draft("7");
            let before=app.controller.coordinator().snapshot();let undo=app.controller.coordinator().undo_len();let redo=app.controller.coordinator().redo_len();assert_eq!(redo,1);
            let sphere_key=app.controller.sphere_recovery_key();let box_key=app.controller.box_recovery_key();
            assert!(matches!(app.retry_undo_recovery_step(),Err(Error::Stale(_))),"sphere={sphere}");
            assert_eq!(app.controller.coordinator().snapshot(),before);assert_eq!(app.controller.coordinator().undo_len(),undo);assert_eq!(app.controller.coordinator().redo_len(),redo);
            assert_eq!(app.controller.sphere_recovery_key(),sphere_key);assert_eq!(app.controller.box_recovery_key(),box_key);
            assert_eq!(app.transcript.draft(),"7");drop(pressure);
        }
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn sphere_viewport_point_retry_reclaims_history_owned_mesh() {
        use om9_core::{budget::{Limits, memory::MemoryClass}, geometry::{Geometry, Mesh}, model::Entity};
        let mut doc = Document::with_template_tolerance("sphere point history recovery");
        let old = Entity::new("old mesh", Geometry::Mesh(Mesh { vertices: vec![[1.,1.,1.]; 400000], faces: vec![], normals: vec![], colors: vec![], corner_uvs: vec![] }), None);
        let old_id = old.id; doc.entities.insert(old_id, old);
        let mut app = MatrixApp::new(doc.clone()).unwrap();
        app.controller = crate::session::Controller::new(doc, Limits { max_bytes: 64 * 1024 * 1024, ..Default::default() }).unwrap();
        app.controller.delete_objects(&[old_id]).unwrap();
        app.controller.submit("Point").unwrap(); app.controller.submit("9,0,0").unwrap();
        app.controller.submit("Sphere").unwrap(); app.controller.accept_world_point([10.,20.,30.]).unwrap();
        let before = app.controller.document().clone();
        let mut pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
            MemoryClass::Kernel, None, app.controller.scene_cache_usage().unwrap()).unwrap();
        pressure.shrink_to(pressure.bytes() - 2048).unwrap();
        assert!(app.accept_viewport_command_point([15.,20.,30.], &None).is_err());
        assert!(app.memory_recovery_open); assert_eq!(app.retry_after_undo_recovery().unwrap(), 1);
        let sphere = app.controller.document().entities.iter().find(|(id, _)| !before.entities.contains_key(id)).unwrap().1;
        let Geometry::Brep(brep) = &sphere.geometry else { panic!("sphere BRep") };
        let measure = om9_kernel::creation::measure(brep, app.controller.coordinator().limits()).unwrap();
        assert!((measure.volume_mm3 - 4. * std::f64::consts::PI * 125. / 3.).abs() < 1e-6);
        assert_eq!(sphere.placement, om9_core::geometry::Transform::translation([10.,20.,30.]));
        drop(pressure); app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn sphere_two_point_retry_keeps_endpoints_for_numeric_and_viewport_input() {
        use om9_core::budget::memory::MemoryClass;
        for numeric in [true, false] {
            let mut app = MatrixApp::new(Document::with_template_tolerance("two point recovery")).unwrap();
            app.controller.submit("Point").unwrap(); app.controller.submit("1,2,3").unwrap();
            app.controller.submit("Sphere 2Point").unwrap(); app.controller.accept_world_point([10.,20.,30.]).unwrap();
            let before = app.controller.document().clone();
            let mut pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
                MemoryClass::Kernel, None, app.controller.scene_cache_usage().unwrap()).unwrap();
            pressure.shrink_to(pressure.bytes() - 2048).unwrap();
            if numeric { app.transcript.set_draft("10"); app.submit_draft(); }
            else { assert!(app.accept_viewport_command_point([20.,20.,30.], &None).is_err()); }
            assert!(app.memory_recovery_open, "numeric={numeric}");
            assert_eq!(app.controller.document(), &before);
            drop(pressure); assert_eq!(app.retry_after_undo_recovery().unwrap(), 0);
            let sphere = app.controller.document().entities.iter().find(|(id, _)| !before.entities.contains_key(id)).unwrap().1;
            let om9_core::geometry::Geometry::Brep(brep) = &sphere.geometry else { panic!("sphere BRep") };
            let measure = om9_kernel::creation::measure(brep, app.controller.coordinator().limits()).unwrap();
            assert!((measure.volume_mm3 - 4. * std::f64::consts::PI * 125. / 3.).abs() < 1e-6);
            assert_eq!(sphere.placement, om9_core::geometry::Transform::translation([15.,20.,30.]));
            app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
        }
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn sphere_two_point_retry_reclaims_history_memory() {
        use om9_core::{budget::{Limits, memory::MemoryClass}, geometry::{Geometry, Mesh}, model::Entity};
        let mut doc = Document::with_template_tolerance("two point history recovery");
        let old = Entity::new("old mesh", Geometry::Mesh(Mesh { vertices: vec![[1.,1.,1.]; 400000], faces: vec![], normals: vec![], colors: vec![], corner_uvs: vec![] }), None);
        let old_id = old.id; doc.entities.insert(old_id, old);
        let mut app = MatrixApp::new(doc.clone()).unwrap();
        app.controller = crate::session::Controller::new(doc, Limits { max_bytes: 64 * 1024 * 1024, ..Default::default() }).unwrap();
        app.controller.delete_objects(&[old_id]).unwrap();
        app.controller.submit("Point").unwrap(); app.controller.submit("9,0,0").unwrap();
        app.controller.submit("Sphere 2Point").unwrap(); app.controller.accept_world_point([10.,20.,30.]).unwrap();
        let before = app.controller.document().clone();
        let mut pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
            MemoryClass::Kernel, None, app.controller.scene_cache_usage().unwrap()).unwrap();
        pressure.shrink_to(pressure.bytes() - 2048).unwrap();
        assert!(app.accept_viewport_command_point([20.,20.,30.], &None).is_err()); assert!(app.memory_recovery_open);
        assert_eq!(app.retry_after_undo_recovery().unwrap(), 1);
        let sphere = app.controller.document().entities.iter().find(|(id, _)| !before.entities.contains_key(id)).unwrap().1;
        let Geometry::Brep(brep) = &sphere.geometry else { panic!("sphere BRep") };
        let measure = om9_kernel::creation::measure(brep, app.controller.coordinator().limits()).unwrap();
        assert!((measure.volume_mm3 - 4. * std::f64::consts::PI * 125. / 3.).abs() < 1e-6);
        assert_eq!(sphere.placement, om9_core::geometry::Transform::translation([15.,20.,30.]));
        drop(pressure); app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn sphere_three_and_four_point_recovery_keeps_all_points() {
        use om9_core::budget::memory::MemoryClass;
        let cases: &[(&str, &[[f64;3]], [f64;3], [f64;3], f64)] = &[
            ("Sphere 3Point", &[[0.,0.,0.], [10.,0.,0.]], [0.,10.,0.], [5.,5.,0.], 50_f64.sqrt()),
            ("Sphere 4Point", &[[0.,0.,0.], [10.,0.,0.], [0.,10.,0.]], [0.,0.,10.], [5.,5.,5.], 75_f64.sqrt()),
        ];
        for (command, points, final_point, center, radius) in cases {
            let mut app = MatrixApp::new(Document::with_template_tolerance("multiple sphere points")).unwrap();
            app.controller.submit("Point").unwrap(); app.controller.submit("1,2,3").unwrap();
            app.controller.submit(command).unwrap();
            for point in *points { app.controller.accept_world_point(*point).unwrap(); }
            let before = app.controller.document().clone(); let key = app.controller.sphere_recovery_key();
            let mut pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
                MemoryClass::Kernel, None, app.controller.scene_cache_usage().unwrap()).unwrap();
            pressure.shrink_to(pressure.bytes() - 2048).unwrap();
            assert!(app.accept_viewport_command_point(*final_point, &None).is_err(), "{command}");
            assert!(app.memory_recovery_open, "{command}"); assert_eq!(app.controller.sphere_recovery_key(), key);
            assert_eq!(app.controller.document(), &before);
            drop(pressure); assert_eq!(app.retry_after_undo_recovery().unwrap(), 0);
            let sphere = app.controller.document().entities.iter().find(|(id, _)| !before.entities.contains_key(id)).unwrap().1;
            let om9_core::geometry::Geometry::Brep(brep) = &sphere.geometry else { panic!("sphere BRep") };
            let measure = om9_kernel::creation::measure(brep, app.controller.coordinator().limits()).unwrap();
            assert!((measure.volume_mm3 - 4. * std::f64::consts::PI * radius.powi(3) / 3.).abs() < 1e-6, "{command}");
            for axis in 0..3 { assert!((sphere.placement.0[axis][3] - center[axis]).abs() < 1e-8, "{command}"); }
            app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
        }
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn sphere_vertical_and_single_curve_modes_recover_numeric_and_point_input() {
        use om9_core::{budget::memory::MemoryClass, geometry::Geometry, model::Entity};
        for mode in ["Vertical", "AroundCurve", "Tangent"] {
            for numeric in [false, true] {
                let mut doc = Document::with_template_tolerance("sphere curve recovery");
                let curve_id = if mode == "Vertical" { None } else {
                    let curve = om9_core::curve_build::polyline(&[[0.,0.,0.],[20.,0.,0.]]).unwrap();
                    let entity = Entity::new("path", Geometry::Curve(curve), None); let id = entity.id;
                    doc.entities.insert(id, entity); Some(id)
                };
                let mut app = MatrixApp::new(doc).unwrap();
                app.controller.submit("Point").unwrap(); app.controller.submit("1,2,3").unwrap();
                if let Some(id) = curve_id { app.controller.select_objects(&[id]).unwrap(); }
                app.controller.submit("Sphere").unwrap(); app.controller.submit(mode).unwrap();
                app.controller.accept_world_point([10.,0.,0.]).unwrap();
                let before = app.controller.document().clone(); let key = app.controller.sphere_recovery_key();
                assert!(key.is_some(), "{mode}");
                let mut pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
                    MemoryClass::Kernel, None, app.controller.scene_cache_usage().unwrap()).unwrap();
                pressure.shrink_to(pressure.bytes() - 2048).unwrap();
                if numeric { app.transcript.set_draft("5"); app.submit_draft(); }
                else { assert!(app.accept_viewport_command_point([10.,5.,0.], &None).is_err()); }
                assert!(app.memory_recovery_open, "{mode} numeric={numeric}");
                assert_eq!(app.controller.sphere_recovery_key(), key); assert_eq!(app.controller.document(), &before);
                drop(pressure); assert_eq!(app.retry_after_undo_recovery().unwrap(), 0);
                let sphere = app.controller.document().entities.iter().find(|(id, _)| !before.entities.contains_key(id)).unwrap().1;
                let Geometry::Brep(brep) = &sphere.geometry else { panic!("sphere BRep") };
                let measure = om9_kernel::creation::measure(brep, app.controller.coordinator().limits()).unwrap();
                assert!((measure.volume_mm3 - 4. * std::f64::consts::PI * 125. / 3.).abs() < 1e-6);
                let center = if mode == "Tangent" { [10.,-5.,0.] } else { [10.,0.,0.] };
                for axis in 0..3 { assert!((sphere.placement.0[axis][3] - center[axis]).abs() < 1e-8, "{mode}"); }
                app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
            }
        }
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn sphere_radius_and_diameter_options_retry_with_original_value() {
        use om9_core::{budget::memory::MemoryClass, geometry::Geometry, model::Entity};
        for (mode, input, expected) in [
            ("Center", "Radius=5", [10.,20.,30.]),
            ("Center", "Diameter=10", [10.,20.,30.]),
            ("TwoPoint", "Diameter=10", [15.,20.,30.]),
            ("Tangent", "Radius=5", [10.,-5.,0.]),
        ] {
            let mut doc = Document::with_template_tolerance("sphere option recovery");
            let path = if mode == "Tangent" {
                let curve = om9_core::curve_build::polyline(&[[0.,0.,0.],[20.,0.,0.]]).unwrap();
                let entity = Entity::new("path", Geometry::Curve(curve), None); let id = entity.id; doc.entities.insert(id, entity); Some(id)
            } else { None };
            let mut app = MatrixApp::new(doc).unwrap();
            app.controller.submit("Point").unwrap(); app.controller.submit("1,2,3").unwrap();
            if let Some(id) = path { app.controller.select_objects(&[id]).unwrap(); }
            app.controller.submit("Sphere").unwrap();
            if mode == "TwoPoint" { app.controller.submit("2Point").unwrap(); }
            if mode == "Tangent" { app.controller.submit("Tangent").unwrap(); }
            app.controller.accept_world_point(if mode == "Tangent" { [10.,0.,0.] } else { [10.,20.,30.] }).unwrap();
            let before = app.controller.document().clone(); let key = app.controller.sphere_recovery_key();
            let mut pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
                MemoryClass::Kernel, None, app.controller.scene_cache_usage().unwrap()).unwrap();
            pressure.shrink_to(pressure.bytes() - 2048).unwrap();
            app.transcript.set_draft(input); app.submit_draft();
            assert!(app.memory_recovery_open, "{mode} {input}");
            assert_eq!(app.controller.sphere_recovery_key(), key); assert_eq!(app.controller.document(), &before);
            drop(pressure); assert_eq!(app.retry_after_undo_recovery().unwrap(), 0);
            let sphere = app.controller.document().entities.iter().find(|(id, _)| !before.entities.contains_key(id)).unwrap().1;
            let Geometry::Brep(brep) = &sphere.geometry else { panic!("sphere BRep") };
            let measure = om9_kernel::creation::measure(brep, app.controller.coordinator().limits()).unwrap();
            assert!((measure.volume_mm3 - 4. * std::f64::consts::PI * 125. / 3.).abs() < 1e-6, "{mode} {input}");
            for axis in 0..3 { assert!((sphere.placement.0[axis][3] - expected[axis]).abs() < 1e-8, "{mode} {input}"); }
            app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
        }
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn sphere_absolute_coordinate_input_recovers_without_changing_geometry() {
        use om9_core::budget::memory::MemoryClass;
        let cases: &[(&str, &[&str], &str, [f64;3], f64)] = &[
            ("Sphere", &["10,20,30"], "15,20,30", [10.,20.,30.], 5.),
            ("Sphere 2Point", &["10,20,30"], "20,20,30", [15.,20.,30.], 5.),
            ("Sphere 3Point", &["0,0,0", "10,0,0"], "0,10,0", [5.,5.,0.], 50_f64.sqrt()),
            ("Sphere 4Point", &["0,0,0", "10,0,0", "0,10,0"], "0,0,10", [5.,5.,5.], 75_f64.sqrt()),
        ];
        for (command, inputs, final_input, center, radius) in cases {
            let mut app = MatrixApp::new(Document::with_template_tolerance("coordinate retry")).unwrap();
            app.controller.submit("Point").unwrap(); app.controller.submit("1,2,3").unwrap();
            app.controller.submit(command).unwrap();
            for input in *inputs { app.controller.submit(input).unwrap(); }
            let before = app.controller.document().clone(); let key = app.controller.sphere_recovery_key();
            let mut pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
                MemoryClass::Kernel, None, app.controller.scene_cache_usage().unwrap()).unwrap();
            pressure.shrink_to(pressure.bytes() - 2048).unwrap();
            app.transcript.set_draft(final_input); app.submit_draft();
            assert!(app.memory_recovery_open, "{command}");
            assert_eq!(app.controller.sphere_recovery_key(), key); assert_eq!(app.controller.document(), &before);
            drop(pressure); assert_eq!(app.retry_after_undo_recovery().unwrap(), 0);
            let sphere = app.controller.document().entities.iter().find(|(id, _)| !before.entities.contains_key(id)).unwrap().1;
            let om9_core::geometry::Geometry::Brep(brep) = &sphere.geometry else { panic!("sphere BRep") };
            let measure = om9_kernel::creation::measure(brep, app.controller.coordinator().limits()).unwrap();
            assert!((measure.volume_mm3 - 4. * std::f64::consts::PI * radius.powi(3) / 3.).abs() < 1e-6, "{command}");
            for axis in 0..3 { assert!((sphere.placement.0[axis][3] - center[axis]).abs() < 1e-8, "{command}"); }
            app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
        }
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn diagonal_box_memory_recovery_preserves_height_input() {
        use om9_core::{budget::memory::MemoryClass, geometry::Geometry};
        for input in ["30", "Height=30", "viewport"] {
            let mut app = MatrixApp::new(Document::with_template_tolerance("box retry")).unwrap();
            app.controller.submit("Point").unwrap(); app.controller.submit("1,2,3").unwrap();
            app.controller.submit("Box").unwrap();
            app.controller.accept_world_point([0.,0.,0.]).unwrap(); app.controller.accept_world_point([10.,20.,0.]).unwrap();
            let before = app.controller.document().clone(); let key = app.controller.box_recovery_key(); assert!(key.is_some());
            let mut pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
                MemoryClass::Kernel, None, app.controller.scene_cache_usage().unwrap()).unwrap();
            pressure.shrink_to(pressure.bytes() - 2048).unwrap();
            if input == "viewport" { assert!(app.accept_viewport_command_point([0.,0.,30.], &None).is_err()); }
            else { app.transcript.set_draft(input); app.submit_draft(); }
            assert!(app.memory_recovery_open, "{input}");
            assert_eq!(app.controller.box_recovery_key(), key); assert_eq!(app.controller.document(), &before);
            drop(pressure); assert_eq!(app.retry_after_undo_recovery().unwrap(), 0);
            let result = app.controller.document().entities.iter().find(|(id, _)| !before.entities.contains_key(id)).unwrap().1;
            let Geometry::Brep(brep) = &result.geometry else { panic!("box BRep") };
            let measure = om9_kernel::creation::measure(brep, app.controller.coordinator().limits()).unwrap();
            assert!((measure.volume_mm3 - 6000.).abs() < 1e-6, "{input}");
            app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
        }
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn box_cube_center_three_point_vertical_recovery_keeps_inputs() {
        use om9_core::{budget::memory::MemoryClass, geometry::Geometry};
        for mode in ["Cube", "Center", "3Point", "Vertical"] {
            for viewport in [false, true] {
                let mut app = MatrixApp::new(Document::with_template_tolerance("box modes retry")).unwrap();
                app.controller.submit("Point").unwrap(); app.controller.submit("1,2,3").unwrap();
                app.controller.submit("Box").unwrap(); app.controller.submit(mode).unwrap();
                app.controller.accept_world_point([0.,0.,0.]).unwrap();
                if mode == "Center" { app.controller.submit("10").unwrap(); app.controller.submit("20").unwrap(); }
                if mode == "3Point" || mode == "Vertical" {
                    app.controller.accept_world_point([10.,0.,0.]).unwrap(); app.controller.submit("20").unwrap();
                }
                let key = app.controller.box_recovery_key(); assert!(key.is_some(), "{mode}");
                let before = app.controller.document().clone();
                let mut pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
                    MemoryClass::Kernel, None, app.controller.scene_cache_usage().unwrap()).unwrap();
                pressure.shrink_to(pressure.bytes() - 2048).unwrap();
                if viewport {
                    let point = match mode { "Cube" => [10.,10.,10.], "Vertical" => [0.,-30.,0.], _ => [0.,0.,30.] };
                    assert!(app.accept_viewport_command_point(point, &None).is_err(), "{mode}");
                } else {
                    app.transcript.set_draft(if mode == "Cube" { "10,10,10" } else { "30" }); app.submit_draft();
                }
                assert!(app.memory_recovery_open, "{mode} viewport={viewport}");
                assert_eq!(app.controller.box_recovery_key(), key); assert_eq!(app.controller.document(), &before);
                drop(pressure); assert_eq!(app.retry_after_undo_recovery().unwrap(), 0);
                let entity = app.controller.document().entities.iter().find(|(id, _)| !before.entities.contains_key(id)).unwrap().1;
                let Geometry::Brep(brep) = &entity.geometry else { panic!("box BRep") };
                let measure = om9_kernel::creation::measure(brep, app.controller.coordinator().limits()).unwrap();
                let expected = if mode == "Cube" { 1000. } else { 6000. };
                assert!((measure.volume_mm3 - expected).abs() < 1e-6, "{mode} viewport={viewport}");
                app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
            }
        }
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn direct_solid_recovery_keeps_command_and_rejects_changed_draft() {
        use om9_core::{budget::memory::MemoryClass, geometry::Geometry};
        for (input, expected) in [("Box 10 20 30", 6000.), ("Sphere 5", 4. * std::f64::consts::PI * 125. / 3.)] {
            let mut app = MatrixApp::new(Document::with_template_tolerance("direct solid recovery")).unwrap();
            app.controller.submit("Point").unwrap(); app.controller.submit("1,2,3").unwrap();
            app.controller.undo().unwrap();
            let before = app.controller.coordinator().snapshot();
            let document_before = app.controller.document().clone();
            let mut pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
                MemoryClass::Kernel, None, app.controller.scene_cache_usage().unwrap()).unwrap();
            pressure.shrink_to(pressure.bytes() - 2048).unwrap();
            app.transcript.set_draft(input); app.submit_draft();
            assert!(app.memory_recovery_open, "{input}");
            assert_eq!(app.controller.coordinator().snapshot(), before);
            app.transcript.set_draft("changed");
            assert!(matches!(app.retry_after_undo_recovery(), Err(Error::Stale(_))));
            assert_eq!(app.controller.coordinator().snapshot(), before);
            app.transcript.set_draft(input); app.submit_draft(); assert!(app.memory_recovery_open);
            drop(pressure); assert_eq!(app.retry_after_undo_recovery().unwrap(), 0);
            let entity = app.controller.document().entities.iter().find(|(id, _)| !document_before.entities.contains_key(id)).unwrap().1;
            let Geometry::Brep(brep) = &entity.geometry else { panic!("solid BRep") };
            let measure = om9_kernel::creation::measure(brep, app.controller.coordinator().limits()).unwrap();
            assert!((measure.volume_mm3 - expected).abs() < 1e-5, "{input}");
            app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &document_before);
        }
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn direct_solid_recovery_reclaims_oldest_history() {
        use om9_core::{budget::{Limits, memory::MemoryClass}, geometry::{Geometry, Mesh}, model::Entity};
        for input in ["Box 10 20 30", "Sphere 5"] {
            let mut doc = Document::with_template_tolerance("direct solid history recovery");
            let old = Entity::new("old mesh", Geometry::Mesh(Mesh { vertices: vec![[1.,1.,1.]; 400000], faces: vec![], normals: vec![], colors: vec![], corner_uvs: vec![] }), None);
            let old_id = old.id; doc.entities.insert(old_id, old);
            let mut app = MatrixApp::new(doc.clone()).unwrap();
            app.controller = crate::session::Controller::new(doc, Limits { max_bytes: 64 * 1024 * 1024, ..Default::default() }).unwrap();
            app.controller.delete_objects(&[old_id]).unwrap(); app.controller.submit("Point").unwrap(); app.controller.submit("9,0,0").unwrap();
            let before = app.controller.document().clone();
            let mut pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
                MemoryClass::Kernel, None, app.controller.scene_cache_usage().unwrap()).unwrap();
            pressure.shrink_to(pressure.bytes() - 2048).unwrap();
            app.transcript.set_draft(input); app.submit_draft(); assert!(app.memory_recovery_open, "{input}");
            assert_eq!(app.retry_after_undo_recovery().unwrap(), 1, "{input}");
            drop(pressure); app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
        }
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn box_center_retry_reclaims_history_memory() {
        use om9_core::{budget::{Limits, memory::MemoryClass}, geometry::{Geometry, Mesh}, model::Entity};
        let mut doc = Document::with_template_tolerance("center box history recovery");
        let old = Entity::new("old mesh", Geometry::Mesh(Mesh { vertices: vec![[1.,1.,1.]; 400000], faces: vec![], normals: vec![], colors: vec![], corner_uvs: vec![] }), None);
        let old_id = old.id; doc.entities.insert(old_id, old);
        let mut app = MatrixApp::new(doc.clone()).unwrap();
        app.controller = crate::session::Controller::new(doc, Limits { max_bytes: 64 * 1024 * 1024, ..Default::default() }).unwrap();
        app.controller.delete_objects(&[old_id]).unwrap(); app.controller.submit("Point").unwrap(); app.controller.submit("9,0,0").unwrap();
        app.controller.submit("Box").unwrap(); app.controller.submit("Center").unwrap(); app.controller.accept_world_point([0.,0.,0.]).unwrap();
        app.controller.submit("10").unwrap(); app.controller.submit("20").unwrap(); let before = app.controller.document().clone();
        let mut pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
            MemoryClass::Kernel, None, app.controller.scene_cache_usage().unwrap()).unwrap();
        pressure.shrink_to(pressure.bytes() - 2048).unwrap();
        app.transcript.set_draft("30"); app.submit_draft(); assert!(app.memory_recovery_open);
        assert_eq!(app.retry_after_undo_recovery().unwrap(), 1);
        let entity = app.controller.document().entities.iter().find(|(id, _)| !before.entities.contains_key(id)).unwrap().1;
        let Geometry::Brep(brep) = &entity.geometry else { panic!("box BRep") };
        let measure = om9_kernel::creation::measure(brep, app.controller.coordinator().limits()).unwrap();
        assert!((measure.volume_mm3 - 6000.).abs() < 1e-6);
        drop(pressure); app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn box_center_recovery_rejects_changed_width() {
        let mut app = MatrixApp::new(Document::with_template_tolerance("changed box width")).unwrap();
        app.controller.submit("Point").unwrap(); app.controller.submit("1,2,3").unwrap();
        app.controller.submit("Box").unwrap(); app.controller.submit("Center").unwrap(); app.controller.accept_world_point([0.,0.,0.]).unwrap();
        app.controller.submit("10").unwrap(); app.controller.submit("20").unwrap();
        let key = app.controller.box_recovery_key().unwrap(); app.transcript.set_draft("30"); app.offer_box_memory_recovery(key, "30".into());
        app.controller.submit("Undo").unwrap(); app.controller.submit("25").unwrap(); let before = app.controller.coordinator().snapshot();
        assert!(matches!(app.retry_after_undo_recovery(), Err(Error::Stale(_))));
        assert_eq!(app.controller.coordinator().snapshot(), before);
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn box_recovery_reclaims_history_and_keeps_geometry() {
        use om9_core::{budget::{Limits, memory::MemoryClass}, geometry::{Geometry, Mesh}, model::Entity};
        let mut doc = Document::with_template_tolerance("box history recovery");
        let old = Entity::new("old mesh", Geometry::Mesh(Mesh { vertices: vec![[1.,1.,1.]; 400000], faces: vec![], normals: vec![], colors: vec![], corner_uvs: vec![] }), None);
        let old_id = old.id; doc.entities.insert(old_id, old);
        let mut app = MatrixApp::new(doc.clone()).unwrap();
        app.controller = crate::session::Controller::new(doc, Limits { max_bytes: 64 * 1024 * 1024, ..Default::default() }).unwrap();
        app.controller.delete_objects(&[old_id]).unwrap(); app.controller.submit("Point").unwrap(); app.controller.submit("9,0,0").unwrap();
        app.controller.submit("Box").unwrap(); app.controller.accept_world_point([0.,0.,0.]).unwrap(); app.controller.accept_world_point([10.,20.,0.]).unwrap();
        let before = app.controller.document().clone();
        let mut pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
            MemoryClass::Kernel, None, app.controller.scene_cache_usage().unwrap()).unwrap();
        pressure.shrink_to(pressure.bytes() - 2048).unwrap();
        app.transcript.set_draft("Height=30"); app.submit_draft(); assert!(app.memory_recovery_open);
        assert_eq!(app.retry_after_undo_recovery().unwrap(), 1);
        let entity = app.controller.document().entities.iter().find(|(id, _)| !before.entities.contains_key(id)).unwrap().1;
        let Geometry::Brep(brep) = &entity.geometry else { panic!("box BRep") };
        let measure = om9_kernel::creation::measure(brep, app.controller.coordinator().limits()).unwrap();
        assert!((measure.volume_mm3 - 6000.).abs() < 1e-6);
        drop(pressure); app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn box_recovery_rejects_replaced_session_before_removing_undo() {
        let mut app = MatrixApp::new(Document::with_template_tolerance("replaced box")).unwrap();
        app.controller.submit("Point").unwrap(); app.controller.submit("1,2,3").unwrap();
        let begin = |app: &mut MatrixApp| {
            app.controller.submit("Box").unwrap(); app.controller.accept_world_point([0.,0.,0.]).unwrap(); app.controller.accept_world_point([10.,20.,0.]).unwrap();
        };
        begin(&mut app); let key = app.controller.box_recovery_key().unwrap();
        app.transcript.set_draft("30"); app.offer_box_memory_recovery(key, "30".into());
        app.controller.cancel(); begin(&mut app); let before = app.controller.coordinator().snapshot();
        assert!(matches!(app.retry_after_undo_recovery(), Err(Error::Stale(_))));
        assert_eq!(app.controller.coordinator().snapshot(), before);
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn box_recovery_exhaustion_preserves_pending_corners_and_redo() {
        use om9_core::budget::memory::MemoryClass;
        let mut app = MatrixApp::new(Document::with_template_tolerance("box memory still full")).unwrap();
        for p in ["1,0,0", "2,0,0"] { app.controller.submit("Point").unwrap(); app.controller.submit(p).unwrap(); }
        app.controller.undo().unwrap(); app.controller.submit("Box").unwrap();
        app.controller.accept_world_point([0.,0.,0.]).unwrap(); app.controller.accept_world_point([10.,20.,0.]).unwrap();
        let before = app.controller.document().clone(); let key = app.controller.box_recovery_key();
        let mut pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
            MemoryClass::Kernel, None, app.controller.scene_cache_usage().unwrap()).unwrap();
        pressure.shrink_to(pressure.bytes() - 2048).unwrap();
        app.transcript.set_draft("30"); app.submit_draft(); assert!(app.memory_recovery_open);
        assert!(app.retry_after_undo_recovery().is_err());
        assert_eq!(app.controller.document(), &before); assert_eq!(app.controller.box_recovery_key(), key);
        assert_eq!((app.controller.coordinator().undo_len(), app.controller.coordinator().redo_len()), (0, 1));
        drop(pressure); app.controller.submit("30").unwrap();
        assert_eq!(app.controller.document().entities.len(), before.entities.len() + 1);
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn sphere_coordinate_retry_reclaims_history_memory() {
        use om9_core::{budget::{Limits, memory::MemoryClass}, geometry::{Geometry, Mesh}, model::Entity};
        let mut doc = Document::with_template_tolerance("coordinate history recovery");
        let old = Entity::new("old mesh", Geometry::Mesh(Mesh { vertices: vec![[1.,1.,1.]; 400000], faces: vec![], normals: vec![], colors: vec![], corner_uvs: vec![] }), None);
        let old_id = old.id; doc.entities.insert(old_id, old);
        let mut app = MatrixApp::new(doc.clone()).unwrap();
        app.controller = crate::session::Controller::new(doc, Limits { max_bytes: 64 * 1024 * 1024, ..Default::default() }).unwrap();
        app.controller.delete_objects(&[old_id]).unwrap();
        app.controller.submit("Point").unwrap(); app.controller.submit("9,0,0").unwrap();
        app.controller.submit("Sphere 4Point").unwrap();
        for point in ["0,0,0", "10,0,0", "0,10,0"] { app.controller.submit(point).unwrap(); }
        let before = app.controller.document().clone();
        let mut pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
            MemoryClass::Kernel, None, app.controller.scene_cache_usage().unwrap()).unwrap();
        pressure.shrink_to(pressure.bytes() - 2048).unwrap();
        app.transcript.set_draft("0,0,10"); app.submit_draft(); assert!(app.memory_recovery_open);
        assert_eq!(app.retry_after_undo_recovery().unwrap(), 1);
        let sphere = app.controller.document().entities.iter().find(|(id, _)| !before.entities.contains_key(id)).unwrap().1;
        let Geometry::Brep(brep) = &sphere.geometry else { panic!("sphere BRep") };
        let measure = om9_kernel::creation::measure(brep, app.controller.coordinator().limits()).unwrap();
        assert!((measure.volume_mm3 - 4. * std::f64::consts::PI * 75_f64.sqrt().powi(3) / 3.).abs() < 1e-6);
        for axis in 0..3 { assert!((sphere.placement.0[axis][3] - 5.).abs() < 1e-8); }
        drop(pressure); app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn sphere_around_curve_recovery_reclaims_history_memory() {
        use om9_core::{budget::{Limits, memory::MemoryClass}, geometry::{Geometry, Mesh}, model::Entity};
        let mut doc = Document::with_template_tolerance("curve sphere history recovery");
        let old = Entity::new("old mesh", Geometry::Mesh(Mesh { vertices: vec![[1.,1.,1.]; 400000], faces: vec![], normals: vec![], colors: vec![], corner_uvs: vec![] }), None);
        let old_id = old.id; doc.entities.insert(old_id, old);
        let curve = om9_core::curve_build::polyline(&[[0.,0.,0.],[20.,0.,0.]]).unwrap();
        let path = Entity::new("path", Geometry::Curve(curve), None); let path_id = path.id; doc.entities.insert(path_id, path);
        let mut app = MatrixApp::new(doc.clone()).unwrap();
        app.controller = crate::session::Controller::new(doc, Limits { max_bytes: 64 * 1024 * 1024, ..Default::default() }).unwrap();
        app.controller.delete_objects(&[old_id]).unwrap();
        app.controller.submit("Point").unwrap(); app.controller.submit("9,0,0").unwrap();
        app.controller.select_objects(&[path_id]).unwrap(); app.controller.submit("Sphere").unwrap();
        app.controller.submit("AroundCurve").unwrap(); app.controller.accept_world_point([10.,0.,0.]).unwrap();
        let before = app.controller.document().clone();
        let mut pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
            MemoryClass::Kernel, None, app.controller.scene_cache_usage().unwrap()).unwrap();
        pressure.shrink_to(pressure.bytes() - 2048).unwrap();
        app.transcript.set_draft("5"); app.submit_draft(); assert!(app.memory_recovery_open);
        assert_eq!(app.retry_after_undo_recovery().unwrap(), 1);
        let sphere = app.controller.document().entities.iter().find(|(id, _)| !before.entities.contains_key(id)).unwrap().1;
        let Geometry::Brep(brep) = &sphere.geometry else { panic!("sphere BRep") };
        let measure = om9_kernel::creation::measure(brep, app.controller.coordinator().limits()).unwrap();
        assert!((measure.volume_mm3 - 4. * std::f64::consts::PI * 125. / 3.).abs() < 1e-6);
        assert_eq!(sphere.placement, om9_core::geometry::Transform::translation([10.,0.,0.]));
        drop(pressure); app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn sphere_four_point_retry_reclaims_history_memory() {
        use om9_core::{budget::{Limits, memory::MemoryClass}, geometry::{Geometry, Mesh}, model::Entity};
        let mut doc = Document::with_template_tolerance("four point history recovery");
        let old = Entity::new("old mesh", Geometry::Mesh(Mesh { vertices: vec![[1.,1.,1.]; 400000], faces: vec![], normals: vec![], colors: vec![], corner_uvs: vec![] }), None);
        let old_id = old.id; doc.entities.insert(old_id, old);
        let mut app = MatrixApp::new(doc.clone()).unwrap();
        app.controller = crate::session::Controller::new(doc, Limits { max_bytes: 64 * 1024 * 1024, ..Default::default() }).unwrap();
        app.controller.delete_objects(&[old_id]).unwrap();
        app.controller.submit("Point").unwrap(); app.controller.submit("9,0,0").unwrap();
        app.controller.submit("Sphere 4Point").unwrap();
        for point in [[0.,0.,0.], [10.,0.,0.], [0.,10.,0.]] { app.controller.accept_world_point(point).unwrap(); }
        let before = app.controller.document().clone();
        let mut pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
            MemoryClass::Kernel, None, app.controller.scene_cache_usage().unwrap()).unwrap();
        pressure.shrink_to(pressure.bytes() - 2048).unwrap();
        assert!(app.accept_viewport_command_point([0.,0.,10.], &None).is_err()); assert!(app.memory_recovery_open);
        assert_eq!(app.retry_after_undo_recovery().unwrap(), 1);
        let sphere = app.controller.document().entities.iter().find(|(id, _)| !before.entities.contains_key(id)).unwrap().1;
        let Geometry::Brep(brep) = &sphere.geometry else { panic!("sphere BRep") };
        let measure = om9_kernel::creation::measure(brep, app.controller.coordinator().limits()).unwrap();
        assert!((measure.volume_mm3 - 4. * std::f64::consts::PI * 75_f64.sqrt().powi(3) / 3.).abs() < 1e-6);
        for axis in 0..3 { assert!((sphere.placement.0[axis][3] - 5.).abs() < 1e-8); }
        drop(pressure); app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn sphere_fit_direct_point_recovery_keeps_input_and_redo() {
        use om9_core::{budget::{Limits, memory::MemoryClass}, geometry::{Geometry, Mesh}, model::Entity};
        for text_input in [false, true] {
            let limits = Limits { max_bytes: 64 * 1024 * 1024, ..Limits::default() };
            let mut doc = Document::with_template_tolerance("fit point recovery");
            let old = Entity::new("old mesh", Geometry::Mesh(Mesh { vertices: vec![[1.,1.,1.]; 400000], faces: vec![], normals: vec![], colors: vec![], corner_uvs: vec![] }), None);
            let id = old.id; doc.entities.insert(id, old);
            let mut app = MatrixApp::new(doc.clone()).unwrap();
            app.controller = crate::session::Controller::new(doc, limits).unwrap();
            app.controller.delete_objects(&[id]).unwrap();
            app.controller.submit("Point").unwrap(); app.controller.submit("9,0,0").unwrap(); app.controller.undo().unwrap();
            app.controller.submit("Sphere FitPoints").unwrap();
            let before = app.controller.document().clone();
            let mut pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
                MemoryClass::Kernel, None, app.controller.scene_cache_usage().unwrap()).unwrap();
            pressure.shrink_to(pressure.bytes() - 64).unwrap();
            if text_input { app.transcript.set_draft("1,0,0"); app.submit_draft(); }
            else { assert!(app.accept_viewport_command_point([1.,0.,0.], &None).is_err()); }
            assert!(app.memory_recovery_open, "text={text_input}, {}", app.status_line());
            assert!(app.controller.solid_preview_markers().is_empty());
            assert_eq!(app.controller.coordinator().redo_len(), 1);
            assert_eq!(app.retry_after_undo_recovery().unwrap(), 1);
            assert_eq!(app.controller.solid_preview_markers(), vec![[1.,0.,0.]]);
            assert_eq!(app.controller.document(), &before);
            assert_eq!(app.controller.coordinator().redo_len(), 1);
            drop(pressure);
        }
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn sphere_fit_point_with_spare_capacity_does_not_offer_pruning() {
        use om9_core::budget::memory::MemoryClass;
        let mut app = MatrixApp::new(Document::with_template_tolerance("fit spare capacity")).unwrap();
        app.controller.submit("Point").unwrap(); app.controller.submit("9,0,0").unwrap();
        app.controller.submit("Point").unwrap(); app.controller.submit("8,0,0").unwrap(); app.controller.undo().unwrap();
        app.controller.submit("Sphere FitPoints").unwrap();
        app.controller.accept_world_point([1.,0.,0.]).unwrap();
        let before = app.controller.document().clone();
        let undo = app.controller.coordinator().undo_len();
        let pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
            MemoryClass::Kernel, None, app.controller.scene_cache_usage().unwrap()).unwrap();
        app.accept_viewport_command_point([0.,1.,0.], &None).unwrap();
        assert!(!app.memory_recovery_open);
        assert_eq!(app.controller.solid_preview_markers(), vec![[1.,0.,0.], [0.,1.,0.]]);
        assert_eq!(app.controller.coordinator().undo_len(), undo);
        assert_eq!(app.controller.coordinator().redo_len(), 1);
        assert_eq!(app.controller.document(), &before);
        drop(pressure);
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn sphere_fit_point_limit_does_not_offer_undo_pruning() {
        use om9_core::budget::Limits;
        let doc = Document::with_template_tolerance("fit point hard limit");
        let mut app = MatrixApp::new(doc.clone()).unwrap();
        app.controller = crate::session::Controller::new(doc, Limits { max_points: 4, ..Limits::default() }).unwrap();
        app.controller.submit("Point").unwrap(); app.controller.submit("9,0,0").unwrap();
        app.controller.submit("Sphere FitPoints").unwrap();
        for point in [[1.,0.,0.], [-1.,0.,0.], [0.,1.,0.], [0.,0.,1.]] {
            app.controller.accept_world_point(point).unwrap();
        }
        let before = app.controller.document().clone();
        let undo = app.controller.coordinator().undo_len();
        let points = app.controller.solid_preview_markers();
        assert!(matches!(app.accept_viewport_command_point([0.,-1.,0.], &None), Err(Error::Budget("Sphere FitPoints"))));
        assert!(!app.memory_recovery_open);
        assert_eq!(app.controller.coordinator().undo_len(), undo);
        assert_eq!(app.controller.solid_preview_markers(), points);
        assert_eq!(app.controller.document(), &before);
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn sphere_fit_native_selection_recovers_memory_and_rejects_changed_selection() {
        use om9_core::{budget::{Limits, memory::MemoryClass}, geometry::{Geometry, Mesh}, model::Entity};
        let limits = Limits { max_bytes: 64 * 1024 * 1024, ..Limits::default() };
        let source = om9_kernel::brep::box_brep([4., 5., 6.], 0.1, &limits).unwrap();
        let native = om9_kernel::brep::to_3dm(&source, 0.001, &limits).unwrap();
        for input in ["Selection", ""] {
            let mut doc = Document::with_template_tolerance("fit native source recovery");
            let old = Entity::new("old mesh", Geometry::Mesh(Mesh { vertices: vec![[1.,1.,1.]; 400000], faces: vec![], normals: vec![], colors: vec![], corner_uvs: vec![] }), None);
            let old_id = old.id;
            let shape = Entity::new("native", Geometry::NativeBrep(native.clone()), None);
            let shape_id = shape.id;
            doc.entities.insert(old_id, old);
            doc.entities.insert(shape_id, shape);
            let mut app = MatrixApp::new(doc.clone()).unwrap();
            app.controller = crate::session::Controller::new(doc, limits).unwrap();
            app.controller.delete_objects(&[old_id]).unwrap();
            app.controller.submit("Point").unwrap(); app.controller.submit("9,0,0").unwrap();
            app.controller.undo().unwrap();
            assert_eq!(app.controller.coordinator().redo_len(), 1);
            app.controller.select_objects(&[shape_id]).unwrap();
            app.controller.submit("Sphere FitPoints").unwrap();
            let before = app.controller.document().clone();
            let mut pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
                MemoryClass::Kernel, None, app.controller.scene_cache_usage().unwrap()).unwrap();
            pressure.shrink_to(pressure.bytes() - 2048).unwrap();
            app.transcript.set_draft(input);
            app.submit_draft();
            assert!(app.memory_recovery_open, "input {input:?}: {}", app.status_line());
            assert_eq!(app.controller.document(), &before);
            assert_eq!(app.controller.coordinator().redo_len(), 1);
            assert_eq!(app.retry_after_undo_recovery().unwrap(), 1);
            assert_eq!(app.controller.solid_preview_markers().len(), 8);
            assert_eq!(app.controller.document(), &before);
            assert_eq!(app.controller.coordinator().redo_len(), 1);
            drop(pressure);
            app.controller.submit("").unwrap();
            let after = app.controller.document().clone();
            app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
            app.controller.redo().unwrap(); assert_eq!(app.controller.document(), &after);
        }

        let mut doc = Document::with_template_tolerance("fit selection stale");
        let shape = Entity::new("native", Geometry::NativeBrep(native.clone()), None);
        let id = shape.id; doc.entities.insert(id, shape);
        let mut app = MatrixApp::new(doc).unwrap();
        app.controller.submit("Point").unwrap(); app.controller.submit("9,0,0").unwrap();
        app.controller.select_objects(&[id]).unwrap();
        app.controller.submit("Sphere FitPoints").unwrap();
        let key = app.controller.sphere_fit_selection_recovery_key().unwrap();
        app.transcript.set_draft("Selection"); app.offer_sphere_selection_memory_recovery(key, "Selection".into());
        app.controller.select_objects(&[]).unwrap(); app.controller.select_objects(&[id]).unwrap();
        let before = app.controller.coordinator().snapshot();
        let undo = app.controller.coordinator().undo_len();
        assert!(matches!(app.retry_after_undo_recovery(), Err(Error::Stale(_))));
        assert_eq!(app.controller.coordinator().snapshot(), before);
        assert_eq!(app.controller.coordinator().undo_len(), undo);

        let key = app.controller.sphere_fit_selection_recovery_key().unwrap();
        app.transcript.set_draft("Selection"); app.offer_sphere_selection_memory_recovery(key, "Selection".into());
        app.transcript.set_draft("changed");
        assert!(matches!(app.retry_after_undo_recovery(), Err(Error::Stale(_))));
        assert_eq!(app.controller.coordinator().snapshot(), before);
        assert_eq!(app.controller.coordinator().undo_len(), undo);

        let mut doc = Document::with_template_tolerance("fit selection exhausted");
        let shape = Entity::new("native", Geometry::NativeBrep(native), None);
        let id = shape.id; doc.entities.insert(id, shape);
        let mut app = MatrixApp::new(doc).unwrap();
        app.controller.submit("Point").unwrap(); app.controller.submit("9,0,0").unwrap();
        app.controller.undo().unwrap();
        assert_eq!(app.controller.coordinator().undo_len(), 0);
        assert_eq!(app.controller.coordinator().redo_len(), 1);
        app.controller.select_objects(&[id]).unwrap();
        app.controller.submit("Sphere FitPoints").unwrap();
        let before = app.controller.document().clone();
        let pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
            MemoryClass::Kernel, None, app.controller.scene_cache_usage().unwrap()).unwrap();
        app.transcript.set_draft("Selection"); app.submit_draft();
        assert!(app.memory_recovery_open);
        assert!(matches!(app.retry_after_undo_recovery(), Err(Error::Budget(_))));
        assert_eq!(app.controller.document(), &before);
        assert_eq!(app.controller.coordinator().redo_len(), 1);
        assert!(app.controller.solid_preview_markers().is_empty());
        drop(pressure);

    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn sphere_fit_selection_append_recovers_without_losing_existing_points() {
        use om9_core::{budget::{Limits, memory::MemoryClass}, geometry::{Geometry, Mesh}, model::Entity};
        let limits = Limits { max_bytes: 64 * 1024 * 1024, ..Limits::default() };
        let mut doc = Document::with_template_tolerance("fit append recovery");
        let old = Entity::new("old mesh", Geometry::Mesh(Mesh { vertices: vec![[1.,1.,1.]; 400000], faces: vec![], normals: vec![], colors: vec![], corner_uvs: vec![] }), None);
        let old_id = old.id; doc.entities.insert(old_id, old);
        let point = Entity::new("point", Geometry::Point([1.,0.,0.]), None);
        let point_id = point.id; doc.entities.insert(point_id, point);
        let mut app = MatrixApp::new(doc.clone()).unwrap();
        app.controller = crate::session::Controller::new(doc, limits).unwrap();
        app.controller.delete_objects(&[old_id]).unwrap();
        app.controller.submit("Point").unwrap(); app.controller.submit("9,0,0").unwrap(); app.controller.undo().unwrap();
        app.controller.select_objects(&[point_id]).unwrap();
        app.controller.submit("Sphere FitPoints").unwrap();
        app.controller.accept_world_point([0.,1.,0.]).unwrap();
        let before = app.controller.document().clone();
        let mut pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
            MemoryClass::Kernel, None, app.controller.scene_cache_usage().unwrap()).unwrap();
        pressure.shrink_to(pressure.bytes() - 64).unwrap();
        app.transcript.set_draft("Selection"); app.submit_draft();
        assert!(app.memory_recovery_open, "{}", app.status_line());
        assert_eq!(app.controller.solid_preview_markers(), vec![[0.,1.,0.]]);
        assert_eq!(app.controller.coordinator().redo_len(), 1);
        assert_eq!(app.retry_after_undo_recovery().unwrap(), 1);
        assert_eq!(app.controller.solid_preview_markers(), vec![[0.,1.,0.], [1.,0.,0.]]);
        assert_eq!(app.controller.document(), &before);
        assert_eq!(app.controller.coordinator().redo_len(), 1);
        drop(pressure);
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn sphere_fit_native_selection_reuses_capacity_without_prompting_to_prune() {
        use om9_core::{budget::{Limits, memory::MemoryClass}, geometry::{Geometry, Mesh}, model::Entity};
        let limits = Limits { max_bytes: 64 * 1024 * 1024, ..Limits::default() };
        let source = om9_kernel::brep::box_brep([4., 5., 6.], 0.1, &limits).unwrap();
        let native = om9_kernel::brep::to_3dm(&source, 0.001, &limits).unwrap();
        let mut doc = Document::with_template_tolerance("fit capacity no prune");
        let mut ids = Vec::new();
        for i in 1..=17u128 {
            let mut point = Entity::new("point", Geometry::Point([i as f64, 0., 0.]), None);
            point.id = om9_core::identity::ObjectId(uuid::Uuid::from_u128(i));
            ids.push(point.id); doc.entities.insert(point.id, point);
        }
        let mut shape = Entity::new("native", Geometry::NativeBrep(native), None);
        shape.id = om9_core::identity::ObjectId(uuid::Uuid::from_u128(100));
        let shape_id = shape.id; ids.push(shape_id); doc.entities.insert(shape_id, shape);
        let old = Entity::new("old mesh", Geometry::Mesh(Mesh { vertices: vec![[1.,1.,1.]; 400000], faces: vec![], normals: vec![], colors: vec![], corner_uvs: vec![] }), None);
        let old_id = old.id; doc.entities.insert(old_id, old);
        let mut app = MatrixApp::new(doc.clone()).unwrap();
        app.controller = crate::session::Controller::new(doc, limits).unwrap();
        app.controller.delete_objects(&[old_id]).unwrap();
        app.controller.submit("Point").unwrap(); app.controller.submit("100,100,100").unwrap();
        app.controller.undo().unwrap();
        app.controller.select_objects(&[shape_id]).unwrap();
        let mut minimum = None;
        for slack in (2048..=16384).step_by(128) {
            let mut pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
                MemoryClass::Kernel, None, app.controller.scene_cache_usage().unwrap()).unwrap();
            pressure.shrink_to(pressure.bytes() - slack).unwrap();
            let result = app.controller.solid_fit_selection_points();
            drop(pressure);
            if result.is_ok() { minimum = Some(slack); break; }
        }
        let slack = minimum.expect("single source must fit") + 32 * std::mem::size_of::<[f64; 3]>() as u64 + 128;
        app.controller.select_objects(&ids).unwrap();
        app.controller.submit("Sphere FitPoints").unwrap();
        let before = app.controller.document().clone();
        let undo = app.controller.coordinator().undo_len();
        assert!(undo > 0);
        assert_eq!(app.controller.coordinator().redo_len(), 1);
        let mut pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
            MemoryClass::Kernel, None, app.controller.scene_cache_usage().unwrap()).unwrap();
        pressure.shrink_to(pressure.bytes() - slack).unwrap();
        app.transcript.set_draft("Selection"); app.submit_draft();
        assert!(!app.memory_recovery_open, "unnecessary recovery offer: {}", app.status_line());
        assert_eq!(app.controller.solid_preview_markers().len(), 25);
        assert_eq!(app.controller.document(), &before);
        assert_eq!(app.controller.coordinator().undo_len(), undo);
        assert_eq!(app.controller.coordinator().redo_len(), 1);
        drop(pressure);
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn sphere_fit_selection_recovery_keeps_prompt_and_clears_submitted_draft() {
        use om9_core::{budget::Limits, geometry::Geometry, model::Entity};
        let limits = Limits::default();
        let source = om9_kernel::brep::box_brep([4., 5., 6.], 0.1, &limits).unwrap();
        let native = om9_kernel::brep::to_3dm(&source, 0.001, &limits).unwrap();
        let entity = Entity::new("native", Geometry::NativeBrep(native), None);
        let id = entity.id;
        let mut doc = Document::with_template_tolerance("fit selection retry UI");
        doc.entities.insert(id, entity);
        let mut app = MatrixApp::new(doc).unwrap();
        app.controller.select_objects(&[id]).unwrap();
        app.controller.submit("Sphere FitPoints").unwrap();
        let key = app.controller.sphere_fit_selection_recovery_key().unwrap();
        app.transcript.set_draft("Selection");
        app.offer_sphere_selection_memory_recovery(key, "Selection".into());
        app.memory_recovery_running = true;
        let context = egui::Context::default();
        let mut out = context.run_ui(egui::RawInput::default(), |ui| app.memory_recovery_ui(ui.ctx()));
        out.textures_delta.clear();
        assert!(!app.memory_recovery_open);
        assert_eq!(app.transcript.draft(), "");
        assert!(app.status_line().contains("selection loaded"), "{}", app.status_line());
        assert!(app.controller.is_running());
        assert_eq!(app.controller.solid_preview_markers().len(), 8);
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn sphere_fit_points_retry_reclaims_history_memory() {
        use om9_core::{budget::{Limits, memory::MemoryClass}, geometry::{Geometry, Mesh}, model::Entity};
        let mut doc = Document::with_template_tolerance("fit points history recovery");
        let old = Entity::new("old mesh", Geometry::Mesh(Mesh { vertices: vec![[1.,1.,1.]; 400000], faces: vec![], normals: vec![], colors: vec![], corner_uvs: vec![] }), None);
        let old_id = old.id; doc.entities.insert(old_id, old);
        let mut app = MatrixApp::new(doc.clone()).unwrap();
        app.controller = crate::session::Controller::new(doc, Limits { max_bytes: 64 * 1024 * 1024, ..Default::default() }).unwrap();
        app.controller.delete_objects(&[old_id]).unwrap();
        app.controller.submit("Point").unwrap(); app.controller.submit("9,0,0").unwrap();
        app.controller.submit("Sphere FitPoints").unwrap();
        for point in [[5.,0.,0.],[-5.,0.,0.],[0.,5.,0.],[0.,0.,5.],[0.,-5.,0.]] { app.controller.accept_world_point(point).unwrap(); }
        let before = app.controller.document().clone(); let key = app.controller.sphere_recovery_key();
        let mut pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
            MemoryClass::Kernel, None, app.controller.scene_cache_usage().unwrap()).unwrap();
        pressure.shrink_to(pressure.bytes() - 2048).unwrap();
        app.transcript.set_draft(""); app.submit_draft();
        assert!(app.memory_recovery_open); assert_eq!(app.controller.sphere_recovery_key(), key);
        assert_eq!(app.retry_after_undo_recovery().unwrap(), 1);
        let sphere = app.controller.document().entities.iter().find(|(id, _)| !before.entities.contains_key(id)).unwrap().1;
        let Geometry::Brep(brep) = &sphere.geometry else { panic!("sphere BRep") };
        let measure = om9_kernel::creation::measure(brep, app.controller.coordinator().limits()).unwrap();
        assert!((measure.volume_mm3 - 4. * std::f64::consts::PI * 125. / 3.).abs() < 1e-6);
        for axis in 0..3 { assert!(sphere.placement.0[axis][3].abs() < 1e-8); }
        drop(pressure); app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn sphere_fit_points_retry_rejects_replaced_later_point() {
        let mut app = MatrixApp::new(Document::with_template_tolerance("fit points changed")).unwrap();
        app.controller.submit("Point").unwrap(); app.controller.submit("9,0,0").unwrap();
        app.controller.submit("Sphere FitPoints").unwrap();
        for point in [[5.,0.,0.],[-5.,0.,0.],[0.,5.,0.],[0.,0.,5.],[0.,-5.,0.]] { app.controller.accept_world_point(point).unwrap(); }
        let key = app.controller.sphere_recovery_key().unwrap(); app.offer_sphere_memory_recovery(key, "".into());
        app.controller.submit("Undo").unwrap(); app.controller.accept_world_point([0.,0.,-5.]).unwrap();
        let before = app.controller.coordinator().snapshot();
        assert!(matches!(app.retry_after_undo_recovery(), Err(Error::Stale(_))));
        assert_eq!(app.controller.coordinator().snapshot(), before);
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn sphere_recovery_refuses_replaced_session_without_removing_undo() {
        let mut app = MatrixApp::new(Document::with_template_tolerance("stale sphere")).unwrap();
        app.controller.submit("Point").unwrap(); app.controller.submit("1,2,3").unwrap();
        app.controller.submit("Sphere").unwrap(); app.controller.submit("10,20,30").unwrap();
        let key = app.controller.sphere_recovery_key().unwrap(); app.offer_sphere_memory_recovery(key, "5".into());
        app.controller.cancel(); app.controller.submit("Sphere").unwrap(); app.controller.submit("10,20,30").unwrap();
        let before = app.controller.coordinator().snapshot();
        assert!(matches!(app.retry_after_undo_recovery(), Err(Error::Stale(_))));
        assert_eq!(app.controller.coordinator().snapshot(), before);
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn sphere_recovery_refuses_changed_draft_without_removing_undo() {
        let mut app = MatrixApp::new(Document::with_template_tolerance("changed radius draft")).unwrap();
        app.controller.submit("Point").unwrap(); app.controller.submit("1,2,3").unwrap();
        app.controller.submit("Sphere").unwrap(); app.controller.submit("10,20,30").unwrap();
        let key = app.controller.sphere_recovery_key().unwrap();
        app.transcript.set_draft("Radius=5"); app.offer_sphere_memory_recovery(key, "Radius=5".into());
        app.transcript.set_draft("Radius=7"); let before = app.controller.coordinator().snapshot();
        assert!(matches!(app.retry_after_undo_recovery(), Err(Error::Stale(_))));
        assert_eq!(app.controller.coordinator().snapshot(), before);
        assert_eq!(app.transcript.draft(), "Radius=7");
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn sphere_recovery_refuses_replaced_controller_with_matching_input() {
        let doc = Document::with_template_tolerance("reopened sphere");
        let mut app = MatrixApp::new(doc.clone()).unwrap();
        app.controller.submit("Point").unwrap(); app.controller.submit("1,2,3").unwrap();
        app.controller.submit("Sphere").unwrap(); app.controller.submit("10,20,30").unwrap();
        let key = app.controller.sphere_recovery_key().unwrap(); app.offer_sphere_memory_recovery(key, "5".into());
        app.controller = crate::session::Controller::new(doc, Default::default()).unwrap();
        app.controller.submit("Point").unwrap(); app.controller.submit("1,2,3").unwrap();
        app.controller.submit("Sphere").unwrap(); app.controller.submit("10,20,30").unwrap();
        assert_eq!(app.controller.sphere_recovery_key(), Some(key));
        let before = app.controller.coordinator().snapshot();
        assert!(matches!(app.retry_after_undo_recovery(), Err(Error::Stale(_))));
        assert_eq!(app.controller.coordinator().snapshot(), before);
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn sphere_recovery_reclaims_history_mesh_and_preserves_result() {
        use om9_core::{budget::{Limits, memory::MemoryClass}, geometry::{Geometry, Mesh}, model::Entity};
        let mut doc = Document::with_template_tolerance("sphere history recovery");
        let old = Entity::new("old mesh", Geometry::Mesh(Mesh { vertices: vec![[1.,1.,1.]; 400000], faces: vec![], normals: vec![], colors: vec![], corner_uvs: vec![] }), None);
        let old_id = old.id; doc.entities.insert(old_id, old);
        let mut app = MatrixApp::new(doc.clone()).unwrap();
        app.controller = crate::session::Controller::new(doc, Limits { max_bytes: 64 * 1024 * 1024, ..Default::default() }).unwrap();
        app.controller.delete_objects(&[old_id]).unwrap();
        app.controller.submit("Point").unwrap(); app.controller.submit("9,0,0").unwrap();
        app.controller.submit("Sphere").unwrap(); app.controller.submit("10,20,30").unwrap();
        let before = app.controller.document().clone();
        let mut pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
            MemoryClass::Kernel, None, app.controller.scene_cache_usage().unwrap()).unwrap();
        pressure.shrink_to(pressure.bytes() - 2048).unwrap();
        app.transcript.set_draft("5"); app.submit_draft(); assert!(app.memory_recovery_open);
        assert_eq!(app.retry_after_undo_recovery().unwrap(), 1);
        let sphere = app.controller.document().entities.iter().find(|(id, _)| !before.entities.contains_key(id)).unwrap().1;
        let Geometry::Brep(brep) = &sphere.geometry else { panic!("sphere BRep") };
        let measure = om9_kernel::creation::measure(brep, app.controller.coordinator().limits()).unwrap();
        assert!((measure.volume_mm3 - 4. * std::f64::consts::PI * 125. / 3.).abs() < 1e-6);
        assert_eq!(sphere.placement, om9_core::geometry::Transform::translation([10.,20.,30.]));
        drop(pressure); app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn sphere_recovery_exhaustion_keeps_center_and_redo() {
        use om9_core::budget::memory::MemoryClass;
        let mut app = MatrixApp::new(Document::with_template_tolerance("sphere memory still full")).unwrap();
        for p in ["1,0,0", "2,0,0"] { app.controller.submit("Point").unwrap(); app.controller.submit(p).unwrap(); }
        app.controller.undo().unwrap(); app.controller.submit("Sphere").unwrap(); app.controller.submit("10,20,30").unwrap();
        let before = app.controller.document().clone(); let key = app.controller.sphere_recovery_key();
        let mut pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
            MemoryClass::Kernel, None, app.controller.scene_cache_usage().unwrap()).unwrap();
        pressure.shrink_to(pressure.bytes() - 2048).unwrap();
        app.transcript.set_draft("5"); app.submit_draft(); assert!(app.memory_recovery_open);
        assert!(app.retry_after_undo_recovery().is_err());
        assert_eq!(app.controller.document(), &before); assert_eq!(app.controller.sphere_recovery_key(), key);
        assert_eq!((app.controller.coordinator().undo_len(), app.controller.coordinator().redo_len()), (0, 1));
        drop(pressure); app.controller.submit("5").unwrap();
        assert_eq!(app.controller.document().entities.len(), before.entities.len() + 1);
    }
    #[test]
    fn memory_recovery_stops_after_history_exhausted_and_keeps_redo() {
        use om9_core::{geometry::{Geometry, Mesh}, model::Entity};
        let mut doc = Document::new("memory still full");
        let mesh = Mesh { vertices: vec![[0.,0.,0.], [2.,0.,0.], [0.,2.,0.]], faces: vec![vec![0,1,2]], normals: vec![], colors: vec![], corner_uvs: vec![] };
        let entity = Entity::new("mesh", Geometry::Mesh(mesh), None); let id = entity.id; doc.entities.insert(id, entity);
        let mut app = MatrixApp::new(doc).unwrap();
        for p in ["1,0,0", "2,0,0"] { app.controller.submit("Point").unwrap(); app.controller.submit(p).unwrap(); }
        app.controller.undo().unwrap(); app.controller.select_objects(&[id]).unwrap();
        let before = app.controller.document().clone();
        let pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::Kernel, None, app.controller.scene_cache_usage().unwrap()).unwrap();
        app.transcript.set_draft("ExtractWireframe"); app.submit_draft(); assert!(app.memory_recovery_open);
        assert!(app.retry_after_undo_recovery().is_err());
        assert_eq!(app.controller.document(), &before);
        assert_eq!((app.controller.coordinator().undo_len(), app.controller.coordinator().redo_len()), (0, 1));
        drop(pressure); app.controller.redo().unwrap();
    }
    #[test]
    fn memory_recovery_retries_after_releasing_history_owned_mesh() {
        use om9_core::{budget::memory::MemoryClass, geometry::{Geometry, Mesh}, model::Entity};
        let mut doc = Document::new("history memory recovery");
        let mesh = |vertices, faces| Mesh { vertices, faces, normals: vec![], colors: vec![], corner_uvs: vec![] };
        let target = Entity::new("target", Geometry::Mesh(mesh(vec![[0.,0.,0.],[2.,0.,0.],[0.,2.,0.]], vec![vec![0,1,2]])), None);
        let target_id = target.id; doc.entities.insert(target_id, target);
        let old = Entity::new("old", Geometry::Mesh(mesh(vec![[1.,1.,1.]; 50000], vec![])), None);
        let old_id = old.id; doc.entities.insert(old_id, old);
        let mut app = MatrixApp::new(doc.clone()).unwrap();
        app.controller = crate::session::Controller::new(doc, om9_core::budget::Limits {
            max_bytes: 32 * 1024 * 1024, ..Default::default()
        }).unwrap();
        app.controller.delete_objects(&[old_id]).unwrap();
        app.controller.submit("Point").unwrap(); app.controller.submit("9,0,0").unwrap();
        app.controller.select_objects(&[target_id]).unwrap();
        let before = app.controller.document().clone();
        let pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
            MemoryClass::Kernel, None, app.controller.scene_cache_usage().unwrap()).unwrap();
        app.transcript.set_draft("ExtractWireframe"); app.submit_draft();
        assert!(app.memory_recovery_open, "initial extraction must be refused");
        assert_eq!(app.retry_after_undo_recovery().unwrap(), 1, "the oldest history held enough memory");
        assert_eq!(app.controller.coordinator().undo_len(), 2);
        assert_eq!(app.controller.document().entities.len(), before.entities.len() + 3);
        drop(pressure);
        app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
    }
    #[cfg(windows)]
    #[test]
    fn saved_budget_above_half_ram_is_rejected_without_changing_active_policy() {
        let mut app = MatrixApp::new(Document::new("RAM cap")).unwrap();
        let max = app
            .controller
            .coordinator()
            .undo_ram_limit_mb()
            .expect("Windows RAM query");
        let current = app.controller.coordinator().undo_policy();
        let path = std::env::temp_dir().join(format!("om9-undo-cap-{}.json", uuid::Uuid::new_v4()));
        let excessive = UndoPolicy {
            minimum_steps: 1,
            max_memory_mb: max + 1,
        };
        std::fs::write(&path, serde_json::to_vec(&excessive).unwrap()).unwrap();
        assert!(app.load_undo_settings(&path).is_err());
        assert_eq!(app.controller.coordinator().undo_policy(), current);
        std::fs::remove_file(path).unwrap();
    }
    fn frame(app: &mut MatrixApp, ctx: &egui::Context, events: Vec<egui::Event>) {
        let mut out = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, Vec2::new(900., 600.))),
                focused: true,
                events,
                ..Default::default()
            },
            |ui| {
                app.undo_settings_ui(ui.ctx());
            },
        );
        out.textures_delta.clear();
    }
    #[test]
    fn options_apply_by_click_persists_without_editing_document_and_rejects_corruption() {
        let path = std::env::temp_dir().join(format!("om9-undo-{}.json", uuid::Uuid::new_v4()));
        let ctx = egui::Context::default();
        let mut app = MatrixApp::new(Document::new("Undo settings")).unwrap();
        app.load_undo_settings(&path).unwrap();
        let before = app.controller.coordinator().snapshot();
        app.transcript.set_draft("Options");
        app.submit_draft();
        assert_eq!(app.undo_settings_draft, Some(UndoPolicy::default()));
        let policy = UndoPolicy {
            minimum_steps: 7,
            max_memory_mb: 64,
        };
        app.undo_settings_draft = Some(policy);
        for _ in 0..3 {
            frame(&mut app, &ctx, vec![]);
        }
        let pos = ctx
            .data(|m| m.get_temp::<Rect>(egui::Id::new("undo-apply-rect")))
            .unwrap()
            .center();
        for pressed in [true, false] {
            frame(
                &mut app,
                &ctx,
                vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: Modifiers::NONE,
                    },
                ],
            );
        }
        assert!(app.undo_settings_draft.is_none());
        assert_eq!(app.controller.coordinator().undo_policy(), policy);
        assert_eq!(app.controller.coordinator().snapshot(), before);
        let mut reopened = MatrixApp::new(Document::new("reopened")).unwrap();
        reopened.load_undo_settings(&path).unwrap();
        assert_eq!(reopened.controller.coordinator().undo_policy(), policy);
        std::fs::write(&path, b"{bad json").unwrap();
        assert!(reopened.load_undo_settings(&path).is_err());
        assert_eq!(reopened.controller.coordinator().undo_policy(), policy);
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn options_sidebar_icon_opens_the_same_policy_and_close_does_not_apply() {
        let ctx = egui::Context::default();
        let mut app = MatrixApp::new(Document::new("Options icon")).unwrap();
        app.icon_column = IconColumn::Left;
        let mut pos = egui::Pos2::ZERO;
        for step in 0..4 {
            app.rail_icon_rects.clear();
            let events = if step == 1 || step == 2 {
                vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed: step == 1,
                        modifiers: Modifiers::NONE,
                    },
                ]
            } else {
                vec![]
            };
            let mut out = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, Vec2::new(400., 200.))),
                    events,
                    focused: true,
                    ..Default::default()
                },
                |ui| app.fixed_sidebar_palette(ui, SectionId::InfoSettings),
            );
            out.textures_delta.clear();
            if step == 0 {
                pos = app.rail_icon_rects[0].center();
            }
        }
        assert_eq!(app.undo_settings_draft, Some(UndoPolicy::default()));
        app.undo_settings_draft = None;
        assert_eq!(
            app.controller.coordinator().undo_policy(),
            UndoPolicy::default()
        );
    }
    #[test]
    fn undo_preferences_survive_new_and_master_restore() {
        let mut c = Controller::new(Document::new("before"), Limits::default()).unwrap();
        let policy = UndoPolicy {
            minimum_steps: 5,
            max_memory_mb: 32,
        };
        c.set_undo_policy(policy).unwrap();
        c.restore_master_document(Document::new("restored"))
            .unwrap();
        assert_eq!(c.coordinator().undo_policy(), policy);
        c.begin_file_operation(crate::file_lifecycle::FileKind::New)
            .unwrap();
        assert_eq!(c.coordinator().undo_policy(), policy);
    }
}
