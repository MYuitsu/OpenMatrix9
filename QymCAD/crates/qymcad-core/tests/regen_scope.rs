//! The rebuild graph: editing a node touches only its descendants, and editing a parameter only what refers to
//! it.
//!
//! The timeline is linear and already a topological order, so the propagation of dirtiness is correct in itself.
//! The gap was elsewhere: the graph could not be asked which nodes depend on a given one, so the code marked
//! dirty with a margin — editing any parameter raised every feature carrying an expression, and the rebuild
//! called that on every pass. These checks hold the boundaries.
use qymcad_core::model::{Param, Project};

fn line_sketch(p: &mut Project, name: &str) -> (u64, usize) {
    let sid = p.add_sketch(name, vec![], None);
    p.add_sketch_node(sid, name);
    let si = p.sketch_index(sid).unwrap();
    p.add_rect_entity(si, 0.0, 0.0, 10.0, 10.0, qymcad_core::feature::Purpose::Real);
    p.regen_sketch(si);
    (sid, si)
}

/// A chain: dependent nodes are visible through `dependents` and independent ones are not.
#[test]
fn dependents_sees_the_chain_and_stops_there() {
    let mut p = Project::default();
    p.new_document();
    let (s1, _) = line_sketch(&mut p, "Sketch 1");
    let base = p.add_extrude_multi(s1, Vec::new(), 10.0, qymcad_core::feature::Reach::Forward, 0.0, Vec::new());
    let (s2, si2) = line_sketch(&mut p, "Sketch 2");
    let cut = p.add_combine_multi_op(
        base,
        s2,
        p.sketches[si2].contour_ids.clone(),
        qymcad_core::model::CombineSpan { height: 5.0, down: 0.0, extent: qymcad_core::feature::Extent::default(), fill: &[] },
        0,
    );
    // an independent part: its own chain, with no shared inputs
    let (s3, _) = line_sketch(&mut p, "Sketch 3");
    let other = p.add_extrude_multi(s3, Vec::new(), 3.0, qymcad_core::feature::Reach::Forward, 0.0, Vec::new());

    let dep = p.dependents(base);
    assert!(dep.contains(&cut), "the cut depends on the base: {dep:?}");
    assert!(!dep.contains(&other), "an unrelated part does not depend on the base: {dep:?}");

    let dep_cut = p.dependents(cut);
    assert!(!dep_cut.contains(&base), "a source does not depend on its consumer");
    assert!(dep_cut.is_empty() || !dep_cut.contains(&other), "and neither does an unrelated part");
    let _ = s3;
}

/// A parameter: editing a name raises only the features where that name is actually mentioned.
#[test]
fn only_features_mentioning_the_parameter_get_dirty() {
    let mut p = Project::default();
    p.new_document();
    p.parameters.push(Param { name: "H".into(), expr: "10".into(), value: 10.0 });
    p.parameters.push(Param { name: "W".into(), expr: "20".into(), value: 20.0 });
    let (s1, _) = line_sketch(&mut p, "Sketch 1");
    let a = p.add_extrude_multi(s1, Vec::new(), 10.0, qymcad_core::feature::Reach::Forward, 0.0, Vec::new());
    let (s2, _) = line_sketch(&mut p, "Sketch 2");
    let b = p.add_extrude_multi(s2, Vec::new(), 20.0, qymcad_core::feature::Reach::Forward, 0.0, Vec::new());
    p.set_feat_dim(a, "height", "H".into());
    p.set_feat_dim(b, "height", "W*2".into());
    for n in p.timeline.iter_mut() {
        n.dirty = false;
    }

    p.mark_param_dependents_dirty_for("H");
    let dirty: Vec<u64> = p.timeline.iter().filter(|n| n.dirty).map(|n| n.id).collect();
    assert!(dirty.contains(&a), "the feature whose expression uses H is marked: {dirty:?}");
    assert!(!dirty.contains(&b), "the feature whose expression uses W*2 is left alone: {dirty:?}");
}

