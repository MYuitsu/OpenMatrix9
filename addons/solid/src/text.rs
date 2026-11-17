// SPDX-License-Identifier: LGPL-2.1-or-later
//! Text (Rhino 5 `TextObject`, Matrix Solid > Text): the letters of an installed TrueType or
//! OpenType font as exact outline curves, planar surfaces or solids.
//!
//! - The panel holds the Text Object dialog's options: the text, the font (name, Bold,
//!   Italic), what to create (Curves, Surfaces, Solids), Height, Solid thickness, Add
//!   spacing, Lower case as small caps, Allow single-stroke fonts and Group objects.
//! - Glyph outlines are read with `ttf-parser`; every line and quadratic piece is raised to
//!   a cubic Bézier without change of shape, so each contour is one exact cubic NURBS curve.
//! - Height is the height of the capital letters (the font's cap height).
//! - Solids are the outlines extruded by the thickness with their holes (`extruded`, as
//!   Extrude Curve Solid=Yes); surfaces are the planar faces with the holes trimmed.
//! - Fillet radius (solids only): the top edges of each letter rounded, the bottom kept flat
//!   ([`rounded`]).

use om9_addon_abi::session::{Geo, Vec3};
use om9_addon_geom::exact::Nc;
use om9_addon_geom::mesh::Frame;
use std::path::PathBuf;
use std::sync::OnceLock;
use ttf_parser::{Face, OutlineBuilder};

/// One installed font face: its file, the index in a collection, family and style.
#[derive(Clone, Debug)]
pub struct FontFace {
    pub path: PathBuf,
    pub index: u32,
    pub family: String,
    pub bold: bool,
    pub italic: bool,
}

fn font_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Ok(w) = std::env::var("WINDIR") {
        dirs.push(PathBuf::from(w).join("Fonts"));
    }
    if let Ok(l) = std::env::var("LOCALAPPDATA") {
        dirs.push(PathBuf::from(l).join("Microsoft").join("Windows").join("Fonts"));
    }
    for d in ["/usr/share/fonts", "/usr/local/share/fonts", "/Library/Fonts", "/System/Library/Fonts"] {
        dirs.push(PathBuf::from(d));
    }
    if let Ok(h) = std::env::var("HOME") {
        dirs.push(PathBuf::from(h).join(".fonts"));
    }
    dirs
}

fn walk(dir: &std::path::Path, depth: usize, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            if depth > 0 {
                walk(&p, depth - 1, out);
            }
        } else if p
            .extension()
            .and_then(|x| x.to_str())
            .is_some_and(|x| matches!(x.to_ascii_lowercase().as_str(), "ttf" | "otf" | "ttc"))
        {
            out.push(p);
        }
    }
}

/// Every installed font face (read once).
pub fn installed() -> &'static [FontFace] {
    static FACES: OnceLock<Vec<FontFace>> = OnceLock::new();
    FACES.get_or_init(|| {
        let mut files = Vec::new();
        for d in font_dirs() {
            walk(&d, 4, &mut files);
        }
        let mut faces = Vec::new();
        for path in files {
            let Ok(data) = std::fs::read(&path) else { continue };
            let n = ttf_parser::fonts_in_collection(&data).unwrap_or(1);
            for index in 0..n {
                let Ok(face) = Face::parse(&data, index) else { continue };
                let family = name(&face, ttf_parser::name_id::TYPOGRAPHIC_FAMILY)
                    .or_else(|| name(&face, ttf_parser::name_id::FAMILY));
                let Some(family) = family else { continue };
                faces.push(FontFace { path: path.clone(), index, family, bold: face.is_bold(), italic: face.is_italic() || face.is_oblique() });
            }
        }
        faces
    })
}

fn name(face: &Face, id: u16) -> Option<String> {
    face.names()
        .into_iter()
        .filter(|n| n.name_id == id && n.is_unicode())
        .find_map(|n| n.to_string())
}

