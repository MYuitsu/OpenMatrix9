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
fn executable_reopens_box_extracts_source_wireframe_and_undoes_as_one_step() {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let path=std::env::temp_dir().join(format!("om9-executable-wireframe-{}.om9",uuid::Uuid::new_v4()));
    let mut child=Command::new(env!("CARGO_BIN_EXE_om9"))
        .arg("--mcp").stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    let input=tool(1,"om9_new",json!({"name":"reopened box"}))
        + &tool(2,"om9_command",json!({"text":"Box 4 5 6"}))
        + &tool(3,"om9_save",json!({"path":path}))
        + &tool(4,"om9_open",json!({"path":path}))
        + &tool(5,"om9_select",json!({"all":true}))
        + &tool(6,"om9_command",json!({"text":"ExtractWireframe"}))
        + &tool(7,"om9_undo",json!({})) + &tool(8,"om9_redo",json!({}));
    child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
    let output=child.wait_with_output().unwrap();
    std::fs::remove_file(path).unwrap();
    assert!(output.status.success());
    let replies:Vec<Value>=std::str::from_utf8(&output.stdout).unwrap().lines()
        .map(|line|serde_json::from_str(line).unwrap()).collect();
    assert_eq!(replies.len(),8);
    for reply in &replies {ok(reply);}
    let state=ok(&replies[5]);
    assert_eq!(state["state"]["document"]["objects"],13);
    let objects=state["state"]["objects"].as_array().unwrap();
    let mut edges=std::collections::BTreeSet::new();
    for object in objects.iter().filter(|o|o["kind"]=="Curve") {
        let geometry:om9_core::geometry::Geometry=serde_json::from_value(object["geometry"]["value"].clone()).unwrap();
        let om9_core::geometry::Geometry::Curve(curve)=geometry else {panic!("expected curve")};
        let evaluate=|t|om9_core::evaluation::evaluate_curve(&curve,om9_core::evaluation::Parameter::Fraction(t),0,
            om9_core::geometry::Transform::identity(),&om9_core::budget::Limits::default()).unwrap()[0];
        let (a,b)=(evaluate(0.),evaluate(1.));
        for p in [a,b] {assert!(p.iter().zip([4.,5.,6.]).all(|(v,d)|v.abs()<1e-8||(*v-d).abs()<1e-8));}
        let corner=|p:[f64;3]|[p[0]>2.,p[1]>2.5,p[2]>3.]; let (a,b)=(corner(a),corner(b));
        assert_eq!((0..3).filter(|i|a[*i]!=b[*i]).count(),1);
        assert!(edges.insert(if a<b {(a,b)} else {(b,a)}));
    }
    assert_eq!(edges.len(),12);
    assert_eq!(ok(&replies[6])["state"]["document"]["objects"],1);
    assert_eq!(ok(&replies[7])["state"]["document"]["objects"],13);
}

