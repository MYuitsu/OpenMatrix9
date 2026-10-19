//! AN OFFSET SURFACE MOVES ITS FACES along their normals into a sheet of their own: the top of a block lifted 5, the
//! side of a cylinder grown by 2, a distance that turns a face inside out refused in words, zero a copy in place.
use qymcad_core::refs::Ref;
use qymcad_core::model::Project;

/// The id of the face of `body` whose normal points along `n`, the largest such.
fn face_along(p: &Project, body: u64, n: [f64; 3]) -> u32 {
    let dot = |a: [f64; 3]| a[0] * n[0] + a[1] * n[1] + a[2] * n[2];
    p.regen_faces[&body].iter().filter(|f| dot(f.normal) > 0.99).max_by(|a, b| a.area.total_cmp(&b.area)).map(|f| f.id).expect("a face along the normal")
}

/// The area of the sheet, as the faces of its body add up.
fn area(p: &Project, sheet: u64) -> f64 {
    p.regen_faces.get(&sheet).map(|fs| fs.iter().map(|f| f.area).sum()).unwrap_or(0.0)
}

#[test]
fn the_top_of_a_block_lifted_5_is_a_sheet_of_1200_at_15() {
    let mut p = Project::default();
    p.new_document();
    let block = p.add_box(40.0, 30.0, 10.0);
    let _ = qymcad_testkit::regenerate(&mut p);
    let top = face_along(&p, block, [0.0, 0.0, 1.0]);
    let sheet = p.add_offset_surface(block, Ref::picks(&[top]), 5.0);
    let (report, shapes) = qymcad_testkit::regenerate(&mut p);
    assert!(report.errors.is_empty(), "the offset did not build: {:?}", report.errors);
    assert!((area(&p, sheet) - 1200.0).abs() < 0.5, "the sheet is {:.1} mm^2, the top 40 x 30 is 1200", area(&p, sheet));
    let b = shapes.get(&sheet).and_then(|s| s.bbox()).expect("the sheet has a box");
    assert!((b[2] - 15.0).abs() < 1e-3 && (b[5] - 15.0).abs() < 1e-3, "the sheet lies at z {:.3}..{:.3}, the top of the block 10 lifted 5 is 15", b[2], b[5]);
    assert!(p.bodies.iter().any(|x| x.id == block && x.visible), "the block went: an offset reads its source, it does not consume it");
}

#[test]
fn the_side_of_a_cylinder_grown_by_2_is_radius_12() {
    let mut p = Project::default();
    p.new_document();
    let cyl = p.add_cylinder(10.0, 20.0);
    let _ = qymcad_testkit::regenerate(&mut p);
    let side = p.regen_faces[&cyl].iter().max_by(|a, b| a.area.total_cmp(&b.area)).map(|f| f.id).expect("the side of the cylinder");
    let sheet = p.add_offset_surface(cyl, Ref::picks(&[side]), 2.0);
    let (report, _) = qymcad_testkit::regenerate(&mut p);
    assert!(report.errors.is_empty(), "the offset did not build: {:?}", report.errors);
    let want = 2.0 * std::f64::consts::PI * 12.0 * 20.0;
    assert!((area(&p, sheet) - want).abs() / want < 0.01, "the sheet is {:.1} mm^2, the side of radius 12 and height 20 is {want:.1}", area(&p, sheet));
}

#[test]
fn a_distance_that_turns_the_face_inside_out_is_refused() {
    let mut p = Project::default();
    p.new_document();
    let cyl = p.add_cylinder(10.0, 20.0);
    let _ = qymcad_testkit::regenerate(&mut p);
    let side = p.regen_faces[&cyl].iter().max_by(|a, b| a.area.total_cmp(&b.area)).map(|f| f.id).expect("the side of the cylinder");
    let sheet = p.add_offset_surface(cyl, Ref::picks(&[side]), -12.0);
    let (report, _) = qymcad_testkit::regenerate(&mut p);
    let node = p.timeline.iter().find(|n| n.kind.body() == Some(sheet)).map(|n| n.id).expect("the node");
    assert!(report.errors.iter().any(|(id, _)| *id == node), "a radius of 10 moved in by 12 was not refused: errors {:?}, area {:.1}", report.errors, area(&p, sheet));
}

#[test]
fn zero_is_a_copy_in_place() {
    let mut p = Project::default();
    p.new_document();
    let block = p.add_box(40.0, 30.0, 10.0);
    let _ = qymcad_testkit::regenerate(&mut p);
    let top = face_along(&p, block, [0.0, 0.0, 1.0]);
    let sheet = p.add_offset_surface(block, Ref::picks(&[top]), 0.0);
    let (report, shapes) = qymcad_testkit::regenerate(&mut p);
    assert!(report.errors.is_empty(), "the copy in place did not build: {:?}", report.errors);
    let b = shapes.get(&sheet).and_then(|s| s.bbox()).expect("the sheet has a box");
    assert!((b[2] - 10.0).abs() < 1e-3 && (area(&p, sheet) - 1200.0).abs() < 0.5, "a zero offset is the top in place: z {:.3}, {:.1} mm^2", b[2], area(&p, sheet));
}

