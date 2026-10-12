// SPDX-License-Identifier: LGPL-2.1-or-later
use openmatrix9_rust::core_objects::{Registry, Stamp};
use openmatrix9_rust::geometry_contract::*;
use openmatrix9_rust::modeling_exchange::{GeometryKind, capabilities};

fn reference() -> GeometryRef {
    GeometryRef {
        stamp: Stamp {
            document: 11,
            object: 7,
            generation: 9,
            revision: 3,
            epoch: 2,
        },
        subelement_ref: Some("Face1".into()),
        local_to_world: Fact::Unknown,
    }
}

fn exchange_options() -> OperationOptions {
    OperationOptions::WorkingExchange {
        archive_version: 5,
        conversion: ConversionPolicy::ExistingConverters,
        preservation: PreservationPolicy::ReportObservedFacts,
    }
}

#[test]
fn inspection_retains_uninspected_facts_and_rejects_future_schema() {
    // Catch fabricated topology/placement or accepting a schema we cannot interpret.
    let mut descriptor = GeometryDescriptor::uninspected(reference(), RepresentationKind::Brep);
    assert_eq!(descriptor.reference.stamp.revision, 3);
    assert_eq!(descriptor.reference.local_to_world, Fact::Unknown);
    assert_eq!(descriptor.unit_context_revision, Fact::Unknown);
    assert_eq!(descriptor.coordinates, Fact::Unknown);
    assert_eq!(descriptor.topology.solid, Fact::Unknown);
    assert_eq!(descriptor.topology.faces, Fact::Unknown);
    assert_eq!(descriptor.source_attributes, Fact::Unknown);
    assert!(descriptor.validate_schema().is_ok());
    descriptor.schema_version = 27;
    assert_eq!(
        descriptor.validate_schema(),
        Err(DescriptorError::UnsupportedSchema(27))
    );
}

#[test]
fn descriptor_reuses_existing_stamp_for_stale_object_rejection() {
    // Catch a second identity lifecycle or losing the captured generation/revision.
    let mut objects = Registry::new();
    let document = objects.document_open().unwrap();
    let stamp = objects.observe(document, 42).unwrap();
    let descriptor = GeometryDescriptor::uninspected(
        GeometryRef {
            stamp,
            subelement_ref: None,
            local_to_world: Fact::Unknown,
        },
        RepresentationKind::Curve,
    );
    assert!(objects.validate(descriptor.reference.stamp));
    assert!(objects.changed(document, 42));
    assert!(!objects.validate(descriptor.reference.stamp));
    objects.removed(document, 42);
    assert_ne!(
        objects.observe(document, 42).unwrap().generation,
        descriptor.reference.stamp.generation
    );
}

#[test]
fn periodic_closed_surface_facts_do_not_classify_a_solid() {
    // Catch inferring volume/solid status from parametric closure.
    let mut descriptor = GeometryDescriptor::uninspected(reference(), RepresentationKind::Surface);
    descriptor.representation = RepresentationDescriptor::Surface(SurfaceFacts {
        form: Fact::Known(SurfaceForm::Analytic),
        facts_scope: Fact::Known(GeometryFactScope::SupportingGeometry),
        domains: Fact::Known([[0.0, std::f64::consts::TAU], [0.0, 7.0]]),
        domain_scope: Fact::Known(DomainScope::SupportingGeometry),
        closed: [Fact::Known(true), Fact::Known(false)],
        periodic: [Fact::Known(true), Fact::Known(false)],
        rational: Fact::Unknown,
    });
    descriptor.topology.closed = Fact::Known(true);
    assert_eq!(
        descriptor.representation.kind(),
        RepresentationKind::Surface
    );
    assert_eq!(descriptor.topology.solid, Fact::Unknown);
    assert_eq!(descriptor.topology.solid_orientation, Fact::Unknown);
    assert_eq!(
        operation_capability(
            OperationId::ExportWorkingV5,
            descriptor.representation.kind(),
            exchange_options()
        ),
        Err(CapabilityError::UnsupportedRepresentation(
            RepresentationKind::Surface
        ))
    );
}

