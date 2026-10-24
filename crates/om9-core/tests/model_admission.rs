// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::budget::{Limits, Usage};
use om9_core::geometry::{
    Bounds, Cloud, Curve, Geometry, Mesh, NurbsCurve, NurbsSurface, Transform, Unit,
};
use om9_core::model::{Definition, Document, Entity, Permissions};

fn rational_quarter_circle() -> NurbsCurve {
    NurbsCurve {
        degree: 2,
        poles: vec![[10.0, 0.0, 0.0], [10.0, 10.0, 0.0], [0.0, 10.0, 0.0]],
        weights: vec![1.0, std::f64::consts::FRAC_1_SQRT_2, 1.0],
        knots: vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
        domain: [0.0, 1.0],
        periodic: false,
        reversed: false,
        seam: None,
    }
}

#[test]
fn finite_and_nurbs_admission_rejects_before_entity_creation() {
    assert!(
        Geometry::Point([f64::NAN, 0.0, 0.0])
            .validate(&Limits::default())
            .is_err()
    );
    let curve = rational_quarter_circle();
    assert!(
        Geometry::Curve(Curve::Nurbs(curve.clone()))
            .validate(&Limits::default())
            .is_ok()
    );
    for bad in 0..5 {
        let mut c = curve.clone();
        match bad {
            0 => c.weights[1] = 0.0,
            1 => c.weights.pop().map(|_| ()).unwrap(),
            2 => c.knots[3] = -1.0,
            3 => c.domain = [1.0, 0.0],
            _ => c.poles[0][2] = f64::INFINITY,
        }
        assert!(
            Geometry::Curve(Curve::Nurbs(c))
                .validate(&Limits::default())
                .is_err(),
            "case {bad}"
        );
    }
}

#[test]
fn surface_grid_dimensions_are_checked_without_overflow() {
    let surface = NurbsSurface {
        degree: [1, 1],
        size: [usize::MAX, 2],
        poles: vec![],
        weights: vec![],
        knots: [vec![0.0, 0.0, 1.0, 1.0], vec![0.0, 0.0, 1.0, 1.0]],
        domain: [[0.0, 1.0], [0.0, 1.0]],
        periodic: [false, false],
        reversed: false,
    };
    assert!(
        Geometry::Surface(surface)
            .validate(&Limits::default())
            .is_err()
    );
}

#[test]
fn mesh_and_cloud_have_distinct_master_types_and_ordered_channels() {
    let mesh = Geometry::Mesh(Mesh {
        vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        faces: vec![vec![0, 1, 2]],
        normals: vec![],
        colors: vec![],

        corner_uvs: Vec::new(),
    });
    assert!(mesh.validate(&Limits::default()).is_ok());
    let cloud = Geometry::Cloud(Cloud {
        points: vec![[2.0, 3.0, 4.0], [5.0, 6.0, 7.0]],
        normals: vec![],
        colors: vec![[255, 0, 0, 255], [0, 255, 0, 255]],
        hidden: vec![false, true],
    });
    assert!(cloud.validate(&Limits::default()).is_ok());
    assert_ne!(mesh.kind(), cloud.kind());
    let invalid = Geometry::Mesh(Mesh {
        vertices: vec![[0.0; 3]],
        faces: vec![vec![0, 1, 2]],
        normals: vec![],
        colors: vec![],

        corner_uvs: Vec::new(),
    });
    assert!(invalid.validate(&Limits::default()).is_err());
    let invalid = Geometry::Cloud(Cloud {
        points: vec![[0.0; 3]],
        normals: vec![],
        colors: vec![[0; 4]; 2],
        hidden: vec![],
    });
    assert!(invalid.validate(&Limits::default()).is_err());
}

#[test]
fn bounds_distinguish_unknown_empty_and_invalid_range() {
    assert_ne!(Bounds::Unknown, Bounds::Empty);
    assert!(
        Bounds::Finite {
            min: [1.0, 0.0, 0.0],
            max: [0.0, 1.0, 1.0]
        }
        .validate()
        .is_err()
    );
    assert!(
        Bounds::Finite {
            min: [0.0; 3],
            max: [1.0; 3]
        }
        .validate()
        .is_ok()
    );
}