/// THE SHEET FOLLOWS THE BODY: the block made 20 tall instead of 10, the sheet 5 over its top rises to 25.
#[test]
fn the_sheet_rises_with_the_top_of_the_block() {
    let mut p = Project::default();
    p.new_document();
    let block = p.add_box(40.0, 30.0, 10.0);
    let _ = qymcad_testkit::regenerate(&mut p);
    let top = face_along(&p, block, [0.0, 0.0, 1.0]);
    let sheet = p.add_offset_surface(block, Ref::picks(&[top]), 5.0);
    let _ = qymcad_testkit::regenerate(&mut p);
    for n in p.timeline.iter_mut() {
        if let qymcad_core::feature::FeatureKind::Box3 { dz, .. } = &mut n.kind {
            *dz = 20.0;
            n.dirty = true;
        }
    }
    let (report, shapes) = qymcad_testkit::regenerate(&mut p);
    assert!(report.errors.is_empty(), "the taller block or its sheet did not build: {:?}", report.errors);
    let b = shapes.get(&sheet).and_then(|s| s.bbox()).expect("the sheet has a box");
    assert!((b[2] - 25.0).abs() < 1e-3, "the sheet lies at z {:.3} over a block 20 tall, 25 was expected", b[2]);
}

/// FACES MEETING AT A SHARP EDGE MOVE APART: the top and the front of the block, lifted 5 and moved forward 5, are two
/// sheets of 1200 and 400 in one surface body.
#[test]
fn the_top_and_the_front_move_as_two_sheets() {
    let mut p = Project::default();
    p.new_document();
    let block = p.add_box(40.0, 30.0, 10.0);
    let _ = qymcad_testkit::regenerate(&mut p);
    let top = face_along(&p, block, [0.0, 0.0, 1.0]);
    let front = face_along(&p, block, [0.0, -1.0, 0.0]);
    let sheet = p.add_offset_surface(block, Ref::picks(&[top, front]), 5.0);
    let (report, _) = qymcad_testkit::regenerate(&mut p);
    assert!(report.errors.is_empty(), "the two faces did not offset: {:?}", report.errors);
    assert!((area(&p, sheet) - 1600.0).abs() < 0.5, "the sheets are {:.1} mm^2, the top 1200 and the front 400 make 1600", area(&p, sheet));
}

/// THE TOP REPLACED BY THE SHEET LIFTED 5: the sides of the block reach up to it, the block 15 tall - 18000.
#[test]
fn the_top_replaced_by_the_lifted_sheet_makes_the_block_taller() {
    let mut p = Project::default();
    p.new_document();
    let block = p.add_box(40.0, 30.0, 10.0);
    let _ = qymcad_testkit::regenerate(&mut p);
    let top = face_along(&p, block, [0.0, 0.0, 1.0]);
    let sheet = p.add_offset_surface(block, Ref::picks(&[top]), 5.0);
    let _ = qymcad_testkit::regenerate(&mut p);
    let replaced = p.add_surface_replace(block, Ref::picks(&[top]), sheet);
    let (report, shapes) = qymcad_testkit::regenerate(&mut p);
    let v = shapes.get(&replaced).map(|s| s.volume()).unwrap_or(0.0);
    assert!(report.errors.is_empty() && (v - 18000.0).abs() < 1.0, "the block with its top replaced is {v:.1} mm^3, 40 x 30 x 15 is 18000; errors {:?}", report.errors);
}

/// THE TOP REPLACED BY A SHEET SUNK 3: the block is cut down to it, 7 tall - 8400.
#[test]
fn the_top_replaced_by_a_sunk_sheet_makes_the_block_lower() {
    let mut p = Project::default();
    p.new_document();
    let block = p.add_box(40.0, 30.0, 10.0);
    let _ = qymcad_testkit::regenerate(&mut p);
    let top = face_along(&p, block, [0.0, 0.0, 1.0]);
    let sheet = p.add_offset_surface(block, Ref::picks(&[top]), -3.0);
    let _ = qymcad_testkit::regenerate(&mut p);
    let replaced = p.add_surface_replace(block, Ref::picks(&[top]), sheet);
    let (report, shapes) = qymcad_testkit::regenerate(&mut p);
    let v = shapes.get(&replaced).map(|s| s.volume()).unwrap_or(0.0);
    assert!(report.errors.is_empty() && (v - 8400.0).abs() < 1.0, "the block with its top replaced is {v:.1} mm^3, 40 x 30 x 7 is 8400; errors {:?}", report.errors);
}
