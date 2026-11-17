//! Opt-in native-window replay with command markers on the application's clock.
use super::*;
use std::time::{Duration, Instant};
use crate::display::ViewportDetail;

pub(super) struct SphereBooleanAudit {
    detail: ViewportDetail,
    origin: crate::session::InvocationOrigin,
    started: Instant,
    previous: Option<Instant>,
    deadline: Instant,
    stage: u8,
    frame: u64,
    done: bool,
    owns_throwaway: bool,
    pointer_navigation: bool,
    dragging: bool,
    motion_started: Option<Instant>,
    pairs: usize,
    coupled: bool,
    resources: bool,
    timeout: Duration,
    created: Vec<om9_core::identity::ObjectId>,
    temporary_ready: bool,
    temporary_presented: bool,
    recovery_probe_pending: bool,
    viewport_only: bool,
    proof_requested: bool,
    proof_saved: bool,
    presented_count: usize,
    last_commit: Option<Instant>,
    verification_count: usize,
    verification_history: usize,
    verification_cursor: usize,
    verification_bodies: Vec<serde_json::Value>,
    verification_guard: Option<(om9_core::identity::DocumentId, u64, u64)>,
}
impl SphereBooleanAudit {
    pub(super) fn from_env() -> Option<Self> {
        let value = std::env::var("OM9_SPHERE_BOOLEAN_AUDIT").ok()?;
        let detail = match value.as_str() { "light" => ViewportDetail::Light, "high" => ViewportDetail::High,
            _ => { eprintln!("OM9_SPHERE_BOOLEAN_AUDIT must be light or high"); return None; } };
        let now = Instant::now();
        let pairs = match std::env::var("OM9_SPHERE_BOOLEAN_AUDIT_PAIRS") {
            Ok(value) => match value.parse::<usize>() {
                Ok(pairs @ 1..=512) => pairs,
                _ => { eprintln!("OM9_SPHERE_BOOLEAN_AUDIT_PAIRS must be 1..512"); return None; }
            },
            Err(_) => 1,
        };
        Some(Self { detail, origin: if std::env::var("OM9_SPHERE_BOOLEAN_AUDIT_ORIGIN").is_ok_and(|s|s=="menu") {
            crate::session::InvocationOrigin::Menu } else { crate::session::InvocationOrigin::Cmd },
            started: now, previous: None, deadline: now + Duration::from_secs(5), stage: 0, frame: 0, done: false, owns_throwaway: false,
            pointer_navigation: std::env::var_os("OM9_SPHERE_BOOLEAN_AUDIT_POINTER").is_some(), dragging: false, motion_started: None,
            pairs, coupled: std::env::var("OM9_SPHERE_BOOLEAN_AUDIT_ARCHITECTURE").is_ok_and(|s|s=="coupled"),
            resources: std::env::var_os("OM9_SPHERE_BOOLEAN_AUDIT_RESOURCES").is_some(),
            timeout: Duration::from_secs(std::env::var("OM9_SPHERE_BOOLEAN_AUDIT_TIMEOUT_SECS").ok()
                .and_then(|s|s.parse::<u64>().ok()).unwrap_or(if pairs>1 {600} else {90}).clamp(30,14400)),
            created: Vec::new(), temporary_ready: false, temporary_presented: false,
            viewport_only: std::env::var_os("OM9_SPHERE_VIEWPORT_AUDIT").is_some(),
            proof_requested: false, proof_saved: false,
            presented_count: 0, last_commit: None,
            verification_count: 0, verification_history: 0, verification_cursor: 0,
            verification_bodies: Vec::new(), verification_guard: None,
            recovery_probe_pending: std::env::var_os("OM9_SPHERE_BOOLEAN_AUDIT_RECOVERY_PROBE").is_some() })
    }
    pub(super) fn can_discard(&self) -> bool { self.done && self.owns_throwaway }
    fn record(&self, app: &mut MatrixApp, event: &str, extra: serde_json::Value) {
        if let Some(log) = &mut app.interaction_log {
            log.record(serde_json::json!({"event":event,"profile":self.detail,"stage":self.stage,"audit_frame":self.frame,
                "elapsed_ms":self.started.elapsed().as_secs_f64()*1000.,"data":extra}));
        }
    }
    fn resources(&self, app: &mut MatrixApp, point: &str, exact_history: bool) {
        if !self.resources { return; }
        let c=app.controller.coordinator();
        let stats=c.undo_memory_stats();
        let process=om9_native_owner::process_memory().map(|m|serde_json::json!({
            "working_set_bytes":m.working_set_bytes,"peak_working_set_bytes":m.peak_working_set_bytes,
            "private_bytes":m.private_bytes,"peak_pagefile_bytes":m.peak_pagefile_bytes}));
        let physical=om9_native_owner::physical_memory().map(|m|serde_json::json!({
            "total_bytes":m.total_bytes,"available_bytes":m.available_bytes}));
        let exact=exact_history.then(||match c.undo_memory_bytes() {
            Ok(bytes)=>serde_json::json!({"bytes":bytes}),Err(e)=>serde_json::json!({"error":e.to_string()}) });
        let data=serde_json::json!({"point":point,"objects":app.controller.document().entities.len(),
            "created":self.created.len(),"undo_steps":c.undo_len(),"redo_steps":c.redo_len(),
            "undo_disabled":c.benchmark_no_undo(),"undo_policy":c.undo_policy(),
            "undo_estimated_bytes":stats.estimated_bytes,"undo_inventory_pending":stats.capture_change_pending,
            "undo_full_inventories":stats.full_inventories,"undo_exact_inventory":exact,
            "undo_deferred_attribution_refreshes":stats.deferred_attribution_refreshes,
            "process":process,"physical":physical,"limits":c.limits()});
        self.record(app,"sphere_audit_resources",data);
    }
    fn command(app: &mut MatrixApp, text: &str) {
        app.transcript.set_draft(text); app.submit_draft();
    }
    fn start_command(&self, app: &mut MatrixApp, command: &str, feature: &str) {
        if self.origin == crate::session::InvocationOrigin::Menu { app.invoke(feature, self.origin); }
        else { Self::command(app, command); }
    }
    fn metrics(&self, app: &mut MatrixApp) -> Result<serde_json::Value> {
        let mut bodies = Vec::new();
        for entity in app.controller.document().entities.values() {
            if matches!(&entity.geometry, om9_core::geometry::Geometry::Brep(_)) {
                bodies.push(self.metric_body(app, entity.id)?);
            }
        }
        Ok(serde_json::json!({"bodies":bodies,"objects":app.controller.document().entities.len()}))
    }
    fn metric_body(&self, app: &MatrixApp, id: om9_core::identity::ObjectId) -> Result<serde_json::Value> {
        let limits = app.controller.coordinator().limits();
        let entity = app.controller.document().entities.get(&id).ok_or(Error::Missing("audit entity"))?;
        if let om9_core::geometry::Geometry::Brep(body) = &entity.geometry {
                let measures = om9_kernel::creation::measure(body, limits).map_err(|e| Error::Io(e.to_string()))?;
                let display = app.controller.scene().ok_or(Error::Missing("audit scene"))?
                    .root_brep_pick_display(&app.controller.document().stamp(entity.id)?)?;
                return Ok(serde_json::json!({"id":entity.id,"triangles":display.triangles.len(),"points":display.points.len(),
                    "volume_mm3":measures.volume_mm3,"closed":measures.closed,"solids":measures.solids,"naked_edges":measures.naked_edges,
                    "bounds_mm":measures.bounds_mm,"placement":entity.placement.0,"coupled_display":body.has_coupled_display_diagnostic(),
                    "coupled_validations":body.coupled_diagnostic_validation_count(),"source_sha256":body.canonical_fingerprints().0}));
        }
        Err(Error::Invalid("audit entity is not a canonical BRep"))
    }
    fn verification_stamp(app: &MatrixApp) -> (om9_core::identity::DocumentId, u64, u64) {
        let doc = app.controller.document();
        (doc.id, doc.revision, app.controller.coordinator().epoch())
    }
    // Verification is outside performance_end. Yield to the native event loop
    // between Undo, Redo, each native source measurement and the final census.
    fn verification_step(&mut self, app: &mut MatrixApp, ctx: &egui::Context) -> Result<bool> {
        if matches!(self.stage, 11 | 12) {
            if app.controller.history_worker_pending() { return Ok(false); }
            let before = self.verification_guard.ok_or(Error::Missing("audit history guard"))?;
            let after = Self::verification_stamp(app);
            if before.0 != after.0 || before.2.checked_add(1) != Some(after.2) {
                return Err(Error::Stale("audit history worker did not publish exactly once"));
            }
            if self.stage == 11 {
                if app.controller.document().entities.len() != self.verification_count - 1 {
                    return Err(Error::Invalid("viewport Undo count"));
                }
                self.stage = 8;
            } else {
                if app.controller.document().entities.len() != self.verification_count
                    || app.controller.coordinator().undo_len() != self.verification_history {
                    return Err(Error::Invalid("viewport Undo/Redo failed"));
                }
                self.record(app,"sphere_viewport_undo_redo",serde_json::json!({"objects":self.verification_count,"undo":self.verification_history}));
                self.stage = 9;
            }
            self.verification_guard = Some(after);
            return Ok(false);
        }
        if self.verification_guard != Some(Self::verification_stamp(app)) {
            return Err(Error::Stale("audit document changed during verification"));
        }
        match self.stage {
            7 => {
                if self.verification_count == 0 { return Err(Error::Invalid("empty viewport audit")); }
                if !app.request_history_ui(ctx, false) {
                    return Err(Error::Invalid("UI Undo request failed"));
                }
                self.stage = 11;
            }
            8 => {
                if !app.request_history_ui(ctx, true) {
                    return Err(Error::Invalid("UI Redo request failed"));
                }
                self.stage = 12;
            }
            9 => {
                if let Some(id) = self.created.get(self.verification_cursor).copied() {
                    self.verification_bodies.push(self.metric_body(app, id)?);
                    self.verification_cursor += 1;
                } else {
                    let bodies = std::mem::take(&mut self.verification_bodies);
                    self.record(app,"sphere_audit_result",serde_json::json!({"objects":self.verification_count,"bodies":bodies}));
                    self.stage = 10;
                }
            }
            10 => {
                if app.gpu_viewports.is_empty() || !app.gpu_viewports.values().all(|p|p.final_complete()) {
                    return Ok(false);
                }
                self.resources(app,"after_verification",true);
                self.record(app,"sphere_audit_end",serde_json::json!({}));
                return Ok(true);
            }
            _ => return Err(Error::Invalid("audit verification stage")),
        }
        self.verification_guard = Some(Self::verification_stamp(app));
        Ok(false)
    }
    // A lattice of independent intersecting pairs has an analytic Union oracle.
    // One normal Sphere command per native frame leaves the real UI/render work
    // in the measurement; source measurements run only after timed navigation.
    fn many_scene_step(&mut self, app: &mut MatrixApp) -> Result<()> {
        match self.stage {
            0 => { self.stage=1; self.deadline=Instant::now(); }
            1 => {
                if self.viewport_only && self.presented_count < self.created.len() {
                    let scene=app.controller.scene().ok_or(Error::Missing("viewport presentation scene"))?;
                    let complete=app.gpu_viewports.len()==4 && app.gpu_viewports.iter().all(|(id,port)| {
                        let view=&app.controller.workspace.views[id];
                        port.final_complete() && port.displays_content(scene,app.controller.selection_state().revision,
                            app.controller.preview_revision(),view.cplane.revision,None,view.mode)
                            && port.published_frame_stats().is_some_and(|s|s.objects==self.created.len() && s.visible>0)
                    });
                    if !complete {
                        if self.last_commit.is_some_and(|t|t.elapsed()>Duration::from_secs(15)) {
                            return Err(Error::Invalid("sphere presentation exceeded 15 seconds"));
                        }
                        return Ok(());
                    }
                    self.presented_count=self.created.len();
                    self.record(app,"sphere_viewport_presented",serde_json::json!({"objects":self.presented_count,
                        "commit_to_present_ms":self.last_commit.map(|t|t.elapsed().as_secs_f64()*1000.)}));
                    if self.created.len()==self.pairs*2 {
                        self.stage=2;self.deadline=Instant::now()+Duration::from_secs(8);
                        return Ok(());
                    }
                }
                let index=self.created.len();
                let columns=(self.pairs as f64).sqrt().ceil() as usize;
                let rows=self.pairs.div_ceil(columns);
                let pair=index/2;
                let x=(pair%columns) as f64*20.-(columns-1) as f64*10.+(index%2) as f64*5.;
                let y=(pair/columns) as f64*16.-(rows-1) as f64*8.;
                self.record(app,"sphere_command_start",serde_json::json!({"index":index+1,"center_mm":[x,y,0.]}));
                if std::env::var_os("OM9_COMMIT_PROFILE").is_some() {eprintln!("OM9_SPHERE_PROFILE begin index={} elapsed_ms={:.3} thread={:?}",index+1,self.started.elapsed().as_secs_f64()*1000.,std::thread::current().id());}
                self.start_command(app,"Sphere","OM9-SOLID-014");
                Self::command(app,&format!("{x},{y},0")); Self::command(app,"5");
                if app.controller.document().entities.len()!=index+1 || app.controller.is_running() {
                    return Err(Error::Invalid("many-sphere command did not commit"));
                }
                let id=app.controller.document().entities.keys().find(|id|!self.created.contains(id)).copied()
                    .ok_or(Error::Missing("new sphere identity"))?;
                self.created.push(id);
                self.last_commit=Some(Instant::now());
                self.record(app,"sphere_command_end",serde_json::json!({"index":index+1}));
                if std::env::var_os("OM9_COMMIT_PROFILE").is_some() {eprintln!("OM9_SPHERE_PROFILE end index={} elapsed_ms={:.3} thread={:?}",index+1,self.started.elapsed().as_secs_f64()*1000.,std::thread::current().id());}
                if self.created.len()%64==0 { self.resources(app,"creation_progress",false); }
                if self.created.len()==self.pairs*2 && !self.viewport_only {
                    let radius=(((columns-1) as f64*10.+7.5).powi(2)
                        +((rows-1) as f64*8.+5.).powi(2)+25.).sqrt();
                    for (id,view) in app.controller.workspace.views.iter_mut() {
                        let aspect=app.view_rects.get(id).map_or(1.,|r|f64::from(r.width()/r.height().max(1.)));
                        view.camera.fit_bounds([2.5,0.,0.],radius,aspect,1.1)?;
                    }
                    self.stage=2;self.deadline=Instant::now()+Duration::from_secs(8);
                }
            }
            2 => {
                let scene=app.controller.scene().ok_or(Error::Missing("audit scene"))?;
                let bodies=self.created.iter().map(|id| {
                    let display=scene.root_brep_pick_display(&app.controller.document().stamp(*id)?)?;
                    let coupled = matches!(&app.controller.document().entities[id].geometry,
                        om9_core::geometry::Geometry::Brep(source) if source.has_coupled_display_diagnostic());
                    let validations = match &app.controller.document().entities[id].geometry {
                        om9_core::geometry::Geometry::Brep(source) => source.coupled_diagnostic_validation_count(), _=>0 };
                    Ok(serde_json::json!({"triangles":display.triangles.len(),"coupled_display":coupled,"coupled_validations":validations}))
                }).collect::<Result<Vec<_>>>()?;
                self.record(app,"sphere_audit_inputs",serde_json::json!({"objects":bodies.len(),"bodies":bodies}));
                app.controller.select_objects(&self.created)?;
                self.record(app,"sphere_selection_start",serde_json::json!({"objects":self.created.len()}));
                self.stage=6;self.deadline=Instant::now()+Duration::from_secs(8);
            }
            6 => {
                self.record(app,"sphere_selection_end",serde_json::json!({"objects":self.created.len()}));
                if self.viewport_only {
                    self.stage=5;self.deadline=Instant::now()+Duration::from_secs(1);
                    return Ok(());
                }
                self.resources(app,"before_boolean",false);
                self.record(app,"boolean_command_start",serde_json::json!({"operation":"BooleanUnion","operands":self.created.len()}));
                self.start_command(app,"BooleanUnion","OM9-SOLID-003");
                Self::command(app,"DeleteInput=Yes");Self::command(app,"");
                self.record(app,"boolean_request_return",serde_json::json!({"capture":app.controller.boolean_union_request_capture()}));
                self.resources(app,"after_request",false);
                if !app.controller.boolean_union_worker_pending() { return Err(Error::Missing("many-sphere Union worker")); }
                self.stage=3;
            }
            3 => {
                if let Some((id,compute_ms,triangles)) = app.controller.boolean_mesh_preview_diagnostics() {
                    if !self.temporary_ready {
                        self.record(app,"boolean_temporary_ready",serde_json::json!({"id":id,"compute_ms":compute_ms,"triangles":triangles}));
                        self.resources(app,"temporary_ready",false);
                        self.temporary_ready = true;
                    }
                    let view = &app.controller.workspace.views[&ViewId::Perspective];
                    let presented = app.controller.scene().is_some_and(|scene| app.gpu_viewports.get(&ViewId::Perspective)
                        .is_some_and(|port| port.displays_content(scene, app.controller.selection_state().revision,
                            app.controller.preview_revision(), view.cplane.revision, app.controller.solid_boolean_two_hidden_sources(), view.mode)));
                    if presented && !self.temporary_presented {
                        self.record(app,"boolean_temporary_presented",serde_json::json!({"id":id,"view":"Perspective"}));
                        self.temporary_presented = true;
                    }
                }
                if app.controller.boolean_union_worker_pending() { return Ok(()); }
                if app.controller.document().entities.len()!=1 || app.controller.is_running() {
                    return Err(Error::Invalid("many-sphere Union did not commit"));
                }
                self.record(app,"boolean_commit_end",serde_json::json!({"objects":1}));
                self.resources(app,"after_boolean",false);
                self.stage=4;self.deadline=Instant::now()+Duration::from_secs(8);
            }
            4 => { self.stage=5;self.deadline=Instant::now()+Duration::from_secs(1); }
            _ => (),
        }
        Ok(())
    }
    pub(super) fn frame(&mut self, app: &mut MatrixApp, ctx: &egui::Context, raw: &mut egui::RawInput) {
        if self.done { return; }
        if self.viewport_only {
            if self.proof_requested && !self.proof_saved {
                for event in &raw.events {
                    if let egui::Event::Screenshot { image, .. } = event
                        && let Some(path)=std::env::var_os("OM9_VIEWPORT_SCREENSHOT_PATH") {
                        let pixels:Vec<u8>=image.pixels.iter().flat_map(|p|p.to_array()).collect();
                        let result=image::RgbaImage::from_raw(image.size[0] as u32,image.size[1] as u32,pixels)
                            .ok_or_else(||"screenshot size".to_owned()).and_then(|image|image.save(&path).map_err(|e|e.to_string()));
                        self.record(app,"sphere_viewport_screenshot",serde_json::json!({"path":path,"saved":result.is_ok(),"created":self.created.len()}));
                        self.proof_saved=result.is_ok();
                    }
                }
            }
            if !self.proof_requested && std::env::var_os("OM9_VIEWPORT_SCREENSHOT_PATH").is_some()
                && self.created.len()>=self.pairs/2 && self.created.len()<self.pairs*2
                && app.gpu_viewports.get(&ViewId::Perspective).and_then(|p|p.published_frame_stats()).is_some_and(|s|s.objects>0 && s.visible>0) {
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
                self.proof_requested=true;
            }
        }
        let now = Instant::now();
        let dt = self.previous.replace(now).map(|p|now.saturating_duration_since(p));
        self.frame += 1;
        if self.frame == 1 {
            self.started = now;
            self.deadline = now + Duration::from_secs(5);
            if !app.controller.document().entities.is_empty() || app.interaction_log.is_none() {
                self.record(app,"sphere_audit_failure",serde_json::json!({"reason":"requires empty document and OM9_INTERACTION_LOG"}));
                self.done=true;ctx.send_viewport_cmd(egui::ViewportCommand::Close);return;
            }
            app.controller.initialize_viewport_detail(self.detail);
            if self.coupled {
                let result = app.controller.coordinator().representation_store().lock()
                    .unwrap_or_else(|e|e.into_inner()).enable_coupled_diagnostic();
                if let Err(error) = result {
                    self.record(app,"sphere_audit_failure",serde_json::json!({"reason":error.to_string()}));
                    self.done=true;ctx.send_viewport_cmd(egui::ViewportCommand::Close);return;
                }
            }
            self.owns_throwaway = true;
            app.display_preferences.viewport_detail = self.detail;
            if self.viewport_only {
                let columns=(self.pairs as f64).sqrt().ceil() as usize;
                let rows=self.pairs.div_ceil(columns);
                let radius=(((columns-1) as f64*10.+7.5).powi(2)+((rows-1) as f64*8.+5.).powi(2)+25.).sqrt();
                for (id,view) in app.controller.workspace.views.iter_mut() {
                    let aspect=app.view_rects.get(id).map_or(1.,|r|f64::from(r.width()/r.height().max(1.)));
                    if let Err(error)=view.camera.fit_bounds([2.5,0.,0.],radius,aspect,1.1) {
                        self.record(app,"sphere_audit_failure",serde_json::json!({"reason":error.to_string()}));
                        self.done=true;ctx.send_viewport_cmd(egui::ViewportCommand::Close);return;
                    }
                }
            }
            let wire = std::env::var("OM9_SPHERE_BOOLEAN_AUDIT_DISPLAY").is_ok_and(|s| s=="wireframe");
            for view in app.controller.workspace.views.values_mut() {
                view.mode = if wire { om9_scene::workspace::DisplayMode::Wireframe } else { om9_scene::workspace::DisplayMode::Shaded };
            }
            self.record(app,"sphere_audit_start",serde_json::json!({"wireframe":wire,"origin":format!("{:?}",self.origin),
                "pointer_navigation":self.pointer_navigation,"light_motion_baseline":app.light_motion_baseline,"pairs":self.pairs,
                "architecture":if self.coupled {"coupled"} else {"separated"},
                "motion_pose_epoch":"gesture_start","viewport_only":self.viewport_only,"temporary_preview_expected":self.pairs>1 && !self.viewport_only,
                "commit_baseline":std::env::var_os("OM9_BOOLEAN_COMMIT_BASELINE").is_some()}));
            self.resources(app,"startup",false);
        }
        self.record(app,"sphere_audit_frame",serde_json::json!({"interval_ms":dt.map(|d|d.as_secs_f64()*1000.)}));
        if self.started.elapsed() > self.timeout {
            self.resources(app,"timeout",false);
            self.record(app,"sphere_audit_failure",serde_json::json!({"reason":"timeout"}));
            self.done=true;ctx.send_viewport_cmd(egui::ViewportCommand::Close);return;
        }
        // Same angular speed in both profiles; advancement is independent of frame count.
        if !self.pointer_navigation && self.stage < 5 && let Some(dt) = dt {
            if let Some(view) = app.controller.workspace.views.get_mut(&ViewId::Perspective) {
                let _ = view.camera.orbit(dt.as_secs_f64()*0.15, 0.);
            }
        }
        if self.pointer_navigation {
            raw.focused = true;
            if let Some(rect) = app.view_rects.get(&ViewId::Perspective) {
                let center = rect.center();
                let moving = (now < self.deadline && matches!(self.stage, 0 | 1 | 2 | 4 | 6)
                    || (self.pairs>1 && self.stage==3))
                    && !(self.pairs>1 && self.stage==1);
                if moving {
                    // Native focus/click events can cancel the product gesture while
                    // the audit still owns its synthetic button. Re-arm through the
                    // normal input path instead of measuring an idle camera as FPS.
                    if self.dragging && app.navigation.gesture.is_none() {
                        raw.events.push(egui::Event::PointerButton { pos:center,button:egui::PointerButton::Secondary,
                            pressed:false,modifiers:Modifiers::NONE });
                        self.dragging = false;
                    }
                    if !self.dragging {
                        self.motion_started = Some(now);
                        raw.events.push(egui::Event::PointerMoved(center));
                        raw.events.push(egui::Event::PointerButton { pos:center,button:egui::PointerButton::Secondary,
                            pressed:true,modifiers:Modifiers::NONE });
                        self.dragging = true;
                    }
                    let t = self.motion_started.map_or(0., |start|now.duration_since(start).as_secs_f32())*0.7;
                    raw.events.push(egui::Event::PointerMoved(center+Vec2::new(t.sin()*60.,t.cos()*35.)));
                    if self.recovery_probe_pending && self.stage==2 && self.motion_started.is_some_and(|start|now.duration_since(start)>Duration::from_secs(1)) {
                        self.recovery_probe_pending = false;
                        raw.events.push(egui::Event::WindowFocused(false));
                        self.record(app,"sphere_audit_recovery_probe",serde_json::json!({"fault":"focus_loss"}));
                    }
                } else if self.dragging {
                    raw.events.push(egui::Event::PointerButton { pos:center,button:egui::PointerButton::Secondary,
                        pressed:false,modifiers:Modifiers::NONE });
                    self.dragging = false;
                }
            }
        }
        // Exercise the product frame clock rather than flooding an unthrottled
        // remote surface with 1 ms UI repaints between actual CAD frames.
        ctx.request_repaint_after(Duration::from_secs_f64(1. / 30.));
        if now < self.deadline { return; }
        if self.viewport_only && self.stage >= 7 {
            match self.verification_step(app, ctx) {
                Ok(false) => (),
                Ok(true) => { self.done=true; ctx.send_viewport_cmd(egui::ViewportCommand::Close); }
                Err(error) => {
                    self.record(app,"sphere_audit_failure",serde_json::json!({"reason":error.to_string()}));
                    self.done=true; ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
            return;
        }
        if self.pairs>1 && (self.stage<5 || self.stage==6) {
            if let Err(error)=self.many_scene_step(app) {
                self.resources(app,"failure",false);
                self.record(app,"sphere_audit_failure",serde_json::json!({"reason":error.to_string(),"prompt":app.controller.prompt_with_options()}));
                self.done=true;ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            return;
        }
        match self.stage {
            0 | 1 => {
                self.record(app,"sphere_command_start",serde_json::json!({"index":self.stage+1}));
                self.start_command(app,"Sphere","OM9-SOLID-014");
                Self::command(app,if self.stage==0 {"0,0,0"} else {"5,0,0"}); Self::command(app,"5");
                let expected=self.stage as usize+1;
                if app.controller.document().entities.len()!=expected {
                    self.record(app,"sphere_audit_failure",serde_json::json!({"reason":"sphere did not commit"}));self.done=true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);return;
                }
                self.record(app,"sphere_command_end",serde_json::json!({"index":self.stage+1}));
                self.stage+=1;self.deadline=Instant::now()+Duration::from_secs(5);
            }
            2 => {
                match self.metrics(app) { Ok(m)=>self.record(app,"sphere_audit_inputs",m),Err(e)=>self.record(app,"sphere_audit_failure",serde_json::json!({"reason":e.to_string()})) }
                let ids=app.controller.document().entities.keys().copied().collect::<Vec<_>>();
                if let Err(e)=app.controller.select_objects(&ids) { self.record(app,"sphere_audit_failure",serde_json::json!({"reason":e.to_string()}));self.done=true;ctx.send_viewport_cmd(egui::ViewportCommand::Close);return; }
                self.record(app,"boolean_command_start",serde_json::json!({}));
                self.start_command(app,"Boolean2Objects","OM9-SOLID-004");
                Self::command(app,"DeleteInput=Yes");Self::command(app,"Union");
                self.stage=3;
            }
            3 => {
                if app.controller.boolean_union_worker_pending() { return; }
                self.record(app,"boolean_preview_ready",serde_json::json!({}));
                Self::command(app,"");
                if app.controller.document().entities.len()!=1 || app.controller.is_running() {
                    self.record(app,"sphere_audit_failure",serde_json::json!({"reason":"Boolean did not commit"}));
                    self.done=true;ctx.send_viewport_cmd(egui::ViewportCommand::Close);return;
                }
                self.record(app,"boolean_commit_end",serde_json::json!({"objects":app.controller.document().entities.len()}));
                self.stage=4;self.deadline=Instant::now()+Duration::from_secs(5);
            }
            4 => {
                self.stage=5;self.deadline=Instant::now()+Duration::from_secs(1);
            }
            _ => {
                if app.gpu_viewports.is_empty() || !app.gpu_viewports.values().all(|port|port.final_complete()) { return; }
                let views=app.gpu_viewports.iter().map(|(id,port)|(id.title(),port.diagnostics())).collect::<BTreeMap<_,_>>();
                self.record(app,"sphere_audit_settled",serde_json::json!({"full_resolution":true,"views":views}));
                self.resources(app,"performance_end",false);
                if self.viewport_only {
                    self.verification_count = app.controller.document().entities.len();
                    self.verification_history = app.controller.coordinator().undo_len();
                    if self.created.is_empty() {
                        self.created = app.controller.document().entities.keys().copied().collect();
                    }
                    self.verification_guard = Some(Self::verification_stamp(app));
                    self.stage = 7;
                    return;
                }
                match self.metrics(app) { Ok(m)=>self.record(app,"sphere_audit_result",m),Err(e)=>self.record(app,"sphere_audit_failure",serde_json::json!({"reason":e.to_string()})) }
                self.resources(app,"after_verification",true);
                self.record(app,"sphere_audit_end",serde_json::json!({}));
                self.done=true;ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }
}
