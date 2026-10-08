use crate::spline::{self, Spline};
use std::{ffi::c_char,sync::{Mutex,OnceLock}};
#[derive(Default)]
struct State {curve: Option<Spline>, message: String}
fn state()-> &'static Mutex<State> {static S:OnceLock<Mutex<State>>=OnceLock::new();S.get_or_init(||Mutex::new(State::default()))}
pub(crate) fn publish(result: Result<Spline,String>)->bool {
    let Ok(mut s)=state().lock() else {return false;};
    match result {Ok(curve)=>{s.curve=Some(curve);s.message.clear();true},Err(e)=>{s.curve=None;s.message=e;false}}
}
/// # Safety
/// `xyz` points to `count*3` readable doubles for the duration of this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_spline_rebuild(xyz:*const f64,count:usize,poles:usize,degree:usize,closed:bool)->bool {
    if xyz.is_null() || !(2..=spline::MAX_SAMPLES).contains(&count) {return publish(Err("Invalid rebuild samples".into()));}
    let input=unsafe {std::slice::from_raw_parts(xyz,count*3)};
    let points=input.chunks_exact(3).map(|p|[p[0],p[1],p[2]]).collect::<Vec<_>>();
    publish(spline::rebuild(&points,poles,degree,closed))
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_spline_options(poles:usize,degree:usize)->bool {spline::rebuild_options(poles,degree).is_ok()}
#[unsafe(no_mangle)]
pub extern "C" fn om9_spline_degree()->usize {state().lock().ok().and_then(|s|s.curve.as_ref().map(|c|c.degree)).unwrap_or(0)}
#[unsafe(no_mangle)]
pub extern "C" fn om9_spline_periodic()->bool {state().lock().ok().is_some_and(|s|s.curve.as_ref().is_some_and(|c|c.periodic))}
#[unsafe(no_mangle)]
pub extern "C" fn om9_spline_pole_count()->usize {state().lock().ok().and_then(|s|s.curve.as_ref().map(|c|c.poles.len())).unwrap_or(0)}
#[unsafe(no_mangle)]
pub extern "C" fn om9_spline_pole(i:usize,axis:usize)->f64 {state().lock().ok().and_then(|s|s.curve.as_ref().and_then(|c|c.poles.get(i).and_then(|p|p.get(axis)).copied())).unwrap_or(f64::NAN)}
#[unsafe(no_mangle)]
pub extern "C" fn om9_spline_knot_count()->usize {state().lock().ok().and_then(|s|s.curve.as_ref().map(|c|c.host_knots().0.len())).unwrap_or(0)}
#[unsafe(no_mangle)]
pub extern "C" fn om9_spline_knot(i:usize)->f64 {state().lock().ok().and_then(|s|s.curve.as_ref().and_then(|c|c.host_knots().0.get(i).copied())).unwrap_or(f64::NAN)}
#[unsafe(no_mangle)]
pub extern "C" fn om9_spline_multiplicity(i:usize)->usize {state().lock().ok().and_then(|s|s.curve.as_ref().and_then(|c|c.host_knots().1.get(i).copied())).unwrap_or(0)}
#[unsafe(no_mangle)]
pub extern "C" fn om9_spline_value(u:f64,axis:usize)->f64 {if !u.is_finite(){return f64::NAN;}state().lock().ok().and_then(|s|s.curve.as_ref().and_then(|c|c.value(u).get(axis).copied())).unwrap_or(f64::NAN)}
/// # Safety
/// `buffer` is writable for `capacity` bytes, or null when capacity is zero.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_spline_message(buffer:*mut c_char,capacity:usize)->usize {
    let Ok(s)=state().lock() else {return 0;};let bytes=s.message.as_bytes();
    if !buffer.is_null() && capacity>0 {let n=bytes.len().min(capacity-1);unsafe{std::ptr::copy_nonoverlapping(bytes.as_ptr(),buffer.cast::<u8>(),n);*buffer.add(n)=0;}}
    bytes.len()
}
