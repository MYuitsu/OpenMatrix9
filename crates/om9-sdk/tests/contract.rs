use std::{ffi::c_void, mem::size_of, ptr};

use om9_sdk::{
    CommandDescriptor, Geometry, Host, HostRequest, Plugin, PluginDescriptor, PluginRequest,
    Result, SDK_VERSION, SdkError, Value,
    abi::{self, HostApi, ReplyCapture, ReplyFn, STATUS_ERROR, STATUS_OK},
};

#[derive(Default)]
struct TestPlugin;

impl Plugin for TestPlugin {
    fn descriptor() -> PluginDescriptor {
        PluginDescriptor {
            id: "org.openmatrix9.sdk-test".into(),
            name: "SDK test".into(),
            version: "0.1.0".into(),
            sdk_version: SDK_VERSION,
            commands: vec![CommandDescriptor {
                id: "AddPoint".into(),
                title: "Add point".into(),
            }],
        }
    }

    fn execute(&mut self, host: &mut Host<'_>, command: &str, _: Value) -> Result<Value> {
        match command {
            "panic" => panic!("contained test panic"),
            "oversized_error" => Err(SdkError::new(
                "test",
                "x".repeat(abi::MAX_MESSAGE_BYTES + 1),
            )),
            _ => Ok(Value::String(host.add_point("Example", [1.0, 2.0, 3.0])?)),
        }
    }
}

om9_sdk::export_plugin!(TestPlugin);

