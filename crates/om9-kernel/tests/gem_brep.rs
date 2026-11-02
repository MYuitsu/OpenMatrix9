// SPDX-License-Identifier: LGPL-2.1-or-later
//! The gem body must be convertible to one editable OCCT solid without changing its boundary.
use om9_core::budget::Limits;
use om9_core::gem::{CustomGemParams, GemCut, GemShape, GemSpec, build_mesh};
use om9_kernel::brep::source_volume;
use om9_kernel::gem::gem_brep;

fn specs() -> Vec<(&'static str, GemSpec)> {
    vec![
        ("round diamond", GemSpec::round(6.5, 3.95).expect("round")),
        ("oval diamond", GemSpec::oval(8.0, 6.0, 4.2).expect("oval")),
        (
            "emerald",
            GemSpec::emerald(9.0, 7.0, 1.5, 4.5, 0.2).expect("emerald"),
        ),
        (
            "baguette",
            GemSpec::baguette(7.0, 3.5, 0.4, 2.8, 0.15).expect("baguette"),
        ),
        (
            "heart",
            GemSpec::from_outer_dimensions(GemShape::Heart, GemCut::Diamond, 7.0, 7.0, 3.5)
                .expect("heart"),
        ),
        (
            "trillion",
            GemSpec::from_outer_dimensions(GemShape::Trillion, GemCut::Diamond, 7.0, 7.0, 3.0)
                .expect("trillion"),
        ),
        (
            "cabochon",
            GemSpec::from_outer_dimensions(GemShape::Oval, GemCut::Cabochon, 10.0, 8.0, 5.0)
                .expect("cabochon"),
        ),
        (
            "custom profile",
            GemSpec::from_profile(
                &[
                    [3.0, 0.0],
                    [2.12, 2.12],
                    [0.0, 3.0],
                    [-2.12, 2.12],
                    [-3.0, 0.0],
                    [-2.12, -2.12],
                    [0.0, -3.0],
                    [2.12, -2.12],
                ],
                GemCut::Diamond,
                &CustomGemParams::default(),
            )
            .expect("profile"),
        ),
    ]
}

#[test]
fn every_gem_cut_becomes_one_closed_editable_solid() {
    let limits = Limits::default();
    for (name, spec) in specs() {
        let gem = build_mesh(&spec).expect(name);
        assert!(gem.closed, "{name}: source mesh is closed");
        let brep = gem_brep(&gem, &limits).expect(name);
        brep.validate(&limits).expect("admitted B-Rep");
        let volume = source_volume(&brep, &limits).expect("source volume");
        let relative = (volume - gem.volume_mm3).abs() / gem.volume_mm3;
        assert!(
            relative < 1.0e-6,
            "{name}: B-Rep volume {volume} differs from mesh {} by {relative}",
            gem.volume_mm3
        );
        assert!(
            !brep.display.faces.is_empty(),
            "{name}: the B-Rep keeps the authored facets as faces"
        );
        assert!(
            brep.source().contains("CASCADE Topology"),
            "{name}: canonical OCCT source"
        );
    }
}

#[test]
fn an_open_mesh_is_refused_instead_of_being_sealed() {
    let limits = Limits::default();
    let spec = GemSpec::round(6.5, 3.95).expect("round");
    let mut gem = build_mesh(&spec).expect("mesh");
    gem.mesh.faces.pop();
    let error = gem_brep(&gem, &limits).expect_err("an open gem is not a solid");
    let text = error.to_string();
    assert!(
        text.contains("not closed") || text.contains("open") || text.contains("shell"),
        "unexpected refusal: {text}"
    );
    assert!(
        gem_brep(&gem, &limits).is_err(),
        "refusal is stable, never a repaired fake solid"
    );
}
