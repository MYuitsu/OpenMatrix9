// SPDX-License-Identifier: LGPL-2.1-or-later
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurveOrientation {
    NoRotation,
    Freeform,
    Roadlike,
}
impl CurveOrientation {
    pub fn name(self) -> &'static str {
        match self {
            Self::NoRotation => "NoRotation",
            Self::Freeform => "Freeform",
            Self::Roadlike => "Roadlike",
        }
    }
}
#[derive(Debug, Clone)]
pub struct TransformOptions {
    pub spacing: Option<f64>,
    pub orientation: CurveOrientation,
    pub basepoint: Option<Point3>,
    pub axes: [bool; 3],
    pub align_cplane: bool,
    pub indices: Vec<usize>,
    pub preview: bool,
}
impl Default for TransformOptions {
    fn default() -> Self {
        Self {
            spacing: None,
            orientation: CurveOrientation::NoRotation,
            basepoint: None,
            axes: [true; 3],
            align_cplane: false,
            indices: Vec::new(),
            preview: true,
        }
    }
}
fn flag(value: &str) -> Result<bool> {
    match value.to_ascii_lowercase().as_str() {
        "yes" | "true" | "on" | "1" => Ok(true),
        "no" | "false" | "off" | "0" => Ok(false),
        _ => Err(Error::Invalid("Yes/No option")),
    }
}

pub(super) fn option_input(pending: &mut TransformPending, text: &str) -> Option<Result<()>> {
    let (key, value) = text.split_once('=').map_or((text, ""), |(k, v)| (k, v));
    let key = key.trim().to_ascii_lowercase();
    let value = value.trim();
    let mut options = pending.options.clone();
    let result = (|| {
        match key.as_str() {
            "preview" => {
                options.preview = if value.is_empty() {
                    !options.preview
                } else {
                    flag(value)?
                }
            }
            "spacing" if pending.feature == ARRAY_CURVE => {
                let v = parse_number(value)?;
                if v <= 0. {
                    return Err(Error::Invalid("positive curve spacing in mm"));
                }
                options.spacing = Some(v);
            }
            "count" if pending.feature == ARRAY_CURVE => {
                let v = parse_number(value)?;
                check_number(ARRAY_CURVE, 0, v)?;
                pending.numbers = vec![v];
                options.spacing = None;
            }
            "orientation" | "roadlike" | "freeform" | "norotation"
                if pending.feature == ARRAY_CURVE =>
            {
                options.orientation = match if value.is_empty() {
                    key.as_str()
                } else {
                    value
                }
                .to_ascii_lowercase()
                .as_str()
                {
                    "roadlike" => CurveOrientation::Roadlike,
                    "freeform" => CurveOrientation::Freeform,
                    "norotation" => CurveOrientation::NoRotation,
                    _ => {
                        return Err(Error::Invalid(
                            "NoRotation, Freeform or Roadlike orientation",
                        ));
                    }
                };
            }
            "basepoint" if pending.feature == ARRAY_CURVE => {
                let input = om9_core::coords::parse_input(value, pending.unit)?;
                options.basepoint = Some(
                    om9_core::coords::resolve_with_base(input, Some(&pending.plane), None)?.xyz(),
                );
            }
            "axes" if pending.feature == SET_POINTS => {
                let v = value.to_ascii_lowercase();
                if v.is_empty() || v.chars().any(|c| !matches!(c, 'x' | 'y' | 'z')) {
                    return Err(Error::Invalid("SetPt Axes=X/Y/Z combination"));
                }
                options.axes = [v.contains('x'), v.contains('y'), v.contains('z')];
            }
            "setx" | "sety" | "setz" if pending.feature == SET_POINTS => {
                let axis = match key.as_str() {
                    "setx" => 0,
                    "sety" => 1,
                    _ => 2,
                };
                options.axes[axis] = if value.is_empty() {
                    !options.axes[axis]
                } else {
                    flag(value)?
                };
            }
            "cplane" | "world" if pending.feature == SET_POINTS => {
                options.align_cplane = key == "cplane"
            }
            "indices" if pending.feature == SET_POINTS => {
                options.indices = value
                    .split(',')
                    .map(|s| {
                        s.trim()
                            .parse::<usize>()
                            .map_err(|_| Error::Invalid("control point indices"))
                    })
                    .collect::<Result<_>>()?;
                options.indices.sort_unstable();
                options.indices.dedup();
            }
            _ => return Ok(false),
        }
        Ok(true)
    })();
    match result {
        Ok(false) => None,
        Ok(true) => {
            pending.options = options;
            Some(Ok(()))
        }
        Err(e) => Some(Err(e)),
    }
}

