//! SURFACES ACROSS FACES AND EDGES: a face copied out as a surface of its own - flat, round, the wall of a hole, a
//! rounding, a chamfer, a face with a hole in it, two faces at once; a patch spanned across an opening - the open top
//! of a shell, the rim of a hole right through. What each must leave is the number of surfaces and their area, worked
//! out from the geometry; the solid it came from stays as it was.
use std::f64::consts::PI;

use qymcad::{Key, Session};
use qymcad_acceptance::bodies::{BLOCK, CHAMFERED, CYLINDER, HOLED, ROUNDED, THROUGH};
use qymcad_acceptance::matrix::{short, show, spot, Body, Spot};
use qymcad_acceptance::probe;

struct Surfaced {
    body: &'static Body,
    make: fn(&mut Session),
    sheets: usize,
    area: f64,
    what: &'static str,
}

/// The surfaces of the document, and the solids, not taken up into anything.
fn split(s: &mut Session) -> (Vec<qymcad::Solid>, Vec<qymcad::Solid>) {
    s.document().bodies.into_iter().filter(|b| !b.consumed).partition(|b| b.sheet)
}

/// Run every case in a session of its own; the area of a round surface is read off its mesh, whose facets lie a
/// little inside the true surface - within half a percent.
fn run_all(all: Vec<Surfaced>) {
    let mut failed = Vec::new();
    for c in &all {
        let problem = qymcad_acceptance::refusal(|| {
            let mut s = Session::start();
            (c.body.build)(&mut s);
            let (_, solids) = split(&mut s);
            let held: f64 = solids.iter().map(|b| b.volume).sum();
            (c.make)(&mut s);
            let (sheets, solids) = split(&mut s);
            let area: f64 = sheets.iter().map(|b| b.area).sum();
            assert!(sheets.len() == c.sheets, "{} surfaces were made, not {}: {:?}; the status line says {:?}", sheets.len(), c.sheets, sheets.iter().map(|b| b.area).collect::<Vec<_>>(), s.status());
            assert!((area - c.area).abs() <= c.area * 5e-3 + 0.01, "the surfaces hold {area} mm^2, not {}", c.area);
            let now: f64 = solids.iter().map(|b| b.volume).sum();
            assert!((now - held).abs() < 1e-6, "the solid changed: {held} mm^3 became {now}");
        });
        if !problem.is_empty() {
            failed.push(format!("{} / {}: {problem}", c.body.name, c.what));
        }
    }
    assert!(failed.is_empty(), "{} of {} cases did not hold:\n{}", failed.len(), all.len(), failed.join("\n"));
}

fn take(s: &mut Session, hint: &str) {
    let hint = s.word(hint);
    s.press_hint(&hint);
}

/// Copy the faces at the spots out as surfaces.
fn copy(s: &mut Session, faces: &[Spot]) {
    take(s, "tb-face-copy-hint");
    for f in faces {
        let p = show(s, *f);
        let at = s.face_at(p);
        s.click(at);
    }
    s.key(Key::Enter);
}

/// Span a patch across the edges at the spots.
fn patch(s: &mut Session, edges: &[Spot]) {
    take(s, "tb-patch-hint");
    for e in edges {
        let p = show(s, *e);
        let at = s.edge_at(p);
        s.click(at);
    }
    s.key(Key::Enter);
}

/// A block shelled 2 thick, open at the top: an opening of 36 by 26 with four inner edges round it.
fn shelled(s: &mut Session) {
    qymcad_acceptance::bodies::apply(s, "tb-shell-hint", |s| s.face_at([20.0, 15.0, 10.0]), "cmd-thickness", "2");
}

static SHELLED: Body = Body { name: "the block shelled 2 thick, open at the top", build: shelled_block };

fn shelled_block(s: &mut Session) {
    qymcad_acceptance::build::block(s);
    shelled(s);
}

fn surf(body: &'static Body, make: fn(&mut Session), sheets: usize, area: f64, what: &'static str) -> Surfaced {
    Surfaced { body, make, sheets, area, what }
}

