// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_app::session::{Controller, LINE};
use om9_core::{
    budget::Limits,
    geometry::{Geometry, NurbsSurface, Transform},
    model::{Document, Entity},
    top11::{GripMode, Top11Action},
};

fn controller() -> Controller {
    Controller::new(
        Document::with_template_tolerance("top11"),
        Limits::default(),
    )
    .unwrap()
}

fn draw_line(c: &mut Controller, start: [f64; 3], end: [f64; 3]) -> om9_core::identity::ObjectId {
    let before: std::collections::BTreeSet<_> = c.document().entities.keys().copied().collect();
    c.submit(LINE).unwrap();
    c.submit(&format!("{},{},{}", start[0], start[1], start[2]))
        .unwrap();
    c.submit(&format!("{},{},{}", end[0], end[1], end[2]))
        .unwrap();
    *c.document()
        .entities
        .keys()
        .find(|id| !before.contains(id))
        .expect("new line")
}

fn select(c: &mut Controller, ids: &[om9_core::identity::ObjectId]) {
    c.select_objects(ids).unwrap();
}

#[test]
fn feature_dispatch_reaches_the_same_action_as_the_toolbar() {
    use om9_app::session::{DispatchOutcome, InvocationOrigin};
    let mut c = controller();
    let id = draw_line(&mut c, [0.; 3], [10., 0., 0.]);
    select(&mut c, &[id]);
    let outcome = c
        .dispatch_effect(Top11Action::Move.feature_id(), InvocationOrigin::Menu)
        .unwrap();
    assert!(matches!(outcome, DispatchOutcome::Top11(_)), "{outcome:?}");
    assert!(c.is_running());
    assert_eq!(c.top11_prompt(), Some("pick the base point"));
    c.accept_world_point([0.; 3]).unwrap();
    c.accept_world_point([1., 2., 0.]).unwrap();
    assert_eq!(
        c.document().entities[&id].placement.point([0.; 3]).unwrap(),
        [1., 2., 0.]
    );
}

#[test]
fn finishing_empty_duplicate_or_trim_reports_no_changes() {
    use om9_app::session::DispatchOutcome;
    for toolbar in [false, true] {
        let mut c = controller();
        let a = draw_line(&mut c, [-10., 0., 0.], [10., 0., 0.]);
        let b = draw_line(&mut c, [0., -10., 0.], [0., 10., 0.]);
        for action in [Top11Action::Duplicate, Top11Action::Trim] {
            if action == Top11Action::Duplicate { select(&mut c, &[a]); } else { select(&mut c, &[a, b]); }
            let before = c.document().clone(); let undo = c.coordinator().undo_len();
            if toolbar { c.top11_invoke(action).unwrap(); } else { c.submit(action.label()).unwrap(); }
            let DispatchOutcome::Top11(result) = c.submit_effect("").unwrap() else { panic!("missing completion feedback") };
            assert_eq!(result.message(), format!("{}: no changes", action.label()));
            assert!(!c.is_running()); assert_eq!(c.document(), &before); assert_eq!(c.coordinator().undo_len(), undo);
        }
        select(&mut c, &[a,b]); c.submit("Trim").unwrap();
        c.top11_trim_many_accept_pick(a, [5.,0.,0.]).unwrap();
        let DispatchOutcome::Top11(result) = c.submit_effect("").unwrap() else { panic!("missing completion feedback") };
        assert_eq!(result.message(), "Trim: 1 object(s)");
    }
}

#[test]
fn mixed_selection_shows_grips_for_compatible_objects_without_changing_others() {
    use om9_core::{geometry::Curve,surface_kernel};let limits=Limits::default();
    for action in [Top11Action::ControlPoints,Top11Action::EditPoints] {for toolbar in [false,true] {
        let line=Entity::new("line",Geometry::Curve(Curve::Line{start:[0.,0.,0.],end:[2.,0.,0.],domain:[0.,1.]}),None);
        let point=Entity::new("point",Geometry::Point([4.,0.,0.]),None);
        let surface=Entity::new("surface",Geometry::Surface(surface_kernel::plane([0.,4.,0.],[1.,0.,0.],[0.,1.,0.],2.,2.,&limits).unwrap()),None);
        let ids=[line.id,point.id,surface.id];let mut doc=Document::with_template_tolerance("mixed grips");for e in [line,point,surface]{doc.entities.insert(e.id,e);}
        let mut c=Controller::new(doc,limits).unwrap();c.select_objects(&ids).unwrap();let before=c.document().clone();
        if toolbar{c.top11_invoke(action).unwrap();}else{c.submit(action.command()).unwrap();}
        let handles=c.top11_grip_handles().unwrap();assert!(handles.iter().any(|h|h.object==ids[0]));assert!(handles.iter().any(|h|h.object==ids[2]));assert!(handles.iter().all(|h|h.object!=ids[1]));
        let h=*handles.iter().find(|h|h.object==ids[0]&&h.index==1).unwrap();c.top11_move_grip(h,[3.,0.,0.]).unwrap();assert_eq!(c.document().entities[&ids[1]],before.entities[&ids[1]]);assert_eq!(c.document().entities[&ids[2]],before.entities[&ids[2]]);
        let after=c.document().clone();let path=std::env::temp_dir().join(format!("om9-mixed-grips-{}.om9",uuid::Uuid::new_v4()));om9_archive::Archive::save(&path,&after,&limits,&om9_core::transaction::Cancellation::default()).unwrap();assert_eq!(om9_archive::Archive::open(&path,&limits).unwrap(),after);std::fs::remove_file(path).unwrap();c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }}
}

#[test]
fn join_mixed_curve_and_point_is_disabled_and_keeps_document() {
    use om9_core::geometry::Curve;let make=|name,start,end|Entity::new(name,Geometry::Curve(Curve::Line{start,end,domain:[0.,1.]}),None);
    let a=make("a",[0.,0.,0.],[1.,0.,0.]);let b=make("b",[1.,0.,0.],[2.,0.,0.]);let p=Entity::new("point",Geometry::Point([0.,1.,0.]),None);let (a_id,b_id,p_id)=(a.id,b.id,p.id);let mut doc=Document::with_template_tolerance("mixed join");for e in [a,b,p]{doc.entities.insert(e.id,e);}let mut c=Controller::new(doc,Limits::default()).unwrap();
    c.select_objects(&[a_id,b_id,p_id]).unwrap();let before=c.document().clone();let undo=c.coordinator().undo_len();assert!(!c.top11_availability().iter().find(|state|state.action==Top11Action::Join).unwrap().enabled);
    assert!(c.top11_invoke(Top11Action::Join).is_err());assert!(c.submit("Join").is_err());assert_eq!(c.document(),&before);assert_eq!(c.coordinator().undo_len(),undo);
}

#[test]
fn copy_and_mirror_accept_typed_world_and_relative_points() {
    use om9_core::geometry::Curve;let limits=Limits::default();
    for action in [Top11Action::Duplicate,Top11Action::Mirror] {for toolbar in [false,true] {
        let entity=Entity::new("source",Geometry::Curve(Curve::Line{start:[2.,0.,0.],end:[3.,0.,0.],domain:[0.,1.]}),None);let id=entity.id;let mut doc=Document::with_template_tolerance("typed Top11 points");doc.entities.insert(id,entity);
        let mut c=Controller::new(doc,limits).unwrap();c.select_objects(&[id]).unwrap();let before=c.document().clone();if toolbar{c.top11_invoke(action).unwrap();}else{c.submit(action.command()).unwrap();}
        if action==Top11Action::Mirror {c.submit("Copy=No").unwrap();}
        c.submit("1,2,0").unwrap();let prompt=c.top11_prompt();assert!(c.submit("not-a-point").is_err());assert_eq!(c.top11_prompt(),prompt);assert_eq!(c.document(),&before);
        c.submit("@0,3,0").unwrap();if action==Top11Action::Duplicate{c.submit("").unwrap();}
        if action==Top11Action::Duplicate {let copy=c.document().entities.values().find(|e|e.id!=id).unwrap();assert_eq!(copy.placement.point([0.;3]).unwrap(),[0.,3.,0.]);}
        else {let Geometry::Curve(Curve::Line{start,end,..})=&c.document().entities[&id].geometry else{panic!("line")};assert!((start[0]-0.).abs()<1e-12&&(end[0]+1.).abs()<1e-12,"{start:?} {end:?}");}
        let after=c.document().clone();let path=std::env::temp_dir().join(format!("om9-top11-typed-points-{}.om9",uuid::Uuid::new_v4()));om9_archive::Archive::save(&path,&after,&limits,&om9_core::transaction::Cancellation::default()).unwrap();assert_eq!(om9_archive::Archive::open(&path,&limits).unwrap(),after);std::fs::remove_file(path).unwrap();c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }}
}

#[test]
fn typed_copy_and_mirror_points_respect_rotated_cplane() {
    use om9_core::geometry::Curve;
    for action in [Top11Action::Duplicate,Top11Action::Mirror] {for toolbar in [false,true] {
        let source=Entity::new("source",Geometry::Curve(Curve::Line{start:[10.,22.,0.],end:[10.,23.,0.],domain:[0.,1.]}),None);let id=source.id;let mut doc=Document::with_template_tolerance("rotated typed points");doc.entities.insert(id,source);let mut c=Controller::new(doc,Limits::default()).unwrap();
        c.set_cplane_origin_rotation([10.,20.,0.],std::f64::consts::FRAC_PI_2).unwrap();c.select_objects(&[id]).unwrap();let before=c.document().clone();if toolbar{c.top11_invoke(action).unwrap();}else{c.submit(action.command()).unwrap();}
        if action==Top11Action::Mirror{c.submit("Copy=No").unwrap();}c.submit("1,2,0").unwrap();c.submit("@0,3,0").unwrap();if action==Top11Action::Duplicate{c.submit("").unwrap();}
        if action==Top11Action::Duplicate{let copy=c.document().entities.values().find(|e|e.id!=id).unwrap();let q=copy.placement.point([0.;3]).unwrap();assert!((q[0]+3.).abs()<1e-9&&q[1].abs()<1e-9,"{q:?}");}
        else{let Geometry::Curve(Curve::Line{start,end,..})=&c.document().entities[&id].geometry else{panic!("line")};for (actual,want) in [(*start,[10.,20.,0.]),(*end,[10.,19.,0.])]{assert!((0..3).all(|i|(actual[i]-want[i]).abs()<1e-9),"{actual:?}");}}
        let after=c.document().clone();c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }}
}

#[test]
fn rejected_rotate_point_does_not_change_relative_base() {
    let mut c=controller();let id=draw_line(&mut c,[1.,2.,0.],[2.,2.,0.]);c.select_objects(&[id]).unwrap();let before=c.document().clone();c.submit("Rotate").unwrap();c.submit("1,2,0").unwrap();
    assert!(c.submit("1,2,5").is_err());assert_eq!(c.document(),&before);assert_eq!(c.relative_base().unwrap().xyz(),[1.,2.,0.]);c.submit("@1,0,0").unwrap();c.submit("@-1,1,0").unwrap();
    let position=c.document().entities[&id].placement.point([2.,2.,0.]).unwrap();assert!((position[0]-1.).abs()<1e-9&&(position[1]-3.).abs()<1e-9&&position[2].abs()<1e-9,"{position:?}");
    c.undo().unwrap();assert_eq!(c.document(),&before);
}

#[test]
fn copy_and_move_preview_match_committed_points_in_four_directions() {
    for action in [Top11Action::Duplicate,Top11Action::Move] {for toolbar in [false,true] {for delta in [[2.,3.,0.],[-2.,3.,0.],[2.,-3.,0.],[-2.,-3.,0.]] {
        let source=Entity::new("point",Geometry::Point([1.,2.,0.]),None);let id=source.id;let mut doc=Document::with_template_tolerance("Top11 preview");doc.entities.insert(id,source);let mut c=Controller::new(doc,Limits::default()).unwrap();c.select_objects(&[id]).unwrap();let before=c.document().clone();
        if toolbar{c.top11_invoke(action).unwrap();}else{c.submit(action.command()).unwrap();}c.accept_world_point([3.,7.,0.]).unwrap();let target=[3.+delta[0],7.+delta[1],0.];let expected=[1.+delta[0],2.+delta[1],0.];
        let wire=c.transform_preview(target,"").unwrap();assert_eq!(wire,vec![[expected,expected]],"{action:?} {delta:?}");assert_eq!(c.document(),&before);
        c.accept_world_point(target).unwrap();if action==Top11Action::Duplicate{c.submit("").unwrap();}
        let actual=if action==Top11Action::Duplicate{c.document().entities.values().find(|e|e.id!=id).unwrap()}else{&c.document().entities[&id]};let Geometry::Point(p)=actual.geometry else{panic!("point")};assert_eq!(actual.placement.point(p).unwrap(),expected);let after=c.document().clone();c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }}}
}

#[test]
fn copy_preview_tracks_repeat_options_and_matches_each_committed_copy() {
    for command in ["Copy FromLastPoint=Yes", "Copy UseLastDirection=Yes", "Copy UseLastDistance=Yes", "Copy FromLastPoint=Yes UseLastDistance=Yes UseLastDirection=Yes", "Copy Vertical=Yes"] {
        let source = Entity::new("point", Geometry::Point([1., 2., 0.]), None);
        let id = source.id;
        let mut doc = Document::with_template_tolerance("Copy preview options");
        doc.entities.insert(id, source);
        let mut c = Controller::new(doc, Limits::default()).unwrap();
        c.select_objects(&[id]).unwrap();
        c.submit(command).unwrap();
        c.accept_world_point([0.; 3]).unwrap();
        for target in [[3., 4., 5.], [4., 8., 10.]] {
            let before = c.document().clone();
            let preview = c.transform_preview(target, "").unwrap();
            assert_eq!(preview.len(), 1, "{command}");
            assert_eq!(c.document(), &before);
            c.accept_world_point(target).unwrap();
            let added = c.document().entities.values().find(|entity| !before.entities.contains_key(&entity.id)).unwrap();
            let Geometry::Point(point) = added.geometry else { panic!("point") };
            assert_eq!(preview[0], [added.placement.point(point).unwrap(); 2], "{command}");
        }
    }
}

#[test]
fn move_preview_includes_placed_nested_instance_members() {
    use om9_core::geometry::Transform;
    use om9_core::model::Definition;
    let mut point = Entity::new("point", Geometry::Point([1., 2., 0.]), None);
    point.placement = Transform::translation([2., 0., 0.]);
    let definition = Definition::new("member", vec![point]);
    let mut doc = Document::with_template_tolerance("nested preview");
    let definition_id = definition.id;
    doc.definitions.insert(definition_id, definition);
    let mut root = Entity::new("instance", Geometry::Instance(definition_id), None);
    root.placement = Transform::translation([0., 5., 0.]);
    let root_id = root.id;
    doc.entities.insert(root_id, root);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[root_id]).unwrap();
    c.submit("Move").unwrap();
    c.accept_world_point([0.; 3]).unwrap();
    let preview = c.transform_preview([-4., 3., 0.], "").unwrap();
    assert_eq!(preview, vec![[[-1., 10., 0.]; 2]]);
    c.accept_world_point([-4., 3., 0.]).unwrap();
    let object = &c.scene().unwrap().objects[0];
    let om9_scene::scene::RenderBuffers::Points { points, .. } = object.buffers.as_ref() else { panic!("points") };
    assert_eq!(object.transform.point(points[0]).unwrap(), preview[0][0]);
}

#[test]
fn mirror_and_rotate_preview_match_commit() {
    for action in [Top11Action::Mirror, Top11Action::Rotate] {
        for toolbar in [false, true] {
            for copy in [false, true] {
                for target in [[4., 3., 0.], [-4., 3., 0.], [4., -3., 0.], [-4., -3., 0.]] {
                    let source = Entity::new("point", Geometry::Point([2., 1., 0.]), None);
                    let id = source.id;
                    let mut doc = Document::with_template_tolerance("transform preview");
                    doc.entities.insert(id, source);
                    let mut c = Controller::new(doc, Limits::default()).unwrap();
                    c.select_objects(&[id]).unwrap();
                    if toolbar { c.top11_invoke(action).unwrap(); } else { c.submit(action.command()).unwrap(); }
                    c.submit(if copy { "Copy=Yes" } else { "Copy=No" }).unwrap();
                    c.accept_world_point([0.; 3]).unwrap();
                    if action == Top11Action::Rotate { c.accept_world_point([1., 0., 0.]).unwrap(); }
                    let before = c.document().clone();
                    let preview = c.transform_preview(target, "").unwrap();
                    assert_eq!(preview.len(), 1, "{action:?} {target:?}");
                    assert_eq!(c.document(), &before);
                    c.accept_world_point(target).unwrap();
                    let actual = if copy { c.document().entities.values().find(|e| e.id != id).unwrap() } else { &c.document().entities[&id] };
                    let Geometry::Point(point) = actual.geometry else { panic!("point") };
                    let actual = actual.placement.point(point).unwrap();
                    let length = (target[0] * target[0] + target[1] * target[1]).sqrt();
                    let (x, y) = (target[0] / length, target[1] / length);
                    let expected = if action == Top11Action::Mirror {
                        let along = 2. * x + y;
                        [2. * along * x - 2., 2. * along * y - 1., 0.]
                    } else { [2. * x - y, 2. * y + x, 0.] };
                    for (a, b) in actual.iter().zip(expected) { assert!((a-b).abs() < 1e-9, "{action:?} {target:?}: {actual:?} {expected:?}"); }
                    for (a, b) in preview[0][0].iter().zip(actual) { assert!((a-b).abs() < 1e-9, "{action:?} {target:?}: {preview:?} {actual:?}"); }
                }
            }
        }
    }
}

#[test]
fn mirror_three_point_preview_and_rotate_tilted_plane_match_commit() {
    use om9_scene::camera::CPlane;
    for action in [Top11Action::Mirror, Top11Action::Rotate] {
        let source = Entity::new("point", Geometry::Point([2., 3., 4.]), None);
        let id = source.id;
        let mut doc = Document::with_template_tolerance("3D transform preview");
        doc.entities.insert(id, source);
        let mut c = Controller::new(doc, Limits::default()).unwrap();
        c.set_cplane(CPlane::new([1., 2., 3.], [0., 1., 0.], [0., 0., 1.]).unwrap()).unwrap();
        c.select_objects(&[id]).unwrap();
        c.submit(action.command()).unwrap();
        c.submit("Copy=No").unwrap();
        if action == Top11Action::Mirror { c.submit("3Point").unwrap(); }
        c.accept_world_point([1., 2., 3.]).unwrap();
        c.accept_world_point([1., 3., 3.]).unwrap();
        let target = [1., 2., 4.];
        let before = c.document().clone();
        let preview = c.transform_preview(target, "").unwrap();
        assert_eq!(preview.len(), 1);
        c.accept_world_point(target).unwrap();
        let entity = &c.document().entities[&id];
        let Geometry::Point(point) = entity.geometry else { panic!("point") };
        let actual = entity.placement.point(point).unwrap();
        let expected = if action == Top11Action::Mirror { [0., 3., 4.] } else { [2., 1., 4.] };
        for (a, b) in actual.iter().zip(expected) { assert!((a-b).abs() < 1e-9, "{action:?}: {actual:?}"); }
        for (a, b) in preview[0][0].iter().zip(actual) { assert!((a-b).abs() < 1e-9, "{action:?}: {preview:?} {actual:?}"); }
        let after = c.document().clone();
        c.undo().unwrap(); assert_eq!(c.document(), &before);
        c.redo().unwrap(); assert_eq!(c.document(), &after);
    }
}

#[test]
fn mirror_three_point_plane_does_not_depend_on_edge_length_product() {
    for scale in [1e-5, 1e-7, 1e-9, 1e-13, 1e-100, 1e100] {
        for sx in [-1., 1.] { for sy in [-1., 1.] {
            let source = Entity::new("point", Geometry::Point([2., 3., 4.]), None);
            let id = source.id;
            let mut doc = Document::with_template_tolerance("small mirror plane"); doc.entities.insert(id, source);
            let mut c = Controller::new(doc, Limits::default()).unwrap(); c.select_objects(&[id]).unwrap();
            c.submit("Mirror").unwrap(); c.submit("Copy=No").unwrap(); c.submit("3Point").unwrap();
            c.accept_world_point([0.; 3]).unwrap(); c.accept_world_point([sx * scale, 0., 0.]).unwrap();
            let last = [0., sy * scale, 0.]; let before = c.document().clone();
            let preview = c.transform_preview(last, "").unwrap();
            assert_eq!(preview.len(), 1, "scale={scale} sx={sx} sy={sy}");
            let expected = [2., 3., -4.];
            assert!((0..3).all(|i| (preview[0][0][i]-expected[i]).abs() < 1e-9));
            c.accept_world_point(last).unwrap();
            let entity = &c.document().entities[&id]; let Geometry::Point(point) = entity.geometry else { panic!("point") };
            let actual = entity.placement.point(point).unwrap();
            assert!((0..3).all(|i| (actual[i]-expected[i]).abs() < 1e-9));
            c.undo().unwrap(); assert_eq!(c.document(), &before);
        }}
    }
}

#[test]
fn mirror_two_point_small_axis_and_three_point_collinear_retry() {
    for toolbar in [false, true] {
        for scale in [1e-13, -1e-13, 1e-100, 1e100] {
            let source = Entity::new("point", Geometry::Point([2., 3., 4.]), None); let id = source.id;
            let mut doc = Document::with_template_tolerance("mirror scale"); doc.entities.insert(id, source);
            let mut c = Controller::new(doc, Limits::default()).unwrap(); c.select_objects(&[id]).unwrap();
            if toolbar { c.top11_invoke(Top11Action::Mirror).unwrap(); } else { c.submit("Mirror").unwrap(); }
            c.submit("Copy=No").unwrap(); c.accept_world_point([0.; 3]).unwrap();
            let target = [scale, 0., 0.]; let preview = c.transform_preview(target, "").unwrap();
            assert_eq!(preview[0][0], [2., -3., 4.]); c.accept_world_point(target).unwrap();
            let Geometry::Point(actual) = c.document().entities[&id].geometry else { panic!("point") };
            assert_eq!(actual, [2., -3., 4.]);
        }
    }
    let source = Entity::new("point", Geometry::Point([2., 3., 4.]), None); let id = source.id;
    let mut doc = Document::with_template_tolerance("mirror retry"); doc.entities.insert(id, source);
    let mut c = Controller::new(doc, Limits::default()).unwrap(); c.select_objects(&[id]).unwrap();
    c.submit("Mirror").unwrap(); c.submit("Copy=No").unwrap(); c.submit("3Point").unwrap();
    c.accept_world_point([0.; 3]).unwrap(); c.accept_world_point([1e-9, 0., 0.]).unwrap();
    let before = c.document().clone();
    assert!(c.accept_world_point([2e-9, 0., 0.]).is_err()); assert_eq!(c.document(), &before);
    assert!(c.is_running()); c.accept_world_point([0., 1e-9, 0.]).unwrap();
    let Geometry::Point(actual) = c.document().entities[&id].geometry else { panic!("point") };
    assert_eq!(actual, [2., 3., -4.]);
}

#[test]
fn copy_tiny_nonzero_offset_and_repeat_distance_keep_preview_and_commit() {
    for toolbar in [false, true] { for size in [1e-13, 1e-200, 1e200] { for sign in [-1., 1.] {
        let source = Entity::new("point", Geometry::Point([0.; 3]), None); let id = source.id;
        let mut doc = Document::with_template_tolerance("tiny copy"); doc.entities.insert(id, source);
        let mut c = Controller::new(doc, Limits::default()).unwrap(); c.select_objects(&[id]).unwrap();
        if toolbar { c.top11_invoke(Top11Action::Duplicate).unwrap(); c.submit("UseLastDistance=Yes").unwrap(); }
        else { c.submit("Copy UseLastDistance=Yes").unwrap(); }
        c.accept_world_point([0.; 3]).unwrap();
        let first = [sign * size, 0., 0.];
        assert_eq!(c.transform_preview(first, "").unwrap(), vec![[first; 2]]);
        c.accept_world_point(first).unwrap();
        let second = [0., sign * size * 2., 0.]; let expected = [0., sign * size, 0.];
        assert_eq!(c.transform_preview(second, "").unwrap(), vec![[expected; 2]]);
        c.accept_world_point(second).unwrap(); c.submit("").unwrap();
        assert!(c.document().entities.values().any(|e| e.id != id && e.placement.point([0.; 3]).unwrap() == expected));
    }}}
}

#[test]
fn rotate_tiny_and_large_reference_vectors_are_scale_independent() {
    for toolbar in [false, true] { for size in [1e-13, 1e-200, 1e200] { for sign in [-1., 1.] {
        let source = Entity::new("point", Geometry::Point([1., 0., 0.]), None); let id = source.id;
        let mut doc = Document::with_template_tolerance("rotate scale"); doc.entities.insert(id, source);
        let mut c = Controller::new(doc, Limits::default()).unwrap(); c.select_objects(&[id]).unwrap();
        if toolbar { c.top11_invoke(Top11Action::Rotate).unwrap(); } else { c.submit("Rotate").unwrap(); }
        c.accept_world_point([0.; 3]).unwrap(); c.accept_world_point([size, 0., 0.]).unwrap();
        let target = [0., sign * size, 0.]; let preview = c.transform_preview(target, "").unwrap();
        assert!((preview[0][0][0]).abs() < 1e-9 && (preview[0][0][1] - sign).abs() < 1e-9);
        c.accept_world_point(target).unwrap(); let actual = c.document().entities[&id].placement.point([1., 0., 0.]).unwrap();
        assert!(actual[0].abs() < 1e-9 && (actual[1] - sign).abs() < 1e-9, "{size} {actual:?}");
    }}}
}

#[test]
fn copy_and_rotate_tiny_typed_coordinates_use_the_same_point_owner() {
    for action in [Top11Action::Duplicate, Top11Action::Rotate] {
        let source = Entity::new("point", Geometry::Point([0.; 3]), None); let id = source.id;
        let mut doc = Document::with_template_tolerance("tiny typed points"); doc.entities.insert(id, source);
        let mut c = Controller::new(doc, Limits::default()).unwrap(); c.select_objects(&[id]).unwrap();
        c.submit(action.command()).unwrap(); c.submit("0,0,0").unwrap();
        c.submit("0.0000000000001,0,0").unwrap();
        if action == Top11Action::Rotate { c.submit("0,0.0000000000001,0").unwrap(); }
        else { c.submit("").unwrap(); }
        if action == Top11Action::Duplicate {
            let copy = c.document().entities.values().find(|e| e.id != id).unwrap();
            assert_eq!(copy.placement.point([0.; 3]).unwrap(), [1e-13, 0., 0.]);
        } else { assert!(!c.is_running()); }
    }
}

#[test]
fn copy_repeat_direction_uses_scale_safe_previous_vector() {
    for size in [1e-200, 1e-13, 1e200] {
        for both in [false, true] {
            let source = Entity::new("point", Geometry::Point([0.; 3]), None); let id = source.id;
            let mut doc = Document::with_template_tolerance("copy direction scale"); doc.entities.insert(id, source);
            let mut c = Controller::new(doc, Limits::default()).unwrap(); c.select_objects(&[id]).unwrap();
            c.submit(if both { "Copy UseLastDirection=Yes UseLastDistance=Yes" } else { "Copy UseLastDirection=Yes" }).unwrap();
            c.accept_world_point([0.; 3]).unwrap(); c.accept_world_point([size, 0., 0.]).unwrap();
            let expected = [if both { size } else { 2. * size }, 0., 0.];
            assert_eq!(c.transform_preview([0., 2. * size, 0.], "").unwrap(), vec![[expected; 2]]);
            let before = c.document().clone(); c.accept_world_point([0., 2. * size, 0.]).unwrap();
            let added = c.document().entities.values().find(|e| !before.entities.contains_key(&e.id)).unwrap();
            assert_eq!(added.placement.point([0.; 3]).unwrap(), expected);
        }
    }
}

#[test]
fn native_brep_toolbar_disables_unavailable_topology_operations() {
    use om9_core::geometry::{Curve, Mesh, NativeBrep};
    let native = || Entity::new("native", Geometry::NativeBrep(NativeBrep {
        display: Mesh { vertices: vec![[0., 0., 0.], [2., 0., 0.], [0., 2., 0.]], faces: vec![vec![0, 1, 2]], normals: vec![], colors: vec![], corner_uvs: vec![] },
        payload: vec![1, 2, 3].into(), crc: 0,
    }), None);
    let first = native(); let second = native();
    let curve = Entity::new("curve", Geometry::Curve(Curve::Line { start: [0.; 3], end: [2., 0., 0.], domain: [0., 1.] }), None);
    let ids = [first.id, second.id, curve.id];
    let mut doc = Document::with_template_tolerance("native BRep availability");
    for entity in [first, second, curve] { doc.entities.insert(entity.id, entity); }
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    for selection in [&ids[..1], &ids[..2], &[ids[0], ids[2]][..]] {
        c.select_objects(selection).unwrap();
        let before = c.document().clone();
        let states = c.top11_availability();
        for action in [Top11Action::Join, Top11Action::Split, Top11Action::Trim] {
            let state = states.iter().find(|s| s.action == action).unwrap();
            assert!(!state.enabled, "{action:?} incorrectly enabled for native BRep selection {selection:?}");
            assert!(!state.reason.is_empty());
        }
        let explode = states.iter().find(|s| s.action == Top11Action::Explode).unwrap();
        assert_eq!(explode.enabled, cfg!(feature = "occt-brep"));
        for action in [Top11Action::Duplicate, Top11Action::Move, Top11Action::Rotate, Top11Action::Mirror] {
            assert!(states.iter().any(|s| s.action == action && s.enabled), "{action:?}");
        }
        for action in [Top11Action::Explode, Top11Action::Join, Top11Action::Split] {
            assert!(c.top11_invoke(action).is_err(), "{action:?}");
            assert_eq!(c.document(), &before); assert!(!c.is_running());
        }
        if selection.len() >= 2 {
            assert!(c.top11_invoke(Top11Action::Trim).is_err());
            assert_eq!(c.document(), &before); assert!(!c.is_running());
        }
    }
    c.select_objects(&ids[..1]).unwrap();
    let before = c.document().clone();
    c.top11_invoke(Top11Action::Mirror).unwrap(); c.submit("Copy=No").unwrap();
    c.accept_world_point([0.; 3]).unwrap(); c.accept_world_point([0., 1., 0.]).unwrap();
    let entity = &c.document().entities[&ids[0]];
    assert!(matches!(entity.geometry, Geometry::NativeBrep(_)));
    assert_eq!(entity.placement.point([2., 0., 0.]).unwrap(), [-2., 0., 0.]);
    c.undo().unwrap(); assert_eq!(c.document(), &before);
}

#[test]
fn rotate_numeric_draft_preview_matches_angle_commit() {
    let source = Entity::new("point", Geometry::Point([2., 0., 0.]), None);
    let id = source.id;
    let mut doc = Document::with_template_tolerance("numeric preview");
    doc.entities.insert(id, source);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap(); c.submit("Rotate").unwrap(); c.accept_world_point([0.; 3]).unwrap();
    let preview = c.transform_preview([0.; 3], "-90").unwrap();
    assert_eq!(preview.len(), 1);
    c.submit("-90").unwrap();
    let entity = &c.document().entities[&id];
    let Geometry::Point(point) = entity.geometry else { panic!("point") };
    let actual = entity.placement.point(point).unwrap();
    for (a, b) in preview[0][0].iter().zip(actual) { assert!((a-b).abs() < 1e-9); }
}

#[test]
fn top11_move_can_start_before_selection_and_undo() {
    let mut c = controller();
    let id = draw_line(&mut c, [0.0; 3], [10.0, 0.0, 0.0]);
    c.select_objects(&[]).unwrap();
    c.top11_invoke(Top11Action::Move).unwrap();
    assert!(c.top11_needs_object());
    assert!(c.submit("").is_err());
    c.top11_accept_object(id).unwrap();
    c.submit("").unwrap();
    assert!(!c.top11_needs_object());
    c.accept_world_point([0.0; 3]).unwrap();
    c.accept_world_point([2.0, 3.0, 0.0]).unwrap();
    assert_eq!(
        c.document().entities[&id]
            .placement
            .point([0.0; 3])
            .unwrap(),
        [2.0, 3.0, 0.0]
    );
    c.undo().unwrap();
    assert_eq!(
        c.document().entities[&id]
            .placement
            .point([0.0; 3])
            .unwrap(),
        [0.0; 3]
    );
}

#[test]
fn mirror_can_start_before_selection_and_cancel() {
    let mut c = controller();
    let id = draw_line(&mut c, [0.0; 3], [10.0, 0.0, 0.0]);
    c.select_objects(&[]).unwrap();
    c.submit("Mirror").unwrap();
    c.top11_accept_object(id).unwrap();
    c.cancel();
    assert!(!c.is_running());
    assert_eq!(c.document().entities.len(), 1);
}

#[test]
fn every_top11_action_can_open_object_selection_and_cancel() {
    for action in Top11Action::ALL {
        if action == Top11Action::RingRail {
            continue;
        }
        let mut c = controller();
        let id = draw_line(&mut c, [0.0; 3], [10.0, 0.0, 0.0]);
        c.select_objects(&[]).unwrap();
        let before = c.document().clone();
        assert!(
            c.top11_availability()
                .iter()
                .any(|state| state.action == action && state.enabled)
        );
        c.top11_invoke(action).unwrap();
        assert!(c.top11_needs_object(), "{action:?}");
        c.top11_accept_object(id).unwrap();
        c.cancel();
        assert_eq!(c.document(), &before, "{action:?}");
    }
}

