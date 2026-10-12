//! Reuses the basis ABI live/initialized immutable buffer contract. Output must
//! be writable bytes, contained in one allocation, nonoverlapping with inputs
//! and live for this call; null+zero requests only the exact required size.
//! Each call validates/copies/serializes into Rust-owned values. No pointers or
//! allocator ownership escape; copying into the native buffer is the final step.
//! Native exceptions stay in C++; crate release panic/OOM policy is abort.
use crate::{support_snapshot as model,nurbs_basis as basis,nurbs_basis_ffi as native};
use crate::analytic_support::{Analytic,AnalyticInput};
#[repr(C)]
#[derive(Clone,Copy,Debug,Default)]
pub struct JsonSummary {pub status:u32,pub reserved:u32,pub needed:usize}
pub(crate) fn limits()->native::Limits {
    let p=model::policy();native::Limits{max_degree:p.max_degree,max_poles:p.max_poles,max_knots:p.max_knots,max_bytes:p.max_bytes,max_coordinate:p.max_coordinate}
}
fn error(status:u32)->JsonSummary {JsonSummary{status,..Default::default()}}
pub(crate) fn output_valid(output:*mut u8,capacity:usize)->bool {
    if output.is_null(){return capacity==0;}
    capacity<=model::MAX_SERIALIZED_BYTES && (output as usize).checked_add(capacity).is_some()
}
pub(crate) unsafe fn copy_json(json:Result<String,model::SerializedLimit>,output:*mut u8,capacity:usize)->JsonSummary {
    let json=match json {Ok(j)=>j,Err(_)=>return error(1000)};
    let needed=json.len();
    if output.is_null(){return JsonSummary{status:0,reserved:0,needed};}
    if capacity<needed{return JsonSummary{status:1001,reserved:0,needed};}
    // SAFETY: validated envelope; caller supplies writable nonoverlapping storage.
    unsafe{std::ptr::copy_nonoverlapping(json.as_ptr(),output,needed)};
    JsonSummary{status:0,reserved:0,needed}
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_support_counts(u:usize,v:usize,uk:usize,vk:usize,surface:u32)->u32 {
    u32::from(surface>1 || !model::support_counts_allowed(u,v,uk,vk,surface==1))
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_support_payload(used:usize,additional:usize)->u32 {
    u32::from(!model::serialized_payload_allowed(used,additional))
}
/// # Safety
/// Value input has no pointers; output uses the same writable/nonoverlap contract.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_support_analytic_json(input:AnalyticInput,output:*mut u8,capacity:usize)->JsonSummary {
    if !output_valid(output,capacity){return error(1002);}
    let owned=match Analytic::capture(input){Ok(v)=>v,Err(_)=>return error(2000)};
    unsafe{copy_json(owned.json(),output,capacity)}
}
/// # Safety
/// Requires the borrowed-buffer and output contract above. Status1..17 retain
/// the basis ABI errors;1000 serialized budget,1001 short output,1002 bad output.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_support_curve_json(c:native::NativeCurve,output:*mut u8,capacity:usize)->JsonSummary {
    if !output_valid(output,capacity){return error(1002);}
    // SAFETY: shared validator rejects count/address flags before borrowing.
    let status=unsafe{native::om9_basis_curve_validate(c,limits())};
    if status!=0{return error(status);}
    let b=basis::CurveBasis {
        // SAFETY: accepted counts/envelopes and live immutable caller buffers.
        poles:unsafe{std::slice::from_raw_parts(c.poles,c.pole_count)},
        weights:unsafe{native::borrowed_weights(c.weights,c.weight_count)},
        axis:unsafe{native::borrowed_axis(c.axis)},
    };
    let owned=match model::Curve::capture(b){Ok(s)=>s,Err(_)=>return error(1)};
    // SAFETY: final output copy only after successful validation/serialization.
    unsafe{copy_json(owned.json(),output,capacity)}
}
/// # Safety
/// Same obligations as curve JSON; surface pole/weight buffers use U-major order.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_support_surface_json(s:native::NativeSurface,output:*mut u8,capacity:usize)->JsonSummary {
    if !output_valid(output,capacity){return error(1002);}
    let status=unsafe{native::om9_basis_surface_validate(s,limits())};
    if status!=0{return error(status);}
    let b=basis::SurfaceBasis {
        // SAFETY: shared validator accepted all dimensions/envelopes; caller owns lifetime.
        poles:unsafe{std::slice::from_raw_parts(s.poles,s.pole_count)},
        weights:unsafe{native::borrowed_weights(s.weights,s.weight_count)},
        u_count:s.u_count,v_count:s.v_count,
        axes:unsafe{[native::borrowed_axis(s.axes[0]),native::borrowed_axis(s.axes[1])]},
    };
    let owned=match model::Surface::capture(b){Ok(v)=>v,Err(_)=>return error(1)};
    unsafe{copy_json(owned.json(),output,capacity)}
}
/// # Safety
/// Same initialized immutable basis buffers and writable output contract as
/// curve JSON. The canonical Bezier basis is checked before owned cloning.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_support_bezier_curve_json(c:native::NativeCurve,output:*mut u8,capacity:usize)->JsonSummary {
    if !output_valid(output,capacity){return error(1002);}
    let status=unsafe{native::om9_basis_curve_validate(c,limits())};
    if status!=0{return error(status);}
    // SAFETY: accepted envelope and live caller buffers under the ABI contract.
    let b=basis::CurveBasis{poles:unsafe{std::slice::from_raw_parts(c.poles,c.pole_count)},
        weights:unsafe{native::borrowed_weights(c.weights,c.weight_count)},axis:unsafe{native::borrowed_axis(c.axis)}};
    let owned=match model::BezierCurve::capture(b){Ok(v)=>v,Err(_)=>return error(3000)};
    unsafe{copy_json(owned.json(),output,capacity)}
}
/// # Safety
/// Same obligations as surface JSON, including U-major control storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_support_bezier_surface_json(s:native::NativeSurface,output:*mut u8,capacity:usize)->JsonSummary {
    if !output_valid(output,capacity){return error(1002);}
    let status=unsafe{native::om9_basis_surface_validate(s,limits())};
    if status!=0{return error(status);}
    // SAFETY: shared validator accepted all buffers; caller supplies lifetime.
    let b=basis::SurfaceBasis{poles:unsafe{std::slice::from_raw_parts(s.poles,s.pole_count)},
        weights:unsafe{native::borrowed_weights(s.weights,s.weight_count)},u_count:s.u_count,v_count:s.v_count,
        axes:unsafe{[native::borrowed_axis(s.axes[0]),native::borrowed_axis(s.axes[1])]}};
    let owned=match model::BezierSurface::capture(b){Ok(v)=>v,Err(_)=>return error(3000)};
    unsafe{copy_json(owned.json(),output,capacity)}
}