probe! {
    budget = 1800;
    /// A FACE COPIED OUT as a surface: the top of a block, its front, the side of a cylinder, the wall of a hole, the
    /// top round the hole, a rounding, a chamfer, and two faces at once.
    fn faces_copied_out() {
        run_all(vec![
            surf(&BLOCK, |s| copy(s, &[spot([20.0, 15.0, 10.0], [10.0, 8.0, 10.0])]), 1, 1200.0, "the top of the block"),
            surf(&BLOCK, |s| copy(s, &[spot([20.0, 0.0, 5.0], [10.0, 0.0, 5.0])]), 1, 400.0, "the front of the block"),
            surf(&BLOCK, |s| copy(s, &[spot([20.0, 15.0, 10.0], [10.0, 8.0, 10.0]), spot([40.0, 15.0, 5.0], [40.0, 8.0, 5.0])]), 1, 1500.0, "the top and the right of the block together"),
            surf(&CYLINDER, |s| copy(s, &[spot([27.071, 7.929, 5.0], [27.071, 7.929, 3.0])]), 1, 2.0 * PI * 10.0 * 10.0, "the side of the cylinder"),
            surf(&HOLED, |s| copy(s, &[spot([20.0, 20.0, 7.5], [21.71, 19.7, 7.5])]), 1, 2.0 * PI * 5.0 * 5.0, "the wall of the hole"),
            surf(&HOLED, |s| copy(s, &[spot([8.0, 8.0, 10.0], [32.0, 8.0, 10.0])]), 1, 1200.0 - PI * 25.0, "the top round the hole"),
            surf(&ROUNDED, |s| copy(s, &[short([20.0, 0.586, 9.414], [10.0, 0.586, 9.414], [20.0, 2.0, 10.0])]), 1, PI * 40.0, "the face of the rounding"),
            surf(&CHAMFERED, |s| copy(s, &[short([20.0, 1.0, 9.0], [10.0, 1.0, 9.0], [20.0, 2.0, 10.0])]), 1, 40.0 * 2.0 * 2f64.sqrt(), "the face of the chamfer"),
        ]);
    }
}

probe! {
    budget = 1800;
    /// A PATCH across an opening: the four inner edges of the open top of a shell (36 by 26), the rim of a hole right
    /// through a block (a disc of 5).
    fn patches_across_openings() {
        run_all(vec![
            surf(
                &SHELLED,
                |s| {
                    patch(
                        s,
                        &[
                            spot([20.0, 2.0, 10.0], [10.0, 2.0, 10.0]),
                            spot([38.0, 15.0, 10.0], [38.0, 8.0, 10.0]),
                            qymcad_acceptance::matrix::Spot { at: [20.0, 28.0, 10.0], alt: [10.0, 28.0, 10.0], from: [1, 1, 1], end: None },
                            qymcad_acceptance::matrix::Spot { at: [2.0, 15.0, 10.0], alt: [2.0, 8.0, 10.0], from: [-1, -1, 1], end: None },
                        ],
                    )
                },
                1,
                36.0 * 26.0,
                "the open top of the shell",
            ),
            surf(&THROUGH, |s| patch(s, &[spot([20.0, 10.0, 10.0], [23.54, 11.46, 10.0])]), 1, PI * 25.0, "the rim of the hole right through"),
        ]);
    }
}

/// Stitch the surfaces lying on the faces at the spots into one.
fn stitch(s: &mut Session, faces: &[Spot]) {
    take(s, "tb-stitch-hint");
    for f in faces {
        let p = show(s, *f);
        let at = s.face_at(p);
        s.click(at);
    }
    s.key(Key::Enter);
}

/// Trim the surface at `keep`, where it is kept, by the body whose face is at `tool`.
fn trim(s: &mut Session, keep: [f64; 3], tool: [f64; 3]) {
    take(s, "tb-trim-surface-hint");
    let at = s.face_at(keep);
    s.click(at);
    let at = s.face_at(tool);
    s.click(at);
    s.key(Key::Enter);
}