#[test]
fn top11_can_remove_picked_selection_and_cutters_before_commit() {
    let mut c = controller();
    let target = draw_line(&mut c, [-10.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let cutter = draw_line(&mut c, [0.0, -10.0, 0.0], [0.0, 10.0, 0.0]);
    c.select_objects(&[]).unwrap();
    c.top11_invoke(Top11Action::Split).unwrap();
    c.top11_accept_object(target).unwrap();
    c.top11_remove_object(target).unwrap();
    assert!(c.submit("").is_err());
    c.top11_accept_object(target).unwrap();
    c.submit("").unwrap();
    c.top11_accept_object(cutter).unwrap();
    c.top11_remove_object(cutter).unwrap();
    assert!(c.submit("").is_err());
    c.top11_accept_object(cutter).unwrap();
    c.submit("").unwrap();
    assert_eq!(c.document().entities.len(), 3);
    c.undo().unwrap();
    assert_eq!(c.document().entities.len(), 2);
    assert!(c.document().entities.contains_key(&target));
    assert!(c.document().entities.contains_key(&cutter));
}

#[test]
fn split_multiple_curves_is_one_undo_and_preserves_unhit_target() {
    let mut c = controller();
    let first = draw_line(&mut c, [-10., 0., 0.], [10., 0., 0.]);
    let second = draw_line(&mut c, [-10., 4., 0.], [10., 4., 0.]);
    let untouched = draw_line(&mut c, [30., 0., 0.], [40., 0., 0.]);
    let cutter = draw_line(&mut c, [0., -5., 0.], [0., 10., 0.]);
    c.select_objects(&[first, second, untouched]).unwrap();
    let undo_before = c.coordinator().undo_len();
    c.top11_invoke(Top11Action::Split).unwrap();
    c.top11_accept_object(cutter).unwrap();
    c.submit("").unwrap();
    assert_eq!(c.document().entities.len(), 6);
    assert!(c.document().entities.contains_key(&untouched));
    assert!(c.document().entities.contains_key(&cutter));
    assert!(!c.document().entities.contains_key(&first));
    assert!(!c.document().entities.contains_key(&second));
    assert_eq!(c.coordinator().undo_len(), undo_before + 1);
    c.undo().unwrap();
    assert_eq!(c.document().entities.len(), 4);
    assert!(c.document().entities.contains_key(&first));
    assert!(c.document().entities.contains_key(&second));
    c.redo().unwrap();
    assert_eq!(c.document().entities.len(), 6);
}

#[test]
fn split_and_trim_tiny_open_nurbs_and_polycurves_remain_open() {
    use om9_core::geometry::{Curve,NurbsCurve};let limits=Limits::default();
    for nurbs in [false,true] {for split in [false,true] {for toolbar in [false,true] {
        let segment=|a,b|Curve::Line{start:[a,0.,0.],end:[b,0.,0.],domain:[0.,1.]};
        let curve=if nurbs {Curve::Nurbs(NurbsCurve{degree:1,poles:vec![[0.,0.,0.],[1e-10,0.,0.]],weights:vec![1.;2],knots:vec![0.,0.,1.,1.],domain:[0.,1.],periodic:false,reversed:false,seam:None})} else {Curve::Polycurve{segments:vec![segment(0.,5e-11),segment(5e-11,1e-10)],reversed:false}};
        let target=Entity::new("tiny",Geometry::Curve(curve),None);let cutter=|name,x|Entity::new(name,Geometry::Curve(Curve::Line{start:[x,-1e-10,0.],end:[x,1e-10,0.],domain:[0.,1.]}),None);
        let a=cutter("a",2.5e-11);let b=cutter("b",7.5e-11);let ids=[target.id,a.id,b.id];let mut doc=Document::with_template_tolerance("tiny open curve");doc.tolerance.as_mut().unwrap().absolute_mm=1e-13;for e in [target,a,b]{doc.entities.insert(e.id,e);}
        let mut c=Controller::new(doc,limits).unwrap();select(&mut c,&[ids[0]]);let before=c.document().clone();let action=if split{Top11Action::Split}else{Top11Action::Trim};if toolbar{c.top11_invoke(action).unwrap();}else{c.submit(action.label()).unwrap();}
        c.top11_accept_object(ids[1]).unwrap();c.top11_accept_object(ids[2]).unwrap();c.submit("").unwrap();if !split{c.top11_trim_many_accept_pick(ids[0],[1e-11,0.,0.]).unwrap();c.submit("").unwrap();}
        let after=c.document().clone();let pieces:Vec<_>=after.entities.iter().filter(|(id,_)|**id!=ids[1]&&**id!=ids[2]).collect();assert_eq!(pieces.len(),if split{3}else{1},"nurbs={nurbs} split={split}");
        let mut total=0.;for (_,e) in pieces{let Geometry::Curve(curve)=&e.geometry else{panic!("curve")};total+=om9_core::curve_ops::arc_length(curve,&limits).unwrap();}assert!((total-(if split{1e-10}else{7.5e-11})).abs()<1e-20,"{total}");
        let path=std::env::temp_dir().join(format!("om9-tiny-open-curve-{}.om9",uuid::Uuid::new_v4()));om9_archive::Archive::save(&path,&after,&limits,&om9_core::transaction::Cancellation::default()).unwrap();assert_eq!(om9_archive::Archive::open(&path,&limits).unwrap(),after);std::fs::remove_file(path).unwrap();
        c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }}}
}

#[test]
fn split_and_trim_accept_cutter_endpoint_within_document_tolerance() {
    use om9_core::geometry::Curve;
    let limits=Limits::default();
    for split in [false,true] {for toolbar in [false,true] {
        let line=|start,end|Geometry::Curve(Curve::Line{start,end,domain:[0.,1.]});
        let target=Entity::new("target",line([0.,0.,0.],[10.,0.,0.]),None);
        let cutter=Entity::new("cutter",line([5.,0.0005,0.],[5.,2.,0.]),None);let ids=[target.id,cutter.id];
        let mut doc=Document::with_template_tolerance("endpoint tolerance");doc.tolerance.as_mut().unwrap().absolute_mm=0.001;doc.entities.insert(ids[0],target);doc.entities.insert(ids[1],cutter);
        let mut c=Controller::new(doc,limits).unwrap();select(&mut c,&[ids[0]]);let before=c.document().clone();
        let action=if split{Top11Action::Split}else{Top11Action::Trim};if toolbar{c.top11_invoke(action).unwrap();}else{c.submit(action.label()).unwrap();}
        c.top11_accept_object(ids[1]).unwrap();c.submit("").unwrap();
        if !split{c.top11_trim_many_accept_pick(ids[0],[2.,0.,0.]).unwrap();c.submit("").unwrap();}
        let after=c.document().clone();let pieces:Vec<_>=after.entities.values().filter(|e|e.label=="target").collect();assert_eq!(pieces.len(),if split{2}else{1});
        for piece in pieces{let Geometry::Curve(curve)=&piece.geometry else{panic!("curve")};let ends=om9_core::curve_ops::curve_endpoints(curve,&limits).unwrap();assert!(ends.iter().any(|p|(p[0]-5.).abs()<1e-10&&p[1].abs()<1e-10),"{ends:?}");}
        let path=std::env::temp_dir().join(format!("om9-endpoint-tolerance-{}.om9",uuid::Uuid::new_v4()));om9_archive::Archive::save(&path,&after,&limits,&om9_core::transaction::Cancellation::default()).unwrap();assert_eq!(om9_archive::Archive::open(&path,&limits).unwrap(),after);std::fs::remove_file(path).unwrap();
        c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }}
}

#[test]
fn split_and_trim_tiny_open_arc_remains_open() {
    use om9_core::geometry::Curve;let limits=Limits::default();let radius=1e-10;let pi=std::f64::consts::PI;
    for split in [false,true] {for toolbar in [false,true] {
        let curve=Curve::Circle{center:[0.;3],x:[1.,0.,0.],y:[0.,1.,0.],radius,domain:[0.,pi],angle_domain:None};
        let target=Entity::new("arc",Geometry::Curve(curve.clone()),None);let cutter=|name,angle:f64|Entity::new(name,Geometry::Curve(Curve::Line{start:[0.,0.,0.],end:[2.*radius*angle.cos(),2.*radius*angle.sin(),0.],domain:[0.,1.]}),None);
        let a=cutter("a",pi/4.);let b=cutter("b",3.*pi/4.);let ids=[target.id,a.id,b.id];let mut doc=Document::with_template_tolerance("tiny open arc");doc.tolerance.as_mut().unwrap().absolute_mm=1e-13;for e in [target,a,b]{doc.entities.insert(e.id,e);}
        let mut c=Controller::new(doc,limits).unwrap();select(&mut c,&[ids[0]]);let before=c.document().clone();let action=if split{Top11Action::Split}else{Top11Action::Trim};if toolbar{c.top11_invoke(action).unwrap();}else{c.submit(action.label()).unwrap();}
        c.top11_accept_object(ids[1]).unwrap();c.top11_accept_object(ids[2]).unwrap();c.submit("").unwrap();if !split {let p=om9_core::curve_ops::point_at(&curve,0.125,&limits).unwrap();c.top11_trim_many_accept_pick(ids[0],p).unwrap();c.submit("").unwrap();}
        let after=c.document().clone();let pieces:Vec<_>=after.entities.iter().filter(|(id,_)|**id!=ids[1]&&**id!=ids[2]).collect();assert_eq!(pieces.len(),if split{3}else{1});
        let mut total=0.;for (_,e) in pieces {let Geometry::Curve(curve)=&e.geometry else{panic!("curve")};total+=om9_core::curve_ops::arc_length(curve,&limits).unwrap();}
        assert!((total-(if split{1.}else{0.75})*pi*radius).abs()<1e-20,"{total}");c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }}
}

#[test]
fn split_and_trim_periodic_quadratic_with_tangent_cutters() {
    use om9_core::{curve_ops,geometry::{Curve,NurbsCurve}};
    let limits=Limits::default();
    for split in [false,true] {for toolbar in [false,true] {for reversed in [false,true] {for shifted in [false,true] {
        let curve=Curve::Nurbs(NurbsCurve{degree:2,poles:vec![[0.,0.,0.],[2.,0.,0.],[0.,2.,0.],[-2.,0.,0.],[0.,0.,0.],[2.,0.,0.]],weights:vec![1.;6],knots:vec![-2.,-1.,0.,1.,2.,3.,4.,5.,6.],domain:[0.,4.],periodic:true,reversed,seam:None});
        let curve=if shifted {curve_ops::set_seam(&curve,0.31,1e-9,&limits).unwrap()} else {curve};
        let target=Entity::new("periodic",Geometry::Curve(curve),None);
        let cutter=|name,x| Entity::new(name,Geometry::Curve(Curve::Line{start:[x,-1.,0.],end:[x,2.,0.],domain:[0.,1.]}),None);
        let a=cutter("right",1.5);let b=cutter("left",-1.5);let ids=[target.id,a.id,b.id];let mut doc=Document::with_template_tolerance("periodic tangents");for e in [target,a,b]{doc.entities.insert(e.id,e);}
        let mut c=Controller::new(doc,limits).unwrap();select(&mut c,&[ids[0]]);let before=c.document().clone();let action=if split {Top11Action::Split}else{Top11Action::Trim};
        if toolbar {c.top11_invoke(action).unwrap();}else{c.submit(action.label()).unwrap();}c.top11_accept_object(ids[1]).unwrap();c.top11_accept_object(ids[2]).unwrap();c.submit("").unwrap_or_else(|e|panic!("split={split} toolbar={toolbar} reversed={reversed} shifted={shifted}: {e:?}"));
        if !split {c.top11_trim_many_accept_pick(ids[0],[0.,1.5,0.]).unwrap_or_else(|e|panic!("pick split={split} toolbar={toolbar} reversed={reversed} shifted={shifted}: {e:?}"));c.submit("").unwrap();}
        let after=c.document().clone();let pieces:Vec<_>=after.entities.iter().filter(|(id,_)|**id!=ids[1]&&**id!=ids[2]).collect();assert_eq!(pieces.len(),if split{2}else{1});
        for (_,e) in pieces {let Geometry::Curve(curve)=&e.geometry else {panic!("curve")};let ends=curve_ops::curve_endpoints(curve,&limits).unwrap();for x in [-1.5,1.5] {assert!(ends.iter().any(|p|(p[0]-x).abs()<1e-8&&(p[1]-0.25).abs()<1e-8),"{ends:?}");}}
        let path=std::env::temp_dir().join(format!("om9-periodic-tangent-{}.om9",uuid::Uuid::new_v4()));om9_archive::Archive::save(&path,&after,&limits,&om9_core::transaction::Cancellation::default()).unwrap();assert_eq!(om9_archive::Archive::open(&path,&limits).unwrap(),after);std::fs::remove_file(path).unwrap();
        c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }}}}
}

#[test]
fn split_and_trim_multispan_quadratic_nurbs_tangent() {
    use om9_core::geometry::{Curve, NurbsCurve};
    for split in [false, true] { for toolbar in [false, true] { for reversed in [false, true] {
        let curve = Curve::Nurbs(NurbsCurve { degree: 2, poles: vec![[0.,100.,0.], [0.5,-100.,0.], [1.,100.,0.], [1.5,200.,0.], [2.,300.,0.]], weights: vec![1.; 5], knots: vec![0.,0.,0.,0.5,0.5,1.,1.,1.], domain: [0.,1.], periodic: false, reversed, seam: None });
        let target = Entity::new("multispan", Geometry::Curve(curve), None);
        let cutter = Entity::new("cutter", Geometry::Curve(Curve::Line { start: [0.,0.,0.], end: [2.,0.,0.], domain: [0.,1.] }), None);
        let ids = [target.id,cutter.id]; let mut doc = Document::with_template_tolerance("multispan tangent"); doc.entities.insert(ids[0],target); doc.entities.insert(ids[1],cutter);
        let mut c = Controller::new(doc,Limits::default()).unwrap(); select(&mut c,&[ids[0]]); let before=c.document().clone();
        let action=if split {Top11Action::Split} else {Top11Action::Trim}; if toolbar {c.top11_invoke(action).unwrap();} else {c.submit(action.label()).unwrap();}
        c.top11_accept_object(ids[1]).unwrap(); c.submit("").unwrap();
        if !split {c.top11_trim_many_accept_pick(ids[0],[0.,100.,0.]).unwrap(); c.submit("").unwrap();}
        let after=c.document().clone();let pieces: Vec<_>=after.entities.iter().filter(|(id,_)|**id!=ids[1]).collect();assert_eq!(pieces.len(),if split {2} else {1});
        for (_,e) in pieces {let Geometry::Curve(curve)=&e.geometry else {panic!("curve")};let ends=om9_core::curve_ops::curve_endpoints(curve,&Limits::default()).unwrap();assert!(ends.iter().any(|p|(p[0]-0.5).abs()<1e-8 && p[1].abs()<1e-8),"{ends:?}");}
        let path=std::env::temp_dir().join(format!("om9-multispan-tangent-{}.om9",uuid::Uuid::new_v4()));om9_archive::Archive::save(&path,&after,&Limits::default(),&om9_core::transaction::Cancellation::default()).unwrap();assert_eq!(om9_archive::Archive::open(&path,&Limits::default()).unwrap(),after);std::fs::remove_file(path).unwrap();
        c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }}}
}

#[test]
fn split_and_trim_quadratic_nurbs_close_crossings_and_tangent() {
    use om9_core::geometry::{Curve, NurbsCurve};
    for tangent in [false, true] { for split in [false, true] { for toolbar in [false, true] {
        let center = if tangent { 0.5_f64 } else { 0.501_f64 }; let width = if tangent { 0.0_f64 } else { 0.001_f64 };
        let constant = center * center - width * width; let scale = 1000.;
        let curve = Curve::Nurbs(NurbsCurve { degree: 2, poles: vec![[0., scale*constant, 0.], [0.5, scale*(constant-center), 0.], [1., scale*(1.-2.*center+constant), 0.]], weights: vec![1.; 3], knots: vec![0., 0., 0., 1., 1., 1.], domain: [0., 1.], periodic: false, reversed: false, seam: None });
        let target = Entity::new("quadratic", Geometry::Curve(curve), None);
        let cutter = Entity::new("cutter", Geometry::Curve(Curve::Line { start: [0., 0., 0.], end: [1., 0., 0.], domain: [0., 1.] }), None);
        let ids = [target.id, cutter.id]; let mut doc = Document::with_template_tolerance("quadratic intersections");
        doc.entities.insert(ids[0], target); doc.entities.insert(ids[1], cutter);
        let mut c = Controller::new(doc, Limits::default()).unwrap(); select(&mut c, &[ids[0]]); let before = c.document().clone();
        let action = if split { Top11Action::Split } else { Top11Action::Trim };
        if toolbar { c.top11_invoke(action).unwrap(); } else { c.submit(action.label()).unwrap(); }
        c.top11_accept_object(ids[1]).unwrap(); c.submit("").unwrap();
        if !split { let pick = if tangent { [0., 250., 0.] } else { [center, -0.001, 0.] }; c.top11_trim_many_accept_pick(ids[0], pick).unwrap(); c.submit("").unwrap(); }
        let after = c.document().clone(); let pieces: Vec<_> = after.entities.iter().filter(|(id, _)| **id != ids[1]).collect();
        assert_eq!(pieces.len(), if split { if tangent {2} else {3} } else { if tangent {1} else {2} });
        let mut ends = Vec::new();
        for (_, e) in pieces { let Geometry::Curve(curve) = &e.geometry else { panic!("curve") }; ends.extend(om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap()); }
        for x in if tangent { vec![center] } else { vec![center-width, center+width] } { assert!(ends.iter().any(|p| (p[0] - x).abs() < 1e-6 && p[1].abs() < 1e-6), "{ends:?}"); }
        let path = std::env::temp_dir().join(format!("om9-quadratic-intersections-{}.om9", uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&path, &after, &Limits::default(), &om9_core::transaction::Cancellation::default()).unwrap();
        assert_eq!(om9_archive::Archive::open(&path, &Limits::default()).unwrap(), after); std::fs::remove_file(path).unwrap();
        c.undo().unwrap(); assert_eq!(c.document(), &before); c.redo().unwrap(); assert_eq!(c.document(), &after);
    }}}
}

#[test]
fn split_and_trim_find_narrow_nurbs_span_crossings() {
    use om9_core::geometry::{Curve, NurbsCurve};
    for split in [false, true] { for toolbar in [false, true] {
        let target = Entity::new("narrow", Geometry::Curve(Curve::Nurbs(NurbsCurve {
            degree: 1, poles: vec![[0., 0., 0.], [1., 0., 0.], [500., 0., 0.], [0., -100., 0.], [502., 0., 0.], [503., 0., 0.]],
            weights: vec![1.; 6], knots: vec![0., 0., 0.4, 0.9, 0.90001, 0.90002, 1., 1.], domain: [0., 1.], periodic: false, reversed: false, seam: None,
        })), None);
        let cutter = Entity::new("cutter", Geometry::Curve(Curve::Line { start: [0., -50., 0.], end: [502., -50., 0.], domain: [0., 1.] }), None);
        let ids = [target.id, cutter.id]; let mut doc = Document::with_template_tolerance("narrow knot intersections");
        doc.entities.insert(ids[0], target); doc.entities.insert(ids[1], cutter);
        let mut c = Controller::new(doc, Limits::default()).unwrap(); select(&mut c, &[ids[0]]); let before = c.document().clone();
        let action = if split { Top11Action::Split } else { Top11Action::Trim };
        if toolbar { c.top11_invoke(action).unwrap(); } else { c.submit(action.label()).unwrap(); }
        c.top11_accept_object(ids[1]).unwrap(); c.submit("").unwrap();
        if !split { c.top11_trim_many_accept_pick(ids[0], [0., -100., 0.]).unwrap(); c.submit("").unwrap(); }
        let after = c.document().clone(); let pieces: Vec<_> = after.entities.iter().filter(|(id, _)| **id != ids[1]).collect();
        assert_eq!(pieces.len(), if split { 3 } else { 2 });
        let mut ends = Vec::new();
        for (_, e) in pieces { let Geometry::Curve(curve) = &e.geometry else { panic!("curve") }; ends.extend(om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap()); }
        for x in [250., 251.] { assert!(ends.iter().any(|p| (p[0] - x).abs() < 1e-4 && (p[1] + 50.).abs() < 1e-4), "{ends:?}"); }
        let path = std::env::temp_dir().join(format!("om9-narrow-nurbs-{}.om9", uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&path, &after, &Limits::default(), &om9_core::transaction::Cancellation::default()).unwrap();
        assert_eq!(om9_archive::Archive::open(&path, &Limits::default()).unwrap(), after); std::fs::remove_file(path).unwrap();
        c.undo().unwrap(); assert_eq!(c.document(), &before); c.redo().unwrap(); assert_eq!(c.document(), &after);
    }}
}

#[test]
fn split_and_trim_find_crossings_on_short_polycurve_segments() {
    use om9_core::geometry::Curve;
    let segments: Vec<_> = (0..600).map(|i| {
        let height = |n| if n == 301 { 3.0 } else { 0.0 };
        Curve::Line { start: [i as f64, height(i), 0.], end: [(i + 1) as f64, height(i + 1), 0.], domain: [0., 1.] }
    }).collect();
    for split in [false, true] { for toolbar in [false, true] {
        let target = Entity::new("toothed", Geometry::Curve(Curve::Polycurve { segments: segments.clone(), reversed: false }), None);
        let cutter = Entity::new("cutter", Geometry::Curve(Curve::Line { start: [300., 1., 0.], end: [302., 1., 0.], domain: [0., 1.] }), None);
        let ids = [target.id, cutter.id]; let mut doc = Document::with_template_tolerance("short polycurve segments");
        doc.entities.insert(ids[0], target); doc.entities.insert(ids[1], cutter);
        let mut c = Controller::new(doc, Limits::default()).unwrap(); select(&mut c, &[ids[0]]); let before = c.document().clone();
        let action = if split { Top11Action::Split } else { Top11Action::Trim };
        if toolbar { c.top11_invoke(action).unwrap(); } else { c.submit(action.label()).unwrap(); }
        c.top11_accept_object(ids[1]).unwrap(); c.submit("").unwrap();
        if !split { c.top11_trim_many_accept_pick(ids[0], [301., 3., 0.]).unwrap(); c.submit("").unwrap(); }
        let after = c.document().clone(); let pieces: Vec<_> = after.entities.iter().filter(|(id, _)| **id != ids[1]).collect();
        assert_eq!(pieces.len(), if split { 3 } else { 2 });
        let mut ends = Vec::new();
        for (_, e) in pieces { let Geometry::Curve(curve) = &e.geometry else { panic!("curve") }; ends.extend(om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap()); }
        for x in [300. + 1. / 3., 302. - 1. / 3.] { assert!(ends.iter().any(|p| (p[0] - x).abs() < 1e-8 && (p[1] - 1.).abs() < 1e-8), "{ends:?}"); }
        let path = std::env::temp_dir().join(format!("om9-polycurve-crossings-{}.om9", uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&path, &after, &Limits::default(), &om9_core::transaction::Cancellation::default()).unwrap();
        assert_eq!(om9_archive::Archive::open(&path, &Limits::default()).unwrap(), after); std::fs::remove_file(path).unwrap();
        c.undo().unwrap(); assert_eq!(c.document(), &before); c.redo().unwrap(); assert_eq!(c.document(), &after);
    }}
}

#[test]
fn split_closed_circle_with_chord_makes_two_pieces() {
    use om9_core::geometry::Curve;
    for y in [0.0, 0.999998] { for toolbar in [false, true] {
        let circle = Entity::new("circle", Geometry::Curve(Curve::Circle { center: [0., 0., 0.], x: [1., 0., 0.], y: [0., 1., 0.], radius: 1., domain: [0., std::f64::consts::TAU], angle_domain: None }), None);
        let target = circle.id;
        let chord = Entity::new("chord", Geometry::Curve(Curve::Line { start: [-2., y, 0.], end: [2., y, 0.], domain: [0., 1.] }), None);
        let cutter = chord.id;
        let mut doc = Document::with_template_tolerance("closed circle split");
        doc.entities.insert(target, circle); doc.entities.insert(cutter, chord);
        let mut c = Controller::new(doc, Limits::default()).unwrap();
        select(&mut c, &[target]); let before = c.document().clone();
        if toolbar { c.top11_invoke(Top11Action::Split).unwrap(); } else { c.submit("Split").unwrap(); }
        c.top11_accept_object(cutter).unwrap(); c.submit("").unwrap();
        let after = c.document().clone();
        assert_eq!(after.entities.len(), 3);
        let mut length = 0.0;
        for (id, entity) in &after.entities { if *id != cutter {
            let Geometry::Curve(curve) = &entity.geometry else { panic!("curve") };
            length += om9_core::curve_ops::arc_length(curve, &Limits::default()).unwrap();
            for p in om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap() { assert!((p[1] - y).abs() < 1e-8); }
        }}
        assert!((length - std::f64::consts::TAU).abs() < 1e-7);
        c.undo().unwrap(); assert_eq!(c.document(), &before);
        c.redo().unwrap(); assert_eq!(c.document(), &after);
    }}
}

#[test]
fn split_and_trim_keep_two_close_line_circle_intersections() {
    use om9_core::geometry::Curve;let y=0.999998_f64;let crossing=(1.-y*y).sqrt();
    for split in [false,true] {for toolbar in [false,true] {
        let target=Entity::new("chord",Geometry::Curve(Curve::Line{start:[-2.,y,0.],end:[2.,y,0.],domain:[0.,1.]}),None);let id=target.id;
        let cutter=Entity::new("circle",Geometry::Curve(Curve::Circle{center:[0.,0.,0.],x:[1.,0.,0.],y:[0.,1.,0.],radius:1.,domain:[0.,std::f64::consts::TAU],angle_domain:None}),None);let cutter_id=cutter.id;
        let mut doc=Document::with_template_tolerance("near tangent chord");doc.entities.insert(id,target);doc.entities.insert(cutter_id,cutter);
        let mut c=Controller::new(doc,Limits::default()).unwrap();select(&mut c,&[id]);let before=c.document().clone();let action=if split{Top11Action::Split}else{Top11Action::Trim};
        if toolbar {c.top11_invoke(action).unwrap();}else{c.submit(action.label()).unwrap();}c.top11_accept_object(cutter_id).unwrap();c.submit("").unwrap();
        if !split {c.top11_trim_many_accept_pick(id,[0.,y,0.]).unwrap();c.submit("").unwrap();}
        let after=c.document().clone();let mut ends:Vec<_>=after.entities.iter().filter(|(id,_)|**id!=cutter_id).map(|(_,e)|match &e.geometry {Geometry::Curve(curve)=>om9_core::curve_ops::curve_endpoints(curve,&Limits::default()).unwrap(),_=>panic!("curve")}).collect();
        ends.sort_by(|a,b|a[0][0].total_cmp(&b[0][0]));assert_eq!(ends.len(),if split{3}else{2});
        assert!((ends[0][1][0]+crossing).abs()<1e-8);assert!((ends.last().unwrap()[0][0]-crossing).abs()<1e-8);
        c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }}
}

#[test]
fn split_and_trim_circle_arcs_only_use_intersections_in_both_intervals() {
    use om9_core::geometry::Curve;let pi=std::f64::consts::PI;
    let arc=|name,center,angles|Entity::new(name,Geometry::Curve(Curve::Circle{center,x:[1.,0.,0.],y:[0.,1.,0.],radius:1.,domain:[7.,12.],angle_domain:Some(angles)}),None);
    for split in [false,true] {for toolbar in [false,true] {
        let target=arc("target",[0.,0.,0.],[0.,pi]);let id=target.id;
        let invalid=arc("lower",[1.,0.,0.],[pi,2.*pi]);let invalid_id=invalid.id;
        let valid=arc("upper",[1.,0.,0.],[0.,pi]);let valid_id=valid.id;
        let mut doc=Document::with_template_tolerance("arc intersection");for e in [target,invalid,valid] {doc.entities.insert(e.id,e);}
        let mut c=Controller::new(doc,Limits::default()).unwrap();select(&mut c,&[id]);let before=c.document().clone();
        let action=if split {Top11Action::Split}else{Top11Action::Trim};if toolbar {c.top11_invoke(action).unwrap();}else{c.submit(action.label()).unwrap();}
        if c.top11_accept_object(invalid_id).is_ok() {assert!(c.submit("").is_err());c.top11_remove_object(invalid_id).unwrap();}assert_eq!(c.document(),&before);
        c.top11_accept_object(valid_id).unwrap();c.submit("").unwrap();
        if !split {c.accept_world_point([-1.,0.,0.]).unwrap();c.submit("").unwrap();}
        let after=c.document().clone();let pieces:Vec<_>=after.entities.iter().filter(|(id,_)|**id!=invalid_id&&**id!=valid_id).collect();assert_eq!(pieces.len(),if split{2}else{1});
        let boundary=[0.5,3_f64.sqrt()/2.,0.];
        for (_,e) in pieces {let Geometry::Curve(curve)=&e.geometry else {panic!("curve")};let ends=om9_core::curve_ops::curve_endpoints(curve,&Limits::default()).unwrap();
            assert!(ends.iter().any(|p|(p[0]-boundary[0]).hypot(p[1]-boundary[1])<1e-9),"{ends:?}");}
        let path=std::env::temp_dir().join(format!("om9-split-arcs-{}.om9",uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&path,&after,&Limits::default(),&om9_core::transaction::Cancellation::default()).unwrap();assert_eq!(om9_archive::Archive::open(&path,&Limits::default()).unwrap(),after);std::fs::remove_file(path).unwrap();
        c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }}
}

#[test]
fn split_and_trim_nearly_parallel_long_lines_use_real_intersection() {
    for split in [false,true] { for toolbar in [false,true] {
        let mut c=controller();let target=draw_line(&mut c,[0.,0.,0.],[1e10,0.,0.]);let cutter=draw_line(&mut c,[0.,-0.74,0.],[1e10,1.26,0.]);
        select(&mut c,&[target]);let before=c.document().clone();let action=if split {Top11Action::Split}else{Top11Action::Trim};
        if toolbar {c.top11_invoke(action).unwrap();}else{c.submit(action.label()).unwrap();}c.top11_accept_object(cutter).unwrap();c.submit("").unwrap();
        if !split {c.accept_world_point([5e9,0.,0.]).unwrap();c.submit("").unwrap();}
        let after=c.document().clone();let mut ends:Vec<_>=after.entities.iter().filter(|(id,_)|**id!=cutter).map(|(_,e)|match &e.geometry{Geometry::Curve(curve)=>om9_core::curve_ops::curve_endpoints(curve,&Limits::default()).unwrap(),_=>panic!("curve")}).collect();
        ends.sort_by(|a,b|a[0][0].total_cmp(&b[0][0]));assert_eq!(ends.len(),if split{2}else{1});assert!((ends[0][1][0]-3.7e9).abs()<0.001);
        if split {assert!((ends[1][0][0]-3.7e9).abs()<0.001);}
        let path=std::env::temp_dir().join(format!("om9-near-parallel-{}.om9",uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&path,&after,&Limits::default(),&om9_core::transaction::Cancellation::default()).unwrap();assert_eq!(om9_archive::Archive::open(&path,&Limits::default()).unwrap(),after);std::fs::remove_file(path).unwrap();
        c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }}
}

#[test]
fn split_long_curve_keeps_distinct_near_endpoint_cutters() {
    for toolbar in [false,true] { for with_duplicate in [false,true] {
        let mut c=controller();let target=draw_line(&mut c,[0.,0.,0.],[1_000_000.,0.,0.]);
        let first=draw_line(&mut c,[0.05,-1.,0.],[0.05,1.,0.]);let second=draw_line(&mut c,[0.10,-1.,0.],[0.10,1.,0.]);
        let duplicate=with_duplicate.then(||draw_line(&mut c,[0.0505,-1.,0.],[0.0505,1.,0.]));
        select(&mut c,&[target]);let before=c.document().clone();
        if toolbar {c.top11_invoke(Top11Action::Split).unwrap();}else{c.submit("Split").unwrap();}
        c.top11_accept_object(first).unwrap();c.top11_accept_object(second).unwrap();if let Some(id)=duplicate {c.top11_accept_object(id).unwrap();}c.submit("").unwrap();
        let after=c.document().clone();let mut lengths:Vec<_>=after.entities.iter().filter(|(id,_)|**id!=first&&**id!=second&&Some(**id)!=duplicate).map(|(_,e)|match &e.geometry {Geometry::Curve(curve)=>om9_core::curve_ops::arc_length(curve,&Limits::default()).unwrap(),_=>panic!("curve")}).collect();
        lengths.sort_by(f64::total_cmp);assert_eq!(lengths.len(),3);assert!((lengths[0]-0.05).abs()<1e-8);assert!((lengths[1]-0.05).abs()<1e-8);assert!((lengths[2]-999_999.9).abs()<1e-6);
        let path=std::env::temp_dir().join(format!("om9-cutter-precision-{}.om9",uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&path,&after,&Limits::default(),&om9_core::transaction::Cancellation::default()).unwrap();assert_eq!(om9_archive::Archive::open(&path,&Limits::default()).unwrap(),after);std::fs::remove_file(path).unwrap();
        c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }}
}

#[test]
fn trim_long_curve_at_near_endpoint_cutter() {
    for toolbar in [false,true] {
        let mut c=controller();let target=draw_line(&mut c,[0.,0.,0.],[1_000_000.,0.,0.]);let cutter=draw_line(&mut c,[0.05,-1.,0.],[0.05,1.,0.]);select(&mut c,&[target]);let before=c.document().clone();
        if toolbar {c.top11_invoke(Top11Action::Trim).unwrap();}else{c.submit("Trim").unwrap();}c.top11_accept_object(cutter).unwrap();c.submit("").unwrap();c.accept_world_point([1.,0.,0.]).unwrap();c.submit("").unwrap();
        let after=c.document().clone();let result=after.entities.iter().find(|(id,_)|**id!=cutter).unwrap().1;let Geometry::Curve(curve)=&result.geometry else{panic!("curve")};let ends=om9_core::curve_ops::curve_endpoints(curve,&Limits::default()).unwrap();
        assert!(ends[0][0].abs()<1e-9);assert!((ends[1][0]-0.05).abs()<1e-8);
        c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }
}

#[test]
fn split_multiple_curves_accepts_cutters_that_hit_different_targets() {
    let mut c = controller();
    let first = draw_line(&mut c, [-10., 0., 0.], [10., 0., 0.]);
    let second = draw_line(&mut c, [-10., 4., 0.], [10., 4., 0.]);
    let cutter_a = draw_line(&mut c, [0., -2., 0.], [0., 2., 0.]);
    let cutter_b = draw_line(&mut c, [2., 2., 0.], [2., 6., 0.]);
    c.select_objects(&[first, second]).unwrap();
    let undo_before = c.coordinator().undo_len();
    c.top11_invoke(Top11Action::Split).unwrap();
    c.top11_accept_object(cutter_a).unwrap();
    c.top11_accept_object(cutter_b).unwrap();
    c.submit("").unwrap();
    assert_eq!(c.document().entities.len(), 6);
    assert_eq!(c.coordinator().undo_len(), undo_before + 1);
    c.undo().unwrap();
    assert_eq!(c.document().entities.len(), 4);
    assert!(c.document().entities.contains_key(&first));
    assert!(c.document().entities.contains_key(&second));
}

#[test]
fn trim_multiple_targets_by_picked_portion_with_undo() {
    let mut c = controller();
    let first = draw_line(&mut c, [-10., 0., 0.], [10., 0., 0.]);
    let second = draw_line(&mut c, [-10., 4., 0.], [10., 4., 0.]);
    let untouched = draw_line(&mut c, [-10., 20., 0.], [10., 20., 0.]);
    let cutter = draw_line(&mut c, [0., -5., 0.], [0., 10., 0.]);
    c.select_objects(&[first, second, untouched, cutter])
        .unwrap();
    let undo_before = c.coordinator().undo_len();
    c.submit("Trim").unwrap();
    assert!(c.accept_world_point([5., 10., 0.]).is_err());
    assert_eq!(c.coordinator().undo_len(), undo_before);
    c.accept_world_point([5., 0., 0.]).unwrap();
    c.accept_world_point([5., 4., 0.]).unwrap();
    c.submit("").unwrap();
    for id in [first, second] {
        let entity = &c.document().entities[&id];
        let Geometry::Curve(curve) = &entity.geometry else {
            panic!("curve")
        };
        let ends = om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap();
        assert!(
            (ends[0][0] + 10.).abs() < 1e-6 && ends[1][0].abs() < 1e-6,
            "{ends:?}"
        );
    }
    assert!(c.document().entities.contains_key(&untouched));
    assert_eq!(c.coordinator().undo_len(), undo_before + 2);
    c.undo().unwrap();
    c.undo().unwrap();
    for id in [first, second] {
        let Geometry::Curve(curve) = &c.document().entities[&id].geometry else {
            panic!("curve")
        };
        assert!(
            (om9_core::curve_ops::arc_length(curve, &Limits::default()).unwrap() - 20.).abs()
                < 1e-6
        );
    }
    c.redo().unwrap();
    c.redo().unwrap();
    assert_eq!(c.document().entities.len(), 4);
}

#[test]
fn trim_selected_curves_cut_each_other_and_recompute_after_each_pick() {
    let mut c = controller();
    let horizontal = draw_line(&mut c, [-10., 0., 0.], [10., 0., 0.]);
    let vertical = draw_line(&mut c, [0., -10., 0.], [0., 10., 0.]);
    c.select_objects(&[horizontal, vertical]).unwrap();
    let undo_before = c.coordinator().undo_len();
    c.submit("Trim").unwrap();
    assert!(
        c.top11_trim_many_ready(),
        "selected curves should cut one another without a separate cutter stage"
    );
    c.top11_trim_many_accept_pick(horizontal, [5., 0., 0.])
        .unwrap();
    c.top11_trim_many_accept_pick(vertical, [0., 5., 0.])
        .unwrap();
    c.submit("").unwrap();
    for id in [horizontal, vertical] {
        let Geometry::Curve(curve) = &c.document().entities[&id].geometry else {
            panic!("curve")
        };
        assert!(
            (om9_core::curve_ops::arc_length(curve, &Limits::default()).unwrap() - 10.).abs()
                < 1e-7
        );
    }
    assert_eq!(c.coordinator().undo_len(), undo_before + 2);
    c.undo().unwrap();
    c.undo().unwrap();
    for id in [horizontal, vertical] {
        let Geometry::Curve(curve) = &c.document().entities[&id].geometry else {
            panic!("curve")
        };
        assert!(
            (om9_core::curve_ops::arc_length(curve, &Limits::default()).unwrap() - 20.).abs()
                < 1e-7
        );
    }
    c.redo().unwrap();
    c.redo().unwrap();
}

#[test]
fn trim_mutual_removes_a_residual_bounded_only_at_an_endpoint() {
    let mut c = controller();
    let horizontal = draw_line(&mut c, [-10., 0., 0.], [10., 0., 0.]);
    let vertical = draw_line(&mut c, [0., -10., 0.], [0., 10., 0.]);
    c.select_objects(&[horizontal, vertical]).unwrap();
    let undo_before = c.coordinator().undo_len();
    c.submit("Trim").unwrap();
    c.top11_trim_many_accept_pick(horizontal, [5., 0., 0.])
        .unwrap();
    assert!(c.top11_trim_many_has_target(horizontal));
    c.top11_trim_many_accept_pick(horizontal, [-5., 0., 0.])
        .unwrap();
    assert!(!c.document().entities.contains_key(&horizontal));
    assert!(c.document().entities.contains_key(&vertical));
    c.submit("").unwrap();
    assert_eq!(c.coordinator().undo_len(), undo_before + 2);
    c.undo().unwrap();
    let Geometry::Curve(curve) = &c.document().entities[&horizontal].geometry else {
        panic!("curve")
    };
    assert!(
        (om9_core::curve_ops::arc_length(curve, &Limits::default()).unwrap() - 10.).abs() < 1e-7
    );
    c.undo().unwrap();
    let Geometry::Curve(curve) = &c.document().entities[&horizontal].geometry else {
        panic!("curve")
    };
    assert!(
        (om9_core::curve_ops::arc_length(curve, &Limits::default()).unwrap() - 20.).abs() < 1e-7
    );
    c.redo().unwrap();
    c.redo().unwrap();
    assert!(!c.document().entities.contains_key(&horizontal));
}

#[test]
fn trim_mutual_only_endpoint_intersection_can_remove_an_open_curve() {
    let mut c = controller();
    let a = draw_line(&mut c, [-10., 0., 0.], [0., 0., 0.]);
    let b = draw_line(&mut c, [0., 0., 0.], [0., 10., 0.]);
    c.select_objects(&[a, b]).unwrap();
    c.submit("Trim").unwrap();
    c.top11_trim_many_accept_pick(a, [-5., 0., 0.]).unwrap();
    assert!(!c.document().entities.contains_key(&a));
    assert!(c.document().entities.contains_key(&b));
    assert!(!c.top11_trim_many_has_target(b));
    c.submit("").unwrap();
    c.undo().unwrap();
    assert!(c.document().entities.contains_key(&a));
}

#[test]
fn trim_mutual_short_line_is_not_treated_as_a_closed_curve() {
    let mut c = controller();
    let short = draw_line(&mut c, [0., 0., 0.], [0.0005, 0., 0.]);
    let cutter = draw_line(&mut c, [0., 0., 0.], [0., 10., 0.]);
    c.select_objects(&[short, cutter]).unwrap();
    c.submit("Trim").unwrap();
    assert!(c.top11_trim_many_has_target(short));
    c.top11_trim_many_accept_pick(short, [0.00025, 0., 0.]).unwrap();
    assert!(!c.document().entities.contains_key(&short));
    c.submit("").unwrap();
    c.undo().unwrap();
    assert!(c.document().entities.contains_key(&short));
}

#[test]
fn trim_closed_circle_uses_cut_at_parameter_seam() {
    use om9_core::geometry::Curve;
    for toolbar in [false,true] {for preselect in [false,true] {for top in [false,true] {
        let circle=Entity::new("circle",Geometry::Curve(Curve::Circle{center:[0.;3],x:[1.,0.,0.],y:[0.,1.,0.],radius:5.,domain:[0.,std::f64::consts::TAU],angle_domain:None}),None);
        let cutter=Entity::new("diameter",Geometry::Curve(Curve::Line{start:[-6.,0.,0.],end:[6.,0.,0.],domain:[0.,1.]}),None);
        let ray=Entity::new("ray",Geometry::Curve(Curve::Line{start:[0.,0.,0.],end:[0.,6.,0.],domain:[0.,1.]}),None);let ids=[circle.id,cutter.id,ray.id];
        let mut doc=Document::with_template_tolerance("seam cutter");doc.entities.insert(ids[0],circle);doc.entities.insert(ids[1],cutter);doc.entities.insert(ids[2],ray);let mut c=Controller::new(doc,Limits::default()).unwrap();
        select(&mut c,if preselect {&ids[..]} else {&ids[..1]});let before=c.document().clone();if toolbar {c.top11_invoke(Top11Action::Trim).unwrap();}else{c.submit("Trim").unwrap();}
        if !preselect {c.top11_accept_object(ids[1]).unwrap();c.top11_accept_object(ids[2]).unwrap();c.submit("").unwrap();}
        let diagonal=5./2_f64.sqrt();let pick=if top {[diagonal,diagonal,0.]} else {[diagonal,-diagonal,0.]};c.top11_trim_many_accept_pick(ids[0],pick).unwrap();c.submit("").unwrap();let after=c.document().clone();
        let curves:Vec<_>=after.entities.iter().filter(|(id,_)|**id!=ids[1]&&**id!=ids[2]).collect();assert_eq!(curves.len(),1);let Geometry::Curve(curve)=&curves[0].1.geometry else{panic!("curve")};
        let ends=om9_core::curve_ops::curve_endpoints(curve,&Limits::default()).unwrap();let other=if top {[0.,5.,0.]}else{[-5.,0.,0.]};assert!(ends.iter().any(|p|(p[0]-5.).abs()<1e-8&&p[1].abs()<1e-8),"{ends:?}");assert!(ends.iter().any(|p|(p[0]-other[0]).abs()<1e-8&&(p[1]-other[1]).abs()<1e-8),"{ends:?}");
        assert!((om9_core::curve_ops::arc_length(curve,&Limits::default()).unwrap()-(if top {7.5}else{5.})*std::f64::consts::PI).abs()<1e-7);
        let path=std::env::temp_dir().join(format!("om9-trim-seam-cut-{}.om9",uuid::Uuid::new_v4()));om9_archive::Archive::save(&path,&after,&Limits::default(),&om9_core::transaction::Cancellation::default()).unwrap();assert_eq!(om9_archive::Archive::open(&path,&Limits::default()).unwrap(),after);std::fs::remove_file(path).unwrap();
        c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }}}
}

#[test]
fn trim_mutual_does_not_delete_a_closed_curve_at_a_single_seam_hit() {
    let mut doc = Document::with_template_tolerance("closed seam intersection");
    let circle = Entity::new(
        "circle",
        Geometry::Curve(om9_core::geometry::Curve::Circle {
            center: [0.; 3],
            x: [1., 0., 0.],
            y: [0., 1., 0.],
            radius: 5.,
            domain: [0., std::f64::consts::TAU],
            angle_domain: None,
        }),
        None,
    );
    let cutter = Entity::new(
        "line",
        Geometry::Curve(om9_core::geometry::Curve::Line {
            start: [5., 0., 0.],
            end: [10., 0., 0.],
            domain: [0., 5.],
        }),
        None,
    );
    let (circle_id, cutter_id) = (circle.id, cutter.id);
    doc.entities.insert(circle_id, circle);
    doc.entities.insert(cutter_id, cutter);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[circle_id, cutter_id]).unwrap();
    c.submit("Trim").unwrap();
    assert!(!c.top11_trim_many_has_target(circle_id));
    assert!(c.top11_trim_many_has_target(cutter_id));
    assert!(
        c.top11_trim_many_accept_pick(circle_id, [0., 5., 0.])
            .is_err()
    );
    assert!(c.document().entities.contains_key(&circle_id));
    c.cancel();
}

#[test]
fn trim_with_one_preselected_curve_waits_for_a_second() {
    let mut c = controller();
    let first = draw_line(&mut c, [-10., 0., 0.], [10., 0., 0.]);
    let second = draw_line(&mut c, [0., -10., 0.], [0., 10., 0.]);
    c.select_objects(&[first]).unwrap();
    c.submit("Trim").unwrap();
    assert!(c.top11_needs_object());
    assert!(c.submit("").is_err());
    assert!(c.top11_needs_object());
    c.top11_accept_object(second).unwrap();
    c.submit("").unwrap();
    assert!(c.top11_trim_many_ready());
    c.top11_trim_many_accept_pick(second, [0., 5., 0.]).unwrap();
    c.submit("").unwrap();
    let Geometry::Curve(curve) = &c.document().entities[&second].geometry else {
        panic!("curve")
    };
    let ends = om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap();
    assert!((ends[0][1] + 10.).abs() < 1e-7 && ends[1][1].abs() < 1e-7);
    c.undo().unwrap();
    assert!(c.document().entities.contains_key(&first));
}

#[test]
fn trim_mutual_allows_trimming_a_former_cutting_curve_after_split() {
    let mut c = controller();
    let horizontal = draw_line(&mut c, [0., 0., 0.], [100., 0., 0.]);
    let cutters: Vec<_> = [25., 50., 75.]
        .into_iter()
        .map(|x| draw_line(&mut c, [x, -2., 0.], [x, 2., 0.]))
        .collect();
    let mut selected = vec![horizontal];
    selected.extend_from_slice(&cutters);
    c.select_objects(&selected).unwrap();
    let undo_before = c.coordinator().undo_len();
    c.submit("Trim").unwrap();
    c.top11_trim_many_accept_pick(horizontal, [37., 0., 0.])
        .unwrap();
    assert!(!c.document().entities.contains_key(&horizontal));
    c.top11_trim_many_accept_pick(cutters[2], [75., 1., 0.])
        .unwrap();
    c.submit("").unwrap();
    let Geometry::Curve(curve) = &c.document().entities[&cutters[2]].geometry else {
        panic!("curve")
    };
    let ends = om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap();
    assert!(
        (ends[0][1] + 2.).abs() < 1e-7 && ends[1][1].abs() < 1e-7,
        "{ends:?}"
    );
    assert_eq!(c.coordinator().undo_len(), undo_before + 2);
    c.undo().unwrap();
    c.undo().unwrap();
    assert!(c.document().entities.contains_key(&horizontal));
    c.redo().unwrap();
    c.redo().unwrap();
}

#[test]
fn trim_many_can_remove_multiple_portions_of_one_original_curve() {
    let mut c = controller();
    let target = draw_line(&mut c, [0., 0., 0.], [100., 0., 0.]);
    let untouched = draw_line(&mut c, [0., 20., 0.], [100., 20., 0.]);
    let cutters: Vec<_> = [25., 50., 75.]
        .into_iter()
        .map(|x| draw_line(&mut c, [x, -2., 0.], [x, 2., 0.]))
        .collect();
    let mut selected = vec![target, untouched];
    selected.extend_from_slice(&cutters);
    c.select_objects(&selected).unwrap();
    let before = c.coordinator().undo_len();
    c.submit("Trim").unwrap();
    c.submit("W37,0,0").unwrap();
    c.submit("W90,0,0").unwrap();
    c.submit("").unwrap();
    let mut spans: Vec<(f64, f64)> = c
        .document()
        .entities
        .values()
        .filter_map(|entity| {
            if entity.id == untouched || cutters.contains(&entity.id) {
                return None;
            }
            let Geometry::Curve(curve) = &entity.geometry else {
                panic!("curve")
            };
            let ends = om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap();
            Some((ends[0][0], ends[1][0]))
        })
        .collect();
    spans.sort_by(|a, b| a.0.total_cmp(&b.0));
    assert_eq!(spans.len(), 2, "{spans:?}");
    assert!(
        spans
            .iter()
            .zip([(0., 25.), (50., 75.)])
            .all(|(a, b)| (a.0 - b.0).abs() < 1e-6 && (a.1 - b.1).abs() < 1e-6),
        "{spans:?}"
    );
    assert_eq!(c.coordinator().undo_len(), before + 2);
    c.undo().unwrap();
    c.undo().unwrap();
    assert!(c.document().entities.contains_key(&target));
    c.redo().unwrap();
    c.redo().unwrap();
    assert_eq!(c.document().entities.len(), 6);
}

#[test]
fn trim_multiple_targets_supports_selection_after_cmd_and_cancel() {
    let mut c = controller();
    let first = draw_line(&mut c, [-10., 0., 0.], [10., 0., 0.]);
    let second = draw_line(&mut c, [-10., 4., 0.], [10., 4., 0.]);
    let left = draw_line(&mut c, [0., -1., 0.], [0., 1., 0.]);
    let right = draw_line(&mut c, [2., 3., 0.], [2., 5., 0.]);
    c.select_objects(&[]).unwrap();
    c.submit("Trim").unwrap();
    c.top11_accept_object(first).unwrap();
    c.top11_accept_object(second).unwrap();
    c.top11_accept_object(left).unwrap();
    c.top11_accept_object(right).unwrap();
    c.top11_remove_object(right).unwrap();
    c.top11_accept_object(right).unwrap();
    c.submit("").unwrap();
    c.submit("W5,0,0").unwrap();
    c.cancel();
    let Geometry::Curve(first_curve) = &c.document().entities[&first].geometry else {
        panic!("curve")
    };
    let Geometry::Curve(second_curve) = &c.document().entities[&second].geometry else {
        panic!("curve")
    };
    assert!(
        (om9_core::curve_ops::arc_length(first_curve, &Limits::default()).unwrap() - 10.).abs()
            < 1e-7
    );
    assert!(
        (om9_core::curve_ops::arc_length(second_curve, &Limits::default()).unwrap() - 20.).abs()
            < 1e-7
    );
    c.undo().unwrap();
    let Geometry::Curve(first_curve) = &c.document().entities[&first].geometry else {
        panic!("curve")
    };
    assert!(
        (om9_core::curve_ops::arc_length(first_curve, &Limits::default()).unwrap() - 20.).abs()
            < 1e-7
    );
}

#[test]
fn trim_many_viewport_hit_uses_explicit_target_even_if_another_is_closer() {
    let mut c = controller();
    let first = draw_line(&mut c, [-10., 0., 0.], [10., 0., 0.]);
    let second = draw_line(&mut c, [-10., 0.5, 0.], [10., 0.5, 0.]);
    let cutter = draw_line(&mut c, [0., -1., 0.], [0., 2., 0.]);
    let outsider = draw_line(&mut c, [30., 0., 0.], [40., 0., 0.]);
    c.select_objects(&[first, second, cutter]).unwrap();
    c.submit("Trim").unwrap();
    assert!(
        c.top11_trim_many_accept_pick(outsider, [5., 0., 0.])
            .is_err()
    );
    c.top11_trim_many_accept_pick(first, [5., 0.3, 0.]).unwrap();
    c.submit("").unwrap();
    let Geometry::Curve(first_curve) = &c.document().entities[&first].geometry else {
        panic!("curve")
    };
    let Geometry::Curve(second_curve) = &c.document().entities[&second].geometry else {
        panic!("curve")
    };
    assert!(
        (om9_core::curve_ops::arc_length(first_curve, &Limits::default()).unwrap() - 10.).abs()
            < 1e-7
    );
    assert!(
        (om9_core::curve_ops::arc_length(second_curve, &Limits::default()).unwrap() - 20.).abs()
            < 1e-7
    );
}

#[test]
fn trim_many_rejects_ambiguous_typed_point_but_accepts_explicit_hit() {
    let mut c = controller();
    let first = draw_line(&mut c, [-10., 0., 0.], [10., 0., 0.]);
    let second = draw_line(&mut c, [-10., -10., 0.], [10., 10., 0.]);
    let cutter = draw_line(&mut c, [-5., -12., 0.], [-5., 2., 0.]);
    c.select_objects(&[first, second, cutter]).unwrap();
    c.submit("Trim").unwrap();
    let before = c.document().clone();
    assert!(c.submit("W0,0,0").is_err());
    assert_eq!(c.document(), &before);
    c.top11_trim_many_accept_pick(second, [-2., -2., 0.])
        .unwrap();
    c.submit("").unwrap();
    let Geometry::Curve(first_curve) = &c.document().entities[&first].geometry else {
        panic!("curve")
    };
    assert!(
        (om9_core::curve_ops::arc_length(first_curve, &Limits::default()).unwrap() - 20.).abs()
            < 1e-7
    );
    let mut lengths: Vec<_> = c
        .selected()
        .iter()
        .map(|id| {
            let Geometry::Curve(curve) = &c.document().entities[id].geometry else {
                panic!("curve")
            };
            om9_core::curve_ops::arc_length(curve, &Limits::default()).unwrap()
        })
        .collect();
    lengths.sort_by(f64::total_cmp);
    assert_eq!(lengths.len(), 2);
    assert!((lengths[0] - 5. * 2.0f64.sqrt()).abs() < 1e-7);
    assert!((lengths[1] - 10. * 2.0f64.sqrt()).abs() < 1e-7);
}

#[test]
fn split_point_keeps_distinct_closed_curve_pick_near_seam() {
    use om9_core::geometry::Curve;
    let radius=1_000_000.;let angle:f64=-0.00000005;
    let pick=[radius*angle.cos(),radius*angle.sin(),0.];
    let curve=Curve::Circle{center:[0.,0.,0.],x:[1.,0.,0.],y:[0.,1.,0.],radius,domain:[0.,std::f64::consts::TAU],angle_domain:None};
    let entity=Entity::new("large circle",Geometry::Curve(curve),None);let id=entity.id;let mut doc=Document::with_template_tolerance("closed seam pick");doc.entities.insert(id,entity);
    let mut c=Controller::new(doc,Limits::default()).unwrap();select(&mut c,&[id]);let before=c.document().clone();c.submit("Split Point").unwrap();
    c.submit(&format!("W{:.17},{:.17},0",pick[0],pick[1])).unwrap();c.submit("W0,1000000,0").unwrap();c.submit("").unwrap();
    let after=c.document().clone();assert_eq!(after.entities.len(),2);
    for entity in after.entities.values() {let Geometry::Curve(curve)=&entity.geometry else {panic!("curve")};let ends=om9_core::curve_ops::curve_endpoints(curve,&Limits::default()).unwrap();
        assert!(ends.iter().any(|p|((p[0]-pick[0]).hypot(p[1]-pick[1]))<0.001),"{ends:?} vs {pick:?}");}
    c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
}

#[test]
fn split_point_keeps_near_joint_cuts_on_long_polycurve_segments() {
    use om9_core::geometry::Curve;
    let target=100_000_000_000.-0.05;
    let curve=Curve::Polycurve{segments:vec![
        Curve::Line{start:[0.,0.,0.],end:[100_000_000_000.,0.,0.],domain:[0.,1.]},
        Curve::Line{start:[100_000_000_000.,0.,0.],end:[100_000_000_001.,0.,0.],domain:[0.,1.]},
    ],reversed:false};
    let entity=Entity::new("long polycurve",Geometry::Curve(curve),None);let id=entity.id;let mut doc=Document::with_template_tolerance("near joint");doc.entities.insert(id,entity);
    let mut c=Controller::new(doc,Limits::default()).unwrap();select(&mut c,&[id]);let before=c.document().clone();c.submit("Split Point").unwrap();c.submit(&format!("W{target},0,0")).unwrap();
    let after=c.document().clone();assert_eq!(after.entities.len(),2);
    let mut ends:Vec<_>=after.entities.values().map(|e|match &e.geometry{Geometry::Curve(curve)=>om9_core::curve_ops::curve_endpoints(curve,&Limits::default()).unwrap(),_=>panic!("curve")}).collect();
    ends.sort_by(|a,b|a[0][0].total_cmp(&b[0][0]));assert!((ends[0][1][0]-target).abs()<0.0001);assert!((ends[1][0][0]-target).abs()<0.0001);
    c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
}

#[test]
fn split_point_accepts_valid_picks_near_long_curve_endpoints() {
    use om9_core::geometry::Curve;
    for toolbar in [false,true] { for x in [0.05,999_999.95] {
        let mut doc=Document::with_template_tolerance("long split");
        let entity=Entity::new("long line",Geometry::Curve(Curve::Line{start:[0.,0.,0.],end:[1_000_000.,0.,0.],domain:[2.,5.]}),None);let id=entity.id;doc.entities.insert(id,entity);
        let mut c=Controller::new(doc,Limits::default()).unwrap();select(&mut c,&[id]);let before=c.document().clone();
        if toolbar {c.top11_invoke(Top11Action::Split).unwrap();c.submit("Point").unwrap();}else{c.submit("Split Point").unwrap();}
        assert!(c.submit("W0,0,0").is_err());assert_eq!(c.document(),&before);
        c.submit(&format!("W{x},0,0")).unwrap();let after=c.document().clone();assert_eq!(after.entities.len(),2);
        let mut endpoints:Vec<_>=after.entities.values().map(|e|match &e.geometry {Geometry::Curve(curve)=>om9_core::curve_ops::curve_endpoints(curve,&Limits::default()).unwrap(),_=>panic!("curve")}).collect();
        endpoints.sort_by(|a,b|a[0][0].total_cmp(&b[0][0]));assert!((endpoints[0][1][0]-x).abs()<1e-8);assert!((endpoints[1][0][0]-x).abs()<1e-8);
        let path=std::env::temp_dir().join(format!("om9-long-split-{}.om9",uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&path,&after,&Limits::default(),&om9_core::transaction::Cancellation::default()).unwrap();assert_eq!(om9_archive::Archive::open(&path,&Limits::default()).unwrap(),after);std::fs::remove_file(path).unwrap();
        c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }}
}

#[test]
fn split_point_line_accepts_world_coordinates_and_undo_redo() {
    let mut doc = Document::with_template_tolerance("split point");
    let mut entity = Entity::new(
        "line",
        Geometry::Curve(om9_core::geometry::Curve::Line {
            start: [0., 0., 0.],
            end: [10., 0., 0.],
            domain: [0., 1.],
        }),
        None,
    );
    entity.placement = om9_core::geometry::Transform::translation([5., 2., 0.]);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Split").unwrap();
    c.submit("Point").unwrap();
    assert!(c.submit("W9,3,0").is_err());
    assert!(c.is_running());
    c.submit("W9,2,0").unwrap();
    assert_eq!(c.document().entities.len(), 2);
    let mut lengths: Vec<f64> = c
        .document()
        .entities
        .values()
        .map(|e| {
            let Geometry::Curve(curve) = &e.geometry else {
                panic!("curve")
            };
            om9_core::curve_ops::arc_length(curve, c.coordinator().limits()).unwrap()
        })
        .collect();
    lengths.sort_by(f64::total_cmp);
    assert!(
        (lengths[0] - 4.).abs() < 1e-8 && (lengths[1] - 6.).abs() < 1e-8,
        "{lengths:?}"
    );
    assert_eq!(c.coordinator().undo_len(), 1);
    c.undo().unwrap();
    assert!(c.document().entities.contains_key(&id));
    c.redo().unwrap();
    assert_eq!(c.document().entities.len(), 2);
}

#[test]
fn split_point_uses_world_distance_for_nonuniform_placement() {
    let mut doc = Document::with_template_tolerance("scaled split");
    let mut line = Entity::new(
        "line",
        Geometry::Curve(om9_core::geometry::Curve::Line {
            start: [0., 0., 0.],
            end: [10., 10., 0.],
            domain: [0., 1.],
        }),
        None,
    );
    line.placement = Transform::scale([10., 1., 1.]).unwrap();
    let id = line.id;
    doc.entities.insert(id, line);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Split Point").unwrap();
    let normal = [-10. / 10100.0f64.sqrt(), 100. / 10100.0f64.sqrt(), 0.];
    let pick = |d: f64| [50. + d * normal[0], 5. + d * normal[1], 0.];
    assert!(c.accept_world_point(pick(0.002)).is_err());
    assert!(c.is_running());
    c.accept_world_point(pick(0.0008)).unwrap();
    assert_eq!(c.document().entities.len(), 2);
    for e in c.document().entities.values() {
        let Geometry::Curve(curve) = &e.geometry else {
            panic!("curve")
        };
        let ends = om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap();
        assert!(
            ends.iter().any(|p| {
                let p = e.placement.point(*p).unwrap();
                (p[0] - 50.).abs() < 1e-7 && (p[1] - 5.).abs() < 1e-7
            }),
            "{ends:?}"
        );
    }
    c.undo().unwrap();
    assert!(c.document().entities.contains_key(&id));
    c.redo().unwrap();
    assert_eq!(c.document().entities.len(), 2);
}

#[test]
fn split_point_nonuniform_circle_preserves_original_angular_parameters() {
    let mut doc = Document::with_template_tolerance("elliptic split");
    let mut entity = Entity::new(
        "circle",
        Geometry::Curve(om9_core::geometry::Curve::Circle {
            center: [0.; 3],
            x: [1., 0., 0.],
            y: [0., 1., 0.],
            radius: 1.,
            domain: [0., std::f64::consts::TAU],
            angle_domain: None,
        }),
        None,
    );
    entity.placement = Transform::scale([10., 1., 1.]).unwrap();
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Split Point").unwrap();
    let q = std::f64::consts::FRAC_1_SQRT_2;
    let normal = [0.1 / 1.01f64.sqrt(), 1. / 1.01f64.sqrt()];
    c.accept_world_point([10. * q + 0.0008 * normal[0], q + 0.0008 * normal[1], 0.])
        .unwrap();
    c.accept_world_point([-10. * q, -q, 0.]).unwrap();
    c.submit("").unwrap();
    assert_eq!(c.document().entities.len(), 2);
    for entity in c.document().entities.values() {
        let Geometry::Curve(curve) = &entity.geometry else {
            panic!("curve")
        };
        let ends = om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap();
        for p in ends {
            let p = entity.placement.point(p).unwrap();
            assert!(
                (p[0].abs() - 10. * q).abs() < 1e-5 && (p[1].abs() - q).abs() < 1e-5,
                "{p:?}"
            );
        }
    }
    c.undo().unwrap();
    assert!(c.document().entities.contains_key(&id));
}

#[test]
fn split_point_finds_a_segment_between_global_samples() {
    let mut doc = Document::with_template_tolerance("many-segment split");
    let mut points: Vec<[f64; 3]> = (0..=128).map(|i| [i as f64, 0., 0.]).collect();
    points[65] = [0., -100., 0.];
    let segments = points
        .windows(2)
        .map(|p| om9_core::geometry::Curve::Line {
            start: p[0],
            end: p[1],
            domain: [0., 1.],
        })
        .collect();
    let entity = Entity::new(
        "polycurve",
        Geometry::Curve(om9_core::geometry::Curve::Polycurve {
            segments,
            reversed: false,
        }),
        None,
    );
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Split Point").unwrap();
    c.submit("W0,-100,0").unwrap();
    assert_eq!(c.document().entities.len(), 2);
    for entity in c.document().entities.values() {
        let Geometry::Curve(curve) = &entity.geometry else {
            panic!("curve")
        };
        let ends = om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap();
        assert!(
            ends.iter()
                .any(|p| p[0].abs() < 1e-8 && (p[1] + 100.).abs() < 1e-8),
            "{ends:?}"
        );
    }
    c.undo().unwrap();
    assert!(c.document().entities.contains_key(&id));
    c.redo().unwrap();
    assert_eq!(c.document().entities.len(), 2);
}

#[test]
fn split_point_finds_nurbs_knot_in_a_narrow_span() {
    let mut doc = Document::with_template_tolerance("narrow knot split");
    let mut entity = Entity::new(
        "nurbs",
        Geometry::Curve(om9_core::geometry::Curve::Nurbs(
            om9_core::geometry::NurbsCurve {
                degree: 1,
                poles: vec![
                    [0., 0., 0.],
                    [1., 0., 0.],
                    [500., 0., 0.],
                    [0., -100., 0.],
                    [502., 0., 0.],
                    [503., 0., 0.],
                ],
                weights: vec![1.; 6],
                knots: vec![0., 0., 0.4, 0.9, 0.90001, 0.90002, 1., 1.],
                domain: [0., 1.],
                periodic: false,
                reversed: false,
                seam: None,
            },
        )),
        None,
    );
    entity.placement = Transform::scale([2., 0.5, 1.]).unwrap();
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Split Point").unwrap();
    c.submit("W0,-50,0").unwrap();
    assert_eq!(c.document().entities.len(), 2);
    for entity in c.document().entities.values() {
        let Geometry::Curve(curve) = &entity.geometry else {
            panic!("curve")
        };
        let ends = om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap();
        assert!(
            ends.iter()
                .any(|p| p[0].abs() < 1e-7 && (p[1] + 100.).abs() < 1e-7),
            "{ends:?}"
        );
    }
    c.undo().unwrap();
    assert!(c.document().entities.contains_key(&id));
    c.redo().unwrap();
    assert_eq!(c.document().entities.len(), 2);
}

#[test]
fn split_point_command_alias_and_cancel_leave_line_intact() {
    let mut c = controller();
    let id = draw_line(&mut c, [0.; 3], [10., 0., 0.]);
    select(&mut c, &[id]);
    c.submit("Split Point").unwrap();
    assert_eq!(c.top11_prompt(), Some("pick a point on the curve"));
    c.cancel();
    assert!(c.document().entities.contains_key(&id));
    assert_eq!(c.document().entities.len(), 1);
}

#[test]
fn split_point_alias_can_select_curve_after_invocation() {
    let mut doc = Document::with_template_tolerance("split selection");
    let entity = Entity::new(
        "line",
        Geometry::Curve(om9_core::geometry::Curve::Line {
            start: [0., 0., 0.],
            end: [10., 0., 0.],
            domain: [0., 10.],
        }),
        None,
    );
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.submit("Split Point").unwrap();
    assert!(c.top11_needs_object());
    assert!(c.submit("").is_err());
    c.top11_accept_object(id).unwrap();
    c.submit("").unwrap();
    c.submit("W4,0,0").unwrap();
    assert_eq!(c.document().entities.len(), 2);
    let mut lengths: Vec<_> = c
        .document()
        .entities
        .values()
        .map(|e| {
            let Geometry::Curve(curve) = &e.geometry else {
                panic!("curve")
            };
            om9_core::curve_ops::arc_length(curve, &Limits::default()).unwrap()
        })
        .collect();
    lengths.sort_by(f64::total_cmp);
    assert!((lengths[0] - 4.).abs() < 1e-8 && (lengths[1] - 6.).abs() < 1e-8);
    c.undo().unwrap();
    assert!(c.document().entities.contains_key(&id));
    c.redo().unwrap();
    assert_eq!(c.document().entities.len(), 2);
}

#[test]
fn split_point_selection_recovers_from_invalid_and_multiple_objects() {
    let mut doc = Document::with_template_tolerance("split selection retry");
    let line = Entity::new(
        "line",
        Geometry::Curve(om9_core::geometry::Curve::Line {
            start: [0., 0., 0.],
            end: [10., 0., 0.],
            domain: [0., 10.],
        }),
        None,
    );
    let point = Entity::new("point", Geometry::Point([0., 1., 0.]), None);
    let (line_id, point_id) = (line.id, point.id);
    doc.entities.insert(line_id, line);
    doc.entities.insert(point_id, point);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    let before = c.document().clone();
    c.submit("Split P").unwrap();
    c.top11_accept_object(point_id).unwrap();
    assert!(c.submit("").is_err());
    assert!(c.top11_needs_object());
    c.top11_accept_object(line_id).unwrap();
    assert!(c.submit("").is_err());
    c.top11_remove_object(point_id).unwrap();
    c.submit("").unwrap();
    assert!(!c.top11_needs_object());
    c.cancel();
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), 0);
}

#[test]
fn split_point_closed_circle_requires_two_points_and_commits_on_enter() {
    let mut doc = Document::with_template_tolerance("split circle");
    let entity = Entity::new(
        "circle",
        Geometry::Curve(om9_core::geometry::Curve::Circle {
            center: [0.; 3],
            x: [1., 0., 0.],
            y: [0., 1., 0.],
            radius: 5.,
            domain: [0., std::f64::consts::TAU],
            angle_domain: None,
        }),
        None,
    );
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Split").unwrap();
    c.submit("Point").unwrap();
    c.submit("W0,5,0").unwrap();
    assert!(c.submit("").is_err());
    assert!(c.submit("W0,5,0").is_err());
    assert!(c.submit("W0,-6,0").is_err());
    assert_eq!(c.coordinator().undo_len(), 0);
    c.submit("W0,-5,0").unwrap();
    c.submit("").unwrap();
    assert_eq!(c.document().entities.len(), 2);
    for entity in c.document().entities.values() {
        let Geometry::Curve(curve) = &entity.geometry else {
            panic!("curve")
        };
        let ends = om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap();
        assert!(
            ends.iter()
                .all(|p| p[0].abs() < 1e-6 && (p[1].abs() - 5.).abs() < 1e-6),
            "{ends:?}"
        );
        assert!(
            (om9_core::curve_ops::arc_length(curve, &Limits::default()).unwrap()
                - 5. * std::f64::consts::PI)
                .abs()
                < 1e-6
        );
    }
    c.undo().unwrap();
    assert!(c.document().entities.contains_key(&id));
    c.redo().unwrap();
    assert_eq!(c.document().entities.len(), 2);
}

#[test]
fn split_point_certified_periodic_nurbs_closed_and_trimmed() {
    use om9_core::{curve_ops,geometry::{Curve,NurbsCurve},transaction::Cancellation};
    let limits=Limits::default();
    let mut n=NurbsCurve{degree:2,poles:vec![[0.,0.,0.],[2.,0.,0.],[2.,2.,0.],[0.,2.,0.],[0.,0.,0.],[2.,0.,0.]],
        weights:vec![1.,0.7,1.2,0.9,1.,0.7],knots:vec![-2.,-1.,0.,1.,2.,3.,4.,5.,6.],
        domain:[0.,4.],periodic:true,reversed:false,seam:None};
    for closed in [false,true] { for reversed in [false,true] { for toolbar in [false,true] { for shifted in [false,true] {
        if shifted && !closed {continue;}
        n.domain=if closed {[0.,4.]} else {[0.3,3.7]};n.reversed=reversed;
        let mut curve=Curve::Nurbs(n.clone());if shifted {curve=curve_ops::set_seam(&curve,0.3,1e-9,&limits).unwrap();}let placement=Transform::translation([3.,-2.,1.]);
        let points=[0.25,0.7].map(|f|placement.point(curve_ops::point_at(&curve,f,&limits).unwrap()).unwrap());
        let mut entity=Entity::new("periodic",Geometry::Curve(curve.clone()),None);entity.placement=placement;let id=entity.id;
        let mut doc=Document::with_template_tolerance("periodic split");doc.entities.insert(id,entity);let mut c=Controller::new(doc,limits).unwrap();
        select(&mut c,&[id]);let before=c.document().clone();
        if toolbar {c.top11_invoke(Top11Action::Split).unwrap();c.submit("Point").unwrap();} else {c.submit("Split Point").unwrap();}
        c.accept_world_point(points[0]).unwrap();
        if closed {assert!(c.submit("").is_err());c.accept_world_point(points[1]).unwrap();c.submit("").unwrap();}
        assert_eq!(c.document().entities.len(),2);
        if closed {for e in c.document().entities.values() {let Geometry::Curve(piece)=&e.geometry else {panic!("curve")};
            for end in curve_ops::curve_endpoints(piece,&limits).unwrap() {let world=e.placement.point(end).unwrap();
                assert!(points.iter().any(|p|p.iter().zip(world).all(|(a,b)|(a-b).abs()<1e-5)),"{world:?} {points:?}");}}}
        let total:f64=c.document().entities.values().map(|e| {let Geometry::Curve(curve)=&e.geometry else {panic!("curve")};curve_ops::arc_length(curve,&limits).unwrap()}).sum();
        let expected=curve_ops::arc_length(&curve,&limits).unwrap();assert!((total-expected).abs()<1e-5,"{total} {expected}");
        let after=c.document().clone();let path=std::env::temp_dir().join(format!("om9-split-periodic-{}.om9",uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&path,&after,&limits,&Cancellation::default()).unwrap();assert_eq!(om9_archive::Archive::open(&path,&limits).unwrap(),after);std::fs::remove_file(path).unwrap();
        c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }}}}
}

#[test]
fn split_point_reseamed_closed_nonperiodic_nurbs() {
    use om9_core::{curve_ops,geometry::Curve};
    let limits=Limits::default();
    for toolbar in [false,true] {for reversed in [false,true] {
        let mut n=om9_core::curve_build::ellipse([0.;3],[1.,0.,0.],[0.,1.,0.],4.,2.,&limits).unwrap();n.reversed=reversed;
        let curve=curve_ops::set_seam(&Curve::Nurbs(n),0.31,1e-9,&limits).unwrap();
        let picks=[0.2,0.7].map(|f|curve_ops::point_at(&curve,f,&limits).unwrap());
        let entity=Entity::new("ellipse",Geometry::Curve(curve.clone()),None);let id=entity.id;
        let mut doc=Document::with_template_tolerance("shifted seam split");doc.entities.insert(id,entity);
        let mut c=Controller::new(doc,limits).unwrap();select(&mut c,&[id]);let before=c.document().clone();
        if toolbar {c.top11_invoke(Top11Action::Split).unwrap();c.submit("Point").unwrap();} else {c.submit("Split Point").unwrap();}
        for p in picks {c.accept_world_point(p).unwrap();}c.submit("").unwrap();
        assert_eq!(c.document().entities.len(),2);
        for e in c.document().entities.values() {let Geometry::Curve(piece)=&e.geometry else {panic!("curve")};
            for end in curve_ops::curve_endpoints(piece,&limits).unwrap() {assert!(picks.iter().any(|p|p.iter().zip(end).all(|(a,b)|(a-b).abs()<1e-5)));}}
        let after=c.document().clone();c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }}
}

#[test]
fn split_point_on_translated_arc_uses_world_coordinates() {
    let mut doc = Document::with_template_tolerance("arc split");
    let mut entity = Entity::new(
        "arc",
        Geometry::Curve(om9_core::geometry::Curve::Circle {
            center: [0.; 3],
            x: [1., 0., 0.],
            y: [0., 1., 0.],
            radius: 5.,
            domain: [0., 1.],
            angle_domain: Some([0., std::f64::consts::PI]),
        }),
        None,
    );
    entity.placement = Transform::translation([10., 20., 0.]);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Split Point").unwrap();
    c.submit("W10,25,0").unwrap();
    assert_eq!(c.document().entities.len(), 2);
    for entity in c.document().entities.values() {
        let Geometry::Curve(curve) = &entity.geometry else {
            panic!("curve")
        };
        assert!(
            (om9_core::curve_ops::arc_length(curve, &Limits::default()).unwrap()
                - 2.5 * std::f64::consts::PI)
                .abs()
                < 1e-6
        );
    }
    c.undo().unwrap();
    assert!(c.document().entities.contains_key(&id));
}

#[test]
fn split_closed_circle_accepts_three_points_including_seam() {
    let mut doc = Document::with_template_tolerance("circle split");
    let circle = Entity::new(
        "circle",
        Geometry::Curve(om9_core::geometry::Curve::Circle {
            center: [0.; 3],
            x: [1., 0., 0.],
            y: [0., 1., 0.],
            radius: 5.,
            domain: [0., std::f64::consts::TAU],
            angle_domain: None,
        }),
        None,
    );
    let id = circle.id;
    doc.entities.insert(id, circle);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.top11_invoke(Top11Action::Split).unwrap();
    c.submit("Point").unwrap();
    for point in [[5., 0., 0.], [0., 5., 0.], [-5., 0., 0.]] {
        c.accept_world_point(point).unwrap();
    }
    c.submit("").unwrap();
    assert_eq!(c.document().entities.len(), 3);
    let mut lengths: Vec<_> = c
        .document()
        .entities
        .values()
        .map(|e| {
            let Geometry::Curve(curve) = &e.geometry else {
                panic!("curve")
            };
            om9_core::curve_ops::arc_length(curve, &Limits::default()).unwrap()
        })
        .collect();
    lengths.sort_by(f64::total_cmp);
    for (actual, expected) in lengths.iter().zip([
        2.5 * std::f64::consts::PI,
        2.5 * std::f64::consts::PI,
        5. * std::f64::consts::PI,
    ]) {
        assert!((actual - expected).abs() < 1e-6, "{lengths:?}");
    }
}

#[test]
fn split_point_closed_polycurve_splits_at_both_picks() {
    let mut doc = Document::with_template_tolerance("closed polycurve");
    let points = [
        [0., 0., 0.],
        [10., 0., 0.],
        [10., 4., 0.],
        [0., 4., 0.],
        [0., 0., 0.],
    ];
    let segments = points
        .windows(2)
        .map(|p| om9_core::geometry::Curve::Line {
            start: p[0],
            end: p[1],
            domain: [0., 1.],
        })
        .collect();
    let entity = Entity::new(
        "rectangle",
        Geometry::Curve(om9_core::geometry::Curve::Polycurve {
            segments,
            reversed: false,
        }),
        None,
    );
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Split Point").unwrap();
    c.accept_world_point([5., 0., 0.]).unwrap();
    c.accept_world_point([5., 4., 0.]).unwrap();
    c.submit("").unwrap();
    assert_eq!(c.document().entities.len(), 2);
    for entity in c.document().entities.values() {
        let Geometry::Curve(curve) = &entity.geometry else {
            panic!("curve")
        };
        let ends = om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap();
        assert!(
            ends.iter().all(
                |p| (p[0] - 5.).abs() < 1e-6 && (p[1].abs() < 1e-6 || (p[1] - 4.).abs() < 1e-6)
            ),
            "{ends:?}"
        );
        assert!(
            (om9_core::curve_ops::arc_length(curve, &Limits::default()).unwrap() - 14.).abs()
                < 1e-6
        );
    }
    c.undo().unwrap();
    assert!(c.document().entities.contains_key(&id));
}

#[test]
fn m03_03_08_006_toolbar_has_eleven_actions_and_ring_rail_opens_the_builder() {
    let mut c = controller();
    let states = c.top11_availability();
    assert_eq!(states.len(), 11);
    let ring = states
        .iter()
        .find(|state| state.action == Top11Action::RingRail)
        .unwrap();
    // M04 owns the builder now; with no command running the row is available.
    assert!(ring.enabled);
    assert!(c.top11_invoke(Top11Action::Duplicate).is_ok());
    assert!(c.top11_needs_object());
    c.cancel();
    let result = c.top11_invoke(Top11Action::RingRail).unwrap();
    assert!(result.message().contains("Ring Rail"));
    assert!(c.builder().is_some());
    let running = c.top11_availability();
    for state in &running {
        assert!(!state.enabled, "an open builder must disable the toolbar");
    }
    assert!(c.top11_invoke(Top11Action::RingRail).is_err());
    c.builder_cancel();
    // The line command path still works after the builder closes.
    let id = draw_line(&mut c, [0.0; 3], [10.0, 0.0, 0.0]);
    select(&mut c, &[id]);
    c.submit(LINE).unwrap();
    for state in c.top11_availability() {
        assert!(!state.enabled, "running command must disable all buttons");
    }
    assert!(c.top11_invoke(Top11Action::Duplicate).is_err());
    c.cancel();
    println!("top11_buttons={}", states.len());
    println!("ringrail_enabled={}", ring.enabled);
}

#[test]
fn ringrail_scene_budget_refusal_is_atomic() {
    use om9_core::model::Definition;
    let mut doc=Document::with_template_tolerance("ringrail scene budget");let leaf=Definition::new("leaf",(0..8).map(|i|Entity::new("point",Geometry::Point([i as f64,0.,0.]),None)).collect());let mut def_id=leaf.id;doc.definitions.insert(def_id,leaf);
    for _ in 0..3 {let def=Definition::new("nested",(0..8).map(|_|Entity::new("child",Geometry::Instance(def_id),None)).collect());def_id=def.id;doc.definitions.insert(def_id,def);}
    let root=Entity::new("root",Geometry::Instance(def_id),None);doc.entities.insert(root.id,root);
    let mut c=Controller::new(doc,Limits{max_bytes:3_000_000,..Limits::default()}).unwrap();let before=c.document().clone();let scene_count=c.scene().unwrap().objects.len();
    c.top11_invoke(Top11Action::RingRail).unwrap();let result=c.builder_apply();
    assert!(result.is_err(),"limited scene budget must refuse this fixture");
    assert_eq!(c.document(),&before,"failed builder must not commit");assert_eq!(c.scene().unwrap().objects.len(),scene_count);assert_eq!(c.coordinator().undo_len(),0);assert!(c.builder().is_some());
}

#[test]
fn ringrail_scene_budget_boundaries_never_report_error_after_commit() {
    use om9_core::model::Definition;
    let mut doc=Document::with_template_tolerance("ringrail scene boundaries");let leaf=Definition::new("leaf",(0..8).map(|i|Entity::new("point",Geometry::Point([i as f64,0.,0.]),None)).collect());let mut def_id=leaf.id;doc.definitions.insert(def_id,leaf);
    for _ in 0..3 {let def=Definition::new("nested",(0..8).map(|_|Entity::new("child",Geometry::Instance(def_id),None)).collect());def_id=def.id;doc.definitions.insert(def_id,def);}
    let root=Entity::new("root",Geometry::Instance(def_id),None);doc.entities.insert(root.id,root);
    let mut refused=0;let mut committed=0;
    for budget in [3_000_000,3_500_000,4_000_000,5_000_000,6_000_000,8_000_000,12_000_000] {
        let mut c=Controller::new(doc.clone(),Limits{max_bytes:budget,..Limits::default()}).unwrap();let before=c.document().clone();let before_count=c.scene().unwrap().objects.len();
        c.top11_invoke(Top11Action::RingRail).unwrap();let result=c.builder_apply();
        if result.is_err() {refused+=1;assert_eq!(c.document(),&before,"post-commit error at {budget}: {result:?}");assert_eq!(c.scene().unwrap().objects.len(),before_count);assert_eq!(c.coordinator().undo_len(),0);}
        else {committed+=1;assert_eq!(c.document().entities.len(),before.entities.len()+1);assert!(c.scene().unwrap().objects.len()>before_count);}
    }
    assert!(refused>0&&committed>0,"test must cover refusal and commit");
}

#[test]
fn ringrail_invalid_diameter_and_cancel_do_not_modify_document() {
    for toolbar in [false,true] {
        let mut c=controller();let before=c.document().clone();
        if toolbar {c.top11_invoke(Top11Action::RingRail).unwrap();}else{c.submit("gvRingRail").unwrap();}
        assert!(c.submit("Diameter=0").is_err());assert_eq!(c.document(),&before);assert_eq!(c.coordinator().undo_len(),0);
        assert!(c.builder().is_some());c.builder_cancel();assert_eq!(c.document(),&before);assert_eq!(c.coordinator().undo_len(),0);
    }
}

#[test]
fn ringrail_toolbar_and_command_paths_create_same_undoable_geometry() {
    use om9_core::geometry::Curve;
    for route in ["toolbar","gvRingRail","RingRail","Top11 RingRail"] {
        let mut c=controller();let before=c.document().clone();
        if route=="toolbar" {c.top11_invoke(Top11Action::RingRail).unwrap();}else{c.submit(route).unwrap();}
        assert!(c.builder().is_some(),"{route}");assert!(c.top11_availability().iter().all(|state|!state.enabled));
        c.submit("Diameter=20").unwrap();c.submit("").unwrap();
        assert!(c.builder().is_none());let after=c.document().clone();assert_eq!(after.entities.len(),1);assert_eq!(after.history.recipes.len(),1);
        let entity=after.entities.values().next().unwrap();let Geometry::Curve(Curve::Circle{center,radius,..})=&entity.geometry else {panic!("ring rail circle")};
        assert_eq!(*center,[0.,0.,0.]);assert_eq!(*radius,10.);assert_eq!(after.history.recipes.values().next().unwrap().command,"RingRail");
        let path=std::env::temp_dir().join(format!("om9-top11-ringrail-{}.om9",uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&path,&after,&Limits::default(),&om9_core::transaction::Cancellation::default()).unwrap();
        assert_eq!(om9_archive::Archive::open(&path,&Limits::default()).unwrap(),after);std::fs::remove_file(path).unwrap();
        c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }
}

#[test]
fn m03_03_08_001_duplicate_is_in_place_one_undo_and_selects_the_copy() {
    let mut c = controller();
    let id = draw_line(&mut c, [0.0; 3], [10.0, 0.0, 0.0]);
    select(&mut c, &[id]);
    let before = c.document().entities[&id].geometry.clone();
    let result = c.top11_duplicate_in_place().unwrap();
    assert!(result.message().contains("Duplicate"));
    assert_eq!(c.document().entities.len(), 2);
    assert_eq!(c.selected().len(), 1);
    let copy_id = *c.selected().iter().next().unwrap();
    assert_ne!(copy_id, id);
    assert_eq!(c.document().entities[&copy_id].geometry, before);
    assert_eq!(c.coordinator().undo_len(), 2);
    c.undo().unwrap();
    assert_eq!(c.document().entities.len(), 1);
    c.redo().unwrap();
    assert_eq!(c.document().entities.len(), 2);
    println!("duplicate_entities={}", c.document().entities.len());
    println!("duplicate_undo_redo_passed=true");
}

#[test]
fn m03_03_08_001_duplicate_base_point_places_multiple_copies_until_enter() {
    let mut c = controller();
    let id = draw_line(&mut c, [0.0; 3], [1.0, 0.0, 0.0]);
    select(&mut c, &[id]);
    c.submit("Copy").unwrap();
    assert_eq!(c.top11_prompt(), Some("pick the base point"));
    c.accept_world_point([0.0; 3]).unwrap();
    c.accept_world_point([5.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([10.0, 0.0, 0.0]).unwrap();
    assert_eq!(c.document().entities.len(), 3);
    assert!(c.is_running());
    c.submit("").unwrap();
    assert!(!c.is_running());
    let placements: Vec<f64> = c
        .document()
        .entities
        .values()
        .map(|entity| entity.placement.point([0.0; 3]).unwrap()[0])
        .collect();
    assert!(placements.contains(&0.0));
    assert!(placements.contains(&5.0));
    assert!(placements.contains(&10.0));
    println!("duplicate_base_copies=2");
    println!("duplicate_total_objects={}", c.document().entities.len());
}

#[test]
fn m03_03_08_003_move_and_rotate_collect_points_and_commit_placements() {
    let mut c = controller();
    let id = draw_line(&mut c, [0.0; 3], [10.0, 0.0, 0.0]);
    select(&mut c, &[id]);
    c.submit("Move").unwrap();
    assert!(c.top11_pending().is_some());
    c.accept_world_point([0.0; 3]).unwrap();
    c.accept_world_point([5.0, -2.0, 0.0]).unwrap();
    assert!(!c.is_running());
    let placement = c.document().entities[&id].placement;
    assert_eq!(placement.point([0.0; 3]).unwrap(), [5.0, -2.0, 0.0]);
    c.undo().unwrap();
    select(&mut c, &[id]);
    c.submit("Rotate").unwrap();
    c.accept_world_point([0.0; 3]).unwrap();
    c.accept_world_point([10.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([0.0, 10.0, 0.0]).unwrap();
    let placement = c.document().entities[&id].placement;
    let end = placement.point([10.0, 0.0, 0.0]).unwrap();
    assert!(
        end[0].abs() < 1e-6 && (end[1] - 10.0).abs() < 1e-6,
        "{end:?}"
    );
    assert_eq!(c.coordinator().undo_len(), 2);
    println!("move_base_mm=[5.0, -2.0, 0.0]");
    println!("rotate_end_mm={end:?}");
}

#[test]
fn m03_03_08_003_mirror_collects_a_plane_and_creates_a_copy() {
    let mut c = controller();
    let id = draw_line(&mut c, [1.0, 0.0, 0.0], [4.0, 0.0, 0.0]);
    select(&mut c, &[id]);
    c.submit("Mirror").unwrap();
    c.accept_world_point([0.0, -5.0, 0.0]).unwrap();
    c.accept_world_point([0.0, 5.0, 0.0]).unwrap();
    assert_eq!(c.document().entities.len(), 2);
    let copy = c
        .document()
        .entities
        .values()
        .find(|entity| entity.id != id)
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Line { start, end, .. }) = &copy.geometry else {
        panic!("mirror did not copy a line");
    };
    assert!((start[0] + 1.0).abs() < 1e-9, "{start:?}");
    assert!((end[0] + 4.0).abs() < 1e-9, "{end:?}");
    println!("mirror_copies=2");
    println!("mirrored_start_x_mm={:.12}", start[0]);
}

#[test]
fn m03_03_08_004_join_explode_split_and_trim_run_from_the_toolbar() {
    let mut c = controller();
    let first = draw_line(&mut c, [0.0; 3], [10.0, 0.0, 0.0]);
    let second = draw_line(&mut c, [10.0, 0.0, 0.0], [10.0, 10.0, 0.0]);
    select(&mut c, &[first, second]);
    c.top11_invoke(Top11Action::Join).unwrap();
    assert_eq!(c.document().entities.len(), 1);
    let joined = *c.document().entities.keys().next().unwrap();
    assert!(matches!(
        c.document().entities[&joined].geometry,
        Geometry::Curve(om9_core::geometry::Curve::Polycurve { .. })
    ));
    select(&mut c, &[joined]);
    c.top11_invoke(Top11Action::Explode).unwrap();
    assert_eq!(c.document().entities.len(), 2);

    let mut split = controller();
    let horizontal = draw_line(&mut split, [-10.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let vertical = draw_line(&mut split, [0.0, -10.0, 0.0], [0.0, 10.0, 0.0]);
    select(&mut split, &[horizontal]);
    split.top11_invoke(Top11Action::Split).unwrap();
    assert_eq!(
        split.top11_prompt(),
        Some("pick cutting curves (Enter to execute)")
    );
    split.top11_accept_object(vertical).unwrap();
    split.submit("").unwrap();
    assert!(!split.is_running());
    assert_eq!(split.document().entities.len(), 3);
    assert_eq!(split.selected().len(), 2);

    let mut trim = controller();
    let horizontal = draw_line(&mut trim, [-10.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let vertical = draw_line(&mut trim, [0.0, -10.0, 0.0], [0.0, 10.0, 0.0]);
    select(&mut trim, &[horizontal]);
    trim.top11_invoke(Top11Action::Trim).unwrap();
    trim.top11_accept_object(vertical).unwrap();
    trim.submit("").unwrap();
    assert_eq!(
        trim.top11_prompt(),
        Some("pick portions to remove (Enter to finish)")
    );
    trim.accept_world_point([-5.0, 0.0, 0.0]).unwrap();
    assert_eq!(trim.document().entities.len(), 2);
    let remainder = trim
        .document()
        .entities
        .values()
        .find(|entity| entity.id != vertical)
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Line { start, .. }) = &remainder.geometry else {
        panic!("trim did not leave a line");
    };
    assert!((start[0] - 0.0).abs() < 1e-9, "{start:?}");
    println!("joined_entities=1");
    println!("exploded_entities=2");
    println!("split_entities=3");
    println!("trim_remaining_entities=2");
    println!("trim_kept_start_x_mm={:.12}", start[0]);
}

#[test]
fn m03_03_08_003_mirror_delete_input_reflects_the_original_in_place() {
    let mut c = controller();
    let id = draw_line(&mut c, [1.0, 0.0, 0.0], [4.0, 0.0, 0.0]);
    select(&mut c, &[id]);
    c.submit("Mirror Delete").unwrap();
    c.accept_world_point([0.0, -5.0, 0.0]).unwrap();
    c.accept_world_point([0.0, 5.0, 0.0]).unwrap();
    assert_eq!(c.document().entities.len(), 1);
    let Geometry::Curve(om9_core::geometry::Curve::Line { start, end, .. }) =
        &c.document().entities[&id].geometry
    else {
        panic!("mirror delete lost the line");
    };
    assert!((start[0] + 1.0).abs() < 1e-9, "{start:?}");
    assert!((end[0] + 4.0).abs() < 1e-9, "{end:?}");
    println!("mirror_delete_entities=1");
    println!("mirror_delete_start_x_mm={:.12}", start[0]);
}

#[test]
fn periodic_nurbs_edit_and_control_points_support_command_and_toolbar() {
    use om9_core::{curve_ops,geometry::{Curve,NurbsCurve}};
    let limits=Limits::default();
    for edit in [false,true] {for toolbar in [false,true] {for reversed in [false,true] {
        let n=NurbsCurve{degree:2,poles:vec![[0.,0.,0.],[2.,0.,0.],[0.,2.,0.],[-2.,0.,0.],[0.,0.,0.],[2.,0.,0.]],weights:vec![1.,0.7,1.2,0.9,1.,0.7],knots:vec![-2.,-1.,0.,1.,2.,3.,4.,5.,6.],domain:[0.,4.],periodic:true,reversed,seam:None};
        let mut e=Entity::new("periodic",Geometry::Curve(Curve::Nurbs(n)),None);e.placement=Transform::translation([10.,20.,0.]);let id=e.id;
        let mut doc=Document::with_template_tolerance("periodic grips");doc.entities.insert(id,e);let mut c=Controller::new(doc,limits).unwrap();select(&mut c,&[id]);let before=c.document().clone();
        let action=if edit {Top11Action::EditPoints}else{Top11Action::ControlPoints};
        if toolbar {c.top11_invoke(action).unwrap();}else{c.submit(if edit {"EditPtOn"}else{"PointsOn"}).unwrap();}
        let handles=c.top11_grip_handles().unwrap();assert_eq!(handles.len(),4);let handle=handles.iter().find(|h|h.index==if edit {1}else{0}).copied().unwrap();
        let target=[handle.point[0]+0.2,handle.point[1]-0.3,handle.point[2]+0.4];c.top11_move_grip(handle,target).unwrap();
        let after=c.document().clone();let Geometry::Curve(Curve::Nurbs(n))=&after.entities[&id].geometry else{panic!("NURBS")};n.certify_repeated_periodic(&limits).unwrap();
        let actual=if edit {curve_ops::curve_grips(&Curve::Nurbs(n.clone()),1e-9,&limits).unwrap().edit_points[1]}else{n.poles[0]};
        let actual=after.entities[&id].placement.point(actual).unwrap();assert!((0..3).all(|i|(actual[i]-target[i]).abs()<1e-8));
        let path=std::env::temp_dir().join(format!("om9-periodic-grips-{}.om9",uuid::Uuid::new_v4()));om9_archive::Archive::save(&path,&after,&limits,&om9_core::transaction::Cancellation::default()).unwrap();assert_eq!(om9_archive::Archive::open(&path,&limits).unwrap(),after);std::fs::remove_file(path).unwrap();
        c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }}}
}

#[test]
fn m03_03_08_002_grip_drag_moves_a_control_point_and_undo_restores_it() {
    let mut c = controller();
    let id = draw_line(&mut c, [0.0; 3], [10.0, 0.0, 0.0]);
    select(&mut c, &[id]);
    c.top11_invoke(Top11Action::ControlPoints).unwrap();
    let handles = c.top11_grip_handles().unwrap();
    assert_eq!(handles.len(), 2);
    let handle = handles
        .into_iter()
        .find(|handle| handle.index == 1)
        .unwrap();
    c.top11_move_grip(handle, [15.0, 2.0, 0.0]).unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Line { end, .. }) =
        &c.document().entities[&id].geometry
    else {
        panic!("grip edit lost the line");
    };
    assert!((end[0] - 15.0).abs() < 1e-9 && (end[1] - 2.0).abs() < 1e-9);
    c.undo().unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Line { end, .. }) =
        &c.document().entities[&id].geometry
    else {
        panic!("undo lost the line");
    };
    assert!((end[0] - 10.0).abs() < 1e-9);
    println!("grip_drag_end_mm=[15.0, 2.0, 0.0]");
    println!("grip_drag_undo_passed=true");
}

#[test]
fn m03_03_08_002_control_point_toggle_uses_f10_and_selection_only() {
    let mut c = controller();
    let id = draw_line(&mut c, [0.0; 3], [10.0, 0.0, 0.0]);
    select(&mut c, &[id]);
    let result = c.top11_invoke(Top11Action::ControlPoints).unwrap();
    assert!(result.message().contains("control points on"));
    assert_eq!(c.top11_grip_mode(), Some(GripMode::Control));
    assert_eq!(c.top11_grip_points().unwrap().len(), 2);
    c.top11_invoke(Top11Action::ControlPoints).unwrap();
    assert_eq!(c.top11_grip_mode(), None);
    c.cancel();
    assert_eq!(c.top11_grip_mode(), None);
    println!("grip_points_on_line=2");
    println!("grip_mode_toggle_passed=true");
}

#[test]
fn command_names_and_top11_prefix_share_the_toolbar_path() {
    let mut c = controller();
    let id = draw_line(&mut c, [0.0; 3], [10.0, 0.0, 0.0]);
    select(&mut c, &[id]);
    c.submit("Copy InPlace").unwrap();
    assert_eq!(c.document().entities.len(), 2);
    c.undo().unwrap();
    select(&mut c, &[id]);
    c.submit("top11 duplicate inplace").unwrap();
    assert_eq!(c.document().entities.len(), 2);
    c.undo().unwrap();
    select(&mut c, &[id]);
    c.submit("Split 0.25").unwrap();
    assert_eq!(c.document().entities.len(), 2);
    let half = *c.selected().iter().next().unwrap();
    select(&mut c, &[half]);
    c.submit("Trim 0.2 0.8").unwrap();
    let Geometry::Curve(curve) = &c.document().entities[&half].geometry else {
        panic!("trim lost the curve");
    };
    assert!(om9_core::curve_ops::arc_length(curve, c.coordinator().limits()).unwrap() > 0.0);
}

#[test]
fn m03_03_08_004_stress_join_explode_split_trim_cycles() {
    let mut completed = 0usize;
    for step in 0..50usize {
        let mut c = controller();
        let first = draw_line(&mut c, [0.0; 3], [10.0, 0.0, 0.0]);
        let second = draw_line(&mut c, [10.0, 0.0, 0.0], [10.0, 10.0, 0.0]);
        select(&mut c, &[first, second]);
        c.top11_invoke(Top11Action::Join).unwrap();
        assert_eq!(c.document().entities.len(), 1);
        let joined = *c.document().entities.keys().next().unwrap();
        select(&mut c, &[joined]);
        c.top11_invoke(Top11Action::Explode).unwrap();
        assert_eq!(c.document().entities.len(), 2);
        let piece = *c.document().entities.keys().next().unwrap();
        select(&mut c, &[piece]);
        c.submit("Split 0.5").unwrap();
        assert_eq!(c.document().entities.len(), 3);
        let half = *c.selected().iter().next().unwrap();
        select(&mut c, &[half]);
        c.submit("Trim 0.1 0.9").unwrap();
        let Geometry::Curve(curve) = &c.document().entities[&half].geometry else {
            panic!("trim lost the curve");
        };
        let length = om9_core::curve_ops::arc_length(curve, c.coordinator().limits()).unwrap();
        assert!(length > 0.0 && length < 10.0, "step {step}: {length}");
        completed += 1;
    }
    assert_eq!(completed, 50);
    println!("stress_cycles={completed}");
}

#[test]
fn move_prompt_names_the_next_point() {
    let mut c = controller();
    let id = draw_line(&mut c, [0.0; 3], [10.0, 0.0, 0.0]);
    select(&mut c, &[id]);
    c.submit("Move").unwrap();
    assert_eq!(c.top11_prompt(), Some("pick the base point"));
    c.accept_world_point([0.0; 3]).unwrap();
    assert_eq!(c.top11_prompt(), Some("pick the destination point"));
    c.cancel();
    assert_eq!(c.top11_prompt(), None);
    assert!(!c.is_running());
}

#[test]
fn m03_03_08_001_duplicate_options_project_and_reuse_the_last_move() {
    let mut c = controller();
    let id = draw_line(&mut c, [0.0; 3], [1.0, 0.0, 0.0]);
    select(&mut c, &[id]);
    c.submit("Copy Vertical=Yes").unwrap();
    c.accept_world_point([0.0; 3]).unwrap();
    c.accept_world_point([3.0, 4.0, 5.0]).unwrap();
    c.submit("").unwrap();
    let vertical = c
        .document()
        .entities
        .values()
        .find(|entity| entity.id != id)
        .unwrap();
    let origin = vertical.placement.point([0.0; 3]).unwrap();
    assert!(
        origin[0].abs() < 1e-9 && origin[1].abs() < 1e-9,
        "{origin:?}"
    );
    assert!((origin[2] - 5.0).abs() < 1e-9, "{origin:?}");
    println!("duplicate_vertical_origin={origin:?}");

    let mut distance = controller();
    let id = draw_line(&mut distance, [0.0; 3], [1.0, 0.0, 0.0]);
    select(&mut distance, &[id]);
    distance.submit("Copy UseLastDistance=Yes").unwrap();
    distance.accept_world_point([0.0; 3]).unwrap();
    distance.accept_world_point([10.0, 0.0, 0.0]).unwrap();
    distance.accept_world_point([0.0, 5.0, 0.0]).unwrap();
    distance.submit("").unwrap();
    assert!(distance.document().entities.values().any(|entity| {
        let point = entity.placement.point([0.0; 3]).unwrap();
        (point[1] - 10.0).abs() < 1e-9
    }));
    println!("duplicate_last_distance_y=10.0");

    let mut direction = controller();
    let id = draw_line(&mut direction, [0.0; 3], [1.0, 0.0, 0.0]);
    select(&mut direction, &[id]);
    direction.submit("Copy UseLastDirection=Yes").unwrap();
    direction.accept_world_point([0.0; 3]).unwrap();
    direction.accept_world_point([10.0, 0.0, 0.0]).unwrap();
    direction.accept_world_point([0.0, 5.0, 0.0]).unwrap();
    direction.submit("").unwrap();
    assert!(direction.document().entities.values().any(|entity| {
        let point = entity.placement.point([0.0; 3]).unwrap();
        (point[0] - 5.0).abs() < 1e-9
    }));
    println!("duplicate_last_direction_x=5.0");

    let mut from_last = controller();
    let id = draw_line(&mut from_last, [0.0; 3], [1.0, 0.0, 0.0]);
    select(&mut from_last, &[id]);
    from_last
        .submit("Copy FromLastPoint=Yes UseLastDistance=Yes")
        .unwrap();
    from_last.accept_world_point([0.0; 3]).unwrap();
    from_last.accept_world_point([5.0, 0.0, 0.0]).unwrap();
    from_last.accept_world_point([5.0, 5.0, 0.0]).unwrap();
    from_last.submit("").unwrap();
    assert!(from_last.document().entities.values().any(|entity| {
        let point = entity.placement.point([0.0; 3]).unwrap();
        (point[0] - 5.0).abs() < 1e-9 && (point[1] - 5.0).abs() < 1e-9
    }));
}

#[test]
fn m03_03_08_004_multiple_cutters_split_and_trim_one_target() {
    let mut split = controller();
    let horizontal = draw_line(&mut split, [-10.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let left = draw_line(&mut split, [-5.0, -5.0, 0.0], [-5.0, 5.0, 0.0]);
    let right = draw_line(&mut split, [5.0, -5.0, 0.0], [5.0, 5.0, 0.0]);
    select(&mut split, &[horizontal]);
    split.top11_invoke(Top11Action::Split).unwrap();
    split.top11_accept_object(left).unwrap();
    split.top11_accept_object(right).unwrap();
    split.submit("").unwrap();
    assert_eq!(split.document().entities.len(), 5);
    assert_eq!(split.selected().len(), 3);
    println!("multi_cutter_split_entities=5");

    let mut trim = controller();
    let horizontal = draw_line(&mut trim, [-10.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let left = draw_line(&mut trim, [-5.0, -5.0, 0.0], [-5.0, 5.0, 0.0]);
    let right = draw_line(&mut trim, [5.0, -5.0, 0.0], [5.0, 5.0, 0.0]);
    select(&mut trim, &[horizontal]);
    trim.top11_invoke(Top11Action::Trim).unwrap();
    trim.top11_accept_object(left).unwrap();
    trim.top11_accept_object(right).unwrap();
    trim.submit("").unwrap();
    trim.accept_world_point([-9.0, 0.0, 0.0]).unwrap();
    assert_eq!(trim.document().entities.len(), 3);
    let Geometry::Curve(curve) = &trim.document().entities[&horizontal].geometry else {
        panic!("curve")
    };
    let ends = om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap();
    assert!(
        (ends[0][0] + 5.).abs() < 1e-6 && (ends[1][0] - 10.).abs() < 1e-6,
        "{ends:?}"
    );
    println!("multi_cutter_trim_entities=3");
}

#[test]
fn trim_cmd_removes_the_picked_long_interval_and_undo_restores_it() {
    let mut c = controller();
    let target = draw_line(&mut c, [0.0; 3], [100.0, 0.0, 0.0]);
    let cutter = draw_line(&mut c, [10.0, -1.0, 0.0], [10.0, 1.0, 0.0]);
    select(&mut c, &[target]);
    c.submit("Trim").unwrap();
    c.top11_accept_object(cutter).unwrap();
    c.submit("").unwrap();
    c.accept_world_point([11.0, 0.0, 0.0]).unwrap();
    let id = *c.selected().iter().next().unwrap();
    let Geometry::Curve(curve) = &c.document().entities[&id].geometry else {
        panic!("expected curve")
    };
    let ends = om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap();
    assert!(
        (ends[0][0]).abs() < 1e-6 && (ends[1][0] - 10.0).abs() < 1e-6,
        "{ends:?}"
    );
    c.submit("").unwrap();
    c.undo().unwrap();
    assert!(c.document().entities.contains_key(&target));
    assert_eq!(c.document().entities.len(), 2);
}

#[test]
fn split_and_trim_use_placed_cutters_and_world_picks() {
    fn make_controller() -> (
        Controller,
        om9_core::identity::ObjectId,
        om9_core::identity::ObjectId,
    ) {
        let mut doc = Document::with_template_tolerance("placed curves");
        let mut target = Entity::new(
            "target",
            Geometry::Curve(om9_core::geometry::Curve::Line {
                start: [0., 0., 0.],
                end: [100., 0., 0.],
                domain: [0., 100.],
            }),
            None,
        );
        target.placement = om9_core::geometry::Transform::translation([100., 20., 0.]);
        let target_id = target.id;
        let mut cutter = Entity::new(
            "cutter",
            Geometry::Curve(om9_core::geometry::Curve::Line {
                start: [0., -5., 0.],
                end: [0., 5., 0.],
                domain: [0., 10.],
            }),
            None,
        );
        cutter.placement = om9_core::geometry::Transform::translation([110., 20., 0.]);
        let cutter_id = cutter.id;
        doc.entities.insert(target_id, target);
        doc.entities.insert(cutter_id, cutter);
        (
            Controller::new(doc, Limits::default()).unwrap(),
            target_id,
            cutter_id,
        )
    }
    for action in [Top11Action::Split, Top11Action::Trim] {
        let (mut c, target, cutter) = make_controller();
        c.select_objects(&[target]).unwrap();
        c.top11_invoke(action).unwrap();
        c.top11_accept_object(cutter).unwrap();
        c.submit("").unwrap();
        if action == Top11Action::Trim {
            c.accept_world_point([111., 20., 0.]).unwrap();
        }
        let mut intervals: Vec<_> = c
            .selected()
            .iter()
            .map(|id| {
                let entity = &c.document().entities[id];
                let Geometry::Curve(curve) = &entity.geometry else {
                    panic!("curve")
                };
                let ends = om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap();
                (
                    entity.placement.point(ends[0]).unwrap()[0],
                    entity.placement.point(ends[1]).unwrap()[0],
                )
            })
            .collect();
        intervals.sort_by(|a, b| a.0.total_cmp(&b.0));
        assert!(
            (intervals[0].0 - 100.).abs() < 1e-5 && (intervals[0].1 - 110.).abs() < 1e-5,
            "{intervals:?}"
        );
        if action == Top11Action::Split {
            assert_eq!(intervals.len(), 2);
            assert!((intervals[1].0 - 110.).abs() < 1e-5 && (intervals[1].1 - 200.).abs() < 1e-5);
        } else {
            assert_eq!(intervals.len(), 1);
        }
        if action == Top11Action::Trim {
            c.submit("").unwrap();
        }
        c.undo().unwrap();
        assert!(c.document().entities.contains_key(&target));
        assert!(c.document().entities.contains_key(&cutter));
        c.redo().unwrap();
    }
}

#[test]
fn split_cutter_uses_document_tolerance_in_world_space() {
    fn fixture(
        gap: f64,
        tolerance: bool,
    ) -> (
        Controller,
        om9_core::identity::ObjectId,
        om9_core::identity::ObjectId,
    ) {
        let mut doc = if tolerance {
            Document::with_template_tolerance("world tolerance")
        } else {
            Document::new("missing tolerance")
        };
        let mut target = Entity::new(
            "target",
            Geometry::Curve(om9_core::geometry::Curve::Line {
                start: [0., 0., 0.],
                end: [10., 0., 0.],
                domain: [0., 10.],
            }),
            None,
        );
        target.placement = Transform::scale([10., 1., 0.1]).unwrap();
        let cutter = Entity::new(
            "cutter",
            Geometry::Curve(om9_core::geometry::Curve::Line {
                start: [50., -1., gap],
                end: [50., 1., gap],
                domain: [0., 2.],
            }),
            None,
        );
        let (target_id, cutter_id) = (target.id, cutter.id);
        doc.entities.insert(target_id, target);
        doc.entities.insert(cutter_id, cutter);
        (
            Controller::new(doc, Limits::default()).unwrap(),
            target_id,
            cutter_id,
        )
    }
    let (mut c, target, cutter) = fixture(0.0005, true);
    c.select_objects(&[target]).unwrap();
    c.submit("Split").unwrap();
    c.top11_accept_object(cutter).unwrap();
    c.submit("").unwrap();
    assert_eq!(c.document().entities.len(), 3);
    for id in c.selected() {
        let entity = &c.document().entities[id];
        let Geometry::Curve(curve) = &entity.geometry else {
            panic!("curve")
        };
        let ends = om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap();
        assert!(
            ends.iter()
                .any(|p| (entity.placement.point(*p).unwrap()[0] - 50.).abs() < 1e-7)
        );
    }
    c.undo().unwrap();
    assert!(c.document().entities.contains_key(&target));
    c.redo().unwrap();
    assert_eq!(c.document().entities.len(), 3);
    let (mut trim, target, cutter) = fixture(0.0005, true);
    trim.select_objects(&[target]).unwrap();
    trim.submit("Trim").unwrap();
    trim.top11_accept_object(cutter).unwrap();
    trim.submit("").unwrap();
    trim.accept_world_point([75., 0., 0.]).unwrap();
    let entity = &trim.document().entities[&target];
    let Geometry::Curve(curve) = &entity.geometry else {
        panic!("curve")
    };
    let ends = om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap();
    assert!((entity.placement.point(ends[0]).unwrap()[0]).abs() < 1e-7);
    assert!((entity.placement.point(ends[1]).unwrap()[0] - 50.).abs() < 1e-7);
    trim.submit("").unwrap();
    trim.undo().unwrap();
    trim.redo().unwrap();
    for (gap, tolerance) in [(0.002, true), (0., false)] {
        let (mut c, target, cutter) = fixture(gap, tolerance);
        c.select_objects(&[target]).unwrap();
        c.submit("Split").unwrap();
        assert!(c.top11_accept_object(cutter).is_err());
        assert!(c.top11_needs_object());
        assert_eq!(c.document().entities.len(), 2);
        assert_eq!(c.coordinator().undo_len(), 0);
    }
}

#[test]
fn split_affine_arc_cutter_preserves_original_angle() {
    let mut doc = Document::with_template_tolerance("affine arc cut");
    let mut target = Entity::new(
        "arc",
        Geometry::Curve(om9_core::geometry::Curve::Circle {
            center: [0.; 3],
            x: [1., 0., 0.],
            y: [0., 1., 0.],
            radius: 1.,
            domain: [0., 1.],
            angle_domain: Some([0., std::f64::consts::PI]),
        }),
        None,
    );
    target.placement = Transform::scale([10., 1., 1.]).unwrap();
    let cutter = Entity::new(
        "cutter",
        Geometry::Curve(om9_core::geometry::Curve::Line {
            start: [5., -2., 0.],
            end: [5., 2., 0.],
            domain: [0., 4.],
        }),
        None,
    );
    let (target_id, cutter_id) = (target.id, cutter.id);
    doc.entities.insert(target_id, target);
    doc.entities.insert(cutter_id, cutter);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[target_id]).unwrap();
    c.submit("Split").unwrap();
    c.top11_accept_object(cutter_id).unwrap();
    c.submit("").unwrap();
    assert_eq!(c.selected().len(), 2);
    for id in c.selected() {
        let e = &c.document().entities[id];
        let Geometry::Curve(curve) = &e.geometry else {
            panic!("curve")
        };
        let ends = om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap();
        assert!(
            ends.iter().any(|p| {
                let p = e.placement.point(*p).unwrap();
                (p[0] - 5.).abs() < 1e-5 && (p[1] - 3.0f64.sqrt() / 2.).abs() < 1e-5
            }),
            "{ends:?}"
        );
    }
    c.undo().unwrap();
    assert!(c.document().entities.contains_key(&target_id));
}

#[test]
fn split_affine_reversed_polycurve_maps_the_correct_segment_parameter() {
    let mut doc = Document::with_template_tolerance("affine reversed polycurve");
    let arc = om9_core::geometry::Curve::Circle {
        center: [0.; 3],
        x: [1., 0., 0.],
        y: [0., 1., 0.],
        radius: 1.,
        domain: [0., 1.],
        angle_domain: Some([0., std::f64::consts::PI]),
    };
    let line = om9_core::geometry::Curve::Line {
        start: [-1., 0., 0.],
        end: [-2., 0., 0.],
        domain: [0., 1.],
    };
    let mut target = Entity::new(
        "polycurve",
        Geometry::Curve(om9_core::geometry::Curve::Polycurve {
            segments: vec![arc, line],
            reversed: true,
        }),
        None,
    );
    target.placement = Transform::scale([10., 1., 1.]).unwrap();
    let cutter = Entity::new(
        "cutter",
        Geometry::Curve(om9_core::geometry::Curve::Line {
            start: [5., -2., 0.],
            end: [5., 2., 0.],
            domain: [0., 4.],
        }),
        None,
    );
    let (target_id, cutter_id) = (target.id, cutter.id);
    doc.entities.insert(target_id, target);
    doc.entities.insert(cutter_id, cutter);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[target_id]).unwrap();
    c.submit("Split").unwrap();
    c.top11_accept_object(cutter_id).unwrap();
    c.submit("").unwrap();
    assert_eq!(c.selected().len(), 2);
    for id in c.selected() {
        let e = &c.document().entities[id];
        let Geometry::Curve(curve) = &e.geometry else {
            panic!("curve")
        };
        let ends = om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap();
        assert!(
            ends.iter().any(|p| {
                let p = e.placement.point(*p).unwrap();
                (p[0] - 5.).abs() < 1e-5 && (p[1] - 3.0f64.sqrt() / 2.).abs() < 1e-5
            }),
            "{ends:?}"
        );
    }
    c.undo().unwrap();
    assert!(c.document().entities.contains_key(&target_id));
}

#[test]
fn split_and_trim_nested_closed_circle_and_periodic_nurbs() {
    use om9_core::{curve_ops, geometry::{Curve,NurbsCurve}};
    let limits=Limits::default();
    for circle in [false,true] {for action in [Top11Action::Split,Top11Action::Trim] {for toolbar in [false,true] {for reversed in [false,true] {for nested in [false,true] {for shifted in [false,true] {
        if shifted && (circle || !nested) {continue;}
        let base=if circle {Curve::Circle{center:[0.;3],x:[1.,0.,0.],y:[0.,1.,0.],radius:2.,domain:[0.,std::f64::consts::TAU],angle_domain:None}} else {
            Curve::Nurbs(NurbsCurve{degree:2,poles:vec![[0.,0.,0.],[2.,0.,0.],[0.,2.,0.],[-2.,0.,0.],[0.,0.,0.],[2.,0.,0.]],weights:vec![1.;6],knots:vec![-2.,-1.,0.,1.,2.,3.,4.,5.,6.],domain:[0.,4.],periodic:true,reversed:false,seam:None})
        };
        let base=if shifted {curve_ops::set_seam(&base,0.31,1e-9,&limits).unwrap()}else{base};
        let curve=if nested {let inner=Curve::Polycurve{segments:vec![base],reversed};Curve::Polycurve{segments:vec![inner],reversed:false}} else {base};
        let target=Entity::new("nested closed",Geometry::Curve(curve.clone()),None);
        let cutter=|name,x|Entity::new(name,Geometry::Curve(Curve::Line{start:[x,-1.,0.],end:[x,if circle {3.}else{2.},0.],domain:[0.,1.]}),None);
        let a=cutter("right",if circle {2.}else{1.5});let b=cutter("left",if circle {-2.}else{-1.5});let ids=[target.id,a.id,b.id];
        let mut doc=Document::with_template_tolerance("nested closed");for e in [target,a,b]{doc.entities.insert(e.id,e);}
        let mut c=Controller::new(doc,limits).unwrap();select(&mut c,&[ids[0]]);let before=c.document().clone();
        if toolbar{c.top11_invoke(action).unwrap();}else{c.submit(action.label()).unwrap();}
        c.top11_accept_object(ids[1]).unwrap_or_else(|e|panic!("first cutter circle={circle} action={action:?} reverse={reversed}: {e:?}"));c.top11_accept_object(ids[2]).unwrap_or_else(|e|panic!("second cutter circle={circle} action={action:?} reverse={reversed}: {e:?}"));c.submit("").unwrap();
        if action==Top11Action::Trim {let pick=curve_ops::point_at(&curve,0.25,&limits).unwrap();c.accept_world_point(pick).unwrap();c.submit("").unwrap();}
        let after=c.document().clone();let pieces:Vec<_>=after.entities.iter().filter(|(id,_)|**id!=ids[1]&&**id!=ids[2]).collect();
        assert_eq!(pieces.len(),if action==Top11Action::Split{2}else{1},"circle={circle} reversed={reversed}");
        let mut length=0.;for (_,entity) in pieces {let Geometry::Curve(curve)=&entity.geometry else{panic!("curve")};length+=curve_ops::arc_length(curve,&limits).unwrap();let ends=curve_ops::curve_endpoints(curve,&limits).unwrap();
            for x in if circle {[-2.,2.]}else{[-1.5,1.5]} {assert!(ends.iter().any(|p|(p[0]-x).abs()<1e-7 && (p[1]-if circle {0.}else{0.25}).abs()<1e-7),"{ends:?}");}
        }
        let original=curve_ops::arc_length(&curve,&limits).unwrap();assert!(length>0. && (if action==Top11Action::Split{(length-original).abs()<1e-6}else{length<original}),"{length} {original}");
        let path=std::env::temp_dir().join(format!("om9-nested-closed-{}.om9",uuid::Uuid::new_v4()));om9_archive::Archive::save(&path,&after,&limits,&om9_core::transaction::Cancellation::default()).unwrap();assert_eq!(om9_archive::Archive::open(&path,&limits).unwrap(),after);std::fs::remove_file(path).unwrap();
        c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }}}}}}
}

#[test]
fn split_and_trim_nested_arc_and_nurbs_through_both_entrypoints() {
    use om9_core::{curve_build::arc_by_center, curve_ops, geometry::{Curve,NurbsCurve}};
    let limits=Limits::default();
    for arc in [false,true] { for action in [Top11Action::Split, Top11Action::Trim] { for toolbar in [false,true] {
        let (first, end, cut, pick) = if arc {
            (arc_by_center([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,0.,std::f64::consts::FRAC_PI_2).unwrap(), [-2.,2.,0.], [1.,3f64.sqrt(),0.], [2.,0.,0.])
        } else {
            (Curve::Nurbs(NurbsCurve { degree:2, poles:vec![[0.,0.,0.],[2.,2.,0.],[4.,0.,0.]], weights:vec![1.;3], knots:vec![0.,0.,0.,1.,1.,1.], domain:[0.,1.], periodic:false, reversed:false, seam:None }), [6.,0.,0.], [2.,1.,0.], [0.,0.,0.])
        };
        let join=if arc { [0.,2.,0.] } else { [4.,0.,0.] };
        let inner=Curve::Polycurve { segments:vec![first,Curve::Line{start:join,end,domain:[0.,1.]}],reversed:false };
        let target=Entity::new("nested mixed",Geometry::Curve(Curve::Polycurve{segments:vec![inner],reversed:false}),None);
        let cutter=Entity::new("cutter",Geometry::Curve(Curve::Line{start:[cut[0],-2.,0.],end:[cut[0],4.,0.],domain:[0.,1.]}),None);
        let (target_id,cutter_id)=(target.id,cutter.id);let mut doc=Document::with_template_tolerance("nested mixed");
        doc.entities.insert(target_id,target);doc.entities.insert(cutter_id,cutter);
        let mut c=Controller::new(doc,limits).unwrap();select(&mut c,&[target_id]);let before=c.document().clone();
        if toolbar { c.top11_invoke(action).unwrap(); } else { c.submit(action.label()).unwrap(); }
        c.top11_accept_object(cutter_id).unwrap();c.submit("").unwrap();
        if action==Top11Action::Trim { c.accept_world_point(pick).unwrap();c.submit("").unwrap(); }
        let after=c.document().clone();let pieces:Vec<_>=after.entities.iter().filter(|(id,_)|**id!=cutter_id).collect();
        assert_eq!(pieces.len(),if action==Top11Action::Split {2}else{1});
        let mut cut_ends=0;let mut original_end=false;
        for (_,entity) in pieces { let Geometry::Curve(curve)=&entity.geometry else{panic!("curve")};for point in curve_ops::curve_endpoints(curve,&limits).unwrap() {
            if (0..3).all(|i|(point[i]-cut[i]).abs()<1e-6) {cut_ends+=1;}
            if (0..3).all(|i|(point[i]-end[i]).abs()<1e-6) {original_end=true;}
        }}
        assert_eq!(cut_ends,if action==Top11Action::Split {2}else{1});assert!(original_end);
        let path=std::env::temp_dir().join(format!("om9-nested-mixed-{}.om9",uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&path,&after,&limits,&om9_core::transaction::Cancellation::default()).unwrap();
        assert_eq!(om9_archive::Archive::open(&path,&limits).unwrap(),after);std::fs::remove_file(path).unwrap();
        c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }}}
}

#[test]
fn split_and_trim_nested_polycurve_with_line_cutter() {
    use om9_core::{curve_ops, geometry::Curve};
    let limits = Limits::default();
    let line = |a, b| Curve::Line { start: [a, 0., 0.], end: [b, 0., 0.], domain: [0., 1.] };
    for action in [Top11Action::Split, Top11Action::Trim] {
        for toolbar in [false, true] {
            let target = Entity::new("nested", Geometry::Curve(Curve::Polycurve {
                segments: vec![Curve::Polycurve { segments: vec![line(0., 2.), line(2., 4.)], reversed: false }, line(4., 6.)],
                reversed: false,
            }), None);
            let cutter = Entity::new("cutter", Geometry::Curve(Curve::Line { start: [3., -1., 0.], end: [3., 1., 0.], domain: [0., 1.] }), None);
            let (target_id, cutter_id) = (target.id, cutter.id);
            let mut doc = Document::with_template_tolerance("nested polycurve");
            doc.entities.insert(target_id, target);
            doc.entities.insert(cutter_id, cutter);
            let mut c = Controller::new(doc, limits).unwrap();
            select(&mut c, &[target_id]);
            let before = c.document().clone();
            if toolbar { c.top11_invoke(action).unwrap(); } else { c.submit(action.label()).unwrap(); }
            c.top11_accept_object(cutter_id).unwrap();
            c.submit("").unwrap();
            if action == Top11Action::Trim { c.accept_world_point([1., 0., 0.]).unwrap(); c.submit("").unwrap(); }
            let after = c.document().clone();
            let mut bounds: Vec<_> = after.entities.iter().filter(|(id, _)| **id != cutter_id).map(|(_, e)| {
                let Geometry::Curve(curve) = &e.geometry else { panic!("curve") };
                let ends = curve_ops::curve_endpoints(curve, &limits).unwrap();
                (ends[0][0], ends[1][0])
            }).collect();
            bounds.sort_by(|a, b| a.0.total_cmp(&b.0));
            let expected: &[(f64, f64)] = if action == Top11Action::Split { &[(0., 3.), (3., 6.)] } else { &[(3., 6.)] };
            assert_eq!(bounds.len(), expected.len());
            for (actual, expected) in bounds.iter().zip(expected) { assert!((actual.0-expected.0).abs()<1e-9 && (actual.1-expected.1).abs()<1e-9, "{bounds:?}"); }
            let path=std::env::temp_dir().join(format!("om9-nested-polycurve-{}.om9", uuid::Uuid::new_v4()));
            om9_archive::Archive::save(&path, &after, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
            assert_eq!(om9_archive::Archive::open(&path, &limits).unwrap(), after);
            std::fs::remove_file(path).unwrap();
            c.undo().unwrap(); assert_eq!(c.document(), &before);
            c.redo().unwrap(); assert_eq!(c.document(), &after);
        }
    }
}

#[test]
fn split_and_trim_polycurve_keep_actual_cut_location() {
    fn line(a: [f64; 3], b: [f64; 3]) -> om9_core::geometry::Curve {
        om9_core::geometry::Curve::Line {
            start: a,
            end: b,
            domain: [0., 1.],
        }
    }
    for action in [Top11Action::Split, Top11Action::Trim] {
        let mut doc = Document::with_template_tolerance("unequal polycurve");
        let target = Entity::new(
            "target",
            Geometry::Curve(om9_core::geometry::Curve::Polycurve {
                segments: vec![
                    line([0., 0., 0.], [1., 0., 0.]),
                    line([1., 0., 0.], [11., 0., 0.]),
                ],
                reversed: false,
            }),
            None,
        );
        let cutter = Entity::new(
            "cutter",
            Geometry::Curve(line([0.5, -1., 0.], [0.5, 1., 0.])),
            None,
        );
        let (target_id, cutter_id) = (target.id, cutter.id);
        doc.entities.insert(target_id, target);
        doc.entities.insert(cutter_id, cutter);
        let mut c = Controller::new(doc, Limits::default()).unwrap();
        c.select_objects(&[target_id]).unwrap();
        c.top11_invoke(action).unwrap();
        c.top11_accept_object(cutter_id).unwrap();
        c.submit("").unwrap();
        if action == Top11Action::Trim {
            c.accept_world_point([0.25, 0., 0.]).unwrap();
        }
        let mut bounds: Vec<_> = c
            .selected()
            .iter()
            .map(|id| {
                let Geometry::Curve(curve) = &c.document().entities[id].geometry else {
                    panic!("curve")
                };
                let ends = om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap();
                (ends[0][0], ends[1][0])
            })
            .collect();
        bounds.sort_by(|a, b| a.0.total_cmp(&b.0));
        let expected: &[(f64, f64)] = if action == Top11Action::Split {
            &[(0., 0.5), (0.5, 11.)]
        } else {
            &[(0.5, 11.)]
        };
        assert_eq!(bounds.len(), expected.len());
        for (actual, expected) in bounds.iter().zip(expected) {
            assert!(
                (actual.0 - expected.0).abs() < 1e-5 && (actual.1 - expected.1).abs() < 1e-5,
                "{bounds:?}"
            );
        }
        if action == Top11Action::Trim {
            c.submit("").unwrap();
        }
        c.undo().unwrap();
        assert!(c.document().entities.contains_key(&target_id));
        c.redo().unwrap();
    }
}

#[test]
fn m03_03_08_004_a_surface_can_be_the_cutting_object() {
    let mut doc = Document::with_template_tolerance("surface cutter");
    let line_entity = Entity::new(
        "line",
        Geometry::Curve(om9_core::geometry::Curve::Line {
            start: [-10.0, 0.0, 0.0],
            end: [10.0, 0.0, 0.0],
            domain: [0.0, 20.0],
        }),
        None,
    );
    let line_id = line_entity.id;
    let surface = NurbsSurface {
        degree: [1, 1],
        size: [2, 2],
        poles: vec![
            [0.0, -5.0, -5.0],
            [0.0, 5.0, -5.0],
            [0.0, -5.0, 5.0],
            [0.0, 5.0, 5.0],
        ],
        weights: vec![1.0; 4],
        knots: [vec![0.0, 0.0, 1.0, 1.0], vec![0.0, 0.0, 1.0, 1.0]],
        domain: [[0.0, 1.0], [0.0, 1.0]],
        periodic: [false, false],
        reversed: false,
    };
    let surface_entity = Entity::new("surface", Geometry::Surface(surface), None);
    let surface_id = surface_entity.id;
    doc.entities.insert(line_id, line_entity);
    doc.entities.insert(surface_id, surface_entity);
    for shifted in [false, true] {
        let mut fixture = doc.clone();
        if shifted {
            fixture.entities.get_mut(&line_id).unwrap().placement =
                Transform::translation([10., 0., 0.]);
            fixture.entities.get_mut(&surface_id).unwrap().placement =
                Transform::translation([12., 0., 0.]);
        }
        let mut c = Controller::new(fixture, Limits::default()).unwrap();
        select(&mut c, &[line_id]);
        c.top11_invoke(Top11Action::Split).unwrap();
        c.top11_accept_object(surface_id).unwrap();
        c.submit("").unwrap();
        assert_eq!(c.document().entities.len(), 3);
        assert_eq!(c.selected().len(), 2);
        let mut cuts: Vec<f64> = c
            .selected()
            .iter()
            .flat_map(|id| {
                let entity = &c.document().entities[id];
                let Geometry::Curve(curve) = &entity.geometry else {
                    panic!("curve")
                };
                om9_core::curve_ops::curve_endpoints(curve, &Limits::default())
                    .unwrap()
                    .into_iter()
                    .map(|p| entity.placement.point(p).unwrap()[0])
                    .collect::<Vec<_>>()
            })
            .collect();
        cuts.sort_by(f64::total_cmp);
        let cut = if shifted { 12. } else { 0. };
        assert!(
            (cuts[1] - cut).abs() < 1e-5 && (cuts[2] - cut).abs() < 1e-5,
            "{cuts:?}"
        );
        c.undo().unwrap();
        assert!(c.document().entities.contains_key(&line_id));
    }
}

#[test]
fn m03_03_08_003_three_point_plane_and_non_rigid_mirror() {
    let mut c = controller();
    let id = draw_line(&mut c, [1.0, 0.0, 0.0], [4.0, 0.0, 0.0]);
    select(&mut c, &[id]);
    c.submit("Mirror 3Point").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([0.0, 1.0, 0.0]).unwrap();
    c.accept_world_point([0.0, 0.0, 1.0]).unwrap();
    assert_eq!(c.document().entities.len(), 2);
    let copy = c
        .document()
        .entities
        .values()
        .find(|entity| entity.id != id)
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Line { start, .. }) = &copy.geometry else {
        panic!("three-point mirror did not copy a line");
    };
    assert!((start[0] + 1.0).abs() < 1e-9, "{start:?}");
    println!("three_point_mirror_start_x_mm={:.12}", start[0]);

    let mut doc = Document::new("non-rigid");
    let mut entity = Entity::new(
        "line",
        Geometry::Curve(om9_core::geometry::Curve::Line {
            start: [0.0; 3],
            end: [1.0, 0.0, 0.0],
            domain: [0.0, 1.0],
        }),
        None,
    );
    entity.placement = Transform::scale([2.0, 1.0, 1.0]).unwrap();
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    select(&mut c, &[id]);
    c.top11_mirror(true).unwrap();
    c.accept_world_point([0.0, -1.0, 0.0]).unwrap();
    c.accept_world_point([0.0, 1.0, 0.0]).unwrap();
    let copy = c
        .document()
        .entities
        .values()
        .find(|entity| entity.id != id)
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Line { end, .. }) = &copy.geometry else {
        panic!("non-rigid mirror did not copy a line");
    };
    assert!((end[0] + 2.0).abs() < 1e-9, "{end:?}");
    println!("non_rigid_mirror_end_x_mm={:.12}", end[0]);
}

#[test]
fn m03_03_08_002_circle_and_surface_control_grips_reach_the_toolbar_adapter() {
    let mut doc = Document::new("grips");
    let circle = Entity::new(
        "circle",
        Geometry::Curve(om9_core::geometry::Curve::Circle {
            center: [0.0; 3],
            x: [1.0, 0.0, 0.0],
            y: [0.0, 1.0, 0.0],
            radius: 2.0,
            domain: [0.0, std::f64::consts::TAU],
            angle_domain: None,
        }),
        None,
    );
    let circle_id = circle.id;
    let surface = Entity::new(
        "surface",
        Geometry::Surface(NurbsSurface {
            degree: [1, 1],
            size: [2, 2],
            poles: vec![
                [0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            weights: vec![1.0; 4],
            knots: [vec![0.0, 0.0, 1.0, 1.0], vec![0.0, 0.0, 1.0, 1.0]],
            domain: [[0.0, 1.0], [0.0, 1.0]],
            periodic: [false, false],
            reversed: false,
        }),
        None,
    );
    let surface_id = surface.id;
    doc.entities.insert(circle_id, circle);
    doc.entities.insert(surface_id, surface);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    select(&mut c, &[circle_id]);
    c.top11_invoke(Top11Action::ControlPoints).unwrap();
    let handles = c.top11_grip_handles().unwrap();
    assert_eq!(handles.len(), 5);
    c.top11_move_grip(handles[1], [5.0, 0.0, 0.0]).unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Circle { radius, .. }) =
        &c.document().entities[&circle_id].geometry
    else {
        panic!("circle grip lost the circle");
    };
    assert!((radius - 5.0).abs() < 1e-9);
    println!("circle_grip_radius={radius:.12}");
    c.top11_invoke(Top11Action::ControlPoints).unwrap();
    select(&mut c, &[surface_id]);
    c.top11_invoke(Top11Action::ControlPoints).unwrap();
    let handles = c.top11_grip_handles().unwrap();
    assert_eq!(handles.len(), 4);
    println!("surface_control_grips={}", handles.len());
    c.top11_invoke(Top11Action::ControlPoints).unwrap();
    c.top11_invoke(Top11Action::EditPoints).unwrap();
    let handles = c.top11_grip_handles().unwrap();
    assert_eq!(handles.len(), 4);
    let target = [
        handles[3].point[0] + 1.0,
        handles[3].point[1] + 1.0,
        handles[3].point[2],
    ];
    c.top11_move_grip(handles[3], target).unwrap();
    println!("surface_edit_grips={}", handles.len());
}

#[test]
fn m03_03_08_002_surface_edit_grips_keep_periodic_and_reversed_records() {
    let limits = Limits::default();
    let cylinder = om9_core::surface_kernel::cylinder(
        [0.0; 3],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        5.0,
        10.0,
        &limits,
    )
    .unwrap();
    let mut reversed = cylinder.clone();
    reversed.reversed = true;
    let mut periodic = cylinder;
    periodic.periodic = [true, false];
    let mut doc = Document::new("surface-edit");
    let reversed_entity = Entity::new("reversed", Geometry::Surface(reversed), None);
    let reversed_id = reversed_entity.id;
    let periodic_entity = Entity::new("periodic", Geometry::Surface(periodic), None);
    let periodic_id = periodic_entity.id;
    doc.entities.insert(reversed_id, reversed_entity);
    doc.entities.insert(periodic_id, periodic_entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();

    select(&mut c, &[reversed_id]);
    c.top11_invoke(Top11Action::EditPoints).unwrap();
    let handles = c.top11_grip_handles().unwrap();
    let target = [
        handles[10].point[0] + 1.0,
        handles[10].point[1],
        handles[10].point[2],
    ];
    c.top11_move_grip(handles[10], target).unwrap();
    let Geometry::Surface(edited) = &c.document().entities[&reversed_id].geometry else {
        panic!("reversed edit lost the surface");
    };
    assert!(edited.reversed);
    assert_eq!(edited.periodic, [false, false]);
    println!("surface_edit_reversed=true");

    c.top11_invoke(Top11Action::EditPoints).unwrap();
    select(&mut c, &[periodic_id]);
    c.top11_invoke(Top11Action::EditPoints).unwrap();
    let handles = c.top11_grip_handles().unwrap();
    let target = [
        handles[0].point[0],
        handles[0].point[1],
        handles[0].point[2] + 1.0,
    ];
    c.top11_move_grip(handles[0], target).unwrap();
    let Geometry::Surface(edited) = &c.document().entities[&periodic_id].geometry else {
        panic!("periodic edit lost the surface");
    };
    assert!(edited.periodic[0]);
    println!("surface_edit_periodic=true");
}

fn large_surface(size: usize) -> NurbsSurface {
    let degree = 3usize;
    let mut knots = vec![0.0; degree + 1];
    let spans = size - degree;
    for step in 1..spans {
        knots.push(step as f64 / spans as f64);
    }
    knots.extend(std::iter::repeat_n(1.0, degree + 1));
    let mut poles = Vec::with_capacity(size * size);
    for i in 0..size {
        for j in 0..size {
            let u = i as f64 / (size - 1) as f64;
            let v = j as f64 / (size - 1) as f64;
            poles.push([
                u * 100.0,
                v * 100.0,
                (u * 8.0).sin() * (v * 8.0).cos() * 5.0,
            ]);
        }
    }
    NurbsSurface {
        degree: [degree, degree],
        size: [size, size],
        weights: vec![1.0; size * size],
        poles,
        knots: [knots.clone(), knots],
        domain: [[0.0, 1.0], [0.0, 1.0]],
        periodic: [false, false],
        reversed: false,
    }
}

#[test]
fn m03_03_08_002_surface_edit_grid_is_fast_through_the_adapter() {
    let mut doc = Document::new("surface-perf");
    let entity = Entity::new("large", Geometry::Surface(large_surface(96)), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    select(&mut c, &[id]);
    c.top11_invoke(Top11Action::EditPoints).unwrap();
    let started = std::time::Instant::now();
    let handles = c.top11_grip_handles().unwrap();
    let handles_ms = started.elapsed().as_secs_f64() * 1000.0;
    assert_eq!(handles.len(), 96 * 96);
    assert!(
        handles_ms < 1500.0,
        "adapter handles took {handles_ms:.3} ms"
    );
    let started = std::time::Instant::now();
    let target = [
        handles[48 * 96 + 48].point[0] + 1.0,
        handles[48 * 96 + 48].point[1],
        handles[48 * 96 + 48].point[2],
    ];
    c.top11_move_grip(handles[48 * 96 + 48], target).unwrap();
    let move_ms = started.elapsed().as_secs_f64() * 1000.0;
    assert!(move_ms < 500.0, "adapter move took {move_ms:.3} ms");
    println!("surface_edit_handles_96_ms={handles_ms:.3}");
    println!("surface_edit_move_96_ms={move_ms:.3}");
}

#[test]
fn edit_points_on_rational_curve_preserves_representation_and_undo() {
    use om9_core::geometry::{Curve, NurbsCurve};
    let nurbs = NurbsCurve { degree: 3, poles: vec![[0.,0.,0.], [1.,3.,0.], [4.,-1.,0.], [6.,2.,0.], [8.,0.,0.]], weights: vec![1.,0.7,1.4,0.8,1.], knots: vec![2.,2.,2.,2.,4.,7.,7.,7.,7.], domain: [2.,7.], periodic: false, reversed: true, seam: None };
    let entity = Entity::new("rational", Geometry::Curve(Curve::Nurbs(nurbs.clone())), None);
    let id = entity.id;
    let mut doc = Document::with_template_tolerance("edit point");
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    select(&mut c, &[id]);
    c.submit("EditPtOn").unwrap();
    let handles = c.top11_grip_handles().unwrap();
    let handle = handles.iter().find(|h| h.index == 2).unwrap();
    let target = [handle.point[0] + 1., handle.point[1] - 2., 3.];
    let before = c.document().clone();
    c.top11_move_grip(*handle, target).unwrap();
    let after = c.document().clone();
    let Geometry::Curve(Curve::Nurbs(result)) = &after.entities[&id].geometry else { panic!("lost NURBS") };
    assert_eq!((&result.knots, &result.weights, result.domain, result.reversed), (&nurbs.knots, &nurbs.weights, nurbs.domain, nurbs.reversed));
    let moved = c.top11_grip_handles().unwrap();
    for (a, b) in handles.iter().zip(&moved) {
        let expected = if a.index == 2 { target } else { a.point };
        assert!((0..3).all(|i| (b.point[i] - expected[i]).abs() < 1e-9));
    }
    c.undo().unwrap(); assert_eq!(c.document(), &before);
    c.redo().unwrap(); assert_eq!(c.document(), &after);
}

#[test]
fn edit_points_circle_drag_reaches_target_and_undo_restores_circle() {
    use om9_core::geometry::Curve;
    for (angles, via_command) in [(None, false), (None, true), (Some([0.4, 2.3]), false), (Some([0.4, 2.3]), true)] {
        let curve = Curve::Circle { center: [1.,2.,3.], x: [1.,0.,0.], y: [0.,1.,0.], radius: 4., domain: if angles.is_none() { [0.,std::f64::consts::TAU] } else { [0.,1.] }, angle_domain: angles };
        let entity = Entity::new("circle", Geometry::Curve(curve), None);
        let id = entity.id;
        let mut doc = Document::with_template_tolerance("circle edit");
        doc.entities.insert(id, entity);
        let mut c = Controller::new(doc, Limits::default()).unwrap();
        select(&mut c, &[id]);
        if via_command { c.submit("EditPtOn").unwrap(); } else { c.top11_invoke(Top11Action::EditPoints).unwrap(); }
        let handle = c.top11_grip_handles().unwrap()[0];
        let target = [handle.point[0] + 2., handle.point[1] - 1., handle.point[2] + 3.];
        let before = c.document().clone();
        c.top11_move_grip(handle, target).unwrap();
        let after = c.document().clone();
        let Geometry::Curve(curve) = &after.entities[&id].geometry else { panic!("lost curve") };
        let endpoints = om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap();
        assert!((0..3).all(|i| (endpoints[0][i] - target[i]).abs() < 1e-8));
        if angles.is_none() { assert!((0..3).all(|i| (endpoints[1][i] - target[i]).abs() < 1e-8)); }
        c.undo().unwrap(); assert_eq!(c.document(), &before);
        c.redo().unwrap(); assert_eq!(c.document(), &after);
    }
}

#[test]
fn edit_points_polycurve_joint_moves_together_and_undo_is_atomic() {
    use om9_core::geometry::Curve;
    for reversed in [false, true] {
        let segment = |start, end| Curve::Line { start, end, domain: [0., 5.] };
        let curve = Curve::Polycurve { segments: vec![segment([0.,0.,0.], [5.,0.,0.]), segment([5.,0.,0.], [5.,5.,0.])], reversed };
        let entity = Entity::new("joined", Geometry::Curve(curve), None);
        let id = entity.id;
        let mut doc = Document::with_template_tolerance("edit joined curve"); doc.entities.insert(id, entity);
        let mut c = Controller::new(doc, Limits::default()).unwrap();
        select(&mut c, &[id]); c.submit("EditPtOn").unwrap();
        let handle = c.top11_grip_handles().unwrap()[1];
        let target = [6.,1.,2.];
        let before = c.document().clone();
        let invalid = if reversed { [0.,0.,0.] } else { [5.,5.,0.] };
        assert!(c.top11_move_grip(handle, invalid).is_err());
        assert_eq!(c.document(), &before);
        c.top11_move_grip(handle, target).unwrap();
        let after = c.document().clone();
        let Geometry::Curve(result) = &after.entities[&id].geometry else { panic!("lost curve") };
        let grips = om9_core::curve_ops::curve_grips(result, 1e-9, &Limits::default()).unwrap();
        assert!((0..3).all(|axis| (grips.edit_points[1][axis] - target[axis]).abs() < 1e-9 && (grips.edit_points[2][axis] - target[axis]).abs() < 1e-9));
        c.undo().unwrap(); assert_eq!(c.document(), &before); c.redo().unwrap(); assert_eq!(c.document(), &after);
    }
}

#[test]
fn control_points_polycurve_joint_preserves_connection_and_atomic_undo() {
    use om9_core::geometry::Curve;
    for reversed in [false, true] {
        let segment = |start, end| Curve::Line { start, end, domain: [0., 5.] };
        let curve = Curve::Polycurve { segments: vec![segment([0.,0.,0.], [5.,0.,0.]), segment([5.,0.,0.], [5.,5.,0.])], reversed };
        let entity = Entity::new("joined", Geometry::Curve(curve), None);
        let id = entity.id;
        let mut doc = Document::with_template_tolerance("control joined curve"); doc.entities.insert(id, entity);
        let mut c = Controller::new(doc, Limits::default()).unwrap();
        select(&mut c, &[id]); c.submit("PointsOn").unwrap();
        let handle = c.top11_grip_handles().unwrap()[1];
        let before = c.document().clone();
        let invalid = if reversed { [0.,0.,0.] } else { [5.,5.,0.] };
        assert!(c.top11_move_grip(handle, invalid).is_err()); assert_eq!(c.document(), &before);
        let target = [6.,1.,2.]; c.top11_move_grip(handle, target).unwrap();
        let after = c.document().clone();
        let Geometry::Curve(result) = &after.entities[&id].geometry else { panic!("lost curve") };
        let grips = om9_core::curve_ops::curve_grips(result, 1e-9, &Limits::default()).unwrap();
        assert!((0..3).all(|axis| (grips.poles[1][axis] - target[axis]).abs() < 1e-9 && (grips.poles[2][axis] - target[axis]).abs() < 1e-9));
        c.undo().unwrap(); assert_eq!(c.document(), &before); c.redo().unwrap(); assert_eq!(c.document(), &after);
    }
}

#[test]
fn polycurve_grip_world_tolerance_keeps_near_closed_seam_and_undo() {
    use om9_core::geometry::Curve;
    for action in [Top11Action::EditPoints, Top11Action::ControlPoints] {
        for reversed in [false, true] {
            let segment = |start, end| Curve::Line { start, end, domain: [0., 5.] };
            let curve = Curve::Polycurve { segments: vec![segment([0.,0.,0.], [5.,0.,0.]), segment([5.,0.,0.], [5.,5.,0.]), segment([5.,5.,0.], [0.,0.005,0.])], reversed };
            let mut entity = Entity::new("near seam", Geometry::Curve(curve), None);
            entity.placement = Transform::scale([10.,0.1,1.]).unwrap();
            let id = entity.id;
            let mut doc = Document::with_template_tolerance("grip world tolerance"); doc.entities.insert(id, entity);
            let mut c = Controller::new(doc, Limits::default()).unwrap();
            select(&mut c, &[id]); c.top11_invoke(action).unwrap();
            let handle = c.top11_grip_handles().unwrap()[0];
            let target = [-10.,-0.1,2.]; let before = c.document().clone();
            c.top11_move_grip(handle, target).unwrap(); let after = c.document().clone();
            let Geometry::Curve(curve) = &after.entities[&id].geometry else { panic!("lost curve") };
            let ends = om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap();
            assert!((0..3).all(|i| (ends[0][i] - ends[1][i]).abs() < 1e-9));
            assert!((0..3).all(|i| (ends[0][i] - [-1.,-1.,2.][i]).abs() < 1e-9));
            c.undo().unwrap(); assert_eq!(c.document(), &before); c.redo().unwrap(); assert_eq!(c.document(), &after);
        }
    }
}

#[test]
fn control_edit_fuses_smooth_line_segments_and_undo_restores_polycurve() {
    use om9_core::geometry::Curve;
    for reversed in [false, true] {
        let segment = |start, end| Curve::Line { start, end, domain: [0.,5.] };
        let curve = Curve::Polycurve { segments: vec![segment([0.,0.,0.], [5.,0.,0.]), segment([5.,0.,0.], [10.,0.,0.])], reversed };
        let entity = Entity::new("smooth", Geometry::Curve(curve), None); let id = entity.id;
        let mut doc = Document::with_template_tolerance("smooth fuse"); doc.entities.insert(id, entity);
        let mut c = Controller::new(doc, Limits::default()).unwrap(); select(&mut c, &[id]); c.submit("PointsOn").unwrap();
        let handle = c.top11_grip_handles().unwrap()[1]; let before = c.document().clone();
        c.top11_move_grip(handle, [5.,2.,0.]).unwrap(); let after = c.document().clone();
        let Geometry::Curve(curve @ Curve::Nurbs(_)) = &after.entities[&id].geometry else { panic!("smooth polycurve not fused") };
        let ends = om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap();
        assert_eq!(ends, if reversed { [[10.,0.,0.], [0.,0.,0.]] } else { [[0.,0.,0.], [10.,0.,0.]] });
        let mid = om9_core::curve_ops::point_at(curve, 0.5, &Limits::default()).unwrap();
        assert!((0..3).all(|i| (mid[i] - [5.,2.,0.][i]).abs() < 1e-9));
        assert_eq!(om9_core::curve_ops::explode_curve(curve, &Limits::default()).unwrap().len(), 1);
        c.cancel();
        assert!(c.top11_invoke(Top11Action::Explode).is_err());
        assert_eq!(c.document(), &after);
        c.undo().unwrap(); assert_eq!(c.document(), &before); c.redo().unwrap(); assert_eq!(c.document(), &after);
    }
}

#[test]
fn control_edit_fuses_smooth_line_and_arc_without_changing_arc() {
    use om9_core::geometry::Curve;
    let line = Curve::Line { start: [0.,0.,0.], end: [5.,0.,0.], domain: [0.,5.] };
    let arc = Curve::Circle { center: [5.,2.,0.], x: [1.,0.,0.], y: [0.,1.,0.], radius: 2., domain: [0.,1.], angle_domain: Some([-std::f64::consts::FRAC_PI_2, 0.]) };
    let curve = Curve::Polycurve { segments: vec![line, arc], reversed: false };
    let entity = Entity::new("smooth arc", Geometry::Curve(curve), None); let id = entity.id;
    let mut doc = Document::with_template_tolerance("smooth fuse mixed"); doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap(); select(&mut c, &[id]); c.top11_invoke(Top11Action::ControlPoints).unwrap();
    let handle = c.top11_grip_handles().unwrap()[0]; c.top11_move_grip(handle, [-1.,0.,0.]).unwrap();
    let Geometry::Curve(curve @ Curve::Nurbs(n)) = &c.document().entities[&id].geometry else { panic!("smooth mixed polycurve not fused") };
    assert_eq!(n.degree, 2);
    let limits = Limits::default();
    for f in [0.5, 0.6, 0.75, 0.9, 1.] {
        let p = om9_core::curve_ops::point_at(curve, f, &limits).unwrap();
        assert!(((p[0]-5.).hypot(p[1]-2.) - 2.).abs() < 1e-9, "{p:?}");
    }
    assert_eq!(om9_core::curve_ops::curve_endpoints(curve, &limits).unwrap(), [[-1.,0.,0.], [7.,2.,0.]]);
}

#[test]
fn control_edit_fuses_rational_multispan_with_affine_placement() {
    use om9_core::geometry::{Curve, NurbsCurve};
    use om9_core::curve_ops::point_at;
    let limits = Limits::default();
    for reversed in [false, true] {
        let mut n = NurbsCurve { degree: 2, poles: vec![[0.,0.,0.],[2.,0.,0.],[4.,0.,0.],[6.,0.,0.]], weights: vec![1.,0.7,1.6,1.2], knots: vec![2.,2.,2.,4.,7.,7.,7.], domain: [2.,7.], periodic: false, reversed: false, seam: None };
        if reversed { n.poles.reverse(); n.weights.reverse(); n.knots = n.knots.iter().rev().map(|k| 9. - k).collect(); n.reversed = true; }
        let line = |end| Curve::Line { start: [6.,0.,0.], end, domain: [0.,4.] };
        let curve = Curve::Polycurve { segments: vec![Curve::Nurbs(n.clone()), line([10.,0.,0.])], reversed: false };
        let mut entity = Entity::new("rational smooth", Geometry::Curve(curve), None);
        entity.placement = Transform([[3.,0.4,0.,2.],[0.,0.5,0.,-1.],[0.,0.,2.,3.],[0.,0.,0.,1.]]);
        let placement = entity.placement; let id = entity.id;
        let mut doc = Document::with_template_tolerance("rational smooth fuse"); doc.entities.insert(id, entity);
        let mut c = Controller::new(doc, limits).unwrap(); select(&mut c, &[id]); c.submit("PointsOn").unwrap();
        let handles = c.top11_grip_handles().unwrap(); let handle = *handles.last().unwrap();
        let target = placement.point([11.,0.,0.]).unwrap();
        let before = c.document().clone(); c.top11_move_grip(handle, target).unwrap(); let after = c.document().clone();
        let Geometry::Curve(actual @ Curve::Nurbs(_)) = &after.entities[&id].geometry else { panic!("not fused") };
        let expected = Curve::Polycurve { segments: vec![Curve::Nurbs(n), line([11.,0.,0.])], reversed: false };
        for i in 0..=100 {
            let f = i as f64 / 100.;
            let a = placement.point(point_at(actual, f, &limits).unwrap()).unwrap();
            let b = placement.point(point_at(&expected, f, &limits).unwrap()).unwrap();
            assert!((0..3).all(|j| (a[j]-b[j]).abs() < 1e-8), "fraction {f}: {a:?} != {b:?}");
        }
        let path = std::env::temp_dir().join(format!("om9-fused-nurbs-{}.om9", uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&path, &after, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
        let loaded = om9_archive::Archive::open(&path, &limits).unwrap();
        std::fs::remove_file(path).unwrap(); assert_eq!(loaded, after);
        c.undo().unwrap(); assert_eq!(c.document(), &before); c.redo().unwrap(); assert_eq!(c.document(), &after);
    }
}

#[test]
fn control_fusion_elevates_multispan_rational_linear_piece_exactly() {
    use om9_core::geometry::{Curve, NurbsCurve};
    use om9_core::curve_ops::point_at;
    let limits = Limits::default();
    let first = NurbsCurve { degree: 1, poles: vec![[0.,0.,0.],[2.,0.,0.],[4.,0.,0.],[6.,0.,0.]], weights: vec![1.,0.8,1.7,1.2], knots: vec![0.,0.,0.2,0.7,1.,1.], domain: [0.,1.], periodic: false, reversed: false, seam: None };
    let second = NurbsCurve { degree: 2, poles: vec![[6.,0.,0.],[8.,0.,0.],[10.,0.,0.]], weights: vec![0.6,1.3,1.], knots: vec![0.,0.,0.,1.,1.,1.], domain: [0.,1.], periodic: false, reversed: false, seam: None };
    let source = Curve::Polycurve { segments: vec![Curve::Nurbs(first.clone()), Curve::Nurbs(second.clone())], reversed: false };
    let entity = Entity::new("multi span", Geometry::Curve(source), None); let id = entity.id;
    let mut doc = Document::with_template_tolerance("fuse elevation"); doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, limits).unwrap(); select(&mut c, &[id]); c.submit("PointsOn").unwrap();
    let handle = *c.top11_grip_handles().unwrap().last().unwrap(); c.top11_move_grip(handle, [11.,0.,0.]).unwrap();
    let Geometry::Curve(actual @ Curve::Nurbs(n)) = &c.document().entities[&id].geometry else { panic!("not fused") };
    assert_eq!(n.degree, 2);
    for i in 0..=100 {
        let f = i as f64 / 100.;
        let actual = point_at(actual, f * 0.5, &limits).unwrap();
        let expected = point_at(&Curve::Nurbs(first.clone()), f, &limits).unwrap();
        assert!((0..3).all(|j| (actual[j]-expected[j]).abs() < 1e-8), "fraction {f}: {actual:?} != {expected:?}");
    }
}

#[test]
fn delete_nurbs_control_point_is_undoable_and_rejects_stale_handles() {
    use om9_core::geometry::{Curve, NurbsCurve};
    let n = NurbsCurve { degree: 2, poles: vec![[0.,0.,0.],[1.,2.,0.],[3.,2.,0.],[4.,0.,0.]], weights: vec![1.,0.7,1.4,1.], knots: vec![0.,0.,0.,0.5,1.,1.,1.], domain: [0.,1.], periodic: false, reversed: true, seam: None };
    let entity = Entity::new("curve", Geometry::Curve(Curve::Nurbs(n.clone())), None); let id = entity.id;
    let mut doc = Document::with_template_tolerance("delete grip"); doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap(); select(&mut c, &[id]); c.submit("PointsOn").unwrap();
    let handle = c.top11_grip_handles().unwrap()[1]; let before = c.document().clone();
    c.top11_delete_grip(handle).unwrap(); let after = c.document().clone();
    let Geometry::Curve(Curve::Nurbs(result)) = &after.entities[&id].geometry else { panic!("lost curve") };
    assert_eq!(result.poles, vec![n.poles[0],n.poles[2],n.poles[3]]); assert_eq!(result.weights, vec![1.,1.4,1.]);
    assert_eq!(result.knots, vec![0.,0.,0.,1.,1.,1.]); assert_eq!(result.domain, n.domain); assert!(result.reversed);
    assert!(c.top11_delete_grip(handle).is_err()); assert_eq!(c.document(), &after);
    c.undo().unwrap(); assert_eq!(c.document(), &before); c.redo().unwrap(); assert_eq!(c.document(), &after);
    let current = c.top11_grip_handles().unwrap()[1]; assert!(c.top11_delete_grip(current).is_err()); assert_eq!(c.document(), &after);
}

#[test]
fn unsupported_control_grip_deletion_keeps_each_entity() {
    use om9_core::geometry::Curve;
    let limits = Limits::default();
    let surface = om9_core::surface_kernel::plane([0.,0.,0.], [1.,0.,0.], [0.,1.,0.], 5., 5., &limits).unwrap();
    for (kind, geometry) in [Geometry::Surface(surface), Geometry::Curve(Curve::Line { start: [0.,0.,0.], end: [5.,0.,0.], domain: [0.,5.] })].into_iter().enumerate() {
        let entity = Entity::new("unsupported", geometry, None); let id = entity.id;
        let mut doc = Document::with_template_tolerance("unsupported delete"); doc.entities.insert(id, entity);
        let mut c = Controller::new(doc, limits).unwrap(); c.select_objects(&[id]).unwrap_or_else(|e| panic!("kind {kind}: {e}")); c.submit("PointsOn").unwrap();
        let handle = c.top11_grip_handles().unwrap()[0]; let before = c.document().clone();
        assert!(c.top11_delete_grip(handle).is_err()); assert_eq!(c.document(), &before);
    }
}

#[test]
fn delete_multiple_control_grips_is_atomic_across_curves() {
    use om9_core::geometry::{Curve, NurbsCurve};
    let make = |offset: f64| NurbsCurve { degree: 2, poles: vec![[offset,0.,0.],[offset+1.,2.,0.],[offset+2.,1.,0.],[offset+3.,2.,0.],[offset+4.,0.,0.]], weights: vec![1.,0.8,1.2,1.4,1.], knots: vec![0.,0.,0.,0.3,0.7,1.,1.,1.], domain: [0.,1.], periodic: false, reversed: false, seam: None };
    let a = Entity::new("a", Geometry::Curve(Curve::Nurbs(make(0.))), None);
    let b = Entity::new("b", Geometry::Curve(Curve::Nurbs(make(10.))), None);
    let ids = [a.id, b.id]; let mut doc = Document::with_template_tolerance("multi delete"); doc.entities.insert(a.id,a); doc.entities.insert(b.id,b);
    let mut c = Controller::new(doc, Limits::default()).unwrap(); select(&mut c, &ids); c.submit("PointsOn").unwrap();
    let handles = c.top11_grip_handles().unwrap();
    let find = |id, index| *handles.iter().find(|h| h.object == id && h.index == index).unwrap();
    let selected = [find(ids[0],1), find(ids[0],3), find(ids[1],2)]; let before = c.document().clone();
    assert!(c.top11_delete_grips(&[selected[0],selected[0]]).is_err()); assert_eq!(c.document(), &before);
    c.top11_delete_grips(&selected).unwrap(); let after = c.document().clone();
    let Geometry::Curve(Curve::Nurbs(a)) = &after.entities[&ids[0]].geometry else { panic!("a lost") };
    let Geometry::Curve(Curve::Nurbs(b)) = &after.entities[&ids[1]].geometry else { panic!("b lost") };
    assert_eq!(a.poles, vec![make(0.).poles[0], make(0.).poles[2], make(0.).poles[4]]);
    assert_eq!(b.poles.len(), 4);
    let path = std::env::temp_dir().join(format!("om9-multi-grip-delete-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &after, &Limits::default(), &om9_core::transaction::Cancellation::default()).unwrap();
    let loaded = om9_archive::Archive::open(&path, &Limits::default()).unwrap(); std::fs::remove_file(path).unwrap(); assert_eq!(loaded, after);
    c.undo().unwrap(); assert_eq!(c.document(), &before); c.redo().unwrap(); assert_eq!(c.document(), &after);
    let handles = c.top11_grip_handles().unwrap();
    let invalid: Vec<_> = handles.iter().filter(|h| (h.object == ids[0] || h.object == ids[1]) && h.index == 1).copied().collect();
    assert!(c.top11_delete_grips(&invalid).is_err()); assert_eq!(c.document(), &after);
}

#[test]
fn delete_surface_control_row_and_curve_point_is_one_atomic_edit() {
    use om9_core::geometry::{Curve, NurbsCurve};
    let limits = Limits::default();
    let surface = NurbsSurface { degree: [2,2], size: [4,4], poles: (0..4).flat_map(|u| (0..4).map(move |v| [u as f64,v as f64,(u*v) as f64*0.1])).collect(), weights: (0..16).map(|i| 1. + i as f64*0.05).collect(), knots: [vec![0.,0.,0.,0.5,1.,1.,1.],vec![0.,0.,0.,0.5,1.,1.,1.]], domain: [[0.,1.],[0.,1.]], periodic: [false,false], reversed: false };
    let curve = NurbsCurve { degree: 2, poles: vec![[10.,0.,0.],[11.,2.,0.],[13.,2.,0.],[14.,0.,0.]], weights: vec![1.;4], knots: vec![0.,0.,0.,0.5,1.,1.,1.], domain: [0.,1.], periodic: false, reversed: false, seam: None };
    let a = Entity::new("surface",Geometry::Surface(surface.clone()),None); let b = Entity::new("curve",Geometry::Curve(Curve::Nurbs(curve)),None); let ids = [a.id,b.id];
    let mut doc = Document::with_template_tolerance("surface control delete"); doc.entities.insert(a.id,a); doc.entities.insert(b.id,b);
    let mut c = Controller::new(doc,limits).unwrap(); select(&mut c,&ids); c.submit("PointsOn").unwrap();
    let handles = c.top11_grip_handles().unwrap();
    let find = |id,index| *handles.iter().find(|h| h.object == id && h.index == index).unwrap();
    let before = c.document().clone();
    assert!(c.top11_delete_grips(&[find(ids[0],5),find(ids[1],1)]).is_err()); assert_eq!(c.document(),&before);
    let selected = [find(ids[0],4),find(ids[0],5),find(ids[0],6),find(ids[0],7),find(ids[1],1)];
    c.top11_delete_grips(&selected).unwrap(); let after = c.document().clone();
    let Geometry::Surface(result) = &after.entities[&ids[0]].geometry else { panic!("surface lost") };
    assert_eq!(result.size,[3,4]); assert_eq!(result.poles, surface.poles[..4].iter().chain(&surface.poles[8..]).copied().collect::<Vec<_>>());
    let Geometry::Curve(Curve::Nurbs(result)) = &after.entities[&ids[1]].geometry else { panic!("curve lost") }; assert_eq!(result.poles.len(),3);
    let path = std::env::temp_dir().join(format!("om9-surface-grip-delete-{}.om9",uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path,&after,&limits,&om9_core::transaction::Cancellation::default()).unwrap(); let loaded = om9_archive::Archive::open(&path,&limits).unwrap(); std::fs::remove_file(path).unwrap(); assert_eq!(loaded,after);
    c.undo().unwrap(); assert_eq!(c.document(),&before); c.redo().unwrap(); assert_eq!(c.document(),&after);
}

#[test]
fn duplicate_invalid_reused_distance_keeps_prompt_and_can_retry() {
    for via_toolbar in [false, true] {
        let mut c = controller(); let id = draw_line(&mut c, [0.;3], [1.,0.,0.]); select(&mut c, &[id]);
        if via_toolbar { c.top11_invoke(Top11Action::Duplicate).unwrap(); c.submit("UseLastDistance=Yes").unwrap(); }
        else { c.submit("Copy UseLastDistance=Yes").unwrap(); }
        c.accept_world_point([0.;3]).unwrap(); c.accept_world_point([10.,0.,0.]).unwrap();
        let before = c.document().clone(); let undo = c.coordinator().undo_len();
        assert!(c.accept_world_point([0.;3]).is_err()); assert_eq!(c.document(), &before); assert_eq!(c.coordinator().undo_len(), undo);
        assert!(c.is_running()); assert_eq!(c.top11_prompt(), Some("pick a destination point (Enter to finish)"));
        c.accept_world_point([0.,5.,0.]).unwrap(); assert_eq!(c.document().entities.len(),3);
        assert!(c.document().entities.values().any(|entity| (entity.placement.point([0.;3]).unwrap()[1]-10.).abs()<1e-9));
        c.submit("").unwrap(); c.undo().unwrap(); assert_eq!(c.document(), &before);
    }
}

#[test]
fn duplicate_budget_failure_keeps_prompt_for_retry() {
    use om9_core::geometry::Curve;
    let limits = Limits { max_objects: 4, ..Limits::default() };
    let entity = Entity::new("source",Geometry::Curve(Curve::Line { start: [0.,0.,0.], end: [1.,0.,0.], domain: [0.,1.] }),None); let id=entity.id;
    let mut doc = Document::with_template_tolerance("duplicate budget retry"); doc.entities.insert(id,entity);
    let mut c = Controller::new(doc,limits).unwrap(); select(&mut c,&[id]); c.submit("Copy").unwrap();
    c.accept_world_point([0.;3]).unwrap(); c.accept_world_point([2.,0.,0.]).unwrap(); let before=c.document().clone(); let undo=c.coordinator().undo_len();
    assert!(c.accept_world_point([4.,0.,0.]).is_err()); assert_eq!(c.document(),&before); assert_eq!(c.coordinator().undo_len(),undo); assert!(c.is_running());
    c.submit("").unwrap(); c.undo().unwrap(); assert_eq!(c.document().entities.len(),1);
}

#[test]
fn mirror_and_rotate_reject_degenerate_points_without_losing_prompt() {
    for via_toolbar in [false,true] {
        let mut c=controller(); let id=draw_line(&mut c,[1.,0.,0.],[2.,0.,0.]); select(&mut c,&[id]);
        if via_toolbar { c.top11_invoke(Top11Action::Mirror).unwrap(); } else { c.submit("Mirror").unwrap(); }
        c.accept_world_point([0.;3]).unwrap(); let before=c.document().clone();
        assert!(c.accept_world_point([0.;3]).is_err()); assert_eq!(c.document(),&before); assert!(c.is_running());
        c.accept_world_point([0.,2.,0.]).unwrap(); assert_eq!(c.document().entities.len(),2);
        c.undo().unwrap(); assert_eq!(c.document(),&before); select(&mut c,&[id]);
        if via_toolbar { c.top11_invoke(Top11Action::Rotate).unwrap(); } else { c.submit("Rotate").unwrap(); }
        c.accept_world_point([0.;3]).unwrap();
        assert!(c.accept_world_point([0.;3]).is_err()); assert!(c.is_running());
        c.accept_world_point([1.,0.,0.]).unwrap();
        assert!(c.accept_world_point([0.;3]).is_err()); assert!(c.is_running()); assert_eq!(c.document(),&before);
        c.accept_world_point([0.,1.,0.]).unwrap();
        let entity=&c.document().entities[&id]; let point=entity.placement.point([1.,0.,0.]).unwrap();
        assert!(point[0].abs()<1e-9 && (point[1]-1.).abs()<1e-9,"{point:?}");
        c.undo().unwrap(); assert_eq!(c.document(),&before);
    }
}

#[test]
fn rotate_reference_points_use_cplane_projection() {
    let mut c=controller(); let id=draw_line(&mut c,[1.,0.,0.],[2.,0.,0.]); select(&mut c,&[id]); c.submit("Rotate").unwrap();
    c.accept_world_point([0.;3]).unwrap(); c.accept_world_point([1.,0.,10.]).unwrap(); c.accept_world_point([0.,1.,10.]).unwrap();
    let point=c.document().entities[&id].placement.point([1.,0.,0.]).unwrap();
    assert!(point[0].abs()<1e-9 && (point[1]-1.).abs()<1e-9,"{point:?}");
}

#[test]
fn mirror_three_point_collinear_retry_and_rotation_axial_reference() {
    let mut c=controller(); let id=draw_line(&mut c,[1.,0.,0.],[2.,0.,0.]); select(&mut c,&[id]); c.submit("Mirror").unwrap(); c.submit("3Point").unwrap();
    c.accept_world_point([0.;3]).unwrap(); c.accept_world_point([1.,0.,0.]).unwrap(); let before=c.document().clone();
    assert!(c.accept_world_point([2.,0.,0.]).is_err()); assert!(c.is_running()); assert_eq!(c.document(),&before);
    c.accept_world_point([0.,1.,0.]).unwrap(); assert_eq!(c.document().entities.len(),2); c.undo().unwrap(); assert_eq!(c.document(),&before); select(&mut c,&[id]);
    c.submit("Rotate").unwrap(); c.accept_world_point([0.;3]).unwrap(); assert!(c.accept_world_point([0.,0.,10.]).is_err()); assert!(c.is_running());
    c.accept_world_point([1.,0.,0.]).unwrap(); assert!(c.accept_world_point([0.,0.,5.]).is_err()); assert!(c.is_running());
    c.accept_world_point([0.,-1.,3.]).unwrap(); let point=c.document().entities[&id].placement.point([1.,0.,0.]).unwrap();
    assert!(point[0].abs()<1e-9 && (point[1]+1.).abs()<1e-9,"{point:?}"); c.undo().unwrap(); assert_eq!(c.document(),&before);
}

#[test]
fn rotate_reference_projection_respects_tilted_cplane_axis() {
    use om9_scene::camera::CPlane;
    let mut c=controller(); let id=draw_line(&mut c,[0.,1.,0.],[0.,2.,0.]); select(&mut c,&[id]);
    c.set_cplane(CPlane::new([0.;3],[0.,1.,0.],[0.,0.,1.]).unwrap()).unwrap(); c.submit("Rotate").unwrap();
    c.accept_world_point([0.;3]).unwrap(); c.accept_world_point([10.,1.,0.]).unwrap(); c.accept_world_point([10.,0.,1.]).unwrap();
    let point=c.document().entities[&id].placement.point([0.,1.,0.]).unwrap();
    assert!(point[0].abs()<1e-9 && point[1].abs()<1e-9 && (point[2]-1.).abs()<1e-9,"{point:?}");
}

#[test]
fn move_overflow_does_not_discard_base_point() {
    let mut c=controller(); let id=draw_line(&mut c,[0.;3],[1.,0.,0.]); select(&mut c,&[id]); c.submit("Move").unwrap();
    c.accept_world_point([-1e308,0.,0.]).unwrap(); let before=c.document().clone();
    assert!(c.accept_world_point([1e308,0.,0.]).is_err()); assert!(c.is_running()); assert_eq!(c.document(),&before);
    c.accept_world_point([-1e308,1.,0.]).unwrap(); let point=c.document().entities[&id].placement.point([0.;3]).unwrap();
    assert_eq!(point,[0.,1.,0.]); c.undo().unwrap(); assert_eq!(c.document(),&before);
}

#[test]
fn join_reversed_lines_retains_each_parameter_domain() {
    use om9_core::geometry::Curve;
    for toolbar in [false,true] {
        let a=Entity::new("a",Geometry::Curve(Curve::Line{start:[0.,0.,0.],end:[1.,0.,0.],domain:[2.,5.]}),None);
        let b=Entity::new("b",Geometry::Curve(Curve::Line{start:[2.,0.,0.],end:[1.,0.,0.],domain:[7.,12.]}),None);
        let ids=[a.id,b.id];let mut doc=Document::with_template_tolerance("join reversed domains");doc.entities.insert(a.id,a);doc.entities.insert(b.id,b);
        let mut c=Controller::new(doc,Limits::default()).unwrap();select(&mut c,&ids);let before=c.document().clone();
        if toolbar {c.top11_invoke(Top11Action::Join).unwrap();}else{c.submit("Join").unwrap();}
        let after=c.document().clone();let joined=after.entities.values().next().unwrap();
        let Geometry::Curve(Curve::Polycurve{segments,..})=&joined.geometry else {panic!("polycurve")};
        let mut domains:Vec<_>=segments.iter().map(|segment|match segment {Curve::Line{domain,..}=>*domain,_=>panic!("line")}).collect();
        domains.sort_by(|a,b|a[0].total_cmp(&b[0]));assert_eq!(domains,vec![[2.,5.],[7.,12.]]);
        let ends=om9_core::curve_ops::curve_endpoints(match &joined.geometry {Geometry::Curve(c)=>c,_=>unreachable!()},&Limits::default()).unwrap();
        assert!(ends.contains(&[0.,0.,0.])&&ends.contains(&[2.,0.,0.]));
        let path=std::env::temp_dir().join(format!("om9-join-domains-{}.om9",uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&path,&after,&Limits::default(),&om9_core::transaction::Cancellation::default()).unwrap();
        assert_eq!(om9_archive::Archive::open(&path,&Limits::default()).unwrap(),after);std::fs::remove_file(path).unwrap();
        c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }
}

#[test]
fn join_curves_uses_world_placement_and_document_tolerance() {
    use om9_core::geometry::Curve;
    for (gap, succeeds, toolbar) in [(0.0005,true,false),(0.0005,true,true),(0.0015,false,false)] {
        let line = |end| Geometry::Curve(Curve::Line { start: [0.,0.,0.], end, domain: [0.,1.] });
        let mut a=Entity::new("a",line([5.,0.,0.]),None); a.placement=Transform::scale([2.,1.,1.]).unwrap();
        let mut b=Entity::new("b",line([0.,5.,0.]),None); b.placement=Transform::translation([10.+gap,0.,0.]);
        let ids=[a.id,b.id]; let mut doc=Document::with_template_tolerance("join placed curves"); doc.entities.insert(a.id,a); doc.entities.insert(b.id,b);
        let mut c=Controller::new(doc,Limits::default()).unwrap(); select(&mut c,&ids); let before=c.document().clone();
        let result=if toolbar { c.top11_invoke(Top11Action::Join).map(|_| ()) } else { c.submit("Join").map(|_| ()) };
        if !succeeds { assert!(result.is_err()); assert_eq!(c.document(),&before); assert_eq!(c.coordinator().undo_len(),0); continue; }
        result.unwrap(); assert_eq!(c.document().entities.len(),1); let after=c.document().clone(); let joined=after.entities.values().next().unwrap();
        let Geometry::Curve(curve) = &joined.geometry else { panic!("curve") };
        let Curve::Polycurve{segments,..}=curve else {panic!("polycurve")};
        assert!(segments.iter().all(|segment| matches!(segment,Curve::Line{domain,..} if *domain==[0.,1.])));
        let ends=om9_core::curve_ops::curve_endpoints(curve,&Limits::default()).unwrap().map(|p| joined.placement.point(p).unwrap());
        assert!(ends.iter().any(|p| p[0].abs()<1e-9 && p[1].abs()<1e-9));
        assert!(ends.iter().any(|p| (p[0]-10.-gap).abs()<1e-9 && (p[1]-5.).abs()<1e-9));
        let path=std::env::temp_dir().join(format!("om9-join-world-{}.om9",uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&path,&after,&Limits::default(),&om9_core::transaction::Cancellation::default()).unwrap(); let loaded=om9_archive::Archive::open(&path,&Limits::default()).unwrap(); std::fs::remove_file(path).unwrap(); assert_eq!(loaded,after);
        c.undo().unwrap(); assert_eq!(c.document(),&before); c.redo().unwrap(); assert_eq!(c.document(),&after);
    }
}

#[test]
fn join_nonuniformly_placed_arc_preserves_world_ellipse() {
    use om9_core::geometry::Curve;
    let line=Entity::new("line",Geometry::Curve(Curve::Line { start: [0.,0.,0.], end: [10.,0.,0.], domain: [0.,1.] }),None);
    let mut arc=Entity::new("arc",Geometry::Curve(Curve::Circle { center: [0.,1.,0.], x: [1.,0.,0.], y: [0.,1.,0.], radius: 1., domain: [0.,1.], angle_domain: Some([-std::f64::consts::FRAC_PI_2,0.]) }),None);
    arc.placement=Transform([[2.,0.,0.,10.],[0.,3.,0.,0.],[0.,0.,1.,0.],[0.,0.,0.,1.]]);
    let ids=[line.id,arc.id]; let mut doc=Document::with_template_tolerance("join affine arc"); doc.entities.insert(line.id,line); doc.entities.insert(arc.id,arc);
    let mut c=Controller::new(doc,Limits::default()).unwrap(); select(&mut c,&ids); let before=c.document().clone(); c.submit("Join").unwrap();
    let joined=c.document().entities.values().next().unwrap(); let Geometry::Curve(Curve::Polycurve { segments, .. })=&joined.geometry else { panic!("polycurve") };
    let arc=segments.iter().find(|segment| matches!(segment,Curve::Nurbs(_))).expect("affine arc");
    for step in 0..=16 { let p=joined.placement.point(om9_core::curve_ops::point_at(arc,step as f64/16.,&Limits::default()).unwrap()).unwrap();
        let radius=((p[0]-10.)/2.).hypot(p[1]/3.-1.); assert!((radius-1.).abs()<1e-9,"{p:?}"); }
    c.undo().unwrap(); assert_eq!(c.document(),&before);
}

#[test]
fn join_curves_without_document_tolerance_is_refused_without_mutation() {
    use om9_core::geometry::Curve;
    let line=|a,b| Entity::new("line",Geometry::Curve(Curve::Line { start: a,end: b,domain: [0.,1.] }),None);
    let a=line([0.,0.,0.],[1.,0.,0.]); let b=line([1.,0.,0.],[2.,0.,0.]); let ids=[a.id,b.id];
    let mut doc=Document::new("no tolerance"); doc.entities.insert(a.id,a); doc.entities.insert(b.id,b);
    let mut c=Controller::new(doc,Limits::default()).unwrap(); select(&mut c,&ids); let before=c.document().clone();
    assert!(c.submit("Join").is_err()); assert_eq!(c.document(),&before); assert_eq!(c.coordinator().undo_len(),0);
}

#[test]
fn join_connected_loop_and_tail_independent_of_object_order() {
    use om9_core::geometry::Curve;
    let mut entities: Vec<_> = (0..5).map(|_| Entity::new("segment",Geometry::Curve(Curve::Line { start: [0.,0.,0.], end: [1.,0.,0.], domain: [0.,1.] }),None)).collect();
    entities.sort_by_key(|e| e.id);
    let ends = [([0.,0.,0.],[1.,0.,0.]),([1.,0.,0.],[2.,0.,0.]),([1.,0.,0.],[1.,1.,0.]),([1.,1.,0.],[0.,1.,0.]),([0.,1.,0.],[1.,0.,0.])];
    for (e,(start,end)) in entities.iter_mut().zip(ends) { e.geometry=Geometry::Curve(Curve::Line {start,end,domain:[0.,1.]}); }
    let ids: Vec<_>=entities.iter().map(|e|e.id).collect(); let mut doc=Document::with_template_tolerance("join loop and tail"); for e in entities { doc.entities.insert(e.id,e); }
    let mut c=Controller::new(doc,Limits::default()).unwrap(); select(&mut c,&ids); let before=c.document().clone(); c.submit("Join").unwrap();
    let joined=c.document().entities.values().next().unwrap(); let Geometry::Curve(Curve::Polycurve {segments,..})=&joined.geometry else { panic!("polycurve") }; assert_eq!(segments.len(),5);
    for pair in segments.windows(2) { let a=om9_core::curve_ops::curve_endpoints(&pair[0],&Limits::default()).unwrap()[1]; let b=om9_core::curve_ops::curve_endpoints(&pair[1],&Limits::default()).unwrap()[0]; assert_eq!(a,b); }
    let after=c.document().clone(); c.undo().unwrap(); assert_eq!(c.document(),&before); c.redo().unwrap(); assert_eq!(c.document(),&after);
}

#[test]
fn join_curves_closes_toleranced_gap_at_world_midpoint() {
    use om9_core::geometry::Curve;
    for via_toolbar in [false,true] {
        let mut a=Entity::new("a",Geometry::Curve(Curve::Line {start:[0.,0.,0.],end:[5.,0.,0.],domain:[0.,1.]}),None); a.placement=Transform::scale([2.,1.,1.]).unwrap();
        let mut b=Entity::new("b",Geometry::Curve(Curve::Line {start:[0.,0.,0.],end:[0.,5.,0.],domain:[0.,1.]}),None); b.placement=Transform::translation([10.0008,0.,0.]);
        let ids=[a.id,b.id]; let mut doc=Document::with_template_tolerance("join midpoint");doc.entities.insert(a.id,a);doc.entities.insert(b.id,b);
        let mut c=Controller::new(doc,Limits::default()).unwrap();select(&mut c,&ids);let before=c.document().clone();
        if via_toolbar {c.top11_invoke(Top11Action::Join).unwrap();}else{c.submit("Join").unwrap();}
        let after=c.document().clone();let joined=after.entities.values().next().unwrap();let Geometry::Curve(Curve::Polycurve {segments,..})=&joined.geometry else {panic!("polycurve")};assert_eq!(segments.len(),2);
        let end=joined.placement.point(om9_core::curve_ops::curve_endpoints(&segments[0],&Limits::default()).unwrap()[1]).unwrap();
        let start=joined.placement.point(om9_core::curve_ops::curve_endpoints(&segments[1],&Limits::default()).unwrap()[0]).unwrap();
        assert!((0..3).all(|i|(end[i]-start[i]).abs()<1e-12),"{end:?} {start:?}");assert!((end[0]-10.0004).abs()<1e-12,"{end:?}");
        c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }
}

#[test]
fn join_toleranced_closed_loop_heals_the_closing_midpoint() {
    use om9_core::geometry::Curve;
    let ends=[([0.,0.,0.],[10.,0.,0.]),([10.,0.,0.],[10.,10.,0.]),([10.,10.,0.],[0.,10.,0.]),([0.,10.,0.],[0.,0.0008,0.])];
    let mut doc=Document::with_template_tolerance("close join gap"); let mut ids=Vec::new();
    for (start,end) in ends {let e=Entity::new("edge",Geometry::Curve(Curve::Line {start,end,domain:[0.,1.]}),None);ids.push(e.id);doc.entities.insert(e.id,e);}
    let mut c=Controller::new(doc,Limits::default()).unwrap();select(&mut c,&ids);let before=c.document().clone();c.submit("Join").unwrap();
    let Geometry::Curve(Curve::Polycurve {segments,..})=&c.document().entities.values().next().unwrap().geometry else{panic!("polycurve")};
    for i in 0..segments.len(){let end=om9_core::curve_ops::curve_endpoints(&segments[i],&Limits::default()).unwrap()[1];let start=om9_core::curve_ops::curve_endpoints(&segments[(i+1)%segments.len()],&Limits::default()).unwrap()[0];assert!((0..3).all(|j|(end[j]-start[j]).abs()<1e-12));}
    assert!(segments.iter().flat_map(|s|om9_core::curve_ops::curve_endpoints(s,&Limits::default()).unwrap()).any(|p| p[0].abs()<1e-12 && (p[1]-0.0004).abs()<1e-12));
    c.undo().unwrap();assert_eq!(c.document(),&before);
}

#[test]
fn join_heals_rational_nurbs_and_arc_endpoints() {
    use om9_core::geometry::{Curve,NurbsCurve};
    for reversed in [false,true] {
        let mut n=NurbsCurve {degree:3,poles:vec![[0.,0.,0.],[1.,1.,0.],[3.,1.,0.],[5.,0.,0.]],weights:vec![1.,0.8,1.2,1.],knots:vec![0.,0.,0.,0.,1.,1.,1.,1.],domain:[0.,1.],periodic:false,reversed:false,seam:None};
        if reversed {n.poles.reverse();n.weights.reverse();n.reversed=true;}
        let curve=Entity::new("nurbs",Geometry::Curve(Curve::Nurbs(n.clone())),None);
        let arc=Entity::new("arc",Geometry::Curve(Curve::Circle {center:[5.0008,1.,0.],x:[1.,0.,0.],y:[0.,1.,0.],radius:1.,domain:[0.,1.],angle_domain:Some([-std::f64::consts::FRAC_PI_2,0.])}),None);
        let ids=[curve.id,arc.id];let mut doc=Document::with_template_tolerance("nurbs arc join");doc.entities.insert(curve.id,curve);doc.entities.insert(arc.id,arc);
        let mut c=Controller::new(doc,Limits::default()).unwrap();select(&mut c,&ids);let before=c.document().clone();c.submit("Join").unwrap();
        let Geometry::Curve(Curve::Polycurve {segments,..})=&c.document().entities.values().next().unwrap().geometry else{panic!("polycurve")};
        let a=om9_core::curve_ops::curve_endpoints(&segments[0],&Limits::default()).unwrap()[1];let b=om9_core::curve_ops::curve_endpoints(&segments[1],&Limits::default()).unwrap()[0];
        assert!((0..3).all(|i|(a[i]-b[i]).abs()<1e-10));assert!((a[0]-5.0004).abs()<1e-10,"{a:?}");
        let original=segments.iter().find_map(|s|match s {Curve::Nurbs(n) if n.degree==3=>Some(n),_=>None}).unwrap();
        assert_eq!(original.weights,n.weights);assert_eq!(original.knots,n.knots);
        for p in &n.poles {if (p[0]-5.).abs()>1e-9 {assert!(original.poles.contains(p));}}
        c.undo().unwrap();assert_eq!(c.document(),&before);
    }
}

#[test]
fn join_rational_nurbs_heals_gap_without_refitting() {
    use om9_core::geometry::{Curve,NurbsCurve};
    let limits=Limits::default();
    for (reversed, via_toolbar, backing) in [(false,false,0),(false,true,0),(true,false,0),(true,true,0),(false,false,1),(false,true,1),(true,false,1),(true,true,1),(false,false,2),(false,true,2),(true,false,2),(true,true,2),(false,false,3),(false,true,3),(true,false,3),(true,true,3),(false,false,4),(false,true,4),(true,false,4),(true,true,4),(false,false,5),(false,true,5),(true,false,5),(true,true,5)] {
        let n=if backing == 5 { NurbsCurve {degree:2,poles:vec![[0.,0.,0.],[1.,2.,0.],[2.,0.,0.],[10.,0.,0.],[11.,-1.,0.],[12.,0.,0.]],weights:vec![1.,0.7,1.2,0.8,1.1,1.],knots:vec![0.,0.,0.,0.5,0.5,0.5,1.,1.,1.],domain:[0.5,0.9],periodic:false,reversed,seam:None} } else if backing == 4 { NurbsCurve {degree:2,poles:vec![[0.,0.,0.],[2.,0.,0.],[2.,2.,0.],[0.,2.,0.],[0.,0.,0.],[2.,0.,0.]],weights:vec![1.,0.7,1.2,0.9,1.,0.7],knots:vec![-2.,-1.,0.,1.,2.,3.,4.,5.,6.],domain:[0.3,2.7],periodic:true,reversed,seam:None} } else if backing != 0 { NurbsCurve {degree:2,poles:vec![[0.,0.,0.],[1.,1.,0.],[2.,0.,0.],[3.,1.,0.]],weights:vec![1.,0.8,1.3,0.9],knots:vec![-1.,-0.5,0.,0.5,1.,1.5,2.],domain:if backing == 3 {[0.,1.]} else if backing == 2 { if reversed {[0.1,1.]} else {[0.,0.9]} } else {[0.1,0.9]},periodic:false,reversed,seam:None} } else { NurbsCurve {degree:2,poles:vec![[0.,0.,0.],[1.,1.,0.],[2.,0.,0.]],weights:vec![1.,0.8,1.3],knots:vec![0.,0.,0.,1.,1.,1.],domain:[0.1,0.9],periodic:false,reversed,seam:None} };
        let original=Curve::Nurbs(n.clone()); let end=om9_core::curve_ops::curve_endpoints(&original,&limits).unwrap()[1];
        let a=Entity::new("trimmed",Geometry::Curve(original.clone()),None);let b=Entity::new("line",Geometry::Curve(Curve::Line {start:[end[0]+0.0008,end[1],end[2]],end:[end[0]+3.,end[1],end[2]],domain:[0.,1.]}),None);
        let ids=[a.id,b.id];let mut doc=Document::with_template_tolerance("trimmed join");doc.entities.insert(a.id,a);doc.entities.insert(b.id,b);
        let mut c=Controller::new(doc,limits).unwrap();select(&mut c,&ids);let before=c.document().clone();if via_toolbar { c.top11_invoke(Top11Action::Join).unwrap(); } else { c.submit("Join").unwrap(); } let after=c.document().clone();
        let Geometry::Curve(Curve::Polycurve {segments,..})=&after.entities.values().next().unwrap().geometry else{panic!("polycurve")};
        let curve=segments.iter().find_map(|s|if let Curve::Nurbs(n)=s{Some(Curve::Nurbs(n.clone()))}else{None}).unwrap();
        let ends=om9_core::curve_ops::curve_endpoints(&curve,&limits).unwrap(); let new_end=ends.iter().min_by(|a,b|(a[0]-end[0]).abs().total_cmp(&(b[0]-end[0]).abs())).unwrap();
        assert!((new_end[0]-(end[0]+0.0004)).abs()<1e-9,"{new_end:?}");
        let forward=om9_core::curve_ops::point_at(&curve,0.,&limits).unwrap(); let ref_start=om9_core::curve_ops::point_at(&original,0.,&limits).unwrap();
        let same=(0..3).map(|i|(forward[i]-ref_start[i]).powi(2)).sum::<f64>().sqrt()<1e-8;
        for i in 0..=100 {let t=i as f64/100.;let a=om9_core::curve_ops::point_at(&original,t,&limits).unwrap();let b=om9_core::curve_ops::point_at(&curve,if same{t}else{1.-t},&limits).unwrap();let distance=(0..3).map(|j|(a[j]-b[j]).powi(2)).sum::<f64>().sqrt();assert!(distance<=0.000401,"fraction {t}, distance {distance}");}
        let path=std::env::temp_dir().join(format!("om9-trimmed-join-{}.om9",uuid::Uuid::new_v4()));om9_archive::Archive::save(&path,&after,&limits,&om9_core::transaction::Cancellation::default()).unwrap();let loaded=om9_archive::Archive::open(&path,&limits).unwrap();std::fs::remove_file(path).unwrap();assert_eq!(loaded,after);
        c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }
}

#[test]
fn join_certified_periodic_nurbs_opens_one_seam_and_heals_gap() {
    use om9_core::geometry::{Curve,NurbsCurve};
    let limits=Limits::default();
    for (reversed,via_toolbar) in [(false,false),(false,true),(true,false),(true,true)] {
        let n=NurbsCurve {degree:2,poles:vec![[0.,0.,0.],[2.,0.,0.],[2.,2.,0.],[0.,2.,0.],[0.,0.,0.],[2.,0.,0.]],weights:vec![1.,0.7,1.2,0.9,1.,0.7],knots:vec![-2.,-1.,0.,1.,2.,3.,4.,5.,6.],domain:[0.,4.],periodic:true,reversed,seam:None};
        n.certify_repeated_periodic(&limits).unwrap();
        let original=Curve::Nurbs(n);let end=om9_core::curve_ops::curve_endpoints(&original,&limits).unwrap()[1];
        let a=Entity::new("periodic",Geometry::Curve(original.clone()),None);
        let b=Entity::new("line",Geometry::Curve(Curve::Line{start:[end[0]+0.0008,end[1],end[2]],end:[end[0]+3.,end[1],end[2]],domain:[0.,1.]}),None);
        let ids=[a.id,b.id];let mut doc=Document::with_template_tolerance("periodic join");doc.entities.insert(a.id,a);doc.entities.insert(b.id,b);
        let mut c=Controller::new(doc,limits).unwrap();select(&mut c,&ids);let before=c.document().clone();
        if via_toolbar {c.top11_invoke(Top11Action::Join).unwrap();}else{c.submit("Join").unwrap();}
        let after=c.document().clone();
        let Geometry::Curve(Curve::Polycurve {segments,..})=&after.entities.values().next().unwrap().geometry else {panic!("polycurve")};
        let curve=segments.iter().find_map(|s|match s {Curve::Nurbs(n)=>{assert!(!n.periodic);Some(Curve::Nurbs(n.clone()))},_=>None}).unwrap();
        let ends=om9_core::curve_ops::curve_endpoints(&curve,&limits).unwrap();
        assert!(ends.iter().any(|p|(p[0]-end[0]-0.0004).abs()<1e-9));
        assert!(ends.iter().any(|p|(p[0]-end[0]).abs()<1e-9));
        let mut maximum=[0_f64;2];
        for i in 0..=100 {let t=i as f64/100.;let a=om9_core::curve_ops::point_at(&original,t,&limits).unwrap();for (j,u) in [t,1.-t].into_iter().enumerate(){let b=om9_core::curve_ops::point_at(&curve,u,&limits).unwrap();let d=(0..3).map(|k|(a[k]-b[k]).powi(2)).sum::<f64>().sqrt();maximum[j]=maximum[j].max(d);}}
        assert!(maximum.into_iter().any(|x| x<=0.000401),"{maximum:?}");
        let path=std::env::temp_dir().join(format!("om9-periodic-join-{}.om9",uuid::Uuid::new_v4()));om9_archive::Archive::save(&path,&after,&limits,&om9_core::transaction::Cancellation::default()).unwrap();let loaded=om9_archive::Archive::open(&path,&limits).unwrap();std::fs::remove_file(path).unwrap();assert_eq!(loaded,after);
        c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }
}

#[test]
fn explode_mesh_separates_unwelded_edges_and_preserves_channels() {
    use om9_core::geometry::Mesh;
    let limits=Limits::default();
    for (via_toolbar,select_after) in [(false,false),(true,false),(false,true),(true,true)] {
        let mesh=Mesh { vertices:vec![[0.,0.,0.],[1.,0.,0.],[1.,1.,0.],[0.,1.,0.],[1.,0.,0.],[2.,0.,0.],[1.,1.,0.]],faces:vec![vec![0,1,2],vec![0,2,3],vec![4,5,6]],normals:vec![[0.,0.,1.];7],colors:(0..7).map(|i|[i,10,20,255]).collect(),corner_uvs:(0..9).map(|i|[i as f64,0.]).collect() };
        let mut entity=Entity::new("mesh",Geometry::Mesh(mesh.clone()),None);entity.placement=Transform::translation([5.,2.,0.]);let id=entity.id;
        let mut doc=Document::with_template_tolerance("mesh explode");doc.entities.insert(id,entity);
        let mut c=Controller::new(doc,limits).unwrap();if !select_after {select(&mut c,&[id]);} let before=c.document().clone();
        if via_toolbar { c.top11_invoke(Top11Action::Explode).unwrap(); } else { c.submit("Explode").unwrap(); }
        if select_after {c.top11_accept_object(id).unwrap();c.submit("").unwrap();}
        let after=c.document().clone();assert_eq!(after.entities.len(),2);
        let mut pieces:Vec<_>=after.entities.values().map(|e| {assert_eq!(e.placement,Transform::translation([5.,2.,0.]));let Geometry::Mesh(m)=&e.geometry else {panic!("mesh")};m}).collect();pieces.sort_by_key(|m|m.faces.len());
        assert_eq!(pieces[0].faces.len(),1);assert_eq!(pieces[1].faces.len(),2);
        assert_eq!(pieces[0].corner_uvs,mesh.corner_uvs[6..]);assert_eq!(pieces[1].corner_uvs,mesh.corner_uvs[..6]);
        for piece in pieces {for (i,vertex) in piece.vertices.iter().enumerate(){let original=mesh.vertices.iter().enumerate().find(|(j,p)| *p==vertex && mesh.colors[*j]==piece.colors[i]).unwrap().0;assert_eq!(piece.normals[i],mesh.normals[original]);}}
        let path=std::env::temp_dir().join(format!("om9-mesh-explode-{}.om9",uuid::Uuid::new_v4()));om9_archive::Archive::save(&path,&after,&limits,&om9_core::transaction::Cancellation::default()).unwrap();let loaded=om9_archive::Archive::open(&path,&limits).unwrap();std::fs::remove_file(path).unwrap();assert_eq!(loaded,after);
        c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }
}

#[test]
fn explode_connected_mesh_is_refused_without_mutation() {
    use om9_core::geometry::Mesh;
    for orphan in [false,true] {
        let mut vertices=vec![[0.,0.,0.],[1.,0.,0.],[1.,1.,0.],[0.,1.,0.]];
        if orphan {vertices.push([3.,3.,0.]);}
        let mesh=Mesh {vertices,faces:vec![vec![0,1,2],vec![0,2,3]],normals:vec![],colors:vec![],corner_uvs:vec![]};
        let e=Entity::new("mesh",Geometry::Mesh(mesh),None);let id=e.id;let mut doc=Document::with_template_tolerance("mesh refusal");doc.entities.insert(id,e);
        let mut c=Controller::new(doc,Limits::default()).unwrap();select(&mut c,&[id]);let before=c.document().clone();assert!(c.submit("Explode").is_err());assert_eq!(c.document(),&before);
    }
}

#[test]
fn explode_keeps_output_pieces_in_each_original_group() {
    use om9_core::geometry::Curve;
    use om9_core::model::Group;
    for via_toolbar in [false,true] {
    let mut doc=Document::with_template_tolerance("group explode");
    let line=|a,b|Curve::Line{start:a,end:b,domain:[0.,1.]};
    let curve=Curve::Polycurve{segments:vec![line([0.,0.,0.],[1.,0.,0.]),line([1.,0.,0.],[2.,0.,0.])],reversed:false};
    let a=Entity::new("polycurve",Geometry::Curve(curve),None);let old=a.id;
    let b=Entity::new("other",Geometry::Point([5.,0.,0.]),None);let other=b.id;
    doc.entities.insert(old,a);doc.entities.insert(other,b);
    let g=Group::new("shared",vec![old,other]);let gid=g.id;doc.groups.insert(gid,g);
    let g2=Group::new("only",vec![old]);let gid2=g2.id;doc.groups.insert(gid2,g2);
    let mut c=Controller::new(doc,Limits::default()).unwrap();select(&mut c,&[old]);let before=c.document().clone();if via_toolbar {c.top11_invoke(Top11Action::Explode).unwrap();}else{c.submit("Explode").unwrap();}let after=c.document().clone();
    let parts:Vec<_>=after.entities.keys().filter(|id|**id!=other).copied().collect();assert_eq!(parts.len(),2);
    let shared=&after.groups[&gid];assert!(shared.members.contains(&other));assert!(!shared.members.contains(&old));assert!(parts.iter().all(|id|shared.members.contains(id)));assert_eq!(shared.members.len(),3);
    let only=&after.groups[&gid2];assert_eq!(only.members.len(),2);assert!(parts.iter().all(|id|only.members.contains(id)));
    let path=std::env::temp_dir().join(format!("om9-explode-groups-{}.om9",uuid::Uuid::new_v4()));om9_archive::Archive::save(&path,&after,&Limits::default(),&om9_core::transaction::Cancellation::default()).unwrap();let loaded=om9_archive::Archive::open(&path,&Limits::default()).unwrap();std::fs::remove_file(path).unwrap();assert_eq!(loaded,after);
    c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }
}

#[test]
fn explode_mesh_keeps_unreferenced_vertex_data() {
    use om9_core::geometry::Mesh;
    for via_toolbar in [false,true] {
    let mesh=Mesh {vertices:vec![[0.,0.,0.],[1.,0.,0.],[0.,1.,0.],[3.,0.,0.],[4.,0.,0.],[3.,1.,0.],[8.,9.,10.]],faces:vec![vec![0,1,2],vec![3,4,5]],normals:vec![[0.,0.,1.];7],colors:vec![[10,20,30,255],[10,20,30,255],[10,20,30,255],[10,20,30,255],[10,20,30,255],[10,20,30,255],[7,8,9,255]],corner_uvs:vec![]};
    let entity=Entity::new("mesh",Geometry::Mesh(mesh),None);let id=entity.id;let mut doc=Document::with_template_tolerance("loose vertex");doc.entities.insert(id,entity);let mut c=Controller::new(doc,Limits::default()).unwrap();select(&mut c,&[id]);let before=c.document().clone();if via_toolbar {c.top11_invoke(Top11Action::Explode).unwrap();}else{c.submit("Explode").unwrap();}let after=c.document().clone();
    let parts:Vec<_>=after.entities.values().map(|e|match &e.geometry {Geometry::Mesh(m)=>m,_=>panic!("mesh")}).collect();assert_eq!(parts.len(),2);
    let found:Vec<_>=parts.iter().flat_map(|m|m.vertices.iter().enumerate().filter(|(_,p)|**p==[8.,9.,10.]).map(|(i,_)|(m.normals[i],m.colors[i]))).collect();assert_eq!(found,vec![([0.,0.,1.],[7,8,9,255])]);
    let path=std::env::temp_dir().join(format!("om9-loose-mesh-{}.om9",uuid::Uuid::new_v4()));om9_archive::Archive::save(&path,&after,&Limits::default(),&om9_core::transaction::Cancellation::default()).unwrap();let loaded=om9_archive::Archive::open(&path,&Limits::default()).unwrap();std::fs::remove_file(path).unwrap();assert_eq!(loaded,after);
    c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }
}

#[test]
fn explode_mesh_output_limit_fails_atomically() {
    use om9_core::geometry::Mesh;
    let mesh=Mesh{vertices:vec![[0.,0.,0.],[1.,0.,0.],[0.,1.,0.],[3.,0.,0.],[4.,0.,0.],[3.,1.,0.]],faces:vec![vec![0,1,2],vec![3,4,5]],normals:vec![],colors:vec![],corner_uvs:vec![]};
    let e=Entity::new("mesh",Geometry::Mesh(mesh),None);let id=e.id;let mut doc=Document::with_template_tolerance("mesh output budget");doc.entities.insert(id,e);
    let mut c=Controller::new(doc,Limits{max_outputs:1,..Limits::default()}).unwrap();select(&mut c,&[id]);let before=c.document().clone();assert!(c.submit("Explode").is_err());assert_eq!(c.document(),&before);
}

#[test]
fn explode_block_only_from_cmd_preserves_members_and_shared_definition() {
    use om9_core::geometry::Curve;
    use om9_core::model::{Definition,Group};
    for select_after in [false,true] {
        let mut member=Entity::new("member",Geometry::Curve(Curve::Line{start:[0.,0.,0.],end:[2.,0.,0.],domain:[0.,1.]}),None);member.placement=Transform::translation([0.,4.,0.]);let member_id=member.id;
        let def=Definition::new("block",vec![member]);let def_id=def.id;
        let mut root=Entity::new("instance",Geometry::Instance(def_id),None);root.placement=Transform::translation([10.,0.,0.]);let id=root.id;
        let mut other=Entity::new("other instance",Geometry::Instance(def_id),None);other.placement=Transform::translation([30.,0.,0.]);let other_id=other.id;
        let mut doc=Document::with_template_tolerance("block explode");doc.definitions.insert(def_id,def);doc.entities.insert(id,root);doc.entities.insert(other_id,other);let group=Group::new("instances",vec![id,other_id]);let gid=group.id;doc.groups.insert(gid,group);
        let mut c=Controller::new(doc,Limits::default()).unwrap();if !select_after {select(&mut c,&[id]);let before=c.document().clone();assert!(c.top11_invoke(Top11Action::Explode).is_err());assert_eq!(c.document(),&before);}
        let before=c.document().clone();c.submit("Explode").unwrap();if select_after {c.top11_accept_object(id).unwrap();c.submit("").unwrap();}
        let after=c.document().clone();assert!(!after.entities.contains_key(&id));assert!(after.entities.contains_key(&other_id));assert_eq!(after.definitions,before.definitions);
        let piece=after.entities.values().find(|e|e.id!=other_id).unwrap();assert_ne!(piece.id,member_id);assert_eq!(piece.placement,Transform::translation([10.,4.,0.]));assert!(matches!(piece.geometry,Geometry::Curve(_)));let members=&after.groups[&gid].members;assert!(members.contains(&piece.id)&&members.contains(&other_id)&&!members.contains(&id));
        let path=std::env::temp_dir().join(format!("om9-block-explode-{}.om9",uuid::Uuid::new_v4()));om9_archive::Archive::save(&path,&after,&Limits::default(),&om9_core::transaction::Cancellation::default()).unwrap();let loaded=om9_archive::Archive::open(&path,&Limits::default()).unwrap();std::fs::remove_file(path).unwrap();assert_eq!(loaded,after);
        c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }
}

#[test]
fn mirror_nested_block_preserves_shared_definitions_and_scene() {
    use om9_core::{model::Definition,transaction::Cancellation};
    let limits=Limits::default();
    for toolbar in [false,true] {for copy in [false,true] {for axis in [false,true] {
        let leaf=Entity::new("point",Geometry::Point([1.,2.,0.]),None);let leaf_id=leaf.id;
        let inner=Definition::new("inner",vec![leaf]);let inner_id=inner.id;
        let mut nested=Entity::new("nested",Geometry::Instance(inner_id),None);nested.placement=Transform::translation([2.,0.,0.]);let nested_id=nested.id;
        let outer=Definition::new("outer",vec![nested]);let outer_id=outer.id;
        let mut root=Entity::new("root",Geometry::Instance(outer_id),None);root.placement=Transform::translation([4.,0.,0.]);let id=root.id;
        let mut doc=Document::with_template_tolerance("mirror block");doc.definitions.insert(inner_id,inner);doc.definitions.insert(outer_id,outer);doc.entities.insert(id,root);
        let mut c=Controller::new(doc,limits).unwrap();select(&mut c,&[id]);let before=c.document().clone();
        assert!(c.top11_availability().iter().any(|a|a.action==Top11Action::Mirror && a.enabled));
        if toolbar {c.top11_invoke(Top11Action::Mirror).unwrap();} else {c.submit("Mirror").unwrap();}
        if !copy {c.submit("Copy=No").unwrap();}
        if axis {c.submit("YAxis").unwrap();} else {c.accept_world_point([0.,-5.,0.]).unwrap();c.accept_world_point([0.,5.,0.]).unwrap();}
        let after=c.document().clone();assert_eq!(after.definitions,before.definitions);assert_eq!(after.entities.len(),if copy {2}else{1});
        let result=if copy {*c.selected().iter().next().unwrap()} else {id};
        assert_eq!(after.entities[&result].geometry,Geometry::Instance(outer_id));
        let p=after.member_transform(result,&[nested_id,leaf_id]).unwrap().point([1.,2.,0.]).unwrap();
        assert!((p[0]+7.).abs()<1e-9 && (p[1]-2.).abs()<1e-9,"{p:?}");
        assert!(c.scene().unwrap().objects.iter().any(|o|o.transform.point([1.,2.,0.]).is_ok_and(|p|(p[0]+7.).abs()<1e-9&&(p[1]-2.).abs()<1e-9)));
        let path=std::env::temp_dir().join(format!("om9-mirror-block-{}.om9",uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&path,&after,&limits,&Cancellation::default()).unwrap();assert_eq!(om9_archive::Archive::open(&path,&limits).unwrap(),after);std::fs::remove_file(path).unwrap();
        c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }}}
}

#[test]
fn mirror_cloud_and_block_in_one_transaction() {
    use om9_core::{geometry::Cloud,model::Definition};
    let limits=Limits::default();let def=Definition::new("block",vec![Entity::new("point",Geometry::Point([1.,0.,0.]),None)]);let def_id=def.id;
    let block=Entity::new("block",Geometry::Instance(def_id),None);let block_id=block.id;
    let cloud=Entity::new("cloud",Geometry::Cloud(Cloud{points:vec![[2.,1.,0.]],normals:vec![[1.,0.,0.]],colors:vec![[12,34,56,255]],hidden:vec![false]}),None);let cloud_id=cloud.id;
    let mut doc=Document::with_template_tolerance("mixed mirror");doc.definitions.insert(def_id,def);doc.entities.insert(block_id,block);doc.entities.insert(cloud_id,cloud);
    let mut c=Controller::new(doc,limits).unwrap();select(&mut c,&[cloud_id]);assert!(c.top11_availability().iter().any(|s|s.action==Top11Action::Mirror && s.enabled));
    select(&mut c,&[block_id,cloud_id]);let before=c.document().clone();let undo=c.coordinator().undo_len();c.submit("Mirror").unwrap();c.submit("Copy=No").unwrap();c.submit("YAxis").unwrap();
    let after=c.document().clone();let Geometry::Cloud(cloud)=&after.entities[&cloud_id].geometry else{panic!("cloud")};
    assert_eq!(cloud.points,[[-2.,1.,0.]]);assert_eq!(cloud.normals,[[-1.,0.,0.]]);assert_eq!(cloud.colors,[[12,34,56,255]]);
    assert_eq!(after.entities[&block_id].placement.point([1.,0.,0.]).unwrap(),[-1.,0.,0.]);assert_eq!(c.coordinator().undo_len(),undo+1);
    c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
}

#[test]
fn mirror_plane_uniformly_scaled_circle_preserves_angular_parameterization() {
    use om9_core::{geometry::Curve,curve_ops};
    let curve=Curve::Circle{center:[1.,2.,0.],x:[1.,0.,0.],y:[0.,1.,0.],radius:2.,domain:[7.,12.],angle_domain:Some([-0.8,2.])};
    for placement in [Transform::scale([2.,2.,2.]).unwrap(),Transform([[-2.,0.,0.,4.],[0.,2.,0.,1.],[0.,0.,3.,0.],[0.,0.,0.,1.]])] {
        for copy in [false,true] {for toolbar in [false,true] {
            let mut entity=Entity::new("arc",Geometry::Curve(curve.clone()),None);entity.placement=placement;let id=entity.id;
            let mut doc=Document::with_template_tolerance("arc parameter");doc.entities.insert(id,entity);
            let mut c=Controller::new(doc,Limits::default()).unwrap();select(&mut c,&[id]);let before=c.document().clone();
            if toolbar {c.top11_invoke(Top11Action::Mirror).unwrap();}else{c.submit("Mirror").unwrap();}
            if !copy {c.submit("Copy=No").unwrap();}c.submit("YAxis").unwrap();
            let after=c.document().clone();let result=if copy {*c.selected().iter().next().unwrap()}else{id};let reflected=&after.entities[&result];
            let Geometry::Curve(reflected_curve @ Curve::Circle{domain,angle_domain,..})=&reflected.geometry else {panic!("circle parameterization changed")};
            assert_eq!(*domain,[7.,12.]);assert_eq!(*angle_domain,Some([-0.8,2.]));
            for fraction in [0.,0.1,0.37,0.9,1.] {
                let original=placement.point(curve_ops::point_at(&curve,fraction,&Limits::default()).unwrap()).unwrap();
                let actual=reflected.placement.point(curve_ops::point_at(reflected_curve,fraction,&Limits::default()).unwrap()).unwrap();
                let expected=[-original[0],original[1],original[2]];
                assert!(actual.iter().zip(expected).all(|(a,b)|(a-b).abs()<1e-10),"{fraction}: {actual:?} vs {expected:?}");
            }
            c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
        }}
    }
}

#[test]
fn mirror_preserves_line_and_polycurve_parameter_domains() {
    use om9_core::geometry::Curve;
    let geometries=[
        Geometry::Curve(Curve::Line{start:[0.,0.,0.],end:[2.,0.,0.],domain:[2.,5.]}),
        Geometry::Curve(Curve::Polycurve{segments:vec![
            Curve::Line{start:[0.,0.,0.],end:[2.,0.,0.],domain:[2.,5.]},
            Curve::Line{start:[2.,0.,0.],end:[2.,1.,0.],domain:[5.,9.]},
        ],reversed:false}),
    ];
    let domains=|geometry:&Geometry| -> Vec<[f64;2]> {
        match geometry {
            Geometry::Curve(Curve::Line{domain,..})=>vec![*domain],
            Geometry::Curve(Curve::Polycurve{segments,..})=>segments.iter().map(|s|match s {Curve::Line{domain,..}=>*domain,_=>panic!("line")}).collect(),
            _=>panic!("curve"),
        }
    };
    for geometry in geometries { for copy in [false,true] { for toolbar in [false,true] {
        let mut entity=Entity::new("parameterized curve",geometry.clone(),None);
        entity.placement=Transform([[2.,0.,0.,4.],[0.,3.,0.,1.],[0.,0.,1.,0.],[0.,0.,0.,1.]]);let id=entity.id;
        let mut doc=Document::with_template_tolerance("mirror domains");doc.entities.insert(id,entity);
        let mut c=Controller::new(doc,Limits::default()).unwrap();select(&mut c,&[id]);let before=c.document().clone();
        if toolbar {c.top11_invoke(Top11Action::Mirror).unwrap();}else{c.submit("Mirror").unwrap();}
        if !copy {c.submit("Copy=No").unwrap();}c.submit("YAxis").unwrap();
        let result=if copy {*c.selected().iter().next().unwrap()}else{id};let after=c.document().clone();
        assert_eq!(domains(&after.entities[&result].geometry),domains(&geometry));
        let reflected=&after.entities[&result];
        let start=match &reflected.geometry {
            Geometry::Curve(Curve::Line{start,..})=>*start,
            Geometry::Curve(Curve::Polycurve{segments,..})=>match &segments[0] {Curve::Line{start,..}=>*start,_=>panic!("line")},
            _=>panic!("curve"),
        };
        assert_eq!(reflected.placement.point(start).unwrap(),[-4.,1.,0.]);
        let path=std::env::temp_dir().join(format!("om9-mirror-domain-{}.om9",uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&path,&after,&Limits::default(),&om9_core::transaction::Cancellation::default()).unwrap();
        assert_eq!(om9_archive::Archive::open(&path,&Limits::default()).unwrap(),after);std::fs::remove_file(path).unwrap();
        c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }}}
}

#[test]
fn mirror_mesh_preserves_face_boundaries_and_corner_uvs() {
    use om9_core::geometry::Mesh;
    let mesh = Mesh {
        vertices: vec![[0.,0.,0.],[2.,0.,0.],[0.,2.,0.],[2.,2.,0.]],
        faces: vec![vec![0,1,2],vec![0,1,3,2]],
        normals: vec![[0.,0.,1.];4], colors: vec![],
        corner_uvs: (0..7).map(|i| [i as f64, (i * i) as f64]).collect(),
    };
    for copy in [false,true] { for toolbar in [false,true] {
        let mut entity=Entity::new("mesh",Geometry::Mesh(mesh.clone()),None);
        entity.placement=Transform::translation([4.,1.,0.]); let id=entity.id;
        let mut doc=Document::with_template_tolerance("mirror mesh UV"); doc.entities.insert(id,entity);
        let mut c=Controller::new(doc,Limits::default()).unwrap(); select(&mut c,&[id]); let before=c.document().clone();
        if toolbar {c.top11_invoke(Top11Action::Mirror).unwrap();} else {c.submit("Mirror").unwrap();}
        if !copy {c.submit("Copy=No").unwrap();} c.submit("YAxis").unwrap();
        let after=c.document().clone(); let result=if copy {*c.selected().iter().next().unwrap()} else {id};
        let Geometry::Mesh(reflected)=&after.entities[&result].geometry else {panic!("mesh")};
        assert_eq!(reflected.faces,vec![vec![0,2,1],vec![0,2,3,1]]);
        assert_eq!(reflected.corner_uvs,vec![mesh.corner_uvs[0],mesh.corner_uvs[2],mesh.corner_uvs[1],mesh.corner_uvs[3],mesh.corner_uvs[6],mesh.corner_uvs[5],mesh.corner_uvs[4]]);
        assert_eq!(after.entities[&result].placement.point(reflected.vertices[1]).unwrap(),[-6.,1.,0.]);
        let path=std::env::temp_dir().join(format!("om9-mirror-mesh-{}.om9",uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&path,&after,&Limits::default(),&om9_core::transaction::Cancellation::default()).unwrap();
        assert_eq!(om9_archive::Archive::open(&path,&Limits::default()).unwrap(),after); std::fs::remove_file(path).unwrap();
        c.undo().unwrap(); assert_eq!(c.document(),&before); c.redo().unwrap(); assert_eq!(c.document(),&after);
    }}
}

#[test]
fn mirror_hatch_light_and_native_brep_preserves_payloads_and_attributes() {
    use om9_core::{geometry::{Hatch,Light,Mesh,NativeBrep},transaction::Cancellation};
    let limits=Limits::default();let mesh=Mesh{vertices:vec![[1.,0.,0.],[3.,0.,0.],[1.,2.,0.]],faces:vec![vec![0,1,2]],normals:vec![],colors:vec![],corner_uvs:vec![]};
    let geometries=[Geometry::Hatch(Hatch{display:mesh.clone(),loops:vec![vec![[1.,0.,0.],[3.,0.,0.],[1.,2.,0.]]],pattern:Some("test".into()),color:[10,20,30,255]}),
        Geometry::Light(Light{display:[1.,0.,0.],location:[1.,0.,2.],direction:[1.,2.,-1.],diffuse:[30,40,50,255],intensity:2.,style:1}),
        Geometry::NativeBrep(NativeBrep{display:mesh,payload:vec![1,2,3,4].into(),crc:0})];
    for geometry in geometries {for copy in [false,true] {for toolbar in [false,true] {
        let mut entity=Entity::new("mirror",geometry.clone(),None);entity.placement=Transform::translation([4.,1.,0.]);let id=entity.id;
        let mut doc=Document::with_template_tolerance("mirror retained objects");doc.entities.insert(id,entity);
        let mut c=Controller::new(doc,limits).unwrap();select(&mut c,&[id]);let before=c.document().clone();
        assert!(c.top11_availability().iter().any(|state|state.action==Top11Action::Mirror && state.enabled));
        if toolbar {c.top11_invoke(Top11Action::Mirror).unwrap();} else {c.submit("Mirror").unwrap();}
        if !copy {c.submit("Copy=No").unwrap();}c.submit("YAxis").unwrap();
        let after=c.document().clone();let result=if copy {*c.selected().iter().next().unwrap()}else{id};let reflected=&after.entities[&result];
        assert_eq!(reflected.geometry,geometry);assert_eq!(reflected.attributes,before.entities[&id].attributes);
        assert_eq!(reflected.placement.point([1.,0.,0.]).unwrap(),[-5.,1.,0.]);
        if let Geometry::Light(light)=&reflected.geometry {let location=reflected.placement.point(light.location).unwrap();let tip=reflected.placement.point(std::array::from_fn(|i|light.location[i]+light.direction[i])).unwrap();
            assert_eq!(location,[-5.,1.,2.]);assert_eq!(std::array::from_fn::<_,3,_>(|i|tip[i]-location[i]),[-1.,2.,-1.]);}
        let displayed=c.scene().unwrap().objects.iter().find(|o|o.root==result).expect("reflected scene object");
        assert_eq!(displayed.display_transform().unwrap().point([1.,0.,0.]).unwrap(),[-5.,1.,0.]);
        let camera=om9_scene::camera::Camera::orthographic([0.,0.,100.],[0.,0.,0.],[0.,1.,0.],40.).unwrap();
        let viewport=om9_scene::camera::Viewport{origin:[0.,0.],size:[256.,256.]};
        let pick=if matches!(geometry,Geometry::Light(_)) {[-5.,1.,0.]} else {[-5.5,1.5,0.]};
        let screen=camera.project(pick,viewport).unwrap().unwrap().screen;
        let hit=c.scene().unwrap().pick(screen,&camera,viewport,3.,om9_scene::scene::PickIntent::Select).unwrap().expect("reflected pick");assert_eq!(hit.root,result);
        let path=std::env::temp_dir().join(format!("om9-mirror-retained-{}.om9",uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&path,&after,&limits,&Cancellation::default()).unwrap();assert_eq!(om9_archive::Archive::open(&path,&limits).unwrap(),after);std::fs::remove_file(path).unwrap();
        c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }}}
}

#[test]
fn explode_toolbar_leaves_block_when_selected_with_polycurve() {
    use om9_core::geometry::Curve;
    use om9_core::model::{Definition,Group};
    let line=|a,b|Curve::Line{start:a,end:b,domain:[0.,1.]};
    let def=Definition::new("block",vec![Entity::new("member",Geometry::Point([3.,0.,0.]),None)]);let def_id=def.id;
    let instance=Entity::new("instance",Geometry::Instance(def_id),None);let block=instance.id;
    let curve=Entity::new("poly",Geometry::Curve(Curve::Polycurve{segments:vec![line([0.,0.,0.],[1.,0.,0.]),line([1.,0.,0.],[2.,0.,0.])],reversed:false}),None);let old=curve.id;
    let mut doc=Document::with_template_tolerance("toolbar block");doc.definitions.insert(def_id,def);doc.entities.insert(block,instance);doc.entities.insert(old,curve);let group=Group::new("group",vec![block,old]);let gid=group.id;doc.groups.insert(gid,group);
    let mut c=Controller::new(doc,Limits::default()).unwrap();select(&mut c,&[block,old]);let before=c.document().clone();c.top11_invoke(Top11Action::Explode).unwrap();let after=c.document().clone();assert_eq!(after.entities[&block],before.entities[&block]);assert!(!after.entities.contains_key(&old));assert_eq!(after.entities.len(),3);assert_eq!(after.groups[&gid].members.len(),3);assert!(after.entities.keys().all(|id|after.groups[&gid].members.contains(id)));c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
}

#[test]
fn explode_cmd_handles_block_and_polycurve_in_one_transaction() {
    use om9_core::geometry::Curve;
    use om9_core::model::Definition;
    let def=Definition::new("block",vec![Entity::new("member",Geometry::Point([3.,0.,0.]),None)]);let def_id=def.id;
    let instance=Entity::new("instance",Geometry::Instance(def_id),None);let block=instance.id;
    let line=|a,b|Curve::Line{start:a,end:b,domain:[0.,1.]};
    let curve=Entity::new("poly",Geometry::Curve(Curve::Polycurve{segments:vec![line([0.,0.,0.],[1.,0.,0.]),line([1.,0.,0.],[2.,0.,0.])],reversed:false}),None);let old=curve.id;
    let mut doc=Document::with_template_tolerance("mixed cmd");doc.definitions.insert(def_id,def);doc.entities.insert(block,instance);doc.entities.insert(old,curve);let mut c=Controller::new(doc,Limits::default()).unwrap();select(&mut c,&[block,old]);let before=c.document().clone();c.submit("Explode").unwrap();let after=c.document().clone();assert!(!after.entities.contains_key(&block));assert!(!after.entities.contains_key(&old));assert_eq!(after.entities.len(),3);assert_eq!(after.definitions,before.definitions);c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
}

#[test]
fn explode_cmd_block_output_limit_is_atomic() {
    use om9_core::model::Definition;
    let def=Definition::new("block",vec![Entity::new("a",Geometry::Point([0.,0.,0.]),None),Entity::new("b",Geometry::Point([1.,0.,0.]),None)]);let def_id=def.id;let root=Entity::new("root",Geometry::Instance(def_id),None);let id=root.id;let mut doc=Document::with_template_tolerance("block budget");doc.definitions.insert(def_id,def);doc.entities.insert(id,root);let mut c=Controller::new(doc,Limits{max_outputs:1,..Limits::default()}).unwrap();select(&mut c,&[id]);let before=c.document().clone();assert!(c.submit("Explode").is_err());assert_eq!(c.document(),&before);
}

#[test]
fn explode_cmd_block_preserves_nested_instance_and_world_transform() {
    use om9_core::model::Definition;
    let mut leaf=Entity::new("leaf",Geometry::Point([1.,0.,0.]),None);leaf.placement=Transform::translation([0.,0.,3.]);let leaf_id=leaf.id;
    let inner=Definition::new("inner",vec![leaf]);let inner_id=inner.id;
    let mut nested=Entity::new("nested",Geometry::Instance(inner_id),None);nested.placement=Transform::translation([0.,2.,0.]);let nested_id=nested.id;
    let outer=Definition::new("outer",vec![nested]);let outer_id=outer.id;
    let mut root=Entity::new("root",Geometry::Instance(outer_id),None);root.placement=Transform::translation([4.,0.,0.]);let root_id=root.id;
    let mut doc=Document::with_template_tolerance("nested block");doc.definitions.insert(inner_id,inner);doc.definitions.insert(outer_id,outer);doc.entities.insert(root_id,root);let expected=doc.member_transform(root_id,&[nested_id,leaf_id]).unwrap();
    let mut c=Controller::new(doc,Limits::default()).unwrap();select(&mut c,&[root_id]);let before=c.document().clone();c.submit("Explode").unwrap();let after=c.document().clone();let output=after.entities.values().next().unwrap();assert_eq!(output.geometry,Geometry::Instance(inner_id));assert_ne!(output.id,nested_id);assert_eq!(after.member_transform(output.id,&[leaf_id]).unwrap(),expected);assert_eq!(after.definitions,before.definitions);c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
}

#[test]
fn explode_block_preserves_hidden_and_locked_members_without_postcommit_error() {
    use om9_core::model::Definition;
    let visible=Entity::new("visible",Geometry::Point([1.,0.,0.]),None);
    let mut hidden=Entity::new("hidden",Geometry::Point([2.,0.,0.]),None);hidden.attributes.hidden=true;
    let mut locked=Entity::new("locked",Geometry::Point([3.,0.,0.]),None);locked.attributes.locked=true;
    let def=Definition::new("mixed",vec![visible,hidden,locked]);let def_id=def.id;let root=Entity::new("block",Geometry::Instance(def_id),None);let id=root.id;let mut doc=Document::with_template_tolerance("block flags");doc.definitions.insert(def_id,def);doc.entities.insert(id,root);
    let mut c=Controller::new(doc,Limits::default()).unwrap();select(&mut c,&[id]);let before=c.document().clone();let prior:Vec<_>=c.scene().unwrap().objects.iter().map(|o|(o.transform,o.visible,o.selectable,o.editable,o.color)).collect();c.submit("Explode").unwrap();let after=c.document().clone();assert_eq!(after.entities.len(),3);let later:Vec<_>=c.scene().unwrap().objects.iter().map(|o|(o.transform,o.visible,o.selectable,o.editable,o.color)).collect();assert_eq!(prior.len(),later.len());for value in prior {assert!(later.contains(&value));}assert_eq!(c.selected().len(),1);c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
}

#[test]
fn explode_block_with_locked_member_layer_refuses_before_commit() {
    use om9_core::model::{Definition,Layer};
    let mut layer=Layer::new("locked",[20,30,40,255]);layer.locked=true;let layer_id=layer.id;
    let visible=Entity::new("visible",Geometry::Point([1.,0.,0.]),None);let locked=Entity::new("locked",Geometry::Point([2.,0.,0.]),Some(layer_id));
    let def=Definition::new("mixed",vec![visible,locked]);let def_id=def.id;let root=Entity::new("block",Geometry::Instance(def_id),None);let id=root.id;
    let mut doc=Document::with_template_tolerance("block layer");doc.layers.insert(layer_id,layer);doc.layer_order.push(layer_id);doc.definitions.insert(def_id,def);doc.entities.insert(id,root);
    let mut c=Controller::new(doc,Limits::default()).unwrap();select(&mut c,&[id]);let before=c.document().clone();let undo=c.coordinator().undo_len();assert!(c.submit("Explode").is_err());assert_eq!(c.document(),&before);assert_eq!(c.coordinator().undo_len(),undo);
}

#[test]
fn duplicate_scene_budget_refusal_leaves_document_and_scene_unchanged() {
    use om9_core::model::Definition;
    let mut doc=Document::with_template_tolerance("duplicate scene budget");let leaf=Definition::new("leaf",(0..8).map(|i|Entity::new("point",Geometry::Point([i as f64,0.,0.]),None)).collect());let mut def_id=leaf.id;doc.definitions.insert(def_id,leaf);
    for _ in 0..3 {let def=Definition::new("nested",(0..8).map(|_|Entity::new("child",Geometry::Instance(def_id),None)).collect());def_id=def.id;doc.definitions.insert(def_id,def);}
    let root=Entity::new("root",Geometry::Instance(def_id),None);let id=root.id;doc.entities.insert(id,root);
    let mut c=Controller::new(doc,Limits{max_bytes:3_000_000,..Limits::default()}).unwrap();select(&mut c,&[id]);let before=c.document().clone();let scene_count=c.scene().unwrap().objects.len();let undo=c.coordinator().undo_len();
    let result=c.top11_duplicate_in_place();assert!(result.is_err(),"result={result:?} entities={} scene={:?}",c.document().entities.len(),c.scene().map(|s|s.objects.len()));assert_eq!(c.document(),&before);assert_eq!(c.scene().unwrap().objects.len(),scene_count);assert_eq!(c.coordinator().undo_len(),undo);
}
