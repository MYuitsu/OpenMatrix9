//! TEXT EXTRUDES WHATEVER FONT IT WAS SET IN.
//!
//! Reported behaviour: "one sketch with a font of my own and that is it, Extrude fails - only the built-in
//! font extrudes. I tried four fonts, none of them works." Measured on the reporter's machine: the font that
//! worked was the system default, `NimbusSans-Regular.otf`, and every font he chose himself was a `.ttf`.
//!
//! The two formats give their outlines differently. A CFF font (OTF) closes a contour without repeating the
//! starting point; a glyf font (TTF) closes it BY RETURNING to that point, so the loop ends with two
//! coincident points in a row - a segment of zero length. OCCT refuses to make a wire out of that, the
//! extrusion of the contour comes back empty and the rebuild says "Extrude failed (check the contour)".
//!
//! The fonts also wind the other way round: glyf outer contours run clockwise, CFF ones anticlockwise.
//!
//! The check runs on the font shipped in this repository, which is a `.ttf` - no font of the reporter's is
//! needed to see it.
use qymcad_core::model::{Project, TextSpec};

/// The font shipped in this repository - a `.ttf`, which is the format the defect lives in.
const FONT: &str = "../../assets/fonts/LiberationSans-Bold.ttf";

#[test]
fn a_text_set_in_a_ttf_font_extrudes_into_a_body() {
    let font = std::fs::read(FONT).expect("the font shipped with the repository");
    let loops: Vec<Vec<qymcad_core::geom::Point2>> = qymcad_core::text::text_outline_contours(&font, 0, "Text", 30.0, 0.0, 0.0).into_iter().map(|c| c.points).collect();
    assert!(loops.len() >= 4, "setup: the string gave {} loops", loops.len());
    let glyphs = loops.clone();

    let mut p = Project::default();
    p.new_document();
    let sid = p.add_sketch("text", vec![], None);
    p.add_sketch_node(sid, "Text sketch");
    let si = p.sketch_index(sid).unwrap();
    p.add_sketch_text(
        si,
        TextSpec {
            at: qymcad_core::geom::Point2::new(0.0, 0.0),
            height: 30.0,
            angle: 0.0,
            text: "Text".into(),
            glyphs,
            font: qymcad_core::model::FontRef { family: "Liberation Sans".into(), path: FONT.into(), index: 0 },
        },
        qymcad_core::feature::Purpose::Real,
    );
    let profiles = p.sketches[si].contour_ids.clone();
    assert!(!profiles.is_empty(), "setup: the text gave no contours to extrude");
    p.add_extrude_multi(sid, profiles, 5.0, qymcad_core::feature::Reach::Forward, 0.0, Vec::new());

    let (report, _shapes) = qymcad_testkit::regenerate(&mut p);
    assert!(report.errors.is_empty(), "the text did not extrude: {:?}", report.errors);

    let last = p.timeline.iter().filter_map(|n| n.kind.body()).next_back().expect("a body was made");
    let mi = p.mesh_index(last).expect("the mesh of the body");
    let mesh = &p.bodies[mi].mesh;
    assert!(!mesh.tris.is_empty(), "the body came out empty");

    // EVERY LOOP IT WAS GIVEN IS IN THE BODY. Each loop of a glyph - the outline of a letter and the inner
    // loop of an `e` alike - is a contour of its own, and what to leave out is the person's choice, made by
    // the selection and the fill; an inner loop left in is a plug that fills the letter, not a hole.
    //
    // So the top face has to measure the AREA COVERED by the loops handed over: the outer ones, with an inner
    // one adding nothing since it lies inside its own letter. Measuring that says the extrusion used what it
    // was given and dropped nothing quietly.
    let area = |loop_: &[qymcad_core::geom::Point2]| {
        let mut s = 0.0;
        for i in 0..loop_.len() {
            let (u, v) = (loop_[i], loop_[(i + 1) % loop_.len()]);
            s += u.x * v.y - v.x * u.y;
        }
        (0.5 * s).abs()
    };
    let inside = |q: qymcad_core::geom::Point2, loop_: &[qymcad_core::geom::Point2]| {
        let mut hit = false;
        for i in 0..loop_.len() {
            let (u, v) = (loop_[i], loop_[(i + 1) % loop_.len()]);
            if (u.y > q.y) != (v.y > q.y) && q.x < (v.x - u.x) * (q.y - u.y) / (v.y - u.y) + u.x {
                hit = !hit;
            }
        }
        hit
    };
    let want: f64 = loops.iter().enumerate().filter(|(i, l)| !loops.iter().enumerate().any(|(j, o)| j != *i && area(o) > area(l) && inside(l[0], o))).map(|(_, l)| area(l)).sum();
    let top: f64 = (0..mesh.tris.len())
        .filter_map(|ti| {
            let t = mesh.triangle(ti);
            let n = [
                (t[1].y - t[0].y) * (t[2].z - t[0].z) - (t[1].z - t[0].z) * (t[2].y - t[0].y),
                (t[1].z - t[0].z) * (t[2].x - t[0].x) - (t[1].x - t[0].x) * (t[2].z - t[0].z),
                (t[1].x - t[0].x) * (t[2].y - t[0].y) - (t[1].y - t[0].y) * (t[2].x - t[0].x),
            ];
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            (len > 1e-12 && n[2] / len > 0.9 && t.iter().all(|v| (v.z - 5.0).abs() < 1e-6)).then_some(0.5 * len)
        })
        .sum();
    assert!((top - want).abs() < 1.0, "the top of the extruded text measures {top:.3} mm^2 while the loops handed to it enclose {want:.3} mm^2");
}

