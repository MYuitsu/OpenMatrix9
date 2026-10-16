//! TWO HOLES IN ONE FACE: a hole stands where it was placed on the face, not at the face's centre, so a plate takes
//! a second hole beside the first - the commonest thing there is in a part.
//!
//! Reported behaviour: the second hole in the top of a block came in green and left the body as it was - both stood
//! at the centre of the face.
use qymcad_core::feature::FaceKey;
use qymcad_core::model::{HoleTool, Project};
use std::f64::consts::PI;

#[test]
fn two_holes_in_the_top_of_a_block_take_twice_the_material() {
    let mut p = Project::default();
    p.new_document();
    let block = p.add_box(40.0, 30.0, 10.0);
    let _ = qymcad_testkit::regenerate(&mut p);
    let f = p.regen_faces.get(&block).and_then(|fs| fs.iter().find(|f| f.normal[2] > 0.9)).cloned().expect("the top of the block");
    let key = FaceKey { index: 0, centroid: [f.centroid.x, f.centroid.y, f.centroid.z], normal: f.normal, id: f.id };
    let c = key.centroid;
    let tool = HoleTool { kind: 0, diameter: 6.0, depth: 15.0, dia2: 0.0, depth2: 0.0 };
    let first = p.add_hole_at(block, key, [c[0] - 10.0, c[1], c[2]], tool);
    let (report, shapes) = qymcad_testkit::regenerate(&mut p);
    assert!(report.errors.is_empty(), "the first hole did not build: {:?}", report.errors);
    let one = shapes.get(&first).map(|s| s.volume()).expect("the body with one hole");
    // the second hole, in the same top face of the drilled body, 20 beside the first
    let f = p.regen_faces.get(&first).and_then(|fs| fs.iter().filter(|f| f.normal[2] > 0.9).max_by(|a, b| a.area.total_cmp(&b.area))).cloned().expect("the top of the drilled block");
    let key = FaceKey { index: 0, centroid: [f.centroid.x, f.centroid.y, f.centroid.z], normal: f.normal, id: f.id };
    let second = p.add_hole_at(first, key, [c[0] + 10.0, c[1], c[2]], tool);
    let (report, shapes) = qymcad_testkit::regenerate(&mut p);
    assert!(report.errors.is_empty(), "the second hole did not build: {:?}", report.errors);
    let two = shapes.get(&second).map(|s| s.volume()).expect("the body with two holes");
    let bore = PI * 9.0 * 10.0; // d6 through 10
    assert!((12000.0 - one - bore).abs() < 1.0, "one hole took {} out of the block, a bore of d6 x 10 is {bore:.1}", 12000.0 - one);
    assert!((one - two - bore).abs() < 1.0, "the second hole took {} more, a bore of d6 x 10 is {bore:.1}", one - two);
}

/// THE HOLE STANDS WHERE IT WAS PLACED on the face: its wall is centred on the point given, 12 and 5 off the centre.
#[test]
fn a_hole_stands_where_it_was_placed() {
    let mut p = Project::default();
    p.new_document();
    let block = p.add_box(40.0, 30.0, 10.0);
    let _ = qymcad_testkit::regenerate(&mut p);
    let f = p.regen_faces.get(&block).and_then(|fs| fs.iter().find(|f| f.normal[2] > 0.9)).cloned().expect("the top of the block");
    let key = FaceKey { index: 0, centroid: [f.centroid.x, f.centroid.y, f.centroid.z], normal: f.normal, id: f.id };
    let at = [key.centroid[0] - 12.0, key.centroid[1] + 5.0, key.centroid[2]];
    let hole = p.add_hole_at(block, key, at, HoleTool { kind: 0, diameter: 4.0, depth: 5.0, dia2: 0.0, depth2: 0.0 });
    let _ = qymcad_testkit::regenerate(&mut p);
    let wall = |p: &Project| {
        p.regen_faces
            .get(&hole)
            .and_then(|fs| fs.iter().find(|f| f.normal[2].abs() < 0.1 && (f.area - PI * 4.0 * 5.0).abs() < 2.0).map(|f| [f.centroid.x, f.centroid.y]))
            .expect("the wall of the hole")
    };
    let w = wall(&p);
    assert!((w[0] - at[0]).abs() < 0.3 && (w[1] - at[1]).abs() < 0.3, "the hole stands at {w:?} across the top, it was placed at {at:?}");
}

/// TWO HOLES CHANGE PLACES IN THE TIMELINE: each drills the body the one above it left, and neither leans on what the
/// other made, so they may be swapped - and the part is the same body after. Reported behaviour: both arrows of the
/// menu stood grey on a part with two holes.
#[test]
fn two_holes_change_places_and_the_body_is_the_same() {
    let mut p = Project::default();
    p.new_document();
    let block = p.add_box(40.0, 30.0, 10.0);
    let _ = qymcad_testkit::regenerate(&mut p);
    let f = p.regen_faces.get(&block).and_then(|fs| fs.iter().find(|f| f.normal[2] > 0.9)).cloned().expect("the top of the block");
    let key = FaceKey { index: 0, centroid: [f.centroid.x, f.centroid.y, f.centroid.z], normal: f.normal, id: f.id };
    let c = key.centroid;
    let first = p.add_hole_at(block, key, [c[0] - 10.0, c[1], c[2]], HoleTool { kind: 0, diameter: 6.0, depth: 15.0, dia2: 0.0, depth2: 0.0 });
    let _ = qymcad_testkit::regenerate(&mut p);
    let f = p.regen_faces.get(&first).and_then(|fs| fs.iter().filter(|f| f.normal[2] > 0.9).max_by(|a, b| a.area.total_cmp(&b.area))).cloned().expect("the top of the drilled block");
    let top = FaceKey { index: 0, centroid: [f.centroid.x, f.centroid.y, f.centroid.z], normal: f.normal, id: f.id };
    let second = p.add_hole_at(first, top, [c[0] + 10.0, c[1], c[2]], HoleTool { kind: 0, diameter: 4.0, depth: 15.0, dia2: 0.0, depth2: 0.0 });
    let (report, shapes) = qymcad_testkit::regenerate(&mut p);
    assert!(report.errors.is_empty(), "setup: the two holes do not build: {:?}", report.errors);
    let before = shapes.get(&second).map(|s| s.volume()).expect("the body with two holes");
    let at = |p: &Project, id| p.timeline.iter().position(|n| n.id == id).expect("the node");
    let (i, j) = (at(&p, first), at(&p, second));
    assert!(j == i + 1, "setup: the holes stand one after the other");
    // the menu's "down" on the first moves the second above it
    assert!(p.can_reorder_feature(j, i), "the second hole cannot go above the first, though neither leans on the other");
    assert!(p.reorder_feature(j, i), "the second hole was not moved above the first");
    assert!(at(&p, first) > at(&p, second), "the timeline did not change its order");
    let (report, shapes) = qymcad_testkit::regenerate(&mut p);
    assert!(report.errors.is_empty(), "after the swap the part does not build: {:?}", report.errors);
    let last = p.timeline.iter().rev().find_map(|n| n.kind.body()).expect("the last body");
    let after = shapes.get(&last).map(|s| s.volume()).expect("the body after the swap");
    assert!((before - (12000.0 - PI * 9.0 * 10.0 - PI * 4.0 * 10.0)).abs() < 1.0, "setup: the two holes take {} out of the block", 12000.0 - before);
    assert!((after - before).abs() < 1e-6, "two holes changed places and the body went from {before} to {after}");
}
