//! A SHELL CENTRED ON THE SURFACE lays half its wall outward and half inward, the open face open where it was. A block
//! 40 x 30 x 10 opened on top with a wall of 2: the outward half is 2600 + 45 pi + 2 pi / 3 = 2743.46 mm^3 (the sides and
//! the bottom moved out by 1, their edges quarter cylinders, their corners eighths of a ball), the inward half
//! 12000 - 38 x 28 x 9 = 2424, together 5167.46 - with a hollow, and nothing above the open face.
//!
//! Reported behaviour: the centred shell came out 16055.5 mm^3, more than the solid block of 12000, with no hollow at
//! all and its wall standing 1 mm above the open face.
use qymcad_core::feature::ShellSide;
use qymcad_core::model::Project;

#[test]
fn a_centred_shell_is_hollow_and_stays_below_its_open_face() {
    let mut p = Project::default();
    p.new_document();
    let block = p.add_box(40.0, 30.0, 10.0);
    let _ = qymcad_testkit::regenerate(&mut p);
    let top = p.regen_faces.get(&block).and_then(|fs| fs.iter().find(|f| f.normal[2] > 0.99).map(|f| f.id)).expect("the top face");
    let shell = p.add_shell_mode(block, 2.0, vec![top], ShellSide::Centred);
    let (report, shapes) = qymcad_testkit::regenerate(&mut p);
    assert!(report.errors.is_empty(), "the centred shell refused: {:?}", report.errors);
    let s = shapes.get(&shell).expect("the shell's body");
    let want = 2600.0 + 45.0 * std::f64::consts::PI + 2.0 * std::f64::consts::PI / 3.0 + 2424.0;
    assert!((s.volume() - want).abs() < 0.05, "a centred wall of 2 on a block opened on top holds {:.2} mm^3, not {want:.2}", s.volume());
    let b = s.bbox().expect("the extents");
    assert!(b[5] < 10.0 + 0.01, "the wall stands above the open face, up to z = {:.3}", b[5]);
    assert!((b[0] + 21.0).abs() < 0.05 && (b[2] + 1.0).abs() < 0.05, "the wall goes out by half its thickness: {b:?}");
}
