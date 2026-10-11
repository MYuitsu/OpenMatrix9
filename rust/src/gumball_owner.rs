// SPDX-License-Identifier: LGPL-2.1-or-later
//! Exact affine admission for native parametric and occurrence properties.
use crate::gumball::GumballError;
#[repr(C)]
#[derive(Clone,Copy,Debug)]
pub struct OwnerScale {pub placement:[f64;16],pub factors:[f64;3]}
fn dot(a:[f64;3],b:[f64;3])->f64 {a.iter().zip(b).map(|(a,b)|a*b).sum()}
pub fn owner_scale(affine:[f64;16],world:[f64;16],old_scale:[f64;3],kind:u32)->Result<OwnerScale,GumballError>{
    if !(1..=3).contains(&kind)||affine.iter().chain(world.iter()).chain(old_scale.iter()).any(|v|!v.is_finite()||v.abs()>1e12)
        || old_scale.iter().any(|v|v.abs()<1e-12)
        || [affine[12],affine[13],affine[14],world[12],world[13],world[14]].iter().any(|v|v.abs()>1e-12)
        || affine[15]!=1.||world[15]!=1. {return Err(GumballError::InvalidInput);}
    let original:[[f64;3];3]=std::array::from_fn(|c|std::array::from_fn(|r|world[4*r+c]));
    crate::gumball::Frame {origin:[world[3],world[7],world[11]],axes:original}.validate()?;
    let m:[f64;16]=std::array::from_fn(|i|{let r=i/4;let c=i%4;(0..4).map(|k|affine[4*r+k]*world[4*k+c]).sum()});
    let columns:[[f64;3];3]=std::array::from_fn(|c|std::array::from_fn(|r|m[4*r+c]));
    let lengths:[f64;3]=std::array::from_fn(|c|dot(columns[c],columns[c]).sqrt());
    if lengths.iter().any(|v|*v<1e-12||!v.is_finite()){return Err(GumballError::InvalidInput);}
    let mut axes:[[f64;3];3]=std::array::from_fn(|c|columns[c].map(|v|v/lengths[c]));
    if [dot(axes[0],axes[1]),dot(axes[0],axes[2]),dot(axes[1],axes[2])].iter().any(|v|v.abs()>1e-8){return Err(GumballError::InvalidFrame);}
    let mut factors=lengths;let mut placement=m;
    if kind==1 { // dimension properties preserve their native owner axes
        for c in 0..3 {let original=std::array::from_fn(|r|world[4*r+c]);if (dot(axes[c],original)-1.).abs()>1e-8{return Err(GumballError::InvalidFrame);}axes[c]=original;}
    }else if kind==2 { // a native Sphere can represent isotropic scaling only
        if (lengths[0]-lengths[1]).abs()>1e-8*lengths[0]||(lengths[0]-lengths[2]).abs()>1e-8*lengths[0]{return Err(GumballError::InvalidFrame);}
        for c in 0..3 {let original=std::array::from_fn(|r|world[4*r+c]);if (dot(axes[c],original)-1.).abs()>1e-8{return Err(GumballError::InvalidFrame);}}
        axes=std::array::from_fn(|c|std::array::from_fn(|r|world[4*r+c]));
    }else {
        let determinant=axes[0][0]*(axes[1][1]*axes[2][2]-axes[1][2]*axes[2][1])-axes[1][0]*(axes[0][1]*axes[2][2]-axes[0][2]*axes[2][1])+axes[2][0]*(axes[0][1]*axes[1][2]-axes[0][2]*axes[1][1]);
        if determinant<0. {axes[0]=axes[0].map(|v|-v);factors[0]=-factors[0];}
        for c in 0..3 {factors[c]*=old_scale[c];}
    }
    for c in 0..3 {for r in 0..3 {placement[4*r+c]=axes[c][r];}}
    if factors.iter().chain(placement.iter()).any(|v|!v.is_finite()||v.abs()>1e12) {
        return Err(GumballError::InvalidInput);
    }
    Ok(OwnerScale{placement,factors})
}
