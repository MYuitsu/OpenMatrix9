//! Public, host-independent OpenMatrix9 plugin SDK.
//!
//! Native plugins are trusted code, not a security sandbox. Build each native
//! target separately and package the resulting libraries together in `.om9p`.
//! All document coordinates and translation components use millimetres. Object
//! IDs are opaque strings; document ownership, permissions, transactions and
//! Undo remain with the host. A host invocation is synchronous and cannot be
//! retained or moved to another thread. Each request creates a fresh plugin
//! instance; persistent state belongs in explicit host-supported storage.

pub mod abi;

use std::{collections::BTreeMap, fmt, marker::PhantomData, rc::Rc};

use serde::{Deserialize, Serialize, de::DeserializeOwned};
pub use serde_json::Value;

pub const SDK_VERSION: u32 = 1;
pub type Result<T> = std::result::Result<T, SdkError>;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SdkError {
    pub code: String,
    pub message: String,
}

impl SdkError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for SdkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for SdkError {}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommandDescriptor {
    pub id: String,
    pub title: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginDescriptor {
    pub id: String,
    pub name: String,
    pub version: String,
    pub sdk_version: u32,
    pub commands: Vec<CommandDescriptor>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum PluginRequest {
    Describe,
    Execute { command: String, arguments: Value },
}

/// Geometry data only: no native kernel handles or application-owned pointers.
/// Circle `x` and `y` are unit, perpendicular basis vectors; radius is in mm.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Geometry {
    Point {
        point: [f64; 3],
    },
    Line {
        start: [f64; 3],
        end: [f64; 3],
    },
    Polyline {
        points: Vec<[f64; 3]>,
    },
    Circle {
        center: [f64; 3],
        x: [f64; 3],
        y: [f64; 3],
        radius: f64,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "method", rename_all = "snake_case")]
pub enum HostRequest {
    Capabilities,
    DocumentInfo,
    ObjectsList,
    SelectionGet,
    SelectionSet {
        ids: Vec<String>,
    },
    SelectionAdd {
        ids: Vec<String>,
    },
    SelectionRemove {
        ids: Vec<String>,
    },
    SelectionToggle {
        ids: Vec<String>,
    },
    LayersList,
    /// Object-local mm coordinates, before the ObjectInfo placement matrix.
    ObjectGeometry {
        id: String,
    },
    ObjectAdd {
        label: String,
        geometry: Geometry,
    },
    ObjectDelete {
        id: String,
    },
    /// Row-major matrix multiplying column vectors; translation is column 3.
    /// Applies a world-space delta: new placement = matrix * old placement.
    /// The host validates finite affine values and geometry applicability.
    ObjectTransform {
        id: String,
        matrix: [[f64; 4]; 4],
    },
    /// Display unit and the document's own tolerance policy, if it states one.
    DocumentUnits,
    /// `"2.5"` uses the document display unit; `"2.5 mm"` / `"0.1 in"` carry their own.
    ParseLength {
        text: String,
    },
    /// `"90"`/`"90deg"`/`"90°"` are degrees; `"1.5707963rad"` converts explicitly.
    ParseAngle {
        text: String,
    },
    /// `x,y` or `x,y,z` in the document display unit; z defaults to zero.
    ParseCoordinates {
        text: String,
    },
    /// Bounded, filtered object query.
    ObjectsQuery {
        #[serde(default)]
        query: ObjectQuery,
    },
    ObjectAttributes {
        id: String,
    },
    /// One attribute edit; the host validates the layer and transaction rules.
    ObjectSetAttributes {
        id: String,
        patch: AttributesPatch,
    },
    ObjectRename {
        id: String,
        label: String,
    },
    /// Create one layer and optionally make it current.
    LayerCreate {
        name: String,
        color: [u8; 4],
        make_current: bool,
    },
    LayerUpdate {
        id: String,
        update: LayerUpdate,
    },
    /// `None` clears the current layer.
    SetCurrentLayer {
        id: Option<String>,
    },
    /// Move objects to a layer; the object colour is cleared so the layer colour applies.
    AssignToLayer {
        ids: Vec<String>,
        layer: String,
    },
    /// Validate one typed parameter against its declared domain.
    ValidateParameter {
        spec: ParameterSpec,
        value: Value,
    },
    /// Fill defaults, check required/unknown names and validate every typed value.
    ApplyParameters {
        schema: Vec<ParameterSpec>,
        #[serde(default)]
        supplied: BTreeMap<String, Value>,
    },
    /// One progress or warning report for the command's event log.
    Message {
        level: MessageLevel,
        text: String,
    },
    /// Curve identity and native parameter domain.
    CurveInfo {
        id: String,
    },
    /// Evaluate a curve at native parameters, with optional derivatives.
    CurveEvaluate {
        id: String,
        parameters: Vec<f64>,
        #[serde(default)]
        order: u8,
    },
    /// Defining data: NURBS poles/weights/knots, arc angle domain, seam.
    CurveDefinition {
        id: String,
    },
    /// Closedness and planarity under an explicit or the document tolerance.
    CurveSemantics {
        id: String,
        #[serde(default)]
        tolerance_mm: Option<f64>,
    },
    /// Add an arc through start, through and end points (object-local mm).
    ObjectAddArc {
        label: String,
        start: [f64; 3],
        through: [f64; 3],
        end: [f64; 3],
    },
    /// Add an interpolated NURBS curve through points (odd degree 1..=11).
    ObjectAddInterpolatedCurve {
        label: String,
        points: Vec<[f64; 3]>,
        degree: u32,
        #[serde(default)]
        mode: KnotMode,
    },
    /// Add a control-point NURBS curve; the domain is derived from the knots.
    ObjectAddNurbsCurve {
        label: String,
        degree: u32,
        poles: Vec<[f64; 3]>,
        weights: Vec<f64>,
        knots: Vec<f64>,
    },
    /// Add a NURBS ellipse on the orthonormal frame (x, y).
    ObjectAddEllipse {
        label: String,
        origin: [f64; 3],
        x: [f64; 3],
        y: [f64; 3],
        radius_x: f64,
        radius_y: f64,
    },
    /// Add a closed rectangular polycurve on the orthonormal frame (x, y).
    ObjectAddRectangle {
        label: String,
        origin: [f64; 3],
        x: [f64; 3],
        y: [f64; 3],
        width: f64,
        height: f64,
    },
    /// Add a closed polygon through the points (object-local mm).
    ObjectAddPolygon {
        label: String,
        points: Vec<[f64; 3]>,
    },
    /// The project path and dirty state, read-only.
    ProjectState,
    /// The change batch recorded for the last committed change (plugin command or host\n/// Undo/Redo).
    LastChange,
    /// Object-local bounding box of one object.
    ObjectBounds {
        id: String,
    },
    /// Sampled length of one curve.
    MeasureCurveLength {
        id: String,
    },
    /// Radius of one circle curve.
    MeasureCurveRadius {
        id: String,
    },
    /// Count objects by geometry kind.
    ObjectSummary,
    /// Triangulated area of one mesh.
    MeasureMeshArea {
        id: String,
    },
    /// Document groups and their members.
    GroupsList,
    /// Block definitions and their member counts.
    DefinitionsList,
    /// Clear the current layer tag.
    LayerClearCurrent,
    /// One object's affine placement matrix (row-major 4x4).
    ObjectPlacement {
        id: String,
    },
    /// One object's identity/placement record.
    ObjectInfoGet {
        id: String,
    },
    /// Create one group from existing member ids.
    GroupCreate {
        name: String,
        members: Vec<String>,
    },
    /// Delete one group record (objects stay).
    GroupDelete {
        id: String,
    },
    /// One layer record by id.
    LayerInfoGet {
        id: String,
    },
    /// Whether one object can be picked under the effective visibility/selectability rules.
    IsObjectPickable {
        id: String,
    },
    /// Retained import source proof for one object, or `null`.
    ObjectProvenance {
        id: String,
    },
    /// Set the document tolerance policy.
    SetTolerance {
        absolute_mm: f64,
        relative: f64,
        angular_rad: f64,
    },
    /// Report one measurement label through the command event log.
    MeasureLabel {
        text: String,
    },
    /// Layer order (bottom to top) as ids.
    LayerOrder,
    /// Segments of a curve (one for a native curve, many for a polycurve).
    CurveSegments {
        id: String,
    },
    /// Evaluate one segment of a curve at native parameters.
    CurveEvaluateSegment {
        id: String,
        index: u32,
        parameters: Vec<f64>,
        #[serde(default)]
        order: u8,
    },
    /// Add a polygon mesh from vertices and faces (indices into vertices).
    ObjectAddMesh {
        label: String,
        vertices: Vec<[f64; 3]>,
        faces: Vec<Vec<u32>>,
    },
    /// Add a full circle on an orthonormal frame.
    ObjectAddCircleFrame {
        label: String,
        center: [f64; 3],
        x: [f64; 3],
        y: [f64; 3],
        radius: f64,
    },
    /// Set one object's affine placement matrix (row-major 4x4).
    ObjectSetPlacement {
        id: String,
        matrix: [[f64; 4]; 4],
    },
    /// One object's extension slots.
    ObjectExtensions {
        id: String,
    },
    /// Set one extension slot (validated JSON object text).
    ObjectSetExtension {
        id: String,
        slot: String,
        version: u32,
        payload: String,
    },
    /// Clear one extension slot.
    ObjectClearExtension {
        id: String,
        slot: String,
    },
    /// Request one atomic save after the command commits.
    SaveProject,
    /// Request a host Save As (path dialog) after the command commits.
    SaveProjectAs,
    /// Distance between two world points, in millimetres.
    MeasureDistance {
        first: [f64; 3],
        second: [f64; 3],
    },
    /// Undirected angle between two directed lines, in degrees.
    MeasureAngle {
        from_a: [f64; 3],
        to_a: [f64; 3],
        from_b: [f64; 3],
        to_b: [f64; 3],
    },
}

/// How a plugin report reaches the command feedback surface.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MessageLevel {
    /// Ordinary progress for the current command.
    Progress,
    /// A nonfatal warning; the command may still commit.
    Warning,
}

