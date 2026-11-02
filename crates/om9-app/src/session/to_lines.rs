//! Convert Curve to Lines (`Convert`, OM9-CURVE-050).

use om9_core::geometry::{Curve, Geometry};

use super::{Controller, Error};

pub const TO_LINES: &str = "OM9-CURVE-050";

#[derive(Debug, Clone)]
pub struct ToLinesPending {
    pub curve: Curve,
    pub count: usize,
}

impl ToLinesPending {
    pub fn prompt(&self) -> &'static str {
        "Count=, or Enter to convert"
    }
}

impl Controller {
    pub fn dispatch_to_lines(&mut self) -> Result<(), Error> {
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
        self.to_lines_pending = Some(ToLinesPending {
            curve: curve.clone(),
            count: 16,
        });
        Ok(())
    }

    pub fn submit_to_lines_text(&mut self, text: &str) -> Option<Result<(), Error>> {
        let mut pending = self.to_lines_pending.take()?;
        let option = text.trim().to_ascii_lowercase();
        if option == "cancel" || option == "esc" {
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("Count=") {
            let Some(count) = value
                .parse::<usize>()
                .ok()
                .filter(|count| *count >= 2 && *count <= 100_000)
            else {
                self.to_lines_pending = Some(pending);
                return Some(Err(Error::Invalid("count must be 2..100000")));
            };
            pending.count = count;
            self.to_lines_pending = Some(pending);
            return Some(Ok(()));
        }
        if !text.trim().is_empty() {
            self.to_lines_pending = Some(pending);
            return Some(Err(Error::Invalid("Convert accepts Count= or Enter")));
        }
        let samples = match om9_core::surface_kernel::sample_curve(
            &pending.curve,
            pending.count + 1,
            self.coordinator.limits(),
        ) {
            Ok(samples) => samples,
            Err(error) => return Some(Err(error)),
        };
        let curve = match om9_core::curve_build::polyline(&samples) {
            Ok(curve) => curve,
            Err(error) => return Some(Err(error)),
        };
        Some(self.commit_geometry(TO_LINES, "Convert", Geometry::Curve(curve)))
    }

    pub fn to_lines_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        if self.to_lines_pending.is_some() {
            vec![("Count=", "Count="), ("Cancel", "Cancel")]
        } else {
            Vec::new()
        }
    }
}
