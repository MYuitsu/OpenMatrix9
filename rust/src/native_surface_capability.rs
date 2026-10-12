// SPDX-License-Identifier: LGPL-2.1-or-later
//! Request policy for existing native Sweep1/Sweep2/Loft adapters.
use crate::geometry_contract::*;
use crate::surface::Kind;

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Stage {
    Declare,Capture,Construct,
    /// Saved scalar/cardinality admission before native rails are read. Stored
    /// closure hints are retained but only Construct verifies actual closure.
    Saved,
}

/// Owned portable values. Native shape/source observations remain adapters.
#[derive(Clone,Debug,PartialEq)]
pub struct Request {
    pub kind:Kind,
    pub stage:Stage,
    pub input_count:usize,
    pub rail_closed:[bool;2],
    pub style:u32,
    pub frenet:bool,
    pub closed:bool,
    pub maintain_height:bool,
    pub section_mode:u32,
    pub point_count:u32,
    pub tolerance:f64,
    pub continuity_a:u32,
    pub continuity_b:u32,
    pub match_start:bool,
    pub match_end:bool,
    pub history:bool,
    pub history_command:bool,
    pub preview:bool,
    pub slashes:Vec<(f64,f64)>,
}
impl Request {
    pub fn new(kind:Kind,stage:Stage,input_count:usize)->Self{
        Self {kind,stage,input_count,rail_closed:[false,false],style:0,frenet:false,
            closed:false,maintain_height:false,section_mode:0,point_count:16,
            tolerance:0.01,continuity_a:0,continuity_b:0,match_start:false,
            match_end:false,history:false,history_command:false,preview:true,slashes:Vec::new()}
    }
}

pub(crate) fn policy(operation:OperationId,input:RepresentationKind,r:&Request)
    ->Result<(Backend,&'static [OperationConstraint],&'static str),CapabilityError>{
    if operation!=OperationId::EditSurface{return Err(CapabilityError::UnsupportedOperation(operation));}
    // Brep covers the existing connected curve-only Compound reader. Native
    // classification must still reject faces, disconnected edges and solids.
    if !matches!(input,RepresentationKind::Curve|RepresentationKind::Polycurve|RepresentationKind::Brep){
        return Err(CapabilityError::UnsupportedRepresentation(input));
    }
    let unsupported=||Err(CapabilityError::UnsupportedOptions);
    if !r.kind.options_valid(r.style,r.closed)||r.section_mode>2||!(2..=256).contains(&r.point_count)
        ||!r.tolerance.is_finite()||r.tolerance<=0.||r.tolerance>1e6
        ||r.continuity_a>2||r.continuity_b>2{return unsupported();}
    let construct=matches!(r.stage,Stage::Construct|Stage::Saved);
    let native_rails=r.stage!=Stage::Saved;
    let rails=r.kind.rails();
    if construct&&r.input_count<rails+r.kind.minimum_sections(){return unsupported();}
    if construct&&r.closed{
        if r.input_count<rails+if rails==0{3}else{2}{return unsupported();}
        if native_rails&&rails>0&&(!r.rail_closed[0]||(rails==2&&!r.rail_closed[1])){return unsupported();}
    }
    if r.maintain_height&&(r.kind!=Kind::Sweep2||(construct&&r.input_count!=3)){return unsupported();}
    if !r.slashes.is_empty(){
        if r.kind!=Kind::Sweep2||(construct&&r.input_count!=3)||(native_rails&&r.rail_closed.iter().any(|closed|*closed))
            ||r.continuity_a!=0||r.continuity_b!=0
            ||crate::surface::slash_parameter(&r.slashes,0.5).is_none(){return unsupported();}
    }
    let constrained=r.continuity_a!=0||r.continuity_b!=0||r.match_start||r.match_end;
    if constrained{
        if r.closed||r.section_mode!=0||r.maintain_height||!r.slashes.is_empty(){return unsupported();}
        let first=match r.kind{Kind::Sweep2=>2,Kind::Loft=>0,Kind::Sweep1=>return unsupported()};
        if construct&&(r.input_count<first+2||r.input_count>first+32){return unsupported();}
        if r.kind==Kind::Sweep2&&(r.match_start||r.match_end)
            ||r.kind==Kind::Loft&&(r.continuity_a!=0||r.continuity_b!=0||!matches!(r.style,0|3)){return unsupported();}
    }
    Ok((Backend::OcctEdit,&[OperationConstraint::ValidateSourceStamp,
        OperationConstraint::ValidateNativeInput,OperationConstraint::PrivateOwnedInput,
        OperationConstraint::ReportConversionFacts],r.kind.feature_id()))
}

