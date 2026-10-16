//! THE CONSTRAINTS OF A SKETCH ON FIGURES AS THE TOOLS DRAW THEM - the auto constraints on, as a person has them: a
//! line drawn level already holds its level, a circle comes with its diameter, a rectangle with its sides level and
//! upright. On such figures a constraint must either do what it says - the geometry shows it - or, where the figure
//! already holds it, be refused in words. What it must never do is leave the sketch over-defined without a word.
use qymcad::{Session, SketchPick};
use qymcad_acceptance::build::{circle, draw, into_the_first_part, line, pick, point};
use qymcad_acceptance::probe;

/// What must come of pressing the constraint.
#[derive(Clone, Copy)]
enum Want {
    /// Added, and the geometry shows it: `holds` answers what is wrong, if anything.
    Added(fn(&mut Session) -> Option<String>),
    /// The figure holds it already: refused in words, or taken without leaving anything over-defined.
    Held,
}

struct Tie {
    figure: fn(&mut Session),
    picks: &'static [(f64, f64)],
    hint: &'static str,
    want: Want,
    what: &'static str,
}

/// A sketch on XY of the first part with the auto constraints as they come, drawn by `draw`.
fn sketched(s: &mut Session, draw: fn(&mut Session)) {
    into_the_first_part(s);
    let xy = s.word("plane-xy-table");
    s.press_word(&xy);
    draw(s);
}

fn run_all(all: Vec<Tie>) {
    let mut failed = Vec::new();
    for c in &all {
        let problem = qymcad_acceptance::refusal(|| {
            let mut s = Session::start();
            sketched(&mut s, c.figure);
            let before = s.document().sketches[0].clone();
            assert!(before.redundant == 0, "the figure is over-defined before the constraint: {before:?}");
            for (n, (x, y)) in c.picks.iter().enumerate() {
                pick(&mut s, *x, *y, n > 0);
            }
            let hint = s.word(c.hint);
            s.press_hint(&hint);
            let said = s.status();
            let after = s.document().sketches[0].clone();
            // what agrees with what is there already may stay as a reference, when the sketch says so in words
            assert!(after.redundant == 0 || qymcad_acceptance::says_redundant(&mut s), "the sketch is left over-defined ({} redundant) without a word; the status line says {said:?}", after.redundant);
            // added, or added with the relations it made redundant lifted - the count is the one part that differs
            let implied = s.word("sk-constraint-added-implied");
            let lifted = implied.split(':').next().unwrap_or(&implied).to_string();
            let added = said == s.word("sk-constraint-added") || said.starts_with(&lifted);
            match c.want {
                Want::Added(holds) => {
                    assert!(added, "the constraint was not added: the status line says {said:?}");
                    if let Some(p) = holds(&mut s) {
                        panic!("added, and the geometry does not show it: {p}");
                    }
                }
                Want::Held => {
                    if !added {
                        assert!(after.constraints == before.constraints, "refused, and the constraints changed: {} became {}", before.constraints, after.constraints);
                        assert!(said != hint && !said.is_empty(), "refused without a word: the status line says {said:?}");
                    }
                }
            }
        });
        if !problem.is_empty() {
            failed.push(format!("{}: {problem}", c.what));
        }
    }
    assert!(failed.is_empty(), "{} of {} cases did not hold:\n{}", failed.len(), all.len(), failed.join("\n"));
}

fn radius_at(s: &mut Session, x: f64, y: f64) -> Option<f64> {
    match s.sketch_under(x, y) {
        Some(SketchPick::Circle { radius, .. }) => Some(radius),
        _ => None,
    }
}

fn centre_at(s: &mut Session, x: f64, y: f64) -> Option<(f64, f64)> {
    match s.sketch_under(x, y) {
        Some(SketchPick::Circle { centre, .. }) => Some(centre),
        _ => None,
    }
}

