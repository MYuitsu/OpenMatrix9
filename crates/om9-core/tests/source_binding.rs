// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    budget::Limits,
    geometry::{Cloud, Geometry, GeometryKind},
    model::{Document, Entity},
    request::*,
    transaction::World,
};
fn fixture() -> (World, SourceBinding) {
    let mut d = Document::new("bindings");
    let e = Entity::new(
        "cloud",
        Geometry::Cloud(Cloud {
            points: vec![[0., 0., 0.], [1., 2., 3.], [4., 5., 6.]],
            normals: vec![],
            colors: vec![],
            hidden: vec![false, true, false],
        }),
        d.current_layer,
    );
    let id = e.id;
    d.entities.insert(id, e);
    let stamp = d.stamp(id).unwrap();
    (
        World::new(vec![d], &Limits::default()).unwrap(),
        SourceBinding {
            role: "FitPoints".into(),
            source: stamp,
            selection: Selection::Whole,
        },
    )
}
fn schema() -> RequestSchema {
    RequestSchema {
        feature: "test-fit".into(),
        roles: vec![Role {
            name: "FitPoints".into(),
            kinds: vec![GeometryKind::Cloud],
            min: 1,
            max: 1,
            access: Access::Reference,
            allow_reuse: false,
        }],
        options: Default::default(),
    }
}
#[test]
fn ordered_subset_preserves_original_native_indices_including_hidden_points() {
    let (world, mut binding) = fixture();
    binding.selection = Selection::Indices(vec![2, 1]);
    let admitted = admit(
        &world,
        &schema(),
        vec![binding],
        Default::default(),
        &Limits::default(),
    )
    .unwrap();
    assert_eq!(
        admitted.inputs[0].selected_points().unwrap(),
        vec![(2, [4., 5., 6.]), (1, [1., 2., 3.])]
    );
}
#[test]
fn whole_fit_points_is_not_replaced_by_a_prefix_or_subset() {
    let (world, binding) = fixture();
    let admitted = admit(
        &world,
        &schema(),
        vec![binding],
        Default::default(),
        &Limits::default(),
    )
    .unwrap();
    assert_eq!(
        admitted.inputs[0].selected_points().unwrap(),
        vec![(0, [0., 0., 0.]), (1, [1., 2., 3.]), (2, [4., 5., 6.])]
    );
}
#[test]
fn duplicate_out_of_range_and_overlapping_subsets_are_rejected() {
    let (world, binding) = fixture();
    for selection in [
        Selection::Indices(vec![1, 1]),
        Selection::Indices(vec![3]),
        Selection::Spans(vec![
            IndexSpan { start: 0, count: 2 },
            IndexSpan { start: 1, count: 2 },
        ]),
        Selection::Spans(vec![IndexSpan {
            start: usize::MAX,
            count: 2,
        }]),
    ] {
        let mut b = binding.clone();
        b.selection = selection;
        assert!(
            admit(
                &world,
                &schema(),
                vec![b],
                Default::default(),
                &Limits::default()
            )
            .is_err()
        );
    }
}
#[test]
fn unknown_role_option_wrong_kind_and_stale_generation_reject() {
    let (world, binding) = fixture();
    let mut b = binding.clone();
    b.role = "extra".into();
    assert!(
        admit(
            &world,
            &schema(),
            vec![b],
            Default::default(),
            &Limits::default()
        )
        .is_err()
    );
    let mut b = binding.clone();
    b.source.generation += 1;
    assert!(
        admit(
            &world,
            &schema(),
            vec![b],
            Default::default(),
            &Limits::default()
        )
        .is_err()
    );
    let mut s = schema();
    s.roles[0].kinds = vec![GeometryKind::Curve];
    assert!(
        admit(
            &world,
            &s,
            vec![binding.clone()],
            Default::default(),
            &Limits::default()
        )
        .is_err()
    );
    assert!(
        admit(
            &world,
            &schema(),
            vec![binding],
            std::collections::BTreeMap::from([("unsupported".into(), Value::Number(2.))]),
            &Limits::default()
        )
        .is_err()
    );
}
#[test]
fn numeric_option_domain_and_dimension_are_explicit() {
    let (world, binding) = fixture();
    let mut s = schema();
    s.options.insert(
        "Radius".into(),
        OptionDomain::Number {
            dimension: Dimension::Length,
            min: Some(0.),
            min_inclusive: false,
            max: Some(100.),
            step: None,
        },
    );
    for value in [
        Value::Number(f64::NAN),
        Value::Number(0.),
        Value::Quantity {
            value: 10.,
            dimension: Dimension::Angle,
        },
    ] {
        assert!(
            admit(
                &world,
                &s,
                vec![binding.clone()],
                std::collections::BTreeMap::from([("Radius".into(), value)]),
                &Limits::default()
            )
            .is_err()
        );
    }
    assert!(
        admit(
            &world,
            &s,
            vec![binding],
            std::collections::BTreeMap::from([(
                "Radius".into(),
                Value::Quantity {
                    value: 10.,
                    dimension: Dimension::Length
                }
            )]),
            &Limits::default()
        )
        .is_ok()
    );
}
#[test]
fn capture_owns_immutable_provenance_and_revalidation_detects_context_change() {
    let (world, binding) = fixture();
    let admitted = admit(
        &world,
        &schema(),
        vec![binding],
        Default::default(),
        &Limits::default(),
    )
    .unwrap();
    let mut changed = world.clone();
    let d = std::sync::Arc::make_mut(changed.documents.values_mut().next().unwrap());
    d.context_revision += 1;
    assert!(admitted.revalidate(&changed).is_err());
    assert_eq!(admitted.inputs[0].selected_points().unwrap().len(), 3);
}

#[test]
fn selected_point_peak_copy_is_part_of_aggregate_admission() {
    let (world, binding) = fixture();
    let limits = Limits {
        max_points: 3,
        ..Limits::default()
    };
    assert!(world.validate(&limits).is_ok(), "master itself fits");
    assert!(
        admit(
            &world,
            &schema(),
            vec![binding],
            Default::default(),
            &limits
        )
        .is_err(),
        "master plus prepared selected-point copy must fit together"
    );
}

#[test]
fn owned_option_bytes_share_budget_with_source_snapshot() {
    let (world, binding) = fixture();
    let mut s = schema();
    s.options
        .insert("Text".into(), OptionDomain::Text { max_bytes: 1000 });
    let base = world.validate(&Limits::default()).unwrap().bytes;
    let limits = Limits {
        max_bytes: base + std::mem::size_of::<BoundInput>() as u64 + 200,
        ..Limits::default()
    };
    let options = std::collections::BTreeMap::from([("Text".into(), Value::Text("x".repeat(300)))]);
    assert!(admit(&world, &s, vec![binding], options, &limits).is_err());
}
