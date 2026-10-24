// SPDX-License-Identifier: LGPL-2.1-or-later
//! Native widget replay for nested shared subobject identities. No desktop input injection.
use super::*;
use om9_core::{
    geometry::{Geometry, Mesh, Transform},
    identity::ObjectId,
    model::{Definition, Entity},
};

fn fixture() -> Document {
    let mut d = Document::new("M02 nested occurrence subobjects");
    let leaf = Entity::new(
        "shared mesh",
        Geometry::Mesh(Mesh {
            vertices: vec![
                [-1., -1., -1.],
                [1., -1., -1.],
                [1., 1., -1.],
                [-1., 1., -1.],
                [-1., -1., 1.],
                [1., -1., 1.],
                [1., 1., 1.],
                [-1., 1., 1.],
            ],
            faces: vec![
                vec![0, 3, 2, 1],
                vec![4, 5, 6, 7],
                vec![0, 1, 5, 4],
                vec![1, 2, 6, 5],
                vec![2, 3, 7, 6],
                vec![3, 0, 4, 7],
            ],
            normals: vec![],
            colors: vec![],

            corner_uvs: Vec::new(),
        }),
        None,
    );
    #[cfg(feature = "gpu-sphere")]
    let leaf = if std::env::var("OM9_UI_AUDIT_BREP").is_ok_and(|s| s == "1") {
        let mut leaf = leaf;
        leaf.geometry = Geometry::Brep(
            om9_kernel::brep::box_brep([2.; 3], 0.01, &Limits::default())
                .expect("native BRep audit fixture"),
        );
        leaf.placement = Transform::translation([-1.; 3]);
        leaf
    } else {
        leaf
    };
    let inner = Definition::new("inner", vec![leaf]);
    let inner_id = inner.id;
    d.definitions.insert(inner_id, inner);
    let mut left = Entity::new("left branch", Geometry::Instance(inner_id), None);
    left.placement = Transform::translation([0., -3., 0.]);
    let mut right = Entity::new("right branch", Geometry::Instance(inner_id), None);
    right.placement = Transform::translation([0., 3., 0.]);
    let outer = Definition::new("outer", vec![left, right]);
    let outer_id = outer.id;
    d.definitions.insert(outer_id, outer);
    let mut a = Entity::new("occurrence A", Geometry::Instance(outer_id), None);
    a.placement = Transform::translation([-8., 0., 0.]);
    let mut b = Entity::new("occurrence B", Geometry::Instance(outer_id), None);
    b.placement = Transform([
        [-1., 0., 0., 8.],
        [0., 1., 0., 0.],
        [0., 0., 1., 0.],
        [0., 0., 0., 1.],
    ]);
    d.entities.insert(a.id, a);
    d.entities.insert(b.id, b);
    d
}
impl MatrixApp {
    fn occurrence_id(&self, label: &str) -> ObjectId {
        self.controller
            .document()
            .entities
            .values()
            .chain(
                self.controller
                    .document()
                    .definitions
                    .values()
                    .flat_map(|d| d.members.iter()),
            )
            .find(|e| e.label == label)
            .unwrap()
            .id
    }
    fn occurrence_pointer(&self, second: bool) -> egui::Pos2 {
        let root = self.occurrence_id("occurrence A");
        let branch = self.occurrence_id(if second {
            "right branch"
        } else {
            "left branch"
        });
        let leaf = self.occurrence_id("shared mesh");
        let world = self
            .controller
            .document()
            .member_transform(root, &[branch, leaf])
            .unwrap()
            .point(
                if std::env::var("OM9_UI_AUDIT_BREP").is_ok_and(|s| s == "1") {
                    [1.3, 1.1, 2.]
                } else {
                    [0.3, 0.1, 1.]
                },
            )
            .unwrap();
        self.occurrence_screen(ViewId::LookingDown, world)
    }
    fn occurrence_screen(&self, view: ViewId, world: [f64; 3]) -> egui::Pos2 {
        let rect = self.view_rects[&view];
        let vp = Viewport {
            origin: [rect.min.x as f64, rect.min.y as f64],
            size: [rect.width() as f64, rect.height() as f64],
        };
        let p = self.controller.workspace.views[&view]
            .camera
            .project(world, vp)
            .unwrap()
            .unwrap()
            .screen;
        egui::pos2(p[0] as f32, p[1] as f32)
    }
    fn occurrence_check(&mut self, name: &str, pass: bool, actual: serde_json::Value) {
        let audit = self.audit.as_mut().unwrap();
        audit
            .checks
            .push(format!("{} {name}", if pass { "PASS" } else { "FAILED" }));
        audit
            .selection_report
            .push(serde_json::json!({"name":name,"pass":pass,"actual":actual}));
    }
    pub(super) fn occurrence_audit_input(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) {
        if self.audit.as_ref().unwrap().awaiting_screenshot {
            ctx.request_repaint();
            return;
        }
        raw.focused = true;
        raw.events
            .retain(|e| !matches!(e, egui::Event::WindowFocused(false)));
        raw.events.push(egui::Event::WindowFocused(true));
        let step = self.audit.as_ref().unwrap().step;
        self.audit.as_mut().unwrap().step += 1;
        let modifiers = if (3..=20).contains(&step) {
            Modifiers::CTRL | Modifiers::SHIFT
        } else {
            Modifiers::NONE
        };
        raw.events.push(egui::Event::ModifiersChanged(modifiers));
        raw.events.push(egui::Event::Key {
            key: Key::Escape,
            physical_key: None,
            pressed: false,
            repeat: false,
            modifiers: Modifiers::NONE,
        });
        match step {
            0=>self.controller=Controller::new(fixture(),Limits::default()).unwrap(),
            3|4|5|9|10|11|15|16|17=> {
                let pos=self.occurrence_pointer(matches!(step,9..=11));
                raw.events.push(egui::Event::PointerMoved(pos));
                if matches!(step,4|5|10|11|16|17) {
                    raw.events.push(egui::Event::PointerButton {pos,button:egui::PointerButton::Primary,pressed:matches!(step,4|10|16),modifiers});
                }
            }
            8=> {
                let root=self.occurrence_id("occurrence A");let path=vec![self.occurrence_id("left branch"),self.occurrence_id("shared mesh")];
                let brep=std::env::var("OM9_UI_AUDIT_BREP").is_ok_and(|s|s=="1");
                let actual=self.sub_object();let pass=actual.as_ref().is_some_and(|r|r.root==root&&r.path==path&&if brep {matches!(r.subset,om9_scene::scene::Subelement::BrepFace(_))}else{r.subset==om9_scene::scene::Subelement::MeshFace(1)});
                let local=if brep {[0.,1.,2.]} else {[-1.,0.,1.]};
                let world=self.controller.document().member_transform(root,&path).unwrap().point(local).unwrap();
                let mut probes:Vec<_>=[ViewId::LookingDown,ViewId::Perspective,ViewId::ThroughFinger,ViewId::SideView].into_iter().map(|view| {
                    let p=self.occurrence_screen(view,world);serde_json::json!({"view":view.title(),"screen":[p.x,p.y],"expected_green":true})
                }).collect();
                let other=[self.occurrence_id("right branch"),self.occurrence_id("shared mesh")];
                let world=self.controller.document().member_transform(root,&other).unwrap().point(local).unwrap();let p=self.occurrence_screen(ViewId::LookingDown,world);
                probes.push(serde_json::json!({"view":"unselected sibling","screen":[p.x,p.y],"expected_green":false}));
                self.occurrence_check("Ctrl+Shift stores the exact nested member path",pass,serde_json::json!({"root":actual.as_ref().map(|r|r.root),"path":actual.as_ref().map(|r|&r.path),"probes":probes}));
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new("m02-occurrence")));
            }
            14=> {
                let refs=self.controller.subselection();let pass=refs.len()==2&&refs[0].root==refs[1].root&&refs[0].path!=refs[1].path&&refs[0].subset==refs[1].subset&&self.controller.coordinator().undo_len()==0;
                self.occurrence_check("shared leaf face selected twice under two paths",pass,serde_json::json!({"paths":refs.iter().map(|r|&r.path).collect::<Vec<_>>(),"undo":self.controller.coordinator().undo_len()}));
            }
            20=> {
                let path=vec![self.occurrence_id("right branch"),self.occurrence_id("shared mesh")];
                let refs=self.controller.subselection();let pass=refs.len()==1&&refs[0].path==path;
                self.occurrence_check("toggling the left path preserves the right path",pass,serde_json::json!({"paths":refs.iter().map(|r|&r.path).collect::<Vec<_>>()}));
            }
            21=>raw.events.push(egui::Event::Key {key:Key::Escape,physical_key:None,pressed:true,repeat:false,modifiers:Modifiers::NONE}),
            24=>self.occurrence_check("Escape clears occurrence selection without document history",self.controller.subselection().is_empty()&&self.controller.selected().is_empty()&&self.controller.coordinator().undo_len()==0,serde_json::json!({"subobjects":self.controller.subselection().len(),"objects":self.controller.selected().len(),"undo":self.controller.coordinator().undo_len()})),
            27=> {
                self.audit.as_mut().unwrap().awaiting_screenshot=true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
            }
            _=>{}
        }
        ctx.request_repaint_after(std::time::Duration::from_millis(40));
    }
}