/// The block with its top copied out as a surface, and after it a boss 10 across about (20, 15) grown 20 from the
/// plane under the block, 10 above its top: the body crosses the surface along the circle of the boss.
fn copied_then_bossed(s: &mut Session) {
    qymcad_acceptance::build::block(s);
    copy(s, &[spot([20.0, 15.0, 10.0], [10.0, 8.0, 10.0])]);
    let xy = s.word("plane-xy-table");
    s.press_word(&xy);
    qymcad_acceptance::build::circle(s, (20.0, 15.0), (25.0, 15.0));
    let finish = s.word("wb-finish");
    s.press_word(&finish);
    take(s, "tb-extrude-hint");
    let length = s.word("f-length");
    s.fill(&length, "20").key(Key::Enter);
}

/// The block with its top copied out as a surface and nothing crossing it: the surface lies on the block's own face.
fn copied_top(s: &mut Session) {
    qymcad_acceptance::build::block(s);
    copy(s, &[spot([20.0, 15.0, 10.0], [10.0, 8.0, 10.0])]);
}

/// The block with its top and its front copied out as two surfaces, meeting along the top front edge.
fn copied_top_and_front(s: &mut Session) {
    copied_top(s);
    copy(s, &[spot([20.0, 0.0, 5.0], [10.0, 0.0, 5.0])]);
}

/// The block with its top and its right copied out as two surfaces, meeting along the top right edge.
fn copied_top_and_right(s: &mut Session) {
    copied_top(s);
    copy(s, &[spot([40.0, 15.0, 5.0], [40.0, 8.0, 5.0])]);
}

static COPIED_THEN_BOSSED: Body = Body { name: "the block with its top copied out, then a boss grown through it", build: copied_then_bossed };
static COPIED_TOP: Body = Body { name: "the block with its top copied out", build: copied_top };
static COPIED_TOP_AND_FRONT: Body = Body { name: "the block with its top and front copied out", build: copied_top_and_front };
static COPIED_TOP_AND_RIGHT: Body = Body { name: "the block with its top and right copied out", build: copied_top_and_right };

probe! {
    budget = 1800;
    /// A SURFACE TRIMMED along a body crossing it: the top kept round the boss, the disc under it cut away; a body that
    /// does not cross the surface - it lies on the block's own face - refused, the surface whole.
    fn surfaces_trimmed_along_bodies() {
        run_all(vec![
            surf(&COPIED_THEN_BOSSED, |s| trim(s, [5.0, 5.0, 10.0], [20.0, 15.0, 20.0]), 1, 1200.0 - PI * 25.0, "the top kept round the boss"),
            surf(&COPIED_THEN_BOSSED, |s| trim(s, [35.0, 25.0, 10.0], [20.0, 15.0, 20.0]), 1, 1200.0 - PI * 25.0, "the top kept from its far corner"),
            surf(&COPIED_TOP, |s| trim(s, [5.0, 5.0, 10.0], [20.0, 0.0, 5.0]), 1, 1200.0, "the top by the block it lies on: refused"),
        ]);
    }
}

probe! {
    budget = 1800;
    /// SURFACES STITCHED into one along the edge they share: the top and the front, the top and the right - one open
    /// surface holding both areas and no volume.
    fn surfaces_stitched_along_shared_edges() {
        run_all(vec![
            surf(&COPIED_TOP_AND_FRONT, |s| stitch(s, &[spot([20.0, 15.0, 10.0], [10.0, 8.0, 10.0]), spot([20.0, 0.0, 5.0], [10.0, 0.0, 5.0])]), 1, 1600.0, "the top and the front"),
            surf(&COPIED_TOP_AND_RIGHT, |s| stitch(s, &[spot([20.0, 15.0, 10.0], [10.0, 8.0, 10.0]), spot([40.0, 15.0, 5.0], [40.0, 8.0, 5.0])]), 1, 1500.0, "the top and the right"),
        ]);
    }
}
