//! Single native chain call; no raw geometry handles retained. Caller supplies
//! initialized immutable aligned nodes in a single live allocation. Selected
//! basis buffers need the same lifetime only after the wrapper chain is accepted;
//! rejected wrappers do not inspect any leaf pointer, count or numeric value.
//! Output has the existing support writable/nonoverlap contract; no partial copy.
//! Native exceptions stay in C++; release panic/OOM abort applies here too.
use crate::{analytic_support::AnalyticInput,derived_support::{self as model,DerivedNode,LeafInput,Tree},nurbs_basis::{CurveBasis,SurfaceBasis},nurbs_basis_ffi as native,support_snapshot_ffi::{JsonSummary,limits,output_valid,copy_json}};
#[repr(C)]
#[derive(Clone,Copy)]
pub struct NativeSupportTree {
    pub leaf_kind:u32,pub dimension:u32,pub analytic:AnalyticInput,
    pub curve:native::NativeCurve,pub surface:native::NativeSurface,
    pub nodes:*const DerivedNode,pub node_count:usize,
}
impl NativeSupportTree {
    pub fn analytic(input:AnalyticInput,nodes:*const DerivedNode,node_count:usize)->Self {
        // SAFETY: native basis structs contain only integer/float/raw-pointer
        // fields. Zero is a valid initialized representation; unused leaf fields
        // are never dereferenced. They are not valid basis envelopes to capture.
        Self{leaf_kind:1,dimension:input.dimension,analytic:input,curve:unsafe{std::mem::zeroed()},surface:unsafe{std::mem::zeroed()},nodes,node_count}
    }
}
fn error(status:u32)->JsonSummary {JsonSummary{status,..Default::default()}}
#[unsafe(no_mangle)]
pub extern "C" fn om9_support_tree_depth(count:usize)->u32 {u32::from(!model::depth_allowed(count))}
/// # Safety
/// Requires live initialized immutable nodes when their envelope is accepted;
/// after chain acceptance, selected basis buffers must satisfy the shared basis
/// ABI contract. Output must be writable/nonoverlapping. Envelope checks cannot
/// establish allocation membership or lifetime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_support_tree_json(i:NativeSupportTree,output:*mut u8,capacity:usize)->JsonSummary {
    if !output_valid(output,capacity){return error(1002);}
    if !model::depth_allowed(i.node_count) || i.nodes.is_null() || !(i.nodes as usize).is_multiple_of(std::mem::align_of::<DerivedNode>()) || (i.nodes as usize).checked_add(i.node_count*std::mem::size_of::<DerivedNode>()).is_none(){return error(4000);}
    if !(1..=5).contains(&i.leaf_kind){return error(4001);}
    // SAFETY: bounded/aligned envelope; caller supplies initialized live storage.
    let nodes=unsafe{std::slice::from_raw_parts(i.nodes,i.node_count)};
    let surface=matches!(i.leaf_kind,3|5) || (i.leaf_kind==1 && i.analytic.kind>=6);
    if model::validate_chain(surface,i.dimension,nodes).is_err(){return error(4002);}
    let leaf=match i.leaf_kind {
        1=>LeafInput::Analytic(i.analytic),
        2|4=>{
            let c=i.curve;let status=unsafe{native::om9_basis_curve_validate(c,limits())};if status!=0{return error(status);}
            // SAFETY: shared validator accepted every buffer; caller owns lifetime.
            let b=CurveBasis{poles:unsafe{std::slice::from_raw_parts(c.poles,c.pole_count)},weights:unsafe{native::borrowed_weights(c.weights,c.weight_count)},axis:unsafe{native::borrowed_axis(c.axis)}};
            if i.leaf_kind==2{LeafInput::NurbsCurve(b)}else{LeafInput::BezierCurve(b)}
        },
        3|5=>{
            let s=i.surface;let status=unsafe{native::om9_basis_surface_validate(s,limits())};if status!=0{return error(status);}
            // SAFETY: accepted basis envelope and caller's initialized live buffers.
            let b=SurfaceBasis{poles:unsafe{std::slice::from_raw_parts(s.poles,s.pole_count)},weights:unsafe{native::borrowed_weights(s.weights,s.weight_count)},u_count:s.u_count,v_count:s.v_count,axes:unsafe{[native::borrowed_axis(s.axes[0]),native::borrowed_axis(s.axes[1])]}};
            if i.leaf_kind==3{LeafInput::NurbsSurface(b)}else{LeafInput::BezierSurface(b)}
        },_=>unreachable!(),
    };
    let tree=match Tree::capture(leaf,i.dimension,nodes){Ok(v)=>v,Err(_)=>return error(4003)};
    unsafe{copy_json(tree.json(),output,capacity)}
}
