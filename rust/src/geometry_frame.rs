// SPDX-License-Identifier: LGPL-2.1-or-later
//! Owned support-local frame. Native adapters supply observations, never handles.
pub const IDENTITY:[f64;16]=[1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.];
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum FrameError {NonFinite,NonAffine,Singular,NonRigid,Limit}
#[derive(Clone,Copy,Debug)]
pub struct Frame {matrix:[f64;16],sign:i32}
const LIMIT:f64=1e150;
const EPS:f64=64.*f64::EPSILON;
fn bounded(values:&[f64])->Result<(),FrameError> {
    if values.iter().any(|v|!v.is_finite()){return Err(FrameError::NonFinite);}
    if values.iter().any(|v|v.abs()>LIMIT){return Err(FrameError::Limit);}
    Ok(())
}
fn determinant(m:&[f64;16])->f64 {
    m[0]*(m[5]*m[10]-m[6]*m[9])-m[1]*(m[4]*m[10]-m[6]*m[8])+m[2]*(m[4]*m[9]-m[5]*m[8])
}
fn affine(m:&[f64;16])->Result<i32,FrameError> {
    bounded(m)?;
    if m[12..]!=[0.,0.,0.,1.]{return Err(FrameError::NonAffine);}
    // Normalize columns for determinant sign without scale overflow/underflow.
    let mut unit=*m;
    for c in 0..3 {
        let length=m[c].hypot(m[4+c]).hypot(m[8+c]);
        if length==0.||!length.is_finite(){return Err(FrameError::Singular);}
        for r in 0..3 {unit[4*r+c]/=length;}
    }
    let det=determinant(&unit);
    if !det.is_finite()||det.abs()<=EPS{return Err(FrameError::Singular);}
    Ok(if det<0.{-1}else{1})
}
fn inverse_rigid(m:[f64;16])->Result<[f64;16],FrameError> {
    if affine(&m)?!=1{return Err(FrameError::NonRigid);}
    for a in 0..3 {for b in 0..3 {
        let dot=(0..3).map(|r|m[4*r+a]*m[4*r+b]).sum::<f64>();
        if (dot-f64::from(a==b)).abs()>EPS{return Err(FrameError::NonRigid);}
    }}
    let mut inverse=IDENTITY;
    for r in 0..3 {for c in 0..3 {inverse[4*r+c]=m[4*c+r];}
        inverse[4*r+3]=-(0..3).map(|c|inverse[4*r+c]*m[4*c+3]).sum::<f64>();
    }
    bounded(&inverse)?;Ok(inverse)
}
fn product(a:[f64;16],b:[f64;16])->Result<[f64;16],FrameError> {
    let mut out=IDENTITY;
    for r in 0..3 {for c in 0..4 {
        out[4*r+c]=(0..4).map(|k|a[4*r+k]*b[4*k+c]).sum();
    }}
    affine(&out)?;Ok(out)
}
impl Frame {
    /// Shape location already includes source placement. Remove that embedding
    /// once; the native path supplies the direct or instance placement policy.
    pub fn compose(parent:[f64;16],path:[f64;16],source:[f64;16],location:[f64;16])->Result<Self,FrameError>{
        affine(&parent)?;affine(&path)?;affine(&location)?;
        let inverse=inverse_rigid(source)?;
        let matrix=product(product(product(parent,path)?,inverse)?,location)?;
        Ok(Self{sign:affine(&matrix)?,matrix})
    }
    pub fn matrix(&self)->[f64;16]{self.matrix}
    pub fn orientation_sign(&self)->i32{self.sign}
    pub fn transform(&self,point:[f64;3])->Result<[f64;3],FrameError>{
        bounded(&point)?;
        let out=std::array::from_fn(|r|self.matrix[4*r+3]+(0..3).map(|c|self.matrix[4*r+c]*point[c]).sum::<f64>());
        bounded(&out)?;Ok(out)
    }
}
