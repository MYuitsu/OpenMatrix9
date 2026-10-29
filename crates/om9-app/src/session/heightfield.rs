//! Heightfield from Image (`Heightfield`, OM9-SURFACE-031).

use om9_core::geometry::{Geometry, NurbsSurface};

use super::{Controller, Error};

pub const HEIGHTFIELD: &str = "OM9-SURFACE-031";

#[derive(Debug, Clone)]
pub struct HeightfieldPending {
    pub file: Option<String>,
    pub height: f64,
    pub size: f64,
}

impl HeightfieldPending {
    pub fn prompt(&self) -> &'static str {
        "File=path, Height=, Size=, or Enter to build"
    }
}

impl Controller {
    pub fn dispatch_heightfield(&mut self) -> Result<(), Error> {
        self.heightfield_pending = Some(HeightfieldPending {
            file: None,
            height: 10.0,
            size: 100.0,
        });
        Ok(())
    }

    pub fn submit_heightfield_text(&mut self, text: &str) -> Option<Result<(), Error>> {
        let mut pending = self.heightfield_pending.take()?;
        let option = text.trim().to_ascii_lowercase();
        if option == "cancel" || option == "esc" {
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("File=") {
            pending.file = Some(value.trim().to_string());
            self.heightfield_pending = Some(pending);
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("Height=") {
            let Some(height) = value.parse::<f64>().ok().filter(|value| value.is_finite()) else {
                self.heightfield_pending = Some(pending);
                return Some(Err(Error::Invalid("height")));
            };
            pending.height = height;
            self.heightfield_pending = Some(pending);
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("Size=") {
            let Some(size) = value
                .parse::<f64>()
                .ok()
                .filter(|value| value.is_finite() && *value > 0.0)
            else {
                self.heightfield_pending = Some(pending);
                return Some(Err(Error::Invalid("size")));
            };
            pending.size = size;
            self.heightfield_pending = Some(pending);
            return Some(Ok(()));
        }
        if !text.trim().is_empty() {
            self.heightfield_pending = Some(pending);
            return Some(Err(Error::Invalid(
                "Heightfield accepts File=, Height=, Size= or Enter",
            )));
        }
        let Some(file) = pending.file else {
            self.heightfield_pending = Some(pending);
            return Some(Err(Error::Invalid("File=path is required")));
        };
        let image = match image::open(&file) {
            Ok(image) => image.to_luma8(),
            Err(_) => return Some(Err(Error::Invalid("image load"))),
        };
        let grid = 32usize;
        let image = image::imageops::resize(
            &image,
            grid as u32,
            grid as u32,
            image::imageops::FilterType::Triangle,
        );
        let step = pending.size / (grid as f64 - 1.0);
        let mut poles = Vec::with_capacity(grid * grid);
        for i in 0..grid {
            for j in 0..grid {
                let value = image.get_pixel(i as u32, j as u32).0[0] as f64 / 255.0;
                poles.push([
                    i as f64 * step - pending.size * 0.5,
                    j as f64 * step - pending.size * 0.5,
                    value * pending.height,
                ]);
            }
        }
        let mut knots = vec![0.0];
        for index in 1..grid - 1 {
            knots.push(index as f64 / (grid as f64 - 1.0));
        }
        knots.push(1.0);
        let mut clamped = vec![0.0];
        clamped.extend(knots.iter().copied());
        clamped.push(1.0);
        let surface = NurbsSurface {
            degree: [1, 1],
            size: [grid, grid],
            poles,
            weights: vec![1.0; grid * grid],
            knots: [clamped.clone(), clamped],
            domain: [[0.0, 1.0], [0.0, 1.0]],
            periodic: [false, false],
            reversed: false,
        };
        Some(self.commit_geometry(HEIGHTFIELD, "Heightfield", Geometry::Surface(surface)))
    }

    pub fn heightfield_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        if self.heightfield_pending.is_some() {
            vec![
                ("File=", "File="),
                ("Height=", "Height="),
                ("Size=", "Size="),
                ("Cancel", "Cancel"),
            ]
        } else {
            Vec::new()
        }
    }
}
