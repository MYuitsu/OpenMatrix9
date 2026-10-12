use openmatrix9_rust::layer_collection::*;
use openmatrix9_rust::layer_state::LayerError;
fn node(id: &str, role: Role, children: &[&str]) -> Node {
    Node {
        id: id.into(),
        label: id.into(),
        native_type: "native fixture type".into(),
        role,
        children: children.iter().map(|s| s.to_string()).collect(),
    }
}
#[test]
fn session_deduplicates_transparent_groups_and_keeps_all_geometry_in_document_order() {
    let nodes = vec![
        node("notes", Role::Metadata, &[]),
        node("a", Role::Geometry, &[]),
        node("outer", Role::Group, &["inner", "a"]),
        node("inner", Role::Group, &["b"]),
        node("b", Role::Geometry, &[]),
        node("shared", Role::Group, &["b"]),
        node("empty", Role::Group, &[]),
    ];
    let result = collect(&nodes, Scope::Session, &[]).unwrap();
    assert_eq!(result.objects, vec!["a", "b"]);
    assert!(result.unsupported.is_empty());
    assert!(result.dependencies.is_empty());
    assert_eq!(
        collect(&nodes, Scope::Selected, &["outer".into()])
            .unwrap()
            .objects,
        vec!["b", "a"]
    );
    assert_eq!(
        collect(&nodes, Scope::Selected, &["shared".into(), "inner".into()]),
        Err(LayerError::DuplicateId)
    );
}
#[test]
fn opaque_final_body_and_placed_blocks_do_not_export_internal_members_twice() {
    let nodes = vec![
        node("body", Role::Aggregate, &["history", "tip"]),
        node("history", Role::Unsupported, &[]),
        node("tip", Role::Geometry, &[]),
        node("definition", Role::Definition, &["member", "nested"]),
        node("member", Role::Geometry, &[]),
        node("nested", Role::Instance, &["inner-definition"]),
        node("inner-definition", Role::Definition, &["inner-member"]),
        node("inner-member", Role::Geometry, &[]),
        node("placed", Role::Instance, &["definition"]),
        node("independent", Role::Geometry, &[]),
    ];
    let result = collect(&nodes, Scope::Session, &[]).unwrap();
    assert_eq!(result.objects, vec!["body", "placed", "independent"]);
    assert!(result.unsupported.is_empty());
    assert_eq!(
        result.dependencies,
        vec![
            "definition",
            "member",
            "nested",
            "inner-definition",
            "inner-member"
        ]
    );
    assert_eq!(
        collect(&nodes, Scope::Selected, &["member".into()])
            .unwrap()
            .objects,
        vec!["member"]
    );
    assert_eq!(
        collect(&nodes, Scope::Selected, &["body".into(), "tip".into()]),
        Err(LayerError::DuplicateId)
    );
}
#[test]
fn unsupported_inventory_is_explicit_and_scoped_never_silently_filtered() {
    let nodes = vec![
        node("good", Role::Geometry, &[]),
        node("unknown", Role::Unsupported, &[]),
        node("group", Role::Group, &["unknown"]),
        node("definition", Role::Definition, &["bad-member"]),
        node("bad-member", Role::Unsupported, &[]),
        node("placed", Role::Instance, &["definition"]),
    ];
    let all = collect(&nodes, Scope::Session, &[]).unwrap();
    assert_eq!(
        all.unsupported
            .iter()
            .map(|n| n.id.as_str())
            .collect::<Vec<_>>(),
        vec!["unknown", "bad-member"]
    );
    assert!(
        collect(&nodes, Scope::Selected, &["good".into()])
            .unwrap()
            .unsupported
            .is_empty()
    );
    assert_eq!(
        collect(&nodes, Scope::Selected, &["group".into()])
            .unwrap()
            .unsupported[0]
            .id,
        "unknown"
    );
    assert_eq!(
        collect(&nodes, Scope::Selected, &["placed".into()])
            .unwrap()
            .unsupported[0]
            .id,
        "bad-member"
    );
    assert_eq!(
        collect(&nodes, Scope::Selected, &["definition".into()]),
        Err(LayerError::InvalidSnapshot)
    );
    assert!(
        collect(&[], Scope::Session, &[])
            .unwrap()
            .objects
            .is_empty()
    );
    assert!(
        collect(&nodes, Scope::Selected, &[])
            .unwrap()
            .objects
            .is_empty()
    );
}
#[test]
fn malformed_graphs_and_foreign_or_overlapping_selections_reject_before_export() {
    assert_eq!(
        collect(
            &[
                node("a", Role::Group, &["b"]),
                node("b", Role::Group, &["a"])
            ],
            Scope::Session,
            &[]
        ),
        Err(LayerError::Cycle)
    );
    assert_eq!(
        collect(&[node("a", Role::Group, &["missing"])], Scope::Session, &[]),
        Err(LayerError::MissingObject)
    );
    assert_eq!(
        collect(
            &[node("a", Role::Geometry, &[])],
            Scope::Selected,
            &["missing".into()]
        ),
        Err(LayerError::MissingObject)
    );
    assert_eq!(
        collect(
            &[node("a", Role::Geometry, &[])],
            Scope::Session,
            &["a".into()]
        ),
        Err(LayerError::InvalidSnapshot)
    );
    assert_eq!(
        collect(
            &[
                node("a", Role::Geometry, &[]),
                node("a", Role::Metadata, &[])
            ],
            Scope::Session,
            &[]
        ),
        Err(LayerError::DuplicateId)
    );
    assert_eq!(
        collect(
            &[node("a", Role::Metadata, &[])],
            Scope::Selected,
            &["a".into()]
        ),
        Err(LayerError::InvalidSnapshot)
    );
    assert_eq!(
        collect(
            &[
                node("a", Role::Geometry, &["b"]),
                node("b", Role::Geometry, &[])
            ],
            Scope::Session,
            &[]
        ),
        Err(LayerError::InvalidSnapshot)
    );
    let deep = (0..256)
        .map(|i| node(&i.to_string(), Role::Group, &[&(i + 1).to_string()]))
        .chain(std::iter::once(node("256", Role::Geometry, &[])))
        .collect::<Vec<_>>();
    assert_eq!(
        collect(&deep, Scope::Session, &[]),
        Err(LayerError::LimitExceeded)
    );
}
