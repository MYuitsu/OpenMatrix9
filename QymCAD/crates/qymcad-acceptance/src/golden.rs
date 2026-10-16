//! A PICTURE OF THE WINDOW AGAINST THE ONE SIGNED FOR IT.
//!
//! What the words and the numbers of a check cannot see - a panel drawn over the canvas, a body in the wrong colour,
//! a toolbar gone blank - shows in a picture. The signed pictures live in `golden/`. A picture is signed by a
//! person who looked at it, never by a run: a missing or different picture fails and leaves the candidate beside
//! the build for that person to look at, and `QYMCAD_GOLDEN_WRITE=<name>[,<name>]` writes the named ones - the
//! word that a person has looked and agrees.
use qymcad::Picture;

/// A channel of a pixel further than this from the signed one makes the pixel different. Anti-aliasing and the
/// rounding of a raster move a channel by a few steps; a colour that is wrong moves it by dozens.
pub const CHANNEL_TOLERANCE: u8 = 32;

/// The share of pixels that may differ before the picture does: 0.2%, about two thousand pixels of a window
/// 1280x800 - an edge of a body stepped otherwise, a caret blinking; not a word, not a button.
pub const PIXEL_SHARE: f64 = 0.002;

/// Where the signed pictures are.
fn signed_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("golden")
}

/// Where the candidates of a run are left for a person to look at.
fn candidates_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/golden-candidates")
}

/// HOW TWO PICTURES DIFFER: the pixels that differ past the tolerance, of all of them; `Err` when their sizes
/// differ, which no tolerance covers.
pub fn difference(a: &Picture, b: &Picture) -> Result<(usize, usize), String> {
    if (a.width, a.height) != (b.width, b.height) {
        return Err(format!("the pictures are {}x{} and {}x{}", a.width, a.height, b.width, b.height));
    }
    let differing = a.rgba.chunks(4).zip(b.rgba.chunks(4)).filter(|(p, q)| p.iter().zip(q.iter()).any(|(x, y)| x.abs_diff(*y) > CHANNEL_TOLERANCE)).count();
    Ok((differing, a.width * a.height))
}

/// HAS ANYTHING BEEN DRAWN OTHERWISE between two pictures of one window: more than a few pixels past the tolerance of a
/// channel. For what a person sees change on a canvas - a radius of 2 against one of 4 moves a few hundred pixels of a
/// window, under the share a signed picture is allowed; a caret blinking moves a dozen.
pub fn changed(a: &Picture, b: &Picture) -> bool {
    difference(a, b).map_or(true, |(differing, _)| differing > 40)
}

/// Has anything been drawn otherwise INSIDE `area` of the window (in points, a pixel to a point), as [`changed`]
/// judges it, leaving out what lies in `except`: the canvas alone, without the status line and the bars that change
/// with every click, and without the fields a click takes the keyboard from.
pub fn changed_in(a: &Picture, b: &Picture, area: qymcad::Rect, except: &[qymcad::Rect]) -> bool {
    if (a.width, a.height) != (b.width, b.height) {
        return true;
    }
    let (x0, y0) = (area.min.x.max(0.0) as usize, area.min.y.max(0.0) as usize);
    let (x1, y1) = ((area.max.x as usize).min(a.width), (area.max.y as usize).min(a.height));
    let differing = (y0..y1)
        .flat_map(|y| (x0..x1).map(move |x| (x, y)))
        .filter(|(x, y)| !except.iter().any(|r| r.expand(2.0).contains(qymcad::pos2(*x as f32, *y as f32))))
        .map(|(x, y)| (y * a.width + x) * 4)
        .filter(|&i| a.rgba[i..i + 4].iter().zip(&b.rgba[i..i + 4]).any(|(p, q)| p.abs_diff(*q) > CHANNEL_TOLERANCE))
        .count();
    differing > 40
}

/// Do two pictures show the same thing, within the tolerance?
pub fn same(a: &Picture, b: &Picture) -> bool {
    matches!(difference(a, b), Ok((differing, total)) if (differing as f64) <= PIXEL_SHARE * total as f64)
}

/// A picture of where two pictures differ: the signed one dimmed, the differing pixels red.
fn marked(signed: &Picture, got: &Picture) -> Picture {
    let rgba = signed
        .rgba
        .chunks(4)
        .zip(got.rgba.chunks(4))
        .flat_map(|(p, q)| if p.iter().zip(q.iter()).any(|(x, y)| x.abs_diff(*y) > CHANNEL_TOLERANCE) { [255, 0, 0, 255] } else { [p[0] / 3, p[1] / 3, p[2] / 3, 255] })
        .collect();
    Picture { width: signed.width, height: signed.height, rgba }
}

/// THE PICTURE `got` IS THE ONE SIGNED AS `name`.
///
/// # Panics
/// When no picture is signed under `name`, or the signed one differs past the tolerance. The candidate, and where
/// they differ, are left in `target/golden-candidates/` first.
pub fn looks_as_signed(name: &str, got: &Picture) {
    let asked = std::env::var("QYMCAD_GOLDEN_WRITE").unwrap_or_default();
    let sign = asked.split(',').any(|n| n.trim() == name);
    check(&signed_dir(), &candidates_dir(), name, got, sign);
}

