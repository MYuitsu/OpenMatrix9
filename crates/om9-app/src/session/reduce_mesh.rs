// SPDX-License-Identifier: LGPL-2.1-or-later
use super::Controller;
use om9_addon_abi::session::{Geo, Input, OptionToken as Token, Reply, Want};
use om9_core::{
    Error, Result,
    budget::memory::MemoryClass,
    geometry::{Geometry, Mesh},
    identity::SourceStamp,
    mesh_repair::reduction::{self, triangle_count},
    model::{Document, MESH_UV_EXTENSION_SLOT},
    rhino_semantics::{MeshUv, mesh_uv_of},
    transaction::{Cancellation, ExternalMemoryReservation, Mutation},
};
use std::sync::mpsc;

struct Output {
    meshes: Vec<(Mesh, Option<MeshUv>)>,
    triangles: usize,
    _memory: ExternalMemoryReservation,
}
struct Job {
    rx: mpsc::Receiver<Result<Output>>,
    cancel: Cancellation,
    apply: bool,
}
impl Drop for Job {
    fn drop(&mut self) {
        self.cancel.cancel();
    }
}

pub(super) struct Reduction {
    sources: Vec<(SourceStamp, usize)>,
    total: usize,
    percent: f64,
    count: usize,
    by_count: bool,
    planar: bool,
    accuracy: u8,
    job: Option<Job>,
    ready: Option<Output>,
}