/// The installed face of `family` closest to the asked style (any face of the family when
/// that style is missing, as Windows does).
pub fn find(family: &str, bold: bool, italic: bool) -> Option<FontFace> {
    let want = family.trim().to_lowercase();
    let same: Vec<&FontFace> = installed().iter().filter(|f| f.family.to_lowercase() == want).collect();
    let pool = if same.is_empty() {
        installed().iter().filter(|f| f.family.to_lowercase().starts_with(&want)).collect()
    } else {
        same
    };
    pool.iter()
        .min_by_key(|f| (u8::from(f.bold != bold) * 2 + u8::from(f.italic != italic), f.family.len()))
        .map(|f| (*f).clone())
}

/// A font file's bytes, read once (the preview follows the pointer).
fn font_data(path: &std::path::Path) -> std::io::Result<std::sync::Arc<Vec<u8>>> {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    static CACHE: OnceLock<Mutex<HashMap<PathBuf, Arc<Vec<u8>>>>> = OnceLock::new();
    let cache = CACHE.get_or_init(Default::default);
    if let Some(d) = cache.lock().ok().and_then(|c| c.get(path).cloned()) {
        return Ok(d);
    }
    let d = Arc::new(std::fs::read(path)?);
    if let Ok(mut c) = cache.lock() {
        c.insert(path.to_owned(), d.clone());
    }
    Ok(d)
}

/// Collects a glyph's contours as cubic Bézier chains: `[p0, c1, c2, p1, c1, c2, p2, …]`.
#[derive(Default)]
struct Pen {
    contours: Vec<Vec<[f64; 2]>>,
    cur: Vec<[f64; 2]>,
}

impl Pen {
    fn last(&self) -> [f64; 2] {
        *self.cur.last().unwrap_or(&[0.0, 0.0])
    }
    fn cubic(&mut self, c1: [f64; 2], c2: [f64; 2], p: [f64; 2]) {
        self.cur.extend([c1, c2, p]);
    }
    fn finish(&mut self) {
        if self.cur.len() >= 4 {
            let (a, b) = (self.cur[0], self.last());
            if (a[0] - b[0]).abs() > 1e-9 || (a[1] - b[1]).abs() > 1e-9 {
                self.line_to(a[0] as f32, a[1] as f32);
                // Snap the closing point exactly.
                let n = self.cur.len();
                self.cur[n - 1] = a;
            }
            self.contours.push(std::mem::take(&mut self.cur));
        } else {
            self.cur.clear();
        }
    }
}

impl OutlineBuilder for Pen {
    fn move_to(&mut self, x: f32, y: f32) {
        self.finish();
        self.cur.push([x as f64, y as f64]);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        let a = self.last();
        let b = [x as f64, y as f64];
        let at = |t: f64| [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t];
        self.cubic(at(1.0 / 3.0), at(2.0 / 3.0), b);
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        // Degree elevation: the same parabola as a cubic.
        let a = self.last();
        let q = [x1 as f64, y1 as f64];
        let b = [x as f64, y as f64];
        let c1 = [a[0] + 2.0 / 3.0 * (q[0] - a[0]), a[1] + 2.0 / 3.0 * (q[1] - a[1])];
        let c2 = [b[0] + 2.0 / 3.0 * (q[0] - b[0]), b[1] + 2.0 / 3.0 * (q[1] - b[1])];
        self.cubic(c1, c2, b);
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.cubic([x1 as f64, y1 as f64], [x2 as f64, y2 as f64], [x as f64, y as f64]);
    }
    fn close(&mut self) {
        self.finish();
    }
}

/// What the Text Object dialog asks for.
#[derive(Clone, Debug)]
pub struct Spec {
    pub text: String,
    pub font: String,
    pub bold: bool,
    pub italic: bool,
    pub height: f64,
    /// Extra space between letters, in model units.
    pub spacing: f64,
    /// Lower case letters as small capitals of this size (fraction of the capitals).
    pub small_caps: Option<f64>,
}

/// One letter's contours, in the text's own plane (x along the line, y up): cubic Bézier
/// chains `[p0, c1, c2, p1, …]`, closed.
pub type Glyph = Vec<Vec<[f64; 2]>>;

