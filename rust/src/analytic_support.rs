//! Owned native analytic metadata, without conversion or shape-validity claims.
use crate::support_snapshot::{Json,SerializedLimit,numbers,policy};
use std::fmt::Write;
#[repr(C)]
#[derive(Clone,Copy,Debug,PartialEq)]
pub struct AnalyticInput {
    /// 1..5 Line/Circle/Ellipse/Hyperbola/Parabola;6..10 Plane/Cylinder/Cone/Sphere/Torus.
    pub kind:u32,pub dimension:u32,pub origin:[f64;3],pub axes:[[f64;3];3],pub radii:[f64;2],pub extra:f64,
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Error {Kind,Dimension,NonFinite,Frame,Parameters}
#[derive(Clone,Copy,Debug)]
pub struct Analytic {input:AnalyticInput,handedness:i32}
fn dot(a:[f64;3],b:[f64;3])->f64 {a.into_iter().zip(b).map(|(x,y)|x*y).sum()}
fn cross(a:[f64;3],b:[f64;3])->[f64;3] {[a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]]}
impl Analytic {
    pub fn capture(i:AnalyticInput)->Result<Self,Error> {
        if !(1..=10).contains(&i.kind){return Err(Error::Kind);}
        if !matches!(i.dimension,2|3) || (i.kind>=6 && i.dimension!=3){return Err(Error::Dimension);}
        if i.origin.into_iter().chain(i.axes.into_iter().flatten()).chain(i.radii).chain([i.extra])
            .any(|v|!v.is_finite() || v.abs()>policy().max_coordinate){return Err(Error::NonFinite);}
        if i.dimension==2 && (i.origin[2]!=0. || i.axes[0][2]!=0. || i.axes[1][2]!=0. || i.axes[2][0]!=0. || i.axes[2][1]!=0.) {return Err(Error::Dimension);}
        let mut handedness=0;
        let unit=|a| (dot(a,a)-1.).abs()<=1e-10;
        if i.kind==1 {
            if !unit(i.axes[0]) || i.axes[1]!=[0.;3] || i.axes[2]!=[0.;3]{return Err(Error::Frame);}
        } else {
            if i.axes.into_iter().any(|a|!unit(a)) || dot(i.axes[0],i.axes[1]).abs()>1e-10 || dot(i.axes[0],i.axes[2]).abs()>1e-10 || dot(i.axes[1],i.axes[2]).abs()>1e-10 {return Err(Error::Frame);}
            let determinant=dot(cross(i.axes[0],i.axes[1]),i.axes[2]);
            if (determinant.abs()-1.).abs()>1e-10{return Err(Error::Frame);}
            handedness=if determinant>0.{1}else{-1};
        }
        let [a,b]=i.radii;
        // Native conics can store zero radii/focal lengths; this is metadata
        // capture, not a regularity certificate or permission to divide by zero.
        let parameters=match i.kind {
            1|6=>a==0. && b==0. && i.extra==0.,
            2|5|7|9=>a>=0. && b==0. && i.extra==0.,
            3=>a>=b && b>=0. && i.extra==0.,
            4|10=>a>=0. && b>=0. && i.extra==0.,
            8=>a>=0. && b==0. && i.extra.abs()>0. && i.extra.abs()<std::f64::consts::FRAC_PI_2,
            _=>false,
        };
        if !parameters{return Err(Error::Parameters);}
        Ok(Self{input:i,handedness})
    }
    pub fn json(&self)->Result<String,SerializedLimit> {
        let i=self.input;let mut j=Json(String::new());
        let names=["line","circle","ellipse","hyperbola","parabola","plane","cylinder","cone","sphere","torus"];
        let result=(||{
            write!(j,"{{\"schema_version\":1,\"kind\":\"analytic_{}\",\"analytic_type\":\"{}\",\"dimension\":{},\"parameter_convention\":\"occt_native\",\"origin_xyz\":",if i.kind<=5{"curve"}else{"surface"},names[(i.kind-1) as usize],i.dimension)?;
            numbers(&mut j,&i.origin)?;j.write_str(",\"axes_xyz\":[")?;
            for (n,a) in i.axes.iter().enumerate(){if n!=0{j.write_char(',')?;}numbers(&mut j,a)?;}
            // The determinant of the embedded XYZ frame is distinct from the
            // native XY sense: UV Z is X cross Y, so both UV senses embed direct.
            write!(j,"],\"frame_handedness\":{},\"uv_sense\":",self.handedness)?;
            if i.dimension==2 && i.kind!=1 {
                let xy=i.axes[0][0]*i.axes[1][1]-i.axes[0][1]*i.axes[1][0];
                write!(j,"{}",if xy>0.{1}else{-1})?;
            } else {j.write_str("null")?;}
            j.write_str(",\"parameters\":{")?;
            match i.kind {
                2|7|9=>write!(j,"\"radius\":{}",i.radii[0])?,
                3|4|10=>write!(j,"\"major_radius\":{},\"minor_radius\":{}",i.radii[0],i.radii[1])?,
                5=>write!(j,"\"focal\":{}",i.radii[0])?,
                8=>write!(j,"\"reference_radius\":{},\"semi_angle\":{}",i.radii[0],i.extra)?,_=>{}
            }
            j.write_str("},\"natural_domains\":[")?;
            match i.kind {
                1|4|5=>j.write_str("\"unbounded\"")?,
                2|3=>numbers(&mut j,&[0.,std::f64::consts::TAU])?,
                6=>j.write_str("\"unbounded\",\"unbounded\"")?,
                7|8=>{numbers(&mut j,&[0.,std::f64::consts::TAU])?;j.write_str(",\"unbounded\"")?;},
                9=>{numbers(&mut j,&[0.,std::f64::consts::TAU])?;j.write_char(',')?;numbers(&mut j,&[-std::f64::consts::FRAC_PI_2,std::f64::consts::FRAC_PI_2])?;},
                10=>{numbers(&mut j,&[0.,std::f64::consts::TAU])?;j.write_char(',')?;numbers(&mut j,&[0.,std::f64::consts::TAU])?;},_=>unreachable!(),
            }
            j.write_str("]}")
        })();result.map(|_|j.0).map_err(|_|SerializedLimit)
    }
}