#[test]
fn seam_occurrences_keep_order_and_distinct_edge_identity_count() {
    // Catch deduplicating a seam use or confusing two uses with two identities.
    let seam = EdgeUse {
        edge_id: 12,
        orientation: Orientation::Forward,
        pcurve_id: Fact::Known(3),
        seam: Fact::Known(true),
    };
    let reversed = EdgeUse {
        orientation: Orientation::Reversed,
        pcurve_id: Fact::Known(4),
        ..seam.clone()
    };
    let topology = TopologyDescriptor {
        faces: Fact::Known(vec![FaceDescriptor {
            surface_id: Fact::Known(8),
            orientation: Fact::Known(Orientation::Forward),
            loops: Fact::Known(vec![TrimLoop {
                role: LoopRole::Outer,
                edge_uses: vec![seam, reversed],
            }]),
        }]),
        ..TopologyDescriptor::uninspected()
    };
    assert_eq!(topology.edge_use_count(), Fact::Known(2));
    assert_eq!(topology.edge_identity_count(), Fact::Known(1));
    let Fact::Known(faces) = &topology.faces else {
        panic!("faces inspected")
    };
    let Fact::Known(loops) = &faces[0].loops else {
        panic!("loops inspected")
    };
    assert_eq!(loops[0].edge_uses[0].pcurve_id, Fact::Known(3));
    assert_eq!(loops[0].edge_uses[1].orientation, Orientation::Reversed);
}

#[test]
fn incomplete_topology_does_not_report_zero_edges() {
    // Catch presenting absent inspection as a measured empty shape.
    let mut topology = TopologyDescriptor::uninspected();
    assert_eq!(topology.edge_use_count(), Fact::Unknown);
    assert_eq!(topology.edge_identity_count(), Fact::Unknown);
    topology.faces = Fact::Known(vec![FaceDescriptor {
        surface_id: Fact::Unknown,
        orientation: Fact::Unknown,
        loops: Fact::Unknown,
    }]);
    assert_eq!(topology.edge_use_count(), Fact::Unknown);
    assert_eq!(topology.edge_identity_count(), Fact::Unknown);
    topology.faces = Fact::Known(vec![]);
    assert_eq!(topology.edge_use_count(), Fact::Known(0));
    assert_eq!(topology.edge_identity_count(), Fact::Known(0));
}

#[test]
fn inspection_is_read_only_even_for_unknown_or_retained_representation() {
    // Catch routing descriptor reads to a solver or disallowing unknown metadata.
    for kind in [
        RepresentationKind::Unknown,
        RepresentationKind::Retained,
        RepresentationKind::Instance,
        RepresentationKind::Curve,
    ] {
        let capability = operation_capability(
            OperationId::Inspect,
            kind,
            OperationOptions::InspectDescriptor,
        )
        .unwrap();
        assert_eq!(capability.backend, Backend::NativeInspection);
        assert!(
            capability
                .constraints
                .contains(&OperationConstraint::ReadOnly)
        );
        assert!(
            capability
                .constraints
                .contains(&OperationConstraint::ValidateSourceStamp)
        );
        assert!(!capability.evidence_id.is_empty());
    }
}

