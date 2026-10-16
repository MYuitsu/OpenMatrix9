//! Text contours from a TTF or OTF font, for labels in a sketch and for engraving.
//!
//! Glyph outlines are extracted through `ttf-parser` and the Bezier splines are flattened into polylines. Font
//! coordinates run Y upwards, which matches the sketch coordinate system.

use crate::geom::{Contour, Point2};

/// A collector of glyph outlines: it flattens quadratic and cubic Beziers into segments.
#[derive(Default)]
struct Outline {
    contours: Vec<Vec<(f32, f32)>>,
    cur: Vec<(f32, f32)>,
    last: (f32, f32),
}

impl Outline {
    /// Close off the loop being collected.
    ///
    /// A POINT REPEATED AT THE SEAM IS A SEGMENT OF ZERO LENGTH, and OCCT will not build a wire out of one:
    /// the extrusion of the contour comes back empty and the rebuild says "Extrude failed (check the
    /// contour)". The two font formats differ exactly here - a glyf outline (TTF) closes by RETURNING to its
    /// starting point, a CFF one (OTF) does not.
    ///
    /// Reported behaviour: "only the built-in font extrudes, I tried four fonts of my own and none works".
    /// Measured: the system default there was an .otf and every font chosen by hand was a .ttf; each TTF loop
    /// carried exactly one such pair and not one of them extruded.
    fn flush(&mut self) {
        let mut c = std::mem::take(&mut self.cur);
        c.dedup_by(|a, b| a.0 == b.0 && a.1 == b.1);
        while c.len() >= 2 && c[0] == c[c.len() - 1] {
            c.pop();
        }
        if c.len() >= 3 {
            self.contours.push(c);
        }
    }
}

impl ttf_parser::OutlineBuilder for Outline {
    fn move_to(&mut self, x: f32, y: f32) {
        self.flush();
        self.cur.push((x, y));
        self.last = (x, y);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.cur.push((x, y));
        self.last = (x, y);
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let (p0x, p0y) = self.last;
        let n = 8;
        for k in 1..=n {
            let t = k as f32 / n as f32;
            let mt = 1.0 - t;
            let px = mt * mt * p0x + 2.0 * mt * t * x1 + t * t * x;
            let py = mt * mt * p0y + 2.0 * mt * t * y1 + t * t * y;
            self.cur.push((px, py));
        }
        self.last = (x, y);
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let (p0x, p0y) = self.last;
        let n = 10;
        for k in 1..=n {
            let t = k as f32 / n as f32;
            let mt = 1.0 - t;
            let px = mt * mt * mt * p0x + 3.0 * mt * mt * t * x1 + 3.0 * mt * t * t * x2 + t * t * t * x;
            let py = mt * mt * mt * p0y + 3.0 * mt * mt * t * y1 + 3.0 * mt * t * t * y2 + t * t * t * y;
            self.cur.push((px, py));
        }
        self.last = (x, y);
    }
    fn close(&mut self) {
        self.flush();
    }
}

/// The contours of the string `text` at a height of `height` mm, with its lower-left corner at (ox, oy),
/// drawn with face number `index` of the file.
///
/// Every glyph yields its own contours: an outer loop plus the inner loops of letters such as O and A.
///
/// THE FACE INDEX IS NOT ALWAYS ZERO. A `.ttc` collection holds several faces in one file - a family with its
/// weights, or several families outright - and zero would silently draw the first of them whatever the person
/// chose from the list.
pub fn text_outline_contours(font: &[u8], index: u32, text: &str, height: f64, ox: f64, oy: f64) -> Vec<Contour> {
    let Ok(face) = ttf_parser::Face::parse(font, index) else {
        return Vec::new();
    };
    let upem = face.units_per_em() as f64;
    if upem <= 0.0 {
        return Vec::new();
    }
    let scale = height / upem;
    let mut out = Vec::new();
    let mut pen = 0.0_f64;
    for ch in text.chars() {
        if ch == ' ' {
            pen += face.glyph_index(' ').and_then(|g| face.glyph_hor_advance(g)).unwrap_or((upem * 0.3) as u16) as f64;
            continue;
        }
        let Some(gid) = face.glyph_index(ch) else { continue };
        let mut b = Outline::default();
        if face.outline_glyph(gid, &mut b).is_some() {
            b.flush();
            for c in &b.contours {
                let pts: Vec<Point2> = c.iter().map(|(x, y)| Point2::new(ox + (pen + *x as f64) * scale, oy + *y as f64 * scale)).collect();
                if pts.len() >= 3 {
                    out.push(Contour::closed(pts));
                }
            }
        }
        pen += face.glyph_hor_advance(gid).unwrap_or(0) as f64;
    }
    out
}

/// What a face inside a font file calls itself: the family and the style, as "Liberation Sans" and "Bold".
///
/// The typographic names (ids 16 and 17) are preferred over the plain ones (1 and 2): for a face like
/// "Semibold Italic" the plain family carries the style with it and would name a family nobody has, so the
/// list of installed fonts would show one family per weight.
pub fn face_name(font: &[u8], index: u32) -> Option<(String, String)> {
    let face = ttf_parser::Face::parse(font, index).ok()?;
    let pick = |want: u16| face.names().into_iter().find(|n| n.name_id == want && n.is_unicode()).and_then(|n| n.to_string()).filter(|s| !s.trim().is_empty());
    let family = pick(ttf_parser::name_id::TYPOGRAPHIC_FAMILY).or_else(|| pick(ttf_parser::name_id::FAMILY))?;
    let style = pick(ttf_parser::name_id::TYPOGRAPHIC_SUBFAMILY).or_else(|| pick(ttf_parser::name_id::SUBFAMILY)).unwrap_or_else(|| "Regular".into());
    Some((family, style))
}

/// The family name alone - what is shown in the bar and recorded with a label.
pub fn family_name(font: &[u8], index: u32) -> Option<String> {
    face_name(font, index).map(|(f, _)| f)
}

/// How many faces a file holds: a `.ttc` collection carries several, an ordinary font one.
pub fn faces_in(font: &[u8]) -> u32 {
    ttf_parser::fonts_in_collection(font).unwrap_or(1)
}

/// Can this face draw every character of `text` - that is, is there anything to write with.
///
/// An icon font has thousands of glyphs and not one letter; chosen by mistake it writes nothing at all, and
/// the sketch comes out empty with no complaint from anybody. Whitespace is not asked about: it advances the
/// pen and needs no glyph.
pub fn can_write(font: &[u8], index: u32, text: &str) -> bool {
    let Ok(face) = ttf_parser::Face::parse(font, index) else { return false };
    text.chars().filter(|c| !c.is_whitespace()).all(|c| face.glyph_index(c).is_some())
}
