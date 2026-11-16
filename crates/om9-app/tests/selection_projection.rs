// SPDX-License-Identifier: LGPL-2.1-or-later
//! Exact selection-frame projection through the real egui path.
use eframe::egui::{self, Color32, Event, Modifiers, Pos2, Rect, Shape, Vec2};
use om9_app::ui::MatrixApp;
use om9_core::{
    geometry::{Geometry, Transform},
    model::{Document, Entity},
};

fn screen_rect() -> Rect {
    Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 720.))
}

fn frames(
    app: &mut MatrixApp,
    context: &egui::Context,
    count: usize,
) -> Vec<egui::epaint::ClippedShape> {
    let mut shapes = Vec::new();
    for _ in 0..count {
        let mut output = context.run_ui(
            egui::RawInput {
                screen_rect: Some(screen_rect()),
                events: vec![Event::ModifiersChanged(Modifiers::NONE)],
                ..Default::default()
            },
            |ui| app.show(ui),
        );
        output.textures_delta.clear();
        shapes = output.shapes;
    }
    shapes
}

fn document_with_three_points() -> Document {
    let mut document = Document::new("overlay");
    for point in [[-6.0, -6.0, 0.0], [6.0, -6.0, 0.0], [0.0, 6.0, 0.0]] {
        let entity = Entity::new("point", Geometry::Point(point), None);
        let id = entity.id;
        document.entities.insert(id, entity);
    }
    document
}

#[test]
fn selection_frame_matches_all_projected_points_after_camera_and_preview_changes() {
    use om9_scene::{camera::Viewport,scene::RenderBuffers};
    let context=egui::Context::default();
    let mut app=MatrixApp::new(document_with_three_points()).unwrap();
    let ids=app.controller.document().entities.keys().copied().collect::<Vec<_>>();
    app.controller.select_objects(&ids).unwrap();
    let undo=app.controller.coordinator().undo_len();
    let before=app.controller.document().clone();
    for step in 0..4 {
        if step==1 {
            for view in app.controller.workspace.views.values_mut() {view.camera.pan([7.,-3.]).unwrap();}
        }
        if step==2 {app.controller.preview_gumball(Transform::translation([2.,3.,4.])).unwrap();}
        if step==3 {app.controller.clear_gumball_preview();app.controller.select_objects(&ids[..2]).unwrap();}
        let shapes=frames(&mut app,&context,3);
        for (id,clip) in &app.view_rects {
            let viewport=Viewport {origin:[clip.min.x as f64,clip.min.y as f64],size:[clip.width() as f64,clip.height() as f64]};
            let camera=&app.controller.workspace.views[id].camera;
            let mut min=Pos2::new(f32::MAX,f32::MAX);let mut max=Pos2::new(f32::MIN,f32::MIN);
            for object in &app.controller.scene().unwrap().objects {
                if !app.controller.selected().contains(&object.root) {continue;}
                let RenderBuffers::Points {points,..}=object.buffers.as_ref() else {panic!("point fixture")};
                for point in points {
                    let point=object.display_transform().unwrap().point(*point).unwrap();
                    if let Some(p)=camera.project(point,viewport).unwrap() {
                        let p=Pos2::new(p.screen[0] as f32,p.screen[1] as f32);min=min.min(p);max=max.max(p);
                    }
                }
            }
            let expected=Rect::from_two_pos(min,max).expand(3.).intersect(clip.shrink(1.));
            assert!(shapes.iter().any(|s|s.clip_rect==*clip && matches!(&s.shape,Shape::Rect(r) if r.stroke.color==Color32::WHITE && r.rect==expected)),"{id:?} step {step} expected {expected:?}");
        }
        assert_eq!(app.controller.document(),&before);
        assert_eq!(app.controller.coordinator().undo_len(),undo);
    }
    app.controller.clear_gumball_preview();
    app.controller.clear_selection();
    let shapes=frames(&mut app,&context,2);
    for clip in app.view_rects.values() {
        assert!(!shapes.iter().any(|s|s.clip_rect==*clip && matches!(&s.shape,Shape::Rect(r) if r.stroke.color==Color32::WHITE && r.rect.width()>3. && r.rect.height()>3.)));
    }
}