#[test]
fn executable_reopens_mixed_wire_sources_and_saves_one_undo_transaction() {
    use std::io::Write;
    use std::process::{Command, Stdio};
    use om9_core::{budget::Limits, evaluation::{evaluate_curve, Parameter}, geometry::{Curve, Geometry, Mesh, Transform}, model::{Document, Entity}, transaction::Cancellation};
    use om9_kernel::creation::{self, Primitive};
    let limits = Limits::default();
    let box_entity = Entity::new("box", Geometry::Brep(creation::primitive(
        Primitive::Box { x_mm:4., y_mm:5., z_mm:6. },0.1,&limits).unwrap()), None);
    let mut mesh = Entity::new("mesh", Geometry::Mesh(Mesh { vertices: vec![[0.,0.,0.],[2.,0.,0.],[0.,2.,0.]],
        faces: vec![vec![0,1,2]], normals: vec![], colors: vec![], corner_uvs: vec![] }), None);
    mesh.placement = Transform::translation([20.,0.,0.]);
    let mut cylinder = Entity::new("closed surface", Geometry::Surface(om9_core::surface_kernel::cylinder(
        [0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,5.,&limits).unwrap()), None);
    cylinder.placement = Transform::translation([40.,0.,0.]);
    let mut document = Document::new("mixed wire executable");
    for entity in [box_entity,mesh,cylinder] { document.entities.insert(entity.id,entity); }
    let base = std::env::temp_dir().join(format!("om9-mixed-wire-executable-{}",uuid::Uuid::new_v4()));
    let input_path = base.with_extension("om9"); let output_path = base.with_extension("result.om9");
    om9_archive::Archive::save(&input_path,&document,&limits,&Cancellation::default()).unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_om9")).arg("--mcp")
        .stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    let input = tool(1,"om9_open",json!({"path":input_path}))
        + &tool(2,"om9_select",json!({"all":true}))
        + &tool(3,"om9_command",json!({"text":"ExtractWireframe"}))
        + &tool(4,"om9_undo",json!({})) + &tool(5,"om9_redo",json!({}))
        + &tool(6,"om9_save",json!({"path":output_path}));
    child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
    let output = child.wait_with_output().unwrap(); assert!(output.status.success());
    let replies:Vec<Value> = std::str::from_utf8(&output.stdout).unwrap().lines().map(|line|serde_json::from_str(line).unwrap()).collect();
    assert_eq!(replies.len(),6);
    for reply in &replies { assert_eq!(ok(reply)["result"]["ok"].as_bool().unwrap_or(true),true,"{reply}"); }
    assert_eq!(ok(&replies[2])["state"]["document"]["objects"],21);
    assert_eq!(ok(&replies[3])["state"]["document"]["objects"],3);
    assert_eq!(ok(&replies[4])["state"]["document"]["objects"],21);
    let saved=om9_archive::Archive::open(&output_path,&limits).unwrap();
    for (id,source) in &document.entities { assert_eq!(saved.entities.get(id),Some(source)); }
    let mut counts=[0,0,0]; let mut cylinder_types=[0,0,0];
    let mut midpoint_sets:[Vec<[f64;3]>;3]=Default::default();
    for entity in saved.entities.values().filter(|e| !document.entities.contains_key(&e.id)) {
        let Geometry::Curve(curve)=&entity.geometry else { panic!("expected curve") };
        let group=if entity.placement==Transform::translation([20.,0.,0.]) {1}
            else if entity.placement==Transform::translation([40.,0.,0.]) {2} else {0};
        let sample=|t|evaluate_curve(curve,Parameter::Fraction(t),0,entity.placement,&limits).unwrap()[0];
        let point=sample(0.5);
        assert!(!midpoint_sets[group].iter().any(|q| (0..3).all(|i| (point[i]-q[i]).abs()<1e-7)),"duplicate curve");
        midpoint_sets[group].push(point); counts[group]+=1;
        match group {
            0 => {
                let x=(point[0]-2.).abs()<1e-7 && [0.,5.].iter().any(|v|(point[1]-v).abs()<1e-7) && [0.,6.].iter().any(|v|(point[2]-v).abs()<1e-7);
                let y=(point[1]-2.5).abs()<1e-7 && [0.,4.].iter().any(|v|(point[0]-v).abs()<1e-7) && [0.,6.].iter().any(|v|(point[2]-v).abs()<1e-7);
                let z=(point[2]-3.).abs()<1e-7 && [0.,4.].iter().any(|v|(point[0]-v).abs()<1e-7) && [0.,5.].iter().any(|v|(point[1]-v).abs()<1e-7);
                assert!(x||y||z,"unexpected box edge {point:?}");
            }
            1 => { assert!(matches!(curve,Curve::Line { .. })); assert!([[21.,0.,0.],[21.,1.,0.],[20.,1.,0.]].iter().any(|q|(0..3).all(|i|(point[i]-q[i]).abs()<1e-7))); }
            _ => {
                let first=sample(0.); let last=sample(1.);
                let kind=if first[2].abs()<1e-7 && last[2].abs()<1e-7 {0}
                    else if (first[2]-5.).abs()<1e-7 && (last[2]-5.).abs()<1e-7 {1} else {2};
                cylinder_types[kind]+=1;
                for t in [0.,0.25,0.5,0.75,1.] {
                    let p=sample(t); assert!(((p[0]-40.).hypot(p[1])-2.).abs()<1e-7 && p[2]>=-1e-7 && p[2]<=5.+1e-7,"{p:?}");
                    if kind==2 { assert!((p[0]-first[0]).abs()<1e-7 && (p[1]-first[1]).abs()<1e-7); }
                }
            }
        }
    }
    assert_eq!(counts,[12,3,3]); assert_eq!(cylinder_types,[1,1,1]);
    std::fs::remove_file(input_path).unwrap(); std::fs::remove_file(output_path).unwrap();
}

#[test]
#[cfg(feature = "occt-brep")]
fn executable_reopens_native_fit_selection_and_undoes() {
    use std::io::Write;
    use std::process::{Command, Stdio};
    use om9_core::{budget::Limits, geometry::{Geometry, Transform}, model::{Document, Entity}, transaction::Cancellation};
    let limits = Limits::default();
    let source = om9_kernel::brep::box_brep([4., 5., 6.], 0.1, &limits).unwrap();
    let native = om9_kernel::brep::to_3dm(&source, 0.001, &limits).unwrap();
    let mut shape = Entity::new("native box", Geometry::NativeBrep(native), None);
    shape.placement = Transform::translation([10., -3., 7.]);
    let mut document = Document::with_template_tolerance("native fit executable");
    document.entities.insert(shape.id, shape);
    let radius = 19.25_f64.sqrt();
    for point in [[12.+radius,-0.5,10.], [12.-radius,-0.5,10.], [12.,-0.5+radius,10.], [12.,-0.5,10.+radius]] {
        let entity = Entity::new("point", Geometry::Point(point), None);
        document.entities.insert(entity.id, entity);
    }
    let base = std::env::temp_dir().join(format!("om9-mcp-native-fit-{}", uuid::Uuid::new_v4()));
    let input_path = base.with_extension("om9");
    let output_path = base.with_extension("result.om9");
    om9_archive::Archive::save(&input_path, &document, &limits, &Cancellation::default()).unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_om9")).arg("--mcp")
        .stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    let input = tool(1, "om9_open", json!({"path": input_path}))
        + &tool(2, "om9_select", json!({"all": true}))
        + &tool(3, "om9_command", json!({"text": "Sphere"}))
        + &tool(4, "om9_command", json!({"text": "FitPoints"}))
        + &tool(5, "om9_command", json!({"text": "Selection"}))
        + &tool(6, "om9_command", json!({"text": ""}))
        + &tool(7, "om9_undo", json!({}))
        + &tool(8, "om9_redo", json!({}))
        + &tool(9, "om9_save", json!({"path": output_path}));
    child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let replies: Vec<Value> = std::str::from_utf8(&output.stdout).unwrap().lines()
        .map(|line| serde_json::from_str(line).unwrap()).collect();
    assert_eq!(replies.len(), 9);
    for reply in &replies { assert_eq!(ok(reply)["result"]["ok"].as_bool().unwrap_or(true), true, "{reply}"); }
    assert_eq!(ok(&replies[5])["state"]["document"]["objects"], 6);
    assert_eq!(ok(&replies[6])["state"]["document"]["objects"], 5);
    assert_eq!(ok(&replies[7])["state"]["document"]["objects"], 6);
    let saved = om9_archive::Archive::open(&output_path, &limits).unwrap();
    for (id, source) in &document.entities { assert_eq!(saved.entities.get(id), Some(source)); }
    let result = saved.entities.values().find(|entity| !document.entities.contains_key(&entity.id)).unwrap();
    let center = result.placement.point([0.,0.,0.]).unwrap();
    for (actual, expected) in center.into_iter().zip([12., -0.5, 10.]) {
        assert!((actual - expected).abs() < 1e-7, "center {center:?}");
    }
    let Geometry::Brep(sphere) = &result.geometry else { panic!("expected sphere") };
    let bounds = om9_kernel::brep::world_bounds(sphere, &Transform::identity(), &limits).unwrap();
    assert!((bounds[3] - radius).abs() < 1e-7, "radius bounds {bounds:?}");
    std::fs::remove_file(input_path).unwrap();
    std::fs::remove_file(output_path).unwrap();
}

#[test]
fn executable_reopens_certified_box_intersection_and_undoes() {
    use std::io::Write;
    use std::process::{Command, Stdio};
    use om9_core::{budget::Limits, geometry::{Geometry, Transform}, model::{Document, Entity}, transaction::Cancellation};
    use om9_kernel::{creation::{self, Primitive}, transform_ops};
    let limits = Limits::default();
    let sphere = creation::primitive(Primitive::Sphere { radius_mm: 3. }, 0.01, &limits).unwrap();
    let rotation = transform_ops::rotation([0.; 3], [1., 0., 0.], std::f64::consts::FRAC_PI_4).unwrap();
    let sphere = transform_ops::transform_brep(&sphere, &rotation, 0.01, &limits).unwrap();
    let block = creation::primitive(Primitive::Box { x_mm: 2., y_mm: 8., z_mm: 8. }, 0.01, &limits).unwrap();
    let placement = Transform([[2., 0., 0., 0.], [0., 1., 0., -4.], [0., 0., 1., -4.], [0., 0., 0., 1.]]);
    let block = transform_ops::transform_brep(&block, &placement, 0.01, &limits).unwrap();
    let mut doc = Document::new("certified intersection");
    for (name, body) in [("sphere", sphere), ("box", block)] {
        let entity = Entity::new(name, Geometry::Brep(body), None);
        doc.entities.insert(entity.id, entity);
    }
    let base = std::env::temp_dir().join(format!("om9-mcp-intersection-{}", uuid::Uuid::new_v4()));
    let input_path = base.with_extension("om9");
    let output_path = base.with_extension("result.om9");
    om9_archive::Archive::save(&input_path, &doc, &limits, &Cancellation::default()).unwrap();
    if let Some(path) = std::env::var_os("OM9_CERTIFIED_INTERSECTION_DESKTOP_FIXTURE") {
        assert!(!std::path::Path::new(&path).exists());
        om9_archive::Archive::save(std::path::Path::new(&path), &doc, &limits, &Cancellation::default()).unwrap();
    }
    let mut child = Command::new(env!("CARGO_BIN_EXE_om9")).arg("--mcp")
        .stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    let input = tool(1, "om9_open", json!({"path": input_path}))
        + &tool(2, "om9_select", json!({"all": true}))
        + &tool(3, "om9_command", json!({"text": "Boolean2Objects"}))
        + &tool(4, "om9_command", json!({"text": "Intersection"}))
        + &tool(5, "om9_command", json!({"text": ""}))
        + &tool(6, "om9_undo", json!({}))
        + &tool(7, "om9_redo", json!({}))
        + &tool(8, "om9_save", json!({"path": output_path}));
    child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let replies: Vec<Value> = std::str::from_utf8(&output.stdout).unwrap().lines()
        .map(|line| serde_json::from_str(line).unwrap()).collect();
    assert_eq!(replies.len(), 8);
    for reply in &replies { assert_eq!(ok(reply)["result"]["ok"].as_bool().unwrap_or(true), true, "{reply}"); }
    assert_eq!(ok(&replies[4])["state"]["document"]["objects"], 3);
    assert_eq!(ok(&replies[5])["state"]["document"]["objects"], 2);
    assert_eq!(ok(&replies[6])["state"]["document"]["objects"], 3);
    let saved = om9_archive::Archive::open(&output_path, &limits).unwrap();
    for (id, source) in &doc.entities { assert_eq!(saved.entities.get(id), Some(source)); }
    let result = saved.entities.values().find(|entity| !doc.entities.contains_key(&entity.id)).unwrap();
    let Geometry::Brep(body) = &result.geometry else { panic!("expected B-rep") };
    let m = creation::measure(body, &limits).unwrap();
    assert!((m.volume_mm3 - 18. * std::f64::consts::PI).abs() < 1e-5 && m.closed && m.naked_edges == 0, "{m:?}");
    std::fs::remove_file(input_path).unwrap();
    std::fs::remove_file(output_path).unwrap();
}

#[test]
#[ignore = "inspect the result saved by an isolated desktop session"]
fn inspect_certified_intersection_desktop_result() {
    use om9_core::{budget::Limits, geometry::Geometry};
    let path = std::env::var_os("OM9_CERTIFIED_INTERSECTION_DESKTOP_RESULT").expect("desktop result path");
    let limits = Limits::default();
    let doc = om9_archive::Archive::open(std::path::Path::new(&path), &limits).unwrap();
    assert_eq!(doc.entities.len(), 1);
    let Geometry::Brep(body) = &doc.entities.values().next().unwrap().geometry else { panic!("expected B-rep") };
    let m = om9_kernel::creation::measure(body, &limits).unwrap();
    assert!((m.volume_mm3 - 18. * std::f64::consts::PI).abs() < 1e-5 && m.closed && m.solids == 1 && m.naked_edges == 0, "{m:?}");
    println!("saved desktop result volume_mm3={:.12}", m.volume_mm3);
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
