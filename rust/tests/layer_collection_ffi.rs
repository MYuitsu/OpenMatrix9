use openmatrix9_rust::{layer_ffi::*, layer_state::*};
fn bytes(s: &str) -> ByteView {
    ByteView {
        data: s.as_ptr(),
        len: s.len(),
    }
}
#[test]
fn collection_abi_owns_scope_result_and_never_retains_native_views() {
    let mut name = String::from("current");
    let children = [bytes(&name)];
    let nodes = [
        CollectionNodeView {
            id: bytes(&name),
            label: bytes("Current shape"),
            native_type: bytes("Part::Feature"),
            children: std::ptr::null(),
            child_count: 0,
            role: 1,
            reserved: 0,
        },
        CollectionNodeView {
            id: bytes("group"),
            label: bytes("Models"),
            native_type: bytes("App::DocumentObjectGroup"),
            children: children.as_ptr(),
            child_count: 1,
            role: 2,
            reserved: 0,
        },
    ];
    let mut collection = 0;
    assert_eq!(
        unsafe {
            om9_layer_collection_prepare(
                nodes.as_ptr(),
                nodes.len(),
                2,
                std::ptr::null(),
                0,
                &mut collection,
            )
        },
        0
    );
    name.clear();
    let mut needed = 0;
    assert_eq!(
        unsafe { om9_layer_collection_json(collection, std::ptr::null_mut(), 0, &mut needed) },
        LayerError::BufferTooSmall as u32
    );
    let mut json = vec![0; needed];
    assert_eq!(
        unsafe {
            om9_layer_collection_json(collection, json.as_mut_ptr(), json.len(), &mut needed)
        },
        0
    );
    let result: serde_json::Value = serde_json::from_slice(&json).unwrap();
    assert_eq!(result["objects"], serde_json::json!(["current"]));
    assert_eq!(
        om9_layer_clipboard_free(collection),
        LayerError::InvalidHandle as u32
    );
    assert_eq!(om9_layer_collection_free(collection), 0);
    assert_eq!(
        om9_layer_collection_free(collection),
        LayerError::InvalidHandle as u32
    );
}
#[test]
fn collection_abi_rejects_bounds_unknown_roles_scope_and_resets_outputs() {
    let mut out = 77;
    assert_eq!(
        unsafe {
            om9_layer_collection_prepare(
                std::ptr::null(),
                MAX_OBJECTS + 1,
                2,
                std::ptr::null(),
                0,
                &mut out,
            )
        },
        LayerError::LimitExceeded as u32
    );
    assert_eq!(out, 0);
    let node = CollectionNodeView {
        id: bytes("unknown"),
        label: bytes("Unsupported model"),
        native_type: bytes("App::FeaturePython"),
        children: std::ptr::null(),
        child_count: 0,
        role: 77,
        reserved: 0,
    };
    out = 77;
    assert_eq!(
        unsafe { om9_layer_collection_prepare(&node, 1, 2, std::ptr::null(), 0, &mut out) },
        LayerError::InvalidSnapshot as u32
    );
    assert_eq!(out, 0);
    out = 77;
    assert_eq!(
        unsafe {
            om9_layer_collection_prepare(std::ptr::null(), 0, 99, std::ptr::null(), 0, &mut out)
        },
        LayerError::UnsupportedVersion as u32
    );
    assert_eq!(out, 0);
    assert_eq!(
        om9_layer_collection_check_size(MAX_METADATA_BYTES + 1),
        LayerError::LimitExceeded as u32
    );
}
