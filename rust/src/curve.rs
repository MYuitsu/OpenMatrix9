//! OM9-CURVE-001/002/003: shared mouse and command-frame session.
//! Host choices: millimetres, 1e-7 mm minimum segment, finite coordinates <1e9 mm.
use crate::circle::{Circle, Mode as CircleMode, Plan as CirclePlan, Size as CircleSize};
use crate::coordinates::{Frame, PointInput};
use crate::ellipse::{Ellipse, Mode as EllipseMode, Plan as EllipsePlan};
use crate::rectangle::{Mode, Rectangle};
use crate::spline::{self, Knots, Spline};
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Effect {
    Waiting,
    Commit,
    Cancelled,
}
#[derive(Default, Clone)]
pub struct CurveSession {
    name: &'static str,
    active: bool,
    points: Vec<[f64; 3]>,
    output: Vec<[f64; 3]>,
    both_sides: bool,
    persistent_close: bool,
    length: Option<f64>,
    awaiting_length: bool,
    input_mm_per_unit: f64,
    frame: Option<Frame>,
    degree: usize,
    knots: Knots,
    spline: Option<Spline>,
    awaiting_option: Option<String>,
    rectangle: Rectangle,
    pub(crate) circle_options: Circle,
    circle_plan: Option<CirclePlan>,
    circle_approx_deviation: f64,
    ellipse_options: Ellipse,
    ellipse_plan: Option<EllipsePlan>,
    ellipse_approx_deviation: f64,
}
pub fn name(icon: &str) -> Option<&'static str> {
    match icon {
        "CurvePolylinePolyline" => Some("Polyline"),
        "CurveLineSingleLine" => Some("Line"),
        "CurveFreeFormInterpolatePoints" => Some("InterpCrv"),
        "OthersCurveRebuild" => Some("Rebuild"),
        "CurveRectangleCornertoCorner" => Some("Rectangle"),
        "CurveCircleCenterRadius" => Some("Circle"),
        "CurveEllipseFromCenter" => Some("Ellipse"),
        _ => None,
    }
}
impl CurveSession {
    pub fn start(&mut self, name: &str) -> Result<(), String> {
        let name = if name.eq_ignore_ascii_case("Line") {
            "Line"
        } else if name.eq_ignore_ascii_case("Polyline") {
            "Polyline"
        } else if name.eq_ignore_ascii_case("InterpCrv") {
            "InterpCrv"
        } else if name.eq_ignore_ascii_case("Rectangle") {
            "Rectangle"
        } else if name.eq_ignore_ascii_case("Circle") {
            "Circle"
        } else if name.eq_ignore_ascii_case("Ellipse") {
            "Ellipse"
        } else {
            return Err("Unknown Curve command".into());
        };
        *self = Self {
            name,
            active: true,
            degree: 3,
            input_mm_per_unit: 1.0,
            ..Self::default()
        };
        Ok(())
    }
    pub fn active(&self) -> bool {
        self.active
    }
    pub fn set_input_scale(&mut self, scale: f64) -> Result<(), String> {
        if !self.active || !self.points.is_empty() || !scale.is_finite() || scale <= 0.0 {
            return Err(
                "Input units must be finite, positive and captured before the first point".into(),
            );
        }
        self.input_mm_per_unit = scale;
        Ok(())
    }
    fn length_value(&self, text: &str) -> Result<f64, String> {
        let value =
            crate::units::parse_length(text, self.input_mm_per_unit).map_err(|e| e.to_string())?;
        if value.abs() > 1e9 {
            return Err("Invalid coordinate or length".into());
        }
        Ok(value)
    }
    fn dimension_text(&self, text: &str, area: bool) -> Result<String, String> {
        if let Ok(value) = text.trim().parse::<f64>() {
            let scale = if area {
                self.input_mm_per_unit * self.input_mm_per_unit
            } else {
                self.input_mm_per_unit
            };
            let value = value * scale;
            if !value.is_finite() || !scale.is_finite() {
                return Err("Unit conversion overflow".into());
            }
            Ok(format!("{}{}", value, if area { "mm2" } else { "mm" }))
        } else {
            Ok(text.to_owned())
        }
    }
    pub fn set_frame(&mut self, origin: [f64; 3], axes: [[f64; 3]; 3]) -> Result<(), String> {
        let frame = Frame::new(origin, axes)?;
        if matches!(self.name, "Rectangle" | "Circle" | "Ellipse") && !self.points.is_empty() {
            return Ok(());
        }
        self.frame = Some(frame);
        Ok(())
    }
    pub fn name(&self) -> &'static str {
        self.name
    }
    pub fn output(&self) -> &[[f64; 3]] {
        &self.output
    }
    pub fn spline(&self) -> Option<&Spline> {
        self.spline.as_ref()
    }
    pub fn circle(&self) -> Option<&CirclePlan> {
        self.circle_plan.as_ref()
    }
    pub fn ellipse(&self) -> Option<&EllipsePlan> {
        self.ellipse_plan.as_ref()
    }
    pub fn ellipse_deformable(&self) -> bool {
        self.name == "Ellipse" && self.ellipse_options.deformable
    }
    pub fn ellipse_mark_foci(&self) -> bool {
        self.name == "Ellipse"
            && self.ellipse_options.mode == EllipseMode::FromFoci
            && self.ellipse_options.mark_foci
    }
    pub fn ellipse_deviation(&self) -> f64 {
        self.ellipse_approx_deviation
    }
    pub fn ellipse_reference_mode(&self) -> u32 {
        if self.name == "Ellipse" && self.active {
            self.ellipse_options.reference_mode(&self.points)
        } else {
            0
        }
    }
    pub fn ellipse_native_reference(
        &mut self,
        p: [f64; 3],
        normal: [f64; 3],
        kind: u32,
    ) -> Result<Effect, String> {
        if self.name != "Ellipse" || !self.active {
            return Err("No active Ellipse".into());
        }
        let mut options = self.ellipse_options.clone();
        let mut points = self.points.clone();
        options.reference(&mut points, p, normal, kind)?;
        if self.points.is_empty() && kind == 2 {
            options.axes = self.frame.map_or(crate::rectangle::AXES, Frame::axes);
        }
        self.ellipse_options = options;
        self.points = points;
        Ok(Effect::Waiting)
    }
    pub fn preview_spline(&self, hover: Option<[f64; 3]>) -> Result<Spline, String> {
        self.preview_spline_closed(hover, false)
    }
    pub fn preview_spline_closed(
        &self,
        hover: Option<[f64; 3]>,
        close: bool,
    ) -> Result<Spline, String> {
        let mut points = self.points.clone();
        if let Some(p) = hover {
            if points
                .last()
                .is_none_or(|&a| spline::distance(a, p) >= spline::TOLERANCE)
            {
                points.push(p);
            }
        }
        let closed = (close || self.persistent_close) && points.len() >= 3;
        if closed
            && points.len() > 3
            && spline::distance(points[0], *points.last().unwrap()) < spline::TOLERANCE
        {
            points.pop();
        }
        spline::interpolate(&points, self.degree, self.knots, closed)
    }
    pub fn points(&self) -> &[[f64; 3]] {
        &self.points
    }
    pub fn cancel(&mut self) -> Effect {
        self.awaiting_length = false;
        self.length = None;
        self.active = false;
        self.points.clear();
        self.output.clear();
        self.spline = None;
        self.circle_plan = None;
        self.circle_options = Circle::default();
        self.circle_approx_deviation = 0.;
        self.ellipse_options = Ellipse::default();
        self.ellipse_plan = None;
        self.ellipse_approx_deviation = 0.;
        self.awaiting_option = None;
        Effect::Cancelled
    }
    pub fn prompt(&self) -> String {
        if !self.active {
            return "Command: ".into();
        }
        if self.name == "Ellipse" {
            if let Some(option) = &self.awaiting_option {
                return format!("Ellipse {option}: ");
            }
            return format!(
                "Ellipse {} (Mode={:?} Center Diameter Corner Vertical FromFoci MarkFoci={} AroundCurve Deformable={} Degree={} PointCount={} Undo): ",
                self.ellipse_options.step(&self.points),
                self.ellipse_options.mode,
                self.ellipse_options.mark_foci,
                self.ellipse_options.deformable,
                self.ellipse_options.degree,
                self.ellipse_options.point_count
            );
        }
        if self.name == "Circle" {
            let advanced = match self.circle_options.mode {
                CircleMode::AroundCurve => {
                    Some(match self.circle_options.reference_mode(&self.points) {
                        1 => "Select native curve (Curve=Object.EdgeN)",
                        2 => "Pick center on selected curve (OnCurve=fraction)",
                        _ => "Radius/Diameter point or value",
                    })
                }
                CircleMode::FitPoints => Some("Fit points / Selection / Enter to finish"),
                CircleMode::Tangent => Some(if self.circle_options.tangent_ready() {
                    "Solve tangent circle / Solution=n / Undo"
                } else if self.circle_options.radius_pending {
                    "Pick radius length from last constraint or enter radius"
                } else if self.circle_options.point_mode {
                    "Free point"
                } else {
                    "Pick tangent curve / Curve=Object.EdgeN@x,y,z / Point"
                }),
                CircleMode::ThreePoint if self.circle_options.radius_pending => {
                    Some("Pick radius length from second point or enter radius")
                }
                CircleMode::ThreePoint if self.circle_options.radius_constraint.is_some() => {
                    Some("Pick center-side direction to orient radius-constrained circle")
                }
                _ => None,
            };
            if let Some(step) = advanced {
                return format!(
                    "Circle {step} (Mode={:?} Radius={:?} Deformable={} Degree={} PointCount={} Vertical={} History={} Undo): ",
                    self.circle_options.mode,
                    self.circle_options.radius_constraint,
                    self.circle_options.deformable,
                    self.circle_options.degree,
                    self.circle_options.point_count,
                    self.circle_options.tangent_vertical,
                    self.circle_options.history
                );
            }
            let step = if self.circle_options.orientating {
                "Orientation direction point"
            } else if self.points.is_empty() {
                if self.circle_options.center_mode() {
                    "Center"
                } else {
                    "First circumference point"
                }
            } else if self.circle_options.center_mode() {
                "Size value or radius/diameter point"
            } else if self.points.len() == 1 {
                "Second circumference point"
            } else {
                "Third circumference point"
            };
            return format!(
                "Circle {step} ( Mode={:?} Size={:?} Radius Diameter Circumference Area 2Point 3Point Vertical Orientation AroundCurve FitPoints Tangent Deformable={} Degree={} PointCount={} Undo ): ",
                self.circle_options.mode,
                self.circle_options.size,
                self.circle_options.deformable,
                self.circle_options.degree,
                self.circle_options.point_count
            );
        }
        if self.name == "Rectangle" {
            if let Some(option) = &self.awaiting_option {
                return format!(
                    "Rectangle {option} (input unit = {} mm; explicit suffix allowed): ",
                    self.input_mm_per_unit
                );
            }
            let step = match self.points.len() {
                0 => "First corner / center",
                1 if matches!(self.rectangle.mode, Mode::Corners | Mode::Center) => {
                    "Opposite corner or Length"
                }
                1 => "End of first edge",
                _ => "Opposite side or Width",
            };
            return format!(
                "Rectangle {step} ( Mode={:?} Length={} Width={} 3Point Vertical Center Undo; Rounded unsupported ): ",
                self.rectangle.mode,
                self.rectangle
                    .length
                    .map_or("pick".into(), |v| v.to_string()),
                self.rectangle
                    .width
                    .map_or("pick".into(), |v| v.to_string())
            );
        }
        if let Some(option) = &self.awaiting_option {
            return format!("{option} of interpolated curve: ");
        }
        if self.awaiting_length {
            return "Length of next segment: ".into();
        }
        if !self.active {
            return "Command: ".into();
        }
        let state = if self.persistent_close { "Yes" } else { "No" };
        if self.name == "InterpCrv" {
            let knots = match self.knots {
                Knots::Uniform => "Uniform",
                Knots::Chord => "Chord",
                Knots::SqrtChord => "SqrtChrd",
            };
            return format!(
                "{} point of interpolated curve. Enter when done ( Degree={} Knots={} PersistentClose={} Close Sharp Undo ): ",
                if self.points.is_empty() {
                    "Start"
                } else {
                    "Next"
                },
                self.degree,
                knots,
                state
            );
        }
        if self.points.is_empty() {
            if self.name == "Line" {
                format!(
                    "Start of line ( BothSides={} ): ",
                    if self.both_sides { "Yes" } else { "No" }
                )
            } else {
                format!("Start of polyline ( PersistentClose={state} ): ")
            }
        } else if self.name == "Line" {
            "End of line: ".into()
        } else {
            let finish = if self.points.len() >= 2 {
                ". Press Enter when done"
            } else {
                ""
            };
            let close = if self.points.len() >= 3 { "Close " } else { "" };
            format!(
                "Next point of polyline{finish} ( PersistentClose={state} {close}Mode=Line Length Undo ): "
            )
        }
    }
    fn finish(&mut self, close: bool) -> Result<Effect, String> {
        if self.points.len() < 2 || (close && self.points.len() < 3) {
            return Err("More distinct points are required".into());
        }
        let mut points = self.points.clone();
        if self.name == "InterpCrv" {
            let closed = close || (self.persistent_close && points.len() >= 3);
            if closed
                && points.len() > 2
                && spline::distance(points[0], *points.last().unwrap()) < spline::TOLERANCE
            {
                points.pop();
            }
            let fitted = spline::interpolate(&points, self.degree, self.knots, closed)?;
            self.spline = Some(fitted);
            self.output = points;
            self.active = false;
            return Ok(Effect::Commit);
        }
        if close || (self.persistent_close && points.len() > 2) {
            let first = points[0];
            if points.last() != Some(&first) {
                points.push(first);
            }
        }
        if self.name == "Line" && self.both_sides {
            let a = points[0];
            let b = points[1];
            points[0] = [2. * a[0] - b[0], 2. * a[1] - b[1], 2. * a[2] - b[2]];
        }
        self.output = points;
        self.active = false;
        Ok(Effect::Commit)
    }
    pub fn mouse_point(&mut self, mut p: [f64; 3], ortho: bool) -> Result<Effect, String> {
        if matches!(self.name, "Circle" | "Ellipse") {
            return self.point(p);
        }
        if p.iter().any(|v| !v.is_finite() || v.abs() > 1e9) {
            return Err("Coordinate must be finite and within 1e9 mm".into());
        }
        if self.name == "Rectangle" {
            return self.rectangle_point(p, ortho);
        }
        if ortho && let Some(a) = self.points.last() {
            let axes = self
                .frame
                .map_or([[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]], Frame::axes);
            let delta = [p[0] - a[0], p[1] - a[1], p[2] - a[2]];
            let local = axes.map(|axis| axis.iter().zip(delta).map(|(x, y)| x * y).sum::<f64>());
            let chosen = usize::from(local[1].abs() > local[0].abs());
            p = std::array::from_fn(|i| a[i] + axes[chosen][i] * local[chosen]);
        }
        self.point(p)
    }
    /// Preview uses the same constraints as a click without changing the session.
    pub fn preview_point(&self, p: [f64; 3], ortho: bool) -> Result<[f64; 3], String> {
        let mut preview = self.clone();
        preview.mouse_point(p, ortho)?;
        preview
            .points
            .last()
            .copied()
            .ok_or("No preview point".into())
    }
    pub fn point(&mut self, mut p: [f64; 3]) -> Result<Effect, String> {
        if self.name == "Ellipse" {
            if !self.active {
                return Err("No active command".into());
            }
            if self.awaiting_option.is_some() {
                return Err("Enter the pending Ellipse option before picking".into());
            }
            let mut next = self.clone();
            if next.points.is_empty() {
                next.ellipse_options.axes = next.frame.map_or(crate::rectangle::AXES, Frame::axes);
            }
            let plan = next.ellipse_options.pick(&mut next.points, p)?;
            let effect = next.ellipse_finish(plan)?;
            *self = next;
            return Ok(effect);
        }
        if self.name == "Circle" {
            if !self.active {
                return Err("No active command".into());
            }
            if self.points.is_empty() {
                self.circle_options.axes =
                    Some(self.frame.map_or(crate::rectangle::AXES, Frame::axes));
            }
            let mut next = self.circle_options.clone();
            let mut points = self.points.clone();
            let plan = next.pick(&mut points, p)?;
            let effect = self.circle_finish(plan)?;
            self.circle_options = next;
            self.points = points;
            return Ok(effect);
        }
        if self.name == "Rectangle" {
            return self.rectangle_point(p, false);
        }
        if !self.active {
            return Err("No active command".into());
        }
        if self.awaiting_length {
            return Err("Enter the numeric length before picking a direction".into());
        }
        if self.awaiting_option.is_some() {
            return Err("Enter the pending option value before picking a point".into());
        }
        if p.iter().any(|x| !x.is_finite() || x.abs() > 1e9) {
            return Err("Coordinate must be finite and within 1e9 mm".into());
        }
        if self.points.len()
            >= if self.name == "InterpCrv" {
                spline::MAX_POLES
            } else {
                4096
            }
        {
            return Err("Point limit reached; finish or cancel".into());
        }
        if let Some(a) = self.points.last() {
            let d = [p[0] - a[0], p[1] - a[1], p[2] - a[2]];
            let n = d.iter().map(|v| v * v).sum::<f64>().sqrt();
            if n < 1e-7 {
                return Err("The next point must differ from the last point".into());
            }
            if let Some(length) = self.length {
                p = [
                    a[0] + d[0] * length / n,
                    a[1] + d[1] * length / n,
                    a[2] + d[2] * length / n,
                ];
            }
        }
        self.points.push(p);
        self.length = None;
        if self.name == "Line" && self.points.len() == 2 {
            self.finish(false)
        } else {
            Ok(Effect::Waiting)
        }
    }
    pub fn input(&mut self, text: &str) -> Result<Effect, String> {
        let text = text.trim();
        if text.eq_ignore_ascii_case("Esc") || text.eq_ignore_ascii_case("Cancel") {
            return Ok(self.cancel());
        }
        if !self.active {
            return Err("No active command".into());
        }
        if let Some(option) = self.awaiting_option.clone() {
            self.awaiting_option = None;
            let result = self.input(&format!("{option}={text}"));
            if result.is_err() {
                self.awaiting_option = Some(option);
            }
            return result;
        }
        if self.name == "Circle"
            && let Some(effect) = self.circle_input(text)?
        {
            return Ok(effect);
        }
        if self.name == "Ellipse"
            && let Some(effect) = self.ellipse_input(text)?
        {
            return Ok(effect);
        }
        if self.name == "Rectangle" && self.rectangle_input(text)?.is_some() {
            return Ok(if self.active {
                Effect::Waiting
            } else {
                Effect::Commit
            });
        }
        if self.awaiting_length {
            let length = self.length_value(text)?;
            if length < 1e-7 {
                return Err("Length must be positive".into());
            }
            self.length = Some(length);
            self.awaiting_length = false;
            return Ok(Effect::Waiting);
        }
        if text.is_empty() {
            return self.finish(false);
        }
        // Option abbreviations use the underlined initial shown in the live prompt.
        let lower = text.to_ascii_lowercase();
        let (key, value) = lower
            .split_once('=')
            .map_or((lower.as_str(), None), |(k, v)| (k.trim(), Some(v.trim())));
        if self.name == "InterpCrv" {
            match key {
                "degree" | "d" | "knots" | "k" if value.is_none() => {
                    self.awaiting_option = Some(
                        if key == "degree" || key == "d" {
                            "Degree"
                        } else {
                            "Knots"
                        }
                        .into(),
                    );
                    return Ok(Effect::Waiting);
                }
                "degree" | "d" => {
                    let degree = value
                        .unwrap()
                        .parse::<usize>()
                        .map_err(|_| "Enter an odd degree from 1 to 11")?;
                    if !(1..=11).contains(&degree) || degree % 2 == 0 {
                        return Err("Enter an odd degree from 1 to 11".into());
                    }
                    self.degree = degree;
                    return Ok(Effect::Waiting);
                }
                "knots" | "k" => {
                    self.knots = match value {
                        Some("uniform" | "u") => Knots::Uniform,
                        Some("chord" | "c") => Knots::Chord,
                        Some("sqrtchrd" | "sqrtchord" | "s") => Knots::SqrtChord,
                        _ => return Err("Knots must be Uniform, Chord or SqrtChrd".into()),
                    };
                    return Ok(Effect::Waiting);
                }
                "persistentclose" | "p" => {
                    self.persistent_close = option_bool(value, self.persistent_close)?;
                    return Ok(Effect::Waiting);
                }
                "close" | "c" if value.is_none() => return self.finish(true),
                "sharp" | "s" if value.is_none() => {
                    if self.points.len() < 3 {
                        return Err("Sharp needs at least 3 points".into());
                    }
                    let mut points = self.points.clone();
                    if spline::distance(points[0], *points.last().unwrap()) >= spline::TOLERANCE {
                        points.push(points[0]);
                    }
                    let fitted = spline::interpolate(&points, self.degree, self.knots, false)?;
                    self.output = points;
                    self.spline = Some(fitted);
                    self.active = false;
                    return Ok(Effect::Commit);
                }
                "undo" | "u" if value.is_none() => {
                    self.points.pop().ok_or("No point to undo")?;
                    return Ok(Effect::Waiting);
                }
                "starttangent" | "endtangent" => {
                    return Err("Tangent constraints are not implemented".into());
                }
                _ => {}
            }
        }
        if self.name == "Polyline" && (key == "close" || key == "c") && value.is_none() {
            return self.finish(true);
        }
        if self.name == "Polyline" && (key == "undo" || key == "u") && value.is_none() {
            self.points.pop().ok_or("No point to undo")?;
            self.length = None;
            return Ok(Effect::Waiting);
        }
        if (key == "bothsides" || key == "b") && self.name == "Line" && self.points.is_empty() {
            self.both_sides = option_bool(value, self.both_sides)?;
            return Ok(Effect::Waiting);
        }
        if (key == "persistentclose" || key == "p") && self.name == "Polyline" {
            self.persistent_close = option_bool(value, self.persistent_close)?;
            return Ok(Effect::Waiting);
        }
        if (key == "mode" || key == "m") && self.name == "Polyline" && !self.points.is_empty() {
            if value.is_none() || matches!(value, Some("line" | "l")) {
                return Ok(Effect::Waiting);
            }
            return Err("Only Mode=Line is implemented".into());
        }
        if (key == "length" || key == "l") && self.name == "Polyline" && !self.points.is_empty() {
            if let Some(value) = value {
                let length = self.length_value(value)?;
                if length < 1e-7 {
                    return Err("Length must be positive".into());
                }
                self.length = Some(length);
            } else {
                self.awaiting_length = true;
            }
            return Ok(Effect::Waiting);
        }
        let point = PointInput::parse(text, |value| self.length_value(value))?
            .resolve(self.frame.unwrap_or_default(), self.points.last().copied())?;
        self.point(point.world)
    }
    fn rectangle_point(&mut self, p: [f64; 3], square: bool) -> Result<Effect, String> {
        if !self.active {
            return Err("No active command".into());
        }
        if self.awaiting_option.is_some() {
            return Err("Enter the pending dimension before picking".into());
        }
        if p.iter().any(|v| !v.is_finite() || v.abs() > 1e9) {
            return Err("Invalid Rectangle coordinate".into());
        }
        if self.points.is_empty() {
            self.rectangle.axes = Some(self.frame.map_or(crate::rectangle::AXES, Frame::axes));
            self.points.push(p);
            return Ok(Effect::Waiting);
        }
        if self.points.len() == 1
            && matches!(self.rectangle.mode, Mode::ThreePoint | Mode::Vertical)
        {
            let edge = self.rectangle.edge(self.points[0], p)?;
            let adjusted: [f64; 3] = std::array::from_fn(|i| self.points[0][i] + edge[i]);
            if adjusted.iter().any(|v| !v.is_finite() || v.abs() > 1e9) {
                return Err("Rectangle edge exceeds coordinate limits".into());
            }
            let mut points = self.points.clone();
            points.push(adjusted);
            let mut rectangle = self.rectangle.clone();
            if square && let Some(width) = rectangle.width {
                rectangle.width =
                    Some(edge.iter().map(|v| v * v).sum::<f64>().sqrt() * width.signum());
            }
            let output = rectangle.numeric_geometry(&points)?;
            self.points = points;
            if let Some(output) = output {
                self.output = output;
                self.active = false;
                return Ok(Effect::Commit);
            }
            return Ok(Effect::Waiting);
        }
        let output = self.rectangle.geometry(&self.points, p, square)?;
        self.points.push(p);
        self.output = output;
        self.active = false;
        Ok(Effect::Commit)
    }
    fn rectangle_input(&mut self, text: &str) -> Result<Option<()>, String> {
        if text.is_empty() {
            return Err("Rectangle needs complete sides; pick a corner or enter dimensions".into());
        }
        let lower = text.to_ascii_lowercase();
        let (key, value) = lower
            .split_once('=')
            .map_or((lower.as_str(), None), |(a, b)| (a.trim(), Some(b.trim())));
        match key {
            "3point" | "3p" | "vertical" | "v" | "center" | "c" | "corners" => {
                if !self.points.is_empty() || value.is_some() {
                    return Err("Choose the Rectangle mode before the first point".into());
                }
                self.rectangle.mode = match key {
                    "3point" | "3p" => Mode::ThreePoint,
                    "vertical" | "v" => Mode::Vertical,
                    "center" | "c" => Mode::Center,
                    _ => Mode::Corners,
                };
                return Ok(Some(()));
            }
            "undo" | "u" if value.is_none() => {
                self.points.pop().ok_or("No Rectangle point to undo")?;
                self.rectangle.length = None;
                self.rectangle.width = None;
                return Ok(Some(()));
            }
            "rounded" | "corner" | "radius" | "arc" | "conic" => {
                return Err("Rounded/Arc/Conic Rectangle is not implemented".into());
            }
            "length" | "l" | "width" | "w" => {
                if self.points.is_empty() {
                    return Err("Pick the first corner before entering dimensions".into());
                }
                if (key == "length" || key == "l") && self.points.len() > 1 {
                    return Err("First edge is already fixed; Undo to change Length".into());
                }
                if value.is_none() {
                    self.awaiting_option = Some(
                        if key == "length" || key == "l" {
                            "Length"
                        } else {
                            "Width"
                        }
                        .into(),
                    );
                    return Ok(Some(()));
                }
                let number = self.length_value(value.unwrap())?;
                return self
                    .rectangle_dimension(key == "length" || key == "l", number)
                    .map(|_| Some(()));
            }
            _ => {}
        }
        if !text.contains(',') {
            let number = self.length_value(text)?;
            return self
                .rectangle_dimension(
                    self.points.len() == 1 && self.rectangle.length.is_none(),
                    number,
                )
                .map(|_| Some(()));
        }
        Ok(None)
    }
    fn rectangle_dimension(&mut self, length: bool, value: f64) -> Result<(), String> {
        if self.points.is_empty() {
            return Err("Pick the first corner before entering dimensions".into());
        }
        if value.abs() < 1e-7 || (length && value < 0.) {
            return Err("Length must be positive; Width must be nonzero".into());
        }
        let mut next = self.rectangle.clone();
        if length {
            next.length = Some(value);
        } else {
            next.width = Some(value);
        }
        let output = next.numeric_geometry(&self.points)?;
        self.rectangle = next;
        if let Some(output) = output {
            self.output = output;
            self.active = false;
        }
        Ok(())
    }
    pub fn preview_geometry(&self, p: [f64; 3], square: bool) -> Result<Vec<[f64; 3]>, String> {
        let mut preview = self.clone();
        preview.mouse_point(p, square)?;
        if !preview.output.is_empty() {
            Ok(preview.output)
        } else {
            Ok(preview.points)
        }
    }
    pub fn rectangle_pick_plane(&self) -> Option<([f64; 3], [f64; 3])> {
        if self.name == "Ellipse" && self.active {
            return self.ellipse_options.pick_plane(&self.points);
        }
        if self.name == "Circle" && self.active {
            return self.circle_options.pick_plane(&self.points);
        }
        if self.name != "Rectangle" || !self.active {
            return None;
        }
        Some((
            *self.points.first()?,
            self.rectangle.pick_normal(&self.points).ok()?,
        ))
    }
    fn ellipse_finish(&mut self, plan: Option<EllipsePlan>) -> Result<Effect, String> {
        let Some(plan) = plan else {
            return Ok(Effect::Waiting);
        };
        let mut deviation = 0.;
        let mut fitted = None;
        if self.ellipse_options.deformable {
            let mut candidate = spline::rebuild(
                &CirclePlan::new([0.; 3], [0., 0., 1.], 1.)?.sample(1024),
                self.ellipse_options.point_count,
                self.ellipse_options.degree,
                true,
            )?;
            for pole in &mut candidate.poles {
                *pole = plan.transform_unit(*pole);
                crate::circle::check_point(*pole)?;
            }
            for i in 0..1024 {
                deviation = f64::max(
                    deviation,
                    spline::distance(
                        candidate.value(i as f64 / 1024.),
                        plan.value(i as f64 / 1024.),
                    ),
                );
            }
            fitted = Some(candidate);
        }
        let output = if let Some(s) = &fitted {
            let mut p = (0..128)
                .map(|i| s.value(i as f64 / 128.))
                .collect::<Vec<_>>();
            p.push(p[0]);
            p
        } else {
            plan.sample()
        };
        for &p in &output {
            crate::circle::check_point(p)?;
        }
        self.output = output;
        self.spline = fitted;
        self.ellipse_plan = Some(plan);
        self.ellipse_approx_deviation = deviation;
        self.active = false;
        Ok(Effect::Commit)
    }
    fn ellipse_input(&mut self, text: &str) -> Result<Option<Effect>, String> {
        if text.is_empty() {
            return Err("Ellipse needs complete axes; pick a point or enter an axis length".into());
        }
        let lower = text.to_ascii_lowercase();
        let (key, value) = lower
            .split_once('=')
            .map_or((lower.as_str(), None), |(a, b)| (a.trim(), Some(b.trim())));
        match key {
            "center" | "diameter" | "corner" | "vertical" | "fromfoci" | "aroundcurve" => {
                if !self.points.is_empty() || value.is_some() {
                    return Err(
                        "Choose Ellipse mode before the first point; Undo to change it".into(),
                    );
                }
                let mode = match key {
                    "diameter" => EllipseMode::Diameter,
                    "corner" => EllipseMode::Corner,
                    "vertical" => EllipseMode::Vertical,
                    "fromfoci" => EllipseMode::FromFoci,
                    "aroundcurve" => EllipseMode::AroundCurve,
                    _ => EllipseMode::Center,
                };
                // A new construction mode must not inherit a selected path.
                self.ellipse_options.set_mode(mode);
            }
            "undo" | "u" if value.is_none() => {
                self.ellipse_options.undo(&mut self.points)?;
            }
            "deformable" => {
                self.ellipse_options.deformable =
                    option_bool(value, self.ellipse_options.deformable)?;
            }
            "markfoci" => {
                if self.ellipse_options.mode != EllipseMode::FromFoci {
                    return Err("MarkFoci is available with FromFoci".into());
                }
                self.ellipse_options.mark_foci =
                    option_bool(value, self.ellipse_options.mark_foci)?;
            }
            "degree" | "pointcount" => {
                if value.is_none() {
                    self.awaiting_option = Some(
                        if key == "degree" {
                            "Degree"
                        } else {
                            "PointCount"
                        }
                        .into(),
                    );
                    return Ok(Some(Effect::Waiting));
                }
                let n = value
                    .unwrap()
                    .parse::<usize>()
                    .map_err(|_| "Enter an integer Degree or PointCount")?;
                if key == "degree" {
                    if !(1..=11).contains(&n) || n >= self.ellipse_options.point_count {
                        return Err("Degree must be 1..11 and less than PointCount".into());
                    }
                    self.ellipse_options.degree = n;
                } else {
                    if n < 4 || n > 256 || n <= self.ellipse_options.degree {
                        return Err("PointCount must be 4..256 and greater than Degree".into());
                    }
                    self.ellipse_options.point_count = n;
                }
            }
            "history" => return Err("Ellipse History is not implemented".into()),
            _ => {
                if text.contains(',') {
                    return Ok(None);
                }
                let size = self.length_value(text)?;
                let p = self.ellipse_options.numeric_point(&self.points, size)?;
                return self.point(p).map(Some);
            }
        }
        Ok(Some(Effect::Waiting))
    }
    fn circle_finish(&mut self, plan: Option<CirclePlan>) -> Result<Effect, String> {
        let Some(plan) = plan else {
            return Ok(Effect::Waiting);
        };
        let mut deviation = 0.;
        let mut spline = None;
        if self.circle_options.deformable {
            // Internal samples are normalized; user-point separation tolerance
            // must not reject a valid tiny radius or large translated center.
            let mut candidate = spline::rebuild(
                &CirclePlan::new([0.; 3], plan.normal, 1.)?.sample(1024),
                self.circle_options.point_count,
                self.circle_options.degree,
                true,
            )?;
            for pole in &mut candidate.poles {
                for (i, value) in pole.iter_mut().enumerate() {
                    *value = plan.center[i] + plan.radius * *value;
                }
            }
            for &pole in &candidate.poles {
                crate::circle::check_point(pole)?;
            }
            for i in 0..1024 {
                let p = candidate.value(i as f64 / 1024.);
                let d: [f64; 3] = std::array::from_fn(|j| p[j] - plan.center[j]);
                let height: f64 = d.iter().zip(plan.normal).map(|(a, b)| a * b).sum();
                let radial = (d.iter().map(|v| v * v).sum::<f64>() - height * height)
                    .max(0.)
                    .sqrt();
                deviation = f64::max(deviation, (radial - plan.radius).hypot(height));
            }
            spline = Some(candidate);
        }
        let output = if let Some(curve) = &spline {
            (0..=128).map(|i| curve.value(i as f64 / 128.)).collect()
        } else {
            plan.outline()
        };
        self.output = output;
        self.spline = spline;
        self.circle_plan = Some(plan);
        self.circle_approx_deviation = deviation;
        self.active = false;
        Ok(Effect::Commit)
    }
    pub fn circle_reference_mode(&self) -> u32 {
        if self.name == "Circle" && self.active {
            self.circle_options.reference_mode(&self.points)
        } else {
            0
        }
    }
    pub fn circle_native_reference(
        &mut self,
        p: [f64; 3],
        normal: [f64; 3],
        kind: u32,
    ) -> Result<Effect, String> {
        if self.name != "Circle" || !self.active {
            return Err("No active Circle".into());
        }
        if self.points.is_empty() {
            self.circle_options.axes = Some(self.frame.map_or(crate::rectangle::AXES, Frame::axes));
        }
        crate::circle::check_point(p)?;
        if kind == 4 && self.circle_options.orientating {
            let plan = self.circle_options.pick(&mut self.points, p)?;
            return self.circle_finish(plan);
        }
        match (self.circle_reference_mode(), kind) {
            (1, 1) => {
                self.circle_options.path_selected = true;
            }
            (2, 2) => {
                let plan = CirclePlan::new(p, normal, 1e-7)?;
                self.circle_options.normal = Some(plan.normal);
                self.points.push(p);
            }
            (3, 3) => {
                self.circle_options.constraint(&mut self.points, p, false)?;
            }
            (3, 5) => {
                self.circle_options.constraint(&mut self.points, p, true)?;
            }
            _ => return Err("Circle reference does not match current construction step".into()),
        }
        Ok(Effect::Waiting)
    }
    pub fn circle_fit_batch(&mut self, points: &[[f64; 3]]) -> Result<Effect, String> {
        if self.circle_reference_mode() != 4
            || points.is_empty()
            || self.points.len() + points.len() > 1024
        {
            return Err("FitPoints selection requires 1 through 1024 total points".into());
        }
        for &p in points {
            crate::circle::check_point(p)?;
        }
        if self.points.is_empty() {
            self.circle_options.axes = Some(self.frame.map_or(crate::rectangle::AXES, Frame::axes));
        }
        self.points.extend_from_slice(points);
        Ok(Effect::Waiting)
    }
    pub fn circle_accept_solution(
        &mut self,
        center: [f64; 3],
        radius: f64,
    ) -> Result<Effect, String> {
        self.circle_accept_spatial_solution(center, self.circle_options.axes()[2], radius)
    }
    pub fn circle_accept_spatial_solution(
        &mut self,
        center: [f64; 3],
        normal: [f64; 3],
        radius: f64,
    ) -> Result<Effect, String> {
        if !self.active || !self.circle_options.tangent_ready() {
            return Err("No pending tangent solve".into());
        }
        if let Some(r) = self.circle_options.radius_constraint
            && (r - radius).abs() > 1e-6 * f64::max(1., r)
        {
            return Err("Native solution violates Radius constraint".into());
        }
        let plan = CirclePlan::new(center, normal, radius)?;
        if self.circle_options.tangent_vertical
            && plan
                .normal
                .iter()
                .zip(self.circle_options.axes()[2])
                .map(|(a, b)| a * b)
                .sum::<f64>()
                .abs()
                > 1e-8
        {
            return Err("Tangent Vertical requires a plane perpendicular to CPlane".into());
        }
        for (i, (p, free)) in self.circle_options.constraints.iter().enumerate() {
            if *free || (i == 0 && self.circle_options.from_first) {
                let d = std::array::from_fn::<_, 3, _>(|i| p[i] - center[i]);
                if d.iter()
                    .zip(plan.normal)
                    .map(|(a, b)| a * b)
                    .sum::<f64>()
                    .abs()
                    > 1e-6
                    || (d.iter().map(|n| n * n).sum::<f64>().sqrt() - radius).abs()
                        > 1e-6 * f64::max(1., radius)
                {
                    return Err("Native solution violates a fixed tangent Point".into());
                }
            }
        }
        self.circle_finish(Some(plan))
    }
    pub fn circle_history(&self) -> bool {
        self.circle_options.history
    }
    pub fn circle_solver_frame(&self) -> ([f64; 3], [[f64; 3]; 3]) {
        let origin = self
            .points
            .first()
            .copied()
            .unwrap_or(self.frame.map_or([0.; 3], Frame::origin));
        (
            origin,
            self.circle_options
                .axes
                .unwrap_or(self.frame.map_or(crate::rectangle::AXES, Frame::axes)),
        )
    }
    pub fn circle_deviations(&self) -> (f64, f64) {
        (
            self.circle_options.fit_deviation,
            self.circle_approx_deviation,
        )
    }
    pub fn circle_hover_plan(&self, p: [f64; 3]) -> Option<CirclePlan> {
        if self.name != "Circle" || !self.active {
            return None;
        }
        self.circle_options
            .clone()
            .pick(&mut self.points.clone(), p)
            .ok()
            .flatten()
    }
    fn circle_input(&mut self, text: &str) -> Result<Option<Effect>, String> {
        let lower = text.to_ascii_lowercase();
        let (key, value) = lower
            .split_once('=')
            .map_or((lower.as_str(), None), |(k, v)| (k.trim(), Some(v.trim())));
        if matches!(key, "vertical" | "v") && self.circle_options.mode == CircleMode::Tangent {
            self.circle_options.tangent_vertical =
                option_bool(value, self.circle_options.tangent_vertical)?;
            return Ok(Some(Effect::Waiting));
        }
        if key == "history" {
            let enabled = option_bool(value, self.circle_options.history)?;
            self.circle_options.history = enabled;
            return Ok(Some(Effect::Waiting));
        }
        if text.is_empty() {
            if self.circle_options.mode == CircleMode::FitPoints {
                let (plan, error) = crate::circle::fit_circle(&self.points)?;
                let effect = self.circle_finish(Some(plan))?;
                self.circle_options.fit_deviation = error;
                return Ok(Some(effect));
            }
            if self.circle_options.tangent_ready() {
                return Ok(Some(Effect::Waiting));
            }
            return Err("Circle needs complete points or a positive size".into());
        }
        let mode = match key {
            "2point" | "2p" => Some(CircleMode::TwoPoint),
            "3point" | "3p" => Some(CircleMode::ThreePoint),
            "vertical" | "v" => Some(CircleMode::Vertical),
            "center" => Some(CircleMode::Center),
            "aroundcurve" => Some(CircleMode::AroundCurve),
            "fitpoints" => Some(CircleMode::FitPoints),
            "tangent" => Some(CircleMode::Tangent),
            _ => None,
        };
        if let Some(mode) = mode {
            if !self.points.is_empty() || value.is_some() {
                return Err("Choose Circle mode before the first point".into());
            }
            let old = &self.circle_options;
            self.circle_options = Circle {
                mode,
                deformable: old.deformable,
                degree: old.degree,
                point_count: old.point_count,
                history: old.history,
                ..Circle::default()
            };
            return Ok(Some(Effect::Waiting));
        }
        if key == "deformable" {
            let enabled = option_bool(value, self.circle_options.deformable)?;
            self.circle_options.deformable = enabled;
            return Ok(Some(Effect::Waiting));
        }
        if key == "degree" || key == "pointcount" {
            let Some(value) = value else {
                self.awaiting_option = Some(key.into());
                return Ok(Some(Effect::Waiting));
            };
            let n = value
                .parse::<usize>()
                .map_err(|_| "Enter a positive integer")?;
            let (count, degree) = if key == "degree" {
                (self.circle_options.point_count, n)
            } else {
                (n, self.circle_options.degree)
            };
            spline::rebuild_options(count, degree)?;
            self.circle_options.point_count = count;
            self.circle_options.degree = degree;
            return Ok(Some(Effect::Waiting));
        }
        if key == "undo" || key == "u" {
            if value.is_some() {
                return Err("Undo takes no value".into());
            }
            self.circle_options.undo(&mut self.points)?;
            return Ok(Some(Effect::Waiting));
        }
        if key == "point" {
            if self.circle_options.mode != CircleMode::Tangent
                || value.is_some()
                || self.circle_options.tangent_ready()
            {
                return Err("Point selects the next free tangent constraint".into());
            }
            self.circle_options.point_mode = true;
            return Ok(Some(Effect::Waiting));
        }
        if key == "fromfirstpoint" {
            if self.circle_options.mode != CircleMode::Tangent {
                return Err("FromFirstPoint requires Tangent mode".into());
            }
            let enabled = option_bool(value, self.circle_options.from_first)?;
            if enabled && self.circle_options.constraints.first().is_some_and(|p| p.1) {
                return Err("First tangent constraint must be a curve".into());
            }
            self.circle_options.from_first = enabled;
            return Ok(Some(Effect::Waiting));
        }
        if key == "solution" {
            if !self.circle_options.tangent_ready() {
                return Err("Solution requires completed tangent constraints".into());
            }
            let n = value
                .ok_or("Enter Solution=n")?
                .parse::<usize>()
                .map_err(|_| "Solution is a positive integer")?;
            if n == 0 || n > 10000 {
                return Err("Solution must be 1 through 10000".into());
            }
            self.circle_options.solution = Some(n - 1);
            return Ok(Some(Effect::Waiting));
        }
        let size = match key {
            "radius" | "r" => Some(CircleSize::Radius),
            "diameter" | "d" => Some(CircleSize::Diameter),
            "circumference" | "c" => Some(CircleSize::Circumference),
            "area" | "a" => Some(CircleSize::Area),
            _ => None,
        };
        if (size == Some(CircleSize::Radius) || self.circle_options.radius_pending)
            && matches!(
                self.circle_options.mode,
                CircleMode::ThreePoint | CircleMode::Tangent
            )
        {
            if let Some(value) = value {
                self.circle_options.set_radius(
                    &self.points,
                    crate::circle::dimension(&self.dimension_text(value, false)?, false)?,
                )?;
            } else if size.is_some() {
                if self.circle_options.mode == CircleMode::ThreePoint && self.points.len() != 2 {
                    return Err("Pick two circumference points before Radius".into());
                }
                self.circle_options.radius_pending = true;
            } else if !text.contains(',') {
                self.circle_options.set_radius(
                    &self.points,
                    crate::circle::dimension(&self.dimension_text(text, false)?, false)?,
                )?;
            } else {
                return Ok(None);
            }
            return Ok(Some(Effect::Waiting));
        }
        if let Some(size) = size {
            if !self.circle_options.center_mode() {
                return Err("Size requires Center, Vertical or AroundCurve mode".into());
            }
            let mut next = self.circle_options.clone();
            next.size = size;
            if let Some(value) = value {
                let plan = next.numeric(
                    &self.points,
                    &self.dimension_text(value, next.size == CircleSize::Area)?,
                )?;
                let effect = self.circle_finish(Some(plan))?;
                self.circle_options = next;
                return Ok(Some(effect));
            }
            self.circle_options = next;
            return Ok(Some(Effect::Waiting));
        }
        if matches!(key, "orientation" | "o") && value.is_none() {
            if self.circle_options.mode != CircleMode::Center || self.points.len() != 1 {
                return Err("Orientation needs a center in Center mode".into());
            }
            self.circle_options.orientating = true;
            return Ok(Some(Effect::Waiting));
        }
        if !text.contains(',') {
            let plan = self.circle_options.numeric(
                &self.points,
                &self.dimension_text(text, self.circle_options.size == CircleSize::Area)?,
            )?;
            return Ok(Some(self.circle_finish(Some(plan))?));
        }
        Ok(None)
    }
}
fn option_bool(value: Option<&str>, current: bool) -> Result<bool, String> {
    match value {
        None => Ok(!current),
        Some("yes" | "y") => Ok(true),
        Some("no" | "n") => Ok(false),
        _ => Err("Option must be Yes or No".into()),
    }
}
