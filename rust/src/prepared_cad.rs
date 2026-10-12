// SPDX-License-Identifier: LGPL-2.1-or-later
//! Admission of prepared native CAD, distinct from read-only inspection.
use crate::geometry_contract::{operation_capability,ConversionPolicy,OperationId,
    OperationOptions,PreservationPolicy,RepresentationKind};
use crate::native_input_tree::{MAX_BYTES,MAX_NODES};
pub const MAX_METADATA_NODES:usize=4_000_000;
/// Shared native result validity only. The caller must separately admit its
/// actual operation/type/options; this does not select an import backend or
/// certify conversion, parameterization, source lifetime or attributes.
#[unsafe(no_mangle)]
pub extern "C" fn om9_native_cad_result_admit(root:u32,valid:u32,solid:u32)->u32 {
    if root>7 || valid>2 || solid>2 {return 1;}
    if valid==0 {return 7;} // KernelFailure: absence is not permission.
    if valid==1 || (root==2 && solid!=2) {return 3;} // InvalidTrim/topology.
    0
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_prepared_cad_admit(root:u32,valid:u32,solid:u32)->u32 {
    let kind=match root {0|1=>RepresentationKind::Brep,2=>RepresentationKind::Solid,
        3=>RepresentationKind::Shell,4=>RepresentationKind::TrimmedFace,
        5=>RepresentationKind::Polycurve,6=>RepresentationKind::Curve,
        7=>RepresentationKind::Point,_=>return 1};
    if valid>2 || solid>2 {return 1;}
    let options=OperationOptions::WorkingExchange{archive_version:5,
        conversion:ConversionPolicy::ExistingConverters,
        preservation:PreservationPolicy::ReportObservedFacts};
    if operation_capability(OperationId::ImportWorkingV5,kind,options).is_err(){return 1;}
    om9_native_cad_result_admit(root,valid,solid)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_prepared_cad_budget(meta_nodes:usize,meta_bytes:usize,native_nodes:usize,native_bytes:usize)->u32 {
    if meta_nodes>MAX_METADATA_NODES || native_nodes>MAX_NODES ||
        meta_bytes.checked_add(native_bytes).is_none_or(|v|v>MAX_BYTES) {2}else{0}
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_prepared_metadata(nodes:usize,bytes:usize,depth:usize)->u32 {
    if depth>64 || nodes>MAX_METADATA_NODES || bytes>MAX_BYTES {2}else{0}
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_prepared_number(value:f64)->u32 {if value.is_finite(){0}else{4}}
/// Exact native scalar facts only; no Python object or callback enters Rust.
#[unsafe(no_mangle)]
pub extern "C" fn om9_prepared_binding_field(field:u32,kind:u32,value:f64,count:usize)->u32 {
    // kind: missing0, Unicode1, float2, bool3, sequence4, other5, signed integer6.
    match field {
        0|1|8 => if kind==1 {0}else{1}, // name/layer/other host string
        3|4 => if kind==3 {0}else{1}, // locked/visible
        5 => if kind==4 && matches!(count,3|4) {0}else{1}, // color arity
        2|6 => {
            if !matches!(kind,2|6) {return 1;}
            if !value.is_finite() {return 4;}
            if (field==2 && value<=0.) || (field==6 && !(0.0..=255.).contains(&value)) {2}else{0}
        }
        7 => if kind!=6 {1}else if !value.is_finite() || value.fract()!=0. ||
            !(f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(&value) {2}else{0},
        9 => { // Native mesh segment index against its bounded facet table.
            if kind!=6{return 1;}if count>crate::discrete_snapshot::MAX_FACES{return 2;}
            if !value.is_finite(){return 4;}
            if value<0.||value.fract()!=0.||value>=count as f64{3}else{0}
        }
        _=>1,
    }
}