#[test]
fn working_exchange_routes_only_existing_independent_geometry() {
    // Catch accidentally enabling a retained/unknown payload or a new solver.
    for (kind, backend) in [
        (RepresentationKind::Point, Backend::OcctOpenNurbsExchange),
        (RepresentationKind::Curve, Backend::OcctOpenNurbsExchange),
        (
            RepresentationKind::Polycurve,
            Backend::OcctOpenNurbsExchange,
        ),
        (RepresentationKind::Brep, Backend::OcctOpenNurbsExchange),
        (
            RepresentationKind::TrimmedFace,
            Backend::OcctOpenNurbsExchange,
        ),
        (RepresentationKind::Shell, Backend::OcctOpenNurbsExchange),
        (RepresentationKind::Solid, Backend::OcctOpenNurbsExchange),
        (RepresentationKind::Mesh, Backend::MeshOpenNurbsExchange),
        (
            RepresentationKind::PointCloud,
            Backend::PointCloudOpenNurbsExchange,
        ),
    ] {
        for operation in [OperationId::ImportWorkingV5, OperationId::ExportWorkingV5] {
            let capability = operation_capability(operation, kind, exchange_options()).unwrap();
            assert_eq!(capability.backend, backend);
            assert!(
                capability
                    .constraints
                    .contains(&OperationConstraint::ExistingNativeConverters)
            );
            assert!(
                capability
                    .constraints
                    .contains(&OperationConstraint::ReportConversionFacts)
            );
            assert!(
                capability
                    .constraints
                    .contains(&OperationConstraint::ValidateSourceStamp)
            );
        }
    }
    for kind in [
        RepresentationKind::Surface,
        RepresentationKind::LightweightExtrusion,
        RepresentationKind::Instance,
        RepresentationKind::Annotation,
        RepresentationKind::SubD,
        RepresentationKind::Mixed,
        RepresentationKind::Retained,
        RepresentationKind::Unknown,
    ] {
        assert_eq!(
            operation_capability(OperationId::ExportWorkingV5, kind, exchange_options()),
            Err(CapabilityError::UnsupportedRepresentation(kind))
        );
    }
}

#[test]
fn option_combinations_and_unrecognized_options_reject() {
    // Catch silently ignoring strict preservation, conversion, version or unknown options.
    let options = [
        OperationOptions::InspectDescriptor,
        OperationOptions::Unrecognized(87),
        OperationOptions::WorkingExchange {
            archive_version: 6,
            conversion: ConversionPolicy::ExistingConverters,
            preservation: PreservationPolicy::ReportObservedFacts,
        },
        OperationOptions::WorkingExchange {
            archive_version: 5,
            conversion: ConversionPolicy::PreserveRepresentation,
            preservation: PreservationPolicy::ReportObservedFacts,
        },
        OperationOptions::WorkingExchange {
            archive_version: 5,
            conversion: ConversionPolicy::Tessellate,
            preservation: PreservationPolicy::ReportObservedFacts,
        },
        OperationOptions::WorkingExchange {
            archive_version: 5,
            conversion: ConversionPolicy::Heal,
            preservation: PreservationPolicy::ReportObservedFacts,
        },
        OperationOptions::WorkingExchange {
            archive_version: 5,
            conversion: ConversionPolicy::ExistingConverters,
            preservation: PreservationPolicy::RequireLocus,
        },
        OperationOptions::WorkingExchange {
            archive_version: 5,
            conversion: ConversionPolicy::ExistingConverters,
            preservation: PreservationPolicy::RequireParameterization,
        },
        OperationOptions::WorkingExchange {
            archive_version: 5,
            conversion: ConversionPolicy::ExistingConverters,
            preservation: PreservationPolicy::RequireTopology,
        },
        OperationOptions::WorkingExchange {
            archive_version: 5,
            conversion: ConversionPolicy::ExistingConverters,
            preservation: PreservationPolicy::RequireAll,
        },
    ];
    for options in options {
        assert_eq!(
            operation_capability(
                OperationId::ExportWorkingV5,
                RepresentationKind::Curve,
                options
            ),
            Err(CapabilityError::UnsupportedOptions)
        );
    }
    assert_eq!(
        operation_capability(
            OperationId::Inspect,
            RepresentationKind::Curve,
            exchange_options()
        ),
        Err(CapabilityError::UnsupportedOptions)
    );
}

#[test]
fn unsupported_operations_never_inherit_coarse_boolean_capability() {
    // Catch granting a solver from the legacy CAD capability bit or unknown ID.
    assert!(capabilities(GeometryKind::CadBrep).boolean);
    for operation in [
        OperationId::Boolean,
        OperationId::EditCurve,
        OperationId::EditSurface,
        OperationId::Unrecognized(99),
    ] {
        assert_eq!(
            operation_capability(operation, RepresentationKind::Solid, exchange_options()),
            Err(CapabilityError::UnsupportedOperation(operation))
        );
    }
}

