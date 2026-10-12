#[path = "common/layers.rs"]
mod common;
use common::*;
use openmatrix9_rust::layer_state::*;

#[test]
fn unlock_parent_keeps_child_and_object_own_locks() {
    let mut s = snapshot();
    s.layers[0].locked = true;
    s.layers[2].locked = true;
    s.layers[2].persistent_locked = Some(true);
    s.objects[0].locked = true;
    assert!(effective_state(&s, "detail-curve").unwrap().locked);
    s.layers[0].locked = false;
    assert!(effective_state(&s, "ring").unwrap().locked);
    assert!(effective_state(&s, "detail-curve").unwrap().locked);
    s.layers[2].locked = false;
    s.layers[2].persistent_locked = Some(false);
    assert!(!effective_state(&s, "detail-curve").unwrap().locked);
}
#[test]
fn visibility_and_color_are_inherited_without_changing_object_values() {
    let mut s = snapshot();
    s.layers[0].visible = false;
    s.layers[2].rgb = [1, 2, 3];
    assert!(!effective_state(&s, "detail-curve").unwrap().visible);
    s.layers[0].visible = true;
    s.layers[2].visible = false;
    s.layers[2].persistent_visible = Some(false);
    assert!(!effective_state(&s, "detail-curve").unwrap().visible);
    s.layers[2].visible = true;
    assert_eq!(effective_state(&s, "detail-curve").unwrap().rgb, [1, 2, 3]);
    s.objects[2].color_source = ColorSource::ByObject;
    assert_eq!(effective_state(&s, "detail-curve").unwrap().rgb, [9, 8, 7]);
}
#[test]
fn locked_visible_objects_snap_and_export_but_cannot_mutate() {
    let mut s = snapshot();
    s.layers[0].locked = true;
    let effective = effective_state(&s, "ring").unwrap();
    assert!(effective.snap_eligible());
    assert!(!effective.selectable());
    assert_eq!(
        can_mutate(&s, &["stone".into(), "ring".into()], &LayerOperation::Edit),
        Err(LayerError::Locked)
    );
    assert!(can_mutate(&s, &["ring".into()], &LayerOperation::Export).is_ok());
    s.layers[0].visible = false;
    assert!(!effective_state(&s, "ring").unwrap().snap_eligible());
}
#[test]
fn assignment_rejects_locked_destination_and_unknown_input_atomically() {
    let mut s = snapshot();
    s.layers[1].locked = true;
    assert_eq!(
        can_mutate(
            &s,
            &["ring".into()],
            &LayerOperation::AssignLayer("gem".into())
        ),
        Err(LayerError::Locked)
    );
    assert_eq!(
        can_mutate(
            &s,
            &["ring".into(), "absent".into()],
            &LayerOperation::Delete
        ),
        Err(LayerError::MissingObject)
    );
    assert_eq!(
        can_mutate(&s, &[], &LayerOperation::Edit),
        Err(LayerError::EmptySelection)
    );
}
#[test]
fn rejects_cycles_missing_parents_and_ambiguous_paths_but_allows_same_leaf() {
    let mut s = snapshot();
    s.layers
        .push(layer("gem-detail", Some("gem"), &["Gem 01", "Detail"]));
    assert!(validate_layers(&s).is_ok());
    s.layers.push(layer("duplicate", None, &["mEtAl 01"]));
    assert_eq!(validate_layers(&s), Err(LayerError::AmbiguousPath));
    s.layers.pop();
    s.layers[2].parent_id = Some("missing".into());
    assert_eq!(validate_layers(&s), Err(LayerError::MissingLayer));
    s.layers[2].parent_id = Some("metal".into());
    s.layers[0].parent_id = Some("detail".into());
    assert_eq!(validate_layers(&s), Err(LayerError::Cycle));
}
#[test]
fn validates_full_path_and_object_identity_without_normalizing_distinct_rgb_layers() {
    let mut s = snapshot();
    s.layers[1].rgb = s.layers[0].rgb;
    assert!(validate_layers(&s).is_ok());
    s.layers[2].path_components[0] = "Wrong parent".into();
    assert_eq!(validate_layers(&s), Err(LayerError::InvalidPath));
    let mut s = snapshot();
    s.objects[1].id = "ring".into();
    assert_eq!(validate_layers(&s), Err(LayerError::DuplicateId));
}
#[test]
fn indexed_queries_are_metadata_only_and_membership_is_exact() {
    let s = snapshot();
    let index = LayerIndex::new(&s).unwrap();
    assert_eq!(index.members("metal").unwrap(), &[0]);
    assert_eq!(index.members("detail").unwrap(), &[2]);
    assert_eq!(
        index.effective("ring").unwrap(),
        effective_state(&s, "ring").unwrap()
    );
    assert_eq!(path_key(&["Straße".into()]), path_key(&["STRAßE".into()]));
    assert_ne!(path_key(&["Straße".into()]), path_key(&["STRASSE".into()]));
}