fn line_at(s: &mut Session, x: f64, y: f64) -> Option<((f64, f64), (f64, f64))> {
    match s.sketch_under(x, y) {
        Some(SketchPick::Line { from, to }) => Some((from, to)),
        _ => None,
    }
}

/// Two circles as the tool draws them: radius 10 about the origin, radius 5 about (30, 0).
fn two_circles(s: &mut Session) {
    circle(s, (0.0, 0.0), (10.0, 0.0));
    circle(s, (30.0, 0.0), (35.0, 0.0));
}

/// A level line from (0, 0) to (20, 0) and another from (0, 10) to (30, 10), both drawn level.
fn two_level_lines(s: &mut Session) {
    line(s, (0.0, 0.0), (20.0, 0.0));
    line(s, (0.0, 10.0), (30.0, 10.0));
}

/// A level line from (0, 0) to (20, 0) and an upright one from (30, 0) to (30, 20).
fn level_and_upright(s: &mut Session) {
    line(s, (0.0, 0.0), (20.0, 0.0));
    line(s, (30.0, 0.0), (30.0, 20.0));
}

/// A rectangle 40 by 20 from the origin and a circle of radius 5 about (60, 10).
fn rectangle_and_circle(s: &mut Session) {
    draw(s, "tb-rect-hint", &[(0.0, 0.0), (40.0, 20.0)]);
    circle(s, (60.0, 10.0), (65.0, 10.0));
}

/// Two rectangles: 40 by 20 from the origin, 10 by 10 from (50, 0).
fn two_rectangles(s: &mut Session) {
    draw(s, "tb-rect-hint", &[(0.0, 0.0), (40.0, 20.0)]);
    draw(s, "tb-rect-hint", &[(50.0, 0.0), (60.0, 10.0)]);
}

/// A slanted line from (0, 0) to (20, 12) and a circle of radius 5 about (40, 0).
fn slant_and_circle(s: &mut Session) {
    line(s, (0.0, 0.0), (20.0, 12.0));
    circle(s, (40.0, 0.0), (45.0, 0.0));
}

/// A point at (13, 7) and the rectangle 40 by 20 from the origin.
fn point_and_rectangle(s: &mut Session) {
    draw(s, "tb-rect-hint", &[(0.0, 0.0), (40.0, 20.0)]);
    point(s, (13.0, 7.0));
}

fn tie(figure: fn(&mut Session), picks: &'static [(f64, f64)], hint: &'static str, want: Want, what: &'static str) -> Tie {
    Tie { figure, picks, hint, want, what }
}

probe! {
    budget = 1800;
    /// CONSTRAINTS ON CIRCLES THE TOOL DREW, each with its diameter: equal and concentric on two of them, tangent of a
    /// slanted line to one.
    fn constraints_on_circles_as_drawn() {
        run_all(vec![
            tie(two_circles, &[(0.0, 10.0), (30.0, 5.0)], "con-equal", Want::Added(|s| {
                let (a, b) = (radius_at(s, 0.0, 10.0)?, radius_at(s, 30.0, 5.0)?);
                ((a - b).abs() > 1e-6).then(|| format!("radii {a} and {b}"))
            }), "equal on two circles with their diameters"),
            tie(two_circles, &[(0.0, 10.0), (30.0, 5.0)], "con-concentric-hint", Want::Added(|s| {
                let a = centre_at(s, 0.0, 10.0).or_else(|| centre_at(s, 10.0, 0.0))?;
                let b = centre_at(s, a.0, a.1 + 5.0).or_else(|| centre_at(s, a.0 + 5.0, a.1))?;
                ((a.0 - b.0).hypot(a.1 - b.1) > 1e-6).then(|| format!("centres {a:?} and {b:?}"))
            }), "concentric on two circles with their diameters"),
            tie(slant_and_circle, &[(10.0, 6.0), (40.0, 5.0)], "con-tangent-hint", Want::Added(|_| None), "tangent of a slanted line to a circle with its diameter"),
        ]);
    }
}

