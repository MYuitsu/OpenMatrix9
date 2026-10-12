//! Owned native supporting chains. Parameters are retained without conversion;
//! capture does not certify offset regularity or a global trim parameter map.
use crate::{analytic_support::{Analytic,AnalyticInput},nurbs_basis::{CurveBasis,SurfaceBasis},support_snapshot::{self as support,Json,SerializedLimit,numbers}};
use std::fmt::Write;
pub const MAX_DEPTH:usize=16;
/// Root-to-leaf wrappers. Infinity bits mark bounds[0..4]; the corresponding
/// numeric slot must be zero. Lower/upper infinity signs follow endpoint position.
#[repr(C)]
#[derive(Clone,Copy,Debug)]
pub struct DerivedNode {
    pub kind:u32,pub dimension:u32,pub direction:[f64;3],pub origin:[f64;3],
    pub offset:f64,pub bounds:[f64;4],pub infinite_mask:u32,pub reserved:u32,
}
pub enum LeafInput<'a> {
    Analytic(AnalyticInput),NurbsCurve(CurveBasis<'a>),NurbsSurface(SurfaceBasis<'a>),
    BezierCurve(CurveBasis<'a>),BezierSurface(SurfaceBasis<'a>),
}
enum Leaf {Analytic(Analytic),NurbsCurve(support::Curve),NurbsSurface(support::Surface),BezierCurve(support::BezierCurve),BezierSurface(support::BezierSurface)}
pub struct Tree {nodes:Vec<DerivedNode>,leaf:Leaf}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Error {Depth,Dimension,Category,Parameters,Domain,Leaf}
pub fn depth_allowed(count:usize)->bool {(1..=MAX_DEPTH).contains(&count)}
/// Validate the entire chain before allocating/cloning basis data. The native
/// caller may use this on borrowed node values before touching leaf buffers.
pub fn validate_chain(surface:bool,dimension:u32,nodes:&[DerivedNode])->Result<(),Error> {
    if !depth_allowed(nodes.len()){return Err(Error::Depth);}
    if !matches!(dimension,2|3) || (surface && dimension!=3){return Err(Error::Dimension);}
    let mut current_surface=surface;
    for n in nodes.iter().rev() {
        if n.dimension!=dimension{return Err(Error::Dimension);}
        if n.reserved!=0 || n.infinite_mask>15 || n.direction.into_iter().chain(n.origin).chain([n.offset]).chain(n.bounds).any(|v|!v.is_finite() || v.abs()>support::policy().max_coordinate){return Err(Error::Parameters);}
        let unit=(n.direction.iter().map(|v|v*v).sum::<f64>()-1.).abs()<=1e-10;
        let zero_dir=n.direction==[0.;3];let zero_origin=n.origin==[0.;3];
        match n.kind {
            1 if !current_surface => {if !zero_origin || (dimension==3 && !unit) || (dimension==2 && !zero_dir){return Err(Error::Parameters);}},
            2 if current_surface => {if !zero_dir || !zero_origin{return Err(Error::Parameters);}},
            3|4 if !current_surface && dimension==3 => {
                if !unit || n.offset!=0. || (n.kind==3 && !zero_origin){return Err(Error::Parameters);}
                current_surface=true;
            },
            5 if !current_surface => {if !zero_dir || !zero_origin || n.offset!=0. || n.infinite_mask!=0{return Err(Error::Parameters);}},
            6 if current_surface => {if !zero_dir || !zero_origin || n.offset!=0.{return Err(Error::Parameters);}},
            _=>return Err(Error::Category),
        }
        let axes=if current_surface{2}else{1};
        if axes==1 && (n.bounds[2..]!=[0.,0.] || n.infinite_mask&12!=0){return Err(Error::Domain);}
        for a in 0..axes {
            let lo=a*2;let hi=lo+1;
            let l=n.infinite_mask&(1<<lo)!=0;let h=n.infinite_mask&(1<<hi)!=0;
            if (l && n.bounds[lo]!=0.) || (h && n.bounds[hi]!=0.) || (!l && !h && n.bounds[lo]>=n.bounds[hi]){return Err(Error::Domain);}
        }
    }
    Ok(())
}
impl Tree {
    /// Native-local curve value and D1, retaining every wrapper's absolute
    /// parameter interval. Surface/extrusion/revolution chains remain distinct.
    pub fn evaluate_curve(&self,parameter:f64)->Result<([f64;3],[f64;3]),support::EvaluationError>{
        let c=self.taylor_curve(parameter,1)?;Ok((c[0],c[1]))
    }
    pub(crate) fn taylor_curve(&self,parameter:f64,order:usize)->Result<Vec<[f64;3]>,support::EvaluationError>{
        use support::EvaluationError;
        if !parameter.is_finite(){return Err(EvaluationError::Parameter);}
        let mut needed=order;
        for n in &self.nodes{
            if !matches!(n.kind,1|5){return Err(EvaluationError::Representation);}
            if n.infinite_mask&1==0&&parameter<n.bounds[0]||n.infinite_mask&2==0&&parameter>n.bounds[1]{return Err(EvaluationError::Parameter);}
            if n.kind==1{needed=needed.checked_add(1).ok_or(EvaluationError::Parameter)?;}
        }
        crate::curve_taylor::order_allowed(needed)?;
        let mut coefficients=match &self.leaf{
            Leaf::Analytic(v)=>v.taylor_curve(parameter,needed)?,
            Leaf::NurbsCurve(v)=>v.taylor(parameter,needed)?,
            Leaf::BezierCurve(v)=>v.taylor(parameter,needed)?,
            _=>return Err(EvaluationError::Representation),
        };
        for n in self.nodes.iter().rev(){
            if n.kind==5{continue;}
            let count=coefficients.len()-1;
            let normals:Vec<[f64;3]>=(0..count).map(|k|{
                let t=coefficients[k+1].map(|v|v*(k+1)as f64);
                if n.dimension==2{[t[1],-t[0],0.]}else{
                    let d=n.direction;[t[1]*d[2]-t[2]*d[1],t[2]*d[0]-t[0]*d[2],t[0]*d[1]-t[1]*d[0]]
                }
            }).collect();
            let normal=crate::curve_taylor::normalize(&normals)?;
            coefficients.truncate(count);
            for(k,c)in coefficients.iter_mut().enumerate(){for j in 0..3{c[j]+=n.offset*normal[k][j];}}
            crate::curve_taylor::finite(&coefficients)?;
        }
        Ok(coefficients)
    }
    pub fn capture(input:LeafInput<'_>,dimension:u32,nodes:&[DerivedNode])->Result<Self,Error> {
        let surface=match &input {LeafInput::Analytic(i)=>i.kind>=6,LeafInput::NurbsSurface(_)|LeafInput::BezierSurface(_)=>true,_=>false};
        validate_chain(surface,dimension,nodes)?;
        let check_curve=|b:&CurveBasis<'_>|if dimension==2 && b.poles.iter().any(|p|p[2]!=0.){Err(Error::Dimension)}else{Ok(())};
        let leaf=match input {
            LeafInput::Analytic(i)=>{if i.dimension!=dimension{return Err(Error::Dimension);}Leaf::Analytic(Analytic::capture(i).map_err(|_|Error::Leaf)?)},
            LeafInput::NurbsCurve(b)=>{check_curve(&b)?;Leaf::NurbsCurve(support::Curve::capture(b).map_err(|_|Error::Leaf)?)},
            LeafInput::BezierCurve(b)=>{check_curve(&b)?;Leaf::BezierCurve(support::BezierCurve::capture(b).map_err(|_|Error::Leaf)?)},
            LeafInput::NurbsSurface(b)=>Leaf::NurbsSurface(support::Surface::capture(b).map_err(|_|Error::Leaf)?),
            LeafInput::BezierSurface(b)=>Leaf::BezierSurface(support::BezierSurface::capture(b).map_err(|_|Error::Leaf)?),
        };
        Ok(Self{nodes:nodes.to_vec(),leaf})
    }
    pub fn json(&self)->Result<String,SerializedLimit> {
        let leaf=match &self.leaf {Leaf::Analytic(v)=>v.json(),Leaf::NurbsCurve(v)=>v.json(),Leaf::NurbsSurface(v)=>v.json(),Leaf::BezierCurve(v)=>v.json(),Leaf::BezierSurface(v)=>v.json()}?;
        let mut j=Json(String::new());
        let names=["offset_curve","offset_surface","linear_extrusion","revolution","trimmed_curve","trimmed_surface"];
        let result=(||{
            for n in &self.nodes {
                write!(j,"{{\"schema_version\":1,\"kind\":\"{}\",\"dimension\":{},\"parameter_convention\":\"occt_native\"",names[(n.kind-1) as usize],n.dimension)?;
                if matches!(n.kind,1|2){write!(j,",\"offset\":{}",n.offset)?;}
                if matches!(n.kind,1|3|4) && n.dimension==3{j.write_str(",\"direction_xyz\":")?;numbers(&mut j,&n.direction)?;}
                if n.kind==4{j.write_str(",\"origin_xyz\":")?;numbers(&mut j,&n.origin)?;}
                j.write_str(",\"natural_domains\":[")?;
                let axes=if matches!(n.kind,1|5){1}else{2};
                for a in 0..axes {
                    if a!=0{j.write_char(',')?;}
                    let lo=a*2;let hi=lo+1;let l=n.infinite_mask&(1<<lo)!=0;let h=n.infinite_mask&(1<<hi)!=0;
                    if l && h {j.write_str("\"unbounded\"")?;} else {
                        j.write_char('[')?;
                        if l{j.write_str("\"-infinity\"")?;}else{write!(j,"{}",n.bounds[lo])?;}
                        j.write_char(',')?;
                        if h{j.write_str("\"infinity\"")?;}else{write!(j,"{}",n.bounds[hi])?;}
                        j.write_char(']')?;
                    }
                }
                j.write_str("],\"basis\":")?;
            }
            j.write_str(&leaf)?;
            for _ in &self.nodes {j.write_char('}')?;}
            Ok::<_,std::fmt::Error>(())
        })();result.map(|_|j.0).map_err(|_|SerializedLimit)
    }
}
