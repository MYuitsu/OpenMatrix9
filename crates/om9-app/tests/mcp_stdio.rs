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
