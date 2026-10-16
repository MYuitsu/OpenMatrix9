//! WHAT A NAMED SET OF LINES MAKES OF CORNERS.
//!
//! Reported: naming a corner took one pair of lines at a time, so a shape whose corners were all wanted had to be
//! rounded one corner, then the next, then the next - each one its own value and its own step of undo, or a value
//! typed again and again into a field that opened over a different corner every time. A person naming lines is
//! naming a *contour*, and a contour has all of its corners in it at once.
//!
//! So a set of lines is read for the corners it makes: **every pair of them that meets**, which is three corners of a
//! triangle and two corners of two separate corners and none at all for a line named on its own. And where **more than
//! two of them stand on one point** - a cross of four lines, a T-joint - every pair of them would be a corner of that
//! one place, so they are taken **as they were named, two at a time**: the third line joining a corner two lines have
//! already made adds nothing, and a fourth beside it makes the second corner. The order the lines were named in is
//! the whole of that rule, which is why it is carried and not recomputed.
use qymcad_core::feature::Purpose;
use qymcad_core::model::Project;

/// A LINE OF THE SKETCH THE TEST ASKS FOR, from one end to the other.
///
/// A struct and not a pair of pairs: a pair of pairs is a type long enough to want a name, and the rules of the
/// code do not let a tuple be named by one - so what is named here has fields.
struct Leg {
    a: (f64, f64),
    b: (f64, f64),
}

/// The same line, written as the test writes it: from here to there.
const fn leg(a: (f64, f64), b: (f64, f64)) -> Leg {
    Leg { a, b }
}

/// A fresh project holding one sketch, with a line from every leg given.
fn lines_of(legs: &[Leg]) -> (Project, usize, Vec<u64>) {
    let mut p = Project::default();
    p.new_document();
    let si = p.new_sketch("S");
    let ids: Vec<u64> = legs.iter().map(|l| p.add_line_entity(si, l.a.0, l.a.1, l.b.0, l.b.1, Purpose::Real)).collect();
    p.regen_sketch(si);
    (p, si, ids)
}

/// The same pairs whichever way round each of them is written: which corner is made is the question, and it does not
/// depend on which of the two lines of it was named first.
fn same_pairs(v: Vec<(u64, u64)>) -> Vec<(u64, u64)> {
    let mut out: Vec<(u64, u64)> = v.into_iter().map(|(a, b)| if a <= b { (a, b) } else { (b, a) }).collect();
    out.sort_unstable();
    out
}

/// Three lines closing a triangle.
fn triangle() -> (Project, usize, Vec<u64>) {
    lines_of(&[leg((0.0, 0.0), (40.0, 0.0)), leg((40.0, 0.0), (20.0, 30.0)), leg((20.0, 30.0), (0.0, 0.0))])
}

/// Four lines of one chain, each meeting the next: a stair of three corners.
fn chain_of_four() -> (Project, usize, Vec<u64>) {
    lines_of(&[leg((0.0, 0.0), (20.0, 0.0)), leg((20.0, 0.0), (20.0, 20.0)), leg((20.0, 20.0), (40.0, 20.0)), leg((40.0, 20.0), (40.0, 40.0))])
}

/// Two separate corners, far from each other, with nothing joining them.
fn two_separate_corners() -> (Project, usize, Vec<u64>) {
    lines_of(&[leg((0.0, 0.0), (20.0, 0.0)), leg((20.0, 0.0), (20.0, 20.0)), leg((100.0, 0.0), (120.0, 0.0)), leg((120.0, 0.0), (120.0, 20.0))])
}

/// Two squares sharing the point (20, 20): four lines stand on that one point.
fn cross_at_one_point() -> (Project, usize, Vec<u64>, u64) {
    let (p, si, _) = lines_of(&[
        leg((0.0, 0.0), (20.0, 0.0)),
        leg((20.0, 0.0), (20.0, 20.0)),
        leg((20.0, 20.0), (0.0, 20.0)),
        leg((0.0, 20.0), (0.0, 0.0)),
        leg((20.0, 20.0), (40.0, 20.0)),
        leg((40.0, 20.0), (40.0, 40.0)),
        leg((40.0, 40.0), (20.0, 40.0)),
        leg((20.0, 40.0), (20.0, 20.0)),
    ]);
    let shared = p.sketches[si].points.iter().find(|q| (q.x - 20.0).abs() < 1e-6 && (q.y - 20.0).abs() < 1e-6).expect("the point the two squares share").id;
    assert_eq!(p.vertex_edges(si, shared).len(), 4, "setup: four lines do not stand on the point");
    let edges = p.vertex_edges(si, shared);
    (p, si, edges, shared)
}

