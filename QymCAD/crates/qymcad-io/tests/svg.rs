//! A test of the SVG import.

use qymcad_io::import_svg;
use std::io::Write;

const FIXTURE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100mm" height="80mm" viewBox="0 0 100 80">
  <rect x="10" y="10" width="40" height="30"/>
  <circle cx="70" cy="40" r="15"/>
</svg>"#;

#[test]
fn imports_rect_and_circle() {
    use qymcad_core::feature::{BasePlane, SketchPlane};
    use qymcad_core::model::Project;
    let path = std::env::temp_dir().join("qymcad_test.svg");
    std::fs::File::create(&path).unwrap().write_all(FIXTURE.as_bytes()).unwrap();

    let sk = import_svg(path.to_str().unwrap()).expect("import ok");
    // SVG lowers into Beziers and then segments; `usvg` keeps no circle or arc primitives
    assert!(!sk.curves.is_empty(), "there are curves");

    // assemble into a sketch and check the closed contours by area: a rectangle of about 1200 and a circle of
    // about 707
    let mut p = Project::default();
    p.new_document();
    let si = p.import_sketch("t.svg", sk.curves, None, SketchPlane::World(BasePlane::XY));
    let areas: Vec<f64> = p.sketches[si].contour_ids.iter().filter_map(|cid| p.contour_index(*cid)).map(|ci| p.contours[ci].area()).collect();

    assert!(areas.iter().any(|a| (a - 1200.0).abs() < 5.0), "a rectangle of about 1200: {areas:?}");
    assert!(areas.iter().any(|a| (a - std::f64::consts::PI * 225.0).abs() < 30.0), "a circle of about 707: {areas:?}");
}

/// Area of every closed contour the file makes, in mm^2.
fn areas_of(svg: &str, name: &str) -> Vec<f64> {
    use qymcad_core::feature::{BasePlane, SketchPlane};
    use qymcad_core::model::Project;
    let path = std::env::temp_dir().join(name);
    std::fs::File::create(&path).unwrap().write_all(svg.as_bytes()).unwrap();
    let sk = import_svg(path.to_str().unwrap()).expect("import ok");
    let mut p = Project::default();
    p.new_document();
    let si = p.import_sketch(name, sk.curves, None, SketchPlane::World(BasePlane::XY));
    p.sketches[si].contour_ids.iter().filter_map(|cid| p.contour_index(*cid)).map(|ci| p.contours[ci].area()).collect()
}

/// THE UNIT OF THE SHEET IS READ. A sheet in millimetres with a viewBox in the same numbers puts one user unit to a
/// millimetre: a 40 x 30 rectangle comes in 40 x 30. Reported behaviour: it came in 151.18 x 113.39, every length
/// 96/25.4 times too long - the reader took CSS pixels for millimetres.
#[test]
fn a_sheet_in_millimetres_comes_in_millimetres() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="42mm" height="32mm" viewBox="0 0 42 32"><rect x="1" y="1" width="40" height="30" fill="none" stroke="black"/></svg>"#;
    let areas = areas_of(svg, "qymcad_test_mm.svg");
    assert!(areas.iter().any(|a| (a - 1200.0).abs() < 1.0), "a 40 x 30 rectangle on a sheet in mm came in as {areas:?}, not 1200");
}

/// A SHEET WITH NO UNIT IS IN CSS PIXELS, 96 TO THE INCH, as the SVG specification says: 96 x 96 user units are
/// 25.4 x 25.4 mm.
#[test]
fn a_sheet_with_no_unit_is_in_pixels_of_a_96th_of_an_inch() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200"><rect x="10" y="10" width="96" height="96" fill="none" stroke="black"/></svg>"#;
    let areas = areas_of(svg, "qymcad_test_px.svg");
    assert!(areas.iter().any(|a| (a - 25.4 * 25.4).abs() < 1.0), "a 96 px square came in as {areas:?}, not 25.4 x 25.4 mm");
}
