// SPDX-License-Identifier: LGPL-2.1-or-later
//! Compare actual cold-native geometry against the immutable current snapshot.
//! No echoed input packet, user data or cached display mesh proves conversion.
use om9_core::{
    Error, Result,
    budget::{Limits, Usage},
    geometry::{Curve, Geometry, NurbsCurve, Point3, Transform},
    identity::{DefinitionId, LayerId, ObjectId},
    model::{Document, Entity},
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
pub(crate) type ExpectedBreps = BTreeMap<ObjectId, (u32, usize)>;
const DEFAULT_LAYER: &str = "ffffffff-ffff-4fff-8fff-ffffffffffff";
pub(crate) fn expected_breps(document: &Document, bytes: &[u8], limits: &Limits) -> Result<ExpectedBreps> {
    let mut usage = om9_archive::admit_json_usage(bytes, limits)?;
    let entities = || document.entities.values().chain(document.definitions.values().flat_map(|d| &d.members));
    let count = entities().filter(|e| matches!(e.geometry, Geometry::NativeBrep(_))).count();
    usage.add(Usage::btree::<ObjectId, u32>(count)?, limits)?;
    usage.add(Usage::btree::<ObjectId, (u32, usize)>(count)?, limits)?;
    let root: Value = serde_json::from_slice(bytes).map_err(|_| invalid())?;
    fields(&root, &["schema_version", "status", "objects"])?;
    if root["schema_version"] != 1 || root["status"] != "expected_brep_fingerprints" {
        return Err(invalid());
    }
    let ids: BTreeMap<_, _> = entities().filter_map(|e| match &e.geometry {
        Geometry::NativeBrep(b) => Some((e.id, b.crc)),
        _ => None,
    }).collect();
    let objects = array(&root["objects"])?;
    if objects.len() != ids.len() { return Err(invalid()); }
    let mut result = ExpectedBreps::new();
    for value in objects {
        fields(value, &["id", "source_crc", "crc", "payload_bytes"])?;
        let id = ObjectId(id(&value["id"])?);
        let crc = value["crc"].as_u64().and_then(|x| u32::try_from(x).ok()).ok_or_else(invalid)?;
        let size = value["payload_bytes"].as_u64().filter(|n| *n > 0 && *n <= limits.max_bytes)
            .and_then(|n| usize::try_from(n).ok()).ok_or_else(invalid)?;
        if !ids.contains_key(&id) || value["source_crc"].as_u64() != ids.get(&id).map(|n| u64::from(*n))
            || result.insert(id, (crc, size)).is_some() {
            return Err(invalid());
        }
    }
    Ok(result)
}
fn invalid() -> Error {
    Error::Invalid("native geometry correspondence")
}
fn verify_model_tolerance(document: &Document, root: &Value) -> Result<()> {
    if let Some(actual) = root.get("model_tolerance") {
        fields(actual, &["absolute_mm", "relative", "angular_rad"])?;
        let absolute = actual["absolute_mm"].as_f64().filter(|v| v.is_finite() && *v > 0.).ok_or_else(invalid)?;
        let relative = actual["relative"].as_f64().filter(|v| v.is_finite() && *v > 0. && *v < 1.).ok_or_else(invalid)?;
        let angular = actual["angular_rad"].as_f64().filter(|v| v.is_finite() && *v > 0. && *v <= std::f64::consts::PI).ok_or_else(invalid)?;
        let _ = (absolute, relative, angular);
    }
    if let Some(tolerance) = document.tolerance() {
        let actual = root.get("model_tolerance").ok_or_else(invalid)?;
        fields(actual, &["absolute_mm", "relative", "angular_rad"])?;
        for (name, expected) in [("absolute_mm", tolerance.absolute_mm), ("relative", tolerance.relative), ("angular_rad", tolerance.angular_rad)] {
            if actual[name].as_f64() != Some(expected) { return Err(invalid()); }
        }
    }
    Ok(())
}
fn array(v: &Value) -> Result<&[Value]> {
    v.as_array().map(Vec::as_slice).ok_or_else(invalid)
}
fn text(v: &Value) -> Result<&str> {
    v.as_str().ok_or_else(invalid)
}
fn id(v: &Value) -> Result<uuid::Uuid> {
    uuid::Uuid::parse_str(text(v)?).map_err(|_| invalid())
}
fn fields(v: &Value, expected: &[&str]) -> Result<()> {
    let map = v.as_object().ok_or_else(invalid)?;
    if map.len() != expected.len() || expected.iter().any(|k| !map.contains_key(*k)) {
        return Err(invalid());
    }
    Ok(())
}
fn number(v: &Value, want: f64, tolerance: f64) -> Result<()> {
    let actual = v.as_f64().filter(|n| n.is_finite()).ok_or_else(invalid)?;
    if !want.is_finite() || (actual - want).abs() > tolerance * (1. + want.abs()) {
        return Err(invalid());
    }
    Ok(())
}
fn point(v: &Value, want: Point3, tolerance: f64) -> Result<()> {
    let a = array(v)?;
    if a.len() != 3 {
        return Err(invalid());
    }
    for i in 0..3 {
        number(&a[i], want[i], tolerance)?;
    }
    Ok(())
}
fn numbers(v: &Value, want: &[f64]) -> Result<()> {
    let a = array(v)?;
    if a.len() != want.len() {
        return Err(invalid());
    }
    for (v, n) in a.iter().zip(want) {
        number(v, *n, 1e-10)?;
    }
    Ok(())
}
fn points(v: &Value, want: &[Point3], t: Transform, normal: bool) -> Result<()> {
    let a = array(v)?;
    if a.len() != want.len() {
        return Err(invalid());
    }
    for (v, p) in a.iter().zip(want) {
        point(
            v,
            if normal {
                transform_normal(t, *p)?
            } else {
                t.point(*p)?
            },
            if normal { 2e-6 } else { 1e-10 },
        )?;
    }
    Ok(())
}
// A fixed absolute tolerance can exceed an entire small curve. Scale the
// geometric allowance by its extent and separately admit transform roundoff.
fn curve_extent(points: &[Point3], t: Transform) -> Result<Point3> {
    let first = t.point(*points.first().ok_or_else(invalid)?)?;
    let (mut lo, mut hi) = (first, first);
    for p in &points[1..] {
        let p = t.point(*p)?;
        for i in 0..3 { lo[i] = lo[i].min(p[i]); hi[i] = hi[i].max(p[i]); }
    }
    Ok(std::array::from_fn(|i| hi[i] - lo[i]))
}
fn curve_point(v: &Value, p: Point3, t: Transform, extent: Point3) -> Result<()> {
    let actual = array(v)?;
    if actual.len() != 3 { return Err(invalid()); }
    let expected = t.point(p)?;
    for i in 0..3 {
        let value = actual[i].as_f64().filter(|n| n.is_finite()).ok_or_else(invalid)?;
        let roundoff = t.0[i][3].abs() + (0..3).map(|j| (t.0[i][j] * p[j]).abs()).sum::<f64>();
        let tolerance = 1e-10 * extent[i] + 32. * f64::EPSILON * roundoff;
        if !tolerance.is_finite() || (value - expected[i]).abs() > tolerance { return Err(invalid()); }
    }
    Ok(())
}
fn vector(t: Transform, p: Point3) -> Point3 {
    std::array::from_fn(|i| (0..3).map(|j| t.0[i][j] * p[j]).sum())
}
fn length(p: Point3) -> f64 {
    p.iter().map(|x| x * x).sum::<f64>().sqrt()
}
fn transform_normal(t: Transform, p: Point3) -> Result<Point3> {
    let m = t.0;
    let c = [
        [
            m[1][1] * m[2][2] - m[1][2] * m[2][1],
            m[1][2] * m[2][0] - m[1][0] * m[2][2],
            m[1][0] * m[2][1] - m[1][1] * m[2][0],
        ],
        [
            m[0][2] * m[2][1] - m[0][1] * m[2][2],
            m[0][0] * m[2][2] - m[0][2] * m[2][0],
            m[0][1] * m[2][0] - m[0][0] * m[2][1],
        ],
        [
            m[0][1] * m[1][2] - m[0][2] * m[1][1],
            m[0][2] * m[1][0] - m[0][0] * m[1][2],
            m[0][0] * m[1][1] - m[0][1] * m[1][0],
        ],
    ];
    let det = (0..3).map(|j| m[0][j] * c[0][j]).sum::<f64>();
    let q: Point3 = std::array::from_fn(|i| (0..3).map(|j| c[i][j] * p[j]).sum::<f64>() / det);
    let l = length(q);
    if !l.is_finite() || l == 0. {
        return Err(Error::Unsupported("zero native normal"));
    }
    Ok(q.map(|x| x / l))
}
fn matrix(v: &Value, t: Transform) -> Result<()> {
    let a = array(v)?;
    if a.len() != 4 {
        return Err(invalid());
    }
    for (row, n) in a.iter().zip(t.0) {
        numbers(row, &n)?;
    }
    Ok(())
}
fn nurbs(v: &Value, n: &NurbsCurve, t: Transform) -> Result<()> {
    fields(
        v,
        &[
            "degree", "poles", "weights", "knots", "domain", "periodic", "reversed", "seam",
        ],
    )?;
    if v["degree"].as_u64() != Some(n.degree as u64)
        || v["periodic"] != n.periodic
        || v["reversed"] != n.reversed
    {
        return Err(invalid());
    }
    match n.seam {
        Some(s) => number(&v["seam"], s, 1e-10)?,
        None => {
            if !v["seam"].is_null() {
                return Err(invalid());
            }
        }
    }
    let poles = array(&v["poles"])?;
    if poles.len() != n.poles.len() { return Err(invalid()); }
    let extent = curve_extent(&n.poles, t)?;
    for (actual, expected) in poles.iter().zip(&n.poles) { curve_point(actual, *expected, t, extent)?; }
    numbers(&v["weights"], &n.weights)?;
    numbers(&v["knots"], &n.knots)?;
    numbers(&v["domain"], &n.domain)
}
fn curve(v: &Value, c: &Curve, t: Transform) -> Result<()> {
    match c {
        Curve::Line { start, end, domain } => {
            fields(v, &["Line"])?;
            let v = &v["Line"];
            fields(v, &["start", "end", "domain"])?;
            let extent = curve_extent(&[*start, *end], t)?;
            curve_point(&v["start"], *start, t, extent)?;
            curve_point(&v["end"], *end, t, extent)?;
            numbers(&v["domain"], domain)
        }
        Curve::Circle {
            center,
            x,
            y,
            radius,
            domain,
            angle_domain,
        } => {
            fields(v, &["Circle"])?;
            let v = &v["Circle"];
            if v.get("angle_domain").is_some() {
                fields(v, &["center", "x", "y", "radius", "domain", "angle_domain"])?;
            } else {
                fields(v, &["center", "x", "y", "radius", "domain"])?;
            }
            let a = vector(t, *x);
            let b = vector(t, *y);
            let la = length(a);
            let lb = length(b);
            if !la.is_finite() || la == 0. || (la - lb).abs() > 1e-10 * (1. + la) {
                return Err(Error::Unsupported("nonuniform native circle placement"));
            }
            point(&v["center"], t.point(*center)?, 1e-10)?;
            point(&v["x"], a.map(|n| n / la), 1e-10)?;
            point(&v["y"], b.map(|n| n / lb), 1e-10)?;
            number(&v["radius"], radius * la, 1e-10)?;
            numbers(&v["domain"], domain)?;
            let angles = angle_domain.unwrap_or(*domain);
            if let Some(native_angles) = v.get("angle_domain") {
                numbers(native_angles, &angles)
            } else {
                numbers(&v["domain"], &angles)
            }
        }
        Curve::Nurbs(n) => {
            fields(v, &["Nurbs"])?;
            nurbs(&v["Nurbs"], n, t)
        }
        Curve::Polycurve { segments, reversed } => {
            fields(v, &["Polycurve"])?;
            let v = &v["Polycurve"];
            fields(v, &["segments", "reversed"])?;
            if v["reversed"] != *reversed {
                return Err(invalid());
            }
            let a = array(&v["segments"])?;
            if a.len() != segments.len() {
                return Err(invalid());
            }
            for (v, c) in a.iter().zip(segments) {
                curve(v, c, t)?;
            }
            Ok(())
        }
    }
}
fn colors(v: &Value, want: &[[u8; 4]]) -> Result<()> {
    let a = array(v)?;
    if a.len() != want.len() {
        return Err(invalid());
    }
    for (v, c) in a.iter().zip(want) {
        color(v, *c)?;
    }
    Ok(())
}
fn color(v: &Value, want: [u8; 4]) -> Result<()> {
    let a = array(v)?;
    if a.len() != 4
        || a.iter()
            .zip(want)
            .any(|(v, n)| v.as_u64() != Some(n as u64))
    {
        return Err(invalid());
    }
    Ok(())
}
pub(crate) fn geometry(v: &Value, g: &Geometry, t: Transform) -> Result<()> {
    geometry_with_brep(v,g,t,None)
}
fn geometry_with_brep(v:&Value,g:&Geometry,t:Transform,expected:Option<(u32,usize)>)->Result<()> {
    match g {
        Geometry::Point(p) => {
            fields(v, &["Point"])?;
            point(&v["Point"], t.point(*p)?, 1e-10)
        }
        Geometry::Curve(c) => {
            fields(v, &["Curve"])?;
            curve(&v["Curve"], c, t)
        }
        Geometry::Instance(i) => {
            fields(v, &["Instance"])?;
            if id(&v["Instance"])? != i.0 {
                return Err(invalid());
            }
            Ok(())
        }
        Geometry::Hatch(_) => {
            fields(v, &["Hatch"])?;
            let v = &v["Hatch"];
            fields(v, &["display", "loops", "pattern", "color"])?;
            Ok(())
        }
        Geometry::Light(light) => {
            fields(v, &["Light"])?;
            let v = &v["Light"];
            fields(
                v,
                &[
                    "display",
                    "location",
                    "direction",
                    "diffuse",
                    "intensity",
                    "style",
                ],
            )?;
            point(&v["display"], light.display, 1e-10)?;
            Ok(())
        }
        Geometry::Brep(_) => {
            // Native source validation belongs to om9-core; the JSON contract only requires the leaf key.
            fields(v, &["Brep"])?;
            Ok(())
        }
        Geometry::NativeBrep(brep) => {
            fields(v, &["NativeBrep"])?;
            let v = &v["NativeBrep"];
            fields(v, &["display", "payload", "crc"])?;
            let (crc,payload_bytes)=expected.unwrap_or((brep.crc,brep.payload.len()));
            if expected.is_none() && t != Transform::identity() {return Err(invalid());}
            if v["crc"].as_u64() != Some(crc as u64) {
                return Err(invalid());
            }
            let chunks = array(&v["payload"])?;
            let mut byte_count = 0usize;
            for chunk in chunks {
                for byte in array(chunk)? {
                    byte.as_u64().and_then(|b| u8::try_from(b).ok()).ok_or_else(invalid)?;
                    byte_count = byte_count.checked_add(1).ok_or_else(invalid)?;
                    if byte_count > payload_bytes { return Err(invalid()); }
                }
            }
            // Native re-serialization can change archive framing bytes while the
            // BRep topology/geometry stays exact. The native kind check above
            // (ON_Brep) plus a non-empty payload establishes correspondence; the
            // BRep geometry oracle re-imports and inspects it independently.
            if byte_count == 0 || byte_count != payload_bytes {
                return Err(invalid());
            }
            Ok(())
        }
        Geometry::Surface(s) => {
            fields(v, &["Surface"])?;
            let v = &v["Surface"];
            fields(
                v,
                &[
                    "degree", "size", "poles", "weights", "knots", "domain", "periodic", "reversed",
                ],
            )?;
            for (field, want) in [("degree", s.degree), ("size", s.size)] {
                let a = array(&v[field])?;
                if a.len() != 2
                    || a.iter()
                        .zip(want)
                        .any(|(v, n)| v.as_u64() != Some(n as u64))
                {
                    return Err(invalid());
                }
            }
            points(&v["poles"], &s.poles, t, false)?;
            numbers(&v["weights"], &s.weights)?;
            for (field, want) in [
                ("knots", [s.knots[0].as_slice(), s.knots[1].as_slice()]),
                ("domain", [s.domain[0].as_slice(), s.domain[1].as_slice()]),
            ] {
                let a = array(&v[field])?;
                if a.len() != 2 {
                    return Err(invalid());
                }
                for (v, w) in a.iter().zip(want) {
                    numbers(v, w)?;
                }
            }
            let a = array(&v["periodic"])?;
            if a.len() != 2
                || a.iter().zip(s.periodic).any(|(v, b)| *v != b)
                || v["reversed"] != s.reversed
            {
                return Err(invalid());
            }
            Ok(())
        }
        Geometry::Mesh(m) => {
            fields(v, &["Mesh"])?;
            let v = &v["Mesh"];
            fields(v, &["vertices", "faces", "normals", "colors"])?;
            points(&v["vertices"], &m.vertices, t, false)?;
            points(&v["normals"], &m.normals, t, true)?;
            colors(&v["colors"], &m.colors)?;
            let a = array(&v["faces"])?;
            if a.len() != m.faces.len() {
                return Err(invalid());
            }
            for (v, w) in a.iter().zip(&m.faces) {
                let a = array(v)?;
                if a.len() != w.len() || a.iter().zip(w).any(|(v, n)| v.as_u64() != Some(*n as u64))
                {
                    return Err(invalid());
                }
            }
            Ok(())
        }
        Geometry::Cloud(c) => {
            fields(v, &["Cloud"])?;
            let v = &v["Cloud"];
            fields(v, &["points", "normals", "colors", "hidden"])?;
            points(&v["points"], &c.points, t, false)?;
            points(&v["normals"], &c.normals, t, true)?;
            colors(&v["colors"], &c.colors)?;
            let a = array(&v["hidden"])?;
            // An all-false runtime channel is equivalent to every point visible.
            // Native 3DM omits that channel; report this conversion explicitly.
            if !(a.is_empty() && c.hidden.iter().all(|b| !b))
                && (a.len() != c.hidden.len() || a.iter().zip(&c.hidden).any(|(v, b)| *v != *b))
            {
                return Err(invalid());
            }
            Ok(())
        }
    }
}
fn native_kind(g: &Geometry) -> &'static str {
    match g {
        Geometry::Point(_) => "ON_Point",
        Geometry::Curve(Curve::Line { .. }) => "ON_LineCurve",
        Geometry::Curve(Curve::Circle { .. }) => "ON_ArcCurve",
        Geometry::Curve(Curve::Nurbs(_)) => "ON_NurbsCurve",
        Geometry::Curve(Curve::Polycurve { .. }) => "ON_PolyCurve",
        Geometry::Surface(_) => "ON_NurbsSurface",
        Geometry::Mesh(_) => "ON_Mesh",
        Geometry::Brep(_) | Geometry::NativeBrep(_) => "ON_Brep",
        Geometry::Cloud(_) => "ON_PointCloud",
        Geometry::Instance(_) => "ON_InstanceRef",
        Geometry::Hatch(_) => "ON_Hatch",
        Geometry::Light(_) => "ON_Light",
    }
}
fn entity(v: &Value, e: &Entity, member: bool,expected:&ExpectedBreps) -> Result<()> {
    fields(
        v,
        &[
            "id",
            "label",
            "is_definition_member",
            "geometry",
            "native_kind",
            "placement",
            "attributes",
            "diagnostics",
        ],
    )?;
    for diagnostic in array(&v["diagnostics"])? {
        text(diagnostic)?;
    }
    let native = text(&v["native_kind"])?;
    let short_line = matches!(&e.geometry, Geometry::Curve(Curve::Line { .. }))
        && native == "ON_NurbsCurve"
        && array(&v["diagnostics"])?.iter().any(|d| d.as_str() == Some("short line uses degree-one NURBS and OM9 user data; external edits may remove or invalidate the metadata"));
    if id(&v["id"])? != e.id.0
        || text(&v["label"])? != e.label
        || v["is_definition_member"] != member
        || (native != native_kind(&e.geometry) && !short_line)
    {
        return Err(invalid());
    }
    matrix(
        &v["placement"],
        if matches!(e.geometry, Geometry::Instance(_)) {
            e.placement
        } else {
            Transform::identity()
        },
    )?;
    geometry_with_brep(&v["geometry"], &e.geometry, e.placement,expected.get(&e.id).copied())?;
    let a = &v["attributes"];
    fields(a, &["layer", "visible", "locked", "color", "color_source"])?;
    let expected = e
        .attributes
        .layer
        .map(|i| i.0)
        .unwrap_or(uuid::Uuid::parse_str(DEFAULT_LAYER).unwrap());
    if id(&a["layer"])? != expected
        || a["visible"] != (e.attributes.visible && !e.attributes.hidden)
        || a["locked"] != e.attributes.locked
    {
        return Err(invalid());
    }
    match e.attributes.color {
        Some(c) => {
            color(&a["color"], c)?;
            if a["color_source"] != "object" {
                return Err(invalid());
            }
        }
        None => {
            if !a["color"].is_null() || a["color_source"] != "layer" {
                return Err(invalid());
            }
        }
    }
    Ok(())
}
#[cfg(test)]
fn verify(document:&Document,bytes:&[u8],limits:&Limits)->Result<Vec<String>> {
    verify_with_brep(document,bytes,limits,&ExpectedBreps::new())
}
pub(crate) fn verify_with_brep(document: &Document, bytes: &[u8], limits: &Limits,expected_brep:&ExpectedBreps) -> Result<Vec<String>> {
    let json_usage = om9_archive::admit_json_usage(bytes, limits)?;
    let root: Value = serde_json::from_slice(bytes).map_err(|_| invalid())?;
    let mut expected_fields = vec![
            "schema_version",
            "status",
            "archive_version",
            "model_unit",
            "page_unit",
            "current_layer",
            "objects",
            "layers",
            "definitions",
            "diagnostics",
        ];
    if root.get("model_tolerance").is_some() { expected_fields.push("model_tolerance"); }
    fields(&root, &expected_fields)?;
    if root["schema_version"] != 1
        || root["status"] != "inspected"
        || root["archive_version"] != 50
        || root["model_unit"] != "Millimeter"
        || root["page_unit"] != serde_json::to_value(document.page_unit).map_err(|_| invalid())?
    {
        return Err(invalid());
    }
    verify_model_tolerance(document, &root)?;
    let expected = document.entities.len()
        + document
            .definitions
            .values()
            .map(|d| d.members.len())
            .sum::<usize>();
    let objects = array(&root["objects"])?;
    if objects.len() != expected {
        return Err(invalid());
    }
    let mut seen = BTreeSet::new();
    for v in objects {
        let oid = ObjectId(id(&v["id"])?);
        if !seen.insert(oid) {
            return Err(invalid());
        }
        if let Some(e) = document.entities.get(&oid) {
            entity(v, e, false,expected_brep)?;
        } else {
            let e = document
                .definitions
                .values()
                .flat_map(|d| &d.members)
                .find(|e| e.id == oid)
                .ok_or_else(invalid)?;
            entity(v, e, true,expected_brep)?;
        }
    }
    let defs = array(&root["definitions"])?;
    if defs.len() != document.definitions.len() {
        return Err(invalid());
    }
    let mut seen = BTreeSet::new();
    for v in defs {
        fields(v, &["id", "label", "members", "model_unit", "linked"])?;
        let did = DefinitionId(id(&v["id"])?);
        if !seen.insert(did) {
            return Err(invalid());
        }
        let d = document.definitions.get(&did).ok_or_else(invalid)?;
        if v["label"] != d.label || v["model_unit"] != "Millimeter" || v["linked"] != false {
            return Err(invalid());
        }
        let members = array(&v["members"])?;
        if members.len() != d.members.len() {
            return Err(invalid());
        }
        for (v, e) in members.iter().zip(&d.members) {
            if id(v)? != e.id.0 {
                return Err(invalid());
            }
        }
    }
    let generated = document.layers.is_empty()
        || document
            .entities
            .values()
            .chain(document.definitions.values().flat_map(|d| &d.members))
            .any(|e| e.attributes.layer.is_none());
    let layers = array(&root["layers"])?;
    if layers.len() != document.layers.len() + usize::from(generated) {
        return Err(invalid());
    }
    let mut seen = BTreeSet::new();
    let fallback = uuid::Uuid::parse_str(DEFAULT_LAYER).unwrap();
    for v in layers {
        fields(
            v,
            &[
                "id",
                "name",
                "parent",
                "visible",
                "locked",
                "color",
                "native_index",
            ],
        )?;
        let lid = LayerId(id(&v["id"])?);
        if !seen.insert(lid) {
            return Err(invalid());
        }
        if let Some(l) = document.layers.get(&lid) {
            if v["name"] != l.name || v["visible"] != l.visible || v["locked"] != l.locked {
                return Err(invalid());
            }
            if l.parent.map(|i| i.0)
                != if v["parent"].is_null() {
                    None
                } else {
                    Some(id(&v["parent"])?)
                }
            {
                return Err(invalid());
            }
            color(&v["color"], l.color)?;
            let index = document
                .layer_order
                .iter()
                .position(|i| *i == lid)
                .ok_or_else(invalid)?;
            if v["native_index"].as_u64() != Some(index as u64) {
                return Err(invalid());
            }
        } else {
            if !generated
                || lid.0 != fallback
                || v["name"] != "Default"
                || !v["parent"].is_null()
                || v["visible"] != true
                || v["locked"] != false
                || v["native_index"].as_u64() != Some(document.layers.len() as u64)
            {
                return Err(invalid());
            }
            color(&v["color"], [0, 0, 0, 255])?;
        }
    }
    let current = document.current_layer.map(|i| i.0).unwrap_or_else(|| {
        if generated {
            fallback
        } else {
            document
                .layer_order
                .first()
                .map(|i| i.0)
                .unwrap_or(fallback)
        }
    });
    if id(&root["current_layer"])? != current {
        return Err(invalid());
    }
    collect_diagnostics(document, &root, objects, generated, json_usage, limits)
}
fn visit_diagnostics(
    document: &Document,
    root: &Value,
    objects: &[Value],
    generated: bool,
    mut visit: impl FnMut(Option<ObjectId>, &str) -> Result<()>,
) -> Result<()> {
    for v in array(&root["diagnostics"])? {
        visit(None, text(v)?)?;
    }
    for object in objects {
        for v in array(&object["diagnostics"])? {
            visit(None, text(v)?)?;
        }
    }
    if document.tolerance().is_none() {
        visit(None, "Document tolerance is unspecified; 3DM model settings use openNURBS defaults, not an OM9 accuracy claim.")?;
    }
    if generated {
        visit(
            None,
            "Unassigned objects map to the native Default layer; OM9 layer assignment stays unchanged.",
        )?;
    }
    if document.current_layer.is_none() {
        visit(
            None,
            "Native 3DM requires a current layer; OM9 current_layer=None stays unchanged.",
        )?;
    }
    if document
        .entities
        .values()
        .chain(document.definitions.values().flat_map(|d| &d.members))
        .any(|e| matches!(&e.geometry,Geometry::Cloud(c) if !c.hidden.is_empty()))
    {
        visit(
            None,
            "Native 3DM omits all-false cloud hidden flags; point rows and visibility are preserved.",
        )?;
    }
    for entity in document
        .entities
        .values()
        .chain(document.definitions.values().flat_map(|d| &d.members))
    {
        match &entity.geometry {
            Geometry::Mesh(m) if !m.normals.is_empty() => {
                visit(
                    Some(entity.id),
                    "mesh normals use normalized inverse-transpose; source normal lengths are not retained.",
                )?;
                visit(
                    Some(entity.id),
                    "native mesh normal components use single precision.",
                )?;
            }
            Geometry::Cloud(c) if !c.normals.is_empty() => visit(
                Some(entity.id),
                "cloud normals use normalized inverse-transpose; source normal lengths are not retained.",
            )?,
            _ => {}
        }
    }
    if document
        .entities
        .values()
        .chain(document.definitions.values().flat_map(|d| &d.members))
        .any(|e| e.attributes.hidden)
    {
        visit(
            None,
            "Geometry-only native output folds OM9 Hidden flags into native visibility; the independent OM9 distinction is not encoded.",
        )?;
    }
    if document
        .layers
        .values()
        .any(|layer| layer.preset_slot.is_some())
    {
        visit(
            None,
            "Geometry-only native output does not encode OM9 preset slot identities.",
        )?;
    }
    visit(
        None,
        "Geometry-only current-state export excludes OM9 History, results, embedded resources/retained payloads, runtime identity, Undo/Redo and document metadata.",
    )?;
    Ok(())
}
fn collect_diagnostics(
    document: &Document,
    root: &Value,
    objects: &[Value],
    generated: bool,
    mut held: Usage,
    limits: &Limits,
) -> Result<Vec<String>> {
    let mut count = 0usize;
    visit_diagnostics(document, root, objects, generated, |id, message| {
        count = count
            .checked_add(1)
            .ok_or(Error::Overflow("export diagnostics"))?;
        let bytes = message
            .len()
            .checked_add(if id.is_some() { 38 } else { 0 })
            .ok_or(Error::Overflow("export diagnostic bytes"))?;
        held.add(
            Usage {
                bytes: bytes as u64,
                ..Usage::default()
            },
            limits,
        )
    })?;
    held.add(Usage::array::<String>(count)?, limits)?;
    let mut result = Vec::new();
    result
        .try_reserve_exact(count)
        .map_err(|_| Error::Budget("exchange diagnostics"))?;
    visit_diagnostics(document, root, objects, generated, |id, message| {
        use std::fmt::Write;
        let mut string = String::new();
        string
            .try_reserve_exact(message.len() + if id.is_some() { 38 } else { 0 })
            .map_err(|_| Error::Budget("exchange diagnostic string"))?;
        if let Some(id) = id {
            write!(&mut string, "{id}: ")
                .map_err(|_| Error::Invalid("export diagnostic formatting"))?;
        }
        string.push_str(message);
        result.push(string);
        Ok(())
    })?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tiny_curve_correspondence_rejects_collapsed_poles_and_endpoints() {
        let n = NurbsCurve { degree: 1, poles: vec![[0.; 3], [2.5e-11, 0., 0.]], weights: vec![1.; 2], knots: vec![0., 0., 1., 1.], domain: [0., 1.], periodic: false, reversed: false, seam: None };
        let c = Curve::Nurbs(n);
        let mut actual = serde_json::to_value(&c).unwrap();
        curve(&actual, &c, Transform::identity()).unwrap();
        actual["Nurbs"]["poles"][1][0] = 0.into();
        assert!(curve(&actual, &c, Transform::identity()).is_err());
        let c = Curve::Line { start: [0.; 3], end: [2.5e-11, 0., 0.], domain: [0., 1.] };
        let mut actual = serde_json::to_value(&c).unwrap();
        actual["Line"]["end"][0] = 0.into();
        assert!(curve(&actual, &c, Transform::identity()).is_err());
        let c = Curve::Line { start: [0.; 3], end: [0.001, 0., 0.], domain: [0., 1.] };
        let placement = Transform::translation([1e9, 0., 0.]);
        let actual = serde_json::json!({"Line":{"start":[1e9,0.,0.],"end":[1e9+0.001,0.,0.],"domain":[0.,1.]}});
        curve(&actual, &c, placement).unwrap();
        let mut collapsed = actual;
        collapsed["Line"]["end"][0] = serde_json::json!(1e9);
        assert!(curve(&collapsed, &c, placement).is_err());
    }
    #[test]
    fn transformed_native_brep_requires_bound_expected_fingerprint() {
        use om9_core::geometry::{Mesh,NativeBrep};
        let native=NativeBrep{display:Mesh{vertices:vec![],faces:vec![],normals:vec![],colors:vec![],corner_uvs:vec![]},payload:vec![1,2,3].into(),crc:10};
        let geometry=Geometry::NativeBrep(native);let entity=Entity::new("source",geometry.clone(),None);let id=entity.id;
        let mut document=Document::new("source");document.entities.insert(id,entity);let limits=Limits::default();
        let response=serde_json::json!({"schema_version":1,"status":"expected_brep_fingerprints","objects":[{"id":id.0.to_string(),"source_crc":10,"crc":20,"payload_bytes":4}]});
        let parse=|v:&Value|expected_breps(&document,&serde_json::to_vec(v).unwrap(),&limits);
        let expected=parse(&response).unwrap();assert_eq!(expected.get(&id),Some(&(20,4)));
        let mut bad=response.clone();bad["objects"][0]["source_crc"]=11.into();assert!(parse(&bad).is_err());
        let mut bad=response.clone();bad["objects"][0]["id"]=uuid::Uuid::new_v4().to_string().into();assert!(parse(&bad).is_err());
        let mut bad=response.clone();bad["objects"]=serde_json::json!([]);assert!(parse(&bad).is_err());
        let mut bad=response.clone();bad["objects"].as_array_mut().unwrap().push(response["objects"][0].clone());assert!(parse(&bad).is_err());
        let mut inspected=serde_json::json!({"NativeBrep":{"display":{},"payload":[[1,2,3,4]],"crc":20}});
        let t=Transform::scale([-1.,1.,1.]).unwrap();assert!(geometry_with_brep(&inspected,&geometry,t,expected.get(&id).copied()).is_ok());
        assert!(geometry_with_brep(&inspected,&geometry,t,None).is_err());
        inspected["NativeBrep"]["crc"]=21.into();assert!(geometry_with_brep(&inspected,&geometry,t,expected.get(&id).copied()).is_err());
        inspected["NativeBrep"]["crc"]=20.into();inspected["NativeBrep"]["payload"]=serde_json::json!([[1,2,3]]);
        assert!(geometry_with_brep(&inspected,&geometry,t,expected.get(&id).copied()).is_err());
        inspected["NativeBrep"]["payload"]=serde_json::json!([[1,2,3,300]]);
        assert!(geometry_with_brep(&inspected,&geometry,t,expected.get(&id).copied()).is_err());
    }
    #[test]
    fn diagnostics_reserve_native_strings_and_om9_loss_messages_with_live_json() {
        let mut d = Document::new("diagnostic budget");
        let layer = om9_core::model::Layer::preset(om9_core::layers::PresetSlot::new(17).unwrap());
        d.layers.insert(layer.id, layer);
        let mut e = Entity::new("hidden root", Geometry::Point([0.; 3]), None);
        e.attributes.hidden = true;
        d.entities.insert(e.id, e);
        let root = serde_json::json!({"diagnostics":["native warning"]});
        let limits = Limits::default();
        let result = collect_diagnostics(&d, &root, &[], false, Usage::default(), &limits).unwrap();
        assert!(result.iter().any(|message| message.contains("tolerance is unspecified")));
        assert!(result.iter().any(|message| message.contains("OM9 Hidden")));
        assert!(result.iter().any(|message| message.contains("preset slot")));
        assert!(result.iter().any(|message| message == "native warning"));
        let bytes: u64 = result
            .iter()
            .map(|message| message.len() as u64)
            .sum::<u64>()
            + Usage::array::<String>(result.len()).unwrap().bytes;
        let tight = Limits {
            max_bytes: bytes + 100 - 1,
            ..limits
        };
        assert!(matches!(
            collect_diagnostics(
                &d,
                &root,
                &[],
                false,
                Usage {
                    bytes: 100,
                    ..Usage::default()
                },
                &tight
            ),
            Err(Error::Budget(_))
        ));
        let exact = Limits {
            max_bytes: bytes + 100,
            ..limits
        };
        assert_eq!(
            collect_diagnostics(
                &d,
                &root,
                &[],
                false,
                Usage {
                    bytes: 100,
                    ..Usage::default()
                },
                &exact
            )
            .unwrap(),
            result
        );
    }
    #[test]
    fn light_packet_matches_inspection() {
        let light = Geometry::Light(om9_core::geometry::Light {
            display: [1., 2., 3.],
            location: [1., 2., 3.],
            direction: [0., 0., -1.],
            diffuse: [200, 180, 160, 255],
            intensity: 0.75,
            style: 8,
        });
        let value = serde_json::to_value(&light).unwrap();
        geometry(&value, &light, Transform::identity()).unwrap();
    }

    #[test]
    fn tolerance_inspection_rejects_missing_or_changed_values() {
        let mut document = Document::new("tolerance");
        document.tolerance = Some(om9_core::history::Tolerance::try_new(0.02, 0.005, 0.1).unwrap());
        let mut root = serde_json::json!({"model_tolerance":{"absolute_mm":0.02,"relative":0.005,"angular_rad":0.1}});
        verify_model_tolerance(&document, &root).unwrap();
        root["model_tolerance"]["absolute_mm"] = serde_json::json!(0.001);
        assert!(verify_model_tolerance(&document, &root).is_err());
        root.as_object_mut().unwrap().remove("model_tolerance");
        assert!(verify_model_tolerance(&document, &root).is_err());
    }

    #[test]
    fn inspection_rejects_duplicate_keys_unknown_status_and_echoed_untransformed_geometry() {
        let d = Document::new("document");
        let limits = Limits::default();
        assert!(verify(&d, br#"{"status":"inspected","status":"written"}"#, &limits).is_err());
        assert!(verify(&d, br#"{"schema_version":1,"status":"written"}"#, &limits).is_err());
        assert!(
            geometry(
                &serde_json::json!({"Point":[1.,2.,3.]}),
                &Geometry::Point([1., 2., 3.]),
                Transform::translation([5., 0., 0.])
            )
            .is_err()
        );
        geometry(
            &serde_json::json!({"Point":[6.,2.,3.]}),
            &Geometry::Point([1., 2., 3.]),
            Transform::translation([5., 0., 0.]),
        )
        .unwrap();
    }
    #[test]
    fn circle_and_inverse_transpose_normals_use_source_placement_once() {
        let t = Transform::scale([2., 2., 2.]).unwrap();
        curve(&serde_json::json!({"Circle":{"center":[0.,0.,0.],"x":[1.,0.,0.],"y":[0.,1.,0.],"radius":6.,"domain":[0.,std::f64::consts::TAU]}}),&Curve::Circle {center:[0.;3],x:[1.,0.,0.],y:[0.,1.,0.],radius:3.,domain:[0.,std::f64::consts::TAU],angle_domain:None},t).unwrap();
        let n = transform_normal(Transform::scale([2., 1., 1.]).unwrap(), [1., 1., 0.]).unwrap();
        assert!((n[0] - 1. / 5f64.sqrt()).abs() < 1e-12);
        assert!((n[1] - 2. / 5f64.sqrt()).abs() < 1e-12);
    }
}