probe! {
    budget = 1800;
    /// CONSTRAINTS THE FIGURE HOLDS ALREADY: level on a line drawn level, parallel on two level lines, perpendicular
    /// on a level and an upright one, parallel on the bottom and the top of a rectangle - refused in words, or taken
    /// without anything left over-defined.
    fn constraints_the_figure_holds_already() {
        run_all(vec![
            tie(two_level_lines, &[(10.0, 0.0)], "con-horizontal-hint", Want::Held, "level on a line drawn level"),
            tie(two_level_lines, &[(10.0, 0.0), (15.0, 10.0)], "con-parallel-hint", Want::Held, "parallel on two lines drawn level"),
            tie(level_and_upright, &[(10.0, 0.0), (30.0, 10.0)], "con-perpendicular-hint", Want::Held, "perpendicular on a level line and an upright one"),
            tie(two_rectangles, &[(20.0, 0.0), (20.0, 20.0)], "con-parallel-hint", Want::Held, "parallel on the bottom and top of a rectangle"),
        ]);
    }
}

probe! {
    budget = 1800;
    /// CONSTRAINTS BETWEEN FIGURES THE TOOLS DREW: equal on sides of two rectangles, collinear on two level lines,
    /// coincident from a corner of a rectangle to the end of a line, midpoint of a point on a side of a rectangle,
    /// tangent of a side of a rectangle to a circle.
    fn constraints_between_figures_as_drawn() {
        run_all(vec![
            tie(two_rectangles, &[(20.0, 0.0), (55.0, 0.0)], "con-equal", Want::Added(|s| {
                let (a, b) = (line_at(s, 20.0, 0.0).or_else(|| line_at(s, 25.0, 0.0))?, line_at(s, 55.0, 0.0).or_else(|| line_at(s, 60.0, 0.0))?);
                let len = |l: ((f64, f64), (f64, f64))| (l.1 .0 - l.0 .0).hypot(l.1 .1 - l.0 .1);
                ((len(a) - len(b)).abs() > 1e-6).then(|| format!("sides of {} and {}", len(a), len(b)))
            }), "equal on the bottoms of two rectangles"),
            tie(two_level_lines, &[(10.0, 0.0), (15.0, 10.0)], "con-collinear-hint", Want::Added(|s| {
                let a = line_at(s, 10.0, 0.0).or_else(|| line_at(s, 10.0, 5.0))?;
                let b = line_at(s, 15.0, a.0 .1)?;
                ((a.0 .1 - b.0 .1).abs() > 1e-6).then(|| format!("lines at {} and {}", a.0 .1, b.0 .1))
            }), "collinear on two lines drawn level"),
            // the rectangle carries no dimensions, so the solver may move it rather than the point: what must hold is
            // the point in the middle of the bottom wherever the bottom now is - the two lowest corners
            tie(point_and_rectangle, &[(13.0, 7.0), (20.0, 0.0)], "con-midpoint-hint", Want::Added(|s| {
                // the four corners of the rectangle come first, its centre after them, the point drawn last
                let at = s.document().sketches[0].places.clone();
                let (corners, point) = (&at[..4], at[at.len() - 1]);
                let mut low: Vec<[f64; 2]> = corners.to_vec();
                low.sort_by(|a, b| a[1].total_cmp(&b[1]));
                let middle = [(low[0][0] + low[1][0]) / 2.0, (low[0][1] + low[1][1]) / 2.0];
                ((point[0] - middle[0]).hypot(point[1] - middle[1]) > 1e-6).then(|| format!("the point stands at {point:?}, the middle of the bottom at {middle:?}"))
            }), "midpoint of a point on the bottom of a rectangle"),
            tie(rectangle_and_circle, &[(40.0, 10.0), (60.0, 5.0)], "con-tangent-hint", Want::Added(|_| None), "tangent of the side of a rectangle to a circle with its diameter"),
        ]);
    }
}