/// AND A DOCUMENT SAVED BEFORE THE FIX EXTRUDES WHEN IT IS OPENED.
///
/// The glyph loops are baked once, by the application, and stored in the file as they were baked. The
/// reporter's own document holds five loops, each with its two coincident points still in it - so repairing
/// only the baking would leave every text already drawn broken for good. The loops here are put together the
/// way that file holds them: closed, and with the starting point repeated at the end.
#[test]
fn a_text_saved_with_a_repeated_seam_point_still_extrudes() {
    let font = std::fs::read(FONT).expect("the font shipped with the repository");
    let glyphs: Vec<Vec<qymcad_core::geom::Point2>> = qymcad_core::text::text_outline_contours(&font, 0, "Text", 30.0, 0.0, 0.0)
        .into_iter()
        .map(|c| {
            let mut pts = c.points;
            let first = pts[0];
            pts.push(first); // exactly the state the saved file is in
            pts
        })
        .collect();
    assert!(glyphs.iter().all(|l| l[0].x == l[l.len() - 1].x && l[0].y == l[l.len() - 1].y), "setup: the seam is not repeated");

    let mut p = Project::default();
    p.new_document();
    let sid = p.add_sketch("text", vec![], None);
    p.add_sketch_node(sid, "Text sketch");
    let si = p.sketch_index(sid).unwrap();
    p.add_sketch_text(
        si,
        TextSpec {
            at: qymcad_core::geom::Point2::new(0.0, 0.0),
            height: 30.0,
            angle: 0.0,
            text: "Text".into(),
            glyphs,
            font: qymcad_core::model::FontRef { family: "Liberation Sans".into(), path: FONT.into(), index: 0 },
        },
        qymcad_core::feature::Purpose::Real,
    );
    let profiles = p.sketches[si].contour_ids.clone();
    assert!(!profiles.is_empty(), "setup: the text gave no contours to extrude");
    p.add_extrude_multi(sid, profiles, 5.0, qymcad_core::feature::Reach::Forward, 0.0, Vec::new());

    let (report, _shapes) = qymcad_testkit::regenerate(&mut p);
    assert!(report.errors.is_empty(), "the saved text did not extrude: {:?}", report.errors);
}
