//! Owned native NURBS support. This preserves basis values, never converts or
//! normalizes geometry and never certifies a restricted trim's parameter map.
use crate::nurbs_basis::{Axis as BorrowedAxis,BasisPolicy,BasisError,CurveBasis,SurfaceBasis};
use std::fmt::{self,Write};
#[cfg(test)]
#[path="support_evaluation_tests.rs"]
mod evaluation_tests;
pub const MAX_SERIALIZED_BYTES:usize=8*1024*1024;
pub fn policy()->BasisPolicy {
    BasisPolicy{max_degree:25,max_poles:8192,max_knots:16384,max_bytes:2*1024*1024,max_coordinate:1e150}
}
pub fn support_counts_allowed(u:usize,v:usize,uk:usize,vk:usize,surface:bool)->bool {
    let p=policy();
    if surface {
        u.checked_mul(v).is_some_and(|poles|u>=2 && v>=2 && poles<=p.max_poles && (2..=p.max_knots).contains(&uk) && (2..=p.max_knots).contains(&vk))
    } else {u>=2 && u<=p.max_poles && v==0 && (2..=p.max_knots).contains(&uk) && vk==0}
}
pub fn serialized_payload_allowed(used:usize,additional:usize)->bool {
    used.checked_add(additional).is_some_and(|n|n<=MAX_SERIALIZED_BYTES)
}
#[derive(Clone,Debug)]
struct Axis {degree:usize,knots:Vec<f64>,multiplicities:Vec<usize>,periodic:bool,domain:[f64;2],cumulative:Vec<usize>}
impl From<BorrowedAxis<'_>> for Axis {
    fn from(a:BorrowedAxis<'_>)->Self {
        // Capture validates the checked multiplicity sum before this conversion.
        let mut sum=0;let cumulative=a.multiplicities.iter().map(|m|{sum+=m;sum}).collect();
        Self{degree:a.degree,knots:a.knots.to_vec(),multiplicities:a.multiplicities.to_vec(),periodic:a.periodic,domain:a.domain,cumulative}
    }
}
#[derive(Clone,Debug)]
pub struct Curve {poles:Vec<[f64;3]>,weights:Option<Vec<f64>>,axis:Axis}
#[derive(Clone,Debug)]
pub struct Surface {poles:Vec<[f64;3]>,weights:Option<Vec<f64>>,u_count:usize,v_count:usize,axes:[Axis;2]}
/// Private wrappers ensure arbitrary multi-span NURBS cannot acquire a Bezier tag.
#[derive(Clone,Debug)]
pub struct BezierCurve(Curve);
#[derive(Clone,Debug)]
pub struct BezierSurface(Surface);
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct SerializedLimit;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum EvaluationError {Parameter,Arithmetic,Representation}
pub(crate) struct Json(pub(crate) String);
impl Write for Json {
    fn write_str(&mut self,text:&str)->fmt::Result {
        if !serialized_payload_allowed(self.0.len(),text.len()){return Err(fmt::Error);}
        self.0.push_str(text);Ok(())
    }
}
pub(crate) fn numbers<T:fmt::Display>(j:&mut Json,values:&[T])->fmt::Result {
    j.write_char('[')?;
    for (i,value) in values.iter().enumerate(){if i!=0{j.write_char(',')?;}write!(j,"{value}")?;}
    j.write_char(']')
}
fn axis(j:&mut Json,a:&Axis)->fmt::Result {
    write!(j,"{{\"degree\":{},\"knots\":",a.degree)?;numbers(j,&a.knots)?;
    j.write_str(",\"multiplicities\":")?;numbers(j,&a.multiplicities)?;
    write!(j,",\"periodic\":{},\"full_domain\":",a.periodic)?;numbers(j,&a.domain)?;j.write_char('}')
}
fn geometry(j:&mut Json,poles:&[[f64;3]],weights:&Option<Vec<f64>>)->fmt::Result {
    j.write_str("\"poles_xyz\":[")?;
    for (i,pole) in poles.iter().enumerate(){if i!=0{j.write_char(',')?;}numbers(j,pole)?;}
    j.write_str("],\"weights\":")?;
    match weights {Some(w)=>numbers(j,w),None=>j.write_str("null")}
}
impl Curve {
    pub fn capture(b:CurveBasis<'_>)->Result<Self,BasisError> {
        b.validate(policy())?;
        Ok(Self{poles:b.poles.to_vec(),weights:b.weights.map(<[f64]>::to_vec),axis:b.axis.into()})
    }
    /// Native-local value and D1 at an absolute OCCT parameter. Compact periodic
    /// basis is retained; evaluation uses only degree+1 temporary control values.
    /// An edge's restriction and placement must be applied separately by callers.
    pub fn evaluate(&self,parameter:f64)->Result<([f64;3],[f64;3]),EvaluationError>{
        let coefficients=self.taylor(parameter,1)?;Ok((coefficients[0],coefficients[1]))
    }
    /// Dk/k! in the retained parameter convention. The order cap includes the
    /// derivatives required by the maximum supported nested offset depth.
    pub(crate) fn taylor(&self,parameter:f64,order:usize)->Result<Vec<[f64;3]>,EvaluationError>{
        crate::curve_taylor::order_allowed(order)?;
        if !parameter.is_finite()||parameter<self.axis.domain[0]||parameter>self.axis.domain[1]{return Err(EvaluationError::Parameter);}
        let axis=&self.axis;let degree=axis.degree;let count=self.poles.len();
        let last=axis.knots.len()-1;
        // At the terminal seam OCCT uses the left derivative; wrapping the query
        // to the first parameter would change D1 on a C0 periodic seam.
        let span=if axis.periodic&&parameter==axis.knots[last]{count-1}else{
            let distinct=axis.knots.partition_point(|u|*u<=parameter).checked_sub(1).ok_or(EvaluationError::Arithmetic)?;
            let span=axis.cumulative[distinct]-1;
            if axis.periodic{span}else{span.min(count-1)}
        };
        let knot=|index:isize|->Result<f64,EvaluationError>{
            let(cycle,index)=if axis.periodic{(index.div_euclid(count as isize),index.rem_euclid(count as isize)as usize)}else{
                (0,usize::try_from(index).map_err(|_|EvaluationError::Arithmetic)?)
            };
            let distinct=axis.cumulative.partition_point(|end|*end<=index);
            let value=*axis.knots.get(distinct).ok_or(EvaluationError::Arithmetic)?+cycle as f64*(axis.knots[last]-axis.knots[0]);
            if !value.is_finite(){return Err(EvaluationError::Arithmetic);}Ok(value)
        };
        let indices=(0..=degree).map(|j|{
            let index=span as isize-degree as isize+j as isize;
            if axis.periodic{Ok((index+degree as isize+1-axis.multiplicities[0]as isize).rem_euclid(count as isize)as usize)}
            else{usize::try_from(index).map_err(|_|EvaluationError::Arithmetic)}
        }).collect::<Result<Vec<_>,_>>()?;
        let weight=|i:usize|self.weights.as_ref().map_or(1.,|w|w[i]);
        let scale=indices.iter().map(|i|weight(*i)).fold(0.,f64::max);
        let mut values=indices.iter().map(|i|{
            let pole=self.poles.get(*i).ok_or(EvaluationError::Arithmetic)?;let w=weight(*i)/scale;
            let mut coefficients=vec![[0.;4];order+1];coefficients[0]=[pole[0]*w,pole[1]*w,pole[2]*w,w];Ok(coefficients)
        }).collect::<Result<Vec<_>,EvaluationError>>()?;
        for level in 1..=degree{
            for j in (level..=degree).rev(){
                let left=knot(span as isize-degree as isize+j as isize)?;
                let right=knot(span as isize+1+j as isize-level as isize)?;
                let width=right-left;if !width.is_finite()||width<=0.{return Err(EvaluationError::Arithmetic);}
                let alpha=(parameter-left)/width;
                // Descending coefficients retain the previous Taylor order on
                // both sides while multiplying by the linear alpha series.
                for k in (0..=order).rev(){
                    let a=values[j-1][k];let b=values[j][k];
                    let derivative=if k==0{[0.;4]}else{std::array::from_fn(|c|(values[j][k-1][c]-values[j-1][k-1][c])/width)};
                    values[j][k]=std::array::from_fn(|c|(1.-alpha)*a[c]+alpha*b[c]+derivative[c]);
                }
            }
        }
        let homogeneous=&values[degree];let w=homogeneous[0][3];
        if !w.is_finite()||w<=0.{return Err(EvaluationError::Arithmetic);}
        let mut coefficients=vec![[0.;3];order+1];
        for k in 0..=order{
            coefficients[k]=std::array::from_fn(|c|(homogeneous[k][c]-(1..=k).map(|j|homogeneous[j][3]*coefficients[k-j][c]).sum::<f64>())/w);
        }
        crate::curve_taylor::finite(&coefficients)?;Ok(coefficients)
    }
    pub fn json(&self)->Result<String,SerializedLimit> {
        let mut j=Json(String::new());
        let result=(||{
            j.write_str("{\"schema_version\":1,\"kind\":\"nurbs_curve\",")?;
            geometry(&mut j,&self.poles,&self.weights)?;j.write_str(",\"axis\":")?;
            axis(&mut j,&self.axis)?;j.write_char('}')
        })();
        result.map(|_|j.0).map_err(|_|SerializedLimit)
    }
}
impl Surface {
    pub fn capture(b:SurfaceBasis<'_>)->Result<Self,BasisError> {
        b.validate(policy())?;
        Ok(Self{poles:b.poles.to_vec(),weights:b.weights.map(<[f64]>::to_vec),u_count:b.u_count,v_count:b.v_count,axes:b.axes.map(Axis::from)})
    }
    pub fn json(&self)->Result<String,SerializedLimit> {
        let mut j=Json(String::new());
        let result=(||{
            write!(j,"{{\"schema_version\":1,\"kind\":\"nurbs_surface\",\"u_count\":{},\"v_count\":{},\"pole_layout\":\"u_major\",",self.u_count,self.v_count)?;
            geometry(&mut j,&self.poles,&self.weights)?;j.write_str(",\"axes\":[")?;
            axis(&mut j,&self.axes[0])?;j.write_char(',')?;axis(&mut j,&self.axes[1])?;j.write_str("]}")
        })();
        result.map(|_|j.0).map_err(|_|SerializedLimit)
    }
}
fn bezier_axis(a:BorrowedAxis<'_>,poles:usize)->Result<(),BasisError> {
    policy().validate_degree(a.degree)?;
    let count=a.degree.checked_add(1).ok_or(BasisError::CountOverflow)?;
    if poles!=count{return Err(BasisError::Cardinality);}
    if a.periodic{return Err(BasisError::PeriodicCardinality);}
    if a.domain!=[0.,1.]{return Err(BasisError::Domain);}
    if a.knots!=[0.,1.]{return Err(BasisError::Knots);}
    if a.multiplicities!=[count,count]{return Err(BasisError::Multiplicity);}
    Ok(())
}
impl BezierCurve {
    pub fn capture(b:CurveBasis<'_>)->Result<Self,BasisError> {
        bezier_axis(b.axis,b.poles.len())?;
        Curve::capture(b).map(Self)
    }
    pub fn evaluate(&self,parameter:f64)->Result<([f64;3],[f64;3]),EvaluationError>{self.0.evaluate(parameter)}
    pub(crate) fn taylor(&self,parameter:f64,order:usize)->Result<Vec<[f64;3]>,EvaluationError>{self.0.taylor(parameter,order)}
    pub fn json(&self)->Result<String,SerializedLimit> {
        let b=&self.0;let mut j=Json(String::new());
        let result=(||{
            j.write_str("{\"schema_version\":1,\"kind\":\"bezier_curve\",")?;
            geometry(&mut j,&b.poles,&b.weights)?;
            write!(j,",\"degree\":{},\"full_domain\":[0,1]}}",b.axis.degree)
        })();result.map(|_|j.0).map_err(|_|SerializedLimit)
    }
}
impl BezierSurface {
    pub fn capture(b:SurfaceBasis<'_>)->Result<Self,BasisError> {
        bezier_axis(b.axes[0],b.u_count)?;bezier_axis(b.axes[1],b.v_count)?;
        Surface::capture(b).map(Self)
    }
    pub fn json(&self)->Result<String,SerializedLimit> {
        let b=&self.0;let mut j=Json(String::new());
        let result=(||{
            write!(j,"{{\"schema_version\":1,\"kind\":\"bezier_surface\",\"u_count\":{},\"v_count\":{},\"pole_layout\":\"u_major\",",b.u_count,b.v_count)?;
            geometry(&mut j,&b.poles,&b.weights)?;
            write!(j,",\"degrees\":[{},{}],\"full_domains\":[[0,1],[0,1]]}}",b.axes[0].degree,b.axes[1].degree)
        })();result.map(|_|j.0).map_err(|_|SerializedLimit)
    }
}
