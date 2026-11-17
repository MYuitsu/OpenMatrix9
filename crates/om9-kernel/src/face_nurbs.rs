// SPDX-License-Identifier: LGPL-2.1-or-later
//! Exact NURBS of BRep faces, read-only: each face's underlying surface (a bounded one whole,
//! an unbounded one cut to the face's parameter box) and its trim loops as 3D NURBS curves
//! (outer loop first, then holes; edges in order and direction round the loop; none for a face
//! that is its whole surface). Used to give
//! add-ons the exact trimmed surfaces they work on (ReTrim, Untrim, edges as input …).
//! [`edge_nurbs`] gives every edge on its own, in the order of the display's edge keys, for
//! picked edges.
use crate::KernelError;
use crate::brep::{BytesContext, byte_bound, native_message, receive_bytes};
use om9_core::{brep::BrepSourceAccess, budget::Limits};
use std::ffi::{c_char, c_int, c_void};

unsafe extern "C" {
    fn om9_occt_face_nurbs(
        source: *const u8,
        length: usize,
        max_bytes: usize,
        sink: unsafe extern "C" fn(*mut c_void, *const u8, usize) -> c_int,
        context: *mut c_void,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_edge_nurbs(
        source: *const u8,
        length: usize,
        max_bytes: usize,
        sink: unsafe extern "C" fn(*mut c_void, *const u8, usize) -> c_int,
        context: *mut c_void,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
}

/// A NURBS curve: clamped knots (`poles + degree + 1` values), poles and weights.
#[derive(Debug, Clone, PartialEq)]
pub struct NurbsCurveData {
    pub degree: usize,
    pub knots: Vec<f64>,
    pub poles: Vec<[f64; 3]>,
    pub weights: Vec<f64>,
}

/// One face: its surface (poles row by row, u outer, v inner) and its loops.
#[derive(Debug, Clone, PartialEq)]
pub struct FaceNurbs {
    pub degree: [usize; 2],
    pub size: [usize; 2],
    pub knots: [Vec<f64>; 2],
    pub poles: Vec<[f64; 3]>,
    pub weights: Vec<f64>,
    /// The face's normal is the surface's reversed.
    pub reversed: bool,
    /// Outer loop first, then the holes; they run round the surface's own normal (outer
    /// counter-clockwise, holes clockwise in its parameters), whatever `reversed` says. Empty:
    /// the face is its whole surface (bounded by the surface's own sides, seam and poles).
    pub loops: Vec<Vec<NurbsCurveData>>,
}

/// The faces of `brep` as exact NURBS (see the module notes).
pub fn face_nurbs(brep: &(impl BrepSourceAccess + ?Sized), limits: &Limits) -> Result<Vec<FaceNurbs>, KernelError> {
    brep.validate(limits).map_err(|e| KernelError::InvalidInput(e.to_string()))?;
    let limit = byte_bound(limits);
    let mut context = BytesContext { bytes: Vec::new(), limit };
    let mut message = [0 as c_char; 512];
    let source = brep.source();
    // SAFETY: the source, context and message buffer stay alive through the synchronous call.
    let status = unsafe {
        om9_occt_face_nurbs(source.as_ptr(), source.len(), limit, receive_bytes,
            (&mut context as *mut BytesContext).cast(), message.as_mut_ptr(), 512)
    };
    if status != 0 {
        return Err(KernelError::InvalidInput(native_message(&message)));
    }
    let text = String::from_utf8(context.bytes).map_err(|_| KernelError::InvalidInput("face NURBS encoding".into()))?;
    parse(&text, limits).map_err(|e| KernelError::InvalidInput(format!("face NURBS: {e}")))
}

/// Every edge of `brep` as an exact NURBS curve in its own direction, in the shape's edge order
/// — the order of `brep.display.edges`, so a picked edge key finds its curve. `None` for a
/// degenerate edge (a pole) or one without a 3D curve.
pub fn edge_nurbs(brep: &(impl BrepSourceAccess + ?Sized), limits: &Limits) -> Result<Vec<Option<NurbsCurveData>>, KernelError> {
    brep.validate(limits).map_err(|e| KernelError::InvalidInput(e.to_string()))?;
    let limit = byte_bound(limits);
    let mut context = BytesContext { bytes: Vec::new(), limit };
    let mut message = [0 as c_char; 512];
    let source = brep.source();
    // SAFETY: the source, context and message buffer stay alive through the synchronous call.
    let status = unsafe {
        om9_occt_edge_nurbs(source.as_ptr(), source.len(), limit, receive_bytes,
            (&mut context as *mut BytesContext).cast(), message.as_mut_ptr(), 512)
    };
    if status != 0 {
        return Err(KernelError::InvalidInput(native_message(&message)));
    }
    let text = String::from_utf8(context.bytes).map_err(|_| KernelError::InvalidInput("edge NURBS encoding".into()))?;
    parse_edges(&text, limits).map_err(|e| KernelError::InvalidInput(format!("edge NURBS: {e}")))
}

/// Read the edge bridge's text: `E n`, then per edge a curve (`C degree poles`, knots, poles)
/// or `D` (none).
#[doc(hidden)]
pub fn parse_edges(text: &str, limits: &Limits) -> Result<Vec<Option<NurbsCurveData>>, &'static str> {
    let mut words = text.split_ascii_whitespace();
    let max = usize::try_from(limits.max_points).unwrap_or(usize::MAX).max(1);
    if words.next() != Some("E") {
        return Err("unexpected tag");
    }
    let n: usize = words.next().ok_or("truncated")?.parse().map_err(|_| "bad count")?;
    if n > max {
        return Err("count over the budget");
    }
    let mut budget = 0usize;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        match words.next().ok_or("truncated")? {
            "D" => out.push(None),
            "C" => out.push(Some(curve(&mut words, max, &mut budget)?)),
            _ => return Err("unexpected tag"),
        }
    }
    if words.next().is_some() {
        return Err("trailing data");
    }
    Ok(out)
}

