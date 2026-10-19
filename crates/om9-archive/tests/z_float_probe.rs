// SPDX-License-Identifier: LGPL-2.1-or-later
//! Guards the exact-f64 property the document-identity evidence depends on.
//!
//! This file started as the diagnostic probe that proved the defect: with
//! serde_json's default fast float parser, `12f64.hypot(6.)` is written exactly
//! (`13.416407864998739`, bits `...eefc`) but decodes one ULP away
//! (`13.41640786499874`, bits `...eefd`), so `Archive::save` refused the
//! document with `Invalid("archive encoding verification")`. The crate now
//! requests `serde_json/float_roundtrip`, and this test keeps that guarantee.
use om9_archive::Archive;
use om9_core::{
    budget::Limits,
    geometry::{Curve, Geometry},
    model::{Document, Entity},
};

#[test]
fn archive_encode_decode_keeps_stored_f64_bits_exact() {
    let limits = Limits::default();
    let parameter = 12f64.hypot(6.);
    let mut document = Document::new("exact floats");
    let entity = Entity::new(
        "segment",
        Geometry::Curve(Curve::Line {
            start: [0., 0., 0.],
            end: [3., 4., 0.],
            domain: [0., parameter],
        }),
        None,
    );
    document.entities.insert(entity.id, entity.clone());
    let bytes = Archive::encode(&document, &limits).unwrap();
    let decoded = Archive::decode(&bytes, &limits).unwrap();
    let stored = match &decoded.entities[&entity.id].geometry {
        Geometry::Curve(Curve::Line { domain, .. }) => domain[1],
        other => panic!("expected a line entity, found {other:?}"),
    };
    assert_eq!(stored.to_bits(), parameter.to_bits());
    assert_eq!(decoded, document);
}