pub(super) fn curve_steps(
    curve: &Curve,
    pending: &TransformPending,
    limits: &Limits,
) -> Result<Vec<Transform>> {
    let total = curve_ops::arc_length(curve, limits)?;
    if !total.is_finite() || total <= 1e-9 {
        return Err(Error::Invalid("array path length"));
    }
    let closed = curve_ops::is_closed(curve, 1e-8, limits)?;
    let distances = if let Some(spacing) = pending.options.spacing {
        let intervals = (total / spacing).floor();
        if !intervals.is_finite() || intervals >= MAX_CURVE_ITEMS as f64 {
            return Err(Error::Invalid("array spacing exceeds item budget"));
        }
        (1..=intervals as usize)
            .map(|i| i as f64 * spacing)
            .filter(|d| !closed || *d < total - 1e-8)
            .collect::<Vec<_>>()
    } else {
        let count = count(pending.numbers.first().copied(), "array count")?;
        if count > MAX_CURVE_ITEMS {
            return Err(Error::Invalid("array count"));
        }
        let divisor = if closed { count } else { count - 1 };
        (1..count)
            .map(|i| total * i as f64 / divisor as f64)
            .collect()
    };
    if distances.is_empty() {
        return Err(Error::Invalid(
            "array spacing leaves no additional station on the path",
        ));
    }
    let base = pending
        .options
        .basepoint
        .unwrap_or(curve_ops::point_at(curve, 0., limits)?);
    let first_tangent = curve_ops::tangent_at(curve, 0., limits)?;
    let up = plane_frame(&pending.plane).z;
    distances
        .into_iter()
        .map(|distance| {
            let fraction = curve_ops::parameter_at_length(curve, distance, limits)?;
            let target = curve_ops::point_at(curve, fraction, limits)?;
            let rotation = match pending.options.orientation {
                CurveOrientation::NoRotation => Transform::identity(),
                CurveOrientation::Freeform => om9_core::transform_session::align_rotation(
                    first_tangent,
                    curve_ops::tangent_at(curve, fraction, limits)?,
                    base,
                )?,
                CurveOrientation::Roadlike => {
                    let tangent = curve_ops::tangent_at(curve, fraction, limits)?;
                    let a = subtract(first_tangent, scale(up, dot(first_tangent, up)));
                    let b = subtract(tangent, scale(up, dot(tangent, up)));
                    if length(a) <= 1e-12 || length(b) <= 1e-12 {
                        return Err(Error::Invalid(
                            "Roadlike tangent is parallel to CPlane up; choose Freeform",
                        ));
                    }
                    let cross = [
                        a[1] * b[2] - a[2] * b[1],
                        a[2] * b[0] - a[0] * b[2],
                        a[0] * b[1] - a[1] * b[0],
                    ];
                    gumball::rotation_about(base, up, dot(cross, up).atan2(dot(a, b)))?
                }
            };
            Transform::translation(subtract(target, base)).compose(rotation)
        })
        .collect()
}

impl Controller {
    pub fn transform_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        let Some(p) = &self.transform_pending else {
            return Vec::new();
        };
        match p.feature {
            ARRAY_CURVE => vec![
                ("Count=", "Count="),
                ("Spacing=", "Spacing="),
                ("NoRotation", "NoRotation"),
                ("Freeform", "Freeform"),
                ("Roadlike", "Roadlike"),
                ("Basepoint=", "Basepoint="),
                ("Preview", "Preview"),
                ("Cancel", "Cancel"),
            ],
            SET_POINTS => vec![
                ("SetX", "SetX"),
                ("SetY", "SetY"),
                ("SetZ", "SetZ"),
                ("World", "World"),
                ("CPlane", "CPlane"),
                ("Indices=", "Indices="),
                ("Preview", "Preview"),
                ("Cancel", "Cancel"),
            ],
            _ => vec![("Preview", "Preview"), ("Cancel", "Cancel")],
        }
    }

    /// Read-only candidate of the same plan the click/Enter commits.
    pub fn transform_preview(&self, point: Point3, draft: &str) -> Result<Vec<[Point3; 2]>> {
        let Some(p) = &self.transform_pending else {
            return Ok(Vec::new());
        };
        if !p.options.preview {
            return Ok(Vec::new());
        }
        let mut p = p.clone();
        match p.next() {
            Some(Field::Point(_)) => {
                p.points.push(point);
                p.field += 1;
            }
            Some(Field::Number(_))
                if p.feature == ARRAY_CURVE
                    && (p.options.spacing.is_some() || !p.numbers.is_empty()) => {}
            Some(Field::Number(_)) if !draft.trim().is_empty() => {
                let v = parse_number(draft)?;
                check_number(p.feature, p.numbers.len(), v)?;
                p.numbers.push(v);
                p.field += 1;
            }
            _ => return Ok(Vec::new()),
        }
        if p.next().is_some()
            && !(p.feature == ARRAY_CURVE && (p.options.spacing.is_some() || !p.numbers.is_empty()))
        {
            return Ok(Vec::new());
        }
        let plan = self.plan_transform(&p)?;
        let mut wire = Vec::new();
        for (id, g) in plan.replacements {
            let e = &self.document().entities[&id];
            wire.extend(om9_core::point_alignment::outline(
                &g,
                e.placement,
                self.coordinator.limits(),
            )?);
        }
        if plan.copies {
            for step in &plan.steps {
                for id in &plan.targets {
                    let e = &self.document().entities[id];
                    wire.extend(om9_core::point_alignment::outline(
                        &e.geometry,
                        step.compose(e.placement)?,
                        self.coordinator.limits(),
                    )?);
                }
            }
        } else {
            for (i, id) in plan.targets.iter().enumerate() {
                let e = &self.document().entities[id];
                let step = if plan.steps.len() == plan.targets.len() {
                    plan.steps.get(i)
                } else {
                    plan.steps.first()
                };
                if let Some(step) = step {
                    wire.extend(om9_core::point_alignment::outline(
                        &e.geometry,
                        step.compose(e.placement)?,
                        self.coordinator.limits(),
                    )?);
                }
            }
        }
        if wire.len() as u64 > self.coordinator.limits().max_points {
            return Err(Error::Budget("transform preview"));
        }
        Ok(wire)
    }
}