impl Reduction {
    fn capture(c: &Controller) -> Result<Option<Self>> {
        let mut sources = Vec::new();
        for id in c.selected_ordered() {
            let entity = &c.document().entities[&id];
            if let Geometry::Mesh(mesh) = &entity.geometry {
                if entity.attributes.locked {
                    return Err(Error::Invalid("selected mesh is locked"));
                }
                let count = triangle_count(mesh);
                if count == 0 {
                    return Err(Error::Invalid("empty mesh"));
                }
                sources.push((c.document().stamp(id)?, count));
            }
        }
        if sources.is_empty() {
            return Ok(None);
        }
        let total = sources.iter().map(|x| x.1).sum::<usize>();
        let count = ((total as f64 * 0.5).round() as usize).max(sources.len());
        Ok(Some(Self {
            sources,
            total,
            percent: 50.,
            count,
            by_count: false,
            planar: false,
            accuracy: 8,
            job: None,
            ready: None,
        }))
    }
    pub(super) fn sources_current(&self, doc: &Document) -> bool {
        self.validate(doc).is_ok()
    }
    fn validate(&self, doc: &Document) -> Result<()> {
        for (s, _) in &self.sources {
            if doc.stamp(s.object).ok().as_ref() != Some(s) {
                return Err(Error::Stale("ReduceMesh source changed"));
            }
        }
        Ok(())
    }
    fn edit(&mut self, name: &str, value: Option<&str>) -> Result<()> {
        let number = || {
            value
                .and_then(|s| s.parse::<f64>().ok())
                .filter(|n| n.is_finite())
                .ok_or(Error::Invalid("ReduceMesh expects a finite number"))
        };
        match name.to_ascii_lowercase().as_str() {
            "method" => {
                self.by_count = match value {
                    Some(s) if s.eq_ignore_ascii_case("Count") => true,
                    Some(s) if s.eq_ignore_ascii_case("Percent") => false,
                    _ => return Err(Error::Invalid("choose Count or Percent")),
                }
            }
            "polygons" => {
                let n = number()?;
                if n.fract() != 0. || n < self.sources.len() as f64 || n > self.total as f64 {
                    return Err(Error::Invalid("polygon count out of range"));
                }
                self.count = n as usize;
                self.percent = 100. * (1. - n / self.total as f64);
                self.by_count = true;
            }
            "percent" => {
                let n = number()?;
                if !(0.0..=100.).contains(&n) {
                    return Err(Error::Invalid("reduction percent must be 0 to 100"));
                }
                self.percent = n;
                self.count = ((self.total as f64 * (1. - n / 100.)).round() as usize)
                    .max(self.sources.len());
                self.by_count = false;
            }
            "planaronly" => {
                self.planar = match value.map(str::to_ascii_lowercase).as_deref() {
                    Some("yes" | "on" | "true") => true,
                    Some("no" | "off" | "false") => false,
                    None => !self.planar,
                    _ => return Err(Error::Invalid("PlanarOnly expects Yes or No")),
                }
            }
            "accuracy" => {
                let n = number()?;
                if !(1.0..=10.).contains(&n) || n.fract() != 0. {
                    return Err(Error::Invalid("accuracy must be 1 to 10"));
                }
                self.accuracy = n as u8;
            }
            _ => return Err(Error::Invalid("ReduceMesh option")),
        }
        self.ready = None;
        Ok(())
    }
    fn reply(&self) -> Reply {
        let mut reply = Reply::ask(
            "Reduce Mesh options. Preview or Enter to accept",
            serde_json::Value::Null,
        )
        .want(Want::Nothing)
        .with_options(vec![
            Token::choice(
                "Method",
                if self.by_count { "Count" } else { "Percent" },
                &["Count", "Percent"],
            ),
            Token::number("Polygons", Some(self.count as f64)),
            Token::number("Percent", Some(self.percent)),
            Token::toggle("PlanarOnly", self.planar),
            Token::number("Accuracy", Some(f64::from(self.accuracy))),
            Token::action("Preview"),
        ]);
        reply.notes = vec![
            ("@when".into(), "Polygons|Method=Count".into()),
            ("@when".into(), "Percent|Method=Percent".into()),
            ("Accuracy".into(), "1 fast – 10 accurate".into()),
            ("".into(), format!("Starting polygon count: {}", self.total)),
        ];
        if self.job.is_some() {
            reply.notes.push(("".into(), "Reducing mesh…".into()));
        }
        if let Some(out) = &self.ready {
            reply
                .notes
                .push(("".into(), format!("Preview: {} triangles", out.triangles)));
        }
        reply
    }
    fn start(&mut self, c: &Controller, apply: bool) -> Result<()> {
        let doc = c.coordinator.capture_tracked_document(c.document)?;
        let limits = *c.coordinator.limits();
        let mut bytes = 0u64;
        for (s, _) in &self.sources {
            let e = &doc.entities[&s.object];
            let Geometry::Mesh(mesh) = &e.geometry else {
                return Err(Error::Stale("ReduceMesh source"));
            };
            bytes = bytes
                .saturating_add(mesh.validate(&limits)?.bytes)
                .saturating_add(
                    e.extensions
                        .values()
                        .map(|v| v.payload.len() as u64)
                        .sum::<u64>(),
                );
        }
        // Admission includes the candidate topology, channel copies and viewport lines.
        let memory = c.coordinator.reserve_external_memory_with_held(
            MemoryClass::CpuCache,
            bytes.saturating_mul(20).saturating_add(1024 * 1024),
            om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes)),
            c.scene_cache_usage()?,
        )?;
        let sources = self.sources.clone();
        let (target, total) = (self.count, self.total);
        let accuracy = self.accuracy;
        let angle = if self.planar { 0.0 } else { 180.0 };
        let cancel = Cancellation::default();
        let flag = cancel.clone();
        let (tx, rx) = mpsc::sync_channel(1);
        std::thread::Builder::new()
            .name("om9-reduce-mesh".into())
            .spawn(move || {
                let compute = || -> Result<Output> {
                    let mut meshes = Vec::new();
                    let mut triangles = 0;
                    // Apportion removable triangles so the targets sum exactly to the
                    // requested batch count, with at least one triangle per object.
                    let budget = target - sources.len();
                    let removable = total - sources.len();
                    let mut cumulative = 0u128;
                    let mut allocated = 0usize;
                    for (s, start) in &sources {
                        let e = &doc.entities[&s.object];
                        let Geometry::Mesh(mesh) = &e.geometry else {
                            return Err(Error::Stale("ReduceMesh source"));
                        };
                        cumulative += (*start - 1) as u128 * budget as u128;
                        let end = if removable == 0 {
                            0
                        } else {
                            ((cumulative + removable as u128 / 2) / removable as u128) as usize
                        };
                        let count = 1 + end - allocated;
                        allocated = end;
                        let uv = mesh_uv_of(e, &limits)?;
                        let out = reduction::reduce(
                            mesh,
                            uv.as_ref(),
                            reduction::Options {
                                triangles: count,
                                feature_angle: angle,
                                accuracy,
                            },
                            &limits,
                            &flag,
                        )?;
                        triangles += triangle_count(&out.0);
                        meshes.push(out);
                    }
                    Ok(Output {
                        meshes,
                        triangles,
                        _memory: memory,
                    })
                };
                let _ = tx.send(compute());
            })
            .map_err(|e| Error::Io(e.to_string()))?;
        self.job = Some(Job { rx, cancel, apply });
        Ok(())
    }
    fn apply(&mut self, c: &mut Controller) -> Result<Reply> {
        self.validate(c.document())?;
        let out = self.ready.take().ok_or(Error::Missing("mesh reduction"))?;
        let mut mutations = Vec::new();
        for ((source, _), (mesh, uv)) in self.sources.iter().zip(out.meshes) {
            c.append_reduced_mesh(*source, mesh, uv, &mut mutations)?;
        }
        if !mutations.is_empty() {
            c.commit_utility_mutations("ReduceMesh", mutations)?;
        }
        Ok(Reply {
            done: true,
            messages: vec![format!(
                "Reduce Mesh: {} → {} triangles",
                self.total, out.triangles
            )],
            ..Default::default()
        })
    }
}