/// One bounded plugin report, surfaced by the host in the same command event log
/// the Command line and observers use.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginMessage {
    pub level: MessageLevel,
    pub text: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocumentInfo {
    pub id: String,
    pub name: String,
    pub revision: u64,
    pub current_layer: Option<String>,
    /// Document display-unit symbol. SDK v1 geometry remains in canonical mm;
    /// this value must not be used to rescale coordinates returned by the SDK.
    pub unit: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ObjectInfo {
    pub id: String,
    pub label: String,
    pub geometry_kind: String,
    pub layer: Option<String>,
    pub visible: bool,
    pub selectable: bool,
    pub editable: bool,
    pub placement: [[f64; 4]; 4],
    /// Object generation: it changes when the identity is replaced, not when the
    /// geometry or attributes are edited.
    pub generation: u64,
    /// Geometry revision of the handle; it changes only with geometry edits.
    pub geometry_revision: u64,
    /// Attributes revision; it changes with attributes or label edits.
    pub attributes_revision: u64,
}

/// One object's stored and effective attribute values.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ObjectAttributes {
    pub id: String,
    pub label: String,
    pub layer: Option<String>,
    /// Explicit object colour, or `None` so the layer colour applies.
    pub color: Option<[u8; 4]>,
    /// Stored flags, before the layer chain is applied.
    pub visible: bool,
    pub hidden: bool,
    pub selectable: bool,
    pub locked: bool,
    /// Effective flags after the whole layer chain.
    pub effective_visible: bool,
    pub effective_selectable: bool,
    pub effective_editable: bool,
    pub generation: u64,
    pub geometry_revision: u64,
    pub attributes_revision: u64,
}

/// One attribute edit. Absent fields stay unchanged; an empty `color` string
/// clears the object colour so the layer colour applies again.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct AttributesPatch {
    /// `"#rrggbb"` or `"#rrggbbaa"`; an empty string clears the object colour.
    pub color: Option<String>,
    pub visible: Option<bool>,
    pub hidden: Option<bool>,
    pub selectable: Option<bool>,
    pub locked: Option<bool>,
}

/// A bounded object query. `layer`, `geometry_kind` and `name_contains` are
/// optional filters; `only_selected` restricts the result to the command's
/// starting selection.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct ObjectQuery {
    pub offset: u64,
    /// Zero selects the host default (256); the host caps one page at 1024.
    pub limit: u64,
    pub layer: Option<String>,
    /// Exact `ObjectInfo::geometry_kind` value, e.g. `"Point"` or `"Curve"`.
    pub geometry_kind: Option<String>,
    /// Case-sensitive substring of the object label.
    pub name_contains: Option<String>,
    pub only_selected: bool,
    /// When false, objects whose stored `visible` flag is false or whose
    /// `hidden` flag is true are omitted. Layer visibility is reported per
    /// object, not used as a filter here.
    pub include_hidden: bool,
}

impl Default for ObjectQuery {
    fn default() -> Self {
        Self {
            offset: 0,
            limit: 0,
            layer: None,
            geometry_kind: None,
            name_contains: None,
            only_selected: false,
            include_hidden: true,
        }
    }
}

/// One page of a query plus the total number of matching objects.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ObjectPage {
    pub total: u64,
    pub offset: u64,
    pub objects: Vec<ObjectInfo>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LayerInfo {
    pub id: String,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub selectable: bool,
    pub current: bool,
    /// Layer colour as `[r, g, b, a]`.
    pub color: [u8; 4],
}

/// One layer edit. Absent fields stay unchanged. A rename and a flag/colour change
/// are one layer record, so they travel in one staged update.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct LayerUpdate {
    pub name: Option<String>,
    pub color: Option<[u8; 4]>,
    pub visible: Option<bool>,
    pub selectable: Option<bool>,
    pub locked: Option<bool>,
}

/// The document's own display unit and tolerance policy. Geometry stays canonical mm.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DocumentUnits {
    /// Display-unit symbol; never use it to rescale coordinates returned by the SDK.
    pub unit: String,
    /// Millimetres in one display unit, for user-facing presentation only.
    pub millimeters_per_unit: f64,
    /// `None` means the document states no tolerance. The host never invents a
    /// default and no plugin may treat a missing policy as the OM9 template.
    pub tolerance: Option<DocumentTolerance>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DocumentTolerance {
    /// Absolute length tolerance in mm; zero is never a valid document policy.
    pub absolute_mm: f64,
    /// Relative tolerance as a fraction (not a percentage).
    pub relative: f64,
    /// Angular tolerance in radians. It is stored in radians and is not a degree value.
    pub angular_rad: f64,
}

