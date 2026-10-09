//! Borrowed, synchronous native boundary for the shared basis validator.
//!
//! All nonempty buffers must be initialized, aligned, immutable, nonoverlapping
//! with concurrent writes, contained in one live allocation and readable for the
//! entire call. Address checks cannot prove allocation lifetime or accessibility.
//! No pointers are retained, no native objects are accessed, and validation does
//! not allocate. Independent immutable buffers may be validated concurrently.
//! Status 0 is valid, 1 malformed ABI, 2..17 typed basis errors. No exceptions or
//! intentional panics cross this ABI; release builds retain the crate's panic=abort
//! policy. Caller ownership and native exception handling remain native duties.
use crate::nurbs_basis::{
    Axis, BasisError, BasisPolicy, CurveBasis, SurfaceBasis, preflight_curve_counts,
    preflight_surface_counts,
};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Limits {
    pub max_degree: usize,
    pub max_poles: usize,
    pub max_knots: usize,
    pub max_bytes: usize,
    pub max_coordinate: f64,
}
/// OM9 native conversion limits, distinct from generated spline limits.
#[unsafe(no_mangle)]
pub extern "C" fn om9_basis_native_limits() -> Limits {
    Limits {
        max_degree: 25,
        max_poles: 1_000_000,
        max_knots: 1_000_000,
        max_bytes: 64 * 1024 * 1024,
        max_coordinate: 1e150,
    }
}
impl From<Limits> for BasisPolicy {
    fn from(p: Limits) -> Self {
        Self {
            max_degree: p.max_degree,
            max_poles: p.max_poles,
            max_knots: p.max_knots,
            max_bytes: p.max_bytes,
            max_coordinate: p.max_coordinate,
        }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct NativeAxis {
    pub degree: usize,
    pub knots: *const f64,
    pub knot_count: usize,
    pub multiplicities: *const usize,
    pub multiplicity_count: usize,
    pub periodic: u32,
    pub domain: [f64; 2],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct NativeCurve {
    pub poles: *const [f64; 3],
    pub pole_count: usize,
    pub weights: *const f64,
    pub weight_count: usize,
    pub axis: NativeAxis,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct NativeSurface {
    pub poles: *const [f64; 3],
    pub pole_count: usize,
    pub weights: *const f64,
    pub weight_count: usize,
    pub u_count: usize,
    pub v_count: usize,
    pub axes: [NativeAxis; 2],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SurfaceCounts {
    pub degrees: [usize; 2],
    pub u_count: usize,
    pub v_count: usize,
    pub pole_count: usize,
    pub weight_count: usize,
    pub knot_counts: [usize; 2],
}
fn status(result: Result<(), BasisError>) -> u32 {
    match result {
        Ok(()) => 0,
        Err(e) => match e {
            BasisError::InvalidPolicy => 2,
            BasisError::Degree => 3,
            BasisError::PoleCount => 4,
            BasisError::PoleLimit => 5,
            BasisError::Knots => 6,
            BasisError::KnotLimit => 7,
            BasisError::GridDimensions => 8,
            BasisError::GridCountMismatch => 9,
            BasisError::CountOverflow => 10,
            BasisError::ByteLimit => 11,
            BasisError::Poles => 12,
            BasisError::Weights => 13,
            BasisError::Multiplicity => 14,
            BasisError::PeriodicCardinality => 15,
            BasisError::Cardinality => 16,
            BasisError::Domain => 17,
        },
    }
}
fn weights_count(n: usize) -> Option<usize> {
    (n != 0).then_some(n)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_basis_curve_counts(
    degree: usize,
    poles: usize,
    weights: usize,
    knots: usize,
    limits: Limits,
) -> u32 {
    let p = limits.into();
    status(BasisPolicy::validate_degree(p, degree).and_then(|()| {
        preflight_curve_counts(poles, weights_count(weights), knots, knots, p).map(|_| ())
    }))
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_basis_surface_counts(c: SurfaceCounts, limits: Limits) -> u32 {
    let p = limits.into();
    status(
        BasisPolicy::validate_degree(p, c.degrees[0])
            .and_then(|()| p.validate_degree(c.degrees[1]))
            .and_then(|()| {
                preflight_surface_counts(
                    c.u_count,
                    c.v_count,
                    c.pole_count,
                    weights_count(c.weight_count),
                    c.knot_counts,
                    c.knot_counts,
                    p,
                )
                .map(|_| ())
            }),
    )
}
fn buffer_valid<T>(ptr: *const T, count: usize) -> bool {
    if count == 0 {
        return true;
    }
    let Some(bytes) = count.checked_mul(std::mem::size_of::<T>()) else {
        return false;
    };
    let addr = ptr as usize;
    !ptr.is_null()
        && addr.is_multiple_of(std::mem::align_of::<T>())
        && bytes <= isize::MAX as usize
        && addr.checked_add(bytes).is_some()
}
fn axis_buffers_valid(a: NativeAxis) -> bool {
    a.periodic <= 1
        && buffer_valid(a.knots, a.knot_count)
        && buffer_valid(a.multiplicities, a.multiplicity_count)
}
fn weights_buffer_valid(ptr: *const f64, count: usize) -> bool {
    if count == 0 {
        ptr.is_null()
    } else {
        buffer_valid(ptr, count)
    }
}
unsafe fn borrowed_axis<'a>(a: NativeAxis) -> Axis<'a> {
    // SAFETY: the caller establishes all buffer validity and lifetime conditions;
    // count and address guards ran before this function. Axes are nonempty.
    Axis {
        degree: a.degree,
        knots: unsafe { std::slice::from_raw_parts(a.knots, a.knot_count) },
        multiplicities: unsafe {
            std::slice::from_raw_parts(a.multiplicities, a.multiplicity_count)
        },
        periodic: a.periodic == 1,
        domain: a.domain,
    }
}
unsafe fn borrowed_weights<'a>(ptr: *const f64, count: usize) -> Option<&'a [f64]> {
    if count == 0 {
        None
    } else {
        // SAFETY: nonempty weight buffer was guarded and remains caller-owned.
        Some(unsafe { std::slice::from_raw_parts(ptr, count) })
    }
}
/// # Safety
/// All buffer ownership, initialization and lifetime conditions in this module's
/// contract must hold. Malformed counts, null/alignment/address flags are rejected
/// before borrowing; these checks cannot establish that a pointer is live.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_basis_curve_validate(c: NativeCurve, limits: Limits) -> u32 {
    let p: BasisPolicy = limits.into();
    let result = p.validate_degree(c.axis.degree).and_then(|()| {
        preflight_curve_counts(
            c.pole_count,
            weights_count(c.weight_count),
            c.axis.knot_count,
            c.axis.multiplicity_count,
            p,
        )
        .map(|_| ())
    });
    if result.is_err() {
        return status(result);
    }
    if !buffer_valid(c.poles, c.pole_count)
        || !weights_buffer_valid(c.weights, c.weight_count)
        || !axis_buffers_valid(c.axis)
    {
        return 1;
    }
    // SAFETY: buffers are guarded; native callers satisfy the documented lifetime.
    let basis = CurveBasis {
        poles: unsafe { std::slice::from_raw_parts(c.poles, c.pole_count) },
        weights: unsafe { borrowed_weights(c.weights, c.weight_count) },
        axis: unsafe { borrowed_axis(c.axis) },
    };
    status(basis.validate(p))
}
/// # Safety
/// Same borrowed-buffer obligations as `om9_basis_curve_validate`. The surface is
/// U-major and grid dimensions must equal the flattened pole/weight count.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_basis_surface_validate(s: NativeSurface, limits: Limits) -> u32 {
    let p: BasisPolicy = limits.into();
    let result = p
        .validate_degree(s.axes[0].degree)
        .and_then(|()| p.validate_degree(s.axes[1].degree))
        .and_then(|()| {
            preflight_surface_counts(
                s.u_count,
                s.v_count,
                s.pole_count,
                weights_count(s.weight_count),
                s.axes.map(|a| a.knot_count),
                s.axes.map(|a| a.multiplicity_count),
                p,
            )
            .map(|_| ())
        });
    if result.is_err() {
        return status(result);
    }
    if !buffer_valid(s.poles, s.pole_count)
        || !weights_buffer_valid(s.weights, s.weight_count)
        || !s.axes.into_iter().all(axis_buffers_valid)
    {
        return 1;
    }
    // SAFETY: buffers are guarded; native callers satisfy the documented lifetime.
    let basis = SurfaceBasis {
        poles: unsafe { std::slice::from_raw_parts(s.poles, s.pole_count) },
        weights: unsafe { borrowed_weights(s.weights, s.weight_count) },
        u_count: s.u_count,
        v_count: s.v_count,
        axes: unsafe { [borrowed_axis(s.axes[0]), borrowed_axis(s.axes[1])] },
    };
    status(basis.validate(p))
}
