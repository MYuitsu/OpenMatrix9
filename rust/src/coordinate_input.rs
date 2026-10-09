//! Stateless coordinate input and owned provenance for native command adapters.
//!
//! Grammar/unit conversion stay in `coordinates::PointInput` and `units`. This
//! layer owns phase routing, world-source bypass, snapshots and JSON reports;
//! it retains no host pointer, document object or mutable command session.
use crate::coordinates::{Frame, Point, PointInput, PointKind};
use crate::units;
use std::fmt::Write;

const MAX_TEXT: usize = 1024;
const MAX_UID: usize = 256;
const MAX_UNITS: usize = 512;
pub const MAX_INPUTS: usize = 64;
pub const MAX_REPORT_BYTES: usize = 1024 * 1024;
const LIMIT: f64 = 1e9;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Expected {
    Point,
    PointOrLength,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    Typed,
    WorldPick,
    WorldSnap,
    OriginShortcut,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Context {
    pub document_owner: u64,
    pub document_uid: String,
    pub frame_revision: u64,
    pub logical_view_slot: u32,
    pub frame: Frame,
    pub unit_context: String,
}

impl Context {
    fn units(&self) -> Result<Option<units::Context>, String> {
        if self.unit_context.len() > MAX_UNITS {
            return Err("Unit context exceeds 512 bytes".into());
        }
        if self.document_owner == 0
            || self.frame_revision == 0
            || self.logical_view_slot >= 4
            || self.document_uid.is_empty()
            || self.document_uid.len() > MAX_UID
        {
            return Err("Invalid coordinate context identity".into());
        }
        Frame::new(self.frame.origin(), self.frame.axes())?;
        if self.unit_context.is_empty() {
            Ok(None)
        } else {
            units::Context::decode(&self.unit_context)
                .map(Some)
                .map_err(|e| e.to_string())
        }
    }

    /// Owner tokens must never be reused by the host. UID alone cannot identify
    /// a reopened document. Current frame/view are absent by design: the command
    /// continues to use the frame captured at command start.
    pub fn verify(&self, owner: u64, uid: &str, unit_context: &str) -> Result<(), String> {
        self.units()?;
        if owner != self.document_owner || uid != self.document_uid {
            return Err("Coordinate document lifetime changed".into());
        }
        if unit_context != self.unit_context {
            return Err("Unit context changed during coordinate input; restart the command".into());
        }
        Ok(())
    }
}
pub struct Request<'a> {
    pub context: &'a Context,
    pub expected: Expected,
    pub source: Source,
    pub original: &'a str,
    pub previous: Option<Point>,
    pub world: Point,
}
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Point(Point),
    Length(f64),
}
#[derive(Clone, Debug, PartialEq)]
pub struct Resolved {
    value: Value,
    context: Context,
    expected: Expected,
    source: Source,
    original: String,
    previous: Option<Point>,
    components: Vec<f64>,
    kind: &'static str,
}

fn check_point(point: Point) -> Result<(), String> {
    if point.iter().any(|n| !n.is_finite() || n.abs() > LIMIT) {
        Err("Coordinate must be finite and within 1e9 mm".into())
    } else {
        Ok(())
    }
}
fn source_name(source: Source) -> &'static str {
    match source {
        Source::Typed => "typed",
        Source::WorldPick => "world_pick",
        Source::WorldSnap => "world_snap",
        Source::OriginShortcut => "origin_shortcut",
    }
}

pub fn resolve(request: &Request<'_>) -> Result<Resolved, String> {
    let units = request.context.units()?;
    if request.original.len() > MAX_TEXT {
        return Err("Coordinate input exceeds 1024 bytes".into());
    }
    if let Some(previous) = request.previous {
        check_point(previous)?;
    }
    let scale = units.as_ref().map_or(1., |context| context.model_mm);
    let parse = |token: &str| units::parse_length(token, scale).map_err(|error| error.to_string());
    let (value, components, kind) = if request.source != Source::Typed {
        if !request.original.is_empty() {
            return Err("World-coordinate sources cannot contain typed text".into());
        }
        check_point(request.world)?;
        (
            Value::Point(request.world),
            request.world.to_vec(),
            source_name(request.source),
        )
    } else if request.expected == Expected::PointOrLength && !request.original.contains(',') {
        let length = parse(request.original)?;
        check_point([length, 0., 0.])?;
        (Value::Length(length), vec![length], "length")
    } else {
        let input = PointInput::parse(request.original, parse)?;
        let point = input.resolve(request.context.frame, request.previous)?;
        let kind = match point.input.kind {
            PointKind::AbsoluteCPlane => "absolute_cplane",
            PointKind::AbsoluteWorld => "absolute_world",
            PointKind::RelativeCPlane => "relative_cplane",
            PointKind::RelativeWorld => "relative_world",
        };
        (
            Value::Point(point.world),
            point.input.components[..point.input.component_count].to_vec(),
            kind,
        )
    };
    Ok(Resolved {
        value,
        context: request.context.clone(),
        expected: request.expected,
        source: request.source,
        original: request.original.into(),
        previous: request.previous,
        components,
        kind,
    })
}