/// A parsed length. A bare `f64` never crosses this boundary as a length.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LengthValue {
    pub millimeters: f64,
}

/// A parsed angle. The degree value is canonical; radians are an explicit conversion.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AngleValue {
    pub degrees: f64,
    pub radians: f64,
}

/// A parsed coordinate tuple in canonical millimetres, before any CPlane frame.
/// Resolving a local frame to world space stays with the host's view/CPlane owner.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Coordinates {
    pub millimeters: [f64; 3],
}

/// Knot spacing rule for an interpolated curve.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum KnotMode {
    /// Spacing follows the distance between the input points.
    #[default]
    ChordLength,
    /// Spacing follows the point index.
    Uniform,
}
/// One curve's identity and parameter domain, in object-local millimetres.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CurveInfo {
    pub id: String,
    /// `"line"`, `"circle"`, `"nurbs"` or `"polycurve"`.
    pub kind: String,
    /// Native parameter domain when one parameter is meaningful; `None` for a polycurve.
    pub domain: Option<[f64; 2]>,
    /// NURBS degree, when this is NURBS.
    pub degree: Option<u64>,
    /// NURBS pole count, when this is NURBS.
    pub poles: Option<u64>,
    /// NURBS periodic flag, when this is NURBS.
    pub periodic: Option<bool>,
    pub reversed: bool,
    /// Polycurve segment count; zero for a single native curve.
    pub segments: u64,
}

/// One named extension slot of an object: version plus the verbatim JSON object text.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ObjectExtension {
    pub version: u32,
    pub payload: String,
}
/// One recorded change: its kind and the object/layer/group/feature id it names.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChangeEntry {
    /// `entity_added`, `entity_removed`, `entity_modified`, `layer_created`,
    /// `layer_modified`, `layer_removed`, `current_layer_changed`,
    /// `layer_order_changed`, `extension_changed`, `group_*` or `feature_*`.
    pub kind: String,
    pub id: Option<String>,
}

/// The change batch recorded for the last committed change (plugin command or host\n/// Undo/Redo).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectChange {
    pub from_revision: u64,
    pub to_revision: u64,
    /// `edit`, `undo`, `redo`, `replace` or `restore`.
    pub cause: String,
    pub changes: Vec<ChangeEntry>,
}
/// The host project's file identity and dirty state.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectInfo {
    /// The last opened or saved project path; `None` for an untitled project.
    pub path: Option<String>,
    /// Whether the document differs from the last saved/opened fingerprint.
    pub dirty: bool,
}
/// A measured distance in canonical millimetres.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct MeasuredDistance {
    pub millimeters: f64,
}

/// A measured undirected angle in degrees.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct MeasuredAngle {
    pub degrees: f64,
}
/// Retained import source proof for one object.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ObjectProvenance {
    /// Original source name (import file name), never inferred from geometry.
    pub source_name: String,
    /// SHA-256 of the embedded source, lowercase hex.
    pub source_sha256: String,
    pub archive_version: u32,
    /// `preserve` or `geometry`, when the import recorded a mode.
    pub import_mode: Option<String>,
    /// Native record kind (`geometry`, `layer`, `other:<type>`, …).
    pub kind: String,
    pub original_uuid: String,
    pub original_index: Option<i32>,
    pub original_family: String,
    /// `unchanged`, `edited`, `deleted` or `retained_only`.
    pub disposition: String,
}

/// One block definition: its label, member count and revision.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BlockDefinitionInfo {
    pub id: String,
    pub label: String,
    pub members: u64,
    pub revision: u64,
}

/// One document group and its member object ids.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GroupInfo {
    pub id: String,
    pub name: String,
    pub members: Vec<String>,
    pub revision: u64,
}

/// A measured area in square millimetres.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct MeasuredArea {
    pub square_millimeters: f64,
}

/// Document object counts by geometry kind.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ObjectSummary {
    pub total: u64,
    pub by_kind: std::collections::BTreeMap<String, u64>,
}
/// Object-local bounding box and its derived size/center, in millimetres.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ObjectBounds {
    pub min: [f64; 3],
    pub max: [f64; 3],
    pub size: [f64; 3],
    pub center: [f64; 3],
}
/// One segment of a curve: a native curve is one segment, a polycurve is many.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CurveSegment {
    pub kind: String,
    pub domain: Option<[f64; 2]>,
    pub angle_domain: Option<[f64; 2]>,
    pub reversed: bool,
}
/// The defining data of one curve. Control points, weights and knots are present only
/// for NURBS; an arc additionally reports its angle domain.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CurveDefinition {
    pub id: String,
    pub kind: String,
    pub domain: Option<[f64; 2]>,
    /// Native angle domain of an arc, when the circle carries one.
    pub angle_domain: Option<[f64; 2]>,
    pub degree: Option<u64>,
    pub poles: Option<Vec<[f64; 3]>>,
    pub weights: Option<Vec<f64>>,
    pub knots: Option<Vec<f64>>,
    pub periodic: Option<bool>,
    pub reversed: bool,
    pub seam: Option<f64>,
    pub segment_count: u64,
}

/// Closedness and planarity of one curve measured under an explicit tolerance.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CurveSemantics {
    pub id: String,
    /// `"line"`, `"circle"`, `"nurbs"`, `"polyline"` or `"polycurve"`.
    pub kind: String,
    pub closed: bool,
    pub planar: bool,
    /// Object-local millimetres, first and last sampled point.
    pub endpoints: [[f64; 3]; 2],
    /// The tolerance the measurement used; never an invented document default.
    pub tolerance_mm: f64,
}
/// One evaluated curve sample. Coordinates are object-local millimetres, before the
/// object placement.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CurveSample {
    /// The native parameter this sample evaluated.
    pub parameter: f64,
    pub point: [f64; 3],
    /// D1..D{order}, object-local millimetres per parameter unit.
    pub derivatives: Vec<[f64; 3]>,
}
/// Numeric or choice dimension of one builder/command parameter.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Dimension {
    Length,
    Angle,
    Ratio,
    Count,
}

/// The accepted domain of one parameter. Bounds are in the dimension's canonical
/// unit: millimetres for a length and degrees for an angle.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ParameterDomain {
    Number {
        dimension: Dimension,
        #[serde(default)]
        min: Option<f64>,
        /// A bound is exclusive unless this is true; an absent bound is unbounded.
        #[serde(default)]
        min_inclusive: bool,
        #[serde(default)]
        max: Option<f64>,
        /// Optional increment measured from `min` (or zero); a value off the grid is refused.
        #[serde(default)]
        step: Option<f64>,
    },
    Choice {
        choices: Vec<String>,
    },
    Boolean,
    Text {
        max_bytes: u64,
    },
}

/// A typed parameter value. The `kind` tag carries the unit, so a length can never be
/// read back as an angle and a bare number is never an implicit millimetre.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ParameterValue {
    Length { millimeters: f64 },
    Angle { degrees: f64 },
    Ratio { value: f64 },
    Count { value: u64 },
    Choice { value: String },
    Boolean { value: bool },
    Text { value: String },
}

/// One declared parameter: name, typed domain, optional typed default and requirement.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ParameterSpec {
    pub name: String,
    pub domain: ParameterDomain,
    #[serde(default)]
    pub default: Option<ParameterValue>,
    #[serde(default)]
    pub required: bool,
}

