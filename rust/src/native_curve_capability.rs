// SPDX-License-Identifier: LGPL-2.1-or-later
//! Existing Curve construction options; no host owners or new CAD solver.
use crate::{circle::Circle,ellipse::Ellipse,rectangle::Rectangle,spline::Knots};
use crate::geometry_contract::*;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Stage{Declare,Capture,Preview,Construct}
#[derive(Clone,Debug,PartialEq)]
pub enum Options{
    Line{both_sides:bool,length:Option<f64>},
    Polyline{close:bool,length:Option<f64>},
    InterpCrv{degree:usize,knots:Knots,close:bool},
    Rectangle(Rectangle),Circle(Circle),Ellipse(Ellipse),
}
impl Options{
    pub fn feature_id(&self)->&'static str{match self{
        Self::Line{..}=>"OM9-CURVE-002",Self::Polyline{..}=>"OM9-CURVE-001",
        Self::InterpCrv{..}=>"OM9-CURVE-003",Self::Rectangle(_)=>"OM9-CURVE-004",
        Self::Circle(_)=>"OM9-CURVE-005",Self::Ellipse(_)=>"OM9-CURVE-006",
    }}
}
#[derive(Clone,Debug,PartialEq)]
pub struct Request{
    pub options:Options,pub stage:Stage,pub input_count:usize,pub output_count:usize,
    pub has_plan:bool,pub mm_per_unit:f64,
}
/// Scalar synchronous query of the existing locked Rust session; no host pointer.
#[unsafe(no_mangle)]
pub extern "C" fn om9_native_curve_capability(representation:u32,stage:u32)->u32{
    std::panic::catch_unwind(||{
        let stage=match stage{1=>Stage::Declare,2=>Stage::Capture,3=>Stage::Construct,4=>Stage::Preview,_=>return 102};
        use RepresentationKind::*;
        let input=match representation{1=>Point,2=>Curve,3=>Polycurve,4=>TrimmedFace,5=>Shell,6=>Solid,
            7=>Brep,8=>Mesh,9=>PointCloud,10=>Instance,11=>Group,12=>Annotation,13=>Retained,_=>return 101};
        match crate::curve_ffi::current_construction_capability(input,stage){
            Some(Ok(c)) if c.backend==Backend::OcctEdit=>4,
            Some(Ok(_)|Err(CapabilityError::UnsupportedOperation(_)))=>100,
            Some(Err(CapabilityError::UnsupportedRepresentation(_)))=>101,
            Some(Err(CapabilityError::UnsupportedOptions))=>102,
            None=>103,
        }
    }).unwrap_or(103)
}
pub(crate) fn policy(operation:OperationId,input:RepresentationKind,r:&Request)
    ->Result<(Backend,&'static [OperationConstraint],&'static str),CapabilityError>{
    if operation!=OperationId::ConstructCurve{return Err(CapabilityError::UnsupportedOperation(operation));}
    use RepresentationKind::*;
    let allowed=input==Point||match &r.options{
        Options::Circle(c) if c.mode==crate::circle::Mode::FitPoints=>
            matches!(input,Curve|Polycurve|TrimmedFace|Shell|Solid|Brep|Mesh|PointCloud),
        Options::Circle(c) if matches!(c.mode,crate::circle::Mode::AroundCurve|crate::circle::Mode::Tangent)=>matches!(input,Curve|Polycurve),
        Options::Ellipse(e) if e.mode==crate::ellipse::Mode::AroundCurve=>matches!(input,Curve|Polycurve),
        _=>false,
    };
    if !allowed{return Err(CapabilityError::UnsupportedRepresentation(input));}
    let unsupported=||Err(CapabilityError::UnsupportedOptions);
    if !r.mm_per_unit.is_finite()||r.mm_per_unit<=0.||r.output_count>4097{return unsupported();}
    let length_valid=|n:f64|n.is_finite()&&(1e-7..=1e9).contains(&n);
    let limit=match &r.options{
        Options::Line{length,..}|Options::Polyline{length,..}=>{
            if length.is_some_and(|n|!length_valid(n)){return unsupported();}4096
        }
        Options::InterpCrv{degree,..}=>{
            if !crate::spline::interpolation_degree_valid(*degree){return unsupported();}crate::spline::MAX_POLES
        }
        Options::Rectangle(rectangle)=>{
            if rectangle.length.is_some_and(|n|!length_valid(n))
                ||rectangle.width.is_some_and(|n|!length_valid(n.abs())){return unsupported();}3
        }
        Options::Circle(circle)=>{
            if crate::spline::rebuild_options(circle.point_count,circle.degree).is_err()
                ||circle.constraints.len()>3||circle.solution.is_some_and(|n|n>=10000)
                ||circle.radius_constraint.is_some_and(|n|crate::circle::Plan::new([0.;3],[0.,0.,1.],n).is_err())
                ||(circle.mode!=crate::circle::Mode::Tangent&&(circle.point_mode||circle.from_first||circle.tangent_vertical||circle.solution.is_some())){
                return unsupported();
            }
            crate::fit_point_input::MAX_POINTS
        }
        Options::Ellipse(ellipse)=>{
            if crate::spline::rebuild_options(ellipse.point_count,ellipse.degree).is_err()||ellipse.point_count<4
                ||(ellipse.mark_foci&&ellipse.mode!=crate::ellipse::Mode::FromFoci){return unsupported();}3
        }
    };
    if r.input_count>limit{return unsupported();}
    if r.stage==Stage::Construct{
        let valid_count=match &r.options{
            Options::Line{..}=>r.output_count==2,Options::Rectangle(_)=>r.output_count==5,
            Options::InterpCrv{..}=>(2..=crate::spline::MAX_POLES).contains(&r.output_count),
            Options::Circle(_)|Options::Ellipse(_)=>(2..=129).contains(&r.output_count),
            Options::Polyline{..}=>(2..=4097).contains(&r.output_count),
        };
        if !r.has_plan||!valid_count{return unsupported();}
    }
    Ok((Backend::OcctEdit,&[OperationConstraint::ValidateSourceStamp,OperationConstraint::ValidateNativeInput,
        OperationConstraint::PrivateOwnedInput,OperationConstraint::ReportConversionFacts],r.options.feature_id()))
}
