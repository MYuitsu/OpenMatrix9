//! Text contours from a font.
use qymcad_core::text::text_outline_contours;

fn find_font() -> Option<Vec<u8>> {
    for p in [
        "/usr/share/fonts/TTF/OpenSans-Regular.ttf",
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/liberation/LiberationSans-Regular.ttf",
    ] {
        if let Ok(b) = std::fs::read(p) {
            return Some(b);
        }
    }
    None
}

#[test]
fn text_makes_glyph_contours() {
    let Some(font) = find_font() else {
        eprintln!("no font found, skipping");
        return;
    };
    let cs = text_outline_contours(&font, 0, "AB", 10.0, 0.0, 0.0);
    assert!(cs.len() >= 3, "A gives an outer loop and a hole, B an outer loop and two holes, so at least three contours; got {}", cs.len());
    for c in &cs {
        assert!(c.closed && c.points.len() >= 3, "a glyph contour is closed");
    }
}

/// The height of text is the actual height of the glyphs rather than merely a factor in a formula.
#[test]
fn text_height_and_origin_are_honest() {
    let Some(font) = find_font() else {
        eprintln!("no font found, skipping");
        return;
    };
    let bbox = |cs: &[qymcad_core::geom::Contour]| {
        let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
        for c in cs {
            for p in &c.points {
                x0 = x0.min(p.x);
                y0 = y0.min(p.y);
                x1 = x1.max(p.x);
                y1 = y1.max(p.y);
            }
        }
        (x0, y0, x1, y1)
    };
    let a = text_outline_contours(&font, 0, "H", 10.0, 0.0, 0.0);
    let b = text_outline_contours(&font, 0, "H", 20.0, 0.0, 0.0);
    assert!(!a.is_empty() && !b.is_empty(), "the glyph H yields contours");
    let (_, ay0, _, ay1) = bbox(&a);
    let (_, by0, _, by1) = bbox(&b);
    let (ha, hb) = (ay1 - ay0, by1 - by0);
    assert!((hb / ha - 2.0).abs() < 0.02, "doubling the height doubles the glyph: {ha:.3} -> {hb:.3}");

    // the origin shifts the whole label by exactly the given vector
    let moved = text_outline_contours(&font, 0, "H", 10.0, 7.0, -3.0);
    let (mx0, my0, _, _) = bbox(&moved);
    let (ax0, ay0b, _, _) = bbox(&a);
    assert!((mx0 - ax0 - 7.0).abs() < 1e-9 && (my0 - ay0b + 3.0).abs() < 1e-9, "the label is shifted by (7,-3)");
}

/// A space advances the pen, or words run into one another, while an unknown character is simply skipped:
/// failing on it is not acceptable.
#[test]
fn spaces_advance_and_unknown_glyphs_are_skipped() {
    let Some(font) = find_font() else {
        eprintln!("no font found, skipping");
        return;
    };
    let right = |cs: &[qymcad_core::geom::Contour]| cs.iter().flat_map(|c| c.points.iter()).map(|p| p.x).fold(f64::MIN, f64::max);
    let no_space = text_outline_contours(&font, 0, "AA", 10.0, 0.0, 0.0);
    let with_space = text_outline_contours(&font, 0, "A A", 10.0, 0.0, 0.0);
    assert!(right(&with_space) > right(&no_space) + 1.0, "the space separated the letters: {} against {}", right(&with_space), right(&no_space));
    // exotic characters and emoji may yield no contours, but must not panic
    let _ = text_outline_contours(&font, 0, "🙂", 10.0, 0.0, 0.0);
}

/// A damaged or empty font gives an empty result rather than a panic: the font file comes from the system and
/// may well be truncated or of another format.
#[test]
fn broken_font_returns_empty_without_panic() {
    assert!(text_outline_contours(&[], 0, "ABC", 10.0, 0.0, 0.0).is_empty(), "an empty font");
    assert!(text_outline_contours(b"not a font at all", 0, "ABC", 10.0, 0.0, 0.0).is_empty(), "rubbish instead of a font");
    if let Some(font) = find_font() {
        let half = font[..font.len() / 3].to_vec();
        let _ = text_outline_contours(&half, 0, "ABC", 10.0, 0.0, 0.0); // truncated: what matters is not crashing
        assert!(text_outline_contours(&font, 0, "", 10.0, 0.0, 0.0).is_empty(), "an empty string gives nothing");
    }
}

/// THE FACE INDEX IS OBEYED, not assumed to be zero.
///
/// A `.ttc` collection holds several faces in one file, and a person choosing the second of them must get the
/// second. Measured here the other way round, on an ordinary font, which holds exactly one: face zero draws
/// the string and face one does not exist, so it draws nothing. A parser that ignored the index would draw
/// the same thing twice.
#[test]
fn the_face_index_reaches_the_parser() {
    let font = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/fonts/LiberationSans-Bold.ttf")).expect("the font shipped with the repository");
    assert_eq!(qymcad_core::text::faces_in(&font), 1, "setup: this file is not a collection");
    assert!(!text_outline_contours(&font, 0, "H", 10.0, 0.0, 0.0).is_empty(), "the only face of the file draws nothing");
    assert!(text_outline_contours(&font, 1, "H", 10.0, 0.0, 0.0).is_empty(), "a face the file does not hold drew something: the index is ignored");
}