/// One curve after its `C` tag: degree, pole count, knots, poles (x y z w).
fn curve<'a>(w: &mut impl Iterator<Item = &'a str>, max: usize, budget: &mut usize) -> Result<NurbsCurveData, &'static str> {
    fn count<'a>(w: &mut impl Iterator<Item = &'a str>, m: usize) -> Result<usize, &'static str> {
        let n: usize = w.next().ok_or("truncated")?.parse().map_err(|_| "bad count")?;
        if n > m { Err("count over the budget") } else { Ok(n) }
    }
    fn real<'a>(w: &mut impl Iterator<Item = &'a str>) -> Result<f64, &'static str> {
        let v: f64 = w.next().ok_or("truncated")?.parse().map_err(|_| "bad number")?;
        if v.is_finite() { Ok(v) } else { Err("not finite") }
    }
    let degree = count(&mut *w, 64)?;
    let n = count(&mut *w, max)?;
    if degree == 0 || n <= degree {
        return Err("bad curve size");
    }
    *budget = budget.saturating_add(n);
    if *budget > max {
        return Err("poles over the budget");
    }
    let knots: Vec<f64> = (0..n + degree + 1).map(|_| real(&mut *w)).collect::<Result<_, _>>()?;
    let mut poles = Vec::with_capacity(n);
    let mut weights = Vec::with_capacity(n);
    for _ in 0..n {
        poles.push([real(&mut *w)?, real(&mut *w)?, real(&mut *w)?]);
        let wt = real(&mut *w)?;
        if wt <= 0.0 {
            return Err("weight not positive");
        }
        weights.push(wt);
    }
    Ok(NurbsCurveData { degree, knots, poles, weights })
}

