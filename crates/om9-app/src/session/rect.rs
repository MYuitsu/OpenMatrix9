//! Rectangle command behaviour ported from the FreeCAD Qt Curve controller.

use om9_core::geometry::{Curve, Geometry, Point3};
use om9_scene::camera::CPlane;

use super::{Controller, Error};

pub const RECTANGLE: &str = "OM9-CURVE-004";

#[derive(Debug, Clone)]
pub struct RectPending {
    pub mode: &'static str,
    pub points: Vec<Point3>,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub square: bool,
}

/// One signed construction shared by the corner preview and the committed curve.
struct CornerFrame {
    origin: Point3,
    x: Point3,
    y: Point3,
    width: f64,
    height: f64,
}

impl CornerFrame {
    fn wire(&self) -> Vec<[Point3; 2]> {
        let corner = |u: f64, v: f64| {
            std::array::from_fn(|axis| self.origin[axis] + self.x[axis] * u + self.y[axis] * v)
        };
        let points = [
            self.origin,
            corner(self.width, 0.),
            corner(self.width, self.height),
            corner(0., self.height),
        ];
        (0..4).map(|i| [points[i], points[(i + 1) % 4]]).collect()
    }
}

impl Default for RectPending {
    fn default() -> Self {
        Self::new()
    }
}

impl RectPending {
    fn corner_frame(&self, point: Point3, plane: CPlane) -> Result<CornerFrame, Error> {
        let first = *self
            .points
            .first()
            .ok_or(Error::Missing("rectangle corner"))?;
        let delta: Point3 = std::array::from_fn(|axis| point[axis] - first[axis]);
        let along_x: f64 = (0..3).map(|axis| delta[axis] * plane.x[axis]).sum();
        let along_y: f64 = (0..3).map(|axis| delta[axis] * plane.y[axis]).sum();
        let mut width = self.width.unwrap_or(along_x.abs());
        let mut height = self.height.unwrap_or(along_y.abs());
        if self.mode == "Center" {
            width *= 2.;
            height *= 2.;
        }
        if self.square {
            width = width.max(height);
            height = width;
        }
        if width <= 1e-9 || height <= 1e-9 {
            return Err(Error::Invalid("rectangle must be non-degenerate"));
        }
        let x = plane
            .x
            .map(|value| if along_x < 0. { -value } else { value });
        let y = plane
            .y
            .map(|value| if along_y < 0. { -value } else { value });
        let origin = if self.mode == "Center" {
            std::array::from_fn(|axis| first[axis] - x[axis] * width * 0.5 - y[axis] * height * 0.5)
        } else {
            first
        };
        Ok(CornerFrame {
            origin,
            x,
            y,
            width,
            height,
        })
    }

    pub fn new() -> Self {
        Self {
            mode: "Corners",
            points: Vec::new(),
            width: None,
            height: None,
            square: false,
        }
    }

    pub fn prompt(&self) -> &'static str {
        if self.mode == "Vertical" && self.points.is_empty() {
            "Base point"
        } else if self.mode == "Vertical" && self.points.len() == 1 {
            "Vertical edge end"
        } else if self.mode == "Vertical" && self.height.is_none() {
            "Width"
        } else if self.mode == "ThreePoint" && self.points.is_empty() {
            "First edge point"
        } else if self.mode == "ThreePoint" && self.points.len() == 1 {
            "Second edge point"
        } else if self.mode == "ThreePoint" && self.height.is_none() {
            "Width"
        } else if self.mode == "Center" && self.points.is_empty() {
            "Center of rectangle"
        } else if self.mode == "Center" && self.width.is_none() {
            "Corner or Width"
        } else if self.points.is_empty() {
            "First corner of rectangle"
        } else if self.width.is_none() {
            "Other corner or Width"
        } else if self.height.is_none() {
            "Height"
        } else {
            "Press Enter to create the rectangle"
        }
    }
}

impl Controller {
    pub fn dispatch_rect(&mut self) -> Result<(), Error> {
        self.rect_pending = Some(RectPending::new());
        Ok(())
    }

