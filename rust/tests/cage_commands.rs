// SPDX-License-Identifier: LGPL-2.1-or-later
#[path = "../src/cage_commands.rs"]
mod cage_commands;
use cage_commands::*;

fn input(name: &str) -> Input {
    Input::new(name, "full-topology-and-global-frame", [0.; 3], [10.; 3]).unwrap()
}
#[test]
fn stable_catalog_and_exact_aliases() {
    assert_eq!(ICONS.len(), 3);
    assert_eq!(
        command(ICONS[0]),
        Some("OM9_TransformCageEditingCreateCage")
    );
    assert_eq!(caption(ICONS[1]), Some("Cage Edit"));
    assert_eq!(kind("OM9-TRANSFORM-041"), Kind::Create);
    assert_eq!(kind("OM9_TransformCageEditingCreateCage"), Kind::Create);
    assert_eq!(kind("Cage"), Kind::Create);
    assert_eq!(kind("CageEdit"), Kind::Capture);
    assert_eq!(kind("OM9-TRANSFORM-020"), Kind::Capture);
    assert_eq!(kind("ReleaseFromCage"), Kind::Release);
    assert_eq!(kind("OM9-TRANSFORM-022"), Kind::Release);
    assert_eq!(kind("CageAccurate"), Kind::None);
}
#[test]
fn creation_has_explicit_defaults_and_never_captures_sources() {
    let mut s = Session::new(Kind::Create).unwrap();
    assert!(s.advance().is_err());
    s.add(input("Box")).unwrap();
    s.advance().unwrap();
    assert_eq!(s.phase, Phase::Options);
    assert_eq!(s.counts, [2; 3]);
    assert_eq!(s.degrees, [1; 3]);
    assert!(s.add(input("Late")).is_err());
    s.advance().unwrap();
    assert_eq!(s.phase, Phase::Ready);
    assert_eq!(s.inputs.len(), 1);
}
#[test]
fn capture_requires_captives_then_an_existing_distinct_control() {
    let mut s = Session::new(Kind::Capture).unwrap();
    s.add(input("Mesh")).unwrap();
    s.advance().unwrap();
    assert_eq!(s.phase, Phase::Control);
    assert!(s.advance().is_err());
    assert!(s.set_control(input("Mesh"), false).is_err());
    s.set_control(input("Cage"), false).unwrap();
    assert_eq!(s.phase, Phase::Options);
    s.option("Region", "Local").unwrap();
    s.option("Falloff", "2.5").unwrap();
    assert!(s.option("Falloff", "NaN").is_err());
    assert_eq!(s.falloff, 2.5);
    s.advance().unwrap();
    assert_eq!(s.phase, Phase::Ready);
}
#[test]
fn invalid_options_and_flat_bbox_are_rejected_without_padding() {
    let mut s = Session::new(Kind::Create).unwrap();
    s.add(Input::new("Flat", "x", [0.; 3], [10., 10., 0.]).unwrap())
        .unwrap();
    assert!(s.advance().is_err());
    let mut s = Session::new(Kind::Create).unwrap();
    s.add(input("Box")).unwrap();
    s.advance().unwrap();
    for (key, value) in [
        ("UCount", "1"),
        ("UCount", "129"),
        ("UDegree", "2"),
        ("VCount", "NaN"),
        ("Falloff", "-1"),
        ("Coordinates", "CPlane"),
        ("Shape", "Line"),
        ("Accurate", "Yes"),
    ] {
        assert!(s.option(key, value).is_err(), "{key}");
    }
    assert_eq!(s.counts, [2; 3]);
    s.option("UCount", "4").unwrap();
    s.option("UDegree", "3").unwrap();
    assert!(s.option("Parameters", "2,2,2,NaN,1,1").is_err());
    assert_eq!(s.counts, [4, 2, 2]);
    s.option("Parameters", "2,2,2,1,1,1").unwrap();
    assert_eq!(s.counts, [2; 3]);
    assert_eq!(s.degrees, [1; 3]);
}
#[test]
fn release_preserves_selected_order_and_freezes_owned_snapshots() {
    let mut s = Session::new(Kind::Release).unwrap();
    s.add(input("B")).unwrap();
    s.add(input("A")).unwrap();
    s.add(input("B")).unwrap();
    s.advance().unwrap();
    assert_eq!(s.phase, Phase::Ready);
    assert_eq!(
        s.inputs
            .iter()
            .map(|i| i.name.to_str().unwrap())
            .collect::<Vec<_>>(),
        ["B", "A"]
    );
    assert!(s.verify("B", "changed-geometry").is_err());
    assert!(s.verify("B", "full-topology-and-global-frame").is_ok());
    assert!(!available(false, true, true, true));
    assert!(!available(true, false, true, true));
    assert!(available(true, true, true, true));
}
