//! A HIDDEN PART HIDES ITS SKETCHES: the outlines of a part follow the ticks of its components, as its bodies do.
//!
//! Reported behaviour: in an assembly, with "Sketch outlines" on, the sketches of a part whose tick was off stayed on
//! screen over the place the part had been hidden from.
use qymcad_core::model::Project;
use qymcad_ui_state::sketch_shown_by_components;

/// An assembly holding a sub-assembly holding a part with one sketch: (project, sub-assembly, part, sketch).
fn nested() -> (Project, u64, u64, u64) {
    let mut p = Project::default();
    let root = p.ensure_root();
    p.set_active_component(Some(root));
    let sub = p.add_assembly("Sub");
    p.set_active_component(Some(sub));
    let part = p.add_part("Part");
    p.set_active_component(Some(part));
    let si = p.new_sketch("Sketch");
    let sid = p.sketches[si].id;
    p.add_sketch_node(sid, "Sketch");
    (p, sub, part, sid)
}

#[test]
fn the_sketch_of_a_part_goes_with_the_part_tick() {
    let (mut p, _, part, sid) = nested();
    let root = p.root;
    assert!(sketch_shown_by_components(&p, sid, root), "a sketch of a shown part is hidden");
    p.components.iter_mut().find(|c| c.id == part).expect("the part").visible = false;
    assert!(!sketch_shown_by_components(&p, sid, root), "the part's tick is off and its sketch is still shown in the assembly");
}

#[test]
fn the_sketch_goes_with_the_tick_of_a_sub_assembly_above_its_part() {
    let (mut p, sub, _, sid) = nested();
    let root = p.root;
    p.components.iter_mut().find(|c| c.id == sub).expect("the sub-assembly").visible = false;
    assert!(!sketch_shown_by_components(&p, sid, root), "the sub-assembly's tick is off and a sketch of its part is still shown");
}