pub trait Plugin: Default {
    fn descriptor() -> PluginDescriptor;
    fn execute(&mut self, host: &mut Host<'_>, command: &str, arguments: Value) -> Result<Value>;
}

/// Invocation-scoped access to host services. Intentionally neither Send nor Sync.
///
/// ```compile_fail
/// fn require_send<T: Send>() {}
/// require_send::<om9_sdk::Host<'static>>();
/// ```
pub struct Host<'a> {
    api: &'a abi::HostApi,
    _thread_bound: PhantomData<Rc<()>>,
}

impl<'a> Host<'a> {
    /// Borrow a caller-owned host table for this synchronous invocation.
    ///
    /// # Safety
    /// A non-null pointer must be aligned and readable for its two-u32 header.
    /// If the header advertises at least `size_of::<HostApi>()` bytes, that full
    /// table must be readable and live for `'a`. Its callback and context must
    /// remain valid on this thread for `'a`, obey the synchronous ABI contract,
    /// never unwind, and provide readable reply bytes for the callback duration.
    /// The caller must restrict `'a` to this invocation and prevent concurrent
    /// mutation of the host context. Structural checks cannot validate pointers.
    pub unsafe fn from_raw(api: *const abi::HostApi) -> Result<Self> {
        if api.is_null() || !(api as usize).is_multiple_of(std::mem::align_of::<abi::HostApi>()) {
            return Err(SdkError::new(
                "invalid_host",
                "Missing or misaligned host table",
            ));
        }
        // Read only the fixed prefix until the table's advertised size is checked.
        let version = unsafe { std::ptr::read(api.cast::<u32>()) };
        let size = unsafe { std::ptr::read(api.cast::<u32>().add(1)) };
        if version != abi::ABI_VERSION {
            return Err(SdkError::new("abi_version", "Unsupported host ABI version"));
        }
        if (size as usize) < std::mem::size_of::<abi::HostApi>() {
            return Err(SdkError::new("abi_size", "Host table is too small"));
        }
        let api = unsafe { &*api };
        if api.call.is_none() {
            return Err(SdkError::new("invalid_host", "Missing host callback"));
        }
        Ok(Self {
            api,
            _thread_bound: PhantomData,
        })
    }

    fn request<T: DeserializeOwned>(&mut self, request: HostRequest) -> Result<T> {
        let bytes = abi::encode(&request)?;
        let mut capture = abi::ReplyCapture::new();
        let call = self
            .api
            .call
            .ok_or_else(|| SdkError::new("invalid_host", "Missing host callback"))?;
        // Host table construction guarantees validity and synchronous callbacks.
        let status = unsafe {
            call(
                self.api.context,
                bytes.as_ptr(),
                bytes.len(),
                capture.context(),
                abi::capture_reply,
            )
        };
        let bytes = capture.finish(status)?;
        serde_json::from_slice(&bytes)
            .map_err(|error| SdkError::new("invalid_reply", error.to_string()))
    }

    pub fn document_info(&mut self) -> Result<DocumentInfo> {
        self.request(HostRequest::DocumentInfo)
    }

    pub fn capabilities(&mut self) -> Result<Vec<String>> {
        self.request(HostRequest::Capabilities)
    }

    pub fn selected_objects(&mut self) -> Result<Vec<String>> {
        self.request(HostRequest::SelectionGet)
    }

    /// Replace the selection with exactly these object IDs; an empty list clears it.
    ///
    /// Every ID must be a canonical, selectable root that still exists in this command;
    /// one bad ID refuses the whole command before publication, so a stale or locked
    /// reference can never leave a half-updated selection. Selection is not document
    /// state: it creates no transaction or Undo entry, and the change is applied only
    /// after the command succeeds. `selected_objects` reads back the staged selection.
    pub fn set_selection(&mut self, ids: Vec<String>) -> Result<()> {
        self.request(HostRequest::SelectionSet { ids })
    }

    /// Add objects to the current selection, keeping the entries it already has.
    ///
    /// The same root/selectability rules and the same all-or-nothing refusal apply.
    pub fn add_to_selection(&mut self, ids: Vec<String>) -> Result<()> {
        self.request(HostRequest::SelectionAdd { ids })
    }

    /// Remove objects from the current selection.
    ///
    /// An ID that is not selected is a no-op, so removing a stale entry is safe; the ID
    /// still has to be a canonical object UUID. Selection is not document state.
    pub fn remove_from_selection(&mut self, ids: Vec<String>) -> Result<()> {
        self.request(HostRequest::SelectionRemove { ids })
    }

    /// Toggle objects in the current selection: selected IDs leave it, the rest join it.
    ///
    /// Joining follows the add rule (existing, selectable root); leaving never needs the
    /// object to be selectable, so a plugin can drop a stale or newly locked entry.
    pub fn toggle_selection(&mut self, ids: Vec<String>) -> Result<()> {
        self.request(HostRequest::SelectionToggle { ids })
    }

    pub fn layers(&mut self) -> Result<Vec<LayerInfo>> {
        self.request(HostRequest::LayersList)
    }

    /// Create one layer and optionally make it current; returns the new layer ID.
    ///
    /// The name grammar, name uniqueness and the layer budget are the document's own
    /// admission rules and are checked when the command prepares. If the layer is made
    /// current, newly added objects can use it in the same command: the transaction
    /// evaluates the proposed layer record and its parent chain. A layer whose flags
    /// this command changes stays closed to new output, because a permission read may
    /// not come from captured state the command rewrites. Existing or already staged
    /// objects may also be assigned to the new layer here.
    pub fn layer_create(
        &mut self,
        name: impl Into<String>,
        color: [u8; 4],
        make_current: bool,
    ) -> Result<String> {
        self.request(HostRequest::LayerCreate {
            name: name.into(),
            color,
            make_current,
        })
    }

    /// Rename or change the colour/flags of one layer.
    ///
    /// Absent update fields stay unchanged. The layer must be editable (unlocked);
    /// the change is staged in the command's transaction, so Undo restores the whole
    /// previous layer record. A rename and flag changes are one layer write.
    pub fn layer_update(&mut self, id: impl Into<String>, update: LayerUpdate) -> Result<()> {
        self.request(HostRequest::LayerUpdate {
            id: id.into(),
            update,
        })
    }

    /// Set the current layer, or clear it with `None`.
    pub fn set_current_layer(&mut self, id: Option<String>) -> Result<()> {
        self.request(HostRequest::SetCurrentLayer { id })
    }

    /// Assign objects to a layer.
    ///
    /// The target layer must exist (including one created by this command). Each
    /// object's current layer must be editable, and the object colour is cleared so
    /// the layer colour applies - the same rule the application's layer assignment
    /// uses. Assignment is an attribute write, so a command may not also transform,
    /// re-attribute or rename the same object.
    pub fn assign_objects_to_layer(
        &mut self,
        ids: Vec<String>,
        layer: impl Into<String>,
    ) -> Result<()> {
        self.request(HostRequest::AssignToLayer {
            ids,
            layer: layer.into(),
        })
    }

    /// Validate one typed builder parameter against its declared domain.
    ///
    /// `value` must be the typed wire form (`{"kind":"length","millimeters":2.5}` and so
    /// on); a bare JSON number is refused because a quantity needs a unit. Bounds, the
    /// optional step grid, the integer rule for a count, choices and text size use the
    /// document's own admission rule. The canonical value is returned (mm/degrees).
    pub fn validate_parameter(
        &mut self,
        spec: ParameterSpec,
        value: Value,
    ) -> Result<ParameterValue> {
        self.request(HostRequest::ValidateParameter { spec, value })
    }