unsafe extern "C" fn add_host(
    _: *mut c_void,
    data: *const u8,
    len: usize,
    context: *mut c_void,
    reply: ReplyFn,
) -> i32 {
    let request: HostRequest =
        serde_json::from_slice(unsafe { std::slice::from_raw_parts(data, len) }).unwrap();
    assert_eq!(
        request,
        HostRequest::ObjectAdd {
            label: "Example".into(),
            geometry: Geometry::Point {
                point: [1.0, 2.0, 3.0]
            }
        }
    );
    unsafe { reply(context, br#""opaque-object-id""#.as_ptr(), 18) }
}

fn api(call: abi::CallFn) -> HostApi {
    HostApi {
        abi_version: abi::ABI_VERSION,
        struct_size: size_of::<HostApi>() as u32,
        context: ptr::null_mut(),
        call: Some(call),
    }
}

fn invoke(api: *const HostApi, bytes: &[u8]) -> Result<Value> {
    let mut capture = ReplyCapture::new();
    let status = unsafe {
        om9_plugin_entry_v1(
            api,
            bytes.as_ptr(),
            bytes.len(),
            capture.context(),
            abi::capture_reply,
        )
    };
    serde_json::from_slice(&capture.finish(status)?)
        .map_err(|error| SdkError::new("test_decode", error.to_string()))
}

fn execute(command: &str) -> Vec<u8> {
    serde_json::to_vec(&PluginRequest::Execute {
        command: command.into(),
        arguments: Value::Null,
    })
    .unwrap()
}

#[test]
fn macro_entry_descriptor_and_host_roundtrip() {
    let api = api(add_host);
    let descriptor = invoke(&api, br#"{"action":"describe"}"#).unwrap();
    assert_eq!(descriptor["sdk_version"], SDK_VERSION);
    assert_eq!(
        invoke(ptr::null(), br#"{"action":"describe"}"#).unwrap(),
        descriptor
    );
    assert_eq!(
        invoke(&api, &execute("AddPoint")).unwrap(),
        "opaque-object-id"
    );
}

#[test]
fn invalid_version_size_and_callback_are_rejected() {
    let mut api = api(add_host);
    api.abi_version = 999;
    assert_eq!(
        invoke(&api, &execute("AddPoint")).unwrap_err().code,
        "abi_version"
    );
    api.abi_version = abi::ABI_VERSION;
    api.struct_size = 8;
    assert_eq!(
        invoke(&api, &execute("AddPoint")).unwrap_err().code,
        "abi_size"
    );
    api.struct_size = size_of::<HostApi>() as u32;
    api.call = None;
    assert_eq!(
        invoke(&api, &execute("AddPoint")).unwrap_err().code,
        "invalid_host"
    );
    assert_eq!(
        invoke(ptr::null(), &execute("AddPoint")).unwrap_err().code,
        "invalid_host"
    );
}

#[test]
fn short_header_is_checked_before_reading_full_host_table() {
    #[repr(C, align(8))]
    struct Header(u32, u32);
    let header = Header(abi::ABI_VERSION, 8);
    let error = invoke((&header as *const Header).cast(), &execute("AddPoint")).unwrap_err();
    assert_eq!(error.code, "abi_size");
}

#[test]
fn malformed_and_oversized_request_are_rejected() {
    let api = api(add_host);
    for bytes in [&b""[..], b"no-json", b"\xff", br#"{"action":"missing"}"#] {
        assert_eq!(invoke(&api, bytes).unwrap_err().code, "invalid_request");
    }
    let mut capture = ReplyCapture::new();
    // The length is rejected without dereferencing this pointer.
    let status = unsafe {
        om9_plugin_entry_v1(
            &api,
            ptr::null(),
            abi::MAX_MESSAGE_BYTES + 1,
            capture.context(),
            abi::capture_reply,
        )
    };
    assert_eq!(capture.finish(status).unwrap_err().code, "invalid_request");
}

#[test]
fn plugin_panic_is_an_error_reply_and_later_invocation_works() {
    let api = api(add_host);
    assert_eq!(
        invoke(&api, &execute("panic")).unwrap_err().code,
        "plugin_panic"
    );
    assert!(invoke(&api, &execute("AddPoint")).is_ok());
}

#[test]
fn oversized_plugin_error_is_replaced_by_bounded_error() {
    assert_eq!(
        invoke(&api(add_host), &execute("oversized_error"))
            .unwrap_err()
            .code,
        "message_size"
    );
}

unsafe extern "C" fn error_host(
    _: *mut c_void,
    _: *const u8,
    _: usize,
    context: *mut c_void,
    reply: ReplyFn,
) -> i32 {
    let error = br#"{"code":"locked","message":"Object is locked"}"#;
    unsafe {
        reply(context, error.as_ptr(), error.len());
    }
    STATUS_ERROR
}

#[test]
fn host_sdk_error_propagates_without_losing_code() {
    let error = invoke(&api(error_host), &execute("AddPoint")).unwrap_err();
    assert_eq!(error, SdkError::new("locked", "Object is locked"));
}

unsafe extern "C" fn missing_host(
    _: *mut c_void,
    _: *const u8,
    _: usize,
    _: *mut c_void,
    _: ReplyFn,
) -> i32 {
    STATUS_OK
}

unsafe extern "C" fn repeated_host(
    _: *mut c_void,
    _: *const u8,
    _: usize,
    context: *mut c_void,
    reply: ReplyFn,
) -> i32 {
    unsafe {
        assert_eq!(reply(context, b"null".as_ptr(), 4), STATUS_OK);
        assert_eq!(reply(context, b"null".as_ptr(), 4), STATUS_ERROR);
    }
    STATUS_OK
}

#[test]
fn missing_and_repeated_host_replies_are_errors() {
    assert_eq!(
        invoke(&api(missing_host), &execute("AddPoint"))
            .unwrap_err()
            .code,
        "missing_reply"
    );
    assert_eq!(
        invoke(&api(repeated_host), &execute("AddPoint"))
            .unwrap_err()
            .code,
        "invalid_reply"
    );
}

#[test]
fn reply_capture_checks_size_null_status_and_copies_bytes() {
    let mut capture = ReplyCapture::new();
    assert_eq!(
        unsafe { abi::capture_reply(capture.context(), ptr::null(), 1) },
        STATUS_ERROR
    );
    assert_eq!(capture.finish(STATUS_OK).unwrap_err().code, "invalid_reply");
    let mut capture = ReplyCapture::new();
    assert_eq!(
        unsafe { abi::capture_reply(capture.context(), ptr::null(), abi::MAX_MESSAGE_BYTES + 1) },
        STATUS_ERROR
    );
    let mut capture = ReplyCapture::new();
    let mut data = b"123".to_vec();
    assert_eq!(
        unsafe { abi::capture_reply(capture.context(), data.as_ptr(), data.len()) },
        STATUS_OK
    );
    data.fill(b'0');
    assert_eq!(capture.finish(STATUS_OK).unwrap(), b"123");
    let mut capture = ReplyCapture::new();
    assert_eq!(
        unsafe { abi::capture_reply(capture.context(), ptr::null(), 0) },
        STATUS_OK
    );
    assert_eq!(capture.finish(44).unwrap_err().code, "invalid_status");
}

#[test]
fn larger_host_table_is_forward_compatible_at_same_abi_version() {
    let mut api = api(add_host);
    api.struct_size += 8;
    // Full v1 table remains readable; extra bytes are neither read nor required.
    assert!(invoke(&api, &execute("AddPoint")).is_ok());
}

#[test]
fn typed_parameter_wire_form_is_unit_safe_and_round_trips() {
    use om9_sdk::{Dimension, ParameterDomain, ParameterSpec, ParameterValue};
    let length: ParameterValue =
        serde_json::from_value(serde_json::json!({"kind": "length", "millimeters": 2.5})).unwrap();
    assert_eq!(length, ParameterValue::Length { millimeters: 2.5 });
    // The unit is the tag, so an angle record cannot stand in for a length.
    assert!(
        serde_json::from_value::<ParameterValue>(
            serde_json::json!({"kind": "angle", "millimeters": 2.5})
        )
        .is_err()
    );
    // A bare number is not an implicit millimetre.
    assert!(serde_json::from_value::<ParameterValue>(serde_json::json!(2.5)).is_err());
    let spec = ParameterSpec {
        name: "radius".into(),
        domain: ParameterDomain::Number {
            dimension: Dimension::Length,
            min: Some(1.0),
            min_inclusive: true,
            max: Some(5.0),
            step: Some(0.5),
        },
        default: None,
        required: true,
    };
    let wire = serde_json::to_value(&spec).unwrap();
    assert_eq!(wire["domain"]["kind"], "number");
    assert_eq!(wire["domain"]["dimension"], "length");
    assert_eq!(serde_json::from_value::<ParameterSpec>(wire).unwrap(), spec);
}

#[test]
fn new_read_dtos_keep_their_wire_shape() {
    let bounds = om9_sdk::ObjectBounds {
        min: [-1.0, -2.0, -3.0],
        max: [1.0, 2.0, 3.0],
        size: [2.0, 4.0, 6.0],
        center: [0.0, 0.0, 0.0],
    };
    let json = serde_json::to_value(&bounds).unwrap();
    assert_eq!(json["min"], serde_json::json!([-1.0, -2.0, -3.0]));
    assert_eq!(json["size"], serde_json::json!([2.0, 4.0, 6.0]));
    let back: om9_sdk::ObjectBounds = serde_json::from_value(json).unwrap();
    assert_eq!(back, bounds);

    let segment = om9_sdk::CurveSegment {
        kind: "circle".into(),
        domain: Some([0.0, 1.0]),
        angle_domain: Some([0.0, 1.5]),
        reversed: false,
    };
    let json = serde_json::to_value(&segment).unwrap();
    assert_eq!(json["kind"], "circle");
    assert_eq!(json["angle_domain"], serde_json::json!([0.0, 1.5]));
    assert_eq!(
        serde_json::from_value::<om9_sdk::CurveSegment>(json).unwrap(),
        segment
    );

    let mut by_kind = std::collections::BTreeMap::new();
    by_kind.insert("Curve".to_string(), 2u64);
    let summary = om9_sdk::ObjectSummary { total: 3, by_kind };
    let json = serde_json::to_value(&summary).unwrap();
    assert_eq!(json["total"], 3);
    assert_eq!(json["by_kind"]["Curve"], 2);

    let area = om9_sdk::MeasuredArea {
        square_millimeters: 100.0,
    };
    assert_eq!(
        serde_json::to_value(area).unwrap()["square_millimeters"],
        100.0
    );

    let change = om9_sdk::ProjectChange {
        from_revision: 1,
        to_revision: 2,
        cause: "undo".into(),
        changes: vec![om9_sdk::ChangeEntry {
            kind: "entity_removed".into(),
            id: Some("id".into()),
        }],
    };
    let json = serde_json::to_value(&change).unwrap();
    assert_eq!(json["cause"], "undo");
    assert_eq!(json["changes"][0]["kind"], "entity_removed");

    let extension = om9_sdk::ObjectExtension {
        version: 1,
        payload: "{\"a\":1}".into(),
    };
    assert_eq!(
        serde_json::to_value(extension).unwrap()["payload"],
        "{\"a\":1}"
    );
}
