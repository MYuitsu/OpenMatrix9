// SPDX-License-Identifier: LGPL-2.1-or-later
//! Menu-binding acceptance for `OM9-L2-03-16`: one stable command ID per documented
//! Curve Tools command, with an explicit disabled reason when no interactive session exists.
use om9_app::{
    curve_menu::{self, BindingState},
    icons, menu_catalog,
    session::Controller,
};
use om9_core::{budget::Limits, model::Document};

#[test]
fn every_documented_curve_menu_row_has_one_stable_id_and_icon() {
    assert_eq!(curve_menu::BINDINGS.len(), 58);
    assert_eq!(menu_catalog::CURVE.len(), 58);
    let mut l3 = std::collections::BTreeSet::new();
    let mut features = std::collections::BTreeSet::new();
    let mut icons_seen = std::collections::BTreeSet::new();
    for (index, binding) in curve_menu::BINDINGS.iter().enumerate() {
        assert!(l3.insert(binding.l3_id), "duplicate L3 {}", binding.l3_id);
        assert!(
            features.insert(binding.feature),
            "duplicate feature {}",
            binding.feature
        );
        assert!(
            icons_seen.insert(binding.icon),
            "duplicate icon {}",
            binding.icon
        );
        assert_eq!(
            binding.l3_id,
            format!("OM9-L3-03-16-{:03}", index + 1),
            "L3 order"
        );
        assert!(
            menu_catalog::CURVE.contains(&binding.icon),
            "{} is not in the authored Curve palette",
            binding.icon
        );
        assert_eq!(
            curve_menu::by_l3(binding.l3_id).map(|row| row.feature),
            Some(binding.feature)
        );
        assert_eq!(
            curve_menu::by_icon(binding.icon).map(|row| row.feature),
            Some(binding.feature)
        );
        assert_eq!(
            curve_menu::by_feature(binding.feature).map(|row| row.l3_id),
            Some(binding.l3_id)
        );
        assert!(!binding.label.is_empty());
        assert!(!binding.source_command.is_empty());
        if binding.state != BindingState::OwnerMissing {
            assert!(!binding.owner.is_empty());
        }
    }
    assert_eq!(l3.len(), 58);
    assert_eq!(features.len(), 58);
}

#[test]
fn implemented_rows_dispatch_to_their_session() {
    for binding in curve_menu::BINDINGS
        .iter()
        .filter(|binding| binding.enabled())
    {
        let feature = binding.feature;
        let binding = curve_menu::by_feature(feature).expect("binding");
        assert!(binding.enabled(), "{feature}");
        assert_eq!(binding.state, BindingState::Implemented);
        assert_eq!(binding.disabled_reason, "");
        assert!(Controller::availability(feature).is_ok(), "{feature}");
        assert!(curve_menu::enabled(feature));
        assert_eq!(curve_menu::disabled_reason(feature), None);
    }
    // Every other row is disabled, has a visible reason, and cannot dispatch.
    for binding in curve_menu::BINDINGS
        .iter()
        .filter(|binding| !binding.enabled())
    {
        assert!(!binding.disabled_reason.is_empty(), "{}", binding.l3_id);
        assert!(
            curve_menu::disabled_reason(binding.feature).is_some(),
            "{}",
            binding.feature
        );
        assert!(
            Controller::availability(binding.feature).is_err(),
            "{} unexpectedly dispatches",
            binding.feature
        );
    }
    let disabled = curve_menu::BINDINGS
        .iter()
        .filter(|binding| !binding.enabled())
        .count();
    assert_eq!(disabled, 3);
}

#[test]
fn disabled_reasons_name_the_missing_interactive_session_and_owner() {
    for binding in curve_menu::BINDINGS
        .iter()
        .filter(|binding| binding.state == BindingState::SessionMissing)
    {
        assert!(
            binding
                .disabled_reason
                .contains("command session is not wired"),
            "{}: {}",
            binding.l3_id,
            binding.disabled_reason
        );
        assert!(
            binding.disabled_reason.contains(binding.owner),
            "{}: {}",
            binding.l3_id,
            binding.disabled_reason
        );
    }
    // The two rows with an empty authored feature id receive a stable fallback ID.
    for binding in curve_menu::BINDINGS
        .iter()
        .filter(|binding| !binding.enabled())
    {
        if icons::feature_for(binding.icon) != Some(binding.feature) {
            assert!(curve_menu::by_feature(binding.feature).is_some());
        }
        if binding.state == BindingState::OwnerMissing {
            assert!(
                binding
                    .disabled_reason
                    .contains("no verified backend owner")
            );
        } else {
            assert!(binding.disabled_reason.contains(binding.owner));
        }
    }
}

#[test]
fn line_and_point_commands_still_create_geometry_through_the_same_ids() {
    let mut controller = Controller::new(Document::new("curve menu"), Limits::default()).unwrap();
    controller.submit("Line").unwrap();
    controller.submit("0,0,0").unwrap();
    controller.submit("3,4,0").unwrap();
    assert_eq!(controller.document().entities.len(), 1);
    let undo = controller.coordinator().undo_len();
    assert_eq!(undo, 1);
    controller.submit("Point").unwrap();
    controller.submit("1,2,3").unwrap();
    assert_eq!(controller.document().entities.len(), 2);
    assert_eq!(controller.coordinator().undo_len(), 2);
}