    /// Apply defaults, refuse unknown/required parameters and validate every typed value.
    ///
    /// This is the builder/command parameter contract: at most 128 declarations and 128
    /// supplied values per call, typed values only, and the result contains the filled
    /// typed map in canonical units.
    pub fn apply_parameters(
        &mut self,
        schema: Vec<ParameterSpec>,
        supplied: BTreeMap<String, Value>,
    ) -> Result<BTreeMap<String, ParameterValue>> {
        self.request(HostRequest::ApplyParameters { schema, supplied })
    }

    /// Read a curve's defining data (object-local millimetres).
    ///
    /// NURBS curves report degree, poles, weights, knots, domain, periodic, reversed and
    /// seam; a circle reports its native domain and optional arc angle domain; a polycurve
    /// reports its segment count. Control data is bounded, and a larger curve is a `budget`
    /// refusal rather than a truncated definition.
    pub fn curve_definition(&mut self, id: impl Into<String>) -> Result<CurveDefinition> {
        self.request(HostRequest::CurveDefinition { id: id.into() })
    }

    /// Measure whether a curve is closed and planar.
    ///
    /// `tolerance_mm` is required to be a finite positive length; when omitted the
    /// document's own stated tolerance is used and a document that states none is a
    /// `tolerance` refusal (no default is invented). A curve whose sampled length is at or
    /// below the tolerance is refused as degenerate instead of being reclassified.
    pub fn curve_semantics(
        &mut self,
        id: impl Into<String>,
        tolerance_mm: Option<f64>,
    ) -> Result<CurveSemantics> {
        self.request(HostRequest::CurveSemantics {
            id: id.into(),
            tolerance_mm,
        })
    }
    /// Report command progress, shown in the host's command event log.
    ///
    /// Text is one line without control characters, at most 512 bytes. A command may
    /// report at most 32 messages; exceeding either bound is a `budget` refusal rather
    /// than a silent truncation. Reports are not document state: they never create an
    /// Undo entry, and a command that returns an error publishes nothing, reports
    /// included.
    pub fn progress(&mut self, text: impl Into<String>) -> Result<()> {
        self.request(HostRequest::Message {
            level: MessageLevel::Progress,
            text: text.into(),
        })
    }

    /// Read one curve's identity and native parameter domain (object-local mm).
    ///
    /// A `"polycurve"` reports its segment count and no single domain, because it needs
    /// an explicit segment parameter that this SDK version does not expose.
    pub fn curve_info(&mut self, id: impl Into<String>) -> Result<CurveInfo> {
        self.request(HostRequest::CurveInfo { id: id.into() })
    }

    /// Evaluate one curve at native parameters, object-local millimetres.
    ///
    /// `parameters` are native domain values (`curve_info().domain`), not 0..1
    /// fractions; a value outside the domain is a `curve` refusal. `order` requests
    /// D1..D3 (0 returns the point only) and a higher order is `unsupported`. At most
    /// 1024 samples are evaluated per call. A polycurve needs an explicit segment
    /// parameter and is refused; the object placement is not applied, so compose it
    /// exactly once in the plugin.
    pub fn evaluate_curve(
        &mut self,
        id: impl Into<String>,
        parameters: Vec<f64>,
        order: u8,
    ) -> Result<Vec<CurveSample>> {
        self.request(HostRequest::CurveEvaluate {
            id: id.into(),
            parameters,
            order,
        })
    }
    /// Report a nonfatal warning; the command may still commit.
    ///
    /// The same one-line, 512-byte and 32-message bounds apply.
    pub fn warning(&mut self, text: impl Into<String>) -> Result<()> {
        self.request(HostRequest::Message {
            level: MessageLevel::Warning,
            text: text.into(),
        })
    }

    /// Display unit and the document's stated tolerance policy.
    ///
    /// A missing tolerance is reported as `None`; it is never replaced by a
    /// default. Lengths, coordinates and translations remain canonical mm.
    pub fn document_units(&mut self) -> Result<DocumentUnits> {
        self.request(HostRequest::DocumentUnits)
    }

    /// Add an arc through three points (object-local millimetres).
    ///
    /// The arc is the circle through the points with an increasing angle domain from
    /// `start` to `end`; collinear or repeated points are a `curve` refusal. The new
    /// object is published in the command's transaction under the document's current
    /// layer and permissions.
    pub fn add_arc(
        &mut self,
        label: impl Into<String>,
        start: [f64; 3],
        through: [f64; 3],
        end: [f64; 3],
    ) -> Result<String> {
        self.request(HostRequest::ObjectAddArc {
            label: label.into(),
            start,
            through,
            end,
        })
    }

    /// Add an interpolated NURBS curve through `points` (object-local millimetres).
    ///
    /// `degree` is the document's interpolated-curve rule: odd 1..=11 and smaller than the
    /// point count; an even or out-of-range degree is a `curve` refusal. `mode` selects
    /// chord-length or uniform knots. The result is one document transaction entry.
    pub fn add_interpolated_curve(
        &mut self,
        label: impl Into<String>,
        points: Vec<[f64; 3]>,
        degree: u32,
        mode: KnotMode,
    ) -> Result<String> {
        self.request(HostRequest::ObjectAddInterpolatedCurve {
            label: label.into(),
            points,
            degree,
            mode,
        })
    }
    /// Add a control-point NURBS curve (object-local millimetres).
    ///
    /// `poles`, `weights` and `knots` are the document's NURBS record: one weight per pole,
    /// `poles.len() + degree + 1` non-decreasing knots, a finite positive degree, and the
    /// standard clamped domain `[knots[degree], knots[poles.len()]]`. A malformed record is
    /// a `curve` refusal, control data is bounded and the curve publishes as one object in
    /// the command's transaction. Periodic/seam construction is not exposed.
    pub fn add_nurbs_curve(
        &mut self,
        label: impl Into<String>,
        degree: u32,
        poles: Vec<[f64; 3]>,
        weights: Vec<f64>,
        knots: Vec<f64>,
    ) -> Result<String> {
        self.request(HostRequest::ObjectAddNurbsCurve {
            label: label.into(),
            degree,
            poles,
            weights,
            knots,
        })
    }
    /// Add an exact rational-quadratic ellipse (object-local millimetres).
    ///
    /// The document has no analytic ellipse kind, so the result is a degree-2 NURBS on the
    /// orthonormal frame `(x, y)` through `origin`; `x` and `y` must be unit and
    /// perpendicular, and both radii positive. A malformed frame or radius is a `curve`
    /// refusal. The curve publishes as one object in the command's transaction.
    pub fn add_ellipse(
        &mut self,
        label: impl Into<String>,
        origin: [f64; 3],
        x: [f64; 3],
        y: [f64; 3],
        radius_x: f64,
        radius_y: f64,
    ) -> Result<String> {
        self.request(HostRequest::ObjectAddEllipse {
            label: label.into(),
            origin,
            x,
            y,
            radius_x,
            radius_y,
        })
    }

