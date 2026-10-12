//! Shared admission before bounded native FitPoints extraction.
pub const MAX_POINTS:usize=1024;
pub fn admit_count(used:usize,added:usize)->bool {
    used.checked_add(added).is_some_and(|total|total<=MAX_POINTS)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_fit_point_count(used:usize,added:usize)->u32 {
    std::panic::catch_unwind(|| if admit_count(used,added) {0} else {2}).unwrap_or(5)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_fit_point_valid(kind:u32,x:f64,y:f64,z:f64)->u32 {
    std::panic::catch_unwind(|| valid_point(kind,x,y,z)).unwrap_or(5)
}
fn valid_point(kind:u32,x:f64,y:f64,z:f64)->u32 {
    if kind!=1 && kind!=2 {return 1;}
    let point=[x,y,z];
    if point.iter().any(|v| !v.is_finite()) {return 4;}
    let valid=if kind==1 {crate::circle::check_point(point).is_ok()}
              else {crate::solid::finite(point)};
    if valid {0} else {3}
}
