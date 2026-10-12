// SPDX-License-Identifier: LGPL-2.1-or-later
//! Bounded Taylor coefficients: coefficient k is Dk/k!, never sampled differences.
use crate::support_snapshot::EvaluationError;
pub(crate) const MAX_ORDER:usize=crate::derived_support::MAX_DEPTH+1;
pub(crate) fn order_allowed(order:usize)->Result<(),EvaluationError>{
 if order>MAX_ORDER{Err(EvaluationError::Parameter)}else{Ok(())}
}
pub(crate) fn finite(values:&[[f64;3]])->Result<(),EvaluationError>{
 if values.iter().flatten().any(|v|!v.is_finite()){Err(EvaluationError::Arithmetic)}else{Ok(())}
}
/// Normalize a vector-valued Taylor series with a nonzero constant term.
/// Scaling by its largest constant component avoids squaring extreme magnitudes.
pub(crate) fn normalize(values:&[[f64;3]])->Result<Vec<[f64;3]>,EvaluationError>{
 let count=values.len();if count==0||count>MAX_ORDER+1{return Err(EvaluationError::Parameter);}
 let scale=values[0].iter().map(|v|v.abs()).fold(0.,f64::max);
 if !scale.is_finite()||scale<=0.{return Err(EvaluationError::Arithmetic);}
 let v:Vec<[f64;3]>=values.iter().map(|a|a.map(|x|x/scale)).collect();finite(&v)?;
 let squared:Vec<f64>=(0..count).map(|k|(0..=k).map(|j|(0..3).map(|c|v[j][c]*v[k-j][c]).sum::<f64>()).sum()).collect();
 let mut length=vec![0.;count];length[0]=squared[0].sqrt();
 if !length[0].is_finite()||length[0]<=0.{return Err(EvaluationError::Arithmetic);}
 for k in 1..count{length[k]=(squared[k]-(1..k).map(|j|length[j]*length[k-j]).sum::<f64>())/(2.*length[0]);}
 let mut result=vec![[0.;3];count];
 for k in 0..count{result[k]=std::array::from_fn(|c|(v[k][c]-(1..=k).map(|j|length[j]*result[k-j][c]).sum::<f64>())/length[0]);}
 finite(&result)?;Ok(result)
}
