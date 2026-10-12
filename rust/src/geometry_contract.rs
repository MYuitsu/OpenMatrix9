// SPDX-License-Identifier: LGPL-2.1-or-later
//! Owned geometry facts and operation policy. Native kernels remain adapters.

use crate::core_objects::Stamp;
use crate::modeling_exchange::GeometryKind;
use std::collections::{BTreeMap, BTreeSet};

pub const SCHEMA_VERSION: u32 = 1;

/// Unknown means that this fact was not inspected, not false or empty.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Fact<T> {
    #[default]
    Unknown,
    Known(T),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RepresentationKind {
    Point,
    PointCloud,
    Curve,
    Polycurve,
    Surface,
    TrimmedFace,
    Shell,
    Solid,
    Brep,
    LightweightExtrusion,
    Mesh,
    Instance,
    Group,
    Annotation,
    SubD,
    Mixed,
    Retained,
    Unknown,
}

impl RepresentationKind {
    /// A legacy CAD BRep tag contains no face/shell/solid classification.
    pub fn from_legacy(kind: GeometryKind) -> Self {
        match kind {
            GeometryKind::CadPoint => Self::Point,
            GeometryKind::CadCurve => Self::Curve,
            GeometryKind::CadBrep => Self::Brep,
            GeometryKind::Mesh => Self::Mesh,
            GeometryKind::PointCloud => Self::PointCloud,
            GeometryKind::SubD => Self::SubD,
            GeometryKind::Mixed => Self::Mixed,
            GeometryKind::Retained => Self::Retained,
            GeometryKind::Unknown => Self::Unknown,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Orientation {
    Forward,
    Reversed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CurveForm {
    Analytic,
    Nurbs,
    Polyline,
    Bezier,
    Derived,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurfaceForm {
    Analytic,
    Nurbs,
    Bezier,
    Derived,
}

/// The domain can restrict a support without changing that support's facts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DomainScope {
    SupportingGeometry,
    EdgeRestriction,
    FaceUvRestriction,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GeometryFactScope {
    SupportingGeometry,
    RestrictedGeometry,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CurveFacts {
    pub form: Fact<CurveForm>,
    pub facts_scope: Fact<GeometryFactScope>,
    pub domain: Fact<[f64; 2]>,
    pub domain_scope: Fact<DomainScope>,
    pub orientation: Fact<Orientation>,
    pub closed: Fact<bool>,
    pub periodic: Fact<bool>,
    pub rational: Fact<bool>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SurfaceFacts {
    pub form: Fact<SurfaceForm>,
    pub facts_scope: Fact<GeometryFactScope>,
    pub domains: Fact<[[f64; 2]; 2]>,
    pub domain_scope: Fact<DomainScope>,
    pub closed: [Fact<bool>; 2],
    pub periodic: [Fact<bool>; 2],
    pub rational: Fact<bool>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RepresentationDescriptor {
    Uninspected(RepresentationKind),
    Curve(CurveFacts),
    Surface(SurfaceFacts),
    TrimmedFace(SurfaceFacts),
}

impl RepresentationDescriptor {
    pub fn kind(&self) -> RepresentationKind {
        match self {
            Self::Uninspected(kind) => *kind,
            Self::Curve(_) => RepresentationKind::Curve,
            Self::Surface(_) => RepresentationKind::Surface,
            Self::TrimmedFace(_) => RepresentationKind::TrimmedFace,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GeometryRef {
    /// Captured from the existing session registry; adapters must revalidate it.
    pub stamp: Stamp,
    /// Revision-scoped host selector, never a durable subelement identity.
    pub subelement_ref: Option<String>,
    /// Provenance only; no transform is applied by this descriptor module.
    pub local_to_world: Fact<[[f64; 4]; 4]>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoordinateSpace {
    Local,
    World,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoopRole {
    Outer,
    Hole,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SolidOrientation {
    Outward,
    Inward,
    Unoriented,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EdgeUse {
    /// Snapshot-local topology identity, shared by multiple seam occurrences.
    pub edge_id: u64,
    pub orientation: Orientation,
    pub pcurve_id: Fact<u64>,
    pub seam: Fact<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrimLoop {
    pub role: LoopRole,
    pub edge_uses: Vec<EdgeUse>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FaceDescriptor {
    pub surface_id: Fact<u64>,
    pub orientation: Fact<Orientation>,
    pub loops: Fact<Vec<TrimLoop>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoundaryComponent {
    pub edge_uses: Vec<EdgeUse>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TopologyDescriptor {
    pub closed: Fact<bool>,
    pub manifold: Fact<bool>,
    /// Supplied only by native topology inspection, never inferred from a surface.
    pub solid: Fact<bool>,
    pub solid_orientation: Fact<SolidOrientation>,
    pub faces: Fact<Vec<FaceDescriptor>>,
    pub free_boundary_components: Fact<Vec<BoundaryComponent>>,
}

impl TopologyDescriptor {
    pub fn uninspected() -> Self {
        Self {
            closed: Fact::Unknown,
            manifold: Fact::Unknown,
            solid: Fact::Unknown,
            solid_orientation: Fact::Unknown,
            faces: Fact::Unknown,
            free_boundary_components: Fact::Unknown,
        }
    }
    /// Count recorded loop occurrences without changing order or deduplicating seams.
    pub fn edge_use_count(&self) -> Fact<usize> {
        let Fact::Known(faces) = &self.faces else {
            return Fact::Unknown;
        };
        let mut count = 0usize;
        for face in faces {
            let Fact::Known(loops) = &face.loops else {
                return Fact::Unknown;
            };
            for trim_loop in loops {
                let Some(next) = count.checked_add(trim_loop.edge_uses.len()) else {
                    return Fact::Unknown;
                };
                count = next;
            }
        }
        Fact::Known(count)
    }
    /// Count distinct identities in recorded loop uses, not connected components.
    pub fn edge_identity_count(&self) -> Fact<usize> {
        let Fact::Known(faces) = &self.faces else {
            return Fact::Unknown;
        };
        let mut identities = BTreeSet::new();
        for face in faces {
            let Fact::Known(loops) = &face.loops else {
                return Fact::Unknown;
            };
            for trim_loop in loops {
                identities.extend(trim_loop.edge_uses.iter().map(|edge| edge.edge_id));
            }
        }
        Fact::Known(identities.len())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GeometryDescriptor {
    pub schema_version: u32,
    pub reference: GeometryRef,
    pub unit_context_revision: Fact<u64>,
    pub coordinates: Fact<CoordinateSpace>,
    pub representation: RepresentationDescriptor,
    pub topology: TopologyDescriptor,
    pub source_attributes: Fact<BTreeMap<String, String>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DescriptorError {
    UnsupportedSchema(u32),
}

impl GeometryDescriptor {
    /// Owns facts only. It neither resolves a native object nor certifies its validity.
    pub fn uninspected(reference: GeometryRef, kind: RepresentationKind) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            reference,
            unit_context_revision: Fact::Unknown,
            coordinates: Fact::Unknown,
            representation: RepresentationDescriptor::Uninspected(kind),
            topology: TopologyDescriptor::uninspected(),
            source_attributes: Fact::Unknown,
        }
    }
    /// Schema compatibility only. Geometry, basis and topology validation remain separate.
    pub fn validate_schema(&self) -> Result<(), DescriptorError> {
        if self.schema_version == SCHEMA_VERSION {
            Ok(())
        } else {
            Err(DescriptorError::UnsupportedSchema(self.schema_version))
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Preservation {
    Preserved,
    Changed,
    NotApplicable,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParameterMap {
    /// An adapter-provided affine map with independent evidence, not a solved map.
    pub source_domain: [f64; 2],
    pub target_domain: [f64; 2],
    pub scale: f64,
    pub offset: f64,
    pub evidence_id: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParameterMapError {
    Uninspected,
    NonFinite,
    InvalidDomain,
    MissingEvidence,
    InconsistentDomain,
    OutsideDomain,
    UnsupportedSchema(u32),
}

impl ParameterMap {
    /// Validates the supplied affine map, not the native conversion that produced it.
    /// Exact endpoint agreement is required; this module does not invent a tolerance.
    pub fn validate(&self) -> Result<(), ParameterMapError> {
        if !self
            .source_domain
            .iter()
            .chain(self.target_domain.iter())
            .chain([self.scale, self.offset].iter())
            .all(|value| value.is_finite())
        {
            return Err(ParameterMapError::NonFinite);
        }
        if self.source_domain[0] >= self.source_domain[1]
            || self.target_domain[0] >= self.target_domain[1]
            || self.scale == 0.0
        {
            return Err(ParameterMapError::InvalidDomain);
        }
        if self.evidence_id.trim().is_empty() {
            return Err(ParameterMapError::MissingEvidence);
        }
        let first = self.scale * self.source_domain[0] + self.offset;
        let last = self.scale * self.source_domain[1] + self.offset;
        if !first.is_finite() || !last.is_finite() {
            return Err(ParameterMapError::NonFinite);
        }
        if [first.min(last), first.max(last)] != self.target_domain {
            return Err(ParameterMapError::InconsistentDomain);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeviationMeasurement {
    Sampled { sample_count: usize },
    GlobalBound,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DeviationReport {
    pub distance: f64,
    pub measurement: DeviationMeasurement,
    pub evidence_id: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GeometryField {
    Basis,
    Domain,
    Direction,
    Seam,
    Trims,
    Adjacency,
    Attributes,
    Units,
    Placement,
    Representation,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConversionReport {
    pub schema_version: u32,
    pub from: RepresentationKind,
    pub to: RepresentationKind,
    pub locus: Fact<Preservation>,
    pub parameterization: Fact<Preservation>,
    pub topology: Fact<Preservation>,
    pub parameter_map: Fact<ParameterMap>,
    pub preserved_fields: Vec<GeometryField>,
    pub dropped_fields: Vec<GeometryField>,
    pub deviation: Fact<DeviationReport>,
    pub warnings: Vec<String>,
}

impl ConversionReport {
    /// Operation permission and equal representation tags supply no preservation evidence.
    pub fn uninspected(from: RepresentationKind, to: RepresentationKind) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            from,
            to,
            locus: Fact::Unknown,
            parameterization: Fact::Unknown,
            topology: Fact::Unknown,
            parameter_map: Fact::Unknown,
            preserved_fields: Vec::new(),
            dropped_fields: Vec::new(),
            deviation: Fact::Unknown,
            warnings: Vec::new(),
        }
    }
    /// Reusing the raw parameter requires an explicit parameterization claim.
    /// A nonidentity map, preserved locus or preserved topology does not suffice.
    pub fn source_parameter_is_reusable(&self) -> bool {
        self.schema_version == SCHEMA_VERSION
            && self.parameterization == Fact::Known(Preservation::Preserved)
            && match &self.parameter_map {
                Fact::Unknown => true,
                Fact::Known(map) => {
                    map.validate().is_ok()
                        && map.scale == 1.0
                        && map.offset == 0.0
                        && map.source_domain == map.target_domain
                }
            }
    }
    /// Applies a structurally valid, externally evidenced affine map. This does
    /// not prove a geometric correspondence or certify native query success.
    pub fn map_source_parameter(&self, source: f64) -> Result<f64, ParameterMapError> {
        if self.schema_version != SCHEMA_VERSION {
            return Err(ParameterMapError::UnsupportedSchema(self.schema_version));
        }
        let Fact::Known(map) = &self.parameter_map else {
            return Err(ParameterMapError::Uninspected);
        };
        map.validate()?;
        if !source.is_finite() {
            return Err(ParameterMapError::NonFinite);
        }
        if source < map.source_domain[0] || source > map.source_domain[1] {
            return Err(ParameterMapError::OutsideDomain);
        }
        let target = map.scale * source + map.offset;
        if !target.is_finite() {
            return Err(ParameterMapError::NonFinite);
        }
        if target < map.target_domain[0] || target > map.target_domain[1] {
            return Err(ParameterMapError::OutsideDomain);
        }
        Ok(target)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperationId {
    Inspect,
    ImportWorkingV5,
    ExportWorkingV5,
    Boolean,
    EditCurve,
    ConstructCurve,
    ConstructSolid,
    NativeCage,
    NativeBuilder,
    EditSurface,
    NativeEdit,
    Unrecognized(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConversionPolicy {
    ExistingConverters,
    PreserveRepresentation,
    Tessellate,
    Heal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreservationPolicy {
    ReportObservedFacts,
    RequireLocus,
    RequireParameterization,
    RequireTopology,
    RequireAll,
}

#[derive(Clone, Debug, PartialEq)]
pub enum OperationOptions {
    InspectDescriptor,
    NativeEdit {kind:u32,flags:u32,mode:u32},
    NativeSurface(crate::native_surface_capability::Request),
    NativeRebuild(crate::native_rebuild_capability::Request),
    NativeCurve(crate::native_curve_capability::Request),
    NativeSolid(crate::native_solid_capability::Request),
    NativeCage(crate::native_cage_capability::Request),
    NativeBuilder(crate::native_builder_capability::Request),
    WorkingExchange {
        archive_version: u32,
        conversion: ConversionPolicy,
        preservation: PreservationPolicy,
    },
    Unrecognized(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Backend {
    NativeInspection,
    OcctOpenNurbsExchange,
    MeshOpenNurbsExchange,
    PointCloudOpenNurbsExchange,
    OcctEdit,
    MeshOcctEdit,
    NativeExplodeAdapters,
    NativeCageAdapters,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperationConstraint {
    ReadOnly,
    ValidateSourceStamp,
    ExistingNativeConverters,
    ReportConversionFacts,
    ValidateNativeInput,
    PrivateOwnedInput,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OperationCapability {
    pub schema_version: u32,
    pub operation: OperationId,
    pub input: RepresentationKind,
    pub options: OperationOptions,
    pub backend: Backend,
    pub constraints: &'static [OperationConstraint],
    pub evidence_id: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CapabilityError {
    UnsupportedOperation(OperationId),
    UnsupportedRepresentation(RepresentationKind),
    UnsupportedOptions,
}

/// Versioned policy for descriptor reads and existing independent working exchange.
/// This is not a kernel dispatcher or a command-enablement claim. The adapter must
/// validate its actual converter branch and native geometry, then report measured
/// conversion facts separately. No solver or preservation option is implied.
pub fn operation_capability(
    operation: OperationId,
    input: RepresentationKind,
    options: OperationOptions,
) -> Result<OperationCapability, CapabilityError> {
    let (backend, constraints, evidence_id) = match operation {
        OperationId::NativeBuilder => {
            let OperationOptions::NativeBuilder(ref request)=options else{return Err(CapabilityError::UnsupportedOperation(operation));};
            crate::native_builder_capability::policy(operation,input,request)?
        }
        OperationId::NativeCage => {
            let OperationOptions::NativeCage(ref request)=options else{return Err(CapabilityError::UnsupportedOperation(operation));};
            crate::native_cage_capability::policy(operation,input,request)?
        }
        OperationId::ConstructSolid => {
            let OperationOptions::NativeSolid(ref request)=options else{return Err(CapabilityError::UnsupportedOperation(operation));};
            crate::native_solid_capability::policy(operation,input,request)?
        }
        OperationId::ConstructCurve => {
            let OperationOptions::NativeCurve(ref request)=options else{return Err(CapabilityError::UnsupportedOperation(operation));};
            crate::native_curve_capability::policy(operation,input,request)?
        }
        OperationId::EditCurve => {
            let OperationOptions::NativeRebuild(ref request)=options else{return Err(CapabilityError::UnsupportedOperation(operation));};
            crate::native_rebuild_capability::policy(operation,input,request)?
        }
        OperationId::EditSurface => {
            let OperationOptions::NativeSurface(ref request)=options else{return Err(CapabilityError::UnsupportedOperation(operation));};
            crate::native_surface_capability::policy(operation,input,request)?
        }
        OperationId::NativeEdit | OperationId::Boolean => {
            let OperationOptions::NativeEdit{kind,flags,mode}=options else{return Err(CapabilityError::UnsupportedOperation(operation));};
            crate::native_edit_capability::policy(operation,input,kind,flags,mode)?
        }
        OperationId::Inspect => {
            if options != OperationOptions::InspectDescriptor {
                return Err(CapabilityError::UnsupportedOptions);
            }
            (
                Backend::NativeInspection,
                &[
                    OperationConstraint::ReadOnly,
                    OperationConstraint::ValidateSourceStamp,
                ][..],
                "RCORE-01.policy.inspect.v1",
            )
        }
        OperationId::ImportWorkingV5 | OperationId::ExportWorkingV5 => {
            if options
                != (OperationOptions::WorkingExchange {
                    archive_version: 5,
                    conversion: ConversionPolicy::ExistingConverters,
                    preservation: PreservationPolicy::ReportObservedFacts,
                })
            {
                return Err(CapabilityError::UnsupportedOptions);
            }
            let backend = match input {
                RepresentationKind::Point
                | RepresentationKind::Curve
                | RepresentationKind::Polycurve
                | RepresentationKind::Brep
                | RepresentationKind::TrimmedFace
                | RepresentationKind::Shell
                | RepresentationKind::Solid => Backend::OcctOpenNurbsExchange,
                RepresentationKind::Mesh => Backend::MeshOpenNurbsExchange,
                RepresentationKind::PointCloud => Backend::PointCloudOpenNurbsExchange,
                _ => return Err(CapabilityError::UnsupportedRepresentation(input)),
            };
            (
                backend,
                &[
                    OperationConstraint::ValidateSourceStamp,
                    OperationConstraint::ExistingNativeConverters,
                    OperationConstraint::ReportConversionFacts,
                ][..],
                "working-exchange.phase1.v5",
            )
        }
        _ => return Err(CapabilityError::UnsupportedOperation(operation)),
    };
    Ok(OperationCapability {
        schema_version: SCHEMA_VERSION,
        operation,
        input,
        options,
        backend,
        constraints,
        evidence_id,
    })
}
