// SPDX-License-Identifier: LGPL-2.1-or-later
//! Focused conversion test for a real 3DM ON_Brep payload (native inventory).
//! Run with:
//! `OM9_BREP_3DM_PAYLOAD=<payload.bin> cargo test -p om9-kernel --features 3dm-brep --test brep_3dm -- --ignored`
use om9_core::budget::Limits;
use om9_kernel::brep::{from_3dm, source_volume, to_3dm};

#[test]
#[ignore = "requires OM9_BREP_3DM_PAYLOAD and the OCCT runtime"]
fn on_brep_payload_becomes_editable_occt_and_round_trips() {
    let path = std::env::var_os("OM9_BREP_3DM_PAYLOAD").expect("OM9_BREP_3DM_PAYLOAD");
    let payload = std::fs::read(&path).expect("read payload");
    let limits = Limits::default();
    let brep = from_3dm(&payload, 0.001, 0.01, &limits).expect("convert 3DM BRep");
    assert!(brep.source().contains("CASCADE Topology"));
    assert!(!brep.display.points.is_empty());
    assert!(!brep.display.triangles.is_empty());
    let volume = source_volume(&brep, &limits).expect("OCCT volume");
    assert!(volume.is_finite() && volume >= 0.0);
    let native = to_3dm(&brep, 0.001, &limits).expect("export ON_Brep payload");
    assert!(!native.payload.is_empty());
    assert!(native.crc != 0);
    let again = from_3dm(&native.payload, 0.001, 0.01, &limits).expect("reconverted payload");
    assert_eq!(brep.display.points.len(), again.display.points.len());
    assert_eq!(brep.display.triangles.len(), again.display.triangles.len());
}