    pub fn submit_rect_text(&mut self, text: &str) -> Option<Result<(), Error>> {
        let mut pending = self.rect_pending.take()?;
        let option = text.trim().to_ascii_lowercase();
        if option == "cancel" || option == "esc" {
            return Some(Ok(()));
        }
        if (option == "3point" || option == "3p") && !pending.points.is_empty() {
            self.rect_pending = Some(pending);
            return Some(Err(Error::Invalid("choose 3Point before the first point")));
        }
        if (option == "vertical" || option == "v") && !pending.points.is_empty() {
            self.rect_pending = Some(pending);
            return Some(Err(Error::Invalid(
                "choose Vertical before the first point",
            )));
        }
        if option == "vertical" || option == "v" {
            pending.mode = "Vertical";
            self.rect_pending = Some(pending);
            return Some(Ok(()));
        }
        if option == "3point" || option == "3p" {
            pending.mode = "ThreePoint";
            self.rect_pending = Some(pending);
            return Some(Ok(()));
        }
        if (option == "center" || option == "c") && !pending.points.is_empty() {
            self.rect_pending = Some(pending);
            return Some(Err(Error::Invalid("choose Center before the first point")));
        }
        if option == "center" || option == "c" {
            pending.mode = "Center";
            self.rect_pending = Some(pending);
            return Some(Ok(()));
        }
        if option == "square" || option == "s" {
            pending.square = !pending.square;
            self.rect_pending = Some(pending);
            return Some(Ok(()));
        }
        if option == "undo" || option == "u" {
            if pending.height.take().is_none() && pending.width.take().is_none() {
                pending.points.pop();
            }
            self.rect_pending = Some(pending);
            return Some(Ok(()));
        }
        if text.contains(',') && !text.contains('=') {
            let plane = self.input_plane();
            let base = pending
                .points
                .last()
                .copied()
                .map(om9_core::coords::WorldPoint::try_new)
                .transpose();
            self.rect_pending = Some(pending);
            return Some((|| {
                let input = om9_core::coords::parse_input(text, self.document().page_unit)?;
                let world = om9_core::coords::resolve_with_base(input, Some(&plane), base?)?;
                self.accept_world_point(world.xyz())
            })());
        }
        if pending.points.is_empty() {
            self.rect_pending = Some(pending);
            return Some(Err(Error::Invalid("rectangle needs a first corner")));
        }
        if let Some(value) = text
            .trim()
            .strip_prefix("Width=")
            .or_else(|| text.trim().strip_prefix("Length="))
        {
            let Some(width) = value.parse::<f64>().ok().filter(|value| *value > 0.0) else {
                self.rect_pending = Some(pending);
                return Some(Err(Error::Invalid("width must be positive")));
            };
            if matches!(pending.mode, "ThreePoint" | "Vertical") {
                pending.height = Some(width);
            } else {
                pending.width = Some(width);
                self.rect_pending = Some(pending);
                return Some(Ok(()));
            }
        }
        if let Some(value) = text.trim().strip_prefix("Height=") {
            let Some(height) = value.parse::<f64>().ok().filter(|value| *value > 0.0) else {
                self.rect_pending = Some(pending);
                return Some(Err(Error::Invalid("height must be positive")));
            };
            pending.height = Some(height);
            if matches!(pending.mode, "ThreePoint" | "Vertical") && pending.points.len() >= 2 {
                // fall through to the edge-frame commit below
            } else {
                self.rect_pending = Some(pending);
                return Some(Ok(()));
            }
        }
        if matches!(pending.mode, "ThreePoint" | "Vertical")
            && pending.points.len() >= 2
            && pending.height.is_some()
        {
            let Some(height) = pending.height else {
                self.rect_pending = Some(pending);
                return Some(Err(Error::Invalid("rectangle width")));
            };
            let first = pending.points[0];
            let second = pending.points[1];
            let e = [
                second[0] - first[0],
                second[1] - first[1],
                second[2] - first[2],
            ];
            let length = (e[0].powi(2) + e[1].powi(2) + e[2].powi(2)).sqrt();
            if length <= 1e-9 {
                self.rect_pending = Some(pending);
                return Some(Err(Error::Invalid("rectangle edge")));
            }
            let x = [e[0] / length, e[1] / length, e[2] / length];
            let plane = self.input_plane();
            let y = if pending.mode == "Vertical" {
                let dot = plane.x[0] * x[0] + plane.x[1] * x[1] + plane.x[2] * x[2];
                let mut y = [
                    plane.x[0] - x[0] * dot,
                    plane.x[1] - x[1] * dot,
                    plane.x[2] - x[2] * dot,
                ];
                let norm = (y[0].powi(2) + y[1].powi(2) + y[2].powi(2)).sqrt();
                if norm <= 1e-12 {
                    self.rect_pending = Some(pending);
                    return Some(Err(Error::Invalid("rectangle vertical frame")));
                }
                y = [y[0] / norm, y[1] / norm, y[2] / norm];
                y
            } else {
                let normal = [
                    plane.x[1] * plane.y[2] - plane.x[2] * plane.y[1],
                    plane.x[2] * plane.y[0] - plane.x[0] * plane.y[2],
                    plane.x[0] * plane.y[1] - plane.x[1] * plane.y[0],
                ];
                let mut y = [
                    normal[1] * x[2] - normal[2] * x[1],
                    normal[2] * x[0] - normal[0] * x[2],
                    normal[0] * x[1] - normal[1] * x[0],
                ];
                let norm = (y[0].powi(2) + y[1].powi(2) + y[2].powi(2)).sqrt();
                if norm <= 1e-12 {
                    self.rect_pending = Some(pending);
                    return Some(Err(Error::Invalid("rectangle edge frame")));
                }
                y = [y[0] / norm, y[1] / norm, y[2] / norm];
                y
            };
            self.rect_pending = None;
            return Some(self.commit_rect(first, x, y, length, height));
        }
        if pending.width.is_some() && pending.height.is_some() {
            let first = pending.points[0];
            let Some(width) = pending.width else {
                self.rect_pending = Some(pending);
                return Some(Err(Error::Invalid("rectangle width")));
            };
            let Some(height) = pending.height else {
                self.rect_pending = Some(pending);
                return Some(Err(Error::Invalid("rectangle height")));
            };
            self.rect_pending = None;
            let plane = self.input_plane();
            if pending.mode == "Center" {
                let origin = [
                    first[0] - plane.x[0] * width * 0.5 - plane.y[0] * height * 0.5,
                    first[1] - plane.x[1] * width * 0.5 - plane.y[1] * height * 0.5,
                    first[2] - plane.x[2] * width * 0.5 - plane.y[2] * height * 0.5,
                ];
                return Some(self.commit_rect(origin, plane.x, plane.y, width, height));
            }
            return Some(self.commit_rect(first, plane.x, plane.y, width, height));
        }
        self.rect_pending = Some(pending);
        Some(Err(Error::Invalid(
            "rectangle needs Width=/Height= or a second corner",
        )))
    }

