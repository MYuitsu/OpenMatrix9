//! A ROUNDING AS WIDE AS THE WALL IT RUNS INTO eats that wall and leaves no crack of it. A pocket w wide is cut into a
//! block and one corner of its end rounded R = w: the rounding takes on the step back of 2e-8 (the limiting case), and
//! the end wall must go with it - one face in, one face out, the count of faces of the body as it was.
//!
//! Reported behaviour: of four roundings R2 at the corners of two pockets 2 wide, the one side of the part took and the
//! other did not - 2 faces instead of 4. The rounding before them, R2 on the far corners, had left 4e-8 of each end
//! wall standing, a strip of 1.6e-7 mm^2 beside the corner the next rounding asked for; with the strips taken out all
//! four take.
use qymcad_core::feature::{Purpose, Reach};
use qymcad_core::model::{CombineSpan, Project};

/// A block 60 x 60 x 10, centred on the origin in plan, with a pocket from (x0, y0) to (x1, y1), 4 deep from its bottom.
fn pocket(x0: f64, y0: f64, x1: f64, y1: f64) -> (Project, u64) {
    let mut p = Project::default();
    p.new_document();
    let block = p.add_box(60.0, 60.0, 10.0);
    let si = p.new_sketch("Pocket");
    let sid = p.sketches[si].id;
    p.add_sketch_node(sid, "Pocket");
    p.add_rect_entity(si, x0, y0, x1, y1, Purpose::Real);
    p.regen_sketch(si);
    let c: Vec<u64> = p.sketches[si].contour_ids.iter().copied().filter(|c| p.contour_profile_xy(*c).is_some()).collect();
    let cut = p.add_combine_multi_op(block, sid, c, CombineSpan { height: 4.0, down: 0.0, extent: qymcad_core::feature::Extent { reach: Reach::Forward, ..Default::default() }, fill: &[] }, 0);
    let (report, _) = qymcad_testkit::regenerate(&mut p);
    assert!(report.errors.is_empty(), "setup: the pocket did not cut: {:?}", report.errors);
    (p, cut)
}

/// The upright edge of body `body` at (x, y).
fn upright(p: &Project, body: u64, x: f64, y: f64) -> u32 {
    p.regen_edges
        .get(&body)
        .and_then(|es| es.iter().find(|e| (e.mid[0] - x).abs() < 1e-3 && (e.mid[1] - y).abs() < 1e-3 && (e.a[2] - e.b[2]).abs() > 1.0).map(|e| e.id))
        .unwrap_or_else(|| panic!("no upright edge at ({x}, {y})"))
}

/// The faces of the body, all of them - the tessellated list leaves a strip too narrow to draw out.
fn faces(shapes: &std::collections::HashMap<u64, qymcad_kernel::Shape>, body: u64) -> u32 {
    shapes.get(&body).and_then(|s| s.face_kinds()).map(|k| k.iter().sum()).unwrap_or(0)
}

#[test]
fn a_rounding_as_wide_as_its_wall_takes_the_wall_away() {
    let mut fails: Vec<String> = Vec::new();
    // the pocket across x and across y, rounded at either end; 2 and 3 wide
    for (name, w, rect, corner) in [
        ("across x, low end", 2.0, (13.0, -10.0, 15.0, 0.0), (13.0, -10.0)),
        ("across x, high end", 2.0, (13.0, -10.0, 15.0, 0.0), (15.0, 0.0)),
        ("across y, low end", 2.0, (-10.0, 13.0, 0.0, 15.0), (-10.0, 15.0)),
        ("across x, 3 wide", 3.0, (-20.0, -10.0, -17.0, 0.0), (-20.0, -10.0)),
    ] {
        let (mut p, cut) = pocket(rect.0, rect.1, rect.2, rect.3);
        let (_, shapes) = qymcad_testkit::regenerate(&mut p);
        let before = faces(&shapes, cut);
        let e = upright(&p, cut, corner.0, corner.1);
        let f = p.add_fillet(cut, w, vec![e]);
        let (report, shapes) = qymcad_testkit::regenerate(&mut p);
        if let Some((_, e)) = report.errors.iter().find(|(id, _)| *id == f) {
            fails.push(format!("{name}: the rounding R{w} refused - {e:?}"));
            continue;
        }
        let after = faces(&shapes, f);
        if after != before {
            fails.push(format!("{name}: R{w} as wide as the wall - {before} faces -> {after}, the wall not taken away whole"));
        }
    }
    assert!(fails.is_empty(), "\nROUNDINGS AS WIDE AS THEIR WALL ({}):\n{}", fails.len(), fails.join("\n"));
}
