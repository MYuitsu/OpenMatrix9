// SPDX-License-Identifier: LGPL-2.1-or-later
use openmatrix9_rust::retained_archive::{
    ManifestError, canonical_uuid, om9_retained_manifest_validate,
    om9_retained_record_identity_valid, om9_retained_uuid_normalize, valid_record_identity,
    validate_manifest,
};
use std::ffi::CString;

const DIGEST: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const UUID: &str = "12345678-abcd-4321-8765-0123456789ab";
const OTHER: &str = "99999999-abcd-4321-8765-0123456789ab";
const NAMESPACE: &str = "a1111111-2222-4333-8444-555555555555";
fn manifest(records: &str, scale: &str) -> String {
    format!(
        r#"{{"schema_version":1,"archive_sha256":"{DIGEST}","records":[{records}],"scale_mm":{scale}}}"#
    )
}
fn row(uuid: &str) -> String {
    format!(
        r#"{{"source_uuid":"{uuid}","class_name":"ON_MorphControl","capability":"retained_cage"}}"#
    )
}
fn check(raw: &str) -> Result<f64, ManifestError> {
    validate_manifest(
        raw,
        DIGEST,
        UUID,
        "ON_MorphControl",
        "retained_cage",
        NAMESPACE,
        1,
    )
}

#[test]
fn early_identity_policy_handles_semantic_uuid_and_namespace() {
    for id in [
        UUID.to_owned(),
        UUID.to_uppercase(),
        format!("{{{UUID}}}"),
        format!("{{{}}}", UUID.to_uppercase()),
    ] {
        assert!(valid_record_identity(&id, NAMESPACE));
        assert!(!valid_record_identity(&id, ""));
        assert!(!valid_record_identity(&id, "import\0a"));
        let id = CString::new(id).unwrap();
        let namespace = CString::new(NAMESPACE).unwrap();
        let empty = CString::new("").unwrap();
        assert!(unsafe { om9_retained_record_identity_valid(id.as_ptr(), namespace.as_ptr()) });
        assert!(!unsafe { om9_retained_record_identity_valid(id.as_ptr(), empty.as_ptr()) });
    }
    let namespace = CString::new(NAMESPACE).unwrap();
    for id in [
        "",
        "00000000-0000-0000-0000-000000000000",
        "{00000000-0000-0000-0000-000000000000}",
        "invalid",
    ] {
        assert!(!valid_record_identity(id, NAMESPACE));
        let id = CString::new(id).unwrap();
        assert!(!unsafe { om9_retained_record_identity_valid(id.as_ptr(), namespace.as_ptr()) });
    }
    let id = CString::new(UUID).unwrap();
    let invalid_utf8 = [0xff_u8, 0];
    assert!(!unsafe { om9_retained_record_identity_valid(std::ptr::null(), namespace.as_ptr()) });
    assert!(!unsafe { om9_retained_record_identity_valid(id.as_ptr(), std::ptr::null()) });
    assert!(!unsafe {
        om9_retained_record_identity_valid(invalid_utf8.as_ptr().cast(), namespace.as_ptr())
    });
    assert!(!unsafe {
        om9_retained_record_identity_valid(id.as_ptr(), invalid_utf8.as_ptr().cast())
    });
}

#[test]
fn namespace_is_a_canonical_import_uuid_not_an_arbitrary_label() {
    for namespace in ["import-a", " ", "00000000-0000-0000-0000-000000000000", "{a1111111-2222-4333-8444-555555555555}", "A1111111-2222-4333-8444-555555555555"] {
        assert!(!valid_record_identity(UUID, namespace), "{namespace}");
    }
    assert!(valid_record_identity(UUID, NAMESPACE));
}

#[test]
fn uuid_normalization_returns_decoder_canonical_spelling_atomically() {
    for value in [
        UUID.to_owned(),
        UUID.to_uppercase(),
        format!("{{{}}}", UUID.to_uppercase()),
    ] {
        assert_eq!(canonical_uuid(&value).as_deref(), Some(UUID));
        let input = CString::new(value).unwrap();
        let mut output = [b'!'; 37];
        assert!(unsafe {
            om9_retained_uuid_normalize(input.as_ptr(), output.as_mut_ptr().cast(), output.len())
        });
        assert_eq!(&output[..36], UUID.as_bytes());
        assert_eq!(output[36], 0);
        output.fill(b'!');
        assert!(!unsafe {
            om9_retained_uuid_normalize(input.as_ptr(), output.as_mut_ptr().cast(), 36)
        });
        assert_eq!(output, [b'!'; 37]);
    }
    let mut output = [b'!'; 37];
    let nil = CString::new("00000000-0000-0000-0000-000000000000").unwrap();
    assert!(!unsafe { om9_retained_uuid_normalize(nil.as_ptr(), output.as_mut_ptr().cast(), 37) });
    assert!(!unsafe {
        om9_retained_uuid_normalize(std::ptr::null(), output.as_mut_ptr().cast(), 37)
    });
    assert_eq!(output, [b'!'; 37]);
}

