//! FLIPPING THE AXIS OF A MATE ANCHORED TO CORNERS TURNS THE PART.
//!
//! Reported behaviour: "flip the axis" answered "Done" and the part neither moved nor turned, even with the anchors in
//! corners, where half a turn shows. The flip was checked only on the middles of faces.
use qymcad_core::feature::{AnchorRef, JointKind};
use qymcad_core::model::{Id, Project};

/// A 40 x 30 x 10 block as a part at `at`. Returns (component, body).
fn block(p: &mut Project, name: &str, at: [f64; 3]) -> (Id, Id) {
    let c = p.add_part(name);
    p.set_active_component(Some(c));
    let si = p.new_sketch(name);
    let sid = p.sketches[si].id;
    p.add_sketch_node(sid, name);
    p.add_rect_entity(si, 0.0, 0.0, 40.0, 30.0, qymcad_core::feature::Purpose::Real);
    p.regen_sketch(si);
    let body = p.add_extrude(sid, 10.0);
    p.finish_base_body(body, 1);
    p.move_component(c, at);
    (c, body)
}

/// The vertex anchor at the local corner `at` of `body`: an end of an edge that ends there.
fn corner(p: &Project, body: Id, at: [f64; 3]) -> AnchorRef {
    let near = |q: [f64; 3]| (0..3).all(|i| (q[i] - at[i]).abs() < 1e-6);
    let e = p.regen_edges[&body].iter().find(|e| near(e.a) || near(e.b)).expect("an edge ending at the corner");
    AnchorRef::Vertex(body, e.id, near(e.b))
}

fn placement(flip: bool) -> [f64; 12] {
    let mut p = Project::default();
    p.new_document();
    let root = p.root;
    p.set_active_component(Some(root));
    let (a, body_a) = block(&mut p, "A", [0.0, 0.0, 0.0]);
    p.set_active_component(Some(root));
    let (b, body_b) = block(&mut p, "B", [60.0, 0.0, 0.0]);
    p.set_grounded(a, true);
    qymcad_testkit::regenerate(&mut p);
    let ca = p.add_connector(a, corner(&p, body_a, [40.0, 0.0, 10.0]));
    let cb = p.add_connector(b, corner(&p, body_b, [40.0, 0.0, 10.0]));
    let jid = p.add_joint(ca, cb, JointKind::Rigid);
    p.solve_joints();
    if flip {
        assert!(p.flip_joint_side(jid), "the mate refused to flip");
        p.solve_joints();
    }
    p.world_transform(b)
}

#[test]
fn flipping_the_axis_turns_the_part_on_its_corner() {
    let (straight, flipped) = (placement(false), placement(true));
    let differs = straight.iter().zip(flipped.iter()).any(|(x, y)| (x - y).abs() > 1e-6);
    assert!(differs, "flipping the axis of a mate on corners left the part as it stood: {straight:?}");
}
