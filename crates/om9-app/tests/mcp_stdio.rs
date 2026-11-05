// SPDX-License-Identifier: LGPL-2.1-or-later
//! The MCP stdio surface is the contract an agent client sees: initialize, list tools, build a
//! circle through the Command Line, read typed geometry back, undo/redo, save and reopen, and
//! start the same feature through the menu route. No screenshot or GPU is involved.
use om9_app::mcp::serve;
use serde_json::{Value, json};
use std::io::Cursor;

fn request(id: u64, method: &str, params: Value) -> String {
    let mut line =
        json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}).to_string();
    line.push('\n');
    line
}

fn tool(id: u64, name: &str, arguments: Value) -> String {
    request(
        id,
        "tools/call",
        json!({"name": name, "arguments": arguments}),
    )
}

fn run(input: String) -> Vec<Value> {
    let mut output = Vec::new();
    serve(Cursor::new(input), &mut output).expect("stdio surface");
    String::from_utf8(output)
        .expect("utf-8 replies")
        .lines()
        .map(|line| serde_json::from_str(line).expect("one JSON object per reply line"))
        .collect()
}

fn text(reply: &Value) -> Value {
    let text = reply["result"]["content"][0]["text"]
        .as_str()
        .expect("tool text content");
    serde_json::from_str(text).expect("tool text is JSON")
}

fn ok(reply: &Value) -> Value {
    assert_eq!(reply["result"]["isError"], false, "{reply}");
    text(reply)
}

fn id_of(reply: &Value) -> u64 {
    reply["id"].as_u64().expect("numeric request id")
}

#[test]
fn memory_tool_is_opt_in_compact_and_does_not_change_document_or_history() {
    let input = request(1, "tools/list", json!({}))
        + &tool(2, "om9_memory", json!({}))
        + &tool(3, "om9_new", json!({"name":"memory observation"}))
        + &tool(4, "om9_command", json!({"text":"Point"}))
        + &tool(5, "om9_command", json!({"text":"1,2,3"}))
        + &tool(6, "om9_state", json!({}))
        + &tool(7, "om9_memory", json!({"additional_pending_bytes":u64::MAX}))
        + &tool(8, "om9_state", json!({}));
    let replies = run(input);
    let schema = replies[0]["result"]["tools"].as_array().unwrap().iter()
        .find(|tool| tool["name"] == "om9_memory").unwrap();
    assert_eq!(schema["inputSchema"]["additionalProperties"], false);
    assert_eq!(replies[1]["result"]["isError"], true, "memory needs an existing document");
    let before = ok(&replies[5]);
    let memory = ok(&replies[6]);
    let after = ok(&replies[7]);
    assert_eq!(before, after);
    assert!(before.get("tracked_total_bytes").is_none(), "normal state must not collect memory diagnostics");
    assert!(memory.get("objects").is_none(), "memory response must not copy geometry into JSON");
    assert_eq!(memory["schema_version"], 1);
    assert_eq!(memory["external_classes"].as_array().unwrap().len(), 6);
    let total = memory["tracked_total_bytes"].as_u64().unwrap();
    assert_eq!(total, memory["tracked_materialized_bytes"].as_u64().unwrap() + memory["tracked_pending_bytes"].as_u64().unwrap());
    assert_eq!(memory["request"]["fits_known_managed_inventory"], false);
    let undo_mb = memory["undo_policy"]["max_memory_mb"].as_u64().unwrap();
    assert!(undo_mb > 0 && undo_mb <= 1024);
    if let Some(limit_mb) = memory["undo_ram_limit_mb"].as_u64() { assert!(undo_mb <= limit_mb); }
    assert_eq!(memory["not_measured"].as_array().unwrap().len(), 4);
    assert!(serde_json::to_vec(&memory).unwrap().len() < 8192);
}

