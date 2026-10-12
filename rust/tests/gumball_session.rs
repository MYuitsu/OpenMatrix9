// SPDX-License-Identifier: LGPL-2.1-or-later
// These fixtures catch wrong frame composition, gesture timing and atomic errors.
use openmatrix9_rust::gumball::*;
use openmatrix9_rust::{core_objects::Stamp, geometry_frame::IDENTITY};

fn begin(handle: Handle) -> BeginContext {
    BeginContext {
        frame: Frame::default(),
        cplane: Frame::default(),
        sources: vec![Source {
            stamp: Stamp {
                document: 1,
                object: 1,
                generation: 1,
                revision: 1,
                epoch: 1,
            },
            parent: 0,
            world: IDENTITY,
        }],
        document: 1,
        view_generation: 1,
        unit_generation: 1,
        mm_per_unit: 1.,
        handle,
        start_ray: PointerRay {
            origin: [1., 1., 10.],
            direction: [0., 0., -1.],
        },
        modifiers: 0,
    }
}
fn started(context: BeginContext) -> GumballSession {
    let mut s = GumballSession::new();
    s.apply(Event::Enable(true)).unwrap();
    s.apply(Event::Begin(context)).unwrap();
    s
}
fn world_move_x() -> GumballSession {
    started(begin(Handle::MoveAxis(Axis::X)))
}
#[test]
fn relocate_points_preview_shared_origin_and_axes_before_accept(){
    let mut s=GumballSession::new();s.apply(Event::Enable(true)).unwrap();s.apply(Event::BeginRelocate).unwrap();
    s.apply(Event::RelocatePoint([10.,20.,30.])).unwrap();assert_eq!(s.snapshot().frame.origin,[10.,20.,30.]);
    s.apply(Event::RelocatePoint([10.,21.,30.])).unwrap();near(s.snapshot().frame.axes[0],[0.,1.,0.]);
    assert_eq!(s.snapshot().phase,Phase::Relocate);
    s.apply(Event::Cancel).unwrap();assert_eq!(s.snapshot().frame,Frame::default());
}
#[test]
fn presentation_fallback_does_not_change_scale_reference_or_numeric_input() {
    let mut s=GumballSession::new();s.apply(Event::Enable(true)).unwrap();
    s.set_extents(&[[-0.1,-1.,-1.],[0.1,1.,1.]]).unwrap();
    let mut settings=s.settings;settings.strength=50.;s.set_settings(settings).unwrap();
    let mut b=begin(Handle::Scale(Axis::X));b.start_ray.origin=[-1.2,0.,10.];
    s.apply(Event::Begin(b)).unwrap();assert_eq!(s.plan().unwrap().matrix,IDENTITY);
    s.apply(Event::Motion(PointerRay{origin:[-1.3,0.,10.],direction:[0.,0.,-1.]})).unwrap();
    assert!((s.plan().unwrap().matrix[0]-1.5).abs()<1e-12);
    s.apply(Event::Numeric("2".into())).unwrap();assert_eq!(s.plan().unwrap().matrix[0],2.);
}
#[test]
fn pointer_strength_and_snap_respect_axis_and_atomic_settings() {
    let mut s=GumballSession::new();s.apply(Event::Enable(true)).unwrap();
    let mut settings=s.settings;settings.strength=50.;s.set_settings(settings).unwrap();
    s.apply(Event::Begin(begin(Handle::MoveAxis(Axis::X)))).unwrap();
    s.apply(Event::Motion(PointerRay{origin:[5.,1.,10.],direction:[0.,0.,-1.]})).unwrap();
    assert_eq!(s.plan().unwrap().matrix[3],2.);
    let before=s.snapshot();assert!(s.set_settings(settings).is_err());assert_eq!(s.snapshot(),before);
    s.apply(Event::SnappedMotion([9.,18.,4.])).unwrap();
    assert_eq!(point(s.plan().unwrap().matrix,[0.;3]),[4.,0.,0.]);
    s.apply(Event::Numeric("7".into())).unwrap();assert_eq!(s.plan().unwrap().matrix[3],7.);
}
#[test]
fn relocate_pointer_uses_cplane_when_no_native_snap_and_rejects_parallel_ray() {
    let mut s=GumballSession::new();s.apply(Event::Enable(true)).unwrap();
    let cp=Frame{origin:[0.,0.,3.],..Frame::default()};
    s.apply(Event::ObserveFrame(Frame::default(),cp)).unwrap();
    s.apply(Event::BeginRelocate).unwrap();
    s.apply(Event::RelocateRay(PointerRay{origin:[2.,4.,10.],direction:[0.,0.,-1.]})).unwrap();
    s.apply(Event::RelocateRay(PointerRay{origin:[3.,4.,10.],direction:[0.,0.,-1.]})).unwrap();
    let before=s.snapshot();
    assert!(s.apply(Event::RelocateRay(PointerRay{origin:[0.;3],direction:[1.,0.,0.]})).is_err());
    assert_eq!(s.snapshot(),before);
    s.apply(Event::Accept).unwrap();near(s.snapshot().frame.origin,[2.,4.,3.]);
}
fn point(m: [f64; 16], p: [f64; 3]) -> [f64; 3] {
    std::array::from_fn(|r| {
        m[4 * r + 3] + m[4 * r] * p[0] + m[4 * r + 1] * p[1] + m[4 * r + 2] * p[2]
    })
}
fn near(a: [f64; 3], b: [f64; 3]) {
    for i in 0..3 {
        assert!((a[i] - b[i]).abs() < 1e-10, "{a:?} != {b:?}");
    }
}
fn modifier(s: &mut GumballSession, key: ModifierKey, pressed: bool, auto_repeat: bool) {
    s.apply(Event::Modifier(ModifierEvent {
        key,
        pressed,
        auto_repeat,
    }))
    .unwrap();
}