// Escape all JSON control characters while preserving Unicode. Numeric values
// are validated before serialization; native code supplies no JSON fragments.
fn quoted(value: &str) -> String {
    let mut output = String::from("\"");
    for c in value.chars() {
        match c {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\u{0}'..='\u{1f}' => {
                write!(&mut output, "\\u{:04x}", c as u32).unwrap();
            }
            _ => output.push(c),
        }
    }
    output.push('"');
    output
}
fn numbers(values: &[f64]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(f64::to_string)
            .collect::<Vec<_>>()
            .join(",")
    )
}

impl Resolved {
    pub fn value(&self) -> &Value {
        &self.value
    }

    pub fn report(&self) -> String {
        let context = &self.context;
        let units = context.units().expect("resolved context was validated");
        let units = match units {
            None => "{\"mode\":\"legacy_mm\",\"raw_context\":\"\",\"model_mm\":1}".into(),
            Some(units) => format!(
                "{{\"mode\":\"document\",\"raw_context\":{},\"revision\":{},\"model_mm\":{},\"page_mm\":{},\"absolute_mm\":{},\"relative_ratio\":{},\"angular_rad\":{}}}",
                quoted(&context.unit_context),
                quoted(&units.revision.to_string()),
                units.model_mm,
                units.page_mm,
                units.absolute_mm,
                units.relative_ratio,
                units.angular_rad
            ),
        };
        let result = match self.value {
            Value::Point(point) => format!("{{\"point_mm\":{}}}", numbers(&point)),
            Value::Length(length) => format!("{{\"length_mm\":{length}}}"),
        };
        let previous = self.previous.map_or_else(|| "null".into(), |p| numbers(&p));
        let source = if self.source == Source::WorldSnap {
            "{\"kind\":\"world_snap\",\"identity\":null,\"identity_status\":\"unavailable\"}".into()
        } else {
            format!("{{\"kind\":{}}}", quoted(source_name(self.source)))
        };
        let axes = context.frame.axes().map(|axis| numbers(&axis)).join(",");
        let expected = match self.expected {
            Expected::Point => "point",
            Expected::PointOrLength => "point_or_length",
        };
        format!(
            "{{\"version\":1,\"grammar\":\"cartesian_v1\",\"locale\":\"dot_decimal\",\"original_text\":{},\"expected\":{},\"source\":{},\"kind\":{},\"components_mm\":{},\"result\":{},\"previous_world_mm\":{},\"context\":{{\"document\":{{\"runtime_owner\":{},\"uid\":{}}},\"view\":{{\"logical_slot\":{},\"identity_scope\":\"logical_document_slot\"}},\"frame\":{{\"revision\":{},\"origin_mm\":{},\"axes\":[{}],\"latch\":\"command_start\"}},\"units\":{}}}}}",
            quoted(&self.original),
            quoted(expected),
            source,
            quoted(self.kind),
            numbers(&self.components),
            result,
            previous,
            quoted(&context.document_owner.to_string()),
            quoted(&context.document_uid),
            context.logical_view_slot,
            quoted(&context.frame_revision.to_string()),
            numbers(&context.frame.origin()),
            axes,
            units
        )
    }
}

pub fn report(inputs: &[Resolved]) -> Result<String, String> {
    if inputs.len() > MAX_INPUTS {
        return Err("Coordinate report exceeds 64 inputs".into());
    }
    if let Some(first) = inputs.first() {
        if inputs.iter().any(|input| input.context != first.context) {
            return Err("Coordinate report inputs have different command contexts".into());
        }
    }
    let result = format!(
        "{{\"version\":1,\"inputs\":[{}]}}",
        inputs
            .iter()
            .map(Resolved::report)
            .collect::<Vec<_>>()
            .join(",")
    );
    if result.len() + 1 > MAX_REPORT_BYTES {
        return Err("Coordinate report exceeds the output budget".into());
    }
    Ok(result)
}

