// SPDX-License-Identifier: LGPL-2.1-or-later
//! Admission for the native rigid Link / diagonal Part::Scale / rigid Link graph.
use crate::gumball::{Frame,GumballError};

pub fn scale_factors(a:[f64;16],frame:Frame)->Result<[f64;3],GumballError> {
    frame.validate()?;
    if a.iter().chain(frame.origin.iter()).any(|v|!v.is_finite()||v.abs()>1e12)
        || a[12..15].iter().any(|v|v.abs()>1e-12)||a[15]!=1. {
        return Err(GumballError::InvalidInput);
    }
    let factors:[f64;3]=std::array::from_fn(|k|{
        (0..3).map(|r|(0..3).map(|c|frame.axes[k][r]*a[4*r+c]*frame.axes[k][c]).sum::<f64>()).sum()
    });
    if factors.iter().any(|v|!v.is_finite()||v.abs()<1e-12||v.abs()>1e12) {
        return Err(GumballError::InvalidInput);
    }
    let close=|actual:f64,expected:f64,budget:f64|{
        (actual-expected).abs()<=1e-8+128.*f64::EPSILON*budget.max(1.)
    };
    for r in 0..3 {
        for c in 0..3 {
            let expected=(0..3).map(|k|frame.axes[k][r]*factors[k]*frame.axes[k][c]).sum::<f64>();
            if !close(a[4*r+c],expected,expected.abs().max(a[4*r+c].abs())) {
                return Err(GumballError::InvalidFrame);
            }
        }
        let product=(0..3).map(|c|a[4*r+c]*frame.origin[c]).sum::<f64>();
        let budget=frame.origin[r].abs()+(0..3).map(|c|(a[4*r+c]*frame.origin[c]).abs()).sum::<f64>();
        if !close(a[4*r+3],frame.origin[r]-product,budget) {
            return Err(GumballError::InvalidFrame);
        }
    }
    Ok(factors)
}

/// # Safety
/// Input arrays are readable for 16 and 12 f64s; output writable for 3 f64s.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_gumball_associative_scale(
    affine:*const f64,frame:*const f64,out:*mut f64)->u32 {
    std::panic::catch_unwind(||{
        if affine.is_null()||frame.is_null()||out.is_null(){return 1;}
        let a:[f64;16]=unsafe{std::slice::from_raw_parts(affine,16)}.try_into().unwrap();
        let f=unsafe{std::slice::from_raw_parts(frame,12)};
        let frame=Frame{origin:[f[0],f[1],f[2]],axes:[[f[3],f[4],f[5]],[f[6],f[7],f[8]],[f[9],f[10],f[11]]]};
        let Ok(factors)=scale_factors(a,frame)else{return 2;};
        unsafe{std::ptr::copy_nonoverlapping(factors.as_ptr(),out,3)};0
    }).unwrap_or(1)
}
