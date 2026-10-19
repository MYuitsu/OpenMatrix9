// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    Error,
    budget::Limits,
    model::Document,
    resource::{Resource, ResourceKind},
};

#[test]
fn admitted_resource_owns_immutable_bytes_hash_and_typed_identity() {
    let mut bytes = vec![0, 1, 255, 17];
    let resource = Resource::from_bytes(
        "opaque payload",
        ResourceKind::OpaqueBinary,
        &bytes,
        &Limits::default(),
    )
    .unwrap();
    bytes.fill(9);
    assert_eq!(resource.payload().unwrap(), &[0, 1, 255, 17]);
    assert_eq!(resource.byte_length(), 4);
    assert!(!resource.id.is_nil());
    let cloned = resource.clone();
    assert!(std::ptr::eq(
        resource.payload().unwrap(),
        cloned.payload().unwrap()
    ));
    assert_eq!(resource.sha256(), cloned.sha256());
}

#[test]
fn descriptor_without_matching_payload_is_not_a_valid_live_document() {
    let limits = Limits::default();
    let resource = Resource::from_bytes(
        "retained bytes",
        ResourceKind::RetainedThreeDm {
            archive_version: 50,
        },
        b"owned data",
        &limits,
    )
    .unwrap();
    let descriptor = serde_json::to_vec(&resource).unwrap();
    let mut detached: Resource = serde_json::from_slice(&descriptor).unwrap();
    assert!(matches!(detached.payload(), Err(Error::Missing(_))));
    assert!(detached.attach_payload(b"other data", &limits).is_err());
    let mut doc = Document::new("resource owner");
    doc.resources.insert(detached.id, detached.clone());
    assert!(doc.validate_manifest(&limits).is_ok());
    assert!(matches!(doc.validate(&limits), Err(Error::Missing(_))));
    detached.attach_payload(b"owned data", &limits).unwrap();
    doc.resources.insert(detached.id, detached);
    assert!(doc.validate(&limits).is_ok());
}

#[test]
fn resource_admission_counts_payload_copy_peak_and_rejects_bad_metadata() {
    let limits = Limits {
        max_bytes: 1024,
        ..Limits::default()
    };
    assert!(matches!(
        Resource::from_bytes("payload", ResourceKind::OpaqueBinary, &[0; 800], &limits),
        Err(Error::Budget(_))
    ));
    assert!(
        Resource::from_bytes(
            "x",
            ResourceKind::RetainedThreeDm { archive_version: 0 },
            b"data",
            &Limits::default()
        )
        .is_err()
    );
    assert!(
        Resource::from_bytes(
            "x\0bad",
            ResourceKind::OpaqueBinary,
            b"data",
            &Limits::default()
        )
        .is_err()
    );
}