/// The letters laid out in the text's plane, starting at its origin. Lines go down by
/// 1.6 × height.
pub fn layout(spec: &Spec) -> Result<Vec<Glyph>, String> {
    let ff = find(&spec.font, spec.bold, spec.italic).ok_or_else(|| format!("Text: font \"{}\" is not installed", spec.font))?;
    let data = font_data(&ff.path).map_err(|e| format!("Text: {}: {e}", ff.path.display()))?;
    let face = Face::parse(&data, ff.index).map_err(|e| format!("Text: {}: {e}", ff.path.display()))?;
    let upem = face.units_per_em() as f64;
    // The capitals' height: the top of "H" (fonts' own cap-height field is often rounded).
    let cap = face
        .glyph_index('H')
        .and_then(|g| face.glyph_bounding_box(g))
        .map(|b| b.y_max as f64)
        .or_else(|| face.capital_height().map(|c| c as f64))
        .filter(|c| *c > 0.0)
        .unwrap_or(0.7 * upem);
    let scale = spec.height / cap;
    let mut out = Vec::new();
    let (mut x, mut y) = (0.0f64, 0.0f64);
    for ch in spec.text.chars() {
        if ch == '\n' {
            x = 0.0;
            y -= 1.6 * spec.height;
            continue;
        }
        let (c, s) = match spec.small_caps {
            Some(f) if ch.is_lowercase() => (ch.to_uppercase().next().unwrap_or(ch), scale * f),
            _ => (ch, scale),
        };
        let Some(gid) = face.glyph_index(c) else {
            x += 0.5 * spec.height;
            continue;
        };
        let mut pen = Pen::default();
        face.outline_glyph(gid, &mut pen);
        pen.finish();
        let glyph: Glyph = pen
            .contours
            .iter()
            .map(|contour| contour.iter().map(|p| [x + p[0] * s, y + p[1] * s]).collect())
            .collect();
        if !glyph.is_empty() {
            out.push(glyph);
        }
        x += face.glyph_hor_advance(gid).unwrap_or(0) as f64 * s + spec.spacing;
    }
    if out.is_empty() {
        return Err("Text: no letters to make".into());
    }
    Ok(out)
}

/// The text's outlines, laid out on `frame` (x along the line, y up), starting at its origin.
/// Each contour is one closed cubic NURBS curve.
pub fn outlines(spec: &Spec, frame: &Frame) -> Result<Vec<Nc>, String> {
    Ok(layout(spec)?.iter().flatten().map(|contour| contour_curve(contour, frame)).collect())
}

fn contour_curve(contour: &[[f64; 2]], frame: &Frame) -> Nc {
    let pts: Vec<Vec3> = contour.iter().map(|p| frame.point([p[0], p[1], 0.0])).collect();
    let segs = (pts.len() - 1) / 3;
    let mut knots = vec![0.0; 4];
    for k in 1..segs {
        knots.extend([k as f64 / segs as f64; 3]);
    }
    knots.extend([1.0; 4]);
    Nc { degree: 3, points: pts.iter().map(|p| [p[0], p[1], p[2], 1.0]).collect(), knots }
}

/// A contour as a closed polyline (the cubic pieces sampled finely).
fn polyline(contour: &[[f64; 2]], per_piece: usize) -> Vec<[f64; 2]> {
    let mut out = vec![contour[0]];
    for piece in contour[1..].chunks_exact(3) {
        let p0 = *out.last().unwrap();
        let (c1, c2, p1) = (piece[0], piece[1], piece[2]);
        for k in 1..=per_piece {
            let t = k as f64 / per_piece as f64;
            let u = 1.0 - t;
            let b = [u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t];
            out.push([
                b[0] * p0[0] + b[1] * c1[0] + b[2] * c2[0] + b[3] * p1[0],
                b[0] * p0[1] + b[1] * c1[1] + b[2] * c2[1] + b[3] * p1[1],
            ]);
        }
    }
    out
}