#[test]
fn aggregate_budget_rejects_overflow_and_sum_of_admitted_items() {
    let limits = Limits {
        max_bytes: 100,
        ..Limits::default()
    };
    let mut total = Usage {
        bytes: 60,
        ..Usage::default()
    };
    assert!(
        total
            .add(
                Usage {
                    bytes: 41,
                    ..Usage::default()
                },
                &limits
            )
            .is_err()
    );
    assert_eq!(
        total.bytes, 60,
        "rejected reservation must not change bookkeeping"
    );
    let mut overflow = Usage {
        bytes: u64::MAX,
        ..Usage::default()
    };
    assert!(
        overflow
            .add(
                Usage {
                    bytes: 1,
                    ..Usage::default()
                },
                &Limits {
                    max_bytes: u64::MAX,
                    ..Limits::default()
                }
            )
            .is_err()
    );
    assert_eq!(overflow.bytes, u64::MAX);
}

#[test]
fn reused_label_never_resolves_a_deleted_object_stamp() {
    let mut doc = Document::new("model");
    let original = Entity::new(
        "same label",
        Geometry::Point([1.0, 2.0, 3.0]),
        doc.current_layer,
    );
    let id = original.id;
    doc.entities.insert(id, original);
    let stamp = doc.stamp(id).unwrap();
    doc.entities.remove(&id);
    let replacement = Entity::new(
        "same label",
        Geometry::Point([4.0, 5.0, 6.0]),
        doc.current_layer,
    );
    assert_ne!(id, replacement.id);
    doc.entities.insert(replacement.id, replacement);
    assert!(doc.resolve(&stamp).is_err());
    let other = Document::new("model");
    assert_ne!(doc.id, other.id);
    assert!(other.resolve(&stamp).is_err());
}

#[test]
fn permission_visibility_selection_edit_and_locked_reference_are_separate() {
    let mut doc = Document::new("permissions");
    let layer = om9_core::model::Layer::new("fixture layer", [220, 220, 220, 255]);
    let layer_id = layer.id;
    doc.layers.insert(layer_id, layer);
    doc.layer_order.push(layer_id);
    doc.current_layer = Some(layer_id);
    let mut entity = Entity::new("point", Geometry::Point([0.0; 3]), doc.current_layer);
    entity.attributes.selectable = false;
    let id = entity.id;
    doc.entities.insert(id, entity);
    assert_eq!(
        doc.permissions(id, true).unwrap(),
        Permissions {
            visible: true,
            selectable: false,
            editable: true,
            reference_snappable: true
        }
    );
    doc.layers.get_mut(&layer_id).unwrap().visible = false;
    assert!(!doc.permissions(id, true).unwrap().visible);
    doc.layers.get_mut(&layer_id).unwrap().visible = true;
    assert!(
        !doc.permissions(id, true).unwrap().selectable,
        "Hide/Show must not enable selection"
    );
    doc.layers.get_mut(&layer_id).unwrap().locked = true;
    assert!(!doc.permissions(id, true).unwrap().editable);
    assert!(doc.permissions(id, true).unwrap().reference_snappable);
    assert!(!doc.permissions(id, false).unwrap().reference_snappable);
}

#[test]
fn nested_instances_compose_world_placement_once_and_preserve_definition() {
    let mut doc = Document::new("instances");
    let member = Entity::new(
        "member",
        Geometry::Point([2.0, 3.0, 4.0]),
        doc.current_layer,
    );
    let member_id = member.id;
    let inner = Definition::new("inner", vec![member]);
    let inner_id = inner.id;
    doc.definitions.insert(inner_id, inner);
    let mut inner_instance = Entity::new(
        "inner instance",
        Geometry::Instance(inner_id),
        doc.current_layer,
    );
    inner_instance.placement = Transform::translation([10.0, 0.0, 0.0]);
    let nested_id = inner_instance.id;
    let outer = Definition::new("outer", vec![inner_instance]);
    let outer_id = outer.id;
    doc.definitions.insert(outer_id, outer);
    let mut instance = Entity::new(
        "outer instance",
        Geometry::Instance(outer_id),
        doc.current_layer,
    );
    instance.placement = Transform::scale([-2.0, 3.0, 1.0]).unwrap();
    let id = instance.id;
    doc.entities.insert(id, instance);
    doc.validate(&Limits::default()).unwrap();
    let world = doc.member_transform(id, &[nested_id, member_id]).unwrap();
    assert_eq!(world.point([2.0, 3.0, 4.0]).unwrap(), [-24.0, 9.0, 4.0]);
    let Geometry::Point(p) = doc.definitions[&inner_id].members[0].geometry else {
        panic!()
    };
    assert_eq!(p, [2.0, 3.0, 4.0]);
}

