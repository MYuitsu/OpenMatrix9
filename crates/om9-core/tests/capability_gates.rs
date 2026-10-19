// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{capability::*, geometry::GeometryKind};
fn operation() -> Operation {
    Operation {
        id: "curve-evaluate".into(),
        feature: "RCORE-00".into(),
        aliases: vec!["curveEval".into()],
        prompt: "Curve parameter: ".into(),
        requires_input: true,
        output: vec![GeometryKind::Point],
        preconditions: Default::default(),
        input_types: vec![GeometryKind::Curve],
        options: vec!["native_parameter".into()],
        backend: Backend::Rust {
            version: "1".into(),
        },
        prepare: true,
        preview: true,
        commit: false,
        receipts: vec![],
    }
}
fn receipt(scope: ReceiptScope) -> Receipt {
    Receipt {
        target: "OM9Standalone".into(),
        source_sha256: "a".repeat(64),
        binary_sha256: "b".repeat(64),
        suite: "curve_oracles".into(),
        scope,
        passed: true,
    }
}
#[test]
fn source_presence_never_enables_a_capability() {
    let mut registry = CapabilityRegistry::default();
    registry.register(operation()).unwrap();
    assert!(!registry.availability("curve-evaluate").enabled);
    assert!(
        registry
            .admit(
                "curve-evaluate",
                GeometryKind::Curve,
                &["native_parameter"],
                Phase::Prepare
            )
            .is_err()
    );
}
#[test]
fn matching_semantic_receipt_enables_only_declared_internal_rust_family() {
    let mut r = CapabilityRegistry::default();
    let mut op = operation();
    op.receipts.push(receipt(ReceiptScope::Semantic));
    r.register(op).unwrap();
    assert!(
        r.admit(
            "curve-evaluate",
            GeometryKind::Curve,
            &["native_parameter"],
            Phase::Prepare
        )
        .is_ok()
    );
    assert!(
        r.admit("curve-evaluate", GeometryKind::Mesh, &[], Phase::Prepare)
            .is_err()
    );
    assert!(
        r.admit(
            "curve-evaluate",
            GeometryKind::Curve,
            &["periodic"],
            Phase::Prepare
        )
        .is_err()
    );
    assert!(
        r.admit("curve-evaluate", GeometryKind::Curve, &[], Phase::Commit)
            .is_err()
    );
}
#[test]
fn commit_requires_actual_target_receipt_and_failed_or_legacy_receipt_is_insufficient() {
    for scope in [ReceiptScope::SourceAudit, ReceiptScope::Semantic] {
        let mut r = CapabilityRegistry::default();
        let mut op = operation();
        op.commit = true;
        op.receipts.push(receipt(scope));
        r.register(op).unwrap();
        assert!(!r.availability("curve-evaluate").enabled);
    }
    let mut r = CapabilityRegistry::default();
    let mut op = operation();
    op.commit = true;
    let mut old = receipt(ReceiptScope::StandaloneRuntime);
    old.target = "legacy_freecad".into();
    op.receipts.push(old);
    r.register(op).unwrap();
    assert!(!r.availability("curve-evaluate").enabled);
}
#[test]
fn native_subset_needs_matching_native_abi_closure_and_runtime_receipts() {
    let mut op = operation();
    op.commit = true;
    op.backend = Backend::Native {
        library: "OCCT".into(),
        version: "8.0.1".into(),
        abi: "msvc-x64".into(),
        toolkits: vec!["TKernel".into()],
        closure_verified: false,
        concurrent: false,
    };
    op.receipts = vec![
        receipt(ReceiptScope::NativeOperation),
        receipt(ReceiptScope::StandaloneRuntime),
    ];
    let mut r = CapabilityRegistry::default();
    r.register(op.clone()).unwrap();
    assert!(!r.availability("curve-evaluate").enabled);
    if let Backend::Native {
        closure_verified, ..
    } = &mut op.backend
    {
        *closure_verified = true;
    }
    let mut r = CapabilityRegistry::default();
    r.register(op).unwrap();
    assert!(r.availability("curve-evaluate").enabled);
}
