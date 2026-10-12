//! Owned current-loop input gate; no native objects, pointers or constructors.
pub const MAX_JSON_BYTES: usize = 32 * 1024 * 1024;
#[derive(Debug, PartialEq, Eq)]
pub enum InputError {
    Json,
    Schema,
    Basis,
}
use crate::builder_history::{Json, parse_document_at_depth};
use crate::nurbs_basis::{Axis, BasisPolicy, CurveBasis, preflight_curve_counts};
use std::collections::BTreeMap;

// Same native basis ceilings; tighter existing Hatch counts are retained.
const POLICY: BasisPolicy = BasisPolicy {
    max_degree: 25,
    max_poles: 500_000,
    max_knots: 1_000_000,
    max_bytes: 64 * 1024 * 1024,
    max_coordinate: 1e150,
};
fn object(value: &Json) -> Result<&BTreeMap<String, Json>, InputError> {
    if let Json::Object(o) = value {
        Ok(o)
    } else {
        Err(InputError::Schema)
    }
}
fn array(value: &Json, min: usize, max: usize) -> Result<&[Json], InputError> {
    if let Json::Array(a) = value
        && a.len() >= min
        && a.len() <= max
    {
        Ok(a)
    } else {
        Err(InputError::Schema)
    }
}
fn field<'a>(o: &'a BTreeMap<String, Json>, key: &str) -> Result<&'a Json, InputError> {
    o.get(key).ok_or(InputError::Schema)
}
fn number(v: &Json) -> Result<f64, InputError> {
    if let Json::Number(n) = v
        && n.is_finite()
    {
        Ok(*n)
    } else {
        Err(InputError::Basis)
    }
}
fn integer(v: &Json, min: usize, max: usize) -> Result<usize, InputError> {
    let n = number(v)?;
    if n < min as f64 || n > max as f64 || n.fract() != 0. {
        Err(InputError::Basis)
    } else {
        Ok(n as usize)
    }
}
fn source_index(v: &Json) -> Result<(), InputError> {
    if matches!(v, Json::Other) {
        Ok(())
    } else {
        integer(v, 0, 16383).map(|_| ())
    }
}
#[derive(Default)]
struct Budget {
    curves: usize,
    numbers: usize,
}
impl Budget {
    fn reserve(&mut self, n: usize) -> Result<(), InputError> {
        self.numbers = self
            .numbers
            .checked_add(n)
            .filter(|n| *n <= 2_000_000)
            .ok_or(InputError::Basis)?;
        Ok(())
    }
}
fn nurbs(o: &BTreeMap<String, Json>, budget: &mut Budget) -> Result<(), InputError> {
    if integer(field(o, "dimension")?, 2, 2)? != 2 {
        return Err(InputError::Basis);
    }
    let order = integer(field(o, "order")?, 2, POLICY.max_degree + 1)?;
    let Json::Boolean(rational) = field(o, "rational")? else {
        return Err(InputError::Schema);
    };
    let cvs = array(field(o, "cvs")?, order, POLICY.max_poles)?;
    let raw_knots = array(field(o, "knots")?, order, POLICY.max_knots)?;
    let expected = cvs
        .len()
        .checked_add(order)
        .and_then(|n| n.checked_sub(2))
        .ok_or(InputError::Basis)?;
    if raw_knots.len() != expected {
        return Err(InputError::Basis);
    }
    let width = if *rational { 3 } else { 2 };
    let values = cvs
        .len()
        .checked_mul(width)
        .and_then(|n| n.checked_add(raw_knots.len()))
        .ok_or(InputError::Basis)?;
    budget.reserve(values)?;
    // Count-only shared preflight before allocating extraction vectors. The
    // distinct-knot upper bound is conservative; actual compact data follows.
    preflight_curve_counts(
        cvs.len(),
        rational.then_some(cvs.len()),
        raw_knots.len(),
        raw_knots.len(),
        POLICY,
    )
    .map_err(|_| InputError::Basis)?;
    let d = array(field(o, "domain")?, 2, 2)?;
    let domain = [number(&d[0])?, number(&d[1])?];
    if domain
        != [
            number(&raw_knots[order - 2])?,
            number(&raw_knots[cvs.len() - 1])?,
        ]
    {
        return Err(InputError::Basis);
    }
    let mut poles = Vec::with_capacity(cvs.len());
    let mut weights = Vec::with_capacity(if *rational { cvs.len() } else { 0 });
    for cv in cvs {
        let row = array(cv, width, width)?;
        let w = if *rational { number(&row[2])? } else { 1. };
        if w <= 0. {
            return Err(InputError::Basis);
        }
        poles.push([number(&row[0])? / w, number(&row[1])? / w, 0.]);
        if *rational {
            weights.push(w);
        }
    }
    let mut knots: Vec<f64> = Vec::new();
    let mut multiplicities: Vec<usize> = Vec::new();
    // openNURBS omits two outer knots. Restore duplicate endpoint observations;
    // ghost poles are validated as expanded nonperiodic data, not compact periodic.
    for v in std::iter::once(&raw_knots[0])
        .chain(raw_knots.iter())
        .chain(std::iter::once(raw_knots.last().ok_or(InputError::Basis)?))
    {
        let n = number(v)?;
        if knots.last().is_some_and(|p| n < *p) {
            return Err(InputError::Basis);
        }
        if knots.last() == Some(&n) {
            *multiplicities.last_mut().ok_or(InputError::Basis)? += 1;
        } else {
            knots.push(n);
            multiplicities.push(1);
        }
    }
    CurveBasis {
        poles: &poles,
        weights: rational.then_some(weights.as_slice()),
        axis: Axis {
            degree: order - 1,
            knots: &knots,
            multiplicities: &multiplicities,
            periodic: false,
            domain,
        },
    }
    .validate(POLICY)
    .map_err(|_| InputError::Basis)
}
fn curve(v: &Json, depth: usize, budget: &mut Budget) -> Result<(), InputError> {
    budget.curves = budget
        .curves
        .checked_add(1)
        .filter(|n| *n <= 16384)
        .ok_or(InputError::Schema)?;
    if depth >= 64 {
        return Err(InputError::Schema);
    }
    let o = object(v)?;
    let Json::String(name) = field(o, "class_name")? else {
        return Err(InputError::Schema);
    };
    match name.as_str() {
        "ON_NurbsCurve" => nurbs(o, budget),
        "ON_PolyCurve" => {
            for child in array(field(o, "segments")?, 1, 16384)? {
                curve(child, depth + 1, budget)?;
            }
            Ok(())
        }
        "ON_LineCurve" | "ON_ArcCurve" | "ON_PolylineCurve" => Ok(()),
        _ => Err(InputError::Schema),
    }
}
/// Validate all current NURBS, including later nested rows, before any native
/// archive read/copy or curve construction. Other native families/topology/source
/// ownership remain the existing C++ adapter's responsibility. Owned JSON limits
/// bound bytes/nodes, not aggregate process allocations or archive deserialization.
pub fn preflight_loops(raw: &str) -> Result<(), InputError> {
    // Native curve depth64 includes extra JSON object/array wrappers per level.
    // Keep other parser consumers' depth64 while admitting this existing range.
    let root = parse_document_at_depth(raw, MAX_JSON_BYTES, 3_000_000, 256)
        .map_err(|_| InputError::Json)?;
    let o = object(&root)?;
    if o.len() != 2 || integer(field(o, "schema_version")?, 1, 1)? != 1 {
        return Err(InputError::Schema);
    }
    let mut budget = Budget::default();
    for row in array(field(o, "loops")?, 1, 16384)? {
        let row = object(row)?;
        if row.len() != 3 {
            return Err(InputError::Schema);
        }
        integer(field(row, "type")?, 0, 1)?;
        source_index(field(row, "source_index")?)?;
        curve(field(row, "curve")?, 0, &mut budget)?;
    }
    Ok(())
}
/// # Safety
/// Nonempty bytes must be immutable, initialized and readable in one live
/// allocation for this synchronous call. No pointer is retained. Address/length
/// guards cannot prove accessibility. Native caller owns UTF8 and exceptions.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_hatch_loop_input(bytes: *const u8, length: usize) -> u32 {
    if bytes.is_null()
        || length == 0
        || length > MAX_JSON_BYTES
        || length > isize::MAX as usize
        || (bytes as usize).checked_add(length).is_none()
    {
        return 1;
    }
    let bytes = unsafe { std::slice::from_raw_parts(bytes, length) };
    let Ok(raw) = std::str::from_utf8(bytes) else {
        return 1;
    };
    match preflight_loops(raw) {
        Ok(()) => 0,
        Err(InputError::Json) => 2,
        Err(InputError::Schema) => 3,
        Err(InputError::Basis) => 4,
    }
}