/// ABI version 1. Strings are borrowed UTF-8 byte slices, not C strings. Empty
/// slices may use null; no pointer is retained. Maximum lengths: original 1024,
/// units 512, UID 256 bytes. Frame is origin,X,Y,Z; previous/world are in mm.
#[repr(C)]
pub struct CoordinateRequest {
    pub version: u32,
    pub expected: u32,
    pub source: u32,
    pub has_previous: u32,
    pub document_owner: u64,
    pub frame_revision: u64,
    pub logical_view_slot: u32,
    pub reserved: u32,
    pub frame: [f64; 12],
    pub previous: Point,
    pub world: Point,
    pub original: *const u8,
    pub original_len: usize,
    pub unit_context: *const u8,
    pub unit_context_len: usize,
    pub document_uid: *const u8,
    pub document_uid_len: usize,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CoordinateValue {
    pub kind: u32,
    pub reserved: u32,
    pub point: Point,
    pub length_mm: f64,
}

unsafe fn text<'a>(pointer: *const u8, length: usize, limit: usize) -> Result<&'a str, String> {
    if length > limit || (length > 0 && pointer.is_null()) {
        return Err("Invalid coordinate input buffer".into());
    }
    if length == 0 {
        return Ok("");
    }
    std::str::from_utf8(unsafe { std::slice::from_raw_parts(pointer, length) })
        .map_err(|_| "Coordinate input must be UTF-8".into())
}
unsafe fn context_from_abi(input: &CoordinateRequest) -> Result<Context, String> {
    if input.version != 1 || input.reserved != 0 {
        return Err("Unsupported coordinate input ABI".into());
    }
    let frame = &input.frame;
    let context = Context {
        document_owner: input.document_owner,
        frame_revision: input.frame_revision,
        document_uid: unsafe { text(input.document_uid, input.document_uid_len, MAX_UID)? }.into(),
        unit_context: unsafe { text(input.unit_context, input.unit_context_len, MAX_UNITS)? }
            .into(),
        logical_view_slot: input.logical_view_slot,
        frame: Frame::new(
            [frame[0], frame[1], frame[2]],
            [
                [frame[3], frame[4], frame[5]],
                [frame[6], frame[7], frame[8]],
                [frame[9], frame[10], frame[11]],
            ],
        )?,
    };
    context.units()?;
    Ok(context)
}
unsafe fn resolve_abi(input: &CoordinateRequest) -> Result<Resolved, String> {
    let context = unsafe { context_from_abi(input)? };
    let expected = match input.expected {
        0 => Expected::Point,
        1 => Expected::PointOrLength,
        _ => return Err("Invalid coordinate phase".into()),
    };
    let source = match input.source {
        0 => Source::Typed,
        1 => Source::WorldPick,
        2 => Source::WorldSnap,
        3 => Source::OriginShortcut,
        _ => return Err("Invalid coordinate source".into()),
    };
    let previous = match input.has_previous {
        0 => None,
        1 => Some(input.previous),
        _ => return Err("Invalid previous-point presence flag".into()),
    };
    resolve(&Request {
        context: &context,
        expected,
        source,
        previous,
        original: unsafe { text(input.original, input.original_len, MAX_TEXT)? },
        world: input.world,
    })
}
unsafe fn error_output(error: *mut u8, capacity: usize, message: &str) {
    if error.is_null() || capacity == 0 {
        return;
    }
    let count = message.len().min(capacity.min(512) - 1);
    unsafe {
        std::ptr::copy_nonoverlapping(message.as_ptr(), error, count);
        *error.add(count) = 0;
    }
}
unsafe fn write_report(output: *mut u8, capacity: usize, value: &str) -> Result<usize, String> {
    let required = value.len() + 1;
    if required > MAX_REPORT_BYTES {
        return Err("Coordinate report exceeds the output budget".into());
    }
    if output.is_null() && capacity == 0 {
        return Ok(required);
    }
    if output.is_null() || capacity < required || capacity > MAX_REPORT_BYTES {
        return Err("Coordinate report buffer is invalid or too small".into());
    }
    unsafe {
        std::ptr::copy_nonoverlapping(value.as_ptr(), output, value.len());
        *output.add(value.len()) = 0;
    }
    Ok(required)
}

/// Resolve one input. Return report bytes including trailing NUL, or zero with
/// error text. A null report with capacity zero probes size without changing
/// value. Failure (including short buffers) leaves value/report untouched.
///
/// # Safety
/// Request/value pointers must be aligned and initialized/writable respectively;
/// string slices and output capacities describe valid, mutually disjoint memory
/// for this call. Null request rejects; null value is allowed. Error may be null;
/// at most min(error_capacity,512) bytes are written, always terminated. Allocation
/// failure follows Rust's process allocation policy. No pointer is retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_coordinate_resolve(
    input: *const CoordinateRequest,
    value: *mut CoordinateValue,
    output: *mut u8,
    capacity: usize,
    error: *mut u8,
    error_capacity: usize,
) -> usize {
    let run = || -> Result<usize, String> {
        let input = unsafe { input.as_ref() }.ok_or("Missing coordinate input")?;
        let resolved = unsafe { resolve_abi(input)? };
        let result = match resolved.value {
            Value::Point(point) => CoordinateValue {
                kind: 0,
                reserved: 0,
                point,
                length_mm: 0.,
            },
            Value::Length(length_mm) => CoordinateValue {
                kind: 1,
                reserved: 0,
                point: [0.; 3],
                length_mm,
            },
        };
        let required = unsafe { write_report(output, capacity, &resolved.report())? };
        if !output.is_null() && !value.is_null() {
            unsafe {
                *value = result;
            }
        }
        Ok(required)
    };
    match run() {
        Ok(required) => {
            unsafe {
                error_output(error, error_capacity, "");
            }
            required
        }
        Err(message) => {
            unsafe {
                error_output(error, error_capacity, &message);
            }
            0
        }
    }
}

