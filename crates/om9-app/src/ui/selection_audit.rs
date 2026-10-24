// SPDX-License-Identifier: LGPL-2.1-or-later
//! Opt-in native M02 input replay. Setup is explicit; gestures use the normal widgets.
use super::*;
use om9_core::{
    geometry::{Curve, Geometry, Mesh, Transform},
    model::Entity,
};

impl MatrixApp {
    fn selection_audit_check(&mut self, name: &str, pass: bool, actual: serde_json::Value) {
        let audit = self.audit.as_mut().expect("audit");
        audit
            .checks
            .push(format!("{} {name}", if pass { "PASS" } else { "FAILED" }));
        audit
            .selection_report
            .push(serde_json::json!({"name":name,"pass":pass,"actual":actual}));
    }
    fn audit_subject(&self) -> om9_core::identity::ObjectId {
        self.controller
            .document()
            .entities
            .values()
            .find(|e| e.label == "clip-line")
            .expect("audit fixture")
            .id
    }
    fn audit_projection(&self, p: [f64; 3]) -> [f64; 2] {
        let rect = self.view_rects[&ViewId::LookingDown];
        self.controller.workspace.views[&ViewId::LookingDown]
            .camera
            .project(
                p,
                Viewport {
                    origin: [rect.min.x as f64, rect.min.y as f64],
                    size: [rect.width() as f64, rect.height() as f64],
                },
            )
            .expect("project")
            .expect("in front")
            .screen
    }
    fn audit_span(&self) -> f64 {
        match self.controller.workspace.views[&ViewId::LookingDown]
            .camera
            .projection
        {
            om9_scene::camera::Projection::Orthographic { vertical_span_mm } => vertical_span_mm,
            _ => unreachable!("audit top camera"),
        }
    }
    pub(super) fn selection_audit_input(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) {
        if self.audit.as_ref().expect("audit").awaiting_screenshot {
            ctx.request_repaint();
            return;
        }
        // All replay inputs, including focus, are synthetic and scoped to this owned window.
        // OS foreground ownership is a separate manual gate; never weaken the production focus guard.
        raw.focused = true;
        raw.events
            .retain(|event| !matches!(event, egui::Event::WindowFocused(false)));
        raw.events.push(egui::Event::WindowFocused(true));
        let step = self.audit.as_ref().expect("audit").step;
        let trace = serde_json::json!({"step":step,"focused":ctx.input(|i|i.focused),"pointer":ctx.input(|i|i.pointer.interact_pos().map(|p|[p.x,p.y])),"held":ctx.input(|i|i.pointer.primary_down()),"gesture":self.gumball_session.is_some(),"band":self.band_start.map(|p|[p.x,p.y]),"status":self.status.message(),"selection":self.controller.selected(),"points":self.audit.as_ref().unwrap().selection_points,"active":format!("{:?}",self.controller.workspace.active)});
        let directory = &self.audit.as_ref().unwrap().directory;
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(directory.join("selection-trace.jsonl"))
        {
            let _ = writeln!(f, "{trace}");
        }
        self.audit.as_mut().expect("audit").step += 1;
        let key = |key| egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        };
        let pointer = |raw: &mut egui::RawInput, p: [f64; 2], pressed: Option<bool>| {
            let pos = egui::pos2(p[0] as f32, p[1] as f32);
            raw.events.push(egui::Event::PointerMoved(pos));
            if let Some(pressed) = pressed {
                raw.events.push(egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Modifiers::NONE,
                });
            }
        };
        raw.events
            .push(egui::Event::ModifiersChanged(Modifiers::NONE));
        for key in [Key::Enter, Key::Escape] {
            raw.events.push(egui::Event::Key {
                key,
                physical_key: None,
                pressed: false,
                repeat: false,
                modifiers: Modifiers::NONE,
            });
        }
        let near = |a: f64, b: f64| (a - b).abs() < 2e-4;
        match step {
            0 => {
                let mut d = Document::new("M02 selection native replay");
                let line = Entity::new(
                    "clip-line",
                    Geometry::Curve(Curve::Line {
                        start: [-5., -3., 0.],
                        end: [5., 3., 0.],
                        domain: [0., 1.],
                    }),
                    None,
                );
                d.entities.insert(line.id, line);
                let polygon = Entity::new(
                    "polygon-face",
                    Geometry::Mesh(Mesh {
                        vertices: vec![
                            [-12., 6., 0.],
                            [-7., 6., 0.],
                            [-7., 11., 0.],
                            [-12., 11., 0.],
                        ],
                        faces: vec![vec![0, 1, 2, 3]],
                        normals: vec![],
                        colors: vec![],

                        corner_uvs: Vec::new(),
                    }),
                    None,
                );
                d.entities.insert(polygon.id, polygon);
                self.controller = Controller::new(d, Limits::default()).expect("fixture");
            }
            3 => pointer(raw, self.audit_projection([2., 1.2, 0.]), None),
            4 => pointer(raw, self.audit_projection([2., 1.2, 0.]), Some(true)),
            5 => pointer(raw, self.audit_projection([2., 1.2, 0.]), Some(false)),
            7 => {
                let pass = self.controller.selected().contains(&self.audit_subject());
                self.selection_audit_check(
                    "click selects the curve and exposes a pivot",
                    pass,
                    serde_json::json!(self.controller.gumball_pivot().map(|f| f.origin)),
                );
            }
            8 => {
                let x =
                    self.audit_span() / self.view_rects[&ViewId::LookingDown].height() as f64 * 45.;
                let points = [
                    self.audit_projection([x, 0., 0.]),
                    self.audit_projection([x + 5., 0., 0.]),
                ];
                self.audit.as_mut().unwrap().selection_points = Some(points);
                pointer(raw, points[0], None);
            }
            9 => pointer(
                raw,
                self.audit.as_ref().unwrap().selection_points.unwrap()[0],
                Some(true),
            ),
            10 => pointer(
                raw,
                self.audit.as_ref().unwrap().selection_points.unwrap()[1],
                None,
            ),
            12 => {
                let id = self.audit_subject();
                let c = &self.controller;
                let stored = c.document().entities[&id].placement;
                let shown = c
                    .scene()
                    .unwrap()
                    .objects
                    .iter()
                    .find(|o| o.root == id)
                    .unwrap()
                    .display_transform()
                    .unwrap();
                let pass = stored == Transform::identity()
                    && near(shown.0[0][3], 5.)
                    && c.coordinator().undo_len() == 0
                    && self.view_rects.len() == 4;
                let curve_samples: Vec<_> = [ViewId::LookingDown, ViewId::Perspective]
                    .into_iter()
                    .map(|view| {
                        let rect = self.view_rects[&view];
                        let p = self.controller.workspace.views[&view]
                            .camera
                            .project(
                                [8., 1.8, 0.],
                                Viewport {
                                    origin: [rect.min.x as f64, rect.min.y as f64],
                                    size: [rect.width() as f64, rect.height() as f64],
                                },
                            )
                            .unwrap()
                            .unwrap();
                        serde_json::json!({"view":view.title(),"screen":p.screen})
                    })
                    .collect();
                self.selection_audit_check("first drag sample previews +5 mm without document or history writes",pass,serde_json::json!({"stored":stored,"shown":shown,"views":self.view_rects.len(),"curve_samples":curve_samples}));
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                    "m02-move",
                )));
            }
            14 => pointer(
                raw,
                self.audit.as_ref().unwrap().selection_points.unwrap()[1],
                Some(false),
            ),
            16 => {
                let p = self.controller.document().entities[&self.audit_subject()].placement;
                self.selection_audit_check(
                    "release commits +5 mm as exactly one Undo entry",
                    near(p.0[0][3], 5.) && self.controller.coordinator().undo_len() == 1,
                    serde_json::json!(p),
                );
                raw.events.push(egui::Event::Text("Undo".into()));
            }
            17 | 20 | 34 | 48 => raw.events.push(key(Key::Enter)),
            19 => {
                let p = self.controller.document().entities[&self.audit_subject()].placement;
                self.selection_audit_check(
                    "CMD Undo restores original geometry",
                    p == Transform::identity(),
                    serde_json::json!(p),
                );
                raw.events.push(egui::Event::Text("Redo".into()));
            }
            22 => {
                let p = self.controller.document().entities[&self.audit_subject()].placement;
                self.selection_audit_check(
                    "CMD Redo restores +5 mm",
                    near(p.0[0][3], 5.),
                    serde_json::json!(p),
                );
            }
            23 => {
                let p = self
                    .gumball_scale_boxes(ViewId::LookingDown)
                    .into_iter()
                    .find(|(h, _)| {
                        *h == om9_core::gumball::Handle::Scale(om9_core::gumball::Axis::X)
                    })
                    .unwrap()
                    .1;
                let centre = self.audit_projection([5., 0., 0.]);
                let a = [p.x as f64, p.y as f64];
                let b = [
                    centre[0] + 2. * (a[0] - centre[0]),
                    centre[1] + 2. * (a[1] - centre[1]),
                ];
                self.audit.as_mut().unwrap().selection_points = Some([a, b]);
                pointer(raw, a, None);
            }
            24 | 38 => pointer(
                raw,
                self.audit.as_ref().unwrap().selection_points.unwrap()[0],
                Some(true),
            ),
            25 | 39 => pointer(
                raw,
                self.audit.as_ref().unwrap().selection_points.unwrap()[1],
                None,
            ),
            26 => ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                "m02-scale",
            ))),
            27 | 41 => pointer(
                raw,
                self.audit.as_ref().unwrap().selection_points.unwrap()[1],
                Some(false),
            ),
            30 => {
                let p = self.controller.document().entities[&self.audit_subject()].placement;
                self.selection_audit_check(
                    "scale square doubles X about current pivot",
                    near(p.0[0][0], 2.)
                        && near(p.0[1][1], 1.)
                        && near(p.0[0][3], 5.)
                        && self.controller.coordinator().undo_len() == 2,
                    serde_json::json!(p),
                );
            }
            33 | 47 => raw.events.push(egui::Event::Text("Undo".into())),
            37 => {
                let r = self.audit_span() / self.view_rects[&ViewId::LookingDown].height() as f64
                    * 90.
                    * 0.6;
                let a = [
                    5. - r / std::f64::consts::SQRT_2,
                    -r / std::f64::consts::SQRT_2,
                    0.,
                ];
                let b = [
                    5. + r / std::f64::consts::SQRT_2,
                    -r / std::f64::consts::SQRT_2,
                    0.,
                ];
                let pair = [self.audit_projection(a), self.audit_projection(b)];
                self.audit.as_mut().unwrap().selection_points = Some(pair);
                pointer(raw, pair[0], None);
            }
            44 => {
                let p = self.controller.document().entities[&self.audit_subject()].placement;
                self.selection_audit_check(
                    "rotation arc makes 90 degree Z turn",
                    near(p.0[0][0], 0.) && near(p.0[1][0], 1.) && near(p.0[0][1], -1.),
                    serde_json::json!(p),
                );
            }
            40 => ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                "m02-rotate",
            ))),
            51 => {
                let centre = self.audit_projection([5., 0., 0.]);
                let end = self.audit_projection([7., 2., 0.]);
                self.audit.as_mut().unwrap().selection_points = Some([centre, end]);
                pointer(raw, centre, None);
            }
            52 => pointer(
                raw,
                self.audit.as_ref().unwrap().selection_points.unwrap()[0],
                Some(true),
            ),
            53 => pointer(
                raw,
                self.audit.as_ref().unwrap().selection_points.unwrap()[1],
                None,
            ),
            55 => raw.events.push(key(Key::Escape)),
            56 => pointer(
                raw,
                self.audit.as_ref().unwrap().selection_points.unwrap()[1],
                Some(false),
            ),
            58 => {
                let p = self.controller.document().entities[&self.audit_subject()].placement;
                let shown = self
                    .controller
                    .scene()
                    .unwrap()
                    .objects
                    .iter()
                    .find(|o| o.root == self.audit_subject())
                    .unwrap()
                    .display_transform()
                    .unwrap();
                self.selection_audit_check(
                    "Escape discards preview and retains old Redo",
                    near(p.0[0][3], 5.)
                        && shown == p
                        && self.controller.coordinator().undo_len() == 1,
                    serde_json::json!({"stored":p,"shown":shown}),
                );
            }
            60 => raw.events.push(egui::Event::Text("Redo".into())),
            61 => raw.events.push(key(Key::Enter)),
            64 => {
                let p = self.controller.document().entities[&self.audit_subject()].placement;
                self.selection_audit_check(
                    "old Redo remains usable after cancelled drag",
                    near(p.0[1][0], 1.),
                    serde_json::json!(p),
                );
            }
            68 => pointer(raw, self.audit_projection([-9.5, 8.5, 0.]), None),
            69 => pointer(raw, self.audit_projection([-9.5, 8.5, 0.]), Some(true)),
            70 => pointer(raw, self.audit_projection([-9.5, 8.5, 0.]), Some(false)),
            72 => {
                let root = self
                    .controller
                    .document()
                    .entities
                    .values()
                    .find(|e| e.label == "polygon-face")
                    .unwrap()
                    .id;
                self.selection_audit_check(
                    "GPU and painter paths select the polygon mesh",
                    self.controller.selected().contains(&root),
                    serde_json::json!(self.controller.selected()),
                );
                let x =
                    self.audit_span() / self.view_rects[&ViewId::LookingDown].height() as f64 * 45.;
                let pair = [
                    self.audit_projection([-9.5 + x, 8.5, 0.]),
                    self.audit_projection([-7.5 + x, 8.5, 0.]),
                ];
                self.audit.as_mut().unwrap().selection_points = Some(pair);
                pointer(raw, pair[0], None);
            }
            73 => pointer(
                raw,
                self.audit.as_ref().unwrap().selection_points.unwrap()[0],
                Some(true),
            ),
            74 => pointer(
                raw,
                self.audit.as_ref().unwrap().selection_points.unwrap()[1],
                None,
            ),
            76 => {
                let root = self
                    .controller
                    .document()
                    .entities
                    .values()
                    .find(|e| e.label == "polygon-face")
                    .unwrap()
                    .id;
                let shown = self
                    .controller
                    .scene()
                    .unwrap()
                    .objects
                    .iter()
                    .find(|o| o.root == root)
                    .unwrap()
                    .display_transform()
                    .unwrap();
                self.selection_audit_check("mesh preview uses selected mesh indices after selection changes",near(shown.0[0][3],2.)&&self.controller.document().entities[&root].placement==Transform::identity(),serde_json::json!({"shown":shown,"mesh_probe":self.audit_projection([-6.,9.5,0.])}));
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                    "m02-mesh",
                )));
            }
            78 => pointer(
                raw,
                self.audit.as_ref().unwrap().selection_points.unwrap()[1],
                Some(false),
            ),
            81 => {
                let root = self
                    .controller
                    .document()
                    .entities
                    .values()
                    .find(|e| e.label == "polygon-face")
                    .unwrap()
                    .id;
                let p = self.controller.document().entities[&root].placement;
                self.selection_audit_check(
                    "mesh release commits one additional transaction",
                    near(p.0[0][3], 2.) && self.controller.coordinator().undo_len() == 3,
                    serde_json::json!(p),
                );
            }
            84 => {
                self.audit.as_mut().unwrap().awaiting_screenshot = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
            }
            _ => {}
        }
        ctx.request_repaint_after(std::time::Duration::from_millis(40));
    }
}
