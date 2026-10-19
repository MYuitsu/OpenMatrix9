//! COMPONENTS ACROSS PLACES AND NESTING: a second part holding a block put by the numbers of its placement at
//! places below zero, in fractions, far off; turned by a number about each axis; parts made inside a sub-assembly
//! inside a sub-assembly. What is read back is where each part stands, which way it looks, and what holds it.
use qymcad::{Key, Session};
use qymcad_acceptance::{build, probe};

fn take(s: &mut Session, hint: &str) {
    let hint = s.word(hint);
    s.press_hint(&hint);
}

fn up_to_the_assembly(s: &mut Session) {
    let assembly = s.word("wb-assembly");
    s.press_word_near(&assembly, qymcad::pos2(0.0, 0.0));
}

/// A first start with a second part holding a 40 x 30 x 10 block, the session in the assembly with it taken.
fn second_part_taken() -> (Session, String) {
    let mut s = Session::start();
    s.key(Key::Escape);
    take(&mut s, "tb-new-part-hint");
    let second = s.document().parts.last().cloned().expect("the second part").name;
    build::rectangle_on_xy(&mut s);
    let finish = s.word("wb-finish");
    s.press_word(&finish);
    let extrude = s.word("tb-extrude-hint");
    s.press_hint(&extrude);
    s.key(Key::Enter);
    up_to_the_assembly(&mut s);
    let row = s.find(&second, qymcad::pos2(0.0, 300.0)).unwrap_or_else(|| panic!("the part {second:?} is not in the tree"));
    s.click(row.center());
    (s, second)
}

fn part(s: &mut Session, name: &str) -> qymcad::Part {
    s.document().parts.into_iter().find(|p| p.name == name).unwrap_or_else(|| panic!("the assembly holds no part {name:?}"))
}

probe! {
    budget = 900;
    /// A PART PUT BY THE NUMBERS stands where they say: below zero, in fractions, a thousand off, back at the origin.
    fn parts_put_by_the_numbers() {
        let failed: Vec<String> = [[-25.0, 12.5, 1000.0], [0.001, -0.5, 0.0], [-1000.0, -1000.0, -1000.0], [0.0, 0.0, 0.0]]
            .into_iter()
            .filter_map(|at: [f64; 3]| {
                let (mut s, second) = second_part_taken();
                for (caption, v) in ["X", "Y", "Z"].into_iter().zip(at) {
                    s.fill(caption, &format!("{v}"));
                }
                s.key(Key::Enter);
                let now = part(&mut s, &second).at;
                (now.iter().zip(at).any(|(n, w)| (n - w).abs() > 1e-6)).then(|| format!("put at {at:?}, it stands at {now:?}"))
            })
            .collect();
        assert!(failed.is_empty(), "{} of 4 places went wrong:\n{}", failed.len(), failed.join("\n"));
    }
}

probe! {
    budget = 900;
    /// A PART TURNED BY A NUMBER about X, Y and Z: 90 typed and the button of an axis pressed - that axis stays where
    /// it was and the other two turn a right angle.
    fn parts_turned_by_a_number() {
        let failed: Vec<String> = [("X", 0usize), ("Y", 1), ("Z", 2)]
            .into_iter()
            .filter_map(|(axis, k)| {
                let (mut s, second) = second_part_taken();
                // the angle is typed first; the button of an axis beside it turns the part by that angle about it
                let caption = s.word("comp-rotation");
                s.fill(&caption, "90");
                let angle = s.field(&caption);
                let button = s
                    .widgets()
                    .into_iter()
                    .filter(|w| w.kind == qymcad::Kind::Button && w.label.ends_with(axis) && (w.rect.center().y - angle.rect.center().y).abs() < 8.0)
                    .min_by(|a, b| (a.rect.center() - angle.rect.center()).length().total_cmp(&(b.rect.center() - angle.rect.center()).length()))
                    .unwrap_or_else(|| panic!("no button {axis} beside the rotation"));
                s.click(button.rect.center());
                let axes = part(&mut s, &second).axes;
                let kept = (axes[k][k] - 1.0).abs() < 1e-6;
                let others_turned = (0..3).filter(|i| *i != k).all(|i| axes[i][i].abs() < 1e-6);
                (!(kept && others_turned)).then(|| format!("turned 90 about {axis}: the part looks {axes:?}; the program says {:?}", s.status()))
            })
            .collect();
        assert!(failed.is_empty(), "{} of 3 turns went wrong:\n{}", failed.len(), failed.join("\n"));
    }
}