/// Re-resolve native request values and serialize a combined report in Rust.
/// Maximum 64 requests, all from one command context; native JSON is not trusted.
/// Size probe and output/error atomicity match om9_coordinate_resolve.
///
/// # Safety
/// Inputs is an aligned initialized array of count requests (null only for zero).
/// All nested buffers and disjoint output/error ranges remain valid for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_coordinate_report(
    inputs: *const CoordinateRequest,
    count: usize,
    output: *mut u8,
    capacity: usize,
    error: *mut u8,
    error_capacity: usize,
) -> usize {
    let run = || -> Result<usize, String> {
        if count > MAX_INPUTS || (count > 0 && inputs.is_null()) {
            return Err("Invalid coordinate report input count".into());
        }
        let inputs = if count == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(inputs, count) }
        };
        let resolved = inputs
            .iter()
            .map(|input| unsafe { resolve_abi(input) })
            .collect::<Result<Vec<_>, _>>()?;
        unsafe { write_report(output, capacity, &report(&resolved)?) }
    };
    match run() {
        Ok(required) => {
            unsafe {
                error_output(error, error_capacity, "");
            }
            required
        }
        Err(message) => {
            unsafe {
                error_output(error, error_capacity, &message);
            }
            0
        }
    }
}

/// Validate snapshot against current owner/UID/units. The frame is start-latched.
///
/// # Safety
/// Input and borrowed UTF-8 slices follow CoordinateRequest's bounds; current UID
/// <=256 and units <=512 bytes. Error output follows resolve's contract.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_coordinate_verify_context(
    input: *const CoordinateRequest,
    current_owner: u64,
    current_uid: *const u8,
    current_uid_len: usize,
    current_units: *const u8,
    current_units_len: usize,
    error: *mut u8,
    error_capacity: usize,
) -> bool {
    let run = || -> Result<(), String> {
        let input = unsafe { input.as_ref() }.ok_or("Missing coordinate context")?;
        let context = unsafe { context_from_abi(input)? };
        context.verify(
            current_owner,
            unsafe { text(current_uid, current_uid_len, MAX_UID)? },
            unsafe { text(current_units, current_units_len, MAX_UNITS)? },
        )
    };
    match run() {
        Ok(()) => {
            unsafe {
                error_output(error, error_capacity, "");
            }
            true
        }
        Err(message) => {
            unsafe {
                error_output(error, error_capacity, &message);
            }
            false
        }
    }
}