/// Read the bridge's text: `F n`, then per face `S du dv nu nv reversed`, the u and v knots,
/// the poles (x y z w …), `W loops`, and per loop `L curves` with each curve as `C degree
/// poles`, its knots and its poles.
#[doc(hidden)]
pub fn parse(text: &str, limits: &Limits) -> Result<Vec<FaceNurbs>, &'static str> {
    let mut words = text.split_ascii_whitespace();
    fn next<'a>(w: &mut impl Iterator<Item = &'a str>) -> Result<&'a str, &'static str> {
        w.next().ok_or("truncated")
    }
    fn count<'a>(w: &mut impl Iterator<Item = &'a str>, max: usize) -> Result<usize, &'static str> {
        let n: usize = next(w)?.parse().map_err(|_| "bad count")?;
        if n > max { Err("count over the budget") } else { Ok(n) }
    }
    fn real<'a>(w: &mut impl Iterator<Item = &'a str>) -> Result<f64, &'static str> {
        let v: f64 = next(w)?.parse().map_err(|_| "bad number")?;
        if v.is_finite() { Ok(v) } else { Err("not finite") }
    }
    fn expect<'a>(w: &mut impl Iterator<Item = &'a str>, tag: &str) -> Result<(), &'static str> {
        if next(w)? == tag { Ok(()) } else { Err("unexpected tag") }
    }
    let max = usize::try_from(limits.max_points).unwrap_or(usize::MAX).max(1);
    let poles_of = |w: &mut std::str::SplitAsciiWhitespace<'_>, n: usize| -> Result<(Vec<[f64; 3]>, Vec<f64>), &'static str> {
        let mut poles = Vec::with_capacity(n);
        let mut weights = Vec::with_capacity(n);
        for _ in 0..n {
            poles.push([real(w)?, real(w)?, real(w)?]);
            let wt = real(w)?;
            if wt <= 0.0 {
                return Err("weight not positive");
            }
            weights.push(wt);
        }
        Ok((poles, weights))
    };
    expect(&mut words, "F")?;
    let faces = count(&mut words, max)?;
    let mut out = Vec::with_capacity(faces);
    let mut budget = 0usize;
    for _ in 0..faces {
        expect(&mut words, "S")?;
        let (du, dv) = (count(&mut words, 64)?, count(&mut words, 64)?);
        let (nu, nv) = (count(&mut words, max)?, count(&mut words, max)?);
        let reversed = next(&mut words)? == "1";
        if du == 0 || dv == 0 || nu <= du || nv <= dv {
            return Err("bad surface size");
        }
        budget = budget.saturating_add(nu.saturating_mul(nv));
        if budget > max {
            return Err("poles over the budget");
        }
        let ku: Vec<f64> = (0..nu + du + 1).map(|_| real(&mut words)).collect::<Result<_, _>>()?;
        let kv: Vec<f64> = (0..nv + dv + 1).map(|_| real(&mut words)).collect::<Result<_, _>>()?;
        let (poles, weights) = poles_of(&mut words, nu * nv)?;
        expect(&mut words, "W")?;
        let wires = count(&mut words, max)?;
        let mut loops = Vec::with_capacity(wires);
        for _ in 0..wires {
            expect(&mut words, "L")?;
            let curves = count(&mut words, max)?;
            let mut lp = Vec::with_capacity(curves);
            for _ in 0..curves {
                expect(&mut words, "C")?;
                lp.push(curve(&mut words, max, &mut budget)?);
            }
            loops.push(lp);
        }
        out.push(FaceNurbs { degree: [du, dv], size: [nu, nv], knots: [ku, kv], poles, weights, reversed, loops });
    }
    if words.next().is_some() {
        return Err("trailing data");
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_bridge_text_and_refuses_bad_input() {
        let limits = Limits::default();
        // One bilinear face with one square loop of four lines.
        let line = |a: [f64; 3], b: [f64; 3]| format!("C 1 2\n0 0 1 1\n{} {} {} 1 {} {} {} 1\n", a[0], a[1], a[2], b[0], b[1], b[2]);
        let text = format!(
            "F 1\nS 1 1 2 2 0\n0 0 1 1\n0 0 1 1\n0 0 0 1 0 1 0 1 1 0 0 1 1 1 0 1\nW 1\nL 4\n{}{}{}{}",
            line([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]),
            line([1.0, 0.0, 0.0], [1.0, 1.0, 0.0]),
            line([1.0, 1.0, 0.0], [0.0, 1.0, 0.0]),
            line([0.0, 1.0, 0.0], [0.0, 0.0, 0.0]),
        );
        let faces = parse(&text, &limits).unwrap();
        assert_eq!(faces.len(), 1);
        assert_eq!(faces[0].size, [2, 2]);
        assert_eq!(faces[0].loops[0].len(), 4);
        assert_eq!(faces[0].loops[0][1].poles, vec![[1.0, 0.0, 0.0], [1.0, 1.0, 0.0]]);
        // Truncated, trailing, bad weights, non-finite numbers: refused.
        assert!(parse(&text[..text.len() - 6], &limits).is_err());
        assert!(parse(&format!("{text} 1"), &limits).is_err());
        assert!(parse(&text.replacen("0 0 0 1 0 1 0 1", "0 0 0 0 0 1 0 1", 1), &limits).is_err());
        assert!(parse(&text.replacen("0 0 0 1 0 1 0 1", "0 0 nan 1 0 1 0 1", 1), &limits).is_err());
        // Counts over the point budget are refused before allocating.
        let tight = Limits { max_points: 3, ..Limits::default() };
        assert!(parse(&text, &tight).is_err());
    }

    #[test]
    fn parses_edges_with_degenerate_ones_and_refuses_bad_input() {
        let limits = Limits::default();
        let text = "E 2\nD\nC 1 2\n0 0 1 1\n0 0 0 1 1 0 0 1\n";
        let edges = parse_edges(text, &limits).unwrap();
        assert_eq!(edges.len(), 2);
        assert!(edges[0].is_none());
        assert_eq!(edges[1].as_ref().unwrap().poles, vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        assert!(parse_edges("E 3\nD\nD\n", &limits).is_err(), "truncated");
        assert!(parse_edges(&format!("{text} D"), &limits).is_err(), "trailing");
        assert!(parse_edges("E 1\nX\n", &limits).is_err());
        assert!(parse_edges(&text.replace("0 0 0 1 1", "0 0 0 -1 1"), &limits).is_err(), "weight");
        assert!(parse_edges("E 99999999999\n", &Limits { max_points: 10, ..Limits::default() }).is_err());
    }
}
