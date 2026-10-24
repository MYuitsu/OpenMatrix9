//! Polyline command behaviour ported from the FreeCAD Qt Curve controller.

use om9_core::geometry::{Geometry, Point3};

use super::{Controller, Error};

pub const POLYLINE: &str = "OM9-CURVE-001";

#[derive(Debug, Clone, Default)]
pub struct PolylinePending {
    pub points: Vec<Point3>,
    pub close: bool,
}

impl PolylinePending {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn prompt(&self) -> &'static str {
        if self.points.is_empty() {
            "Start of polyline"
        } else {
            "Next point or Close"
        }
    }
}

impl Controller {
    pub fn dispatch_polyline(&mut self) -> Result<(), Error> {
        self.polyline_pending = Some(PolylinePending::new());
        Ok(())
    }

    pub fn submit_polyline_text(&mut self, text: &str) -> Option<Result<(), Error>> {
        let mut pending = self.polyline_pending.take()?;
        let option = text.trim().to_ascii_lowercase();
        if option == "cancel" || option == "esc" {
            return Some(Ok(()));
        }
        if option == "undo" || option == "u" {
            pending.points.pop();
            self.polyline_pending = Some(pending);
            return Some(Ok(()));
        }
        if option == "close" || option == "c" {
            pending.close = true;
            self.polyline_pending = Some(pending);
            return Some(self.commit_polyline());
        }
        if text.trim().is_empty() {
            if pending.points.len() < 2 {
                self.polyline_pending = Some(pending);
                return Some(Err(Error::Invalid("polyline needs two points")));
            }
            self.polyline_pending = Some(pending);
            return Some(self.commit_polyline());
        }
        self.polyline_pending = Some(pending);
        Some(Err(Error::Invalid(
            "polyline accepts points, Close, Undo or Cancel",
        )))
    }

    pub fn accept_polyline_point(&mut self, point: Point3) -> Result<(), Error> {
        let Some(mut pending) = self.polyline_pending.take() else {
            return Err(Error::Missing("polyline command"));
        };
        if pending.points.last().is_some_and(|last| *last == point) {
            self.polyline_pending = Some(pending);
            return Err(Error::Invalid("polyline point repeats"));
        }
        pending.points.push(point);
        self.polyline_pending = Some(pending);
        Ok(())
    }

    fn commit_polyline(&mut self) -> Result<(), Error> {
        let Some(pending) = self.polyline_pending.take() else {
            return Err(Error::Missing("polyline command"));
        };
        let mut points = pending.points;
        if points.len() < 2 {
            return Err(Error::Invalid("polyline needs two points"));
        }
        if pending.close {
            points.push(points[0]);
        }
        let curve = om9_core::curve_build::polyline(&points)?;
        self.commit_geometry(POLYLINE, "Polyline", Geometry::Curve(curve))
    }

    pub fn polyline_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        let Some(pending) = self.polyline_pending.as_ref() else {
            return Vec::new();
        };
        let mut tokens = Vec::new();
        if pending.points.len() >= 2 {
            tokens.push(("Close", "Close"));
        }
        if !pending.points.is_empty() {
            tokens.push(("Undo", "Undo"));
        }
        tokens.push(("Cancel", "Cancel"));
        tokens
    }

    pub fn polyline_preview(&self, point: Point3) -> Vec<[Point3; 2]> {
        let Some(pending) = self.polyline_pending.as_ref() else {
            return Vec::new();
        };
        if pending.points.is_empty() {
            return Vec::new();
        }
        let mut points = pending.points.clone();
        points.push(point);
        if pending.close {
            points.push(points[0]);
        }
        points.windows(2).map(|pair| [pair[0], pair[1]]).collect()
    }
}
