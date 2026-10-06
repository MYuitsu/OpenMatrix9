//! OM9-VIEW-001: image aspect and corner-based placement in a native C-plane.
//! OM9 limits: positive dimensions >=1e-7 mm, coordinates/dimensions <=1e9 mm.
#[derive(Clone, Copy, Debug)]
pub struct Frame {
    axes: [[f64; 3]; 3],
}
#[derive(Clone, Copy, Debug)]
pub struct Placement {
    pub corner: [f64; 3],
    pub center: [f64; 3],
    pub x: [f64; 3],
    pub y: [f64; 3],
    pub normal: [f64; 3],
    pub width: f64,
    pub height: f64,
}
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a.iter().zip(b).map(|(a, b)| a * b).sum()
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn coordinate(point: [f64; 3]) -> bool {
    point.iter().all(|v| v.is_finite() && v.abs() <= 1e9)
}
impl Frame {
    pub fn new(axes: [[f64; 3]; 3]) -> Result<Self, &'static str> {
        if axes.iter().flatten().any(|v| !v.is_finite()) {
            return Err("Construction plane must be finite");
        }
        for i in 0..3 {
            for j in 0..3 {
                if (dot(axes[i], axes[j]) - if i == j { 1. } else { 0. }).abs() > 1e-6 {
                    return Err("Construction plane must be orthonormal");
                }
            }
        }
        if dot(cross(axes[0], axes[1]), axes[2]) < 1. - 1e-6 {
            return Err("Construction plane must be right handed");
        }
        Ok(Self { axes })
    }
}
impl Placement {
    pub fn from_width(
        frame: Frame,
        corner: [f64; 3],
        width: f64,
        aspect: f64,
        vertical: bool,
    ) -> Result<Self, &'static str> {
        Self::build(frame, corner, frame.axes[0], width, aspect, vertical)
    }
    pub fn from_points(
        frame: Frame,
        corner: [f64; 3],
        reference: [f64; 3],
        aspect: f64,
        ortho: bool,
        vertical: bool,
    ) -> Result<Self, &'static str> {
        if !coordinate(reference) || !coordinate(corner) {
            return Err("Invalid image reference point");
        }
        let delta = std::array::from_fn(|i| reference[i] - corner[i]);
        let mut local = [dot(delta, frame.axes[0]), dot(delta, frame.axes[1])];
        if ortho {
            if local[0].abs() >= local[1].abs() {
                local[1] = 0.;
            } else {
                local[0] = 0.;
            }
        }
        let width = local[0].hypot(local[1]);
        if width < 1e-7 {
            return Err("Choose a different second reference point");
        }
        let x = std::array::from_fn(|i| {
            (frame.axes[0][i] * local[0] + frame.axes[1][i] * local[1]) / width
        });
        Self::build(frame, corner, x, width, aspect, vertical)
    }
    fn build(
        frame: Frame,
        corner: [f64; 3],
        x: [f64; 3],
        width: f64,
        aspect: f64,
        vertical: bool,
    ) -> Result<Self, &'static str> {
        if !coordinate(corner) || !width.is_finite() || !(1e-7..=1e9).contains(&width) {
            return Err("Image width must be positive and finite");
        }
        if !aspect.is_finite() || aspect <= 0. {
            return Err("Image aspect ratio must be positive and finite");
        }
        let height = width / aspect;
        if !height.is_finite() || !(1e-7..=1e9).contains(&height) {
            return Err("Image height is outside supported dimensions");
        }
        let y = if vertical {
            frame.axes[2]
        } else {
            cross(frame.axes[2], x)
        };
        let normal = cross(x, y);
        let center = std::array::from_fn(|i| corner[i] + x[i] * width / 2. + y[i] * height / 2.);
        let opposite = std::array::from_fn(|i| corner[i] + x[i] * width + y[i] * height);
        let along_x = std::array::from_fn(|i| corner[i] + x[i] * width);
        let along_y = std::array::from_fn(|i| corner[i] + y[i] * height);
        if !coordinate(center)
            || !coordinate(opposite)
            || !coordinate(along_x)
            || !coordinate(along_y)
        {
            return Err("Image corners are outside supported coordinates");
        }
        Ok(Self {
            corner,
            center,
            x,
            y,
            normal,
            width,
            height,
        })
    }
}

/// Compute an image rectangle without modifying session/document state.
/// Flags: 1=Ortho, 2=Vertical, 4=scalar width rather than reference point.
/// Output: corner3, center3, x3, y3, normal3, width, height. Failure leaves it untouched.
///
/// # Safety
/// Nonnull axes/corner/reference must point to readable aligned arrays of 9/3/3
/// doubles; output must point to 17 writable aligned doubles. Reference may be
/// null when flag4 is set. All allocations must remain valid for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_picture_frame_plan(
    axes: *const f64,
    corner: *const f64,
    reference: *const f64,
    width: f64,
    aspect: f64,
    flags: u32,
    output: *mut f64,
) -> bool {
    if axes.is_null()
        || corner.is_null()
        || output.is_null()
        || flags & !7 != 0
        || (flags & 4 == 0 && reference.is_null())
    {
        return false;
    }
    let axes = unsafe { *axes.cast::<[[f64; 3]; 3]>() };
    let corner = unsafe { *corner.cast::<[f64; 3]>() };
    let Ok(frame) = Frame::new(axes) else {
        return false;
    };
    let result = if flags & 4 != 0 {
        Placement::from_width(frame, corner, width, aspect, flags & 2 != 0)
    } else {
        let reference = unsafe { *reference.cast::<[f64; 3]>() };
        Placement::from_points(
            frame,
            corner,
            reference,
            aspect,
            flags & 1 != 0,
            flags & 2 != 0,
        )
    };
    let Ok(p) = result else {
        return false;
    };
    let mut values = [0.; 17];
    for (i, point) in [p.corner, p.center, p.x, p.y, p.normal].iter().enumerate() {
        values[i * 3..i * 3 + 3].copy_from_slice(point);
    }
    values[15] = p.width;
    values[16] = p.height;
    unsafe {
        std::ptr::copy_nonoverlapping(values.as_ptr(), output, 17);
    }
    true
}