    pub fn accept_rect_point(&mut self, point: Point3) -> Result<(), Error> {
        let Some(mut pending) = self.rect_pending.take() else {
            return Err(Error::Missing("rectangle command"));
        };
        pending.points.push(point);
        if pending.points.len() < 2 {
            self.rect_pending = Some(pending);
            return Ok(());
        }
        let first = pending.points[0];
        let plane = self.input_plane();
        if matches!(pending.mode, "ThreePoint" | "Vertical") {
            let z = [
                plane.x[1] * plane.y[2] - plane.x[2] * plane.y[1],
                plane.x[2] * plane.y[0] - plane.x[0] * plane.y[2],
                plane.x[0] * plane.y[1] - plane.x[1] * plane.y[0],
            ];
            if pending.mode == "Vertical" && pending.points.len() == 2 {
                let delta = [
                    pending.points[1][0] - first[0],
                    pending.points[1][1] - first[1],
                    pending.points[1][2] - first[2],
                ];
                let axial = delta[0] * z[0] + delta[1] * z[1] + delta[2] * z[2];
                let length = axial.abs();
                if length <= 1e-9 {
                    self.rect_pending = Some(pending);
                    return Err(Error::Invalid("rectangle vertical edge must be positive"));
                }
                let sign = if axial < 0.0 { -1.0 } else { 1.0 };
                pending.points[1] = [
                    first[0] + z[0] * length * sign,
                    first[1] + z[1] * length * sign,
                    first[2] + z[2] * length * sign,
                ];
            }
            if pending.points.len() == 2 {
                self.rect_pending = Some(pending);
                return Ok(());
            }
            if pending.points.len() >= 3 {
                let second = pending.points[1];
                let point = pending.points[2];
                let e = [
                    second[0] - first[0],
                    second[1] - first[1],
                    second[2] - first[2],
                ];
                let length = (e[0].powi(2) + e[1].powi(2) + e[2].powi(2)).sqrt();
                let x = [e[0] / length, e[1] / length, e[2] / length];
                if pending.mode == "Vertical" {
                    let dot = plane.x[0] * x[0] + plane.x[1] * x[1] + plane.x[2] * x[2];
                    let mut y = [
                        plane.x[0] - x[0] * dot,
                        plane.x[1] - x[1] * dot,
                        plane.x[2] - x[2] * dot,
                    ];
                    let y_norm = (y[0].powi(2) + y[1].powi(2) + y[2].powi(2)).sqrt();
                    if y_norm <= 1e-12 {
                        self.rect_pending = Some(pending);
                        return Err(Error::Invalid("rectangle vertical frame"));
                    }
                    y = [y[0] / y_norm, y[1] / y_norm, y[2] / y_norm];
                    let height = ((point[0] - first[0]) * y[0]
                        + (point[1] - first[1]) * y[1]
                        + (point[2] - first[2]) * y[2])
                        .abs();
                    if height <= 1e-9 {
                        self.rect_pending = Some(pending);
                        return Err(Error::Invalid("rectangle width must be positive"));
                    }
                    self.rect_pending = None;
                    return self.commit_rect(first, x, y, length, height);
                }
                let normal = [
                    plane.x[1] * plane.y[2] - plane.x[2] * plane.y[1],
                    plane.x[2] * plane.y[0] - plane.x[0] * plane.y[2],
                    plane.x[0] * plane.y[1] - plane.x[1] * plane.y[0],
                ];
                let mut y = [
                    normal[1] * x[2] - normal[2] * x[1],
                    normal[2] * x[0] - normal[0] * x[2],
                    normal[0] * x[1] - normal[1] * x[0],
                ];
                let y_norm = (y[0].powi(2) + y[1].powi(2) + y[2].powi(2)).sqrt();
                if y_norm <= 1e-12 {
                    self.rect_pending = Some(pending);
                    return Err(Error::Invalid("rectangle edge frame"));
                }
                y = [y[0] / y_norm, y[1] / y_norm, y[2] / y_norm];
                let height = ((point[0] - first[0]) * y[0]
                    + (point[1] - first[1]) * y[1]
                    + (point[2] - first[2]) * y[2])
                    .abs();
                if height <= 1e-9 {
                    self.rect_pending = Some(pending);
                    return Err(Error::Invalid("rectangle width must be positive"));
                }
                self.rect_pending = None;
                return self.commit_rect(first, x, y, length, height);
            }
            self.rect_pending = Some(pending);
            return Ok(());
        }
        let frame = match pending.corner_frame(point, plane) {
            Ok(frame) => frame,
            Err(error) => {
                self.rect_pending = Some(pending);
                return Err(error);
            }
        };
        self.commit_rect(frame.origin, frame.x, frame.y, frame.width, frame.height)
    }