/// Initialized scalar envelope. Native caller retains exclusive immutable
/// aligned storage for exactly 2*slash_count f64 values during the call.
/// Count/mask/address checks precede reads; no owner or pointer is retained.
#[repr(C)]
#[derive(Clone,Copy,Debug)]
pub struct SurfaceCapabilityInput {
    pub kind:u32,pub representation:u32,pub stage:u32,pub flags:u32,
    pub style:u32,pub section_mode:u32,pub point_count:u32,
    pub continuity_a:u32,pub continuity_b:u32,pub rail_closed:u32,
    pub input_count:usize,pub tolerance:f64,
    pub slash_pairs:*const f64,pub slash_count:usize,
}
/// # Safety
/// For a nonzero bounded count, slash_pairs must name live initialized aligned
/// immutable caller-owned f64 storage of at least 2*slash_count elements.
/// Null/misaligned/excessive-count envelopes reject without pointer access.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_native_surface_capability(input:SurfaceCapabilityInput)->u32{
    std::panic::catch_unwind(||{
        let kind=match input.kind{1=>Kind::Sweep1,2=>Kind::Sweep2,3=>Kind::Loft,_=>return 100};
        let representation=match input.representation{
            2=>RepresentationKind::Curve,3=>RepresentationKind::Polycurve,7=>RepresentationKind::Brep,_=>return 101,
        };
        let stage=match input.stage{1=>Stage::Declare,2=>Stage::Capture,3=>Stage::Construct,_=>return 102};
        if input.flags&!255!=0||input.rail_closed&!3!=0||input.slash_count>64{return 102;}
        let mut r=Request::new(kind,stage,input.input_count);
        r.style=input.style;r.section_mode=input.section_mode;r.point_count=input.point_count;
        r.tolerance=input.tolerance;r.continuity_a=input.continuity_a;r.continuity_b=input.continuity_b;
        r.rail_closed=[input.rail_closed&1!=0,input.rail_closed&2!=0];
        r.frenet=input.flags&1!=0;r.closed=input.flags&2!=0;r.maintain_height=input.flags&4!=0;
        r.match_start=input.flags&8!=0;r.match_end=input.flags&16!=0;r.history=input.flags&32!=0;
        r.history_command=input.flags&64!=0;r.preview=input.flags&128!=0;
        if input.slash_count!=0{
            // Bounded scalar data only. A live allocation cannot be established
            // from an address; the native caller guarantees lifetime/storage.
            let address=input.slash_pairs as usize;
            let count=input.slash_count*2;
            let bytes=count*std::mem::size_of::<f64>();
            if address==0||address%std::mem::align_of::<f64>()!=0||address.checked_add(bytes).is_none(){return 102;}
            if r.slashes.try_reserve_exact(input.slash_count).is_err(){return 103;}
            // SAFETY: caller's documented initialized, aligned, immutable
            // allocation remains live; checked count is <=128 scalar values.
            let pairs=unsafe{std::slice::from_raw_parts(input.slash_pairs,count)};
            for pair in pairs.chunks_exact(2){r.slashes.push((pair[0],pair[1]));}
        }
        match operation_capability(OperationId::EditSurface,representation,OperationOptions::NativeSurface(r)){
            Ok(capability) if capability.backend==Backend::OcctEdit=>4,
            Ok(_)|Err(CapabilityError::UnsupportedOperation(_))=>100,
            Err(CapabilityError::UnsupportedRepresentation(_))=>101,
            Err(CapabilityError::UnsupportedOptions)=>102,
        }
    }).unwrap_or(103)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_native_surface_declared(kind:u32)->bool{
    let input=SurfaceCapabilityInput{kind,representation:2,stage:1,flags:128,style:0,
        section_mode:0,point_count:16,continuity_a:0,continuity_b:0,rail_closed:0,
        input_count:0,tolerance:0.01,slash_pairs:std::ptr::null(),slash_count:0};
    // SAFETY: zero count carries no borrowed storage and reads no pointer.
    (unsafe{om9_native_surface_capability(input)})<100
}
