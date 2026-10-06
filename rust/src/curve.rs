//! OM9-CURVE-001/002: shared mouse and command-frame session.
//! Host choices: millimetres, 1e-7 mm minimum segment, finite coordinates <1e9 mm.
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
    frame: Option<([f64; 3], [[f64; 3]; 3])>,
}
pub fn name(icon: &str) -> Option<&'static str> {
    match icon {
        "CurvePolylinePolyline" => Some("Polyline"),
        "CurveLineSingleLine" => Some("Line"),
        _ => None,
    }
}
impl CurveSession {
    pub fn start(&mut self, name: &str) -> Result<(), String> {
        let name = if name.eq_ignore_ascii_case("Line") {
            "Line"
        } else if name.eq_ignore_ascii_case("Polyline") {
            "Polyline"
        } else {
            return Err("Unknown Curve command".into());
        };
        *self = Self {
            name,
            active: true,
            ..Self::default()
        };
        Ok(())
    }
    pub fn active(&self) -> bool {
        self.active
    }
    pub fn set_frame(&mut self, origin: [f64; 3], axes: [[f64; 3]; 3]) -> Result<(), String> {
        if origin.iter().any(|v| !v.is_finite() || v.abs() > 1e9)
            || axes.iter().flatten().any(|v| !v.is_finite())
        {
            return Err("Invalid construction plane".into());
        }
        for i in 0..3 {
            for j in 0..3 {
                let dot: f64 = axes[i].iter().zip(axes[j]).map(|(a, b)| a * b).sum();
                if (dot - if i == j { 1.0 } else { 0.0 }).abs() > 1e-6 {
                    return Err("Construction plane must be orthonormal".into());
                }
            }
        }
        self.frame = Some((origin, axes));
        Ok(())
    }
    pub fn name(&self) -> &'static str {
        self.name
    }
    pub fn output(&self) -> &[[f64; 3]] {
        &self.output
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
        Effect::Cancelled
    }
    pub fn prompt(&self) -> String {
        if self.awaiting_length {
            return "Length of next segment: ".into();
        }
        if !self.active {
            return "Command: ".into();
        }
        let state = if self.persistent_close { "Yes" } else { "No" };
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
        if p.iter().any(|v| !v.is_finite() || v.abs() > 1e9) {
            return Err("Coordinate must be finite and within 1e9 mm".into());
        }
        if ortho && let Some(a) = self.points.last() {
            let axes = self
                .frame
                .map_or([[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]], |frame| frame.1);
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
        if !self.active {
            return Err("No active command".into());
        }
        if self.awaiting_length {
            return Err("Enter the numeric length before picking a direction".into());
        }
        if p.iter().any(|x| !x.is_finite() || x.abs() > 1e9) {
            return Err("Coordinate must be finite and within 1e9 mm".into());
        }
        if self.points.len() >= 4096 {
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
        if self.awaiting_length {
            let length = number(text)?;
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
                let length = number(value)?;
                if length < 1e-7 {
                    return Err("Length must be positive".into());
                }
                self.length = Some(length);
            } else {
                self.awaiting_length = true;
            }
            return Ok(Effect::Waiting);
        }
        let (relative, coords) =
            if let Some(s) = text.strip_prefix('r').or_else(|| text.strip_prefix('R')) {
                (true, s)
            } else {
                (false, text)
            };
        let values = coords
            .split(',')
            .map(number)
            .collect::<Result<Vec<_>, _>>()?;
        if !(2..=3).contains(&values.len()) {
            return Err("Enter x,y or x,y,z; unsupported option".into());
        }
        let mut p = [values[0], values[1], *values.get(2).unwrap_or(&0.)];
        if let Some((origin, axes)) = self.frame {
            let local = p;
            for i in 0..3 {
                p[i] = axes[0][i] * local[0]
                    + axes[1][i] * local[1]
                    + axes[2][i] * local[2]
                    + if relative { 0.0 } else { origin[i] };
            }
        }
        if relative {
            let a = self
                .points
                .last()
                .ok_or("Relative input requires a previous point")?;
            for i in 0..3 {
                p[i] += a[i];
            }
        }
        self.point(p)
    }
}
fn number(text: &str) -> Result<f64, String> {
    let text = text.trim();
    let (value, scale) = if let Some(s) = text.strip_suffix("mm") {
        (s, 1.)
    } else if let Some(s) = text.strip_suffix("cm") {
        (s, 10.)
    } else if let Some(s) = text.strip_suffix("in") {
        (s, 25.4)
    } else {
        (text, 1.)
    };
    let value = value.trim().parse::<f64>().map_err(|_| {
        "Enter coordinates or a supported option; this option is not implemented".to_string()
    })? * scale;
    if !value.is_finite() || value.abs() > 1e9 {
        return Err("Invalid coordinate or length".into());
    }
    Ok(value)
}

fn option_bool(value: Option<&str>, current: bool) -> Result<bool, String> {
    match value {
        None => Ok(!current),
        Some("yes" | "y") => Ok(true),
        Some("no" | "n") => Ok(false),
        _ => Err("Option must be Yes or No".into()),
    }
}
