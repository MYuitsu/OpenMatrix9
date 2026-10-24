//! Extend Curve (`Extend`, OM9-CURVE-027): straight extension along the end tangent.

use om9_core::geometry::{Curve, Geometry};

use super::{Controller, Error};

pub const EXTEND: &str = "OM9-CURVE-027";

#[derive(Debug, Clone)]
pub struct ExtendPending {
    pub curve: Curve,
    pub length: f64,
    pub at_start: bool,
}

impl ExtendPending {
    pub fn prompt(&self) -> &'static str {
        "Length=, or Enter to extend by 5"
    }
}

impl Controller {
    pub fn dispatch_extend_curve(&mut self) -> Result<(), Error> {
        let id = self
            .selection
            .iter()
            .next()
            .copied()
            .ok_or(Error::Missing("curve selection"))?;
        let entity = self
            .document()
            .entities
            .get(&id)
            .ok_or(Error::Missing("selected curve"))?;
        let Geometry::Curve(curve) = &entity.geometry else {
            return Err(Error::Invalid("curve selection"));
        };
        self.extend_curve_pending = Some(ExtendPending {
            curve: curve.clone(),
            length: 5.0,
            at_start: false,
        });
        Ok(())
    }

    pub fn submit_extend_curve_text(&mut self, text: &str) -> Option<Result<(), Error>> {
        let mut pending = self.extend_curve_pending.take()?;
        let option = text.trim().to_ascii_lowercase();
        if option == "cancel" || option == "esc" {
            return Some(Ok(()));
        }
        if option == "atstart" || option == "start" {
            pending.at_start = true;
            self.extend_curve_pending = Some(pending);
            return Some(Ok(()));
        }
        if option == "atend" || option == "end" {
            pending.at_start = false;
            self.extend_curve_pending = Some(pending);
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("Length=") {
            let Some(length) = value.parse::<f64>().ok().filter(|value| value.is_finite() && *value > 0.0) else {
                self.extend_curve_pending = Some(pending);
                return Some(Err(Error::Invalid("length must be positive")));
            };
            pending.length = length;
            self.extend_curve_pending = Some(pending);
            return Some(Ok(()));
        }
        if !text.trim().is_empty() {
            self.extend_curve_pending = Some(pending);
            return Some(Err(Error::Invalid("Extend accepts Length= or Enter")));
        }
        let samples = match om9_core::surface_kernel::sample_curve(
            &pending.curve,
            16,
            self.coordinator.limits(),
        ) {
            Ok(samples) => samples,
            Err(error) => return Some(Err(error)),
        };
        if samples.len() < 2 {
            return Some(Err(Error::Invalid("extend source samples")));
        }
        let (anchor, direction) = if pending.at_start {
            let start = samples[0];
            let after = samples[1];
            (start, [start[0] - after[0], start[1] - after[1], start[2] - after[2]])
        } else {
            let end = samples[samples.len() - 1];
            let before = samples[samples.len() - 2];
            (end, [end[0] - before[0], end[1] - before[1], end[2] - before[2]])
        };
        let norm = (direction[0].powi(2) + direction[1].powi(2) + direction[2].powi(2)).sqrt();
        if norm <= 1e-12 {
            return Some(Err(Error::Invalid("extend tangent")));
        }
        let target = [
            anchor[0] + direction[0] / norm * pending.length,
            anchor[1] + direction[1] / norm * pending.length,
            anchor[2] + direction[2] / norm * pending.length,
        ];
        Some(self.commit_geometry(
            EXTEND,
            "Extend",
            Geometry::Curve(Curve::Line {
                start: anchor,
                end: target,
                domain: [0.0, 1.0],
            }),
        ))
    }

    pub fn extend_curve_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        let Some(pending) = self.extend_curve_pending.as_ref() else {
            return Vec::new();
        };
        let mut tokens = vec![("Length=", "Length=")];
        tokens.push(if pending.at_start {
            ("End", "End")
        } else {
            ("Start", "Start")
        });
        tokens.push(("Cancel", "Cancel"));
        tokens
    }
}