    /// Add a closed rectangle (four line segments) on the orthonormal frame `(x, y)`.
    ///
    /// The rectangle starts at `origin`, runs `width` along `x` and `height` along `y` and
    /// closes on itself as one polycurve; the frame must be unit and perpendicular and both
    /// sizes positive. The curve publishes as one object in the command's transaction.
    pub fn add_rectangle(
        &mut self,
        label: impl Into<String>,
        origin: [f64; 3],
        x: [f64; 3],
        y: [f64; 3],
        width: f64,
        height: f64,
    ) -> Result<String> {
        self.request(HostRequest::ObjectAddRectangle {
            label: label.into(),
            origin,
            x,
            y,
            width,
            height,
        })
    }
    /// Add a closed polygon through three or more points (object-local millimetres).
    ///
    /// The polygon is a closed polycurve: the first point is appended when the caller did
    /// not repeat it, and no segment may be degenerate. Fewer than three distinct points,
    /// repeated consecutive points or a degenerate edge are a `curve` refusal. The polygon
    /// publishes as one object in the command's transaction.
    pub fn add_polygon(
        &mut self,
        label: impl Into<String>,
        points: Vec<[f64; 3]>,
    ) -> Result<String> {
        self.request(HostRequest::ObjectAddPolygon {
            label: label.into(),
            points,
        })
    }
    /// Read one object's extension slots as `slot -> { version, payload }`.
    ///
    /// Extensions are the document's verbatim JSON-object metadata (jewelry weight,
    /// provenance, plugin data). The payload is returned as stored, not re-encoded, and
    /// reading stages no mutation.
    pub fn object_extensions(
        &mut self,
        id: impl Into<String>,
    ) -> Result<std::collections::BTreeMap<String, ObjectExtension>> {
        self.request(HostRequest::ObjectExtensions { id: id.into() })
    }

    /// Set one extension slot with `version >= 1` and a non-empty JSON object payload.
    ///
    /// The core record validation decides admission (positive version, object text, size
    /// budget) and a malformed slot/value is an `extension` refusal. One source write per
    /// object per command still applies, so this conflicts with a placement, attribute,
    /// label or layer write to the same object.
    pub fn set_object_extension(
        &mut self,
        id: impl Into<String>,
        slot: impl Into<String>,
        version: u32,
        payload: impl Into<String>,
    ) -> Result<()> {
        self.request(HostRequest::ObjectSetExtension {
            id: id.into(),
            slot: slot.into(),
            version,
            payload: payload.into(),
        })
    }

    /// Clear one extension slot; clearing an absent slot is a no-op.
    pub fn clear_object_extension(
        &mut self,
        id: impl Into<String>,
        slot: impl Into<String>,
    ) -> Result<()> {
        self.request(HostRequest::ObjectClearExtension {
            id: id.into(),
            slot: slot.into(),
        })
    }
    /// Add a polygon mesh from vertices and face index lists (object-local millimetres).
    ///
    /// Faces are indices into `vertices`; the mesh record validation decides admission
    /// (bounded channels, in-range indices, finite points), so a malformed mesh is a `mesh`
    /// refusal. Normals and per-face colors are not accepted in this version. The mesh
    /// publishes as one object in the command's transaction.
    pub fn add_mesh(
        &mut self,
        label: impl Into<String>,
        vertices: Vec<[f64; 3]>,
        faces: Vec<Vec<u32>>,
    ) -> Result<String> {
        self.request(HostRequest::ObjectAddMesh {
            label: label.into(),
            vertices,
            faces,
        })
    }
    /// Add a full circle on an arbitrary orthonormal frame.
    ///
    /// Unlike `add_circle` (which lies on the world XY plane) this places the circle on
    /// the plane through `center` spanned by `x` and `y`; the frame must be unit and
    /// perpendicular and the radius positive, otherwise it is a `curve` refusal. The
    /// native parameter is the angle in radians and the angle domain is `[0, tau]`.
    pub fn add_circle_on_frame(
        &mut self,
        label: impl Into<String>,
        center: [f64; 3],
        x: [f64; 3],
        y: [f64; 3],
        radius: f64,
    ) -> Result<String> {
        self.request(HostRequest::ObjectAddCircleFrame {
            label: label.into(),
            center,
            x,
            y,
            radius,
        })
    }
    /// Measure one object's object-local bounding box, size and center.
    ///
    /// A point contributes its position, a curve contributes its sampled points (the
    /// owner's 33-sample curve, object-local millimetres) and a mesh contributes its
    /// vertices. Other geometry kinds are `unsupported`; a sample that exceeds the
    /// limits is `budget`. The placement is not applied, so compose it exactly once.
    pub fn object_bounds(&mut self, id: impl Into<String>) -> Result<ObjectBounds> {
        self.request(HostRequest::ObjectBounds { id: id.into() })
    }
    /// Report one measurement label through the command event log.
    ///
    /// The text becomes one `Progress` line (at most 512 bytes, at most 32 per command), so
    /// a measurement value reaches the Command line and its observers. Empty or oversized
    /// text is a `message` refusal. The report is discarded if the command later fails.
    pub fn report_measurement(&mut self, text: impl Into<String>) -> Result<()> {
        self.request(HostRequest::MeasureLabel { text: text.into() })
    }
    /// List the document's layer order, bottom to top, as layer ids.
    ///
    /// The order is the document's `layer_order`; a layer created by this command is
    /// appended so a later step in the same command sees it. Reading stages no mutation.
    pub fn layer_order(&mut self) -> Result<Vec<String>> {
        self.request(HostRequest::LayerOrder)
    }
    /// Clear the current layer tag.
    ///
    /// The document keeps its layer records; only the current-layer pointer becomes
    /// `None`, so new objects use the document default. Clearing an already-empty current
    /// layer is a no-op and stages no mutation.
    pub fn clear_current_layer(&mut self) -> Result<()> {
        self.request(HostRequest::LayerClearCurrent)
    }
    /// List the document's block definitions with member counts.
    ///
    /// This is a read-only view of the definition records; inserting an instance or editing
    /// a definition is not exposed in this SDK version. Reading stages no mutation.
    pub fn block_definitions(&mut self) -> Result<Vec<BlockDefinitionInfo>> {
        self.request(HostRequest::DefinitionsList)
    }
    /// List the document's groups with their member object ids.
    ///
    /// This is a read-only view of the group records; creating, editing or dissolving a
    /// group is not exposed in this SDK version. Reading stages no mutation.
    /// Create one group from existing member ids.
    ///
    /// The name must be non-empty and bounded, and every member must be an existing,
    /// non-deleted object; otherwise it is a `group`/`missing` refusal. The group is created
    /// in the command's transaction, so one Undo removes it.
    /// Delete one group record.
    ///
    /// Only the group record is removed; its member objects stay in the document. The
    /// deletion is one transaction mutation, so one Undo restores the group. An unknown id
    /// is `missing`.
    /// Read one layer's record by id.
    ///
    /// The record carries the effective visible/selectable permissions after the parent
    /// chain, the lock inversion (`locked = !editable`) and the current flag, matching the
    /// layer list. Reading stages no mutation.
    /// Set the document tolerance policy (absolute mm, relative, angular radians).
    ///
    /// The document's own validation decides admission: each value must be finite and
    /// strictly positive (except a non-negative relative), otherwise it is a `tolerance`
    /// refusal. The change is a transaction mutation, so one Undo restores the previous
    /// policy (or removes it when none was stated).
    /// Read the retained import source proof for one object.
    ///
    /// `None` means the object carries no complete source chain: a partial or absent
    /// namespace, source or hash agreement is never reported as a native import. Reading
    /// stages no mutation.
    /// Whether one object can be picked: effective visible **and** selectable.
    ///
    /// The effective flags come from the whole layer chain plus the object's own
    /// attributes. `locked` blocks editing, not picking, so it does not make this false.
    /// Reading stages no mutation.
    pub fn is_object_pickable(&mut self, id: impl Into<String>) -> Result<bool> {
        self.request(HostRequest::IsObjectPickable { id: id.into() })
    }
    pub fn object_provenance(&mut self, id: impl Into<String>) -> Result<Option<ObjectProvenance>> {
        self.request(HostRequest::ObjectProvenance { id: id.into() })
    }
    pub fn set_document_tolerance(
        &mut self,
        absolute_mm: f64,
        relative: f64,
        angular_rad: f64,
    ) -> Result<()> {
        self.request(HostRequest::SetTolerance {
            absolute_mm,
            relative,
            angular_rad,
        })
    }
    pub fn layer_info(&mut self, id: impl Into<String>) -> Result<LayerInfo> {
        self.request(HostRequest::LayerInfoGet { id: id.into() })
    }
    pub fn delete_group(&mut self, id: impl Into<String>) -> Result<()> {
        self.request(HostRequest::GroupDelete { id: id.into() })
    }
    pub fn create_group(
        &mut self,
        name: impl Into<String>,
        members: Vec<String>,
    ) -> Result<String> {
        self.request(HostRequest::GroupCreate {
            name: name.into(),
            members,
        })
    }
    pub fn groups(&mut self) -> Result<Vec<GroupInfo>> {
        self.request(HostRequest::GroupsList)
    }
    /// Measure a polygon mesh's surface area in square millimetres.
    ///
    /// Each face is triangulated as a fan around its first vertex and the triangle areas
    /// are summed in object-local space, so the value is planar-face area (a non-planar
    /// quad is measured as its fan triangles). A non-mesh object is `unsupported`.
    pub fn measure_mesh_area(&mut self, id: impl Into<String>) -> Result<MeasuredArea> {
        self.request(HostRequest::MeasureMeshArea { id: id.into() })
    }
    /// Count the document's objects by geometry kind.
    ///
    /// The count starts from the open document, removes objects deleted by this command and
    /// includes objects this command already added, so a later step in the same command sees
    /// a consistent summary. Kinds use the geometry tag (`Point`, `Curve`, `Mesh`, `Brep`,
    /// `Surface`, `Cloud`, `Instance`, `Hatch`, `Light`).
    pub fn object_summary(&mut self) -> Result<ObjectSummary> {
        self.request(HostRequest::ObjectSummary)
    }
    /// Measure a circle curve's radius in object-local millimetres.
    ///
    /// Only native circle geometry (including an arc angle domain) has a radius; a line,
    /// NURBS or polycurve is `unsupported`, so a caller never receives a guessed radius.
    pub fn measure_curve_radius(&mut self, id: impl Into<String>) -> Result<MeasuredDistance> {
        self.request(HostRequest::MeasureCurveRadius { id: id.into() })
    }
    /// Measure a curve's length from the owner's fixed 33-sample sweep.
    ///
    /// The result is a sampled length in object-local millimetres: the segments between
    /// consecutive samples are summed, so a highly curved or discontinuous curve is an
    /// approximation, not an exact arc length. Unsupported geometry is `unsupported`, and
    /// a sample failure is `measure`.
    pub fn measure_curve_length(&mut self, id: impl Into<String>) -> Result<MeasuredDistance> {
        self.request(HostRequest::MeasureCurveLength { id: id.into() })
    }
    /// List a curve's segments: a native curve is one segment, a polycurve is many.
    ///
    /// Each entry reports the segment kind, native domain and arc angle domain when the
    /// segment is a circle. A nested polycurve is refused rather than flattened, and the
    /// document budget bounds the list.
    pub fn curve_segments(&mut self, id: impl Into<String>) -> Result<Vec<CurveSegment>> {
        self.request(HostRequest::CurveSegments { id: id.into() })
    }

