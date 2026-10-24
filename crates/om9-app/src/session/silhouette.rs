//! Silhouette (`Silhouette`, OM9-CURVE-053): surface contour seen along the CPlane normal.

use om9_core::geometry::{Geometry, Point3};

use super::{Controller, Error};

pub const SILHOUETTE: &str = "OM9-CURVE-053";

impl Controller {
    pub fn dispatch_silhouette(&mut self) -> Result<(), Error> {
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
        let Geometry::Surface(surface) = &entity.geometry else {
            return Err(Error::Invalid("surface selection"));
        };
        let surface = surface.clone();
        let plane = self.input_plane();
        let view = [
            plane.x[1] * plane.y[2] - plane.x[2] * plane.y[1],
            plane.x[2] * plane.y[0] - plane.x[0] * plane.y[2],
            plane.x[0] * plane.y[1] - plane.x[1] * plane.y[0],
        ];
        let steps = 48;
        let [[u0, u1], [v0, v1]] = surface.domain;
        let mut grid = vec![vec![[0.0; 3]; steps + 1]; steps + 1];
        let mut normals = vec![vec![[0.0; 3]; steps + 1]; steps + 1];
        for (i, row) in grid.iter_mut().enumerate() {
            for (j, cell) in row.iter_mut().enumerate() {
                let u = u0 + (u1 - u0) * i as f64 / steps as f64;
                let v = v0 + (v1 - v0) * j as f64 / steps as f64;
                *cell = om9_core::surface_kernel::evaluate(
                    &surface,
                    u,
                    v,
                    self.coordinator.limits(),
                )?;
                normals[i][j] = om9_core::surface_kernel::normal(
                    &surface,
                    u,
                    v,
                    self.coordinator.limits(),
                )?;
            }
        }
        let signed = |normal: Point3| {
            let value = normal[0] * view[0] + normal[1] * view[1] + normal[2] * view[2];
            if value.abs() < 1e-9 {
                1e-9
            } else {
                value
            }
        };
        let mut segments: Vec<[Point3; 2]> = Vec::new();
        for i in 0..steps {
            for j in 0..steps {
                let corners = [
                    (grid[i][j], signed(normals[i][j])),
                    (grid[i + 1][j], signed(normals[i + 1][j])),
                    (grid[i + 1][j + 1], signed(normals[i + 1][j + 1])),
                    (grid[i][j + 1], signed(normals[i][j + 1])),
                ];
                let mut crossings = Vec::new();
                for edge in 0..4 {
                    let (a, da) = corners[edge];
                    let (b, db) = corners[(edge + 1) % 4];
                    if da == 0.0 {
                        crossings.push(a);
                    } else if da * db < 0.0 {
                        let t = da / (da - db);
                        crossings.push([
                            a[0] + (b[0] - a[0]) * t,
                            a[1] + (b[1] - a[1]) * t,
                            a[2] + (b[2] - a[2]) * t,
                        ]);
                    }
                }
                if crossings.len() == 2 {
                    segments.push([crossings[0], crossings[1]]);
                } else if crossings.len() == 4 {
                    let signed_center = corners
                        .iter()
                        .map(|(_, value)| *value)
                        .sum::<f64>();
                    if signed_center < 0.0 {
                        segments.push([crossings[0], crossings[1]]);
                        segments.push([crossings[2], crossings[3]]);
                    } else {
                        segments.push([crossings[1], crossings[2]]);
                        segments.push([crossings[3], crossings[0]]);
                    }
                }
            }
        }
        if segments.is_empty() {
            return Err(Error::Invalid("surface has no silhouette along the view"));
        }
        let mut used = vec![false; segments.len()];
        let mut chains: Vec<Vec<Point3>> = Vec::new();
        for start in 0..segments.len() {
            if used[start] {
                continue;
            }
            used[start] = true;
            let mut chain = vec![segments[start][0], segments[start][1]];
            loop {
                let tail = *chain.last().expect("chain tail");
                let head = chain[0];
                let mut extended = false;
                for index in 0..segments.len() {
                    if used[index] {
                        continue;
                    }
                    let [a, b] = segments[index];
                    if distance(a, tail) < 1e-6 {
                        used[index] = true;
                        chain.push(b);
                        extended = true;
                        break;
                    }
                    if distance(b, tail) < 1e-6 {
                        used[index] = true;
                        chain.push(a);
                        extended = true;
                        break;
                    }
                    if distance(a, head) < 1e-6 {
                        used[index] = true;
                        chain.insert(0, b);
                        extended = true;
                        break;
                    }
                    if distance(b, head) < 1e-6 {
                        used[index] = true;
                        chain.insert(0, a);
                        extended = true;
                        break;
                    }
                }
                if !extended {
                    break;
                }
            }
            if chain.len() >= 2 {
                chains.push(chain);
            }
        }
        let mut merged = true;
        while merged {
            merged = false;
            'outer: for left in 0..chains.len() {
                for right in (left + 1)..chains.len() {
                    let left_end = *chains[left].last().expect("chain end");
                    let left_start = chains[left][0];
                    let right_start = chains[right][0];
                    let right_end = *chains[right].last().expect("chain end");
                    let combined = if distance(left_end, right_start) < 1e-6 {
                        let mut combined = chains[left].clone();
                        combined.extend(chains[right].iter().copied());
                        Some(combined)
                    } else if distance(left_end, right_end) < 1e-6 {
                        let mut combined = chains[left].clone();
                        combined.extend(chains[right].iter().rev().copied());
                        Some(combined)
                    } else if distance(left_start, right_end) < 1e-6 {
                        let mut combined = chains[right].clone();
                        combined.extend(chains[left].iter().copied());
                        Some(combined)
                    } else if distance(left_start, right_start) < 1e-6 {
                        let mut combined = chains[right].iter().rev().copied().collect::<Vec<_>>();
                        combined.extend(chains[left].iter().copied());
                        Some(combined)
                    } else {
                        None
                    };
                    if let Some(combined) = combined {
                        chains[left] = combined;
                        chains.remove(right);
                        merged = true;
                        break 'outer;
                    }
                }
            }
        }
        let mut made = 0;
        for chain in chains {
            let curve = om9_core::curve_build::polyline(&chain)?;
            self.commit_geometry(SILHOUETTE, "Silhouette", Geometry::Curve(curve))?;
            made += 1;
        }
        if made == 0 {
            return Err(Error::Invalid("surface has no silhouette along the view"));
        }
        Ok(())
    }
}

fn distance(a: Point3, b: Point3) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}