#[test]
fn validates_units_and_semantic_uuid_identity_without_matching_other_capabilities() {
    let rows = format!(
        "{},{}",
        row(&format!("{{{}}}", UUID.to_uppercase())),
        row(OTHER)
            .replace("ON_MorphControl", "ON_Mesh")
            .replace("retained_cage", "mesh")
    );
    assert_eq!(check(&manifest(&rows, "25.4")), Ok(25.4));
    assert_eq!(check(&manifest(&row(UUID), "2.5")), Ok(2.5));
}

#[test]
fn rejects_invalid_or_duplicate_uuid_anywhere_in_the_table() {
    for id in [
        "",
        "00000000-0000-0000-0000-000000000000",
        "12345678-abcd-4321-8765-0123456789ag",
        "12345678abcd432187650123456789ab",
        "{12345678-abcd-4321-8765-0123456789ab",
        "12345678-abcd-4321-8765-0123456789ab trailing",
    ] {
        assert_eq!(
            check(&manifest(&format!("{},{}", row(UUID), row(id)), "1")),
            Err(ManifestError::Identity),
            "{id}"
        );
    }
    for duplicate in [UUID.to_owned(), UUID.to_uppercase(), format!("{{{UUID}}}")] {
        assert_eq!(
            check(&manifest(
                &format!("{},{}", row(UUID), row(&duplicate)),
                "1"
            )),
            Err(ManifestError::Identity)
        );
    }
    assert_eq!(
        check(&manifest(&format!("{},null", row(UUID)), "1")),
        Err(ManifestError::Identity)
    );
}

#[test]
fn selected_record_and_namespace_must_be_valid_and_match_exactly() {
    let raw = manifest(&row(UUID), "1");
    for (id, class, capability, namespace) in [
        (OTHER, "ON_MorphControl", "retained_cage", NAMESPACE),
        (UUID, "ON_NurbsCage", "retained_cage", NAMESPACE),
        (UUID, "ON_MorphControl", "mesh", NAMESPACE),
        (UUID, "ON_MorphControl", "retained_cage", ""),
        (UUID, "ON_MorphControl", "retained_cage", "a\0b"),
    ] {
        assert_eq!(
            validate_manifest(&raw, DIGEST, id, class, capability, namespace, 1),
            Err(ManifestError::Identity)
        );
    }
    assert_eq!(check(&manifest("", "1")), Err(ManifestError::Identity));
    assert_eq!(
        check(
            &raw.replace("\"records\":[", "\"records\":{\"rows\":[")
                .replace("],\"scale_mm\"", "]},\"scale_mm\"")
        ),
        Err(ManifestError::Identity)
    );
}

#[test]
fn schema_and_digest_are_strict() {
    let raw = manifest(&row(UUID), "1");
    for digest in [
        DIGEST.to_uppercase(),
        "abcd".to_owned(),
        "g".repeat(64),
        "0".repeat(64),
    ] {
        assert_eq!(
            validate_manifest(
                &raw,
                &digest,
                UUID,
                "ON_MorphControl",
                "retained_cage",
                NAMESPACE,
                1
            ),
            Err(ManifestError::Integrity)
        );
    }
    assert_eq!(
        check(&raw.replace(DIGEST, &DIGEST.to_uppercase())),
        Err(ManifestError::Integrity)
    );
    for schema in [0, 2, i64::MAX] {
        assert_eq!(
            validate_manifest(
                &raw,
                DIGEST,
                UUID,
                "ON_MorphControl",
                "retained_cage",
                NAMESPACE,
                schema
            ),
            Err(ManifestError::Integrity)
        );
    }
    for value in ["2", "1.5", "\"1\"", "null"] {
        assert_eq!(
            check(&raw.replace(
                "\"schema_version\":1",
                &format!("\"schema_version\":{value}")
            )),
            Err(ManifestError::Integrity)
        );
    }
}

#[test]
fn retained_replay_rejects_unresolved_or_malformed_dependency_reports() {
    let raw = manifest(&row(UUID), "1");
    for report in [r#"["Missing block member"]"#, "null", "{}", "true"] {
        let raw = raw.replacen('{', &format!("{{\"issues\":{report},"), 1);
        assert!(check(&raw).is_err(), "{report}");
    }
    let clean = raw.replacen('{', "{\"issues\":[],", 1);
    assert_eq!(check(&clean), Ok(1.));
}