#[test]
fn world_axis_numeric_move_is_exact() {
    let mut s = world_move_x();
    s.apply(Event::Numeric("5 mm".into())).unwrap();
    assert_eq!(point(s.plan().unwrap().matrix, [0.; 3]), [5., 0., 0.]);
}

#[test]
fn ctrl_rotation_changes_widget_axes_and_cancel_restores_frame() {
    let mut b=begin(Handle::Rotate(Axis::Z));b.modifiers=2;
    b.start_ray.origin=[1.,0.,10.];
    let mut s=started(b);
    s.apply(Event::Motion(PointerRay {origin:[0.,1.,10.],direction:[0.,0.,-1.]})).unwrap();
    near(s.snapshot().frame.axes[0],[0.,1.,0.]);
    near(s.snapshot().frame.origin,[0.;3]);
    assert!(s.plan().is_err());
    s.apply(Event::Cancel).unwrap();assert_eq!(s.snapshot().frame,Frame::default());
}
#[test]
fn numeric_yz_move_without_motion_stays_in_its_plane() {
    let mut s=started(begin(Handle::MovePlane(Plane::YZ)));
    s.apply(Event::Numeric("5 mm".into())).unwrap();
    near(point(s.plan().unwrap().matrix,[0.;3]),[0.,5.,0.]);
}
#[test]
fn source_ancestry_is_bounded_before_begin_changes_the_session() {
    let mut b=begin(Handle::MoveAxis(Axis::X));let prototype=b.sources[0].clone();
    b.sources=(1..=66).map(|i|{let mut row=prototype.clone();row.stamp.object=i;row.parent=i-1;row}).collect();
    let mut s=GumballSession::new();s.apply(Event::Enable(true)).unwrap();let before=s.snapshot();
    assert!(s.apply(Event::Begin(b)).is_err());assert_eq!(s.snapshot(),before);
}
#[test]
fn committed_report_keeps_exact_input_frame_units_and_source_identity() {
    let mut s=world_move_x();s.apply(Event::Numeric("5.123456789123 mm".into())).unwrap();s.apply(Event::Accept).unwrap();
    let report=s.report().unwrap();
    assert!(report.contains("\"numeric\":\"5.123456789123 mm\""));assert!(report.contains("5.123456789123"));
    assert!(report.contains("\"mm_per_unit\":1"));assert!(report.contains("\"sources\":[{\"document\":1,\"object\":1,"));
    s.apply(Event::Cancel).unwrap();assert!(s.report().is_err());
}
#[test]
fn object_axis_moves_in_world_direction() {
    let mut b = begin(Handle::MoveAxis(Axis::X));
    b.frame = Frame::new([10., 20., 30.], [[0., 1., 0.], [-1., 0., 0.], [0., 0., 1.]]).unwrap();
    let mut s = started(b);
    s.apply(Event::Numeric("5".into())).unwrap();
    assert_eq!(
        point(s.plan().unwrap().matrix, [10., 20., 30.]),
        [10., 25., 30.]
    );
}
#[test]
fn rotation_keeps_pivot_and_uses_degrees() {
    let mut b = begin(Handle::Rotate(Axis::Z));
    b.frame.origin = [3., 4., 0.];
    let mut s = started(b);
    s.apply(Event::Numeric("90".into())).unwrap();
    near(point(s.plan().unwrap().matrix, [5., 4., 0.]), [3., 6., 0.]);
    near(point(s.plan().unwrap().matrix, [3., 4., 0.]), [3., 4., 0.]);
}
#[test]
fn scale_x_and_shift_uniform_keep_pivot() {
    let mut b = begin(Handle::Scale(Axis::X));
    b.frame.origin = [3., 4., 5.];
    let mut s = started(b);
    s.apply(Event::Numeric("2".into())).unwrap();
    assert_eq!(point(s.plan().unwrap().matrix, [4., 6., 8.]), [5., 6., 8.]);
    modifier(&mut s, ModifierKey::Shift, true, false);
    assert_eq!(point(s.plan().unwrap().matrix, [4., 6., 8.]), [5., 8., 11.]);
}
#[test]
fn negative_scale_is_valid_but_singular_and_nonfinite_are_atomic() {
    let mut s = started(begin(Handle::Scale(Axis::X)));
    s.apply(Event::Numeric("-2".into())).unwrap();
    assert_eq!(point(s.plan().unwrap().matrix, [1., 2., 3.]), [-2., 2., 3.]);
    for bad in ["0", "NaN", "inf", "2 mm", "1e999"] {
        let before = s.snapshot();
        assert!(s.apply(Event::Numeric(bad.into())).is_err());
        assert_eq!(s.snapshot(), before);
    }
}
#[test]
fn length_input_uses_captured_document_units() {
    let mut b = begin(Handle::MoveAxis(Axis::X));
    b.mm_per_unit = 25.4;
    let mut s = started(b);
    s.apply(Event::Numeric("2".into())).unwrap();
    assert_eq!(s.plan().unwrap().matrix[3], 50.8);
    s.apply(Event::Numeric("5 mm".into())).unwrap();
    assert_eq!(s.plan().unwrap().matrix[3], 5.);
    assert!(s.apply(Event::Numeric("90 deg".into())).is_err());
}
#[test]
fn alt_taps_toggle_copy_once_and_autorepeat_is_ignored() {
    let mut s = world_move_x();
    modifier(&mut s, ModifierKey::Alt, true, false);
    assert!(s.snapshot().copy);
    modifier(&mut s, ModifierKey::Alt, true, true);
    assert!(s.snapshot().copy);
    modifier(&mut s, ModifierKey::Alt, false, false);
    modifier(&mut s, ModifierKey::Alt, true, false);
    assert!(!s.snapshot().copy);
}
#[test]
fn ctrl_at_start_relocates_widget_but_ctrl_after_motion_extrudes() {
    let mut b = begin(Handle::MoveAxis(Axis::X));
    b.modifiers = 2;
    let mut s = started(b);
    s.apply(Event::Motion(PointerRay {
        origin: [6., 1., 10.],
        direction: [0., 0., -1.],
    }))
    .unwrap();
    assert_eq!(s.snapshot().frame.origin, [5., 0., 0.]);
    assert!(s.plan().is_err());
    s.apply(Event::Cancel).unwrap();
    assert_eq!(s.snapshot().frame.origin, [0.; 3]);
    let mut s = world_move_x();
    s.apply(Event::Motion(PointerRay {
        origin: [6., 1., 10.],
        direction: [0., 0., -1.],
    }))
    .unwrap();
    modifier(&mut s, ModifierKey::Ctrl, true, false);
    assert_eq!(s.plan().unwrap().operation, Operation::Extrude);
    modifier(&mut s, ModifierKey::Ctrl, false, false);
    assert_eq!(s.plan().unwrap().operation, Operation::Move);
}
#[test]
fn numeric_enter_during_drag_constrains_and_release_requests_commit() {
    let mut s = world_move_x();
    s.apply(Event::Motion(PointerRay {
        origin: [3., 1., 10.],
        direction: [0., 0., -1.],
    }))
    .unwrap();
    s.apply(Event::Numeric("5".into())).unwrap();
    assert_eq!(s.apply(Event::Accept).unwrap(), Effect::Preview);
    s.apply(Event::Motion(PointerRay {
        origin: [20., 1., 10.],
        direction: [0., 0., -1.],
    }))
    .unwrap();
    assert_eq!(s.plan().unwrap().matrix[3], 5.);
    assert_eq!(s.apply(Event::Release).unwrap(), Effect::Commit);
    s.apply(Event::CommitSucceeded).unwrap();
    assert_eq!(s.snapshot().phase, Phase::Ready);
}
#[test]
fn constrained_axis_distance_follows_pointer_side_and_signed_input() {
    for input in [5., -5.] {
        let mut s = world_move_x();
        let ray = |x| PointerRay { origin: [x, 1., 10.], direction: [0., 0., -1.] };
        s.apply(Event::Motion(ray(-2.))).unwrap();
        s.apply(Event::Numeric(format!("{input} mm"))).unwrap();
        s.apply(Event::Accept).unwrap();
        assert_eq!(s.plan().unwrap().matrix[3], -input, "entry uses current drag side");
        for (x, direction) in [(4., 1.), (-2., -1.), (1., -1.), (5., 1.)] {
            s.apply(Event::Motion(ray(x))).unwrap();
            assert_eq!(s.plan().unwrap().matrix[3], input * direction);
        }
        s.apply(Event::Release).unwrap();
        assert_eq!(s.plan().unwrap().matrix[3], input);
    }
}
#[test]
fn constrained_extrusion_can_cross_start_to_negative_distance() {
    let mut s = world_move_x();
    s.apply(Event::Motion(PointerRay { origin: [4., 1., 10.], direction: [0., 0., -1.] })).unwrap();
    s.apply(Event::Numeric("5 mm".into())).unwrap();
    s.apply(Event::Accept).unwrap();
    modifier(&mut s, ModifierKey::Ctrl, true, false);
    s.apply(Event::Motion(PointerRay { origin: [-2., 1., 10.], direction: [0., 0., -1.] })).unwrap();
    s.apply(Event::Release).unwrap();
    assert_eq!(s.plan().unwrap().operation, Operation::Extrude);
    assert_eq!(point(s.plan().unwrap().matrix, [0.; 3]), [-5., 0., 0.]);
}
#[test]
fn numeric_click_enter_requests_commit() {
    let mut s = world_move_x();
    s.apply(Event::Numeric("5".into())).unwrap();
    assert_eq!(s.apply(Event::Accept).unwrap(), Effect::Commit);
}
#[test]
fn ctrl_numeric_click_changes_frame_and_never_has_geometry_plan() {
    for handle in [Handle::MoveAxis(Axis::X),Handle::Rotate(Axis::Z),Handle::Scale(Axis::X)] {
        let mut b=begin(handle);b.modifiers=2;
        let mut s=started(b);
        s.apply(Event::Numeric(if matches!(handle,Handle::Rotate(_)){"90"}else{"5 mm"}.into())).unwrap();
        assert!(s.plan().is_err());
        if matches!(handle,Handle::Rotate(_)) {
            assert!(s.snapshot().frame.axes[0][1]>0.999999);
        } else { assert_eq!(s.snapshot().frame.origin,[5.,0.,0.]); }
        assert_eq!(s.apply(Event::Accept).unwrap(),Effect::FrameChanged);
        assert_eq!(s.snapshot().phase,Phase::Ready);
    }
}
#[test]
fn relocate_x_parallel_to_cplane_normal_requires_explicit_y() {
    let mut s=GumballSession::new();s.apply(Event::Enable(true)).unwrap();s.apply(Event::BeginRelocate).unwrap();
    s.apply(Event::RelocatePoint([0.;3])).unwrap();s.apply(Event::RelocatePoint([0.,0.,1.])).unwrap();
    let before=s.snapshot();assert!(s.apply(Event::Accept).is_err());assert_eq!(s.snapshot(),before);
    s.apply(Event::RelocatePoint([0.,1.,0.])).unwrap();assert_eq!(s.snapshot().phase,Phase::Ready);
}
#[test]
fn single_owner_relocate_survives_reselect_but_replacement_does_not() {
    let stamp=begin(Handle::MoveAxis(Axis::X)).sources[0].stamp;
    let mut s=GumballSession::new();s.apply(Event::Enable(true)).unwrap();
    s.apply(Event::ObserveOwner(Some(stamp),Frame::default(),Frame::default())).unwrap();
    s.apply(Event::BeginRelocate).unwrap();s.apply(Event::RelocatePoint([10.,20.,30.])).unwrap();s.apply(Event::RelocatePoint([11.,20.,30.])).unwrap();s.apply(Event::Accept).unwrap();
    let relocated=s.snapshot().frame;
    s.apply(Event::ObserveOwner(None,Frame::default(),Frame::default())).unwrap();
    s.apply(Event::ObserveOwner(Some(stamp),Frame::default(),Frame::default())).unwrap();assert_eq!(s.snapshot().frame,relocated);
    s.apply(Event::Align(Alignment::World)).unwrap();s.apply(Event::ObserveOwner(Some(stamp),Frame::default(),Frame::default())).unwrap();assert_eq!(s.snapshot().frame,Frame::default());
    s.apply(Event::BeginRelocate).unwrap();s.apply(Event::RelocatePoint([10.,20.,30.])).unwrap();s.apply(Event::RelocatePoint([11.,20.,30.])).unwrap();s.apply(Event::Accept).unwrap();
    let mut replacement=stamp;replacement.generation+=1;
    s.apply(Event::ObserveOwner(Some(replacement),Frame::default(),Frame::default())).unwrap();assert_eq!(s.snapshot().frame,Frame::default());
}
#[test]
fn hover_does_not_consume_undo_checkpoint() {
    let mut s = world_move_x();
    s.apply(Event::Numeric("5".into())).unwrap();
    for x in 2..100 {
        s.apply(Event::Motion(PointerRay {
            origin: [x as f64, 1., 10.],
            direction: [0., 0., -1.],
        }))
        .unwrap();
    }
    s.apply(Event::Undo).unwrap();
    assert_eq!(s.plan().unwrap().matrix, IDENTITY);
    s.apply(Event::Cancel).unwrap();
    assert_eq!(s.snapshot().phase, Phase::Ready);
    assert_eq!(s.snapshot().frame.origin, [0.; 3]);
}
#[test]
fn relocate_origin_x_and_optional_y_changes_only_frame() {
    let mut s = GumballSession::new();
    s.apply(Event::Enable(true)).unwrap();
    s.apply(Event::BeginRelocate).unwrap();
    s.apply(Event::RelocatePoint([10., 20., 30.])).unwrap();
    assert!(s.apply(Event::RelocatePoint([10., 20., 30.])).is_err());
    s.apply(Event::RelocatePoint([10., 21., 30.])).unwrap();
    s.apply(Event::Accept).unwrap();
    assert_eq!(s.snapshot().frame.origin, [10., 20., 30.]);
    assert_eq!(s.snapshot().frame.axes[0], [0., 1., 0.]);
    assert!(s.plan().is_err());
}
#[test]
fn source_roots_remove_parent_child_duplicates_not_link_occurrences() {
    let mut b = begin(Handle::MoveAxis(Axis::X));
    let parent = b.sources[0].clone();
    let mut child = parent.clone();
    child.stamp.object = 2;
    child.parent = 1;
    let mut link = parent.clone();
    link.stamp.object = 3;
    b.sources = vec![child, parent.clone(), link, parent];
    let s = started(b);
    assert_eq!(
        s.plan()
            .unwrap()
            .sources
            .iter()
            .map(|x| x.stamp.object)
            .collect::<Vec<_>>(),
        vec![1, 3]
    );
}
#[test]
fn invalid_frame_begin_and_cross_document_sources_are_atomic() {
    let mut s = GumballSession::new();
    s.apply(Event::Enable(true)).unwrap();
    let before = s.snapshot();
    let mut b = begin(Handle::MoveAxis(Axis::X));
    b.frame.axes[1] = b.frame.axes[0];
    assert!(s.apply(Event::Begin(b)).is_err());
    assert_eq!(s.snapshot(), before);
    let mut b = begin(Handle::MoveAxis(Axis::X));
    b.sources[0].stamp.document = 2;
    assert!(s.apply(Event::Begin(b)).is_err());
    assert_eq!(s.snapshot(), before);
}
#[test]
fn plane_and_origin_motion_use_world_rays() {
    for h in [Handle::MovePlane(Plane::XY), Handle::MoveFree] {
        let mut s = started(begin(h));
        s.apply(Event::Motion(PointerRay {
            origin: [4., 5., 10.],
            direction: [0., 0., -1.],
        }))
        .unwrap();
        assert_eq!(point(s.plan().unwrap().matrix, [0.; 3]), [3., 4., 0.]);
    }
}
#[test]
fn both_relocate_aliases_share_one_command() {
    for n in [
        "RelocateGumball",
        "GumballRelocate",
        "_GumballRelocate",
        "OM9_RelocateGumball",
    ] {
        assert_eq!(kind(n), Some(CommandKind::Relocate));
    }
    assert_eq!(command("InfoSettingsGumballON"), Some("OM9_Gumball"));
}
#[test]
fn relocate_auto_axes_follow_captured_cplane() {
    let mut s = GumballSession::new();
    s.apply(Event::Enable(true)).unwrap();
    let cp = Frame::new([0.; 3], [[1., 0., 0.], [0., 0., 1.], [0., -1., 0.]]).unwrap();
    s.apply(Event::ObserveFrame(Frame::default(), cp)).unwrap();
    s.apply(Event::BeginRelocate).unwrap();
    s.apply(Event::RelocatePoint([0.; 3])).unwrap();
    s.apply(Event::RelocatePoint([1., 0., 0.])).unwrap();
    s.apply(Event::Accept).unwrap();
    assert_eq!(s.snapshot().frame.axes, cp.axes);
}
#[test]
fn frame_observations_normalize_tangent_and_project_normal() {
    let f = Frame::from_directions([1., 2., 3.], [2., 0., 0.], [1., 0., 4.]).unwrap();
    assert_eq!(f.axes, Frame::default().axes);
    assert!(Frame::from_directions([0.; 3], [1., 0., 0.], [2., 0., 0.]).is_err());
}