    /// Evaluate one curve segment at native parameters (object-local millimetres).
    ///
    /// `index` addresses `curve_segments`; `0` on a native curve evaluates the curve itself
    /// and a larger index is a `curve` refusal. The same D0-D3 order and 1024-sample bound
    /// as `evaluate_curve` apply, and a nested polycurve is `unsupported`.
    pub fn evaluate_curve_segment(
        &mut self,
        id: impl Into<String>,
        index: u32,
        parameters: Vec<f64>,
        order: u8,
    ) -> Result<Vec<CurveSample>> {
        self.request(HostRequest::CurveEvaluateSegment {
            id: id.into(),
            index,
            parameters,
            order,
        })
    }
    /// Read one object's identity/placement record without paging a query.
    ///
    /// The record is the same `ObjectInfo` a query returns: id, label, geometry kind,
    /// placement, layer, flags and revisions. Reading stages no mutation.
    pub fn object_info(&mut self, id: impl Into<String>) -> Result<ObjectInfo> {
        self.request(HostRequest::ObjectInfoGet { id: id.into() })
    }
    /// Read one object's affine placement matrix (row-major, object-local -> world).
    ///
    /// This is the object-local placement only; a parent/instance transform is not composed
    /// here. Reading stages no mutation.
    pub fn object_placement(&mut self, id: impl Into<String>) -> Result<[[f64; 4]; 4]> {
        self.request(HostRequest::ObjectPlacement { id: id.into() })
    }
    /// Read the change batch recorded for the last committed change.
    ///
    /// The host records the committed difference of every plugin command (cause `edit`)
    /// and of host Undo/Redo (cause `undo`/`redo`), as one bounded batch with real
    /// `from_revision`/`to_revision`. `None` means nothing has been recorded yet. Other
    /// host edits outside these paths are not in the feed; compare
    /// `document_info().revision` for those. Reading stages no mutation.
    pub fn last_change(&mut self) -> Result<Option<ProjectChange>> {
        self.request(HostRequest::LastChange)
    }
    /// Set one object's affine placement matrix (row-major, object-local -> world).
    ///
    /// The matrix must be a finite affine transform: last row `[0,0,0,1]` and an
    /// invertible 3x3 linear part; otherwise it is a `placement` refusal. The write is
    /// one source write like attributes/rename/layer, so mixing them in one command
    /// conflicts, and Undo restores the previous placement. An object created in the
    /// same command already carries its placement through the insert and is refused here.
    pub fn set_object_placement(
        &mut self,
        id: impl Into<String>,
        matrix: [[f64; 4]; 4],
    ) -> Result<()> {
        self.request(HostRequest::ObjectSetPlacement {
            id: id.into(),
            matrix,
        })
    }
    /// Read the project path and dirty state.
    ///
    /// This is the host's file identity, not document state: `path` is the last opened or
    /// saved `.om9` path (absent for an untitled project) and `dirty` compares the current
    /// document against the last saved/opened fingerprint. A command cannot change either
    /// value directly, and reading stages no mutation.
    pub fn project_state(&mut self) -> Result<ProjectInfo> {
        self.request(HostRequest::ProjectState)
    }