#[test]
fn a_triangle_of_three_lines_is_three_corners() {
    let (p, si, l) = triangle();
    let made = p.corners_of_lines(si, &l);
    assert_eq!(made.len(), 3, "a closed contour of three lines is three corners, not one and not two");
    assert_eq!(same_pairs(made), same_pairs(vec![(l[0], l[1]), (l[1], l[2]), (l[2], l[0])]), "each line of the contour with each other one");
    assert_eq!(p.corners_of_lines(si, &l[..2]).len(), 1, "two of the three make one corner and wait for the third");
}

#[test]
fn a_line_named_on_its_own_makes_no_corner() {
    let (p, si, l) = chain_of_four();
    assert!(p.corners_of_lines(si, &l[..1]).is_empty(), "one line is half of a corner, not a corner");
    assert!(p.corners_of_lines(si, &[]).is_empty(), "no lines, no corner");
}

#[test]
fn a_line_naming_nothing_of_the_set_adds_no_corner_and_takes_none_away() {
    let (p, si, l) = lines_of(&[
        leg((0.0, 0.0), (20.0, 0.0)),
        leg((20.0, 0.0), (20.0, 20.0)),
        // on the other side of the sheet: it meets nothing of the set
        leg((200.0, 0.0), (220.0, 0.0)),
    ]);
    // named between the two lines of the first corner: it stands in no corner at all, and the corner named before it
    // is still there afterwards
    assert_eq!(p.corners_of_lines(si, &[l[0], l[2], l[1]]), vec![(l[0], l[1])], "a line that meets nothing of the set is half of the next corner and takes no corner away");
}

#[test]
fn two_cornors_far_from_each_other_are_two_corners() {
    let (p, si, l) = two_separate_corners();
    assert_eq!(p.corners_of_lines(si, &l), vec![(l[0], l[1]), (l[2], l[3])], "a set may hold more than one contour");
    assert_eq!(p.corners_of_lines(si, &l[..3]).len(), 1, "the first contour is made and the fourth line of the second is still wanted");
}

#[test]
fn a_chain_of_four_lines_is_three_corners() {
    let (p, si, l) = chain_of_four();
    assert_eq!(p.corners_of_lines(si, &l), vec![(l[0], l[1]), (l[1], l[2]), (l[2], l[3])]);
}

#[test]
fn four_lines_on_one_point_are_taken_two_at_a_time_as_they_were_named() {
    let (p, si, e, _) = cross_at_one_point();
    assert_eq!(p.corners_of_lines(si, &e[..2]), vec![(e[0], e[1])], "the first two make the first corner");
    assert_eq!(p.corners_of_lines(si, &e[..3]), vec![(e[0], e[1])], "the third line joins a corner that two lines have already made and makes none of its own");
    assert_eq!(p.corners_of_lines(si, &e), vec![(e[0], e[1]), (e[2], e[3])], "the fourth makes the second corner, and it makes it with the third");
}

#[test]
fn the_order_the_lines_were_named_in_decides_which_are_paired() {
    let (p, si, e, _) = cross_at_one_point();
    let back: Vec<u64> = e.iter().rev().copied().collect();
    assert_eq!(p.corners_of_lines(si, &back), vec![(back[0], back[1]), (back[2], back[3])], "the same four lines named the other way round pair up the other way");
    // and the set read off a map is not the set read off the hand: the same drawing must round the same corners twice
    assert_eq!(p.corners_of_lines(si, &e), p.corners_of_lines(si, &e), "the same naming twice gives the same corners");
}

#[test]
fn a_line_named_twice_is_one_line() {
    let (p, si, l) = triangle();
    let twice = vec![l[0], l[1], l[0], l[2]];
    assert_eq!(p.corners_of_lines(si, &twice), p.corners_of_lines(si, &[l[0], l[1], l[2]]), "a line standing twice at one point would be paired with itself");
}

#[test]
fn two_lines_along_one_straight_line_make_no_corner() {
    let (p, si, _) = lines_of(&[leg((0.0, 0.0), (20.0, 0.0)), leg((20.0, 0.0), (40.0, 0.0))]);
    assert!(
        p.corners_of_lines(si, &p.vertex_edges(si, p.sketches[si].points.iter().find(|q| (q.x - 20.0).abs() < 1e-6).unwrap().id)).is_empty(),
        "they share a point and no angle: there is nothing to round between them"
    );
}

#[test]
fn a_set_of_corners_says_where_each_of_them_is() {
    let (p, si, l) = triangle();
    for &(a, b) in &p.corners_of_lines(si, &l) {
        assert!(p.corner_of_pair(si, a, b).is_some(), "a pair the set named is a corner of the drawing");
    }
}
