//! OM9-CURVE-005 owned persistent Circle replay. No live session or published spline.
use crate::circle::{Circle, Mode, Plan, Point, Size, check_point, fit_circle};
use crate::spline::{self, Spline};
pub const CONFIG_LEN: usize = 24;
pub const OUTPUT_LEN: usize = 4096;
#[derive(Clone)]
pub struct Recipe {
    pub options: Circle,
    /// Already converted mm radius for a numeric Center/Vertical/AroundCurve commit.
    pub numeric_radius: Option<f64>,
}
pub struct Replay {
    pub plan: Plan,
    pub spline: Option<Spline>,
    pub fit_deviation: f64,
    pub approx_deviation: f64,
}
impl Recipe {
    pub fn new(options: &Circle, numeric_radius: Option<f64>) -> Self {
        Self {
            options: options.clone(),
            numeric_radius,
        }
    }
    pub fn encode(&self) -> [f64; CONFIG_LEN] {
        let c = &self.options;
        let mut out = [0.; CONFIG_LEN];
        out[0] = 2.;
        out[1] = match c.mode {
            Mode::Center => 0.,
            Mode::TwoPoint => 1.,
            Mode::ThreePoint => 2.,
            Mode::Vertical => 3.,
            Mode::AroundCurve => 4.,
            Mode::FitPoints => 5.,
            Mode::Tangent => 6.,
        };
        out[2] = match c.size {
            Size::Radius => 0.,
            Size::Diameter => 1.,
            Size::Circumference => 2.,
            Size::Area => 3.,
        };
        out[3] = self.numeric_radius.unwrap_or(0.);
        out[4] = c.radius_constraint.unwrap_or(0.);
        out[5] = f64::from(c.normal.is_some());
        out[6..9].copy_from_slice(&c.normal.unwrap_or([0.; 3]));
        for (i, a) in c.axes().iter().enumerate() {
            out[9 + i * 3..12 + i * 3].copy_from_slice(a);
        }
        out[18] = f64::from(c.deformable);
        out[19] = c.degree as f64;
        out[20] = c.point_count as f64;
        out[21] = f64::from(c.from_first);
        out[22] = c.solution.map_or(-1., |n| n as f64);
        out[23] = f64::from(c.tangent_vertical);
        out
    }
    pub fn decode(values: &[f64]) -> Result<Self, String> {
        if values.len() != CONFIG_LEN || values.iter().any(|v| !v.is_finite()) || values[0] != 2. {
            return Err("Unsupported or corrupt Circle History recipe".into());
        }
        let boolean = |i: usize| -> Result<bool, String> {
            match values[i] {
                0. => Ok(false),
                1. => Ok(true),
                _ => Err("Invalid Circle History boolean".into()),
            }
        };
        let integer = |i: usize, max: usize| -> Result<usize, String> {
            let v = values[i];
            if v < 0. || v > max as f64 || v.fract() != 0. {
                Err("Invalid Circle History integer".into())
            } else {
                Ok(v as usize)
            }
        };
        let mode = match integer(1, 6)? {
            0 => Mode::Center,
            1 => Mode::TwoPoint,
            2 => Mode::ThreePoint,
            3 => Mode::Vertical,
            4 => Mode::AroundCurve,
            5 => Mode::FitPoints,
            _ => Mode::Tangent,
        };
        let size = match integer(2, 3)? {
            0 => Size::Radius,
            1 => Size::Diameter,
            2 => Size::Circumference,
            _ => Size::Area,
        };
        let radius = |i: usize| -> Result<Option<f64>, String> {
            let r = values[i];
            if r == 0. {
                Ok(None)
            } else if (1e-7..=1e9).contains(&r) {
                Ok(Some(r))
            } else {
                Err("Invalid Circle History radius".into())
            }
        };
        let options = Circle {
            mode,
            size,
            axes: Some(std::array::from_fn(|i| {
                [values[9 + i * 3], values[10 + i * 3], values[11 + i * 3]]
            })),
            normal: if boolean(5)? {
                Some([values[6], values[7], values[8]])
            } else {
                None
            },
            radius_constraint: radius(4)?,
            deformable: boolean(18)?,
            degree: integer(19, 11)?,
            point_count: integer(20, 256)?,
            from_first: boolean(21)?,
            solution: if values[22] == -1. {
                None
            } else {
                Some(integer(22, 100000)?)
            },
            tangent_vertical: boolean(23)?,
            ..Circle::default()
        };
        let result = Self {
            options,
            numeric_radius: radius(3)?,
        };
        result.validate()?;
        Ok(result)
    }
    fn validate(&self) -> Result<(), String> {
        let encoded = self.encode();
        if encoded.iter().any(|v| !v.is_finite()) {
            return Err("Circle History contains nonfinite data".into());
        }
        let axes = self.options.axes();
        for i in 0..3 {
            for j in 0..3 {
                let dot: f64 = axes[i].iter().zip(axes[j]).map(|(a, b)| a * b).sum();
                if (dot - f64::from(i == j)).abs() > 1e-8 {
                    return Err("Circle History frame must be orthonormal".into());
                }
            }
        }
        let x = axes[0];
        let y = axes[1];
        let z = axes[2];
        let cross = [
            x[1] * y[2] - x[2] * y[1],
            x[2] * y[0] - x[0] * y[2],
            x[0] * y[1] - x[1] * y[0],
        ];
        if cross.iter().zip(z).map(|(a, b)| a * b).sum::<f64>() < 1. - 1e-8 {
            return Err("Circle History frame must be right handed".into());
        }
        if let Some(n) = self.options.normal {
            Plan::new([0.; 3], n, 1.)?;
        }
        for r in [self.numeric_radius, self.options.radius_constraint]
            .into_iter()
            .flatten()
        {
            Plan::new([0.; 3], [0., 0., 1.], r)?;
        }
        spline::rebuild_options(self.options.point_count, self.options.degree)?;
        if self.numeric_radius.is_some() && !self.options.center_mode() {
            return Err("Numeric Circle History radius requires a center mode".into());
        }
        Ok(())
    }
}
pub fn replay(
    recipe: &Recipe,
    points: &[Point],
    native_plan: Option<[f64; 7]>,
) -> Result<Replay, String> {
    recipe.validate()?;
    if points.is_empty() || points.len() > 1024 {
        return Err("Circle History needs 1 through 1024 recorded points".into());
    }
    for &p in points {
        check_point(p)?;
    }
    let mut options = recipe.options.clone();
    // A native Direction source supplies the current endpoint-minus-center
    // vector. Circle::pick expects a unit normal before projecting radius picks.
    if let Some(normal) = options.normal {
        options.normal = Some(Plan::new([0.; 3], normal, 1.)?.normal);
    }
    options.orientating = false;
    options.oriented = false;
    options.radius_pending = false;
    options.constraints.clear();
    let mut fit_deviation = 0.;
    let plan = if options.mode == Mode::Tangent {
        if points.len()
            != if options.radius_constraint.is_some() {
                2
            } else {
                3
            }
        {
            return Err("Circle History tangent constraint count is invalid".into());
        }
        let p = native_plan.ok_or("Circle History tangent needs native constraint replay")?;
        let plan = Plan::new([p[0], p[1], p[2]], [p[3], p[4], p[5]], p[6])?;
        if options.tangent_vertical
            && plan
                .normal
                .iter()
                .zip(options.axes()[2])
                .map(|(a, b)| a * b)
                .sum::<f64>()
                .abs()
                > 1e-8
        {
            return Err("Circle History tangent Vertical needs a perpendicular plane".into());
        }
        if options.from_first {
            let d: Point = std::array::from_fn(|i| points[0][i] - plan.center[i]);
            let height = d.iter().zip(plan.normal).map(|(a, b)| a * b).sum::<f64>();
            let length = d.iter().map(|v| v * v).sum::<f64>().sqrt();
            if height.abs() > 1e-6 || (length - plan.radius).abs() > 1e-6 * plan.radius.max(1.) {
                return Err("Circle History fixed first contact is off its circle".into());
            }
        }
        if let Some(radius) = options.radius_constraint {
            if (radius - plan.radius).abs() > 1e-6 * radius.max(1.) {
                return Err("Circle History tangent radius constraint changed".into());
            }
        }
        plan
    } else if options.mode == Mode::FitPoints {
        let (plan, deviation) = fit_circle(points)?;
        fit_deviation = deviation;
        plan
    } else {
        let mut picks = points.to_vec();
        if options.mode == Mode::AroundCurve {
            let p = native_plan.ok_or("Circle History AroundCurve needs native path replay")?;
            let validated = Plan::new([p[0], p[1], p[2]], [p[3], p[4], p[5]], p[6])?;
            picks[0] = validated.center;
            options.normal = Some(validated.normal);
        }
        if let Some(radius) = recipe.numeric_radius {
            if picks.len() != 1 {
                return Err("Numeric Circle History needs one center".into());
            }
            Plan::new(
                picks[0],
                options.normal.unwrap_or(if options.mode == Mode::Vertical {
                    options.axes()[1]
                } else {
                    options.axes()[2]
                }),
                radius,
            )?
        } else {
            let expected = if options.mode == Mode::ThreePoint {
                3
            } else {
                2
            };
            if picks.len() != expected {
                return Err("Circle History has an invalid picked point count".into());
            }
            if let Some(radius) = options.radius_constraint {
                if options.mode == Mode::ThreePoint {
                    options.set_radius(&picks[..2], radius)?;
                }
            }
            if options.mode == Mode::AroundCurve {
                options.mode = Mode::Center;
            }
            let mut accumulated = Vec::new();
            let mut result = None;
            for p in picks {
                result = options.pick(&mut accumulated, p)?;
            }
            result.ok_or("Circle History recipe did not produce geometry")?
        }
    };
    let mut approx_deviation = 0.;
    let spline = if options.deformable {
        let mut curve = spline::rebuild(
            &Plan::new([0.; 3], plan.normal, 1.)?.sample(1024),
            options.point_count,
            options.degree,
            true,
        )?;
        for pole in &mut curve.poles {
            for (axis, value) in pole.iter_mut().enumerate() {
                *value = plan.center[axis] + plan.radius * *value;
            }
            check_point(*pole)?;
        }
        for i in 0..1024 {
            let p = curve.value(i as f64 / 1024.);
            let d: Point = std::array::from_fn(|j| p[j] - plan.center[j]);
            let h: f64 = d.iter().zip(plan.normal).map(|(a, b)| a * b).sum();
            let r = (d.iter().map(|v| v * v).sum::<f64>() - h * h)
                .max(0.)
                .sqrt();
            approx_deviation = f64::max(approx_deviation, (r - plan.radius).hypot(h));
        }
        Some(curve)
    } else {
        None
    };
    Ok(Replay {
        plan,
        spline,
        fit_deviation,
        approx_deviation,
    })
}
impl Replay {
    pub fn encode(&self) -> Result<Vec<f64>, String> {
        let mut out = Vec::with_capacity(OUTPUT_LEN);
        out.extend(self.plan.center);
        out.extend(self.plan.normal);
        out.extend([self.plan.radius, self.fit_deviation, self.approx_deviation]);
        if let Some(spline) = &self.spline {
            let (knots, mults) = spline.host_knots();
            out.extend([
                spline.degree as f64,
                spline.poles.len() as f64,
                knots.len() as f64,
                f64::from(spline.periodic),
            ]);
            out.extend(spline.poles.iter().flatten().copied());
            out.extend(knots);
            out.extend(mults.iter().map(|m| *m as f64));
        } else {
            out.extend([0.; 4]);
        }
        if out.len() > OUTPUT_LEN {
            return Err("Circle History spline output exceeds bounded buffer".into());
        }
        out.resize(OUTPUT_LEN, 0.);
        Ok(out)
    }
}
/// # Safety
/// config has 24 readable doubles; points has count*3 readable doubles with
/// count<=1024; native_plan is null or seven readable doubles; output has 4096
/// writable doubles. Buffers are aligned, valid for this call and do not alias.
/// Failure leaves output unchanged and never touches live command/global spline.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_circle_history_replay(
    config: *const f64,
    points: *const f64,
    count: usize,
    native_plan: *const f64,
    output: *mut f64,
) -> bool {
    if config.is_null() || points.is_null() || output.is_null() || count == 0 || count > 1024 {
        return false;
    }
    let result = std::panic::catch_unwind(|| {
        let recipe = Recipe::decode(unsafe { std::slice::from_raw_parts(config, CONFIG_LEN) })?;
        let points = unsafe { std::slice::from_raw_parts(points, count * 3) }
            .as_chunks::<3>()
            .0;
        let native = if native_plan.is_null() {
            None
        } else {
            Some(unsafe { std::ptr::read(native_plan.cast::<[f64; 7]>()) })
        };
        replay(&recipe, points, native)?.encode()
    });
    if let Ok(Ok(encoded)) = result {
        unsafe { std::ptr::copy_nonoverlapping(encoded.as_ptr(), output, OUTPUT_LEN) };
        true
    } else {
        false
    }
}
/// # Safety
/// anchor and endpoint each address three readable doubles. Data are copied
/// immediately; no live command state is used. Invalid bounds return NaN.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_circle_history_radius(
    anchor: *const f64,
    endpoint: *const f64,
) -> f64 {
    if anchor.is_null() || endpoint.is_null() {
        return f64::NAN;
    }
    let a = unsafe { std::ptr::read(anchor.cast::<Point>()) };
    let b = unsafe { std::ptr::read(endpoint.cast::<Point>()) };
    if check_point(a).is_err() || check_point(b).is_err() {
        return f64::NAN;
    }
    let radius = a
        .iter()
        .zip(b)
        .map(|(a, b)| (a - b) * (a - b))
        .sum::<f64>()
        .sqrt();
    if (1e-7..=1e9).contains(&radius) {
        radius
    } else {
        f64::NAN
    }
}