#[test]
fn legacy_mapping_keeps_brep_subtype_uninspected_and_mesh_separate() {
    // Catch refining legacy CAD to a solid, or treating a render mesh as CAD input.
    assert_eq!(
        RepresentationKind::from_legacy(GeometryKind::CadBrep),
        RepresentationKind::Brep
    );
    assert_eq!(
        RepresentationKind::from_legacy(GeometryKind::Mesh),
        RepresentationKind::Mesh
    );
    for legacy in [
        GeometryKind::CadPoint,
        GeometryKind::CadCurve,
        GeometryKind::CadBrep,
        GeometryKind::Mesh,
        GeometryKind::PointCloud,
        GeometryKind::SubD,
        GeometryKind::Mixed,
        GeometryKind::Retained,
        GeometryKind::Unknown,
    ] {
        let kind = RepresentationKind::from_legacy(legacy);
        assert_eq!(
            operation_capability(OperationId::ExportWorkingV5, kind, exchange_options()).is_ok(),
            capabilities(legacy).export_v5
        );
    }
}

#[test]
fn conversion_permission_creates_no_preservation_or_parameter_map() {
    // Catch treating routing permission as conversion evidence.
    assert!(
        operation_capability(
            OperationId::ExportWorkingV5,
            RepresentationKind::Curve,
            exchange_options()
        )
        .is_ok()
    );
    let mut report =
        ConversionReport::uninspected(RepresentationKind::Curve, RepresentationKind::Curve);
    assert_eq!(report.locus, Fact::Unknown);
    assert_eq!(report.parameterization, Fact::Unknown);
    assert_eq!(report.topology, Fact::Unknown);
    assert_eq!(report.parameter_map, Fact::Unknown);
    assert_eq!(report.deviation, Fact::Unknown);
    assert!(!report.source_parameter_is_reusable());
    report.locus = Fact::Known(Preservation::Preserved);
    report.topology = Fact::Known(Preservation::Preserved);
    assert!(!report.source_parameter_is_reusable());
    report.parameter_map = Fact::Known(ParameterMap {
        source_domain: [0.0, 1.0],
        target_domain: [2.0, 4.0],
        scale: 2.0,
        offset: 2.0,
        evidence_id: "independent-map-fixture".into(),
    });
    assert!(!report.source_parameter_is_reusable());
    report.parameterization = Fact::Known(Preservation::Preserved);
    assert!(!report.source_parameter_is_reusable());
    report.parameter_map = Fact::Unknown;
    assert!(report.source_parameter_is_reusable());
    report.schema_version = 27;
    assert!(!report.source_parameter_is_reusable());
}

fn affine_map() -> ParameterMap {
    ParameterMap {
        source_domain: [0.0, 1.0],
        target_domain: [2.0, 4.0],
        scale: 2.0,
        offset: 2.0,
        evidence_id: "independent-affine-map".into(),
    }
}

#[test]
fn map_queries_require_finite_consistent_domains_and_evidence() {
    // Catch trusting a declared map with invalid coefficients/domains/provenance.
    for (map, expected) in [
        (
            ParameterMap {
                scale: f64::NAN,
                ..affine_map()
            },
            ParameterMapError::NonFinite,
        ),
        (
            ParameterMap {
                offset: f64::INFINITY,
                ..affine_map()
            },
            ParameterMapError::NonFinite,
        ),
        (
            ParameterMap {
                source_domain: [f64::NAN, 1.0],
                ..affine_map()
            },
            ParameterMapError::NonFinite,
        ),
        (
            ParameterMap {
                target_domain: [2.0, f64::INFINITY],
                ..affine_map()
            },
            ParameterMapError::NonFinite,
        ),
        (
            ParameterMap {
                source_domain: [1.0, 1.0],
                ..affine_map()
            },
            ParameterMapError::InvalidDomain,
        ),
        (
            ParameterMap {
                target_domain: [4.0, 2.0],
                ..affine_map()
            },
            ParameterMapError::InvalidDomain,
        ),
        (
            ParameterMap {
                scale: 0.0,
                ..affine_map()
            },
            ParameterMapError::InvalidDomain,
        ),
        (
            ParameterMap {
                evidence_id: "  ".into(),
                ..affine_map()
            },
            ParameterMapError::MissingEvidence,
        ),
        (
            ParameterMap {
                target_domain: [2.0, 5.0],
                ..affine_map()
            },
            ParameterMapError::InconsistentDomain,
        ),
    ] {
        let mut report =
            ConversionReport::uninspected(RepresentationKind::Curve, RepresentationKind::Curve);
        report.parameterization = Fact::Known(Preservation::Preserved);
        assert_eq!(map.validate(), Err(expected));
        report.parameter_map = Fact::Known(map);
        assert_eq!(report.map_source_parameter(0.25), Err(expected));
        assert!(!report.source_parameter_is_reusable());
    }
}