#[test]
fn unit_conversion_changes_lengths_without_scaling_dimensionless_basis() {
    assert_eq!(Unit::Inch.length_to_mm(2.0).unwrap(), 50.8);
    assert_eq!(Unit::Millimeter.length_to_mm(10.0).unwrap(), 10.0);
    let mut c = rational_quarter_circle();
    let weights = c.weights.clone();
    let knots = c.knots.clone();
    c.convert_lengths(Unit::Inch).unwrap();
    assert_eq!(c.poles[0], [254.0, 0.0, 0.0]);
    assert_eq!(c.weights, weights);
    assert_eq!(c.knots, knots);
    assert!(Unit::Inch.length_to_mm(f64::MAX).is_err());
}

#[test]
fn point_map_cannot_be_admitted_below_its_fixed_entity_storage_floor() {
    let mut d = Document::new("fixed storage");
    for i in 0..32 {
        let e = Entity::new("p", Geometry::Point([i as f64, 0., 0.]), None);
        d.entities.insert(e.id, e);
    }
    let floor = (d.entities.len() * std::mem::size_of::<Entity>()) as u64;
    let limits = Limits {
        max_bytes: floor - 1,
        ..Limits::default()
    };
    assert!(matches!(
        d.validate(&limits),
        Err(om9_core::Error::Budget(_))
    ));
}

#[test]
fn retained_unused_string_and_geometry_capacity_is_part_of_the_byte_budget() {
    let mut name = String::with_capacity(64 * 1024);
    name.push('d');
    let mut d = Document::new(name);
    let mut points = Vec::with_capacity(4096);
    points.push([1., 2., 3.]);
    let mut label = String::with_capacity(32 * 1024);
    label.push('p');
    let e = Entity::new(
        label,
        Geometry::Cloud(om9_core::geometry::Cloud {
            points,
            normals: vec![],
            colors: vec![],
            hidden: vec![],
        }),
        None,
    );
    d.entities.insert(e.id, e);
    assert!(matches!(
        d.validate(&Limits {
            max_bytes: 128 * 1024,
            ..Limits::default()
        }),
        Err(om9_core::Error::Budget(_))
    ));
}

#[test]
fn coordinator_retention_includes_undo_label_capacity_and_both_document_snapshots() {
    use om9_core::transaction::*;
    let mut d = Document::new("undo storage");
    let e = Entity::new("p", Geometry::Point([0.; 3]), None);
    let oid = e.id;
    d.entities.insert(oid, e);
    let did = d.id;
    let l = Limits::default();
    let old = d.validate(&l).unwrap().bytes;
    let mut c = TransactionCoordinator::new(World::new(vec![d], &l).unwrap(), l).unwrap();
    let mut label = String::with_capacity(64 * 1024);
    label.push_str("Move");
    let label_capacity = label.capacity() as u64;
    let p = c
        .prepare(TransactionRequest::new(
            label,
            vec![Mutation::Placement {
                source: c.world().documents[&did].stamp(oid).unwrap(),
                placement: Transform::translation([1., 0., 0.]),
            }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    let current = c.world().documents[&did].validate(&l).unwrap().bytes;
    assert!(c.retained_usage().unwrap().bytes >= old + current + label_capacity);
    c.undo().unwrap();
    assert!(c.retained_usage().unwrap().bytes >= old + current + label_capacity);
}

#[test]
fn replacing_redo_branch_accounts_old_redo_and_new_undo_together_before_acceptance() {
    use om9_core::transaction::*;
    let d = Document::new("redo peak");
    let did = d.id;
    let l = Limits {
        max_history: 1,
        ..Limits::default()
    };
    let mut c = TransactionCoordinator::new(World::new(vec![d], &l).unwrap(), l).unwrap();
    let a = Entity::new("A", Geometry::Point([1.; 3]), None);
    let aid = a.id;
    let p = c
        .prepare(TransactionRequest::new(
            "A",
            vec![Mutation::Insert {
                document: did,
                entity: a,
            }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    c.undo().unwrap();
    let before = c.snapshot();
    let b = Entity::new("B", Geometry::Point([2.; 3]), None);
    assert!(matches!(
        c.prepare(TransactionRequest::new(
            "B",
            vec![Mutation::Insert {
                document: did,
                entity: b
            }]
        )),
        Err(om9_core::Error::Budget(_))
    ));
    assert_eq!(c.snapshot(), before);
    c.redo().unwrap();
    assert!(c.world().documents[&did].entities.contains_key(&aid));
}