    /// Ask the host to open its Save As flow after this command commits.
    ///
    /// The command's transaction still commits first; the host then starts its own Save As
    /// operation, which asks the user for a destination and performs the atomic save. A
    /// plugin cannot choose the destination or write a path itself, and combining this with
    /// `save_project` in one command is a `save` refusal so the order is never ambiguous.
    pub fn save_project_as(&mut self) -> Result<()> {
        self.request(HostRequest::SaveProjectAs)
    }
    /// Request that the host save the project after this command commits.
    ///
    /// The request is staged, not performed inside the command: the command's transaction
    /// commits first and then the host writes the project with its atomic save path. It is
    /// refused before commit when the project has no path (untitled). A save failure after
    /// commit is reported as a warning event and the document stays dirty, so a command is
    /// never reported as failed after its edits were published. A plugin cannot open, create
    /// or replace the active document; those remain interactive host operations.
    pub fn save_project(&mut self) -> Result<()> {
        self.request(HostRequest::SaveProject)
    }
    /// Measure the distance between two world points, in millimetres.
    ///
    /// Points are world millimetres supplied by the plugin; a non-finite coordinate or one
    /// beyond the document's measurement bound is a `measure` refusal.
    pub fn measure_distance(
        &mut self,
        first: [f64; 3],
        second: [f64; 3],
    ) -> Result<MeasuredDistance> {
        self.request(HostRequest::MeasureDistance { first, second })
    }

    /// Measure the undirected angle between two directed lines, in degrees.
    ///
    /// Each line is `from -> to`; a zero-length line has no direction and is a `measure`
    /// refusal. The result is in `[0, 180]` and uses the same `atan2` rule as the
    /// document analysis command.
    pub fn measure_angle(
        &mut self,
        from_a: [f64; 3],
        to_a: [f64; 3],
        from_b: [f64; 3],
        to_b: [f64; 3],
    ) -> Result<MeasuredAngle> {
        self.request(HostRequest::MeasureAngle {
            from_a,
            to_a,
            from_b,
            to_b,
        })
    }
    /// Parse one length with the host's typed parser (at most 256 bytes).
    ///
    /// A bare number uses the document display unit, so `"2.5"` in an inch
    /// document is 63.5 mm. A trailing token such as `"2.5 mm"` or `"0.1 in"`
    /// carries its own unit, including `cm`, `m`, `ft` and `um`. Non-finite
    /// values, malformed text and unsupported units are errors, never zero.
    pub fn parse_length(&mut self, text: impl Into<String>) -> Result<LengthValue> {
        self.request(HostRequest::ParseLength { text: text.into() })
    }

    /// Parse one angle with the host's typed parser (at most 256 bytes).
    ///
    /// A bare number and `deg`/`°` tokens are degrees. A `rad` token converts
    /// explicitly, so `"1.5707963267948966rad"` is 90 degrees. The absolute
    /// degree bound and the non-finite refusal come from the core numeric
    /// policy, not from a plugin-supplied default.
    pub fn parse_angle(&mut self, text: impl Into<String>) -> Result<AngleValue> {
        self.request(HostRequest::ParseAngle { text: text.into() })
    }

    /// Parse `x,y` or `x,y,z` (at most 256 bytes) in the document display unit.
    ///
    /// This is the command prompt's typed coordinate grammar: components are
    /// separated by commas, the third component defaults to zero and every
    /// value is converted to canonical mm. The result is not a world point
    /// until the host applies an explicit CPlane frame; no plugin may treat the
    /// tuple as already transformed, and placement must not be applied twice.
    pub fn parse_coordinates(&mut self, text: impl Into<String>) -> Result<Coordinates> {
        self.request(HostRequest::ParseCoordinates { text: text.into() })
    }

    pub fn object_geometry(&mut self, id: impl Into<String>) -> Result<Geometry> {
        self.request(HostRequest::ObjectGeometry { id: id.into() })
    }

    pub fn objects(&mut self) -> Result<Vec<ObjectInfo>> {
        self.request(HostRequest::ObjectsList)
    }

    /// Query objects with filters, paging and revision fields.
    ///
    /// The result is ordered by object ID and includes only what this command's
    /// document already contains plus its own staged additions and attribute
    /// edits. A page is capped at 1024 objects and the host refuses a query whose
    /// page would exceed the reply budget rather than truncating silently.
    pub fn query_objects(&mut self, query: ObjectQuery) -> Result<ObjectPage> {
        self.request(HostRequest::ObjectsQuery { query })
    }

    /// Stored and effective attribute values for one object.
    pub fn object_attributes(&mut self, id: impl Into<String>) -> Result<ObjectAttributes> {
        self.request(HostRequest::ObjectAttributes { id: id.into() })
    }

    /// Apply one attribute edit to one object and return the staged result.
    ///
    /// The host refuses the edit when the object's layer is locked. Object-level
    /// `locked`/`hidden` flags are themselves editable (that is how a plugin
    /// unlocks or unhides an object) and the change is staged in the command's
    /// single transaction, so Undo restores the previous record. A command may
    /// not combine this edit with a transform or a rename of the same object:
    /// the document transaction contract allows one kind of source write per
    /// object per command.
    pub fn set_object_attributes(
        &mut self,
        id: impl Into<String>,
        patch: AttributesPatch,
    ) -> Result<ObjectAttributes> {
        self.request(HostRequest::ObjectSetAttributes {
            id: id.into(),
            patch,
        })
    }

    /// Rename one object. The label grammar and the one-edit-per-object rule of
    /// [`Host::set_object_attributes`] apply.
    pub fn rename_object(&mut self, id: impl Into<String>, label: impl Into<String>) -> Result<()> {
        self.request(HostRequest::ObjectRename {
            id: id.into(),
            label: label.into(),
        })
    }

    pub fn add_object(&mut self, label: impl Into<String>, geometry: Geometry) -> Result<String> {
        self.request(HostRequest::ObjectAdd {
            label: label.into(),
            geometry,
        })
    }

    pub fn add_point(&mut self, label: impl Into<String>, point: [f64; 3]) -> Result<String> {
        self.add_object(label, Geometry::Point { point })
    }

    pub fn add_line(
        &mut self,
        label: impl Into<String>,
        start: [f64; 3],
        end: [f64; 3],
    ) -> Result<String> {
        self.add_object(label, Geometry::Line { start, end })
    }

    pub fn add_polyline(
        &mut self,
        label: impl Into<String>,
        points: Vec<[f64; 3]>,
    ) -> Result<String> {
        self.add_object(label, Geometry::Polyline { points })
    }

    pub fn add_circle(
        &mut self,
        label: impl Into<String>,
        center: [f64; 3],
        x: [f64; 3],
        y: [f64; 3],
        radius: f64,
    ) -> Result<String> {
        self.add_object(
            label,
            Geometry::Circle {
                center,
                x,
                y,
                radius,
            },
        )
    }

    pub fn delete_object(&mut self, id: impl Into<String>) -> Result<()> {
        self.request(HostRequest::ObjectDelete { id: id.into() })
    }

    pub fn transform_object(&mut self, id: impl Into<String>, matrix: [[f64; 4]; 4]) -> Result<()> {
        self.request(HostRequest::ObjectTransform {
            id: id.into(),
            matrix,
        })
    }
}

/// Export the one C entry point required by the OM9 native plugin loader.
/// Invoke once in a plugin crate with `crate-type = ["cdylib"]`.
#[macro_export]
macro_rules! export_plugin {
    ($plugin:ty) => {
        /// OM9 SDK v1 entry point. The caller must uphold `abi::dispatch` safety.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn om9_plugin_entry_v1(
            host: *const $crate::abi::HostApi,
            request: *const u8,
            request_len: usize,
            reply_context: *mut ::std::ffi::c_void,
            reply: $crate::abi::ReplyFn,
        ) -> i32 {
            unsafe {
                $crate::abi::dispatch::<$plugin>(host, request, request_len, reply_context, reply)
            }
        }
    };
}