    fn commit_rect(
        &mut self,
        origin: Point3,
        x: Point3,
        y: Point3,
        width: f64,
        height: f64,
    ) -> Result<(), Error> {
        let curve = om9_core::curve_build::rectangle(origin, x, y, width, height)?;
        self.commit_geometry(RECTANGLE, "Rectangle", Geometry::Curve(curve))
    }

    pub fn rect_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        let Some(pending) = self.rect_pending.as_ref() else {
            return Vec::new();
        };
        let mut tokens = vec![
            ("Square", "Square"),
            ("Length=", "Length="),
            ("Cancel", "Cancel"),
        ];
        if pending.points.is_empty() {
            tokens.insert(0, ("Center", "Center"));
            tokens.insert(0, ("3Point", "3Point"));
            tokens.insert(0, ("Vertical", "Vertical"));
        }
        if !pending.points.is_empty() {
            tokens.insert(0, ("Undo", "Undo"));
        }
        tokens
    }

    pub fn rect_preview(&self, point: Point3) -> Vec<[Point3; 2]> {
        let Some(pending) = self.rect_pending.as_ref() else {
            return Vec::new();
        };
        let Some(origin) = pending.points.first().copied() else {
            return Vec::new();
        };
        let plane = self.input_plane();
        if matches!(pending.mode, "ThreePoint" | "Vertical") {
            if pending.points.len() == 1 {
                return vec![[origin, point]];
            }
            let mut second = pending.points[1];
            if pending.mode == "Vertical" {
                let z = [
                    plane.x[1] * plane.y[2] - plane.x[2] * plane.y[1],
                    plane.x[2] * plane.y[0] - plane.x[0] * plane.y[2],
                    plane.x[0] * plane.y[1] - plane.x[1] * plane.y[0],
                ];
                let delta = [
                    second[0] - origin[0],
                    second[1] - origin[1],
                    second[2] - origin[2],
                ];
                let axial = delta[0] * z[0] + delta[1] * z[1] + delta[2] * z[2];
                let length = axial.abs();
                if length <= 1e-9 {
                    return Vec::new();
                }
                let sign = if axial < 0.0 { -1.0 } else { 1.0 };
                second = [
                    origin[0] + z[0] * length * sign,
                    origin[1] + z[1] * length * sign,
                    origin[2] + z[2] * length * sign,
                ];
            }
            let e = [
                second[0] - origin[0],
                second[1] - origin[1],
                second[2] - origin[2],
            ];
            let length = (e[0].powi(2) + e[1].powi(2) + e[2].powi(2)).sqrt();
            if length <= 1e-9 {
                return Vec::new();
            }
            let x = [e[0] / length, e[1] / length, e[2] / length];
            let y = if pending.mode == "Vertical" {
                let dot = plane.x[0] * x[0] + plane.x[1] * x[1] + plane.x[2] * x[2];
                let mut y = [
                    plane.x[0] - x[0] * dot,
                    plane.x[1] - x[1] * dot,
                    plane.x[2] - x[2] * dot,
                ];
                let y_norm = (y[0].powi(2) + y[1].powi(2) + y[2].powi(2)).sqrt();
                if y_norm <= 1e-12 {
                    return Vec::new();
                }
                y = [y[0] / y_norm, y[1] / y_norm, y[2] / y_norm];
                y
            } else {
                let normal = [
                    plane.x[1] * plane.y[2] - plane.x[2] * plane.y[1],
                    plane.x[2] * plane.y[0] - plane.x[0] * plane.y[2],
                    plane.x[0] * plane.y[1] - plane.x[1] * plane.y[0],
                ];
                let mut y = [
                    normal[1] * x[2] - normal[2] * x[1],
                    normal[2] * x[0] - normal[0] * x[2],
                    normal[0] * x[1] - normal[1] * x[0],
                ];
                let y_norm = (y[0].powi(2) + y[1].powi(2) + y[2].powi(2)).sqrt();
                if y_norm <= 1e-12 {
                    return Vec::new();
                }
                y = [y[0] / y_norm, y[1] / y_norm, y[2] / y_norm];
                y
            };
            let height = ((point[0] - origin[0]) * y[0]
                + (point[1] - origin[1]) * y[1]
                + (point[2] - origin[2]) * y[2])
                .abs();
            if height <= 1e-9 {
                return Vec::new();
            }
            let corner_at = |x_scale: f64, y_scale: f64| {
                [
                    origin[0] + x[0] * x_scale + y[0] * y_scale,
                    origin[1] + x[1] * x_scale + y[1] * y_scale,
                    origin[2] + x[2] * x_scale + y[2] * y_scale,
                ]
            };
            let corners = [
                origin,
                corner_at(length, 0.0),
                corner_at(length, height),
                corner_at(0.0, height),
            ];
            return (0..4)
                .map(|index| [corners[index], corners[(index + 1) % 4]])
                .collect();
        }
        pending
            .corner_frame(point, plane)
            .map_or_else(|_| Vec::new(), |frame| frame.wire())
    }
}

#[allow(dead_code)]
fn _curve_is_rectangle(_: &Curve) {}
