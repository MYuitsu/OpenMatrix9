// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{budget::Limits,rhino_semantics::{classify_trim_uv,validate_trim_loop_topology,TrimmedSurface},surface_kernel};

#[test]
fn trim_uv_classifies_outer_hole_and_ambiguous_boundaries() {
    let limits=Limits::default();
    let surface=surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    let trim=TrimmedSurface {underlying:surface,outer:vec![[0.1,0.1],[0.9,0.1],[0.9,0.9],[0.1,0.9]],
        inners:vec![vec![[0.4,0.4],[0.6,0.4],[0.6,0.6],[0.4,0.6]]],reversed:false};
    trim.validate(&limits).unwrap();validate_trim_loop_topology(&trim).unwrap();
    assert_eq!(classify_trim_uv(&trim,[0.2,0.2]).unwrap(),Some(true));
    assert_eq!(classify_trim_uv(&trim,[0.5,0.5]).unwrap(),Some(false));
    assert_eq!(classify_trim_uv(&trim,[0.05,0.5]).unwrap(),Some(false));
    assert_eq!(classify_trim_uv(&trim,[0.4,0.5]).unwrap(),None);
    assert_eq!(classify_trim_uv(&trim,[0.1,0.1]).unwrap(),None);
    assert!(classify_trim_uv(&trim,[f64::NAN,0.5]).is_err());
}
