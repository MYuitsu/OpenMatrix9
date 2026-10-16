//! A THICKENED FACE OF A PART: a plate grown out of the face and welded on. Grown into the body it adds nothing and
//! says so; grown on the top of a cylinder it makes the cylinder taller, its side one face as a block's sides are.
//!
//! Reported behaviour: -2 stood green and left the body as it was; the top of a cylinder thickened 3 left the side in
//! two cylindrical faces with a seam where the top had been.
use qymcad_core::model::Project;

fn thickened_top(p: &mut Project, body: u64, t: f64) -> (u64, Vec<String>) {
    let _ = qymcad_testkit::regenerate(p);
    let top = p.regen_faces.get(&body).and_then(|fs| fs.iter().filter(|f| f.normal[2] > 0.9).max_by(|a, b| a.centroid.z.total_cmp(&b.centroid.z))).map(|f| f.id).expect("the top face");
    let th = p.add_thicken(body, top, t);
    let (report, _) = qymcad_testkit::regenerate(p);
    (th, report.errors.iter().filter(|(id, _)| *id == th).map(|(_, e)| format!("{e:?}")).collect())
}

#[test]
fn a_plate_grown_into_the_body_is_refused_in_words() {
    let mut p = Project::default();
    p.new_document();
    let block = p.add_box(40.0, 30.0, 10.0);
    let (_, errors) = thickened_top(&mut p, block, -2.0);
    assert!(!errors.is_empty(), "the top of the block thickened -2 grew the plate into the body and stood green");
}

#[test]
fn the_top_of_a_cylinder_thickened_makes_it_taller_with_one_side() {
    let mut p = Project::default();
    p.new_document();
    // extruded from a circle of the sketch, as a person makes one
    let si = p.new_sketch("Base");
    let sid = p.sketches[si].id;
    p.add_sketch_node(sid, "Base");
    p.add_circle_entity(si, 0.0, 0.0, 10.0, qymcad_core::feature::Purpose::Real);
    p.regen_sketch(si);
    let c: Vec<u64> = p.sketches[si].contour_ids.iter().copied().filter(|c| p.contour_profile_xy(*c).is_some()).collect();
    let e = p.add_extrude_multi(sid, c, 10.0, qymcad_core::feature::Reach::Forward, 0.0, vec![]);
    let cyl = p.finish_base_body(e, 1);
    let (th, errors) = thickened_top(&mut p, cyl, 3.0);
    assert!(errors.is_empty(), "the top of the cylinder thickened 3 was refused: {errors:?}");
    let (report, shapes) = qymcad_testkit::regenerate(&mut p);
    assert!(report.errors.is_empty(), "{:?}", report.errors);
    let v = shapes.get(&th).map(|s| s.volume()).unwrap_or(0.0);
    let want = std::f64::consts::PI * 100.0 * 13.0;
    assert!((v - want).abs() < 1.0, "the cylinder thickened 3 on top is {v:.1} mm^3, a cylinder of 10 x 13 is {want:.1}");
    let sides = p.regen_faces.get(&th).map_or(0, |fs| fs.iter().filter(|f| f.normal[2].abs() < 0.5).count());
    assert_eq!(sides, 1, "the side of the taller cylinder is {sides} faces, a seam where the top was");
}