#[test]
fn memory_tool_rejects_invalid_requests_without_mutating_state() {
    let mut server = om9_app::mcp::Server::new();
    let mut call = |name: &str, arguments: Value| server.handle(&json!({
        "jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":name,"arguments":arguments}
    })).unwrap();
    ok(&call("om9_new", json!({})));
    let before = ok(&call("om9_state", json!({})));
    for args in [json!({"additional_pending_bytes":-1}), json!({"additional_pending_bytes":1.5}),
        json!({"additional_pending_bytes":"8"}), json!({"additional_pending_bytes":null}),
        json!({"unexpected":true}), json!([])] {
        assert_eq!(call("om9_memory", args)["result"]["isError"], true);
    }
    assert_eq!(ok(&call("om9_state", json!({}))), before);
}

#[test]
fn benchmark_session_skips_undo_for_box_sphere_and_boolean_only_when_requested() {
    fn call(server: &mut om9_app::mcp::Server, name: &str, args: Value) -> Value {
        let reply = server.handle(&json!({"jsonrpc":"2.0","id":1,"method":"tools/call", "params":{"name":name,"arguments":args}})).unwrap();
        ok(&reply)
    }
    let mut default_server = om9_app::mcp::Server::new();
    call(&mut default_server, "om9_new", json!({}));
    let normal = call(&mut default_server, "om9_command", json!({"text":"Box 10 10 10"}));
    assert_eq!(normal["state"]["undo"], 1);
    assert_eq!(normal["state"]["benchmark_no_undo"], false);

    let mut server = om9_app::mcp::Server::new();
    let state = call(&mut server, "om9_new", json!({"benchmark_no_undo":true}));
    assert_eq!(state["benchmark_no_undo"], true);
    let box_reply = call(&mut server, "om9_command", json!({"text":"Box 10 10 10"}));
    assert_eq!(box_reply["result"]["ok"], true);
    let box_id = box_reply["state"]["objects"][0]["id"].clone();
    for (name, args) in [
        ("om9_command", json!({"text":"Sphere"})),
        ("om9_point", json!({"point":[5,5,10]})),
        ("om9_command", json!({"text":"1.5"})),
    ] {
        let reply = call(&mut server, name, args);
        assert_eq!(reply["result"]["ok"], true, "{reply}");
        assert_eq!(reply["state"]["undo"], 0);
    }
    let state = call(&mut server, "om9_state", json!({}));
    let sphere = state["objects"].as_array().unwrap().iter().find(|o| o["id"] != box_id).unwrap()["id"].clone();
    call(&mut server, "om9_select", json!({"ids":[box_id,sphere]}));
    for command in ["Boolean2Objects", "DeleteInput=Yes", "Difference", ""] {
        let reply = call(&mut server, "om9_command", json!({"text":command}));
        assert_eq!(reply["result"]["ok"], true, "{reply}");
        assert_eq!(reply["state"]["undo"], 0);
    }
    let state = call(&mut server, "om9_state", json!({}));
    assert_eq!(state["objects_total"], 1);
    assert_eq!(state["objects"][0]["geometry"]["kind"], "Brep");
}