#[test]
fn affine_map_queries_check_bounds_without_claiming_identity() {
    // Catch reusing raw parameters for nonidentity maps or accepting out-of-domain queries.
    let mut report =
        ConversionReport::uninspected(RepresentationKind::Curve, RepresentationKind::Curve);
    assert_eq!(
        report.map_source_parameter(0.25),
        Err(ParameterMapError::Uninspected)
    );
    report.parameter_map = Fact::Known(affine_map());
    assert_eq!(report.map_source_parameter(0.25), Ok(2.5));
    assert_eq!(report.map_source_parameter(0.0), Ok(2.0));
    assert_eq!(report.map_source_parameter(1.0), Ok(4.0));
    assert_eq!(
        report.map_source_parameter(-0.01),
        Err(ParameterMapError::OutsideDomain)
    );
    assert_eq!(
        report.map_source_parameter(f64::NAN),
        Err(ParameterMapError::NonFinite)
    );
    assert!(!report.source_parameter_is_reusable());
    report.parameter_map = Fact::Known(ParameterMap {
        scale: -2.0,
        offset: 4.0,
        ..affine_map()
    });
    assert_eq!(report.map_source_parameter(0.25), Ok(3.5));
    report.parameter_map = Fact::Known(ParameterMap {
        target_domain: [0.0, 1.0],
        scale: 1.0,
        offset: 0.0,
        ..affine_map()
    });
    assert_eq!(report.map_source_parameter(0.25), Ok(0.25));
    assert!(!report.source_parameter_is_reusable());
    report.parameterization = Fact::Known(Preservation::Preserved);
    assert!(report.source_parameter_is_reusable());
    report.schema_version = 27;
    assert_eq!(
        report.map_source_parameter(0.25),
        Err(ParameterMapError::UnsupportedSchema(27))
    );
    assert!(!report.source_parameter_is_reusable());
}

#[test]
fn cloned_companion_owns_topology_and_conversion_evidence() {
    // Catch sharing mutable source buffers between snapshots.
    let mut original =
        GeometryDescriptor::uninspected(reference(), RepresentationKind::TrimmedFace);
    original.topology.faces = Fact::Known(vec![FaceDescriptor {
        surface_id: Fact::Known(1),
        orientation: Fact::Known(Orientation::Forward),
        loops: Fact::Known(vec![TrimLoop {
            role: LoopRole::Hole,
            edge_uses: vec![EdgeUse {
                edge_id: 2,
                orientation: Orientation::Reversed,
                pcurve_id: Fact::Known(3),
                seam: Fact::Known(false),
            }],
        }]),
    }]);
    let mut companion = original.clone();
    let Fact::Known(faces) = &mut companion.topology.faces else {
        panic!("faces")
    };
    let Fact::Known(loops) = &mut faces[0].loops else {
        panic!("loops")
    };
    loops[0].edge_uses[0].edge_id = 9;
    assert_ne!(original.topology.faces, companion.topology.faces);
    let mut report =
        ConversionReport::uninspected(RepresentationKind::TrimmedFace, RepresentationKind::Brep);
    report.warnings.push("native subtype changed".into());
    let mut independent_report = report.clone();
    independent_report.warnings[0].clear();
    assert_eq!(report.warnings, ["native subtype changed"]);
}