/// `H` and `h` are two parameters: editing `H` raises the feature written over `H` and leaves the one over `h`.
///
/// Reported behaviour: a parameter H changed from 20 to 40 under an extrusion of height H, the properties said 40,
/// the body stayed 20 high; with the name h the body was rebuilt. The rebuild asked by the key of `param_map`,
/// which was lower-cased, and `h` is not mentioned in a formula written `H`.
#[test]
fn a_parameter_in_capitals_raises_what_is_written_over_it() {
    let mut p = Project::default();
    p.new_document();
    p.parameters.push(Param { name: "H".into(), expr: "20".into(), value: 20.0 });
    p.parameters.push(Param { name: "h".into(), expr: "30".into(), value: 30.0 });
    let (s1, _) = line_sketch(&mut p, "Sketch 1");
    let a = p.add_extrude_multi(s1, Vec::new(), 20.0, qymcad_core::feature::Reach::Forward, 0.0, Vec::new());
    let (s2, _) = line_sketch(&mut p, "Sketch 2");
    let b = p.add_extrude_multi(s2, Vec::new(), 30.0, qymcad_core::feature::Reach::Forward, 0.0, Vec::new());
    p.set_feat_dim(a, "height", "H".into());
    p.set_feat_dim(b, "height", "h".into());
    assert_eq!(p.param_map().get("H"), Some(&20.0), "the parameter H is not in the scope under its own name");
    assert_eq!(p.param_map().get("h"), Some(&30.0), "the parameter h is not in the scope under its own name");
    for n in p.timeline.iter_mut() {
        n.dirty = false;
    }

    for key in p.param_map().into_keys().filter(|k| k == "H") {
        p.mark_param_dependents_dirty_for(&key);
    }
    let dirty: Vec<u64> = p.timeline.iter().filter(|n| n.dirty).map(|n| n.id).collect();
    assert!(dirty.contains(&a), "the feature whose expression uses H is not marked: {dirty:?}");
    assert!(!dirty.contains(&b), "the feature whose expression uses h is marked by an edit of H: {dirty:?}");
}

/// A name is mentioned only in its own case: `h` is not `H`.
#[test]
fn a_parameter_name_is_matched_in_its_own_case() {
    use qymcad_core::expr::mentions;
    assert!(mentions("H*2", "H"));
    assert!(!mentions("H*2", "h"));
    assert!(!mentions("h*2", "H"));
}

/// The name of a parameter must not be found inside another name: `L` is not mentioned in `Length`.
#[test]
fn parameter_name_is_matched_as_a_whole_identifier() {
    assert!(qymcad_core::expr::mentions("L*2", "L"));
    assert!(qymcad_core::expr::mentions("2*L + 3", "L"));
    assert!(qymcad_core::expr::mentions("Length/2", "Length"));
    assert!(!qymcad_core::expr::mentions("Length/2", "L"), "L is not part of Length");
    assert!(!qymcad_core::expr::mentions("HOLE_D", "D"), "D is not the tail of HOLE_D");
    assert!(!qymcad_core::expr::mentions("", "L"));
}

/// A background rebuild brought back takes the live placement, except a mate value given by an expression: that one
/// is what the rebuild evaluated, and the live one is the parameter's old value. Reported behaviour: a hinge angle
/// typed as a parameter stayed at its old turn when the parameter changed.
#[test]
fn a_rebuild_keeps_the_mate_value_its_expression_gave() {
    let mut live = Project::default();
    live.new_document();
    let j = live.add_joint(0, 0, qymcad_core::feature::JointKind::Revolute);
    live.set_feat_dim(j, "angle", "PA".into());
    live.joints[0].drive = [Some(90.0), Some(3.0), None];
    let mut rebuilt = live.clone();
    rebuilt.joints[0].drive = [Some(45.0), Some(7.0), None];
    rebuilt.take_placement_from(&live);
    assert!(rebuilt.joints[0].drive[0] == Some(45.0), "the angle given as PA kept the old live value: {:?}", rebuilt.joints[0].drive);
    assert!(rebuilt.joints[0].drive[1] == Some(3.0), "the offset given by no expression did not take the live value: {:?}", rebuilt.joints[0].drive);
}
