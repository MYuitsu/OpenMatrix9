//! Value-only native gates. Native readers still own host lifetime and inspection.
//! A descriptor is an observation, never a mutation authorization. The native
//! reader must verify the existing source stamp before/after capture, include its
//! unit/world provenance and keep unidentified topology and preservation unknown.
use crate::core_objects::Stamp;
use crate::geometry_contract::*;
use crate::modeling_exchange::GeometryKind;
#[unsafe(no_mangle)]
pub extern "C" fn om9_geometry_contract_schema() -> u32 {
    SCHEMA_VERSION
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CompanionLimits {
    pub max_poles: usize,
    pub max_knots: usize,
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_geometry_companion_limits() -> CompanionLimits {
    CompanionLimits {
        max_poles: 8192,
        max_knots: 16384,
    }
}
/// Count-only companion budget, before native value extraction or JSON arrays.
/// 0 allows extraction, 1 leaves basis explicitly uninspected, 2 malformed counts.
#[unsafe(no_mangle)]
pub extern "C" fn om9_geometry_companion_counts(
    u: usize,
    v: usize,
    uk: usize,
    vk: usize,
    surface: u32,
) -> u32 {
    let limits = om9_geometry_companion_limits();
    let (poles, knots) = match surface {
        0 if u >= 2 && v == 0 && uk >= 2 && vk == 0 => (u, [uk, 0]),
        1 if u >= 2 && v >= 2 && uk >= 2 && vk >= 2 => {
            let Some(poles) = u.checked_mul(v) else {
                return 2;
            };
            (poles, [uk, vk])
        }
        _ => return 2,
    };
    u32::from(poles > limits.max_poles || knots.into_iter().any(|count| count > limits.max_knots))
}

#[unsafe(no_mangle)]
pub extern "C" fn om9_geometry_operation(
    operation: u32,
    legacy_kind: u32,
    archive_version: u32,
    options: u32,
) -> u32 {
    let op = match operation {
        1 => OperationId::Inspect,
        2 => OperationId::ImportWorkingV5,
        3 => OperationId::ExportWorkingV5,
        _ => OperationId::Unrecognized(operation),
    };
    let opts = match (operation, archive_version, options) {
        (1, 0, 0) => OperationOptions::InspectDescriptor,
        (2 | 3, 5, 1) => OperationOptions::WorkingExchange {
            archive_version,
            conversion: ConversionPolicy::ExistingConverters,
            preservation: PreservationPolicy::ReportObservedFacts,
        },
        _ => OperationOptions::Unrecognized(options),
    };
    match operation_capability(
        op,
        RepresentationKind::from_legacy(GeometryKind::from(legacy_kind)),
        opts,
    ) {
        Ok(c) => match c.backend {
            Backend::NativeInspection => 0,
            Backend::OcctOpenNurbsExchange => 1,
            Backend::MeshOpenNurbsExchange => 2,
            Backend::PointCloudOpenNurbsExchange => 3,
            Backend::OcctEdit => 4,
            Backend::MeshOcctEdit => 5,
            Backend::NativeExplodeAdapters => 6,
            Backend::NativeCageAdapters => 7,
        },
        Err(CapabilityError::UnsupportedOperation(_)) => 100,
        Err(CapabilityError::UnsupportedRepresentation(_)) => 101,
        Err(CapabilityError::UnsupportedOptions) => 102,
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct DescriptorInput {
    pub stamp: Stamp,
    pub legacy_kind: u32,
    pub representation: u32,
    pub known: u32,
    pub values: u32,
    pub domains: [f64; 4],
    pub domain_count: u32,
    pub form: u32,
    pub facts_scope: u32,
    pub domain_scope: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct DescriptorSummary {
    pub status: u32,
    pub schema_version: u32,
    pub representation: u32,
    pub solid: u32,
    pub known: u32,
    pub values: u32,
    pub form: u32,
    pub domain_count: u32,
    pub domains: [f64; 4],
    pub facts_scope: u32,
    pub domain_scope: u32,
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_geometry_descriptor(i: DescriptorInput) -> DescriptorSummary {
    let error = |status| DescriptorSummary {
        status,
        ..Default::default()
    };
    if [
        i.stamp.document,
        i.stamp.object,
        i.stamp.generation,
        i.stamp.revision,
        i.stamp.epoch,
    ]
    .contains(&0)
    {
        return error(1);
    }
    let (kind, legacy) = match i.representation {
        0 => (RepresentationKind::Unknown, None),
        1 => (RepresentationKind::Point, Some(1)),
        2 => (RepresentationKind::Curve, Some(2)),
        3 => (RepresentationKind::Polycurve, Some(2)),
        4 => (RepresentationKind::TrimmedFace, Some(3)),
        5 => (RepresentationKind::Shell, Some(3)),
        6 => (RepresentationKind::Solid, Some(3)),
        7 => (RepresentationKind::Brep, Some(3)),
        8 => (RepresentationKind::Mesh, Some(4)),
        9 => (RepresentationKind::PointCloud, Some(5)),
        10 => (RepresentationKind::Instance, Some(9)),
        11 => (RepresentationKind::Group, Some(9)),
        _ => return error(2),
    };
    if legacy.is_some_and(|k| k != i.legacy_kind) || !(1..=9).contains(&i.legacy_kind) {
        return error(2);
    }
    let mask = match i.representation {
        2 => 7,
        4 => 31,
        _ => 0,
    };
    if i.known & !mask != 0 || i.values & !i.known != 0 || i.form > 4 {
        return error(4);
    }
    let expected_domains = match i.representation {
        2 => 1,
        4 => 2,
        _ => 0,
    };
    if i.domain_count != 0 && i.domain_count != expected_domains {
        return error(3);
    }
    if expected_domains == 0 && i.form != 0 {
        return error(4);
    }
    let facts_scope = match i.facts_scope {
        0 if i.known == 0 && i.form == 0 => Fact::Unknown,
        1 if expected_domains != 0 => Fact::Known(GeometryFactScope::SupportingGeometry),
        2 if expected_domains != 0 => Fact::Known(GeometryFactScope::RestrictedGeometry),
        _ => return error(5),
    };
    let domain_scope = match (i.domain_scope, i.domain_count, i.representation) {
        (0, 0, _) => Fact::Unknown,
        (1, 1 | 2, 2 | 4) => Fact::Known(DomainScope::SupportingGeometry),
        (2, 1, 2) => Fact::Known(DomainScope::EdgeRestriction),
        (3, 2, 4) => Fact::Known(DomainScope::FaceUvRestriction),
        _ => return error(5),
    };
    for d in 0..i.domain_count as usize {
        let [a, b] = [i.domains[2 * d], i.domains[2 * d + 1]];
        if !a.is_finite() || !b.is_finite() || a >= b || !(b - a).is_finite() {
            return error(3);
        }
    }
    let mut descriptor = GeometryDescriptor::uninspected(
        GeometryRef {
            stamp: i.stamp,
            subelement_ref: None,
            local_to_world: Fact::Unknown,
        },
        kind,
    );
    descriptor.topology.solid = match i.representation {
        6 => Fact::Known(true),
        1..=5 | 8 | 9 => Fact::Known(false),
        _ => Fact::Unknown,
    };
    // Native scalar observations become owned portable facts. An underlying
    // surface's periodic/closed directions never supply solid/manifold facts.
    let boolean = |bit: u32| {
        if i.known & (1u32 << bit) != 0u32 {
            Fact::Known(i.values & (1u32 << bit) != 0u32)
        } else {
            Fact::Unknown
        }
    };
    let curve = if i.representation == 2 {
        Some(CurveFacts {
            facts_scope: facts_scope.clone(),
            domain_scope: domain_scope.clone(),
            form: match i.form {
                1 => Fact::Known(CurveForm::Analytic),
                2 => Fact::Known(CurveForm::Nurbs),
                3 => Fact::Known(CurveForm::Bezier),
                4 => Fact::Known(CurveForm::Derived),
                _ => Fact::Unknown,
            },
            domain: if i.domain_count == 1 {
                Fact::Known([i.domains[0], i.domains[1]])
            } else {
                Fact::Unknown
            },
            closed: boolean(0),
            periodic: boolean(1),
            rational: boolean(2),
            ..Default::default()
        })
    } else {
        None
    };
    let surface = if i.representation == 4 {
        Some(SurfaceFacts {
            facts_scope,
            domain_scope,
            form: match i.form {
                1 => Fact::Known(SurfaceForm::Analytic),
                2 => Fact::Known(SurfaceForm::Nurbs),
                3 => Fact::Known(SurfaceForm::Bezier),
                4 => Fact::Known(SurfaceForm::Derived),
                _ => Fact::Unknown,
            },
            domains: if i.domain_count == 2 {
                Fact::Known([[i.domains[0], i.domains[1]], [i.domains[2], i.domains[3]]])
            } else {
                Fact::Unknown
            },
            closed: [boolean(0), boolean(1)],
            periodic: [boolean(2), boolean(3)],
            rational: boolean(4),
        })
    } else {
        None
    };
    if let Some(facts) = curve {
        descriptor.representation = RepresentationDescriptor::Curve(facts);
    }
    if let Some(facts) = surface {
        descriptor.representation = RepresentationDescriptor::TrimmedFace(facts);
    }
    let (form, domain_count, domains) = match &descriptor.representation {
        RepresentationDescriptor::Curve(c) => (
            match c.form {
                Fact::Known(CurveForm::Analytic) => 1,
                Fact::Known(CurveForm::Nurbs) => 2,
                Fact::Known(CurveForm::Bezier) => 3,
                Fact::Known(CurveForm::Derived) => 4,
                _ => 0,
            },
            if matches!(c.domain, Fact::Known(_)) {
                1
            } else {
                0
            },
            match c.domain {
                Fact::Known(d) => [d[0], d[1], 0., 0.],
                _ => [0.; 4],
            },
        ),
        RepresentationDescriptor::TrimmedFace(s) => (
            match s.form {
                Fact::Known(SurfaceForm::Analytic) => 1,
                Fact::Known(SurfaceForm::Nurbs) => 2,
                Fact::Known(SurfaceForm::Bezier) => 3,
                Fact::Known(SurfaceForm::Derived) => 4,
                _ => 0,
            },
            if matches!(s.domains, Fact::Known(_)) {
                2
            } else {
                0
            },
            match s.domains {
                Fact::Known(d) => [d[0][0], d[0][1], d[1][0], d[1][1]],
                _ => [0.; 4],
            },
        ),
        _ => (0, 0, [0.; 4]),
    };
    let (facts_scope, domain_scope) = match &descriptor.representation {
        RepresentationDescriptor::Curve(c) => (&c.facts_scope, &c.domain_scope),
        RepresentationDescriptor::TrimmedFace(s) => (&s.facts_scope, &s.domain_scope),
        _ => (&Fact::Unknown, &Fact::Unknown),
    };
    DescriptorSummary {
        status: 0,
        schema_version: descriptor.schema_version,
        representation: i.representation,
        solid: match descriptor.topology.solid {
            Fact::Unknown => 0,
            Fact::Known(false) => 1,
            Fact::Known(true) => 2,
        },
        known: i.known,
        values: i.values,
        form,
        domain_count,
        domains,
        facts_scope: match facts_scope {
            Fact::Unknown => 0,
            Fact::Known(GeometryFactScope::SupportingGeometry) => 1,
            Fact::Known(GeometryFactScope::RestrictedGeometry) => 2,
        },
        domain_scope: match domain_scope {
            Fact::Unknown => 0,
            Fact::Known(DomainScope::SupportingGeometry) => 1,
            Fact::Known(DomainScope::EdgeRestriction) => 2,
            Fact::Known(DomainScope::FaceUvRestriction) => 3,
        },
    }
}
