//! Cross Section Profiles (`CrossSectionProfiles`, OM9-CURVE-043).

use super::{Controller, Error};

pub const CROSS_SECTION: &str = "OM9-CURVE-043";

#[derive(Debug, Clone, Copy)]
pub struct CrossSectionPending {
    pub count: usize,
    pub spacing: f64,
}

impl CrossSectionPending {
    pub fn prompt(&self) -> &'static str {
        "Count=, Spacing=, or Enter to generate sections"
    }
}

impl Controller {
    pub fn dispatch_cross_section(&mut self) -> Result<(), Error> {
        let id = self
            .selection
            .iter()
            .next()
            .copied()
            .ok_or(Error::Missing("surface selection"))?;
        let entity = self
            .document()
            .entities
            .get(&id)
            .ok_or(Error::Missing("selected surface"))?;
        if !matches!(entity.geometry, om9_core::geometry::Geometry::Surface(_)) {
            return Err(Error::Invalid("surface selection"));
        }
        self.cross_section_pending = Some(CrossSectionPending {
            count: 3,
            spacing: 10.0,
        });
        Ok(())
    }

    pub fn submit_cross_section_text(&mut self, text: &str) -> Option<Result<(), Error>> {
        let mut pending = self.cross_section_pending.take()?;
        let option = text.trim().to_ascii_lowercase();
        if option == "cancel" || option == "esc" {
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("Count=") {
            let Some(count) = value.parse::<usize>().ok().filter(|value| (1..=64).contains(value)) else {
                self.cross_section_pending = Some(pending);
                return Some(Err(Error::Invalid("count must be 1..64")));
            };
            pending.count = count;
            self.cross_section_pending = Some(pending);
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("Spacing=") {
            let Some(spacing) = value.parse::<f64>().ok().filter(|value| value.is_finite() && *value > 0.0) else {
                self.cross_section_pending = Some(pending);
                return Some(Err(Error::Invalid("spacing must be positive")));
            };
            pending.spacing = spacing;
            self.cross_section_pending = Some(pending);
            return Some(Ok(()));
        }
        if !text.trim().is_empty() {
            self.cross_section_pending = Some(pending);
            return Some(Err(Error::Invalid("CrossSectionProfiles accepts Count=, Spacing= or Enter")));
        }
        let plane = self.input_plane();
        let normal = [
            plane.x[1] * plane.y[2] - plane.x[2] * plane.y[1],
            plane.x[2] * plane.y[0] - plane.x[0] * plane.y[2],
            plane.x[0] * plane.y[1] - plane.x[1] * plane.y[0],
        ];
        let mut made = 0;
        let mut last_error = None;
        for index in 0..pending.count {
            let offset = (index as f64 - (pending.count as f64 - 1.0) * 0.5) * pending.spacing;
            let origin = [
                plane.origin[0] + normal[0] * offset,
                plane.origin[1] + normal[1] * offset,
                plane.origin[2] + normal[2] * offset,
            ];
            if let Err(error) = self.set_cplane(plane.with_origin(origin).ok()?) {
                last_error = Some(error);
                continue;
            }
            match self.dispatch_section() {
                Ok(()) => made += 1,
                Err(error) => last_error = Some(error),
            }
        }
        if let Err(error) = self.set_cplane(plane) {
            return Some(Err(error));
        }
        if made == 0 {
            return Some(Err(last_error.unwrap_or(Error::Invalid("no section profiles"))));
        }
        Some(Ok(()))
    }

    pub fn cross_section_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        if self.cross_section_pending.is_some() {
            vec![
                ("Count=", "Count="),
                ("Spacing=", "Spacing="),
                ("Cancel", "Cancel"),
            ]
        } else {
            Vec::new()
        }
    }
}
