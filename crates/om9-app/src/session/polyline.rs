//! Shared Polyline option and geometry session for menu, CMD and viewport input.
use super::{Controller, Error};
use crate::polyline_options::{
    PolylineCommandOption, PolylineOptionId, PolylineOptionInterface, PolylineOptionValue,
};
use om9_core::{
    budget::Limits,
    geometry::{Curve, Geometry, Point3},
};
pub const POLYLINE: &str = "OM9-CURVE-001";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PolylineMode {
    #[default]
    Line,
    Arc,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum ArcInput {
    #[default]
    End,
    Direction,
    Center,
}
#[derive(Debug, Clone, Default)]
pub struct PolylinePending {
    pub points: Vec<Point3>,
    segments: Vec<Curve>,
    pub close: bool,
    pub persistent_close: bool,
    pub mode: PolylineMode,
    pub helpers: bool,
    pub length: Option<f64>,
    pub length_entry: bool,
    arc_input: ArcInput,
    direction: Option<Point3>,
    center: Option<Point3>,
    prompt: String,
}
impl PolylinePending {
    pub fn new() -> Self {
        let mut pending = Self::default();
        pending.refresh_prompt();
        pending
    }
    pub fn prompt(&self) -> &str {
        &self.prompt
    }
    fn options(&self) -> Vec<PolylineCommandOption> {
        use PolylineOptionId as Id;
        use PolylineOptionValue as Value;
        if self.length_entry {
            return Vec::new();
        }
        let mut options = vec![
            PolylineCommandOption {
                id: Id::PersistentClose,
                value: Value::Flag(self.persistent_close),
            },
            PolylineCommandOption {
                id: Id::Mode,
                value: Value::Choice(if self.mode == PolylineMode::Line {
                    "Line"
                } else {
                    "Arc"
                }),
            },
            PolylineCommandOption {
                id: Id::Helpers,
                value: Value::Flag(self.helpers),
            },
        ];
        if !self.points.is_empty() {
            if self.mode == PolylineMode::Line {
                if self.points.len() >= 3 {
                    options.push(PolylineCommandOption {
                        id: Id::Close,
                        value: Value::None,
                    });
                }
                options.push(PolylineCommandOption {
                    id: Id::Length,
                    value: Value::Number(self.length),
                });
            } else {
                options.push(PolylineCommandOption {
                    id: Id::Direction,
                    value: Value::Point(self.direction),
                });
                options.push(PolylineCommandOption {
                    id: Id::Center,
                    value: Value::Point(self.center),
                });
            }
            options.push(PolylineCommandOption {
                id: Id::Undo,
                value: Value::None,
            });
        }
        options
    }
    fn refresh_prompt(&mut self) {
        if self.length_entry {
            self.prompt = "Length".into();
            return;
        }
        let prefix = if self.points.is_empty() {
            "Start of polyline"
        } else if self.arc_input == ArcInput::Direction {
            "Direction of polyline arc"
        } else if self.arc_input == ArcInput::Center {
            "Center of polyline arc"
        } else {
            "Next point of polyline. Press Enter when done"
        };
        self.prompt = format!(
            "{prefix} ({}): ",
            self.options()
                .iter()
                .map(|option| option.label())
                .collect::<Vec<_>>()
                .join(" ")
        );
    }
    fn segment(&self, end: Point3, limits: &Limits) -> Result<Curve, Error> {
        let start = *self.points.last().ok_or(Error::Invalid("polyline start"))?;
        if self.mode == PolylineMode::Line {
            let curve = Curve::Line {
                start,
                end,
                domain: [0.0, 1.0],
            };
            curve.validate(limits)?;
            return Ok(curve);
        }
        if let Some(center) = self.center {
            let x = unit(sub(start, center))?;
            let radius = norm(sub(start, center));
            if (norm(sub(end, center)) - radius).abs() > 1e-7 * radius.max(1.0) {
                return Err(Error::Invalid(
                    "polyline arc endpoint must share the radius",
                ));
            }
            let normal = unit(cross(x, sub(end, center)))?;
            let y = cross(normal, x);
            let delta = sub(end, center);
            let sweep = dot(delta, y).atan2(dot(delta, x));
            return om9_core::curve_build::arc_by_center(center, x, y, radius, 0.0, sweep);
        }
        let tangent = self
            .direction
            .or_else(|| {
                self.segments.last().and_then(|curve| {
                    om9_core::curve_ops::fraction_derivatives(curve, 1.0, 1, limits)
                        .ok()
                        .map(|jet| jet[1])
                })
            })
            .ok_or(Error::Invalid("choose the polyline arc Direction first"))?;
        let tangent = unit(tangent)?;
        let chord = sub(end, start);
        let normal = unit(cross(tangent, chord))?;
        let inward = cross(normal, tangent);
        let radius = dot(chord, chord) / (2.0 * dot(chord, inward));
        let center = add(start, scale(inward, radius));
        let x = scale(inward, -1.0);
        let delta = sub(end, center);
        let mut sweep = dot(delta, tangent).atan2(dot(delta, x));
        if sweep <= 0.0 {
            sweep += std::f64::consts::TAU;
        }
        om9_core::curve_build::arc_by_center(center, x, tangent, radius, 0.0, sweep)
    }
}
impl Controller {
    pub fn dispatch_polyline(&mut self) -> Result<(), Error> {
        self.polyline_pending = Some(PolylinePending::new());
        Ok(())
    }
    pub fn submit_polyline_text(&mut self, text: &str) -> Option<Result<(), Error>> {
        let mut pending = self.polyline_pending.take()?;
        // Full option names and initials resolve from the same current snapshot
        // as keyboard and pointer activation. C is Close in Line, Center in Arc.
        let option = pending
            .options()
            .into_iter()
            .find(|option| option.matches(text))
            .map_or_else(
                || text.trim().to_ascii_lowercase(),
                |option| option.id.name().to_ascii_lowercase(),
            );
        if option == "cancel" || option == "esc" {
            return Some(Ok(()));
        }
        let result = (|| {
            if pending.length_entry {
                pending.length = Some(positive_length(&option)?);
                pending.length_entry = false;
            } else {
                match option.as_str() {
                    "undo" | "u" => {
                        if pending.arc_input != ArcInput::End || pending.center.is_some() {
                            pending.arc_input = ArcInput::End;
                            pending.center = None;
                        } else {
                            pending.points.pop();
                            pending.segments.pop();
                        }
                        pending.length = None;
                    }
                    "close" | "c" if pending.mode == PolylineMode::Line => {
                        if pending.points.len() < 3 {
                            return Err(Error::Invalid("polyline Close needs three points"));
                        }
                        pending.close = true;
                    }
                    "length" | "l" => {
                        if pending.points.is_empty() || pending.mode != PolylineMode::Line {
                            return Err(Error::Invalid(
                                "Length needs a polyline point in Line mode",
                            ));
                        }
                        pending.length_entry = true;
                    }
                    "persistentclose" | "p" => pending.persistent_close = !pending.persistent_close,
                    "p=y" | "p=yes" | "persistentclose=yes" | "persistentclose=y" => {
                        pending.persistent_close = true
                    }
                    "p=n" | "p=no" | "persistentclose=no" | "persistentclose=n" => {
                        pending.persistent_close = false
                    }
                    "helpers" | "h" => pending.helpers = !pending.helpers,
                    "h=y" | "h=yes" | "helpers=yes" | "helpers=y" => pending.helpers = true,
                    "h=n" | "h=no" | "helpers=no" | "helpers=n" => pending.helpers = false,
                    "mode" | "m" | "m=l" | "mode=l" | "mode=line" | "m=a" | "mode=a"
                    | "mode=arc" => {
                        pending.mode = match option.as_str() {
                            "m=l" | "mode=l" | "mode=line" => PolylineMode::Line,
                            "m=a" | "mode=a" | "mode=arc" => PolylineMode::Arc,
                            _ if pending.mode == PolylineMode::Line => PolylineMode::Arc,
                            _ => PolylineMode::Line,
                        };
                        pending.center = None;
                        pending.length = None;
                        pending.arc_input = if pending.mode == PolylineMode::Arc
                            && !pending.points.is_empty()
                            && pending.segments.is_empty()
                            && pending.direction.is_none()
                        {
                            ArcInput::Direction
                        } else {
                            ArcInput::End
                        };
                    }
                    "direction" | "d" | "center" | "c"
                        if pending.mode == PolylineMode::Arc && !pending.points.is_empty() =>
                    {
                        pending.arc_input = if option == "direction" || option == "d" {
                            ArcInput::Direction
                        } else {
                            ArcInput::Center
                        };
                        pending.center = None;
                    }
                    "" => {
                        if pending.segments.is_empty()
                            || pending.arc_input != ArcInput::End
                            || pending.center.is_some()
                        {
                            return Err(Error::Invalid("polyline needs a completed segment"));
                        }
                    }
                    _ if option.starts_with("length=") || option.starts_with("l=") => {
                        if pending.points.is_empty() || pending.mode != PolylineMode::Line {
                            return Err(Error::Invalid(
                                "Length needs a polyline point in Line mode",
                            ));
                        }
                        pending.length = Some(positive_length(
                            option.split_once('=').expect("assignment").1,
                        )?);
                    }
                    _ if option.contains(',') => {
                        let input =
                            om9_core::coords::parse_input(text.trim(), self.document().page_unit)?;
                        let plane = self.input_plane();
                        let base = pending
                            .points
                            .last()
                            .copied()
                            .map(om9_core::coords::WorldPoint::try_new)
                            .transpose()?;
                        let world = om9_core::coords::resolve_with_base(input, Some(&plane), base)?;
                        return self.accept_pending_polyline_point(&mut pending, world.xyz());
                    }
                    _ => return Err(Error::Invalid("polyline option or point")),
                }
            }
            Ok(())
        })();
        pending.refresh_prompt();
        let finish = result.is_ok() && (pending.close || option.is_empty());
        self.polyline_pending = Some(pending);
        Some(if finish {
            let result = self.commit_polyline();
            if result.is_err()
                && let Some(pending) = &mut self.polyline_pending
            {
                pending.close = false;
            }
            result
        } else {
            result
        })
    }
    fn accept_pending_polyline_point(
        &self,
        pending: &mut PolylinePending,
        mut point: Point3,
    ) -> Result<(), Error> {
        om9_core::geometry::finite(point)?;
        if pending.length_entry {
            return Err(Error::Invalid("enter the polyline Length first"));
        }
        if let Some(&last) = pending.points.last() {
            if pending.arc_input == ArcInput::Direction {
                pending.direction = Some(unit(sub(point, last))?);
                pending.arc_input = ArcInput::End;
                return Ok(());
            }
            if pending.arc_input == ArcInput::Center {
                unit(sub(point, last))?;
                pending.center = Some(point);
                pending.arc_input = ArcInput::End;
                return Ok(());
            }
            if let Some(length) = pending.length {
                point = add(last, scale(unit(sub(point, last))?, length));
            }
            let segment = pending.segment(point, self.coordinator.limits())?;
            if pending.mode == PolylineMode::Arc {
                // Keep the accepted endpoint identical to the analytic arc,
                // including a Center endpoint accepted within radius tolerance.
                point = om9_core::curve_ops::fraction_derivatives(
                    &segment,
                    1.0,
                    0,
                    self.coordinator.limits(),
                )?[0];
            }
            pending.segments.push(segment);
        }
        pending.points.push(point);
        pending.length = None;
        pending.center = None;
        if pending.mode == PolylineMode::Arc
            && pending.segments.is_empty()
            && pending.direction.is_none()
        {
            pending.arc_input = ArcInput::Direction;
        }
        Ok(())
    }
    pub fn accept_polyline_point(&mut self, point: Point3) -> Result<(), Error> {
        let Some(mut pending) = self.polyline_pending.take() else {
            return Err(Error::Missing("polyline command"));
        };
        let result = self.accept_pending_polyline_point(&mut pending, point);
        pending.refresh_prompt();
        self.polyline_pending = Some(pending);
        result
    }
    fn commit_polyline(&mut self) -> Result<(), Error> {
        let pending = self
            .polyline_pending
            .as_ref()
            .ok_or(Error::Missing("polyline command"))?;
        let mut segments = pending.segments.clone();
        if segments.is_empty() {
            return Err(Error::Invalid("polyline needs two points"));
        }
        if pending.close && pending.points.first() != pending.points.last() {
            segments.push(Curve::Line {
                start: *pending.points.last().expect("points"),
                end: pending.points[0],
                domain: [0.0, 1.0],
            });
        }
        let persistent = pending.persistent_close;
        let helpers = pending.helpers;
        let mode = pending.mode;
        let curve = Curve::Polycurve {
            segments,
            reversed: false,
        };
        self.commit_geometry(POLYLINE, "Polyline", Geometry::Curve(curve))?;
        self.polyline_pending = if persistent {
            let mut next = PolylinePending {
                persistent_close: true,
                helpers,
                mode,
                ..PolylinePending::default()
            };
            next.refresh_prompt();
            Some(next)
        } else {
            None
        };
        Ok(())
    }
    pub fn polyline_options(&self) -> Option<(bool, bool, bool)> {
        self.polyline_pending.as_ref().map(|pending| {
            (
                pending.persistent_close,
                pending.helpers,
                pending.points.len() >= 3,
            )
        })
    }
    pub fn polyline_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        // Compatibility for the existing non-Polyline token selector. Polyline
        // prompt/render/input all consume the typed interface directly.
        self.polyline_command_options()
            .into_iter()
            .map(|option| (option.id.name(), option.id.name()))
            .collect()
    }
    pub fn polyline_preview(&self, point: Point3) -> Vec<[Point3; 2]> {
        let Some(pending) = self.polyline_pending.as_ref() else {
            return Vec::new();
        };
        let mut preview = pending.clone();
        let _ = self.accept_pending_polyline_point(&mut preview, point);
        preview
            .segments
            .iter()
            .flat_map(|curve| match curve {
                Curve::Line { start, end, .. } => vec![[*start, *end]],
                _ => (0..32)
                    .filter_map(|i| {
                        let a = om9_core::curve_ops::fraction_derivatives(
                            curve,
                            i as f64 / 32.0,
                            0,
                            self.coordinator.limits(),
                        )
                        .ok()?;
                        let b = om9_core::curve_ops::fraction_derivatives(
                            curve,
                            (i + 1) as f64 / 32.0,
                            0,
                            self.coordinator.limits(),
                        )
                        .ok()?;
                        Some([a[0], b[0]])
                    })
                    .collect(),
            })
            .collect()
    }
}
impl PolylineOptionInterface for Controller {
    fn polyline_command_options(&self) -> Vec<PolylineCommandOption> {
        self.polyline_pending
            .as_ref()
            .map_or_else(Vec::new, PolylinePending::options)
    }

    fn activate_polyline_option(&mut self, option: PolylineOptionId) -> Result<(), Error> {
        if !self
            .polyline_command_options()
            .iter()
            .any(|current| current.id == option)
        {
            return Err(Error::Invalid(
                "polyline option unavailable in this input stage",
            ));
        }
        self.submit_polyline_text(option.name())
            .ok_or(Error::Missing("polyline command"))?
    }
}
fn positive_length(value: &str) -> Result<f64, Error> {
    value
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite() && *value > 0.0)
        .ok_or(Error::Invalid("polyline length"))
}
fn sub(a: Point3, b: Point3) -> Point3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn add(a: Point3, b: Point3) -> Point3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn scale(a: Point3, s: f64) -> Point3 {
    a.map(|v| v * s)
}
fn dot(a: Point3, b: Point3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn norm(a: Point3) -> f64 {
    a[0].hypot(a[1]).hypot(a[2])
}
fn unit(a: Point3) -> Result<Point3, Error> {
    let length = norm(a);
    if !length.is_finite() || length <= 1e-12 {
        return Err(Error::Invalid("polyline direction or repeated point"));
    }
    Ok(scale(a, 1.0 / length))
}
fn cross(a: Point3, b: Point3) -> Point3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
