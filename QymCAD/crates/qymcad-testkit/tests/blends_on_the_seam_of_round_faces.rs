//! BLENDS ON THE SEAM OF A ROUND FACE, across the round surfaces and the blending tools. A quarter cut out of a round
//! solid next to its seam puts an edge of the cut exactly on the seam, the seam running on from its end; every tool that
//! rolls or cuts along an edge has to take that edge as it takes any other. Failures are gathered and told at once.
//!
//! Reported behaviour: of eight edges of a symmetric part one would not take any radius, the seam of the round corner
//! beside it running up from its end.
use qymcad_core::feature::{ChamferMode, Purpose, Reach};
use qymcad_core::model::{ChamferShape, CombineSpan, Project};

/// A round solid with the quarter from its seam line (x > 0, y = 0) clockwise cut out of it, and the edge the cut leaves
/// on the seam: (project, the cut body, the edge).
fn cut_at_the_seam(solid: &str, at_seam: bool) -> (Project, u64, u32) {
    let mut p = Project::default();
    p.new_document();
    let (body, reach, h) = match solid {
        "cylinder" => (p.add_cylinder(5.0, 6.0), Reach::Forward, 4.0),
        "cone" => (p.add_cone(5.0, 3.0, 6.0), Reach::Forward, 4.0),
        "sphere" => (p.add_sphere(5.0), Reach::Forward, 10.0),
        "torus" => (p.add_torus(10.0, 3.0), Reach::BothWays, 20.0),
        _ => unreachable!(),
    };
    let si = p.new_sketch("Tool");
    let sid = p.sketches[si].id;
    p.add_sketch_node(sid, "Tool");
    // at the seam: the quarter x > 0, y < 0, its face y = 0 on the seam line; the control: the same quarter turned half a
    // turn, x < 0, y > 0, the seam far from it
    if at_seam {
        p.add_rect_entity(si, 0.0, -20.0, 20.0, 0.0, Purpose::Real);
    } else {
        p.add_rect_entity(si, -20.0, 0.0, 0.0, 20.0, Purpose::Real);
    }
    p.regen_sketch(si);
    let c: Vec<u64> = p.sketches[si].contour_ids.iter().copied().filter(|c| p.contour_profile_xy(*c).is_some()).collect();
    let cut = p.add_combine_multi_op(body, sid, c, CombineSpan { height: h, down: 0.0, extent: qymcad_core::feature::Extent { reach, ..Default::default() }, fill: &[] }, 0);
    let (report, _) = qymcad_testkit::regenerate(&mut p);
    assert!(report.errors.is_empty(), "{solid}: setup, the cut did not build: {:?}", report.errors);
    // the edge where the face of the cut in the plane y = 0 meets the round surface: the one farthest out
    let sign = if at_seam { 1.0 } else { -1.0 };
    let edge = p
        .regen_edges
        .get(&cut)
        .and_then(|es| es.iter().filter(|e| e.mid[1].abs() < 1e-3 && sign * e.mid[0] > 0.1).max_by(|a, b| (sign * a.mid[0]).total_cmp(&(sign * b.mid[0]))).map(|e| e.id))
        .unwrap_or_else(|| panic!("{solid}: no edge of the cut on the line y = 0"));
    (p, cut, edge)
}

/// What the tool made of the edge: `None` when it was taken, or the reason it was not.
fn blend(solid: &str, tool: &str, at_seam: bool) -> Option<String> {
    let (mut p, cut, edge) = cut_at_the_seam(solid, at_seam);
    let before = qymcad_testkit::regenerate(&mut p).1.get(&cut).map(|s| s.volume()).unwrap_or(0.0);
    let sign = if at_seam { 1.0 } else { -1.0 };
    let node = match tool {
        "fillet" => p.add_fillet(cut, 0.5, vec![edge]),
        "chamfer" => p.add_chamfer(cut, 0.5, vec![edge]),
        "fillet by vertices" => {
            let end = p.vertex_pool(cut).into_iter().find(|v| v.centroid[1].abs() < 1e-3 && sign * v.centroid[0] > 0.1).map(|v| v.desc).expect("an end of the edge");
            p.add_fillet_at_vertices(cut, 0.4, qymcad_core::refs::Ref::picks(&[edge]), vec![(end, 0.6)])
        }
        "chamfer of two setbacks" => p.add_chamfer_ex(cut, 0.5, ChamferShape { mode: ChamferMode::TwoDist, d2: 0.3, flip: false, ref_face: 0 }, vec![edge]),
        "fillet of every edge" => p.add_fillet(cut, 0.3, vec![]),
        "chamfer of every edge" => p.add_chamfer(cut, 0.3, vec![]),
        _ => unreachable!(),
    };
    let (report, shapes) = qymcad_testkit::regenerate(&mut p);
    if let Some((_, e)) = report.errors.iter().find(|(id, _)| *id == node) {
        return Some(format!("refused - {e:?}"));
    }
    if let Some(w) = p.regen_warnings.get(&node) {
        return Some(format!("left an edge out - {w:?}"));
    }
    let after = shapes.get(&node).map(|s| s.volume()).unwrap_or(before);
    ((after - before).abs() < 1e-3).then(|| format!("changed nothing ({before:.3} -> {after:.3})"))
}

/// EVERY BLEND TAKES THE EDGE ON THE SEAM WHEN IT TAKES THE SAME EDGE AWAY FROM IT: the seam is where the face was made,
/// not a property of the shape, so a tool that takes the edge of the quarter turned half a turn - the seam far off - must
/// take it at the seam too. What neither takes is the geometry's, told apart.
#[test]
fn every_blend_takes_an_edge_on_the_seam_of_every_round_surface() {
    let mut fails: Vec<String> = Vec::new();
    for solid in ["cylinder", "cone", "sphere", "torus"] {
        for tool in ["fillet", "chamfer", "fillet by vertices", "chamfer of two setbacks", "fillet of every edge", "chamfer of every edge"] {
            let (seam, away) = (blend(solid, tool, true), blend(solid, tool, false));
            match (seam, away) {
                (Some(why), None) => fails.push(format!("{solid} / {tool}: taken away from the seam, not on it - {why}")),
                (Some(why), Some(also)) => eprintln!("NEITHER {solid} / {tool}: on the seam {why}; away from it {also}"),
                _ => {}
            }
        }
    }
    assert!(fails.is_empty(), "\nBLENDS ON THE SEAM ({}):\n{}", fails.len(), fails.join("\n"));
}
