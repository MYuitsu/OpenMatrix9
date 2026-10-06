use openmatrix9_rust::core_picture_frame::om9_picture_frame_plan;
use openmatrix9_rust::core_picture_frame::{Frame, Placement};

fn xy() -> Frame {
    Frame::new([[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]]).unwrap()
}

#[test]
fn om9_view_001_two_points_preserve_source_aspect_and_corner() {
    let p = Placement::from_points(xy(), [10., 20., 0.], [16., 28., 0.], 2., false, false).unwrap();
    assert!((p.width - 10.).abs() < 1e-9);
    assert!((p.height - 5.).abs() < 1e-9);
    assert_eq!(p.corner, [10., 20., 0.]);
    for i in 0..3 {
        assert!(
            (p.center[i] - p.x[i] * p.width / 2. - p.y[i] * p.height / 2. - p.corner[i]).abs()
                < 1e-9
        );
    }
}

#[test]
fn om9_view_001_shift_and_vertical_use_construction_plane_axes() {
    let frame = Frame::new([[0., 1., 0.], [0., 0., 1.], [1., 0., 0.]]).unwrap();
    let p = Placement::from_points(frame, [0., 0., 0.], [0., -4., 2.], 2., true, true).unwrap();
    assert_eq!(p.x, [0., -1., 0.]);
    assert_eq!(p.y, [1., 0., 0.]);
    assert_eq!(p.normal, [0., 0., 1.]);
    assert_eq!(p.width, 4.);
    assert_eq!(p.height, 2.);
}

#[test]
fn om9_view_001_cmd_width_is_positive_and_aspect_is_validated() {
    let p = Placement::from_width(xy(), [0., 0., 0.], 12., 3., false).unwrap();
    assert_eq!(p.width, 12.);
    assert_eq!(p.height, 4.);
    assert_eq!(p.center, [6., 2., 0.]);
    for width in [0., -1., f64::NAN, f64::INFINITY, 1e10] {
        assert!(Placement::from_width(xy(), [0., 0., 0.], width, 2., false).is_err());
    }
    for aspect in [0., -2., f64::NAN, f64::INFINITY] {
        assert!(Placement::from_width(xy(), [0., 0., 0.], 10., aspect, false).is_err());
    }
    assert!(Placement::from_points(xy(), [0., 0., 0.], [0., 0., 5.], 2., false, false).is_err());
    assert!(Frame::new([[1., 0., 0.], [1., 0., 0.], [0., 0., 1.]]).is_err());
}

#[test]
fn om9_view_001_validates_every_corner_not_only_diagonal() {
    let frame = Frame::new([
        [
            std::f64::consts::FRAC_1_SQRT_2,
            std::f64::consts::FRAC_1_SQRT_2,
            0.,
        ],
        [
            -std::f64::consts::FRAC_1_SQRT_2,
            std::f64::consts::FRAC_1_SQRT_2,
            0.,
        ],
        [0., 0., 1.],
    ])
    .unwrap();
    assert!(Placement::from_width(frame, [0.9e9, 0., 0.], 0.5e9, 1., false).is_err());
}

#[test]
fn om9_view_001_ffi_is_atomic_and_agrees_with_native_array_layout() {
    let axes = [1., 0., 0., 0., 1., 0., 0., 0., 1.];
    let corner = [2., 3., 4.];
    let mut out = [-99.; 17];
    assert!(unsafe {
        om9_picture_frame_plan(
            axes.as_ptr(),
            corner.as_ptr(),
            std::ptr::null(),
            12.,
            3.,
            4,
            out.as_mut_ptr(),
        )
    });
    assert_eq!(&out[..6], &[2., 3., 4., 8., 5., 4.]);
    assert_eq!(&out[15..], &[12., 4.]);
    let before = out;
    assert!(!unsafe {
        om9_picture_frame_plan(
            axes.as_ptr(),
            corner.as_ptr(),
            std::ptr::null(),
            -1.,
            3.,
            4,
            out.as_mut_ptr(),
        )
    });
    assert_eq!(out, before);
    assert!(!unsafe {
        om9_picture_frame_plan(
            std::ptr::null(),
            corner.as_ptr(),
            std::ptr::null(),
            12.,
            3.,
            4,
            out.as_mut_ptr(),
        )
    });
    assert!(!unsafe {
        om9_picture_frame_plan(
            axes.as_ptr(),
            corner.as_ptr(),
            std::ptr::null(),
            12.,
            3.,
            0,
            out.as_mut_ptr(),
        )
    });
}
