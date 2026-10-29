//! Divide Curve (`Divide`, OM9-CURVE-036): place points along a curve.

use om9_core::geometry::{Curve, Geometry};

use super::{Controller, Error};

pub const DIVIDE: &str = "OM9-CURVE-036";

#[derive(Debug, Clone)]
pub struct DividePending {
    pub curve: Curve,
    pub count: usize,
}

impl DividePending {
    pub fn prompt(&self) -> &'static str {
        "Count=, or Enter to divide"
    }
}

impl Controller {
    pub fn dispatch_divide(&mut self) -> Result<(), Error> {
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
        self.divide_pending = Some(DividePending {
            curve: curve.clone(),
            count: 4,
        });
        Ok(())
    }

    pub fn submit_divide_text(&mut self, text: &str) -> Option<Result<(), Error>> {
        let mut pending = self.divide_pending.take()?;
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
                self.divide_pending = Some(pending);
                return Some(Err(Error::Invalid("count must be 2..100000")));
            };
            pending.count = count;
            self.divide_pending = Some(pending);
            return Some(Ok(()));
        }
        if !text.trim().is_empty() {
            self.divide_pending = Some(pending);
            return Some(Err(Error::Invalid("Divide accepts Count= or Enter")));
        }
        let stations = match om9_core::surface_kernel::arc_length_stations(
            &pending.curve,
            pending.count + 1,
            self.coordinator.limits(),
        ) {
            Ok(stations) => stations,
            Err(error) => return Some(Err(error)),
        };
        for (point, _) in stations.iter().skip(1).take(pending.count - 1) {
            if let Err(error) = self.commit_geometry(DIVIDE, "Divide", Geometry::Point(*point)) {
                return Some(Err(error));
            }
        }
        Some(Ok(()))
    }

    pub fn divide_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        if self.divide_pending.is_some() {
            vec![("Count=", "Count="), ("Cancel", "Cancel")]
        } else {
            Vec::new()
        }
    }
}