/// Signed distance from `p` to a letter (negative inside; holes by the even-odd rule).
fn letter_distance(p: [f64; 2], rings: &[Vec<[f64; 2]>]) -> f64 {
    let mut best = f64::MAX;
    let mut inside = false;
    for ring in rings {
        for w in ring.windows(2) {
            let (a, b) = (w[0], w[1]);
            let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
            let len2 = dx * dx + dy * dy;
            let t = if len2 > 0.0 { (((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / len2).clamp(0.0, 1.0) } else { 0.0 };
            let (qx, qy) = (a[0] + t * dx - p[0], a[1] + t * dy - p[1]);
            best = best.min(qx * qx + qy * qy);
            if (a[1] > p[1]) != (b[1] > p[1]) && p[0] < a[0] + (p[1] - a[1]) / (b[1] - a[1]) * dx {
                inside = !inside;
            }
        }
    }
    if inside { -best.sqrt() } else { best.sqrt() }
}

/// Letters as solids `thickness` thick with their top edges rounded by `radius` (a quarter
/// circle meeting the side walls and the top face tangentially), flat underneath so they sit
/// on a surface. Each letter is one closed mesh, laid out on `frame`.
///
/// The shape is the letter's prism eroded by `radius`, grown back by a ball of `radius`, and
/// cut at the bottom: from the exact 2-D distance `d` to the outline, the signed distance is
/// `|max(q, 0)| + min(max(q.x, q.y), 0) - r` with `q = (d + r, z - (t - r))`, the level
/// surface of which is meshed on a fine grid.
pub fn rounded(glyphs: &[Glyph], frame: &Frame, thickness: f64, radius: f64, height: f64) -> Vec<om9_addon_geom::mesh::Mesh> {
    use om9_addon_geom::sdf::{Grid, level};
    let t = thickness.max(1e-3);
    let r = radius.clamp(0.0, t);
    let cell = (r / 3.0).min(height / 90.0).max(height / 400.0).max(1e-3);
    // A whole number of cells through the thickness: the top face lies on grid points, so it
    // is exactly flat and exactly `t` high.
    let cell = t / (t / cell).ceil();
    let mut out = Vec::new();
    for glyph in glyphs {
        let rings: Vec<Vec<[f64; 2]>> = glyph.iter().map(|c| polyline(c, 10)).collect();
        let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
        for p in rings.iter().flatten() {
            for i in 0..2 {
                lo[i] = lo[i].min(p[i]);
                hi[i] = hi[i].max(p[i]);
            }
        }
        let pad = 2.0 * cell;
        let origin = [lo[0] - pad, lo[1] - pad, -pad];
        let dims = [
            ((hi[0] - lo[0] + 2.0 * pad) / cell).ceil() as usize + 1,
            ((hi[1] - lo[1] + 2.0 * pad) / cell).ceil() as usize + 1,
            ((t + 2.0 * pad) / cell).ceil() as usize + 1,
        ];
        let plane: Vec<f64> = (0..dims[1])
            .flat_map(|j| (0..dims[0]).map(move |i| (i, j)))
            .map(|(i, j)| letter_distance([origin[0] + i as f64 * cell, origin[1] + j as f64 * cell], &rings))
            .collect();
        let mut values = Vec::with_capacity(plane.len() * dims[2]);
        for k in 0..dims[2] {
            let z = origin[2] + k as f64 * cell;
            for d in &plane {
                let (qx, qy) = (d + r, z - (t - r));
                let outside = (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt();
                let sd = outside + qx.max(qy).min(0.0) - r;
                values.push(sd.max(-z));
            }
        }
        let mut m = level(&Grid { origin, cell, dims, values }, 0.0);
        if m.triangles.is_empty() {
            continue;
        }
        for v in &mut m.vertices {
            *v = frame.point(*v);
        }
        out.push(m);
    }
    out
}

/// The created objects: the outline curves, the planar faces, or the solids.
pub fn build(curves: &[Nc], create: &str, thickness: f64, normal: Vec3) -> Vec<Geo> {
    match create {
        "curves" => curves.iter().map(|c| c.to_geo()).collect(),
        "surfaces" => om9_surfaces::planar_faces(curves),
        _ => om9_surfaces::extruded(curves, [0.0; 3], [normal[0] * thickness, normal[1] * thickness, normal[2] * thickness], true),
    }
}