#[test]
fn a_client_can_export_the_selected_solid_as_stl() {
    let directory = std::env::temp_dir().join(format!("om9-mcp-stl-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("test directory");
    let target = directory.join("box.stl");

    let mut input = String::new();
    input += &request(
        1,
        "initialize",
        json!({"protocolVersion": "2024-11-05", "capabilities": {}}),
    );
    input += &tool(2, "om9_new", json!({"name": "mcp stl"}));
    input += &tool(3, "om9_command", json!({"text": "Box 10 10 10"}));
    input += &tool(4, "om9_select", json!({"all": true}));
    let typed = tool(5, "om9_command", json!({"text": "ExportSTL"}));
    input += &typed;
    input += &tool(
        6,
        "om9_export_stl",
        json!({"path": target.display().to_string()}),
    );
    let replies = run(input);
    let by_id: std::collections::BTreeMap<u64, &Value> =
        replies.iter().map(|reply| (id_of(reply), reply)).collect();

    let route = ok(by_id[&5]);
    assert_eq!(route["result"]["ok"], true, "{route}");
    assert!(
        route["result"]["stl"].as_str().is_some(),
        "typed ExportSTL must announce the dialog route: {route}"
    );

    let exported = ok(by_id[&6]);
    assert_eq!(exported["triangles"], 12);
    assert_eq!(exported["units"], "mm");
    let bytes = std::fs::read(&target).expect("exported file");
    assert_eq!(bytes.len() as u64, exported["bytes"].as_u64().unwrap());
    assert_eq!(
        u32::from_le_bytes(bytes[80..84].try_into().unwrap()),
        12,
        "binary STL triangle count"
    );

    let _ = std::fs::remove_dir_all(&directory);
}

#[test]
fn stl_export_profile_is_reported_and_invalid_profile_preserves_file() {
    let target = std::env::temp_dir().join(format!("om9-mcp-profile-{}.stl", uuid::Uuid::new_v4()));
    let mut server = om9_app::mcp::Server::new();
    let list = server.handle(&json!({"jsonrpc":"2.0","id":9,"method":"tools/list","params":{}})).unwrap();
    let tools = list["result"]["tools"].as_array().unwrap();
    let export = tools.iter().find(|item| item["name"] == "om9_export_stl").unwrap();
    assert_eq!(export["inputSchema"]["properties"]["linear_tolerance_mm"]["type"], "number");
    assert_eq!(export["inputSchema"]["properties"]["angular_tolerance_rad"]["type"], "number");
    let mut call = |name: &str, args: Value| {
        server.handle(&json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":name,"arguments":args}})).unwrap()
    };
    ok(&call("om9_new", json!({})));
    ok(&call("om9_command", json!({"text":"Box 10 10 10"})));
    ok(&call("om9_select", json!({"all":true})));
    let exported = ok(&call("om9_export_stl", json!({"path":target, "linear_tolerance_mm":0.02, "angular_tolerance_rad":0.15})));
    assert_eq!(exported["requested_tolerance_mm"], 0.02);
    assert_eq!(exported["requested_angle_rad"], 0.15);
    assert!(exported["max_occt_topology_tolerance_upper_mm"].as_f64().is_some_and(|value| value >= 1e-7));
    assert_eq!(exported["self_intersections_checked"], false);
    assert_eq!(exported["topology_scope"], "source_scoped_indexed_soup_before_encoding");
    assert_eq!(exported["geometric_deviation_verification"], "unverified");
    assert_eq!(exported["stl_unit_metadata"], false);
    assert!(exported["source_document_id"].as_str().is_some_and(|id| uuid::Uuid::parse_str(id).is_ok()));
    assert_eq!(exported["duplicate_faces"], 0);
    assert_eq!(exported["edge_connected_components"], 1);
    assert!(exported["encoded_vertex_identity_ambiguous"].is_boolean());
    if exported["encoded_vertex_identity_ambiguous"] == false {
        assert_eq!(exported["encoded_closed_by_topology_checks"], exported["watertight"]);
    } else {
        assert!(exported["encoded_closed_by_topology_checks"].is_null());
    }
    let before = std::fs::read(&target).unwrap();
    let rejected = call("om9_export_stl", json!({"path":target, "linear_tolerance_mm":-1.0}));
    assert_eq!(rejected["result"]["isError"], true);
    assert_eq!(std::fs::read(&target).unwrap(), before);
    std::fs::remove_file(target).unwrap();
}

#[test]
fn a_client_can_build_read_undo_save_and_reopen_without_a_screenshot() {
    let directory = std::env::temp_dir().join(format!("om9-mcp-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("test directory");
    let project = directory.join("circle.om9");

    let mut input = String::new();
    input += &request(
        1,
        "initialize",
        json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "1"}
        }),
    );
    input += "{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n";
    input += &request(2, "tools/list", json!({}));
    input += &tool(3, "om9_new", json!({"name": "mcp circle"}));
    input += &tool(4, "om9_command", json!({"text": "Circle"}));
    input += &tool(5, "om9_point", json!({"point": [1.0, 2.0, 3.0]}));
    input += &tool(6, "om9_command", json!({"text": "5"}));
    input += &tool(7, "om9_undo", json!({}));
    input += &tool(8, "om9_redo", json!({}));
    input += &tool(
        9,
        "om9_save",
        json!({"path": project.display().to_string()}),
    );
    input += &tool(
        10,
        "om9_open",
        json!({"path": project.display().to_string()}),
    );
    input += &tool(11, "om9_dispatch", json!({"feature": "OM9-CURVE-005"}));
    input += &tool(12, "om9_command", json!({"text": "Cancel"}));
    input += &request(13, "no/such/method", json!({}));

    let replies = run(input);
    assert_eq!(replies.len(), 13, "a notification never gets a reply line");
    let by_id: std::collections::BTreeMap<u64, &Value> =
        replies.iter().map(|reply| (id_of(reply), reply)).collect();

    assert_eq!(by_id[&1]["result"]["serverInfo"]["name"], "om9");
    let names: Vec<&str> = by_id[&2]["result"]["tools"]
        .as_array()
        .expect("tool list")
        .iter()
        .map(|tool| tool["name"].as_str().expect("tool name"))
        .collect();
    for name in [
        "om9_new",
        "om9_open",
        "om9_command",
        "om9_state",
        "om9_undo",
    ] {
        assert!(
            names.contains(&name),
            "tools/list is missing {name}: {names:?}"
        );
    }

    assert_eq!(ok(by_id[&3])["document"]["objects"], 0);

    let started = ok(by_id[&4]);
    assert_eq!(started["result"]["ok"], true);
    assert_eq!(started["state"]["command"]["running"], true);

    let built = ok(by_id[&6]);
    assert_eq!(built["result"]["ok"], true, "{built}");
    assert_eq!(built["state"]["command"]["running"], false);
    assert_eq!(built["state"]["document"]["objects"], 1);
    assert_eq!(built["state"]["undo"], 1);
    let object = &built["state"]["objects"][0];
    assert_eq!(object["kind"], "Curve");
    assert_eq!(object["geometry"]["kind"], "Curve");
    assert_eq!(
        object["geometry"]["value"]["Curve"]["Circle"]["radius"],
        5.0
    );
    assert_eq!(
        object["geometry"]["value"]["Curve"]["Circle"]["center"],
        json!([1.0, 2.0, 3.0])
    );
    assert_eq!(built["state"]["last_change"]["cause"], "Edit");
    assert!(
        built["state"]["last_change"]["changes"]
            .to_string()
            .contains("EntityAdded"),
        "the typed change batch names the created object: {}",
        built["state"]["last_change"]
    );

    let undone = ok(by_id[&7]);
    assert_eq!(undone["state"]["document"]["objects"], 0);
    assert_eq!(undone["state"]["undo"], 0);
    assert_eq!(undone["state"]["redo"], 1);
    assert_eq!(undone["state"]["last_change"]["cause"], "Undo");

    let redone = ok(by_id[&8]);
    assert_eq!(redone["state"]["document"]["objects"], 1);

    let saved = ok(by_id[&9]);
    assert!(project.exists(), "om9_save wrote {}", saved["saved"]);
    assert!(saved["bytes"].as_u64().unwrap_or(0) > 0);

    let reopened = ok(by_id[&10]);
    assert_eq!(reopened["document"]["objects"], 1);

    let menu = ok(by_id[&11]);
    assert_eq!(menu["result"]["ok"], true, "{menu}");
    assert_eq!(menu["state"]["command"]["running"], true);
    assert_eq!(menu["state"]["command"]["feature"], "OM9-CURVE-005");

    let cancelled = ok(by_id[&12]);
    assert_eq!(cancelled["state"]["command"]["running"], false);

    assert_eq!(by_id[&13]["error"]["code"], -32601);

    let _ = std::fs::remove_dir_all(&directory);
}