#[test]
fn source_dependencies_must_resolve_across_records_and_components_without_cycles() {
    let raw = manifest(&row(UUID), "1");
    let with_dependency = raw.replace("\"class_name\"", &format!("\"dependencies\":[\"{OTHER}\"],\"class_name\""));
    assert!(check(&with_dependency).is_err(), "missing dependency must fail");
    let component = format!(r#"{{"source_uuid":"{OTHER}","dependencies":[]}}"#);
    let resolved = with_dependency.replacen('{', &format!("{{\"components\":[{component}],"), 1);
    assert_eq!(check(&resolved), Ok(1.));
    let cyclic_component = component.replace("[]", &format!("[\"{UUID}\"]"));
    let cyclic = with_dependency.replacen('{', &format!("{{\"components\":[{cyclic_component}],"), 1);
    assert!(check(&cyclic).is_err(), "block dependency cycle must fail");
    let duplicate = resolved.replace(OTHER, UUID);
    assert!(check(&duplicate).is_err(), "component identity must not alias geometry");
    for dependencies in ["null", "{}", "[7]", "[\"invalid\"]"] {
        assert!(check(&raw.replace("\"class_name\"", &format!("\"dependencies\":{dependencies},\"class_name\""))).is_err());
    }
}

#[test]
fn dependency_ffi_enforces_utf8_and_input_budget() {
    use openmatrix9_rust::retained_archive::om9_retained_dependencies_valid;
    let raw = manifest(&row(UUID), "1");
    unsafe {
        assert!(om9_retained_dependencies_valid(raw.as_ptr(), raw.len()));
        assert!(!om9_retained_dependencies_valid(std::ptr::null(), 1));
        assert!(!om9_retained_dependencies_valid(raw.as_ptr(), usize::MAX));
        assert!(!om9_retained_dependencies_valid([0xff].as_ptr(), 1));
    }
}

#[test]
fn units_require_positive_finite_number() {
    for scale in ["0", "-1", "\"25.4\"", "null", "true"] {
        assert_eq!(
            check(&manifest(&row(UUID), scale)),
            Err(ManifestError::Scale)
        );
    }
    assert_eq!(check(&manifest(&row(UUID), "1e-300")), Ok(1e-300));
}

#[test]
fn shared_parser_rejects_malformed_duplicate_keys_and_retained_budgets() {
    let raw = manifest(&row(UUID), "1");
    assert_eq!(
        check(&raw.replace("ON_MorphControl", "ON_MorphContr\\u+06fl")),
        Err(ManifestError::Json)
    );
    for invalid in [
        format!("{raw} null"),
        raw.replace("\"scale_mm\":1", "\"scale_mm\":1,\"scale_\\u006dm\":2"),
        raw.replace("\"scale_mm\":1", "\"scale_mm\":01"),
        raw.replace("\"scale_mm\":1", "\"scale_mm\":1e999"),
        " ".repeat(32 * 1024 * 1024 + 1),
        format!("{{\"extra\":{}0{}}}", "[".repeat(65), "]".repeat(65)),
        format!(
            "{{\"extra\":[{}]}}",
            "null,".repeat(1_000_000).trim_end_matches(',')
        ),
    ] {
        assert_eq!(check(&invalid), Err(ManifestError::Json));
    }
    // Retained manifests can exceed the builder recipe byte/node/depth limits.
    let extended = raw.replacen(
        '{',
        &format!(
            "{{\"extra\":[{}],\"deep\":{}0{},",
            "null,".repeat(20_000).trim_end_matches(','),
            "[".repeat(40),
            "]".repeat(40)
        ),
        1,
    );
    assert_eq!(check(&extended), Ok(1.));
    let large = raw.replacen(
        '{',
        &format!("{{\"extra\":\"{}\",", "x".repeat(1_048_577)),
        1,
    );
    assert_eq!(check(&large), Ok(1.));
}

#[test]
fn ffi_reports_stable_codes_and_only_writes_output_after_success() {
    let digest = CString::new(DIGEST).unwrap();
    let uuid = CString::new(UUID).unwrap();
    let class = CString::new("ON_MorphControl").unwrap();
    let capability = CString::new("retained_cage").unwrap();
    let namespace = CString::new(NAMESPACE).unwrap();
    let mut scale;
    for (raw, schema, expected) in [
        (manifest(&row(UUID), "25.4"), 1, 0),
        ("{".to_owned(), 1, -1),
        (manifest(&row(UUID), "1"), 2, -2),
        (manifest(&row(OTHER), "1"), 1, -3),
        (manifest(&row(UUID), "0"), 1, -4),
    ] {
        let raw = CString::new(raw).unwrap();
        scale = -99.;
        let code = unsafe {
            om9_retained_manifest_validate(
                raw.as_ptr(),
                digest.as_ptr(),
                uuid.as_ptr(),
                class.as_ptr(),
                capability.as_ptr(),
                namespace.as_ptr(),
                schema,
                &mut scale,
            )
        };
        assert_eq!(code, expected);
        assert_eq!(scale, if expected == 0 { 25.4 } else { -99. });
    }
    scale = -99.;
    let invalid_utf8 = [0xff_u8, 0];
    let code = unsafe {
        om9_retained_manifest_validate(
            invalid_utf8.as_ptr().cast(),
            digest.as_ptr(),
            uuid.as_ptr(),
            class.as_ptr(),
            capability.as_ptr(),
            namespace.as_ptr(),
            1,
            &mut scale,
        )
    };
    assert_eq!(code, -1);
    assert_eq!(scale, -99.);
    let code = unsafe {
        om9_retained_manifest_validate(
            std::ptr::null(),
            digest.as_ptr(),
            uuid.as_ptr(),
            class.as_ptr(),
            capability.as_ptr(),
            namespace.as_ptr(),
            1,
            &mut scale,
        )
    };
    assert_eq!(code, -1);
    assert_eq!(scale, -99.);
}