/// The picture `got` against the one signed as `name` in `signed`, candidates left in `candidates`; `sign` writes it
/// as the signed one instead.
fn check(signed: &std::path::Path, candidates: &std::path::Path, name: &str, got: &Picture, sign: bool) {
    let signed_path = signed.join(format!("{name}.png"));
    if sign {
        std::fs::create_dir_all(signed).expect("the folder of signed pictures is made");
        std::fs::write(&signed_path, got.png()).expect("the signed picture writes");
        return;
    }
    std::fs::create_dir_all(candidates).expect("the folder of candidates is made");
    let candidate = candidates.join(format!("{name}.png"));
    let Some(want) = std::fs::read(&signed_path).ok().and_then(|b| Picture::from_png(&b)) else {
        std::fs::write(&candidate, got.png()).expect("the candidate writes");
        panic!("no picture is signed as {name:?}: the candidate is {}; a person who has looked at it signs it with QYMCAD_GOLDEN_WRITE={name}", candidate.display());
    };
    match difference(&want, got) {
        Ok((differing, total)) if (differing as f64) <= PIXEL_SHARE * total as f64 => {}
        outcome => {
            std::fs::write(&candidate, got.png()).expect("the candidate writes");
            let marks = candidates.join(format!("{name}.diff.png"));
            if outcome.is_ok() {
                std::fs::write(&marks, marked(&want, got).png()).expect("the marks write");
            }
            panic!(
                "the window does not look as signed as {name:?}: {outcome:?} pixels differ past {CHANNEL_TOLERANCE} steps, {} allowed; the candidate is {}, the differences {}",
                (PIXEL_SHARE * (want.width * want.height) as f64) as usize,
                candidate.display(),
                marks.display()
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use qymcad::Picture;

    /// A flat grey picture of `w` x `h`.
    fn grey(w: usize, h: usize, v: u8) -> Picture {
        Picture { width: w, height: h, rgba: [v, v, v, 255].repeat(w * h) }
    }

    /// THE SAME PICTURE IS THE SAME, and so is one moved within the tolerance in every pixel.
    #[test]
    fn a_picture_within_the_tolerance_is_the_same() {
        let a = grey(100, 100, 100);
        assert!(super::same(&a, &a.clone()), "a picture is not the same as itself");
        assert!(super::same(&a, &grey(100, 100, 100 + super::CHANNEL_TOLERANCE)), "a picture moved within the tolerance in every pixel differs");
    }

    /// A PATCH PAST THE TOLERANCE DIFFERS once it covers more than the allowed share, and not before.
    #[test]
    fn a_patch_past_the_share_differs() {
        let a = grey(100, 100, 100);
        let patch = |n: usize| {
            let mut b = a.clone();
            for px in b.rgba.chunks_mut(4).take(n) {
                px[0] = 255;
            }
            b
        };
        let allowed = (super::PIXEL_SHARE * 10_000.0) as usize;
        assert!(super::same(&a, &patch(allowed)), "{allowed} pixels changed of 10000 already differ");
        assert!(!super::same(&a, &patch(allowed + 1)), "{} pixels changed of 10000 do not differ", allowed + 1);
    }

    /// PICTURES OF DIFFERENT SIZES DIFFER, whatever they hold.
    #[test]
    fn pictures_of_different_sizes_differ() {
        assert!(super::difference(&grey(10, 10, 0), &grey(10, 11, 0)).is_err(), "pictures of different sizes compared as one");
    }

    /// A PICTURE SURVIVES BEING WRITTEN AND READ AS A PNG, byte for byte.
    #[test]
    fn a_picture_survives_its_png() {
        let mut a = grey(7, 5, 30);
        a.rgba[4 * 12 + 1] = 200;
        assert_eq!(Picture::from_png(&a.png()), Some(a), "a picture read back from its PNG is another picture");
    }

    /// SIGNED, THE SAME PICTURE PASSES; UNSIGNED OR CHANGED, IT FAILS and leaves the candidate and its differences.
    #[test]
    fn a_picture_passes_only_as_signed() {
        let root = std::env::temp_dir().join(format!("qymcad-golden-check-{}", std::process::id()));
        let (signed, candidates) = (root.join("signed"), root.join("candidates"));
        let a = grey(40, 30, 90);
        let unsigned = crate::refusal(|| super::check(&signed, &candidates, "grey", &a, false));
        assert!(unsigned.contains("no picture is signed") && candidates.join("grey.png").exists(), "an unsigned picture: {unsigned:?}");
        super::check(&signed, &candidates, "grey", &a, true);
        assert!(crate::refusal(|| super::check(&signed, &candidates, "grey", &a, false)).is_empty(), "the signed picture itself did not pass");
        let mut b = a.clone();
        for px in b.rgba.chunks_mut(4).take(40) {
            px[1] = 250;
        }
        let changed = crate::refusal(|| super::check(&signed, &candidates, "grey", &b, false));
        assert!(changed.contains("does not look as signed") && candidates.join("grey.diff.png").exists(), "a picture with a row changed: {changed:?}");
        let _ = std::fs::remove_dir_all(&root);
    }
}