impl Controller {
    fn append_reduced_mesh(
        &self,
        source: SourceStamp,
        mesh: Mesh,
        uv: Option<MeshUv>,
        mutations: &mut Vec<Mutation>,
    ) -> Result<()> {
        let entity = &self.document().entities[&source.object];
        if matches!(&entity.geometry,Geometry::Mesh(old) if old==&mesh) {
            return Ok(());
        }
        mutations.push(Mutation::Geometry {
            source,
            geometry: Geometry::Mesh(mesh),
        });
        if let Some(uv) = uv
            && Some(&uv) != mesh_uv_of(entity, self.coordinator.limits())?.as_ref()
        {
            let mut extensions = entity.extensions.clone();
            extensions.insert(MESH_UV_EXTENSION_SLOT.into(), uv.to_extension()?);
            mutations.push(Mutation::Extensions { source, extensions });
        }
        Ok(())
    }

    pub(super) fn reduce_mesh_step(&mut self, input: &Input) -> Result<Reply> {
        if self.reduce_mesh.is_none() && matches!(input, Input::Start | Input::Enter) {
            self.reduce_mesh = Reduction::capture(self)?;
        }
        let Some(mut state) = self.reduce_mesh.take() else {
            return Ok(
                Reply::ask("Select meshes to reduce, Enter", serde_json::Value::Null)
                    .want(Want::Objects),
            );
        };
        let first = self
            .addon_session
            .as_ref()
            .is_some_and(|s| s.reply.want == Want::Objects);
        if let Err(error) = state.validate(self.document()) {
            self.cancel_addon_session();
            return Err(error);
        }
        let result = (|| {
            let mut preview = false;
            match input {
                Input::Option { name, .. } if name.eq_ignore_ascii_case("Preview") => {
                    preview = true
                }
                Input::Option { name, value } => {
                    if state.job.is_some() {
                        return Err(Error::Busy);
                    }
                    state.edit(name, value.as_deref())?;
                }
                Input::Enter if !first => {
                    if state.ready.is_some() {
                        return state.apply(self);
                    }
                    if let Some(job) = &mut state.job {
                        job.apply = true;
                    } else {
                        state.start(self, true)?;
                    }
                }
                _ => {}
            }
            if preview && state.ready.is_none() && state.job.is_none() {
                state.start(self, false)?;
            }
            let mut reply = state.reply();
            if let Some(out) = &state.ready {
                let factor = self.document().page_unit.factor();
                for ((source, _), (mesh, _)) in state.sources.iter().zip(&out.meshes) {
                    let e = &self.document().entities[&source.object];
                    let vertices = mesh
                        .vertices
                        .iter()
                        .map(|&p| e.placement.point(p).map(|v| v.map(|x| x * factor)))
                        .collect::<Result<_>>()?;
                    let triangles = mesh
                        .faces
                        .iter()
                        .flat_map(|f| (1..f.len() - 1).map(|i| [f[0], f[i], f[i + 1]]))
                        .collect();
                    reply.preview.push(Geo::Mesh {
                        name: e.label.clone(),
                        vertices,
                        triangles,
                        meta: None,
                        layer: None,
                    });
                }
            }
            Ok(reply)
        })();
        if !result.as_ref().is_ok_and(|reply| reply.done) {
            self.reduce_mesh = Some(state);
        }
        result
    }
    pub fn reduce_mesh_worker_pending(&self) -> bool {
        self.reduce_mesh.as_ref().is_some_and(|s| s.job.is_some())
    }
    pub fn poll_reduce_mesh(&mut self) -> Option<Result<()>> {
        let state = self.reduce_mesh.as_mut()?;
        let job = state.job.as_ref()?;
        let apply = job.apply;
        let output = match job.rx.try_recv() {
            Ok(r) => r,
            Err(mpsc::TryRecvError::Empty) => return None,
            Err(mpsc::TryRecvError::Disconnected) => {
                Err(Error::Io("ReduceMesh worker disconnected".into()))
            }
        };
        state.job = None;
        match output {
            Ok(out) => state.ready = Some(out),
            Err(e) => {
                self.cancel_addon_session();
                return Some(Err(e));
            }
        }
        Some(self.session_step(if apply {
            Input::Enter
        } else {
            Input::Selection
        }))
    }
}