probe! {
    /// PARTS INSIDE A SUB-ASSEMBLY INSIDE A SUB-ASSEMBLY: each stands under the one it was made in.
    fn parts_nested_two_deep() {
        let mut s = Session::start();
        s.key(Key::Escape);
        take(&mut s, "tb-new-subassembly-hint");
        let outer = s.document().parts.last().cloned().expect("the outer sub-assembly").name;
        take(&mut s, "tb-new-subassembly-hint");
        let inner = s.document().parts.last().cloned().expect("the inner sub-assembly");
        take(&mut s, "tb-new-part-hint");
        let deep = s.document().parts.last().cloned().expect("the part inside");
        assert!(inner.assembly && inner.parent.as_deref() == Some(outer.as_str()), "the inner sub-assembly {:?} stands under {:?}, not under {outer:?}", inner.name, inner.parent);
        assert!(deep.parent.as_deref() == Some(inner.name.as_str()), "the part {:?} made inside {:?} stands under {:?}", deep.name, inner.name, deep.parent);
        assert!(s.document().context == deep.name, "the person was not put inside the new part: the path says {:?}", s.document().context);
    }
}

probe! {
    budget = 900;
    /// A MIRRORED COPY OF A PART ABOUT EACH SIDE OF ITS BLOCK: the side at x = 40, the front at y = 0, the top at z = 10,
    /// each picked as the plane; the copy stands beyond that side, and the two blocks together run twice as far that way.
    fn mirrored_copies_about_each_side() {
        let mut problems = Vec::new();
        for (plane, axis, from, to) in [([40.0, 15.0, 5.0], 0usize, 0.0, 80.0), ([20.0, 0.0, 5.0], 1, -30.0, 30.0), ([20.0, 15.0, 10.0], 2, 0.0, 20.0)] {
            let mut s = qymcad_acceptance::contract::fixtures::Fixture::BlockInAssemblyPicked.start();
            take(&mut s, "tb-mirror-part-hint");
            let at = s.face_at(plane);
            s.click(at);
            s.key(Key::Enter);
            let doc = s.document();
            let bodies: Vec<&qymcad::Solid> = doc.bodies.iter().filter(|b| b.visible && !b.consumed).collect();
            // a body's box is in its part's own frame: its corners are carried to the assembly by where the part stands
            let (mut lo, mut hi) = (f64::MAX, f64::MIN);
            for b in &bodies {
                let Some(p) = doc.parts.iter().find(|p| Some(p.key) == b.part_key) else { continue };
                for k in 0..8 {
                    let c = [if k & 1 == 0 { b.min[0] } else { b.max[0] }, if k & 2 == 0 { b.min[1] } else { b.max[1] }, if k & 4 == 0 { b.min[2] } else { b.max[2] }];
                    let w = p.at[axis] + (0..3).map(|i| p.axes[i][axis] * c[i]).sum::<f64>();
                    (lo, hi) = (lo.min(w), hi.max(w));
                }
            }
            if bodies.len() != 2 || (lo - from).abs() > 1e-3 || (hi - to).abs() > 1e-3 {
                problems.push(format!("about the side at {plane:?}: {} bodies running {lo}..{hi} along axis {axis}, not two running {from}..{to}", bodies.len()));
            }
        }
        assert!(problems.is_empty(), "{}", problems.join("\n"));
    }
}
