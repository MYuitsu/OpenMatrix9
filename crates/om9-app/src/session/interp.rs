//! Interpolated curve command (`InterpCrv`) ported from the Qt Curve controller.

use om9_core::geometry::{Curve, Geometry, Point3};

use super::{Controller, Error};

pub const INTERP: &str = "OM9-CURVE-003";

#[derive(Debug, Clone)]
pub struct InterpPending {
    pub points: Vec<Point3>,
    pub degree: usize,
    pub close: bool,
}

impl Default for InterpPending {
    fn default() -> Self {
        Self::new()
    }
}

impl InterpPending {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            degree: 3,
            close: false,
        }
    }

    pub fn prompt(&self) -> &'static str {
        if self.points.is_empty() {
            "First interpolation point"
        } else {
            "Next point or Enter to fit"
        }
    }
}

impl Controller {
    pub fn dispatch_interp(&mut self) -> Result<(), Error> {
        self.interp_pending = Some(InterpPending::new());
        Ok(())
    }

    pub fn submit_interp_text(&mut self, text: &str) -> Option<Result<(), Error>> {
        let mut pending = self.interp_pending.take()?;
        let option = text.trim().to_ascii_lowercase();
        if option == "cancel" || option == "esc" {
            return Some(Ok(()));
        }
        if option == "undo" || option == "u" {
            pending.points.pop();
            self.interp_pending = Some(pending);
            return Some(Ok(()));
        }
        if option == "close" || option == "c" {
            pending.close = true;
            self.interp_pending = Some(pending);
            return Some(self.commit_interp());
        }
        if let Some(value) = text.trim().strip_prefix("Degree=") {
            let Some(degree) = value
                .parse::<usize>()
                .ok()
                .filter(|degree| *degree >= 1 && *degree <= 11 && degree % 2 == 1)
            else {
                self.interp_pending = Some(pending);
                return Some(Err(Error::Invalid("degree must be odd 1..11")));
            };
            pending.degree = degree;
            self.interp_pending = Some(pending);
            return Some(Ok(()));
        }
        if text.trim().is_empty() {
            self.interp_pending = Some(pending);
            return Some(self.commit_interp());
        }
        self.interp_pending = Some(pending);
        Some(Err(Error::Invalid(
            "InterpCrv accepts points, Degree=, Close, Undo or Cancel",
        )))
    }

    pub fn accept_interp_point(&mut self, point: Point3) -> Result<(), Error> {
        let Some(mut pending) = self.interp_pending.take() else {
            return Err(Error::Missing("interp command"));
        };
        if pending.points.last().is_some_and(|last| *last == point) {
            self.interp_pending = Some(pending);
            return Err(Error::Invalid("interpolation point repeats"));
        }
        pending.points.push(point);
        self.interp_pending = Some(pending);
        Ok(())
    }

    fn commit_interp(&mut self) -> Result<(), Error> {
        let Some(pending) = self.interp_pending.take() else {
            return Err(Error::Missing("interp command"));
        };
        let mut points = pending.points;
        if pending.close && points.len() >= 3 {
            points.push(points[0]);
        }
        if points.len() < 4 {
            return Err(Error::Invalid("InterpCrv needs four points"));
        }
        if points.len() <= pending.degree {
            return Err(Error::Invalid("InterpCrv needs more points than degree"));
        }
        let curve = om9_core::curve_build::interpolate(
            &points,
            pending.degree,
            om9_core::curve_build::KnotMode::ChordLength,
            self.coordinator.limits(),
        )?;
        self.commit_geometry(INTERP, "InterpCrv", Geometry::Curve(Curve::Nurbs(curve)))
    }

    pub fn interp_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        let Some(pending) = self.interp_pending.as_ref() else {
            return Vec::new();
        };
        let mut tokens = vec![("Degree=", "Degree=")];
        if pending.points.len() >= 4 {
            tokens.push(("Close", "Close"));
        }
        if !pending.points.is_empty() {
            tokens.push(("Undo", "Undo"));
        }
        tokens.push(("Cancel", "Cancel"));
        tokens
    }

    pub fn interp_preview(&self, point: Point3) -> Vec<[Point3; 2]> {
        let Some(pending) = self.interp_pending.as_ref() else {
            return Vec::new();
        };
        if pending.points.is_empty() {
            return Vec::new();
        }
        let mut points = pending.points.clone();
        points.push(point);
        if points.len() > pending.degree
            && let Ok(curve) = om9_core::curve_build::interpolate(
                &points,
                pending.degree,
                om9_core::curve_build::KnotMode::ChordLength,
                self.coordinator.limits(),
            )
            && let Ok(samples) = om9_core::surface_kernel::sample_curve(
                &Curve::Nurbs(curve),
                32,
                self.coordinator.limits(),
            )
        {
            return samples.windows(2).map(|pair| [pair[0], pair[1]]).collect();
        }
        points.windows(2).map(|pair| [pair[0], pair[1]]).collect()
    }
}
