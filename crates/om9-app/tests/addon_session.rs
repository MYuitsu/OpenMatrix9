// SPDX-License-Identifier: LGPL-2.1-or-later
//! Interactive add-on commands through the real controller: start from CMD, menu code and
//! typed alias; command-line sub-options; clicked and typed points; preview without document
//! objects; one transaction on finish; Esc cancels; another command closes the session.
use om9_addon_abi::session::{Interactive, session_json};
use om9_addons::host::{Host, HostConfig, InstallOptions, Startup, build_package};
use om9_addons::manifest::RuntimeKind;
use om9_addons::runtime::{LoadRequest, Runtime};
use om9_app::addons::FeatureName;
use om9_app::session::{Controller, InvocationOrigin};
use om9_core::budget::Limits;
use om9_core::geometry::{Curve, Geometry};
use om9_core::model::Document;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Runs the real add-on package code in-process through the same JSON boundary as the DLL.
struct InProcess;
impl Runtime for InProcess {
    fn load(&mut self, req: &LoadRequest) -> Result<Vec<String>, String> {
        Ok(req.manifest.commands.iter().map(|c| c.id.clone()).collect())
    }
    fn unload(&mut self, _id: &str) {}
    fn session(&mut self, _id: &str, event: &[u8]) -> Result<Vec<u8>, String> {
        let (code, out) = session_json::<om9_matrix::Matrix>(event);
        if code == 0 {
            Ok(out)
        } else {
            Err(String::from_utf8_lossy(&out).into())
        }
    }
}

struct RecordingRuntime(Arc<Mutex<Vec<om9_addon_abi::session::Event>>>);
impl Runtime for RecordingRuntime {
    fn load(&mut self, req: &LoadRequest) -> Result<Vec<String>, String> {
        Ok(req.manifest.commands.iter().map(|c| c.id.clone()).collect())
    }
    fn unload(&mut self, _id: &str) {}
    fn session(&mut self, _id: &str, event: &[u8]) -> Result<Vec<u8>, String> {
        self.0.lock().unwrap().push(serde_json::from_slice(event).map_err(|e| e.to_string())?);
        let (code, out) = session_json::<om9_matrix::Matrix>(event);
        if code == 0 { Ok(out) } else { Err(String::from_utf8_lossy(&out).into()) }
    }
}

struct MixedHistoryReply { delete_parent: bool }
impl Runtime for MixedHistoryReply {
    fn load(&mut self, req: &LoadRequest) -> Result<Vec<String>, String> {
        Ok(req.manifest.commands.iter().map(|c| c.id.clone()).collect())
    }
    fn unload(&mut self, _id: &str) {}
    fn session(&mut self, _id: &str, event: &[u8]) -> Result<Vec<u8>, String> {
        use om9_addon_abi::session::{Event, Geo, RecordUpdate, Reply, TransformOp};
        let event: Event = serde_json::from_slice(event).map_err(|e| e.to_string())?;
        let matrix = [1., 0., 0., 0., 0., 1., 0., 1., 0., 0., 1., 0., 0., 0., 0., 1.];
        let reply = Reply { done: true, create: vec![Geo::Point { point: [8., 2., 0.] }],
            transform: Some(TransformOp { matrices: vec![matrix], keep_original: false, per_object: vec![] }),
            records: vec![RecordUpdate { id: event.selection.first().ok_or("selection")?.id.clone(), record: serde_json::json!({"label": "fixture"}) }],
            delete: if self.delete_parent { vec![event.selection.first().ok_or("selection")?.id.clone()] } else { vec![] },
            ..Default::default() };
        serde_json::to_vec(&reply).map_err(|e| e.to_string())
    }
}

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[cfg(feature = "occt-brep")]
fn calculate_gold(c: &mut Controller) -> om9_core::Result<()> {
    c.set_metal_panel_material(Some(om9_core::metal_weight::DEFAULT_MATERIAL_ID))?;
    c.submit("Calculate")
}

pub fn features() -> Vec<(String, Option<String>)> {
    let v: serde_json::Value =
        serde_json::from_str(include_str!("../assets/features.json")).unwrap();
    v.as_array()
        .unwrap()
        .iter()
        .map(|f| {
            (
                f["id"].as_str().unwrap().to_owned(),
                f["command"].as_str().map(str::to_owned),
            )
        })
        .collect()
}

fn host(tmp: &Path) -> Arc<Mutex<Host>> {
    host_with(tmp, &[])
}

#[test]
fn ring_rail_addon_and_direct_share_sizes_front_placement_and_undo() {
    for (system, size, expected_diameter) in [("vnm", 18., 18.), ("vnm_circumference", 56.548667764616276, 18.), ("us", 7.25, 17.53), ("iso", 56.548667764616276, 18.),
        ("diameter", 18., 18.), ("diameter_inches", 0.75, 19.05), ("circumference_inches", 2.2, 2.2 * 25.4 / std::f64::consts::PI)] {
        let tmp = std::env::temp_dir().join(format!("om9-rail-parity-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&tmp).unwrap();
        let mut addon = Controller::new(Document::with_template_tolerance("rail addon"), Limits::default()).unwrap();
        let feats = features();
        let names: Vec<_> = feats.iter().map(|(id, command)| FeatureName { id, command: command.as_deref() }).collect();
        addon.attach_addons(host(&tmp), &names);
        for icon in ["TopIconRingRail", "BuilderRingRail"] {
            let target = addon.icon_target(icon, None).unwrap();
            assert!(target.addon && target.enabled);
            assert_eq!(target.command, "BuilderRingRail");
            addon.run_addon_command(&target.command).unwrap();
            assert_eq!(addon.addon_session().unwrap().code, "BuilderRingRail");
            assert_eq!(addon.addon_builder().unwrap().value("rr_size_system"), Some(&om9_addons::settings::SettingValue::Str("vnm".into())));
            addon.cancel_addon_session();
            addon.cancel_addon_builder();
            assert!(addon.document().entities.is_empty());
            assert_eq!(addon.coordinator().undo_len(), 0);
        }
        addon.submit("gvRingRail").unwrap();
        assert!(addon.addon_session().is_some());
        assert_eq!(addon.addon_session().unwrap().code, "BuilderRingRail");
        addon.submit(&format!("SizeSystem={system}")).unwrap();
        addon.submit(&format!("RingSize={size}")).unwrap();
        for command in ["CenterX=2", "CenterY=-3", "CenterZ=4"] { addon.submit(command).unwrap(); }
        assert!(addon.document().entities.is_empty());
        assert_eq!(addon.coordinator().undo_len(), 0);
        assert!(!addon.addon_session().unwrap().lines.is_empty());
        addon.submit("").unwrap();
        let addon_geometry = addon.document().entities.values().next().unwrap().geometry.clone();

        let mut direct = Controller::new(Document::with_template_tolerance("rail direct"), Limits::default()).unwrap();
        for icon in ["TopIconRingRail", "BuilderRingRail"] {
            let target = direct.icon_target(icon, None).unwrap();
            assert!(!target.addon && target.enabled);
            direct.dispatch(&target.command, InvocationOrigin::Menu).unwrap();
            assert_eq!(direct.builder().unwrap().kind(), om9_core::builder::BuilderKind::RingRail);
            direct.builder_cancel();
        }
        direct.submit("RingRail").unwrap();
        assert!(direct.builder_is_open());
        direct.submit(&format!("SizeSystem={system}")).unwrap();
        if system == "diameter" { direct.submit(&format!("Diameter={size}")).unwrap(); }
        else { direct.submit(&format!("RingSize={size}")).unwrap(); }
        for command in ["CenterX=2mm", "CenterY=-3mm", "CenterZ=4mm"] { direct.submit(command).unwrap(); }
        let preview = direct.builder().unwrap().preview().unwrap().geometry.clone();
        assert!(direct.document().entities.is_empty());
        assert_eq!(direct.coordinator().undo_len(), 0);
        direct.submit("").unwrap();
        let direct_geometry = &direct.document().entities.values().next().unwrap().geometry;
        assert_eq!(direct_geometry, &preview);
        assert_eq!(direct_geometry, &addon_geometry);
        let Geometry::Curve(Curve::Circle { center, x, y, radius, .. }) = direct_geometry else { panic!("circle") };
        assert_eq!((*center, *x, *y), ([2., -3., 4.], [0., 0., 1.], [1., 0., 0.]));
        assert!((2. * radius - expected_diameter).abs() < 1e-9, "{system} size {size}: got {}, expected {expected_diameter}", 2. * radius);
        for controller in [&mut direct, &mut addon] {
            assert_eq!(controller.coordinator().undo_len(), 1);
            controller.undo().unwrap();
            assert!(controller.document().entities.is_empty());
            controller.redo().unwrap();
            assert_eq!(controller.document().entities.values().next().unwrap().geometry, addon_geometry);
        }
        fs::remove_dir_all(tmp).unwrap();
    }
}

#[test]
fn ring_rail_both_routes_record_f6_history_and_reopen_in_mm_or_inches() {
    use om9_core::{geometry::Unit, transaction::Cancellation};
    for addon_route in [false, true] { for page_unit in [Unit::Millimeter, Unit::Inch] {
        let tmp = std::env::temp_dir().join(format!("om9-rail-edit-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&tmp).unwrap();
        let mut document = Document::with_template_tolerance("rail edit");
        document.page_unit = page_unit;
        let mut controller = Controller::new(document, Limits::default()).unwrap();
        if addon_route {
            let feats = features();
            let names: Vec<_> = feats.iter().map(|(id, command)| FeatureName { id, command: command.as_deref() }).collect();
            controller.attach_addons(host(&tmp), &names);
        }
        controller.submit("gvRingRail").unwrap();
        controller.submit(if addon_route { "CenterX=2" } else { "CenterX=2mm" }).unwrap();
        controller.submit("").unwrap();
        assert_eq!(controller.coordinator().undo_len(), 1);
        assert_eq!(controller.document().entities.len(), 1);
        let object = *controller.document().entities.keys().next().unwrap();
        let recipe = controller.document().history.owner(object).expect("both routes record Ring Rail");
        let source = controller.document().entities[&object].geometry.clone();
        let Geometry::Curve(Curve::Circle { center, radius, .. }) = &source else { panic!("circle") };
        assert!((center[0] * page_unit.factor() - 2.).abs() < 1e-10);
        assert!((radius * 2. * page_unit.factor() - 18.).abs() < 1e-10);

        let path = tmp.join("rail.om9");
        om9_archive::Archive::save(&path, controller.document(), &Limits::default(), &Cancellation::default()).unwrap();
        let document = om9_archive::Archive::open(&path, &Limits::default()).unwrap();
        let mut reopened = Controller::new(document, Limits::default()).unwrap();
        reopened.select_objects(&[object]).unwrap();
        let context = om9_app::f6::context(reopened.document(), &std::collections::BTreeSet::from([object]),
            om9_app::f6::Providers { m04_builder: true, ..Default::default() });
        assert!(context.actions.iter().any(|action| action.id == "EditHistory" && action.availability.enabled()));
        reopened.dispatch("EditHistory", InvocationOrigin::Menu).unwrap();
        assert_eq!(reopened.builder().unwrap().preview().unwrap().geometry, source);
        reopened.submit("Diameter=20mm").unwrap();
        reopened.builder_reset().unwrap();
        assert_eq!(reopened.builder().unwrap().output(), object);
        assert_eq!(reopened.builder().unwrap().preview().unwrap().geometry, source);
        assert_eq!(reopened.coordinator().undo_len(), 0);
        reopened.submit("Diameter=20mm").unwrap();
        reopened.submit("").unwrap();
        assert_eq!(reopened.document().entities.len(), 1);
        assert_eq!(reopened.document().history.owner(object), Some(recipe));
        let Geometry::Curve(Curve::Circle { radius, .. }) = &reopened.document().entities[&object].geometry else { panic!("edited circle") };
        assert!((radius * 2. * page_unit.factor() - 20.).abs() < 1e-10);
        assert_eq!(reopened.coordinator().undo_len(), 1);
        reopened.undo().unwrap();
        assert_eq!(reopened.document().entities[&object].geometry, source);
        reopened.redo().unwrap();
        reopened.dispatch("EditHistory", InvocationOrigin::Menu).unwrap();
        let undo = reopened.coordinator().undo_len();
        reopened.submit("").expect("unchanged Apply keeps matching geometry revision");
        assert_eq!(reopened.document().entities.len(), 1);
        assert_eq!(reopened.coordinator().undo_len(), undo);
        fs::remove_dir_all(tmp).unwrap();
    }}
}

#[test]
fn ring_rail_replace_rebuilds_two_dependents_on_one_guarded_worker() {
    use om9_app::command_options::CommandOptionInterface;
    use om9_core::{builder::BuilderKind, request::Value, units::LengthMm};
    for addon in [false, true] { for scenario in ["commit", "cancel", "stale", "locked"] {
        let tmp = std::env::temp_dir().join(format!("om9-rail-dependents-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&tmp).unwrap();
        let mut c = Controller::new(Document::with_template_tolerance("dependent rail"), Limits::default()).unwrap();
        c.begin_builder(BuilderKind::RingRail).unwrap(); c.builder_apply().unwrap();
        let rail = c.current_ring_rail().unwrap().unwrap();
        let mut outputs = vec![rail];
        for offset in [1., 2.] {
            c.select_objects(&[*outputs.last().unwrap()]).unwrap();
            c.begin_builder(BuilderKind::OutsideRail).unwrap();
            c.builder_set_field("offset_mm", Value::length(LengthMm::try_new(offset).unwrap())).unwrap();
            let output = c.builder().unwrap().output();
            c.builder_apply().unwrap(); outputs.push(output);
        }
        if scenario == "locked" {
            let mut doc = c.document().clone();
            let owner = doc.history.owner(outputs[2]).unwrap();
            doc.history.recipes.get_mut(&owner).unwrap().policy.lock = true;
            c = Controller::new(doc, Limits::default()).unwrap();
        }
        if addon {
            let feats = features(); let names: Vec<_> = feats.iter().map(|(id, command)| FeatureName { id, command: command.as_deref() }).collect();
            c.attach_addons(host(&tmp), &names);
        }
        let before = c.document().clone();
        let undo = c.coordinator().undo_len();
        c.submit("gvRingRail").unwrap();
        c.submit(if addon { "RingSize=20" } else { "Diameter=20mm" }).unwrap();
        c.submit("").unwrap(); c.submit("Yes").unwrap();
        assert!(c.join_history_worker_pending());
        assert!(!c.builder_editable());
        assert!(c.prompt().contains("updating dependent features"));
        assert!(c.command_options().is_empty());
        assert!(c.builder_reset().is_err());
        assert!(c.switch_builder(BuilderKind::OutsideRail).is_err());
        assert!(c.builder_set_field("diameter_mm", Value::length(LengthMm::try_new(22.).unwrap())).is_err());
        assert_eq!(c.document(), &before);
        assert_eq!(c.coordinator().undo_len(), undo);
        if scenario == "cancel" {
            c.cancel();
            assert!(!c.join_history_worker_pending());
        } else {
            if scenario == "stale" { c.select_objects(&[rail]).unwrap(); }
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
            let result = loop {
                if let Some(result) = c.poll_join_history_worker() { break result; }
                assert!(std::time::Instant::now() < deadline, "Ring Rail worker timeout");
                std::thread::sleep(std::time::Duration::from_millis(1));
            };
            if scenario == "commit" { result.unwrap(); } else { assert!(result.is_err(), "{scenario}"); }
        }
        if scenario == "commit" {
            let after = c.document().clone();
            for (id, expected) in outputs.iter().zip([10., 11., 13.]) {
                let Geometry::Curve(Curve::Circle { radius, .. }) = &after.entities[id].geometry else { panic!("dependent circle") };
                assert!((radius - expected).abs() < 1e-10, "{addon}/{id}: {radius}");
                assert_eq!(after.history.owner(*id), before.history.owner(*id));
                let object = c.scene().unwrap().objects.iter().find(|object| object.root == *id).unwrap();
                assert_eq!(object.source, after.stamp(*id).unwrap());
            }
            assert_eq!(c.coordinator().undo_len(), undo + 1);
            c.undo().unwrap(); assert_eq!(c.document(), &before);
            c.redo().unwrap(); assert_eq!(c.document(), &after);
            let path = tmp.join("dependent-rails.om9");
            om9_archive::Archive::save(&path, &after, &Limits::default(), &om9_core::transaction::Cancellation::default()).unwrap();
            let reopened = om9_archive::Archive::open(&path, &Limits::default()).unwrap();
            assert_eq!(reopened.entities, after.entities);
            assert_eq!(reopened.history, after.history);
        } else {
            assert_eq!(c.document(), &before, "{addon}/{scenario}");
            assert_eq!(c.coordinator().undo_len(), undo);
        }
        fs::remove_dir_all(tmp).unwrap();
    }}
}

#[test]
fn ring_rail_replace_yes_no_cancel_are_shared_and_atomic() {
    use om9_app::{command_options::CommandOptionInterface, session::ring_rail::ReplaceChoice};
    for addon in [false, true] { for choice in [ReplaceChoice::Yes, ReplaceChoice::No, ReplaceChoice::Cancel] {
        let tmp = std::env::temp_dir().join(format!("om9-rail-replace-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&tmp).unwrap();
        let mut c = Controller::new(Document::with_template_tolerance("rail replace"), Limits::default()).unwrap();
        if addon {
            let feats = features();
            let names: Vec<_> = feats.iter().map(|(id, command)| FeatureName { id, command: command.as_deref() }).collect();
            c.attach_addons(host(&tmp), &names);
        }
        c.submit("RingRail").unwrap(); c.submit("").unwrap();
        let first = c.current_ring_rail().unwrap().unwrap();
        let before = c.document().clone();
        let undo = c.coordinator().undo_len();
        c.submit("gvRingRail").unwrap();
        c.submit(if addon { "RingSize=20" } else { "Diameter=20mm" }).unwrap();
        c.submit("").unwrap();
        assert!(c.ring_rail_confirmation_pending());
        assert_eq!(c.document(), &before);
        assert_eq!(c.coordinator().undo_len(), undo);
        let options = c.command_options();
        assert_eq!(options.iter().map(|option| option.id.name()).collect::<Vec<_>>(), ["Yes", "No", "Cancel"]);
        assert_eq!((options[0].shortcut, options[1].shortcut), (Some('Y'), Some('N')));
        let old_yes = options[0].id;
        c.submit(match choice { ReplaceChoice::Yes => "Yes", ReplaceChoice::No => "No", ReplaceChoice::Cancel => "Cancel" }).unwrap();
        assert!(!c.ring_rail_confirmation_pending());
        assert!(c.activate_command_option(old_yes).is_err());
        if choice == ReplaceChoice::Cancel {
            assert_eq!(c.document(), &before);
            assert_eq!(c.coordinator().undo_len(), undo);
        } else {
            let current = c.current_ring_rail().unwrap().unwrap();
            assert_eq!(c.coordinator().undo_len(), undo + 1);
            assert_eq!(c.document().entities.len(), if choice == ReplaceChoice::Yes { 1 } else { 2 });
            let Geometry::Curve(Curve::Circle { radius, .. }) = c.document().entities[&current].geometry else { panic!("circle") };
            assert!((radius * 2. - 20.).abs() < 1e-10);
            if choice == ReplaceChoice::Yes {
                assert_eq!(current, first);
                assert_eq!(c.document().history.owner(current), before.history.owner(first));
                assert_eq!(c.document().entities[&current].attributes, before.entities[&first].attributes);
            } else {
                assert_ne!(current, first);
                assert_eq!(c.document().entities[&first], before.entities[&first]);
            }
            c.undo().unwrap();
            assert_eq!(c.current_ring_rail().unwrap(), Some(first));
            assert_eq!(c.document().entities, before.entities);
            assert_eq!(c.document().resources, before.resources);
            c.redo().unwrap();
            assert_eq!(c.current_ring_rail().unwrap(), Some(current));
            let path = tmp.join("multiple-rails.om9");
            om9_archive::Archive::save(&path, c.document(), &Limits::default(), &om9_core::transaction::Cancellation::default()).unwrap();
            let cold = Controller::new(om9_archive::Archive::open(&path, &Limits::default()).unwrap(), Limits::default()).unwrap();
            assert_eq!(cold.current_ring_rail().unwrap(), Some(current));
        }
        fs::remove_dir_all(tmp).unwrap();
    }}
}

#[test]
fn ring_rail_confirmation_rejects_changed_parameters_document_and_old_options() {
    use om9_app::{command_options::CommandOptionInterface, session::ring_rail::ReplaceChoice};
    for addon in [false, true] {
        let tmp = std::env::temp_dir().join(format!("om9-rail-stale-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&tmp).unwrap();
        let mut c = Controller::new(Document::with_template_tolerance("rail stale"), Limits::default()).unwrap();
        if addon {
            let feats = features(); let names: Vec<_> = feats.iter().map(|(id, command)| FeatureName { id, command: command.as_deref() }).collect();
            c.attach_addons(host(&tmp), &names);
        }
        c.submit("RingRail").unwrap(); c.submit("").unwrap();
        let before = c.document().clone();
        c.submit("RingRail").unwrap(); c.submit("").unwrap();
        let old_yes = c.command_options()[0].id;
        if addon { c.set_builder_text("rr_ring_size", "21").unwrap(); }
        else { c.builder_set_field("diameter_mm", om9_core::request::Value::length(om9_core::units::LengthMm::try_new(21.).unwrap())).unwrap(); }
        assert!(matches!(c.decide_ring_rail_replace(ReplaceChoice::Yes), Err(om9_core::Error::Stale(_))));
        assert_eq!(c.document(), &before);
        c.submit("").unwrap();
        assert!(c.ring_rail_confirmation_pending());
        assert!(c.activate_command_option(old_yes).is_err());
        c.submit("No").unwrap();
        assert_eq!(c.document().entities.len(), 2);
        c.submit("RingRail").unwrap(); c.submit("").unwrap();
        c.store_f6_state(b"{}").unwrap();
        let changed = c.document().clone();
        assert!(matches!(c.decide_ring_rail_replace(ReplaceChoice::Yes), Err(om9_core::Error::Stale(_))));
        assert_eq!(c.document(), &changed);
        c.cancel();
        c.submit("Line").unwrap();
        let active = (c.is_running(), c.addon_session().map(|session| session.code.clone()));
        assert!(active.0 || active.1.is_some());
        assert!(c.decide_ring_rail_replace(ReplaceChoice::Cancel).is_err());
        assert_eq!((c.is_running(), c.addon_session().map(|session| session.code.clone())), active, "a late Cancel must not cancel another command");
        c.cancel();
        fs::remove_dir_all(tmp).unwrap();
    }
}

#[test]
fn ring_rail_replacement_resets_pose_and_handles_unrecorded_or_locked_sources() {
    use om9_app::session::ring_rail::ReplaceChoice;
    for record in [false, true] {
        let mut c = Controller::new(Document::with_template_tolerance("rail without history"), Limits::default()).unwrap();
        c.set_history_record(false).unwrap();
        c.submit("RingRail").unwrap(); c.submit("").unwrap();
        let object = c.current_ring_rail().unwrap().unwrap();
        assert!(c.document().history.owner(object).is_none());
        let mut placed = c.document().clone();
        let entity = placed.entities.get_mut(&object).unwrap();
        entity.placement = om9_core::geometry::Transform::translation([10., 0., 0.]);
        entity.geometry_revision += 1;
        let mut c = Controller::new(placed, Limits::default()).unwrap();
        c.set_history_record(record).unwrap();
        let before = c.document().clone();
        c.submit("RingRail").unwrap(); c.submit("Diameter=20mm").unwrap(); c.submit("").unwrap();
        c.decide_ring_rail_replace(ReplaceChoice::Yes).unwrap();
        assert_eq!(c.document().entities.len(), 1);
        assert_eq!(c.document().entities[&object].placement, om9_core::geometry::Transform::identity());
        assert_eq!(c.document().history.owner(object).is_some(), record);
        assert_eq!(c.coordinator().undo_len(), 1);
        c.undo().unwrap();
        assert_eq!(c.document().entities, before.entities);
        c.redo().unwrap();
        let mut locked = c.document().clone();
        locked.entities.get_mut(&object).unwrap().attributes.locked = true;
        let mut c = Controller::new(locked, Limits::default()).unwrap();
        let before = c.document().clone();
        c.submit("RingRail").unwrap(); c.submit("").unwrap();
        assert!(c.ring_rail_replace_unavailable().is_some());
        assert!(c.decide_ring_rail_replace(ReplaceChoice::Yes).is_err());
        assert_eq!(c.document(), &before);
        c.decide_ring_rail_replace(ReplaceChoice::No).unwrap();
        assert_eq!(c.document().entities.len(), 2);
        assert_eq!(c.document().entities[&object], before.entities[&object]);
    }
}

#[cfg(feature = "occt-brep")]
#[test]
fn metal_weights_addon_icon_and_alias_use_the_same_brep_measurement() {
    let tmp = std::env::temp_dir().join(format!("om9-metal-route-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&tmp).unwrap();
    let addon = host(&tmp);
    let feats = features();
    let names: Vec<FeatureName> = feats.iter().map(|(id, command)| FeatureName {
        id, command: command.as_deref(),
    }).collect();
    let mut c = Controller::new(Document::with_template_tolerance("metal route"), Limits::default()).unwrap();
    c.submit("Box 10 20 30").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    c.submit("Point").unwrap();
    c.submit("40,0,0").unwrap();
    let point = c.document().entities.values().find(|entity| matches!(entity.geometry, Geometry::Point(_))).unwrap().id;
    c.select_objects(&[id]).unwrap();
    let before = c.document().clone();
    let undo = c.coordinator().undo_len();
    c.attach_addons(addon.clone(), &names);
    let target = c.icon_target("BuilderMetalWeights", None).unwrap();
    assert!(target.addon && target.enabled, "target={target:?}; binding={:?}", c.addon_code("BuilderMetalWeights"));
    c.run_addon_command(&target.command).unwrap();
    assert!(c.metal_report().is_none());
    calculate_gold(&mut c).unwrap();
    let icon_report = c.metal_report().expect("source-based report").clone();
    assert!((icon_report.volume_mm3 - 6000.0).abs() < 1e-6);
    assert!((icon_report.grams - 93.0).abs() < 1e-6);
    assert!((icon_report.dwt - 93.0 / 1.555_173_84).abs() < 1e-6);
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), undo);
    c.request_viewport_detail(om9_app::display::ViewportDetail::High).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while c.viewport_detail_pending() {
        if let Some(result) = c.poll_viewport_detail() { result.unwrap(); }
        assert!(std::time::Instant::now() < deadline, "viewport detail timed out");
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    c.run_addon_command(&target.command).unwrap();
    calculate_gold(&mut c).unwrap();
    assert_eq!(c.metal_report(), Some(&icon_report));
    c.submit("gvMetalWeights").unwrap();
    calculate_gold(&mut c).unwrap();
    assert_eq!(c.metal_report(), Some(&icon_report));
    c.submit("gvMetalWeights all").unwrap();
    assert_eq!(c.metal_catalog_report().unwrap().estimates.len(), c.metal_registry().len());
    c.submit("MetalWeights material=platinum_950").unwrap();
    assert!((c.metal_report().unwrap().grams - 124.8).abs() < 1e-6);
    c.submit("gvMetalWeights").unwrap();
    calculate_gold(&mut c).unwrap();
    c.select_objects(&[point]).unwrap();
    c.run_addon_command(&target.command).unwrap();
    assert!(calculate_gold(&mut c).is_err());
    assert_eq!(c.metal_report(), Some(&icon_report));
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), undo);
    c.select_objects(&[id]).unwrap();
    addon.lock().unwrap().disable("om9_matrix").unwrap();
    c.refresh_addon_names(&names);
    let fallback = c.icon_target("BuilderMetalWeights", None).unwrap();
    assert!(!fallback.addon && fallback.enabled);
    c.dispatch(&fallback.command, InvocationOrigin::Menu).unwrap();
    calculate_gold(&mut c).unwrap();
    assert_eq!(c.metal_report(), Some(&icon_report));
    c.submit("gvMetalWeights").unwrap();
    calculate_gold(&mut c).unwrap();
    assert_eq!(c.metal_report(), Some(&icon_report));
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), undo);
}

#[cfg(feature = "occt-brep")]
#[test]
fn metal_weights_addon_route_refuses_open_brep_and_keeps_previous_report() {
    use om9_core::{budget::{Usage, memory::MemoryClass}, model::Entity};
    let tmp = std::env::temp_dir().join(format!("om9-metal-open-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&tmp).unwrap();
    let feats = features();
    let names: Vec<FeatureName> = feats.iter().map(|(id, command)| FeatureName {
        id, command: command.as_deref(),
    }).collect();
    let mut c = Controller::new(Document::with_template_tolerance("open metal"), Limits::default()).unwrap();
    c.submit("Box 10 20 30").unwrap();
    let box_id = *c.document().entities.keys().next().unwrap();
    c.select_objects(&[box_id]).unwrap();
    c.attach_addons(host(&tmp), &names);
    c.run_addon_command("BuilderMetalWeights").unwrap();
    calculate_gold(&mut c).unwrap();
    let prior = c.metal_report().unwrap().clone();
    let shell = om9_kernel::creation::extrude_wire(
        &[[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [10.0, 2.0, 0.0]],
        [0.0, 0.0, 2.0], om9_kernel::creation::DEFAULT_DEFLECTION_MM, &Limits::default(),
    ).unwrap();
    let bytes = shell.storage_usage(&Limits::default()).unwrap().bytes;
    let mut reservation = c.coordinator().reserve_external_memory(MemoryClass::Kernel, bytes, None).unwrap();
    reservation.set_allocated_bytes(bytes).unwrap();
    let source = c.coordinator().register_reserved_brep_candidate(shell, &mut reservation, Usage::default()).unwrap();
    let entity = Entity::new("open shell", Geometry::Brep(source), None);
    let open_id = entity.id;
    c.commit_entity_reserved_brep("test", "open shell", entity, &mut reservation).unwrap();
    c.select_objects(&[open_id]).unwrap();
    let before = c.document().clone();
    let undo = c.coordinator().undo_len();
    c.run_addon_command("BuilderMetalWeights").unwrap();
    assert!(calculate_gold(&mut c).is_err());
    assert_eq!(c.metal_report(), Some(&prior));
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), undo);
}

#[cfg(all(feature = "occt-brep", feature = "test-fixtures"))]
#[test]
fn metal_weights_enabled_addon_routes_a_selected_brep_instance_to_source() {
    use om9_core::model::{Definition, Entity};
    let tmp = std::env::temp_dir().join(format!("om9-metal-instance-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&tmp).unwrap();
    let addon = host(&tmp);
    let feats = features();
    let names: Vec<FeatureName> = feats.iter().map(|(id, command)| FeatureName {
        id, command: command.as_deref(),
    }).collect();
    let mut c = Controller::new(Document::with_template_tolerance("metal instance"), Limits::default()).unwrap();
    c.submit("Box 10 5 2").unwrap();
    let source = c.document().entities.values().next().unwrap().geometry.clone();
    let definition = Definition::new("solid", vec![Entity::new("member", source, None)]);
    let root = Entity::new("instance", Geometry::Instance(definition.id), None);
    let id = root.id;
    c.test_insert_metal_definitions(vec![definition], vec![root]).unwrap();
    c.select_objects(&[id]).unwrap();
    let before = c.document().clone();
    let undo = c.coordinator().undo_len();
    c.attach_addons(addon, &names);
    let target = c.icon_target("BuilderMetalWeights", None).unwrap();
    assert!(target.addon && target.enabled);
    c.run_addon_command(&target.command).unwrap();
    assert!(c.metal_report().is_none());
    calculate_gold(&mut c).unwrap();
    let report = c.metal_report().unwrap().clone();
    assert!((report.volume_mm3 - 100.).abs() < 1e-6);
    c.submit("gvMetalWeights").unwrap();
    calculate_gold(&mut c).unwrap();
    assert_eq!(c.metal_report(), Some(&report));
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), undo);
}

#[cfg(all(feature = "occt-brep", windows))]
fn native_matrix_host(tmp: &Path) -> Arc<Mutex<Host>> {
    let dll = PathBuf::from(std::env::var_os("OM9_TEST_MATRIX_DLL").expect("native Matrix DLL"));
    let stage = tmp.join("stage");
    fs::create_dir_all(stage.join("icons")).unwrap();
    fs::create_dir_all(stage.join("lang")).unwrap();
    let source = repo().join("addons/matrix/addon");
    let manifest = fs::read_to_string(source.join("om9_addon.toml")).unwrap()
        .replace("@PLATFORM@", "windows-x64").replace("@ENTRY@", "om9_matrix.dll");
    fs::write(stage.join("om9_addon.toml"), manifest).unwrap();
    fs::copy(dll, stage.join("om9_matrix.dll")).unwrap();
    for dir in ["icons", "lang"] {
        for entry in fs::read_dir(source.join(dir)).unwrap() {
            let path = entry.unwrap().path();
            fs::copy(&path, stage.join(dir).join(path.file_name().unwrap())).unwrap();
        }
    }
    let catalog = om9_app::addons::catalog().unwrap();
    let (_, zip) = build_package(&stage, Some(&catalog)).unwrap();
    let package = tmp.join("matrix.zip");
    fs::write(&package, zip).unwrap();
    let mut native = om9_app::addons::new_host(HostConfig::new(
        om9_app::addons::om9_version(), catalog, tmp.join("config"),
    ));
    native.startup(&Startup::default());
    let installed = native.install_zip(&package, &InstallOptions {
        enable: true, grant_permissions: true, ..Default::default()
    }).unwrap();
    assert!(installed.enabled, "{:?}", installed.warnings);
    Arc::new(Mutex::new(native))
}

#[cfg(all(feature = "occt-brep", windows))]
#[test]
#[ignore = "requires OM9_TEST_MATRIX_DLL pointing to the freshly built native add-on"]
fn metal_weights_with_installed_native_matrix_package_uses_brep_source() {
    let tmp = std::env::temp_dir().join(format!("om9-metal-native-{}", uuid::Uuid::new_v4()));
    let native = native_matrix_host(&tmp);
    let feats = features();
    let names: Vec<FeatureName> = feats.iter().map(|(id, command)| FeatureName {
        id, command: command.as_deref(),
    }).collect();
    let mut document = Document::with_template_tolerance("native metal");
    #[cfg(feature = "test-fixtures")]
    let render = {
        let id = om9_core::identity::MaterialId::new();
        document.exchange.materials.insert(id, om9_core::exchange::RenderMaterial {
            id, name: "native render".into(), diffuse: [128; 4],
        });
        id
    };
    let mut c = Controller::new(document, Limits::default()).unwrap();
    c.submit("Box 10 20 30").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    c.select_objects(&[id]).unwrap();
    let before = c.document().clone();
    let undo = c.coordinator().undo_len();
    c.attach_addons(native, &names);
    let target = c.icon_target("BuilderMetalWeights", None).unwrap();
    assert!(target.addon && target.enabled, "target={target:?}; binding={:?}; rows={:?}",
        c.addon_code("BuilderMetalWeights"), c.addon_host().unwrap().lock().unwrap().list().iter()
            .map(|row| (&row.id, &row.status, row.commands.len())).collect::<Vec<_>>());
    c.run_addon_command(&target.command).unwrap();
    assert!(c.metal_report().is_none());
    calculate_gold(&mut c).unwrap();
    let report = c.metal_report().unwrap().clone();
    assert!((report.volume_mm3 - 6000.0).abs() < 1e-6);
    assert!((report.grams - 93.0).abs() < 1e-6);
    c.submit("gvMetalWeights").unwrap();
    calculate_gold(&mut c).unwrap();
    assert_eq!(c.metal_report(), Some(&report));
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), undo);
    #[cfg(feature = "test-fixtures")]
    {
        use om9_core::model::{Definition, Entity};
        let source = c.document().entities[&id].geometry.clone();
        let mut member = Entity::new("member", source, None);
        member.attributes.material_source = 1;
        member.attributes.material = Some(render);
        let definition = Definition::new("native solid", vec![member]);
        let instance = Entity::new("instance", Geometry::Instance(definition.id), None);
        let instance_id = instance.id;
        c.test_insert_metal_definitions(vec![definition], vec![instance]).unwrap();
        c.select_objects(&[instance_id]).unwrap();
        let before_instance = c.document().clone();
        let undo_instance = c.coordinator().undo_len();
        c.run_addon_command(&target.command).unwrap();
        calculate_gold(&mut c).unwrap();
        let instance_report = c.metal_report().unwrap().clone();
        assert!((instance_report.volume_mm3 - 6000.0).abs() < 1e-6);
        c.submit("gvMetalWeights").unwrap();
        calculate_gold(&mut c).unwrap();
        assert_eq!(c.metal_report(), Some(&instance_report));
        c.bind_metal_material(render, Some("platinum_950")).unwrap();
        c.set_metal_panel_object_materials(true);
        c.submit("Calculate").unwrap();
        assert!((c.metal_report().unwrap().grams - 124.8).abs() < 1e-6);
        c.enable_background_metal_weights();
        c.submit("Calculate").unwrap();
        assert!(c.metal_worker_pending());
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            if let Some(result) = c.poll_metal_worker() { result.unwrap(); break; }
            assert!(std::time::Instant::now() < deadline, "native package metal worker timeout");
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!((c.metal_report().unwrap().grams - 124.8).abs() < 1e-6);
        assert_eq!(c.document(), &before_instance);
        assert_eq!(c.coordinator().undo_len(), undo_instance);
    }
}

#[cfg(all(feature = "occt-brep", windows))]
#[test]
#[ignore = "requires OM9_TEST_MATRIX_DLL pointing to the freshly built native add-on"]
fn native_addon_polyline_commits_viewport_end_snap_preview() {
    use eframe::egui::{self, Event, Key, Modifiers, PointerButton, Pos2, Rect, Vec2};
    use om9_app::ui::MatrixApp;
    use om9_core::model::Entity;
    use om9_scene::{camera::Viewport, osnap::SnapKind, workspace::ViewId};

    let tmp = std::env::temp_dir().join(format!("om9-native-polyline-{}", uuid::Uuid::new_v4()));
    let host = native_matrix_host(&tmp);
    let features = features();
    let names: Vec<_> = features.iter().map(|(id, command)| FeatureName {
        id,
        command: command.as_deref(),
    }).collect();
    let start = [-8., 0., 0.];
    let end = [8., 4., 0.];
    let elevated_end = [3., -5., 9.];
    let mut document = Document::with_template_tolerance("native add-on viewport End");
    for (a, b) in [
        (start, [-8., 8., 0.]),
        (end, [12., 8., 0.]),
        (elevated_end, [6., -6., 9.]),
    ] {
        let line = Entity::new("End reference", Geometry::Curve(Curve::Line {
            start: a,
            end: b,
            domain: [0., 1.],
        }), None);
        document.entities.insert(line.id, line);
    }
    let mut app = MatrixApp::new(document).unwrap();
    assert!(host.lock().unwrap().bound_commands().iter().any(|(code, _, _)| code == "CurvePolylinePolyline"));
    app.controller.attach_addons(host, &names);
    let before = app.controller.document().clone();
    let undo = app.controller.coordinator().undo_len();
    let ctx = egui::Context::default();
    let frame = |app: &mut MatrixApp, events| {
        let mut output = ctx.run_ui(egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 900.))),
            events,
            ..Default::default()
        }, |ui| app.show(ui));
        output.textures_delta.clear();
    };
    let click = |app: &mut MatrixApp, pos| {
        frame(app, vec![Event::PointerMoved(pos)]);
        for pressed in [true, false] {
            frame(app, vec![Event::PointerButton {
                pos, button: PointerButton::Primary, pressed, modifiers: Modifiers::NONE,
            }]);
        }
    };
    let key = |key| Event::Key {
        key, physical_key: None, pressed: true, repeat: false, modifiers: Modifiers::NONE,
    };
    frame(&mut app, vec![]);
    frame(&mut app, vec![]);
    assert_eq!(app.controller.addon_code("CurvePolylinePolyline").as_deref(), Some("CurvePolylinePolyline"));
    let input = app.command_input_rect().unwrap().center();
    click(&mut app, input);
    frame(&mut app, vec![Event::Text("CurvePolylinePolyline".into())]);
    assert_eq!(app.transcript.draft(), "CurvePolylinePolyline");
    frame(&mut app, vec![key(Key::Enter)]);
    assert_eq!(app.controller.addon_session().map(|session| session.code.as_str()),
        Some("CurvePolylinePolyline"), "{}: {}", app.status_line(), app.transcript.draft());
    app.controller.workspace.active = ViewId::LookingDown;
    let rect = app.view_rects[&ViewId::LookingDown];
    let viewport = Viewport {
        origin: [rect.min.x as f64, rect.min.y as f64],
        size: [rect.width() as f64, rect.height() as f64],
    };
    let camera = app.controller.workspace.views[&ViewId::LookingDown].camera.clone();
    let hover_end = |app: &mut MatrixApp, expected: [f64; 3]| {
        let near = [expected[0] + 0.2, expected[1], expected[2]];
        let [x, y] = camera.project(near, viewport).unwrap().unwrap().screen;
        let at = Pos2::new(x as f32, y as f32);
        frame(app, vec![Event::PointerMoved(at)]);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        loop {
            if let Some(candidate) = app.snap_candidate().filter(|hit| hit.kind == SnapKind::End) {
                assert_eq!(candidate.world, expected);
                break;
            }
            assert!(std::time::Instant::now() < deadline, "{}", app.status_line());
            std::thread::sleep(std::time::Duration::from_millis(5));
            frame(app, vec![]);
        }
        at
    };

    let at = hover_end(&mut app, start);
    click(&mut app, at);
    assert_eq!(app.controller.addon_session().unwrap().last_point, Some(start));
    app.set_snap_enabled(SnapKind::End, false);
    frame(&mut app, vec![key(Key::E), Event::Text("e".into())]);
    assert_eq!(app.snap_override(), Some(SnapKind::End), "{}", app.transcript.draft());
    let at = hover_end(&mut app, end);
    assert!(app.controller.addon_session().unwrap().hover_lines.contains(&[start, end]));
    click(&mut app, at);
    assert_eq!(app.controller.addon_session().unwrap().last_point, Some(end));
    assert_eq!(app.snap_override(), None);
    assert_eq!(app.controller.document(), &before);
    assert_eq!(app.controller.coordinator().undo_len(), undo);

    click(&mut app, input);
    frame(&mut app, vec![key(Key::Enter)]);
    assert!(app.controller.addon_session().is_none());
    let committed = app.controller.document().clone();
    assert_eq!(committed.entities.len(), before.entities.len() + 1);
    let created = committed.entities.values().find(|entity| !before.entities.contains_key(&entity.id)).unwrap();
    let Geometry::Curve(Curve::Line { start: committed_start, end: committed_end, .. }) = &created.geometry else {
        panic!("expected the two-point polyline to commit a line");
    };
    assert_eq!([*committed_start, *committed_end], [start, end]);
    assert_eq!(app.controller.coordinator().undo_len(), undo + 1);
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
    app.controller.redo().unwrap();
    assert_eq!(app.controller.document(), &committed);

    app.controller.undo().unwrap();
    let submit = |app: &mut MatrixApp, text: &str| {
        app.focus_command_editor();
        frame(app, vec![]);
        frame(app, vec![Event::Text(text.into())]);
        assert_eq!(app.transcript.draft(), text);
        frame(app, vec![key(Key::Enter)]);
    };
    submit(&mut app, "CurvePolylinePolyline");
    app.set_snap_enabled(SnapKind::End, true);
    let at = hover_end(&mut app, start);
    click(&mut app, at);
    hover_end(&mut app, end);
    assert!(app.controller.addon_session().unwrap().hover_lines.contains(&[start, end]));
    submit(&mut app, "Between");
    assert!(app.transcript.prompt().starts_with("Between: first point"));
    assert!(app.controller.addon_session().unwrap().hover_lines.is_empty(),
        "Between must discard the previous endpoint preview");

    let first = [-8., 8., 0.];
    let midpoint = [0., 6., 0.];
    app.set_snap_enabled(SnapKind::End, false);
    submit(&mut app, "End");
    assert_eq!(app.snap_override(), Some(SnapKind::End), "{}: {}", app.status_line(), app.transcript.draft());
    let at = hover_end(&mut app, first);
    click(&mut app, at);
    assert!(app.transcript.prompt().starts_with("Between: second point"));
    assert_eq!(app.controller.addon_session().unwrap().last_point, Some(start));
    frame(&mut app, vec![key(Key::E), Event::Text("e".into())]);
    assert_eq!(app.snap_override(), Some(SnapKind::End));
    let at = hover_end(&mut app, end);
    let preview = &app.controller.addon_session().unwrap().hover_lines;
    assert!(preview.contains(&[start, midpoint]));
    assert!(!preview.contains(&[start, end]));
    click(&mut app, at);
    assert_eq!(app.controller.addon_session().unwrap().last_point, Some(midpoint));
    assert_eq!(app.snap_override(), None);
    assert_eq!(app.controller.document(), &before);
    assert_eq!(app.controller.coordinator().undo_len(), undo);

    click(&mut app, input);
    frame(&mut app, vec![key(Key::Enter)]);
    assert!(app.controller.addon_session().is_none());
    let document = app.controller.document();
    assert_eq!(document.entities.len(), before.entities.len() + 1);
    let created = document.entities.values().find(|entity| !before.entities.contains_key(&entity.id)).unwrap();
    assert!(matches!(&created.geometry, Geometry::Curve(Curve::Line { start: a, end: b, .. }) if *a == start && *b == midpoint));
    assert_eq!(app.controller.coordinator().undo_len(), undo + 1);
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);

    let typed_start = [-4., -8., 3.];
    app.controller.set_project(true);
    app.set_snap_enabled(SnapKind::End, true);
    for (planar, expected_end) in [(false, [3., -5., 0.]), (true, [3., -5., 3.])] {
        app.controller.set_planar(planar);
        submit(&mut app, "CurvePolylinePolyline");
        submit(&mut app, "-4,-8,3");
        assert_eq!(app.controller.addon_session().unwrap().last_point, Some(typed_start));

        let at = hover_end(&mut app, elevated_end);
        let preview = &app.controller.addon_session().unwrap().hover_lines;
        assert!(preview.contains(&[typed_start, expected_end]), "Planar {planar}: {preview:?}");
        click(&mut app, at);
        assert_eq!(app.controller.addon_session().unwrap().last_point, Some(expected_end));
        assert_eq!(app.controller.document(), &before);
        assert_eq!(app.controller.coordinator().undo_len(), undo);

        click(&mut app, input);
        frame(&mut app, vec![key(Key::Enter)]);
        assert!(app.controller.addon_session().is_none());
        let document = app.controller.document();
        assert_eq!(document.entities.len(), before.entities.len() + 1);
        assert!(document.entities.values().any(|entity| {
            matches!(&entity.geometry, Geometry::Curve(Curve::Line { start, end, .. })
                if *start == typed_start && *end == expected_end)
        }));
        assert_eq!(app.controller.coordinator().undo_len(), undo + 1);
        app.controller.undo().unwrap();
        assert_eq!(app.controller.document(), &before);
    }
}

/// A host with the OM9 Matrix package installed and enabled (all menus in one add-on), in
/// English: these tests check the command texts as written.
fn host_with(tmp: &Path, _menus: &[&str]) -> Arc<Mutex<Host>> {
    host_lang(tmp, "en")
}

/// The same host in the given interface language (`vi` is the package default).
fn host_lang(tmp: &Path, lang: &str) -> Arc<Mutex<Host>> {
    let catalog = om9_app::addons::catalog().unwrap();
    let mut h = Host::new(HostConfig::new(
        om9_app::addons::om9_version(),
        catalog.clone(),
        tmp.join("config"),
    ));
    h.register_runtime(RuntimeKind::Python, Box::new(InProcess));
    h.startup(&Startup::default());
    let zip = stage_zip(tmp, &catalog);
    let r = h
        .install_zip(
            &zip,
            &InstallOptions {
                enable: true,
                ..Default::default()
            },
        )
        .unwrap();
    assert!(r.enabled, "{:?}", r.warnings);
    h.set_setting("om9_matrix", "language", lang).unwrap();
    Arc::new(Mutex::new(h))
}

fn stage_zip(tmp: &Path, catalog: &om9_addons::catalog::Catalog) -> PathBuf {
    let stage = tmp.join("stage-matrix");
    fs::create_dir_all(stage.join("icons")).unwrap();
    let addon = repo().join("addons/matrix/addon");
    let manifest = fs::read_to_string(addon.join("om9_addon.toml"))
        .unwrap()
        .replace("@PLATFORM@", "any")
        .replace("runtime = \"native\"", "runtime = \"python\"")
        .replace("@ENTRY@", "entry.bin");
    fs::write(stage.join("om9_addon.toml"), manifest).unwrap();
    fs::write(stage.join("entry.bin"), b"in-process").unwrap();
    for e in fs::read_dir(addon.join("icons")).unwrap() {
        let e = e.unwrap().path();
        fs::copy(&e, stage.join("icons").join(e.file_name().unwrap())).unwrap();
    }
    // The interface tables (lang/vi.toml) travel with the package.
    fs::create_dir_all(stage.join("lang")).unwrap();
    for e in fs::read_dir(addon.join("lang")).unwrap() {
        let e = e.unwrap().path();
        fs::copy(&e, stage.join("lang").join(e.file_name().unwrap())).unwrap();
    }
    let (_, zip) = build_package(&stage, Some(catalog)).unwrap_or_else(|e| panic!("{e:?}"));
    let path = tmp.join("matrix.zip");
    fs::write(&path, zip).unwrap();
    path
}

fn tools_host(tmp: &Path) -> Arc<Mutex<Host>> {
    let catalog = om9_app::addons::catalog().unwrap();
    let stage = tmp.join("stage-tools");
    fs::create_dir_all(&stage).unwrap();
    let manifest = fs::read_to_string(repo().join("addons/tools/addon/om9_addon.toml")).unwrap()
        .replace("@PLATFORM@", "any").replace("runtime = \"native\"", "runtime = \"python\"")
        .replace("@ENTRY@", "entry.bin");
    fs::write(stage.join("om9_addon.toml"), manifest).unwrap();
    fs::write(stage.join("entry.bin"), b"in-process").unwrap();
    let (_, bytes) = build_package(&stage, Some(&catalog)).unwrap();
    let package = tmp.join("tools.zip");
    fs::write(&package, bytes).unwrap();
    let mut host = Host::new(HostConfig::new(om9_app::addons::om9_version(), catalog, tmp.join("config")));
    host.register_runtime(RuntimeKind::Python, Box::new(InProcess));
    host.startup(&Startup::default());
    assert!(host.install_zip(&package, &InstallOptions { enable: true, ..Default::default() }).unwrap().enabled);
    Arc::new(Mutex::new(host))
}

fn boolean_host(tmp: &Path) -> Arc<Mutex<Host>> {
    let catalog = om9_app::addons::catalog().unwrap();
    let stage = tmp.join("stage-boolean");
    fs::create_dir_all(&stage).unwrap();
    let manifest = fs::read_to_string(repo().join("addons/setting_tools/boolean/addon/om9_addon.toml")).unwrap()
        .replace("@PLATFORM@", "any").replace("runtime = \"native\"", "runtime = \"python\"")
        .replace("@ENTRY@", "entry.bin");
    fs::write(stage.join("om9_addon.toml"), manifest).unwrap();
    fs::write(stage.join("entry.bin"), b"in-process").unwrap();
    let (_, bytes) = build_package(&stage, Some(&catalog)).unwrap();
    let package = tmp.join("boolean.zip"); fs::write(&package, bytes).unwrap();
    let mut host = Host::new(HostConfig::new(om9_app::addons::om9_version(), catalog, tmp.join("config")));
    host.register_runtime(RuntimeKind::Python, Box::new(InProcess));
    host.startup(&Startup::default());
    assert!(host.install_zip(&package, &InstallOptions { enable: true, ..Default::default() }).unwrap().enabled);
    Arc::new(Mutex::new(host))
}

#[test]
fn tools_menu_route_snapshot_covers_all_icons_and_fallbacks() {
    use om9_app::menu_catalog::TOOLS;
    let feats = features();
    let names: Vec<FeatureName> = feats.iter().map(|(id, command)| FeatureName { id, command: command.as_deref() }).collect();
    assert_eq!(TOOLS.len(), 31);
    let mut rows = Vec::new();
    for matrix in [false, true] {
        let tmp = std::env::temp_dir().join(format!("om9-tools-routes-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&tmp).unwrap();
        let addon = if matrix { host(&tmp) } else { tools_host(&tmp) };
        let package = if matrix { "om9_matrix" } else { "om9_tools" };
        let mut c = Controller::new(Document::with_template_tolerance("tools routes"), Limits::default()).unwrap();
        c.attach_addons(addon.clone(), &names);
        for icon in TOOLS {
            let target = c.icon_target(icon, None);
            if matrix || *icon != "BuilderBooleanBuilder" {
                let target = target.as_ref().expect("provided Tools route");
                assert!(target.addon, "{package}: {icon}");
                assert_eq!(&target.command, icon, "{package}: {icon}");
                if *icon == "gvJoinHistory" { assert!(!target.enabled); }
            }
            rows.push(serde_json::json!({"package":package,"state":"enabled","icon":icon,
                "route":target.as_ref().map(|t| &t.command),"addon":target.as_ref().map(|t| t.addon),"enabled":target.as_ref().map(|t| t.enabled)}));
        }
        addon.lock().unwrap().disable(package).unwrap();
        c.refresh_addon_names(&names);
        for icon in TOOLS {
            let target = c.icon_target(icon, None);
            if let Some(target) = &target { assert!(!target.addon, "disabled package: {icon}"); }
            if *icon == "BuilderProfile" {
                assert!(!target.as_ref().expect("Profile fallback").enabled);
            }
            rows.push(serde_json::json!({"package":package,"state":"disabled","icon":icon,
                "route":target.as_ref().map(|t| &t.command),"addon":target.as_ref().map(|t| t.addon),"enabled":target.as_ref().map(|t| t.enabled)}));
        }
        fs::remove_dir_all(tmp).unwrap();
    }
    let snapshot = serde_json::json!({"date":"2026-11-17", "scope":"headless icon_target resolution; no command completion claim", "rows":rows});
    if let Some(path) = std::env::var_os("OM9_TOOLS_ROUTE_SNAPSHOT") {
        fs::write(path, serde_json::to_vec_pretty(&snapshot).unwrap()).unwrap();
    }
}

#[test]
fn join_history_creation_is_visible_but_unavailable() {
    let feats = features();
    let names: Vec<FeatureName> = feats.iter().map(|(id, command)| FeatureName { id, command: command.as_deref() }).collect();
    for matrix in [false, true] {
        let tmp = std::env::temp_dir().join(format!("om9-join-unavailable-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&tmp).unwrap();
        let addon = if matrix { host(&tmp) } else { tools_host(&tmp) };
        let package = if matrix { "om9_matrix" } else { "om9_tools" };
        let mut c = Controller::new(Document::with_template_tolerance("join unavailable"), Limits::default()).unwrap();
        c.attach_addons(addon.clone(), &names);
        let target = c.icon_target("gvJoinHistory", None).unwrap();
        assert!(target.addon && !target.enabled);
        let before = c.document().clone();
        let undo = c.coordinator().undo_len();
        for name in ["gvJoinHistory", "JoinHistory", "OM9-TOOLS-017"] {
            assert!(!c.is_available(name), "{name}");
            assert!(c.submit(name).is_err(), "{name}");
            assert!(c.addon_session().is_none());
        }
        assert!(c.run_addon_command("gvJoinHistory").is_err());
        assert_eq!(c.document(), &before);
        assert_eq!(c.coordinator().undo_len(), undo);
        addon.lock().unwrap().disable(package).unwrap();
        c.refresh_addon_names(&names);
        let target = c.icon_target("gvJoinHistory", None).unwrap();
        assert!(!target.addon && !target.enabled);
        fs::remove_dir_all(tmp).unwrap();
    }
}

#[test]
fn boolean_builder_refusal_through_host_preserves_document_and_undo() {
    use om9_core::{geometry::Mesh, model::{Definition, Entity}};
    let tmp = std::env::temp_dir().join(format!("om9-boolean-refusal-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&tmp).unwrap();
    let mut doc = Document::with_template_tolerance("boolean refusal");
    let valid = Entity::new("closed", Geometry::Mesh(Mesh {
        vertices: vec![[0., 0., 0.], [4., 0., 0.], [0., 4., 0.], [0., 0., 4.]],
        faces: vec![vec![0, 2, 1], vec![0, 1, 3], vec![1, 2, 3], vec![2, 0, 3]],
        normals: vec![], colors: vec![], corner_uvs: vec![],
    }), None);
    let invalid = Entity::new("open", Geometry::Mesh(Mesh {
        vertices: vec![[1., 1., 1.], [2., 1., 1.], [1., 2., 1.]],
        faces: vec![vec![0, 1, 2]], normals: vec![], colors: vec![], corner_uvs: vec![],
    }), None);
    let mut cutter = Entity::new("cutter", valid.geometry.clone(), None);
    cutter.placement = om9_core::geometry::Transform::translation([1., 0., 0.]);
    let definition = Definition::new("mesh definition", vec![Entity::new("member", valid.geometry.clone(), None)]);
    let mut instance = Entity::new("instance", Geometry::Instance(definition.id), None);
    instance.placement = om9_core::geometry::Transform::translation([-0.5, 0., 0.]);
    let instance_id = instance.id;
    doc.definitions.insert(definition.id, definition);
    doc.entities.insert(instance_id, instance);
    let (valid_id, invalid_id, cutter_id) = (valid.id, invalid.id, cutter.id);
    doc.entities.insert(valid_id, valid);
    doc.entities.insert(invalid_id, invalid);
    doc.entities.insert(cutter_id, cutter);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    let feats = features();
    let names: Vec<FeatureName> = feats.iter().map(|(id, command)| FeatureName { id, command: command.as_deref() }).collect();
    c.attach_addons(host(&tmp), &names);
    let before = c.document().clone();
    c.select_objects(&[invalid_id]).unwrap();
    c.run_addon_command("BuilderBooleanBuilder").unwrap();
    assert!(c.submit("").is_err());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), 0);
    c.select_objects(&[valid_id]).unwrap();
    c.refresh_addon_session();
    c.submit("").unwrap();
    c.select_objects(&[invalid_id]).unwrap();
    c.refresh_addon_session();
    assert!(c.submit("").is_err());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), 0);
    assert!(c.addon_session().is_some());
    c.cancel_addon_session();
    // A caller can edit a previously accepted source while the add-on is waiting for cutters.
    // The retained first-group snapshot must not be used after that edit.
    c.select_objects(&[valid_id]).unwrap();
    c.run_addon_command("BuilderBooleanBuilder").unwrap();
    c.submit("").unwrap();
    c.select_objects(&[cutter_id]).unwrap();
    c.refresh_addon_session();
    assert!(!c.addon_session().unwrap().lines.is_empty(), "{:?}", c.addon_session().unwrap().reply.error);
    let old_preview = c.addon_session().unwrap().lines.clone();
    c.center_selected().unwrap();
    c.refresh_addon_session();
    assert_ne!(c.addon_session().unwrap().lines, old_preview, "a changed cutter refreshes the preview even without a selection change");
    c.select_objects(&[valid_id]).unwrap();
    c.center_selected().unwrap();
    let after_edit = c.document().clone();
    let undo = c.coordinator().undo_len();
    c.refresh_addon_session();
    assert!(c.addon_session().unwrap().lines.is_empty(), "a stale object clears preview on refresh");
    assert!(matches!(c.submit(""), Err(om9_core::Error::Stale(_))));
    assert_eq!(c.document(), &after_edit);
    assert_eq!(c.coordinator().undo_len(), undo);
    assert!(c.addon_session().unwrap().lines.is_empty());
    c.cancel_addon_session();
    c.undo().unwrap();
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
    c.select_objects(&[valid_id]).unwrap();
    c.run_addon_command("BuilderBooleanBuilder").unwrap();
    c.submit("").unwrap();
    c.select_objects(&[cutter_id]).unwrap();
    c.refresh_addon_session();
    c.submit("").unwrap();
    assert_ne!(c.document(), &before);
    assert!(c.addon_session().is_none());
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
    c.select_objects(&[instance_id]).unwrap();
    c.run_addon_command("BuilderBooleanBuilder").unwrap();
    c.submit("").unwrap();
    c.select_objects(&[cutter_id]).unwrap();
    c.refresh_addon_session();
    c.submit("").unwrap();
    assert!(matches!(c.document().entities[&instance_id].geometry, Geometry::Mesh(_)));
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
    c.select_objects(&[valid_id]).unwrap();
    c.run_addon_command("BuilderBooleanBuilder").unwrap();
    c.submit("KeepCutters=Yes").unwrap();
    c.submit("").unwrap();
    c.select_objects(&[instance_id]).unwrap();
    c.refresh_addon_session();
    c.submit("").unwrap();
    assert_eq!(c.document().entities[&instance_id], before.entities[&instance_id]);
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
    fs::remove_dir_all(tmp).unwrap();
}

#[test]
fn boolean_builder_receives_source_mesh_in_world_units() {
    use om9_core::{geometry::{Mesh, Transform}, model::{Definition, Entity}};
    let tmp = std::env::temp_dir().join(format!("om9-boolean-source-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&tmp).unwrap();
    let mut doc = Document::with_template_tolerance("Boolean source");
    doc.tolerance = Some(om9_core::history::Tolerance::try_new(0.02, 0.01, 0.1).unwrap());
    let mesh = Mesh { vertices: vec![[0., 0., 0.], [2., 0., 0.], [2., 2., 0.], [0., 2., 0.]],
        faces: vec![vec![0, 1, 2, 3]], normals: vec![], colors: vec![], corner_uvs: vec![] };
    let mut entity = Entity::new("quad", Geometry::Mesh(mesh), None);
    entity.placement = Transform::translation([3., 1., 0.]);
    let id = entity.id;
    let mut member = Entity::new("member", entity.geometry.clone(), None);
    member.placement = Transform::translation([0., 2., 0.]);
    let inner = Definition::new("inner", vec![member]);
    let mut nested = Entity::new("nested", Geometry::Instance(inner.id), None);
    nested.placement = Transform::translation([0., 0., 1.]);
    let outer = Definition::new("outer", vec![nested]);
    let mut instance = Entity::new("instance", Geometry::Instance(outer.id), None);
    instance.placement = Transform::translation([3., 1., 0.]);
    let instance_id = instance.id;
    doc.definitions.insert(inner.id, inner);
    doc.definitions.insert(outer.id, outer);
    doc.entities.insert(instance_id, instance);
    let multi = Definition::new("multiple", vec![Entity::new("a", entity.geometry.clone(), None), Entity::new("b", entity.geometry.clone(), None)]);
    let multi_instance = Entity::new("multi", Geometry::Instance(multi.id), None);
    let multi_id = multi_instance.id;
    doc.definitions.insert(multi.id, multi);
    doc.entities.insert(multi_id, multi_instance);
    doc.entities.insert(id, entity);
    let concave = Entity::new("concave", Geometry::Mesh(Mesh {
        vertices: vec![[0., 0., 0.], [2., 0., 0.], [0.5, 0.5, 0.], [0., 2., 0.]],
        faces: vec![vec![0, 1, 2, 3]], normals: vec![], colors: vec![], corner_uvs: vec![],
    }), None);
    let concave_id = concave.id;
    doc.entities.insert(concave_id, concave);
    let crossed = Entity::new("crossed", Geometry::Mesh(Mesh {
        vertices: vec![[0., 0., 0.], [2., 2., 0.], [0., 2., 0.], [2., 0., 0.]],
        faces: vec![vec![0, 1, 2, 3]], normals: vec![], colors: vec![], corner_uvs: vec![],
    }), None);
    let crossed_id = crossed.id;
    doc.entities.insert(crossed_id, crossed);
    let catalog = om9_app::addons::catalog().unwrap();
    let mut h = Host::new(HostConfig::new(om9_app::addons::om9_version(), catalog.clone(), tmp.join("config")));
    let recorded = Arc::new(Mutex::new(Vec::new()));
    h.register_runtime(RuntimeKind::Python, Box::new(RecordingRuntime(recorded.clone())));
    h.startup(&Startup::default());
    let zip = stage_zip(&tmp, &catalog);
    assert!(h.install_zip(&zip, &InstallOptions { enable: true, ..Default::default() }).unwrap().enabled);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.attach_addons(Arc::new(Mutex::new(h)), &[]);
    c.run_addon_command("BuilderBooleanBuilder").unwrap();
    let events = recorded.lock().unwrap();
    assert_eq!(events.last().unwrap().settings["_om9_document_tolerance_mm"].as_f64(), Some(0.02));
    let selected = &events.last().unwrap().selection[0];
    let actual = selected.mesh.as_ref().unwrap();
    assert_eq!(actual.vertices, vec![[3., 1., 0.], [5., 1., 0.], [5., 3., 0.], [3., 3., 0.]]);
    assert_eq!(actual.triangles.len(), 2);
    assert_eq!(selected.bbox, [[3., 1., 0.], [5., 3., 0.]]);
    drop(events);
    c.cancel_addon_session();
    #[cfg(feature = "occt-brep")]
    {
    c.request_viewport_detail(om9_app::display::ViewportDetail::High).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while c.viewport_detail_pending() {
        if let Some(result) = c.poll_viewport_detail() { result.unwrap(); }
        assert!(std::time::Instant::now() < deadline, "viewport detail timeout");
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    c.run_addon_command("BuilderBooleanBuilder").unwrap();
    let events = recorded.lock().unwrap();
    assert_eq!(events.first().unwrap().selection[0].mesh, events.last().unwrap().selection[0].mesh);
    drop(events);
    c.cancel_addon_session();
    }
    c.select_objects(&[instance_id]).unwrap();
    c.run_addon_command("BuilderBooleanBuilder").unwrap();
    let events = recorded.lock().unwrap();
    let selected = &events.last().unwrap().selection[0];
    assert_eq!(selected.kind, "mesh");
    assert_eq!(selected.mesh.as_ref().unwrap().vertices, vec![[3., 3., 1.], [5., 3., 1.], [5., 5., 1.], [3., 5., 1.]]);
    drop(events);
    c.cancel_addon_session();
    c.select_objects(&[multi_id]).unwrap();
    assert!(matches!(c.run_addon_command("BuilderBooleanBuilder"), Err(om9_core::Error::Unsupported(_))));
    c.select_objects(&[concave_id]).unwrap();
    c.run_addon_command("BuilderBooleanBuilder").unwrap();
    let events = recorded.lock().unwrap();
    let mesh = events.last().unwrap().selection[0].mesh.as_ref().unwrap();
    assert_eq!(mesh.triangles.len(), 2);
    let area: f64 = mesh.triangles.iter().map(|triangle| {
        let [a,b,c] = triangle.map(|i| mesh.vertices[i as usize]);
        ((b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])) / 2.
    }).sum();
    assert!((area - 1.).abs() < 1e-10);
    drop(events);
    c.cancel_addon_session();
    c.select_objects(&[crossed_id]).unwrap();
    assert!(c.run_addon_command("BuilderBooleanBuilder").is_err());
    fs::remove_dir_all(tmp).unwrap();
}

#[test]
fn boolean_builder_worker_commits_cancels_and_rejects_stale_selection() {
    use om9_core::{geometry::{Mesh, Transform}, model::Entity};
    for mode in 0..20 {
        let tmp = std::env::temp_dir().join(format!("om9-boolean-worker-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&tmp).unwrap();
        let mesh = Mesh { vertices: vec![[0.,0.,0.],[4.,0.,0.],[0.,4.,0.],[0.,0.,4.]],
            faces: vec![vec![0,2,1],vec![0,1,3],vec![1,2,3],vec![2,0,3]], normals: vec![], colors: vec![], corner_uvs: vec![] };
        let object = Entity::new("object", Geometry::Mesh(mesh.clone()), None);
        let mut cutter = Entity::new("cutter", Geometry::Mesh(mesh.clone()), None);
        cutter.placement = Transform::translation([if mode == 4 { 20. } else { 1. },0.,0.]);
        let (a, b) = (object.id, cutter.id);
        let mut doc = Document::with_template_tolerance("Boolean worker");
        if mode == 19 {
            let definition = om9_core::model::Definition::new("cutter mesh", vec![Entity::new("member", Geometry::Mesh(mesh.clone()), None)]);
            cutter.geometry = Geometry::Instance(definition.id);
            doc.definitions.insert(definition.id, definition);
        }
        doc.entities.insert(a, object);
        doc.entities.insert(b, cutter);
        let extra = if mode == 7 || mode == 17 {
            let mut object = Entity::new("second object", Geometry::Mesh(mesh.clone()), None);
            object.placement = Transform::translation([0., 10., 0.]);
            let mut cutter = Entity::new("second cutter", Geometry::Mesh(mesh), None);
            cutter.placement = Transform::translation([1., 10., 0.]);
            let ids = (object.id, cutter.id);
            doc.entities.insert(object.id, object);
            doc.entities.insert(cutter.id, cutter);
            Some(ids)
        } else { None };
        let reopen_path = tmp.join("reopen.om9");
        if mode == 11 {
            om9_archive::Archive::save(&reopen_path, &doc, &Limits::default(), &om9_core::transaction::Cancellation::default()).unwrap();
        }
        let mut app = om9_app::ui::MatrixApp::new(doc).unwrap();
        let feats = features();
        let names: Vec<FeatureName> = feats.iter().map(|(id, command)| FeatureName { id, command: command.as_deref() }).collect();
        app.controller.attach_addons(if mode == 12 { boolean_host(&tmp) } else { host(&tmp) }, &names);
        if mode == 11 {
            app.controller.begin_file_operation(om9_app::file_lifecycle::FileKind::Open).unwrap();
            let token = app.controller.file_operation().unwrap().token();
            app.controller.accept_file_path(token, &reopen_path).unwrap();
        }
        let objects = extra.map_or_else(|| vec![a], |(object, _)| vec![a, object]);
        app.controller.select_objects(&objects).unwrap();
        if mode == 1 {
            app.controller.submit("BooleanBuilder").unwrap();
        } else {
            let target = app.controller.icon_target("BuilderBooleanBuilder", None).unwrap();
            assert!(target.addon && target.enabled);
            app.controller.run_addon_command(&target.command).unwrap();
        }
        if mode == 3 { app.controller.submit("KeepCutters=Yes").unwrap(); }
        if mode == 4 { app.controller.submit("Mode=Intersection").unwrap(); }
        if mode == 9 { app.controller.submit("Mode=Union").unwrap(); }
        if mode == 10 { app.controller.submit("Mode=Intersection").unwrap(); }
        app.controller.submit("").unwrap();
        let cutters = extra.map_or_else(|| vec![b], |(_, cutter)| vec![b, cutter]);
        app.controller.select_objects(&cutters).unwrap();
        app.controller.refresh_addon_session();
        assert!(app.controller.boolean_builder_worker_preview());
        if mode == 18 {
            let before = app.controller.document().clone();
            app.controller.submit("").unwrap();
            app.controller.select_objects(&[a]).unwrap();
            assert!(matches!(app.controller.poll_boolean_builder_worker(), Some(Err(om9_core::Error::Stale(_)))));
            assert_eq!(app.controller.document(), &before);
            assert_eq!(app.controller.coordinator().undo_len(), 0);
            fs::remove_dir_all(tmp).unwrap();
            continue;
        }
        if mode == 15 || mode == 16 || mode == 17 {
            let before = app.controller.document().clone();
            let ctx = eframe::egui::Context::default();
            if mode != 16 { app.controller.submit("").unwrap(); } else {
                let mut output = ctx.run_ui(eframe::egui::RawInput {
                    screen_rect: Some(eframe::egui::Rect::from_min_size(eframe::egui::Pos2::ZERO, eframe::egui::Vec2::new(1280., 900.))),
                    events: vec![eframe::egui::Event::Key { key: eframe::egui::Key::Enter, physical_key: None,
                        pressed: true, repeat: false, modifiers: eframe::egui::Modifiers::NONE }], ..Default::default()
                }, |ui| app.show(ui));
                output.textures_delta.clear();
            }
            assert!(app.controller.boolean_builder_worker_pending() || app.controller.document() != &before, "mode {mode}: Enter neither committed nor kept the worker");
            assert!(!app.controller.boolean_builder_worker_preview());
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
            while app.controller.boolean_builder_worker_pending() {
                if let Some(result) = app.controller.poll_boolean_builder_worker() { result.unwrap(); break; }
                assert!(std::time::Instant::now() < deadline, "promoted Boolean preview timeout");
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
            assert_ne!(app.controller.document(), &before);
            if let Some((object, cutter)) = extra {
                assert_ne!(app.controller.document().entities[&object].geometry, before.entities[&object].geometry);
                assert!(!app.controller.document().entities.contains_key(&cutter));
            }
            assert_eq!(app.controller.coordinator().undo_len(), 1);
            app.controller.undo().unwrap();
            assert_eq!(app.controller.document(), &before);
            fs::remove_dir_all(tmp).unwrap();
            continue;
        }
        if mode == 13 || mode == 14 {
            let before = app.controller.document().clone();
            if mode == 13 { app.controller.select_objects(&[a]).unwrap(); }
            else { app.controller.set_builder_text("bb_mode", "union").unwrap(); }
            assert!(matches!(app.controller.poll_boolean_builder_worker(), Some(Err(om9_core::Error::Stale(_)))));
            assert_eq!(app.controller.document(), &before);
            assert!(app.controller.addon_session().unwrap().lines.is_empty());
            if mode == 13 { app.controller.select_objects(&[b]).unwrap(); }
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
            while !app.controller.boolean_builder_worker_pending() {
                app.controller.refresh_addon_session();
                assert!(std::time::Instant::now() < deadline, "stale preview retry timeout");
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
            loop {
                if let Some(result) = app.controller.poll_boolean_builder_worker() { result.unwrap(); break; }
                assert!(std::time::Instant::now() < deadline, "retried preview timeout");
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
            assert!(!app.controller.addon_session().unwrap().lines.is_empty());
            assert_eq!(app.controller.document(), &before);
            fs::remove_dir_all(tmp).unwrap();
            continue;
        }
        let preview_deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        let preview_result = if mode == 0 {
            let ctx = eframe::egui::Context::default();
            while app.controller.boolean_builder_worker_pending() {
                let mut output = ctx.run_ui(eframe::egui::RawInput {
                    screen_rect: Some(eframe::egui::Rect::from_min_size(eframe::egui::Pos2::ZERO, eframe::egui::Vec2::new(1280., 900.))),
                    ..Default::default()
                }, |ui| app.show(ui));
                output.textures_delta.clear();
                assert!(std::time::Instant::now() < preview_deadline, "Boolean preview UI timeout");
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
            Ok(())
        } else { loop {
            if let Some(result) = app.controller.poll_boolean_builder_worker() { break result; }
            assert!(std::time::Instant::now() < preview_deadline, "Boolean preview timeout");
            std::thread::sleep(std::time::Duration::from_millis(1));
        }};
        if mode == 4 { assert!(preview_result.is_err()); } else {
            preview_result.unwrap();
            assert!(!app.controller.addon_session().unwrap().lines.is_empty());
            assert_eq!(app.controller.addon_session().unwrap().lines.len(), app.controller.addon_session().unwrap().reply.preview.len());
        }
        let before = app.controller.document().clone();
        app.controller.submit("").unwrap();
        assert!(app.controller.boolean_builder_worker_pending());
        assert!(app.controller.prompt_text().contains("computing"));
        assert_eq!(app.controller.document(), &before);
        if mode == 1 || mode == 5 || mode == 8 {
            if mode == 5 || mode == 8 {
                let ctx = eframe::egui::Context::default();
                let event = if mode == 5 { eframe::egui::Event::Key { key: eframe::egui::Key::Escape, physical_key: None,
                    pressed: true, repeat: false, modifiers: eframe::egui::Modifiers::NONE } } else { eframe::egui::Event::Text("x".into()) };
                let mut output = ctx.run_ui(eframe::egui::RawInput {
                    screen_rect: Some(eframe::egui::Rect::from_min_size(eframe::egui::Pos2::ZERO, eframe::egui::Vec2::new(1280., 900.))),
                    events: vec![event], ..Default::default()
                }, |ui| app.show(ui));
                output.textures_delta.clear();
            } else { app.controller.cancel_boolean_builder_worker(); }
            assert!(!app.controller.boolean_builder_worker_pending());
            assert_eq!(app.controller.document(), &before);
            if mode == 1 {
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
                loop {
                    match app.controller.submit("") {
                        Ok(()) => break,
                        Err(om9_core::Error::Busy) => {
                            assert!(std::time::Instant::now() < deadline, "canceled worker permit not released");
                            std::thread::sleep(std::time::Duration::from_millis(1));
                        }
                        Err(error) => panic!("retry after cancellation: {error}"),
                    }
                }
                assert!(app.controller.boolean_builder_worker_pending());
                app.controller.cancel_boolean_builder_worker();
                assert_eq!(app.controller.document(), &before);
            }
        } else {
            if mode == 2 { app.controller.select_objects(&[a]).unwrap(); }
            if mode == 6 { app.controller.set_builder_text("bb_mode", "union").unwrap(); }
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
            let result = if mode == 0 {
                let ctx = eframe::egui::Context::default();
                while app.controller.boolean_builder_worker_pending() {
                    let mut output = ctx.run_ui(eframe::egui::RawInput {
                        screen_rect: Some(eframe::egui::Rect::from_min_size(eframe::egui::Pos2::ZERO, eframe::egui::Vec2::new(1280., 900.))),
                        ..Default::default()
                    }, |ui| app.show(ui));
                    output.textures_delta.clear();
                    assert!(std::time::Instant::now() < deadline, "Boolean UI worker timeout");
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
                Ok(())
            } else { loop {
                if let Some(result) = app.controller.poll_boolean_builder_worker() { break result; }
                assert!(std::time::Instant::now() < deadline, "Boolean worker timeout");
                std::thread::sleep(std::time::Duration::from_millis(1));
            }};
            if mode == 2 || mode == 6 {
                assert!(matches!(result, Err(om9_core::Error::Stale(_))));
                assert_eq!(app.controller.document(), &before);
                assert_eq!(app.controller.coordinator().undo_len(), 0);
            } else if mode == 4 {
                assert!(result.is_err());
                assert_eq!(app.controller.document(), &before);
                assert_eq!(app.controller.coordinator().undo_len(), 0);
                assert!(app.controller.addon_session().is_some());
            } else {
                result.unwrap();
                assert_ne!(app.controller.document(), &before);
                assert_eq!(app.controller.document().entities.contains_key(&b), mode == 3);
                if [0, 3, 9, 10].contains(&mode) {
                    let Geometry::Mesh(mesh) = &app.controller.document().entities[&a].geometry else { panic!("mesh output") };
                    let volume: f64 = mesh.faces.iter().map(|face| {
                        let a = mesh.vertices[face[0] as usize];
                        let b = mesh.vertices[face[1] as usize];
                        let c = mesh.vertices[face[2] as usize];
                        om9_addon_abi::session::dot(a, om9_addon_abi::session::cross(b, c)) / 6.0
                    }).sum();
                    let expected = if mode == 9 { 101. / 6. } else if mode == 10 { 27. / 6. } else { 37. / 6. };
                    assert!((volume.abs() - expected).abs() < 1e-6, "mode {mode}: {volume}");
                }
                if let Some((object, cutter)) = extra {
                    assert_ne!(app.controller.document().entities[&object].geometry, before.entities[&object].geometry);
                    assert!(!app.controller.document().entities.contains_key(&cutter));
                }
                assert_eq!(app.controller.coordinator().undo_len(), 1);
                let after = app.controller.document().clone();
                app.controller.undo().unwrap();
                assert_eq!(app.controller.document(), &before);
                app.controller.redo().unwrap();
                assert_eq!(app.controller.document(), &after);
            }
        }
        fs::remove_dir_all(tmp).unwrap();
    }
}

#[test]
fn boolean_preview_keeps_viewport_object_selection_available() {
    use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2, Rect, Vec2};
    use om9_core::{geometry::{Mesh, Transform}, model::Entity};
    use om9_scene::{camera::{Projection, Viewport}, workspace::ViewId};
    let tmp = std::env::temp_dir().join(format!("om9-boolean-preview-pick-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&tmp).unwrap();
    let mesh = Mesh { vertices: vec![[0.,0.,0.],[4.,0.,0.],[0.,4.,0.],[0.,0.,4.]],
        faces: vec![vec![0,2,1],vec![0,1,3],vec![1,2,3],vec![2,0,3]], normals: vec![], colors: vec![], corner_uvs: vec![] };
    let object = Entity::new("object", Geometry::Mesh(mesh.clone()), None);
    let mut cutter = Entity::new("cutter", Geometry::Mesh(mesh.clone()), None);
    cutter.placement = Transform::translation([1.,0.,0.]);
    let mut next = Entity::new("next cutter", Geometry::Mesh(mesh), None);
    next.placement = Transform::translation([7.,0.,0.]);
    let (a,b,c) = (object.id,cutter.id,next.id);
    let mut doc = Document::with_template_tolerance("Boolean preview picking");
    for entity in [object,cutter,next] { doc.entities.insert(entity.id, entity); }
    let mut app = om9_app::ui::MatrixApp::new(doc).unwrap();
    app.controller.attach_addons(host(&tmp), &[]);
    let view = ViewId::LookingDown;
    app.controller.workspace.views.get_mut(&view).unwrap().camera.projection = Projection::Orthographic { vertical_span_mm: 20. };
    let ctx = egui::Context::default();
    let frame = |app: &mut om9_app::ui::MatrixApp, events| {
        let mut out = ctx.run_ui(egui::RawInput { screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.,900.))), events, ..Default::default() }, |ui| app.show(ui));
        out.textures_delta.clear();
    };
    frame(&mut app, vec![]);
    let rect = app.view_rects[&view];
    let viewport = Viewport { origin: [rect.min.x as f64, rect.min.y as f64], size: [rect.width() as f64, rect.height() as f64] };
    let point = app.controller.workspace.views[&view].camera.project([7.5,0.5,0.], viewport).unwrap().unwrap().screen;
    let point = Pos2::new(point[0] as f32, point[1] as f32);
    assert!(rect.contains(point));
    app.controller.select_objects(&[a]).unwrap();
    app.controller.run_addon_command("BuilderBooleanBuilder").unwrap();
    app.controller.submit("").unwrap();
    app.controller.select_objects(&[b]).unwrap();
    app.controller.refresh_addon_session();
    assert!(app.controller.boolean_builder_worker_preview());
    frame(&mut app, vec![Event::PointerMoved(point), Event::PointerButton { pos: point, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE }]);
    frame(&mut app, vec![Event::PointerMoved(point), Event::PointerButton { pos: point, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE }]);
    assert!(app.controller.selected().contains(&c), "viewport click must select the new cutter");
    app.controller.cancel_boolean_builder_worker();
    fs::remove_dir_all(tmp).unwrap();
}

#[cfg(feature = "test-fixtures")]
#[test]
fn boolean_builder_worker_refuses_unavailable_admission_without_mutation() {
    use om9_core::{geometry::{Mesh, Transform}, model::Entity};
    let tmp = std::env::temp_dir().join(format!("om9-boolean-quota-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&tmp).unwrap();
    let mesh = Mesh { vertices: vec![[0.,0.,0.],[4.,0.,0.],[0.,4.,0.],[0.,0.,4.]],
        faces: vec![vec![0,2,1],vec![0,1,3],vec![1,2,3],vec![2,0,3]], normals: vec![], colors: vec![], corner_uvs: vec![] };
    let object = Entity::new("object", Geometry::Mesh(mesh.clone()), None);
    let mut cutter = Entity::new("cutter", Geometry::Mesh(mesh), None);
    cutter.placement = Transform::translation([1.,0.,0.]);
    let (a,b) = (object.id, cutter.id);
    let mut doc = Document::with_template_tolerance("Boolean quota");
    doc.entities.insert(a, object); doc.entities.insert(b, cutter);
    let mut limits = Limits::default(); limits.max_bytes = 16 * 1024 * 1024;
    let mut c = Controller::new(doc, limits).unwrap();
    c.test_enable_background_boolean_builder();
    let feats = features();
    let names: Vec<FeatureName> = feats.iter().map(|(id, command)| FeatureName { id, command: command.as_deref() }).collect();
    c.attach_addons(host(&tmp), &names);
    c.select_objects(&[a]).unwrap(); c.run_addon_command("BuilderBooleanBuilder").unwrap(); c.submit("").unwrap();
    c.select_objects(&[b]).unwrap(); c.refresh_addon_session();
    let before = c.document().clone();
    assert!(matches!(c.submit(""), Err(om9_core::Error::Budget(_))));
    assert!(!c.boolean_builder_worker_pending());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), 0);
    fs::remove_dir_all(tmp).unwrap();
}

#[test]
fn original_tools_aliases_match_menu_routes_and_geometry() {
    let feats = features();
    let names: Vec<FeatureName> = feats.iter().map(|(id, command)| FeatureName { id, command: command.as_deref() }).collect();
    for matrix in [false, true] {
        let tmp = std::env::temp_dir().join(format!("om9-tools-alias-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&tmp).unwrap();
        let file = tmp.join("trace.pgm");
        let mut image = b"P5\n20 20\n255\n".to_vec();
        for y in 0..20 { for x in 0..20 { image.push(if (5..15).contains(&x) && (5..15).contains(&y) { 0 } else { 255 }); } }
        fs::write(&file, image).unwrap();
        let addon = if matrix { host(&tmp) } else { tools_host(&tmp) };
        let package = if matrix { "om9_matrix" } else { "om9_tools" };
        addon.lock().unwrap().set_setting(package, "it_file", &file.to_string_lossy()).unwrap();
        addon.lock().unwrap().set_setting(package, "it_smooth", "false").unwrap();
        for (code, alias) in [("BuilderOutsideRingRail", "gvOutsideRingRail"), ("BuilderImageTrace", "gvImageTrace")] {
            let mut c = Controller::new(Document::with_template_tolerance("tools aliases"), Limits::default()).unwrap();
            // No feature-name fallback: the package manifest itself supplies the alias.
            c.attach_addons(addon.clone(), &[]);
            assert_eq!(c.addon_code(alias).as_deref(), Some(code));
            let menu = c.icon_target(code, None).unwrap();
            assert!(menu.addon && menu.enabled && menu.command == code);
            c.run_addon_command(&menu.command).unwrap();
            let start = c.addon_session().unwrap().reply.clone();
            if code == "BuilderImageTrace" { c.session_point([0., 0., 0.]).unwrap(); } else { c.submit("").unwrap(); }
            let geometry = c.document().entities.values().next().expect("menu geometry").geometry.clone();
            assert!(matches!(geometry, Geometry::Curve(_)));
            assert_eq!(c.coordinator().undo_len(), 1);
            c.undo().unwrap();
            assert!(c.document().entities.is_empty());
            c.submit(alias).unwrap();
            let session = c.addon_session().unwrap();
            assert_eq!(session.code, code);
            assert_eq!(session.reply, start);
            if code == "BuilderImageTrace" { c.session_point([0., 0., 0.]).unwrap(); } else { c.submit("").unwrap(); }
            assert_eq!(c.document().entities.values().next().unwrap().geometry, geometry);
            assert_eq!(c.coordinator().undo_len(), 1);
            c.undo().unwrap();
        }
        addon.lock().unwrap().disable(package).unwrap();
        let mut c = Controller::new(Document::with_template_tolerance("disabled tools"), Limits::default()).unwrap();
        c.attach_addons(addon.clone(), &names);
        for (code, alias, fallback, enabled) in [("BuilderOutsideRingRail", "gvOutsideRingRail", "OM9-TOOLS-003", true), ("BuilderImageTrace", "gvImageTrace", "OM9-TOOLS-016", false)] {
            assert!(c.addon_code(alias).is_none());
            let target = c.icon_target(code, None).expect("visible builtin fallback");
            assert!(!target.addon);
            assert_eq!(target.command, fallback);
            assert_eq!(target.enabled, enabled, "{code} fallback availability");
        }
        fs::remove_dir_all(tmp).unwrap();
    }
}

#[test]
fn interactive_addon_scale_rebuilds_join_history_atomically() {
    use om9_app::ui::MatrixApp;
    use om9_core::model::Entity;
    let tmp = std::env::temp_dir().join(format!("om9-addon-history-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&tmp).unwrap();
    let addon = host(&tmp);
    let feats = features();
    let names: Vec<FeatureName> = feats.iter().map(|(id, command)| FeatureName { id, command: command.as_deref() }).collect();
    for mode in 0..3 {
        let mut doc = Document::with_template_tolerance("add-on History");
        let mut ids = Vec::new();
        for (a, b) in [(0., 1.), (1., 2.), (2., 3.)] {
            let entity = Entity::new("parent", Geometry::Curve(Curve::Line { start: [a, 0., 0.], end: [b, 0., 0.], domain: [0., 1.] }), None);
            ids.push(entity.id);
            doc.entities.insert(entity.id, entity);
        }
        let mut app = MatrixApp::new(doc).unwrap();
        let c = &mut app.controller;
        c.select_objects(&ids[..2]).unwrap();
        c.create_join_history().unwrap();
        let first = c.document().history.recipes.values().next().unwrap().outputs[0];
        c.select_objects(&[first, ids[2]]).unwrap();
        c.create_join_history().unwrap();
        let second = c.document().history.recipes.values().find(|r| r.outputs[0] != first).unwrap().outputs[0];
        if mode == 1 { c.select_objects(&ids[..1]).unwrap(); } else { c.select_objects(&ids).unwrap(); }
        c.attach_addons(addon.clone(), &names);
        let before = c.document().clone();
        c.run_addon_command("TransformScaleScale3D").unwrap();
        c.submit("0,0,0").unwrap();
        c.submit("2").unwrap();
        assert!(c.join_history_worker_pending());
        assert_eq!(c.document(), &before);
        let event = c.events().events().back().unwrap();
        assert_eq!(event.command, "TransformScaleScale3D");
        assert_eq!(event.kind, om9_app::command_events::EventKind::Progress);
        if mode == 2 { c.cancel_join_history_worker(); }
        else {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
            while c.join_history_worker_pending() {
                let _ = c.poll_join_history_worker();
                assert!(std::time::Instant::now() < deadline, "History worker timeout");
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        }
        if mode == 0 {
            let after = c.document().clone();
            let Geometry::Curve(curve) = &after.entities[&second].geometry else { panic!("curve") };
            let ends = om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap();
            assert!(ends.contains(&[0., 0., 0.]) && ends.contains(&[6., 0., 0.]), "{ends:?}");
            assert_eq!(c.events().events().back().unwrap().outcome, Some(om9_app::command_events::EventOutcome::Committed));
            c.undo().unwrap();
            assert_eq!(c.document(), &before);
            c.redo().unwrap();
            assert_eq!(c.document(), &after);
        } else {
            assert_eq!(c.document(), &before);
            assert_eq!(c.coordinator().undo_len(), 2);
            let outcome = if mode == 1 { om9_app::command_events::EventOutcome::Refused } else { om9_app::command_events::EventOutcome::Cancelled };
            assert_eq!(c.events().events().back().unwrap().outcome, Some(outcome));
        }
    }
    fs::remove_dir_all(tmp).unwrap();
}

#[test]
fn addon_mixed_insert_and_parent_edit_is_atomic_and_delete_is_refused() {
    use om9_app::ui::MatrixApp;
    use om9_core::model::Entity;
    let feats = features();
    let names: Vec<FeatureName> = feats.iter().map(|(id, command)| FeatureName { id, command: command.as_deref() }).collect();
    for mode in 0..3 {
        let tmp = std::env::temp_dir().join(format!("om9-addon-mixed-history-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&tmp).unwrap();
        let catalog = om9_app::addons::catalog().unwrap();
        let mut h = Host::new(HostConfig::new(om9_app::addons::om9_version(), catalog.clone(), tmp.join("config")));
        h.register_runtime(RuntimeKind::Python, Box::new(MixedHistoryReply { delete_parent: mode == 2 }));
        h.startup(&Startup::default());
        let zip = stage_zip(&tmp, &catalog);
        assert!(h.install_zip(&zip, &InstallOptions { enable: true, ..Default::default() }).unwrap().enabled);
        let mut doc = Document::with_template_tolerance("mixed add-on History");
        let mut ids = Vec::new();
        for (a, b) in [(0., 1.), (1., 2.), (2., 3.)] {
            let entity = Entity::new("parent", Geometry::Curve(Curve::Line { start: [a, 0., 0.], end: [b, 0., 0.], domain: [0., 1.] }), None);
            ids.push(entity.id);
            doc.entities.insert(entity.id, entity);
        }
        let mut app = MatrixApp::new(doc).unwrap();
        let c = &mut app.controller;
        c.select_objects(&ids[..2]).unwrap();
        c.create_join_history().unwrap();
        let first = c.document().history.recipes.values().next().unwrap().outputs[0];
        c.select_objects(&[first, ids[2]]).unwrap();
        c.create_join_history().unwrap();
        let second = c.document().history.recipes.values().find(|r| r.outputs[0] != first).unwrap().outputs[0];
        if mode == 1 { c.select_objects(&ids[..1]).unwrap(); } else { c.select_objects(&ids).unwrap(); }
        c.attach_addons(Arc::new(Mutex::new(h)), &names);
        let before = c.document().clone();
        let result = c.run_addon_command("TransformScaleScale3D");
        if mode == 2 {
            assert!(matches!(result, Err(om9_core::Error::Unsupported("add-on History edit includes unsupported mutations"))));
            assert!(!c.join_history_worker_pending());
        } else {
            result.unwrap();
            assert!(c.join_history_worker_pending());
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
            while c.join_history_worker_pending() {
                let _ = c.poll_join_history_worker();
                assert!(std::time::Instant::now() < deadline, "History worker timeout");
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        }
        if mode == 0 {
            let after = c.document().clone();
            assert_eq!(after.entities.len(), before.entities.len() + 1);
            assert!(ids.iter().any(|id| after.entities[id].extensions.contains_key("om9.gem")));
            let Geometry::Curve(curve) = &after.entities[&second].geometry else { panic!("curve") };
            let ends = om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap();
            assert!(ends.contains(&[0., 1., 0.]) && ends.contains(&[3., 1., 0.]), "{ends:?}");
            c.undo().unwrap();
            assert_eq!(c.document(), &before);
            c.redo().unwrap();
            assert_eq!(c.document(), &after);
        } else {
            assert_eq!(c.document(), &before);
            assert_eq!(c.coordinator().undo_len(), 2);
        }
        fs::remove_dir_all(tmp).unwrap();
    }
}

/// Release-mode, headless command comparison. Run explicitly with `--ignored --nocapture`.
#[test]
#[ignore]
fn benchmark_builtin_vs_addon_curve_commands() {
    use std::time::Instant;

    let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let tmp = std::env::temp_dir().join(format!("om9-curve-benchmark-{}-{stamp}", std::process::id()));
    fs::create_dir_all(&tmp).unwrap();
    let host = host(&tmp);
    let feats = features();
    let names: Vec<FeatureName> = feats.iter().map(|(id, c)| FeatureName { id, command: c.as_deref() }).collect();
    let mut timings = vec![Vec::<u128>::new(); 4];

    // Alternate which implementation runs first to reduce systematic ordering bias.
    for iteration in 0..45 {
        for (case, addon) in (0..4).map(|x| (x / 2, (x % 2 == 1) ^ (iteration % 2 == 1))) {
            let mut c = Controller::new(Document::with_template_tolerance("curve-perf"), Limits::default()).unwrap();
            if addon { c.attach_addons(host.clone(), &names); }
            let started = Instant::now();
            if addon {
                c.run_addon_command(if case == 0 { "CurveLineSingleLine" } else { "CurveCircleCenterRadius" }).unwrap();
            } else {
                c.submit(if case == 0 { "OM9-CURVE-002" } else { "OM9-CURVE-005" }).unwrap();
            }
            if case == 0 {
                c.submit("0,0,0").unwrap();
                c.submit("10,0,0").unwrap();
            } else {
                if addon { c.session_point([0.0, 0.0, 0.0]).unwrap(); }
                else { c.accept_circle_point([0.0, 0.0, 0.0]).unwrap(); }
                c.submit("5").unwrap();
            }
            let elapsed = started.elapsed().as_nanos();
            assert_eq!(c.document().entities.len(), 1);
            assert_eq!(c.coordinator().undo_len(), 1);
            let Geometry::Curve(curve) = &c.document().entities.values().next().unwrap().geometry else { panic!("expected curve"); };
            if case == 0 {
                let p = om9_core::curve_ops::sample_curve(curve, 2, &Limits::default()).unwrap();
                assert_eq!(p.first().unwrap(), &[0.0, 0.0, 0.0]);
                assert_eq!(p.last().unwrap(), &[10.0, 0.0, 0.0]);
            } else {
                let Curve::Circle { center, radius, .. } = curve else { panic!("expected circle: {curve:?}") };
                assert_eq!(*center, [0.0, 0.0, 0.0]);
                assert!((*radius - 5.0).abs() < 1e-9);
            }
            if iteration >= 5 { timings[case * 2 + usize::from(addon)].push(elapsed); }
        }
    }
    for (i, samples) in timings.iter_mut().enumerate() {
        samples.sort_unstable();
        println!("PERF case={} impl={} n={} median_ns={} p95_ns={} min_ns={} max_ns={}",
            if i < 2 { "line" } else { "circle" }, if i % 2 == 0 { "builtin" } else { "addon" },
            samples.len(), (samples[19] + samples[20]) / 2, samples[37], samples[0], samples[39]);
    }
}

#[test]
fn curve_commands_run_interactively() {
    assert_eq!(
        <om9_curves::Curves as Interactive>::SESSION_COMMANDS.len(),
        13
    );
    let tmp = std::env::temp_dir().join(format!("om9-app-session-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();
    let feats = features();
    let names: Vec<FeatureName> = feats
        .iter()
        .map(|(id, c)| FeatureName {
            id,
            command: c.as_deref(),
        })
        .collect();
    let mut c = Controller::new(Document::new("curves"), Limits::default()).unwrap();
    assert!(!c.is_available("CurveCircleCenterRadius"));
    c.attach_addons(host(&tmp), &names);
    for code in om9_curves::Curves::SESSION_COMMANDS {
        assert!(c.is_available(code), "{code} is enabled from its menu icon");
    }
    assert_eq!(
        c.addon_code("rectangle").as_deref(),
        Some("CurveRectangleCornertoCorner")
    );
    assert_eq!(
        c.addon_code("Polyline").as_deref(),
        Some("CurvePolylinePolyline")
    );

    // With the package enabled, its Polyline, Circle … take over OM9's own icon and name.
    assert!(
        c.icon_target("Polyline", None)
            .is_some_and(|t| t.addon && t.command == "CurvePolylinePolyline")
    );
    // Polyline: prompt + sub-options, clicks, a typed relative point, Enter.
    c.run_addon_command("CurvePolylinePolyline").unwrap();
    let s = c.addon_session().expect("session open");
    assert!(
        c.prompt_text().starts_with("Start of polyline"),
        "{}",
        c.prompt_text()
    );
    assert!(
        s.tokens(c.tr())
            .iter()
            .any(|(_, t)| t == "PersistentClose=No")
    );
    assert!(c.session_wants_point());
    c.session_point([0.0, 0.0, 0.0]).unwrap();
    c.session_point([10.0, 0.0, 0.0]).unwrap();
    c.session_hover([10.0, 5.0, 0.0]);
    assert!(
        !c.addon_session().unwrap().hover_lines.is_empty(),
        "rubber segment preview"
    );
    assert!(
        c.document().entities.is_empty(),
        "preview is never a document object"
    );
    c.submit("@0,5").unwrap(); // relative to the last point
    c.submit("Undo").unwrap();
    c.submit("10,5").unwrap(); // absolute CPlane coordinates
    assert!(
        c.addon_session()
            .unwrap()
            .tokens(c.tr())
            .iter()
            .any(|(n, _)| n == "Close")
    );
    let undo = c.coordinator().undo_len();
    c.submit("Close").unwrap();
    assert!(c.addon_session().is_none());
    assert_eq!(c.document().entities.len(), 1);
    assert_eq!(c.coordinator().undo_len(), undo + 1, "one transaction");
    let e = c.document().entities.values().next().unwrap();
    let Geometry::Curve(Curve::Nurbs(n)) = &e.geometry else {
        panic!("{:?}", e.geometry)
    };
    assert_eq!((n.degree, n.poles.len()), (1, 4), "closed: start repeated");
    assert_eq!(n.poles[2], [10.0, 5.0, 0.0]);
    assert!(c.take_addon_messages().iter().any(|m| m.contains("closed")));

    // Circle from its menu icon: the Diameter sub-option (as in Matrix), typed diameter.
    c.dispatch("CurveCircleCenterRadius", InvocationOrigin::Menu)
        .unwrap();
    c.session_point([0.0, 0.0, 0.0]).unwrap();
    c.submit("Diameter").unwrap();
    c.submit("17.35").unwrap();
    let circle = c
        .document()
        .entities
        .values()
        .find_map(|e| match &e.geometry {
            Geometry::Curve(Curve::Circle { radius, .. }) => Some(*radius),
            _ => None,
        })
        .unwrap();
    assert!((circle - 8.675).abs() < 1e-12);

    // Esc cancels; nothing is created. Another command closes the session and runs.
    let before = c.document().entities.len();
    c.run_addon_command("CurveRectangleCornertoCorner").unwrap();
    c.session_point([0.0, 0.0, 0.0]).unwrap();
    c.cancel();
    assert!(c.addon_session().is_none());
    assert_eq!(c.document().entities.len(), before);
    c.run_addon_command("CurveRectangleCornertoCorner").unwrap();
    c.run_addon_command("CurveCircleCenterRadius").unwrap();
    assert_eq!(c.addon_session().unwrap().code, "CurveCircleCenterRadius");
    c.submit("Circle").unwrap();
    assert_eq!(
        c.addon_session().map(|s| s.code.as_str()),
        Some("CurveCircleCenterRadius"),
        "the typed name runs the add-on's Circle"
    );
    c.cancel();
    // A refused value keeps the session open with the reason.
    c.run_addon_command("CurvePolygonCenterRadius").unwrap();
    let err = c.submit("NumSides=2").unwrap_err().to_string();
    assert!(err.contains("NumSides"), "{err}");
    assert!(c.addon_session().is_some());
    c.submit("NumSides=6").unwrap();
    c.session_point([0.0, 0.0, 0.0]).unwrap();
    c.submit("3").unwrap();
    assert!(c.addon_session().is_none());
    assert_eq!(c.document().entities.len(), before + 1);
    // Undo removes the polygon.
    c.undo().unwrap();
    assert_eq!(c.document().entities.len(), before);
    fs::remove_dir_all(&tmp).unwrap();
}

#[test]
fn transform_commands_act_on_the_selection() {
    let tmp = std::env::temp_dir().join(format!("om9-app-transform-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();
    let feats = features();
    let names: Vec<FeatureName> = feats
        .iter()
        .map(|(id, c)| FeatureName {
            id,
            command: c.as_deref(),
        })
        .collect();
    let mut c = Controller::new(Document::new("transform"), Limits::default()).unwrap();
    c.attach_addons(host_with(&tmp, &["curves", "transform"]), &names);
    assert!(c.is_available("OM9-TOP11-006"), "Top 11 Move icon");
    assert_eq!(c.addon_code("move").as_deref(), Some("TopIconMove"));
    // A circle to work on.
    c.run_addon_command("CurveCircleCenterRadius").unwrap();
    c.session_point([0.0, 0.0, 0.0]).unwrap();
    c.submit("2").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    let center = |c: &Controller, id| {
        let e = c.document().entities.get(&id).unwrap();
        e.placement.point([0.0; 3]).unwrap()
    };

    // Move with nothing selected asks for objects first; selecting advances the prompt.
    c.select_objects(&[]).unwrap();
    c.run_addon_command("TopIconMove").unwrap();
    assert!(
        c.prompt_text().starts_with("Select objects to move"),
        "{}",
        c.prompt_text()
    );
    c.select_objects(&[id]).unwrap();
    c.refresh_addon_session();
    assert!(
        c.prompt_text().starts_with("Point to move from"),
        "{}",
        c.prompt_text()
    );
    c.session_point([0.0, 0.0, 0.0]).unwrap();
    c.session_hover([5.0, 0.0, 0.0]);
    assert!(
        !c.addon_session().unwrap().hover_lines.is_empty(),
        "moved selection previewed"
    );
    let undo = c.coordinator().undo_len();
    c.session_point([10.0, 5.0, 0.0]).unwrap();
    assert_eq!(center(&c, id), [10.0, 5.0, 0.0]);
    assert_eq!(c.coordinator().undo_len(), undo + 1);
    assert_eq!(c.document().entities.len(), 1, "Move does not copy");

    // Polar array: 6 items → 5 copies, one transaction, one Undo.
    c.run_addon_command("TransformArrayPolar").unwrap();
    c.submit("0,0").unwrap();
    c.submit("6").unwrap();
    c.submit("").unwrap();
    assert_eq!(c.document().entities.len(), 6);
    c.undo().unwrap();
    assert_eq!(c.document().entities.len(), 1);

    // Mirror keeps the original (Copy=Yes) across the CPlane y axis.
    c.run_addon_command("TopIconMirror").unwrap();
    c.submit("YAxis").unwrap();
    assert_eq!(c.document().entities.len(), 2);
    let mirrored = c
        .document()
        .entities
        .values()
        .map(|e| e.placement.point([0.0; 3]).unwrap())
        .find(|p| p[0] < 0.0)
        .unwrap();
    assert!((mirrored[0] + 10.0).abs() < 1e-9 && (mirrored[1] - 5.0).abs() < 1e-9);

    // Scale 3D with a typed factor; Rotate with a typed angle.
    c.run_addon_command("TransformScaleScale3D").unwrap();
    c.submit("10,5").unwrap();
    c.submit("2").unwrap();
    let e = c.document().entities.get(&id).unwrap();
    assert!((e.placement.0[0][0] - 2.0).abs() < 1e-12);
    c.run_addon_command("TopIconRotate").unwrap();
    c.submit("10,5").unwrap();
    c.submit("90").unwrap();
    assert!(c.addon_session().is_none());
    fs::remove_dir_all(&tmp).unwrap();
}

#[test]
fn gem_cutter_uses_its_panel_and_command_line() {
    use om9_core::geometry::Mesh;
    use om9_core::model::{Entity, Extension, GEM_EXTENSION_SLOT};
    let tmp = std::env::temp_dir().join(format!("om9-app-cutter-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();
    let feats = features();
    let names: Vec<FeatureName> = feats
        .iter()
        .map(|(id, c)| FeatureName {
            id,
            command: c.as_deref(),
        })
        .collect();
    let mut doc = Document::new("cutter");
    // A Ø2 gem at (5,0,0), then moved +1 in z by its placement: the cutter must follow.
    let mut gem = Entity::new(
        "Gem 2.00 mm",
        Geometry::Mesh(Mesh {
            vertices: vec![
                [4.0, 0.0, 0.0],
                [6.0, 0.0, 0.0],
                [5.0, 1.0, 0.0],
                [5.0, 0.0, -1.0],
            ],
            faces: vec![vec![0, 1, 2], vec![0, 3, 1], vec![1, 3, 2], vec![2, 3, 0]],
            normals: vec![],
            colors: vec![],
            corner_uvs: Vec::new(),
        }),
        doc.current_layer,
    );
    gem.placement = om9_core::geometry::Transform::translation([0.0, 0.0, 1.0]);
    gem.extensions.insert(
        GEM_EXTENSION_SLOT.to_owned(),
        Extension::new(
            1,
            r#"{"cut":"round","diameter_mm":2.0,"center":[5,0,0],"axis":[0,0,1],"x":[1,0,0]}"#,
        )
        .unwrap(),
    );
    let gid = gem.id;
    doc.entities.insert(gid, gem);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.attach_addons(host_with(&tmp, &["gem_cutter"]), &names);
    assert_eq!(
        c.addon_code("gvAzureCutter").as_deref(),
        Some("BuilderGemCutter")
    );
    c.submit("gvAzureCutter").unwrap();
    assert!(
        c.prompt_text().starts_with("Select gems"),
        "{}",
        c.prompt_text()
    );
    assert!(
        c.addon_builder().is_some_and(|b| b.interactive),
        "panel opens with the command"
    );
    c.select_objects(&[gid]).unwrap();
    c.refresh_addon_session();
    assert!(c.prompt_text().contains("1 cutters"), "{}", c.prompt_text());
    assert!(
        !c.addon_session().unwrap().lines.is_empty(),
        "cutter preview"
    );
    // Command-line option sets the panel value; the preview follows.
    c.submit("GirdleScale=120").unwrap();
    assert_eq!(c.addon_builder().unwrap().tokens()[0].1, "GirdleScale=120");
    let before = c.document().entities.len();
    c.submit("").unwrap();
    assert!(
        c.addon_session().is_none() && c.addon_builder().is_none(),
        "{} / {:?}",
        c.prompt_text(),
        c.take_addon_messages()
    );
    assert_eq!(c.document().entities.len(), before + 1);
    let cutter = c
        .document()
        .entities
        .values()
        .find(|e| e.label.starts_with("Gem Cutter"))
        .unwrap();
    assert!(
        cutter.extension(GEM_EXTENSION_SLOT).is_none(),
        "a cutter is not a gem"
    );
    let Geometry::Mesh(m) = &cutter.geometry else {
        panic!()
    };
    let top = m.vertices.iter().map(|v| v[2]).fold(f64::MIN, f64::max);
    assert!(
        (top - 2.0).abs() < 1e-9,
        "top 50 % of Ø2 above the moved girdle at z=1: {top}"
    );
    let widest = m
        .vertices
        .iter()
        .map(|v| ((v[0] - 5.0).powi(2) + v[1] * v[1]).sqrt())
        .fold(0.0, f64::max);
    assert!(
        (widest - 1.2).abs() < 1e-9,
        "GirdleScale 120 % of Ø2: {widest}"
    );
    // The values accepted become the panel defaults next time.
    c.submit("gvAzureCutter").unwrap();
    assert_eq!(c.addon_builder().unwrap().tokens()[0].1, "GirdleScale=120");
    c.cancel();
    fs::remove_dir_all(&tmp).unwrap();
}

/// Every command the bundled add-ons provide has an OM9 icon whose catalog code calls it
/// once the add-on is installed (menu palettes, toolbars and icon history share this lookup).
#[test]
fn every_addon_command_has_an_icon_that_calls_it() {
    let tmp = std::env::temp_dir().join(format!("om9-app-icons-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();
    let feats = features();
    let names: Vec<FeatureName> = feats
        .iter()
        .map(|(id, c)| FeatureName {
            id,
            command: c.as_deref(),
        })
        .collect();
    let mut c = Controller::new(Document::new("icons"), Limits::default()).unwrap();
    c.attach_addons(host(&tmp), &names);
    let text = fs::read_to_string(repo().join("addons/matrix/addon/om9_addon.toml")).unwrap();
    let codes: Vec<&str> = text
        .split("[[commands]]")
        .skip(1)
        .filter_map(|block| block.lines().find_map(|l| l.trim().strip_prefix("id = ")))
        .map(|v| v.trim_matches('"'))
        .collect();
    assert_eq!(codes.len(), om9_matrix::ALL.len());
    // OM9 draws a few commands with the same icon name (Ring Rail on the Top 11 bar and in
    // Tools): that icon runs one of them; both are the same Matrix command.
    let mut by_icon: std::collections::BTreeMap<&str, Vec<&str>> = Default::default();
    for code in &codes {
        if let Some(icon) = om9_app::icons::icon_for_command(code) {
            by_icon.entry(icon).or_default().push(code);
        }
    }
    for code in codes {
        let icon = om9_app::icons::icon_for_command(code)
            .unwrap_or_else(|| panic!("no OM9 icon for {code}"));
        let target = c
            .icon_target(icon, None)
            .unwrap_or_else(|| panic!("{icon}: no command"));
        assert!(target.addon, "{icon} → {target:?}");
        assert_eq!(target.enabled, code != "gvJoinHistory", "{icon} → {target:?}");
        assert!(by_icon[icon].contains(&target.command.as_str()), "{icon}: {} is not one of {:?}", target.command, by_icon[icon]);
        if by_icon[icon].len() == 1 {
            assert_eq!(target.command, code, "{icon}");
        }
        assert_eq!(c.is_available(code), code != "gvJoinHistory", "{code}");
    }
    fs::remove_dir_all(&tmp).unwrap();
}

fn gem_record(c: &Controller, id: om9_core::identity::ObjectId) -> serde_json::Value {
    let e = c.document().entities.get(&id).unwrap();
    serde_json::from_str(&e.extension(om9_core::model::GEM_EXTENSION_SLOT).unwrap().payload).unwrap()
}

/// Gem Loader → Halo → Gem Reporter (material) → replace by another cut → Gem Update →
/// prongs, each through the controller as from the icons, one Undo step each.
#[test]
fn jewelry_flow_through_the_controller() {
    let tmp = std::env::temp_dir().join(format!("om9-app-jewelry-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();
    let feats = features();
    let names: Vec<FeatureName> = feats
        .iter()
        .map(|(id, c)| FeatureName {
            id,
            command: c.as_deref(),
        })
        .collect();
    let mut c = Controller::new(Document::new("ring"), Limits::default()).unwrap();
    c.attach_addons(host(&tmp), &names);

    // Gem Loader: the panel opens with the command; the size is typed on the command line.
    c.run_addon_command("BuilderGemLoader").unwrap();
    assert!(c.addon_builder().is_some_and(|b| b.interactive));
    c.submit("Length=6.5").unwrap();
    assert!(c.prompt_text().contains("Round 6.50 mm"), "{}", c.prompt_text());
    c.session_point([0.0, 0.0, 0.0]).unwrap();
    let undo = c.coordinator().undo_len();
    c.submit("").unwrap();
    assert!(c.addon_session().is_none() && c.addon_builder().is_none());
    assert_eq!(c.coordinator().undo_len(), undo + 1);
    let center = *c.document().entities.keys().next().unwrap();
    assert_eq!(gem_record(&c, center)["diameter_mm"], 6.5);

    // Halo around it.
    c.select_objects(&[center]).unwrap();
    c.run_addon_command("BuilderHalo").unwrap();
    c.refresh_addon_session();
    assert!(c.prompt_text().contains("22 accent stones"), "{}", c.prompt_text());
    c.submit("").unwrap();
    assert_eq!(c.document().entities.len(), 1 + 22 + 1);

    // Gem Reporter assigns ruby to the center stone.
    c.select_objects(&[center]).unwrap();
    c.run_addon_command("BuilderGemReporter").unwrap();
    c.submit("Material=ruby").unwrap();
    c.submit("").unwrap();
    assert_eq!(gem_record(&c, center)["material"], "ruby");
    assert!(c.take_addon_messages().iter().any(|m| m.contains("ruby")));

    // The loader replaces the selected stone by an oval in the same place.
    c.select_objects(&[center]).unwrap();
    c.run_addon_command("BuilderGemLoader").unwrap();
    c.submit("Cut=oval").unwrap();
    c.submit("Length=8").unwrap();
    c.submit("").unwrap();
    assert!(!c.document().entities.contains_key(&center), "old stone replaced");
    let oval = c
        .document()
        .entities
        .iter()
        .find(|(_, e)| e.label.starts_with("Gem Oval"))
        .map(|(id, _)| *id)
        .unwrap();
    assert_eq!(gem_record(&c, oval)["center"], serde_json::json!([0.0, 0.0, 0.0]));
    assert_eq!(c.document().entities.len(), 1 + 22 + 1);

    // Gem Update rebuilds the stone in place (same object).
    c.select_objects(&[oval]).unwrap();
    c.run_addon_command("BuilderGemUpdate").unwrap();
    c.submit("").unwrap();
    assert!(c.document().entities.contains_key(&oval));

    // Four prongs on the oval.
    c.select_objects(&[oval]).unwrap();
    c.run_addon_command("BuilderProngAdder").unwrap();
    c.refresh_addon_session();
    c.submit("").unwrap();
    let prongs = c
        .document()
        .entities
        .values()
        .filter(|e| e.label == "Prong")
        .count();
    assert_eq!(prongs, 4);
    // One Undo removes the prongs only.
    c.undo().unwrap();
    assert_eq!(
        c.document().entities.values().filter(|e| e.label == "Prong").count(),
        0
    );
    fs::remove_dir_all(&tmp).unwrap();
}

/// A stone remembers the prongs made on it; Match Attributes makes the same prongs on another
/// stone (two selection groups through the controller).
#[test]
fn styles_follow_stones_through_the_controller() {
    let tmp = std::env::temp_dir().join(format!("om9-app-styles-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();
    let feats = features();
    let names: Vec<FeatureName> = feats
        .iter()
        .map(|(id, c)| FeatureName {
            id,
            command: c.as_deref(),
        })
        .collect();
    let mut c = Controller::new(Document::new("styles"), Limits::default()).unwrap();
    c.attach_addons(host(&tmp), &names);
    c.run_addon_command("BuilderGemLoader").unwrap();
    c.session_point([0.0, 0.0, 0.0]).unwrap();
    c.session_point([20.0, 0.0, 0.0]).unwrap();
    c.submit("").unwrap();
    let mut stones: Vec<_> = c.document().entities.keys().copied().collect();
    stones.sort_by(|a, b| {
        gem_record(&c, *a)["center"][0]
            .as_f64()
            .unwrap()
            .total_cmp(&gem_record(&c, *b)["center"][0].as_f64().unwrap())
    });
    let (a, b) = (stones[0], stones[1]);
    // Six prongs on stone a: a remembers them.
    c.select_objects(&[a]).unwrap();
    c.run_addon_command("BuilderProngAdder").unwrap();
    c.submit("Count=6").unwrap();
    c.submit("").unwrap();
    let prongs = |c: &Controller| c.document().entities.values().filter(|e| e.label == "Prong").count();
    assert_eq!(prongs(&c), 6);
    let style = &gem_record(&c, a)["styles"]["BuilderProngAdder"];
    assert_eq!(style["prong_count"], 6, "{style}");
    assert!(style.get("halo_size").is_none(), "only the prong builder's own values: {style}");
    // Match Attributes: first the stone that gets the style, then the stone that has it.
    c.select_objects(&[b]).unwrap();
    c.run_addon_command("OthersMatchAttributes").unwrap();
    c.submit("").unwrap();
    c.select_objects(&[a]).unwrap();
    c.refresh_addon_session();
    assert!(c.prompt_text().contains("BuilderProngAdder"), "{}", c.prompt_text());
    c.submit("").unwrap();
    assert!(c.addon_session().is_none(), "{}", c.prompt_text());
    assert_eq!(prongs(&c), 12);
    assert_eq!(gem_record(&c, b)["styles"]["BuilderProngAdder"]["prong_count"], 6);
    fs::remove_dir_all(&tmp).unwrap();
}

/// Surface commands make NURBS surfaces in the document, and later commands see them (their
/// exact definition and their tessellation).
#[test]
fn surfaces_round_trip_through_the_document() {
    let tmp = std::env::temp_dir().join(format!("om9-app-surfaces-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();
    let feats = features();
    let names: Vec<FeatureName> = feats
        .iter()
        .map(|(id, c)| FeatureName {
            id,
            command: c.as_deref(),
        })
        .collect();
    let mut c = Controller::new(Document::new("surfaces"), Limits::default()).unwrap();
    c.attach_addons(host(&tmp), &names);
    c.run_addon_command("SurfacePlaneCornertoCorner").unwrap();
    c.session_point([0.0, 0.0, 0.0]).unwrap();
    c.session_point([4.0, 3.0, 0.0]).unwrap();
    assert!(c.addon_session().is_none(), "{}", c.prompt_text());
    let plane = *c.document().entities.keys().next().unwrap();
    assert!(matches!(
        c.document().entities[&plane].geometry,
        Geometry::Surface(_)
    ));
    // Offset it 2 mm: the command reads the exact surface back.
    c.select_objects(&[plane]).unwrap();
    c.run_addon_command("SurfaceOffsetSurface").unwrap();
    // Rhino 5 OffsetSrf: the surface is already selected, so the next prompt is the distance.
    c.submit("2").unwrap();
    if c.addon_session().is_some() {
        c.submit("").unwrap();
    }
    assert!(c.addon_session().is_none(), "{}", c.prompt_text());
    assert_eq!(c.document().entities.len(), 2);
    // Extrude Normal to Surface (Rhino 5 Fin) reads the exact plane.
    c.select_objects(&[]).unwrap();
    c.run_addon_command("CurvePolylinePolyline").unwrap();
    c.session_point([1.0, 1.0, 0.0]).unwrap();
    c.session_point([3.0, 1.0, 0.0]).unwrap();
    c.submit("").unwrap();
    let curve = *c
        .document()
        .entities
        .keys()
        .find(|id| matches!(c.document().entities[*id].geometry, Geometry::Curve(_)))
        .unwrap();
    c.select_objects(&[curve, plane]).unwrap();
    c.run_addon_command("SurfaceExtrudeCurveNormalToSurface").unwrap();
    c.refresh_addon_session();
    // Rhino 5 Fin: a place on the curve, its height, then Enter for a uniform height.
    assert!(c.prompt_text().contains("Pick a location on the curve"), "{}", c.prompt_text());
    c.session_point([2.0, 1.0, 0.0]).unwrap();
    assert!(c.prompt_text().starts_with("Height"), "{}", c.prompt_text());
    c.submit("1").unwrap();
    c.submit("").unwrap();
    assert!(c.addon_session().is_none(), "{}", c.prompt_text());
    assert_eq!(c.document().entities.len(), 4);
    fs::remove_dir_all(&tmp).unwrap();
}

/// SubD: a cage from the add-on keeps its faces through the document, so a face selection
/// made by one command is used by the next (Paint Selection → Extrude → Subdivision Level).
#[test]
fn subd_face_selection_survives_the_document() {
    let tmp = std::env::temp_dir().join(format!("om9-app-subd-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();
    let feats = features();
    let names: Vec<FeatureName> = feats
        .iter()
        .map(|(id, c)| FeatureName {
            id,
            command: c.as_deref(),
        })
        .collect();
    let mut c = Controller::new(Document::new("subd"), Limits::default()).unwrap();
    c.attach_addons(host(&tmp), &names);
    c.run_addon_command("ClayooPrimitivesBox").unwrap();
    c.session_point([0.0, 0.0, 0.0]).unwrap();
    c.session_point([4.0, 4.0, 0.0]).unwrap();
    c.submit("4").unwrap();
    assert!(c.addon_session().is_none(), "{}", c.prompt_text());
    let cube = *c.document().entities.keys().next().unwrap();
    let tris = |c: &Controller| match &c.document().entities[&cube].geometry {
        Geometry::Mesh(m) => m.faces.len(),
        g => panic!("{g:?}"),
    };
    assert_eq!(tris(&c), 6 * 4 * 2, "a 2 × 2 cage on every side");
    // Paint one top face, extrude it.
    c.select_objects(&[cube]).unwrap();
    c.run_addon_command("ClayooSelectionPaintSelection").unwrap();
    c.session_point([1.0, 1.0, 0.0]).unwrap();
    assert!(c.prompt_text().contains("1 faces selected"), "{}", c.prompt_text());
    c.submit("").unwrap();
    c.select_objects(&[cube]).unwrap();
    c.run_addon_command("ClayooEditExtrude").unwrap();
    c.refresh_addon_session();
    assert!(c.prompt_text().contains("Distance"), "the selection is used: {}", c.prompt_text());
    c.submit("2").unwrap();
    assert!(c.addon_session().is_none(), "{}", c.prompt_text());
    assert_eq!(tris(&c), (24 + 4) * 2, "four side walls");
    // Smooth it.
    c.select_objects(&[cube]).unwrap();
    c.run_addon_command("ClayooEditSubdivisionLevel").unwrap();
    c.submit("2").unwrap();
    assert!(c.addon_session().is_none(), "{}", c.prompt_text());
    assert_eq!(tris(&c), 28 * 16 * 2);
    fs::remove_dir_all(&tmp).unwrap();
}

/// A deformation (Taper) rebuilds a moved mesh in place: the object keeps its id and
/// placement, its world shape follows the deformation.
#[test]
fn deformations_rebuild_objects_in_place() {
    let tmp = std::env::temp_dir().join(format!("om9-app-deform-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();
    let feats = features();
    let names: Vec<FeatureName> = feats
        .iter()
        .map(|(id, c)| FeatureName {
            id,
            command: c.as_deref(),
        })
        .collect();
    let mut c = Controller::new(Document::new("deform"), Limits::default()).unwrap();
    c.attach_addons(host(&tmp), &names);
    // A 10 × 2 × 2 box (Cube Cutter) at the origin, then moved +5 in x.
    c.run_addon_command("BuilderPlaneandCubeCutters").unwrap();
    c.submit("X=10").unwrap();
    c.submit("Y=2").unwrap();
    c.submit("Z=2").unwrap();
    c.submit("").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    c.select_objects(&[id]).unwrap();
    c.run_addon_command("TopIconMove").unwrap();
    c.refresh_addon_session();
    c.session_point([0.0, 0.0, 0.0]).unwrap();
    c.session_point([5.0, 0.0, 0.0]).unwrap();
    let placement = c.document().entities[&id].placement;
    // Taper along x from 0 to 10 (world): the far end shrinks to half.
    c.select_objects(&[id]).unwrap();
    c.run_addon_command("TransformTaper").unwrap();
    c.refresh_addon_session();
    c.session_point([0.0, 0.0, 0.0]).unwrap();
    c.session_point([10.0, 0.0, 0.0]).unwrap();
    c.submit("1").unwrap();
    c.submit("0.5").unwrap();
    assert!(c.addon_session().is_none(), "{}", c.prompt_text());
    let e = &c.document().entities[&id];
    assert_eq!(e.placement, placement, "the object keeps its placement");
    let Geometry::Mesh(m) = &e.geometry else { panic!() };
    let world: Vec<[f64; 3]> = m.vertices.iter().map(|v| e.placement.point(*v).unwrap()).collect();
    let at_end = world.iter().filter(|v| (v[0] - 10.0).abs() < 1e-6).map(|v| v[1].abs()).fold(0.0, f64::max);
    assert!((at_end - 0.5).abs() < 1e-6, "half width at x = 10: {at_end}");
    fs::remove_dir_all(&tmp).unwrap();
}

/// Vietnamese (the package default): menu names, command names, sub-options and prompts are
/// shown in Vietnamese; English and Vietnamese option names both work when typed.
#[test]
fn vietnamese_interface_keeps_english_typing() {
    let tmp = std::env::temp_dir().join(format!("om9-app-session-vi-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();
    let feats = features();
    let names: Vec<FeatureName> = feats
        .iter()
        .map(|(id, c)| FeatureName {
            id,
            command: c.as_deref(),
        })
        .collect();
    let mut c = Controller::new(Document::new("vi"), Limits::default()).unwrap();
    assert_eq!(c.tr().menu("Curve"), "Curve", "English without add-ons");
    let host = host_lang(&tmp, "vi");
    c.attach_addons(host.clone(), &names);
    assert_eq!(c.tr().menu("Curve"), "Đường cong");
    assert_eq!(
        c.tr().command("CurveLineSingleLine", "Single Line"),
        "Đoạn thẳng"
    );

    c.run_addon_command("CurvePolylinePolyline").unwrap();
    assert!(
        c.prompt_text().starts_with("Điểm đầu đường gấp khúc"),
        "{}",
        c.prompt_text()
    );
    let tokens = c.addon_session().unwrap().tokens(c.tr());
    assert!(
        tokens
            .iter()
            .any(|(n, t)| n == "PersistentClose" && t == "Luôn khép kín=Không"),
        "{tokens:?}"
    );
    // Typed in English, in Vietnamese with accents and without.
    c.submit("PersistentClose=Yes").unwrap();
    assert!(
        c.addon_session()
            .unwrap()
            .tokens(c.tr())
            .iter()
            .any(|(_, t)| t == "Luôn khép kín=Có")
    );
    c.submit("luon khep kin=khong").unwrap();
    assert!(
        c.addon_session()
            .unwrap()
            .tokens(c.tr())
            .iter()
            .any(|(_, t)| t == "Luôn khép kín=Không")
    );
    c.submit("Luôn khép kín=Có").unwrap();
    assert!(
        c.addon_session()
            .unwrap()
            .tokens(c.tr())
            .iter()
            .any(|(_, t)| t == "Luôn khép kín=Có")
    );
    c.session_point([0.0, 0.0, 0.0]).unwrap();
    c.session_point([10.0, 0.0, 0.0]).unwrap();
    c.session_point([10.0, 10.0, 0.0]).unwrap();
    assert!(
        c.prompt_text()
            .starts_with("Điểm tiếp theo của đường gấp khúc"),
        "{}",
        c.prompt_text()
    );
    c.submit("hoan tac").unwrap(); // Undo by its Vietnamese name
    c.submit("").unwrap();
    assert!(c.addon_session().is_none());
    assert_eq!(c.document().entities.len(), 1);

    // English on request: every text as written.
    host.lock()
        .unwrap()
        .set_setting("om9_matrix", "language", "en")
        .unwrap();
    c.refresh_addon_names(&names);
    assert_eq!(c.tr().menu("Curve"), "Curve");
    c.run_addon_command("CurvePolylinePolyline").unwrap();
    assert!(
        c.prompt_text().starts_with("Start of polyline"),
        "{}",
        c.prompt_text()
    );
    c.cancel_addon_session();
}

#[test]
fn general_options_persist_matrix_language_and_change_line_summary() {
    use eframe::egui::{self, Event, Key, Modifiers, PointerButton, Pos2, Rect, Vec2};
    use om9_app::ui::MatrixApp;

    let tmp = std::env::temp_dir().join(format!("om9-options-language-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&tmp).unwrap();
    let host = host_lang(&tmp, "vi");
    let mut app = MatrixApp::new(Document::with_template_tolerance("Language options")).unwrap();
    app.controller.attach_addons(host.clone(), &[]);
    let ctx = egui::Context::default();
    let frame = |app: &mut MatrixApp, events| {
        let mut output = ctx.run_ui(egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 900.))),
            events,
            ..Default::default()
        }, |ui| app.show(ui));
        output.textures_delta.clear();
        output
    };
    let click = |app: &mut MatrixApp, pos| {
        frame(app, vec![Event::PointerMoved(pos)]);
        for pressed in [true, false] {
            frame(app, vec![Event::PointerButton {
                pos, button: PointerButton::Primary, pressed, modifiers: Modifiers::NONE,
            }]);
        }
    };
    let find_text = |app: &mut MatrixApp, label: &str| {
        frame(app, vec![]).shapes.iter().find_map(|shape| match &shape.shape {
            egui::Shape::Text(text) if text.galley.job.text == label => {
                Some(text.pos + text.galley.size() / 2.)
            }
            _ => None,
        }).unwrap_or_else(|| panic!("missing Options label {label:?}"))
    };
    let line_summary = |app: &mut MatrixApp| {
        app.controller.run_addon_command("CurveLineSingleLine").unwrap();
        app.controller.session_point([0., 0., 0.]).unwrap();
        app.controller.session_point([3., 0., 0.]).unwrap();
        app.controller.take_addon_messages()
    };
    assert!(line_summary(&mut app).iter().any(|line| line == "Đoạn thẳng: dài 3 mm"));
    frame(&mut app, vec![]);
    let cmd = app.command_input_rect().unwrap().center();
    click(&mut app, cmd);
    frame(&mut app, vec![Event::Text("Options".into())]);
    frame(&mut app, vec![Event::Key {
        key: Key::Enter, physical_key: None, pressed: true, repeat: false, modifiers: Modifiers::NONE,
    }]);
    find_text(&mut app, "Single-click maximize");
    let general = find_text(&mut app, "General");
    click(&mut app, general);
    find_text(&mut app, "OM9 Matrix language");
    let prefs = tmp.join("config/userprefs.toml");
    let blocked_save = prefs.with_extension("toml.tmp");
    fs::create_dir(&blocked_save).unwrap();
    for expected in ["Đường cong", "Curve"] {
        let vietnamese = find_text(&mut app, "Tiếng Việt");
        click(&mut app, vietnamese);
        let english = find_text(&mut app, "English");
        click(&mut app, english);
        assert_eq!(app.controller.tr().menu("Curve"), expected);
        if expected == "Đường cong" {
            assert!(app.status_line().contains("preferences"));
            fs::remove_dir(&blocked_save).unwrap();
        }
    }
    let (saved, warning) = om9_addons::prefs::Prefs::load(&prefs);
    assert!(warning.is_none());
    assert_eq!(saved.addons["om9_matrix"].settings["language"],
        om9_addons::settings::SettingValue::Str("en".into()));
    assert!(line_summary(&mut app).iter().any(|line| line == "Line: length 3 mm"));
    fs::remove_dir_all(&tmp).unwrap();
}

#[test]
fn curves_are_picked_by_clicking_and_sketch_follows_the_pointer() {
    let tmp = std::env::temp_dir().join(format!("om9-app-pick-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();
    let feats = features();
    let names: Vec<FeatureName> = feats
        .iter()
        .map(|(id, c)| FeatureName {
            id,
            command: c.as_deref(),
        })
        .collect();
    let mut c = Controller::new(Document::new("pick"), Limits::default()).unwrap();
    c.attach_addons(host(&tmp), &names);
    let line = |c: &mut Controller, a: [f64; 3], b: [f64; 3]| {
        let before: Vec<_> = c.document().entities.keys().copied().collect();
        c.run_addon_command("CurvePolylinePolyline").unwrap();
        c.session_point(a).unwrap();
        c.session_point(b).unwrap();
        c.submit("").unwrap();
        *c.document().entities.keys().find(|k| !before.contains(k)).unwrap()
    };
    let a = line(&mut c, [-10.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let b = line(&mut c, [0.0, -10.0, 0.0], [0.0, 10.0, 0.0]);
    assert_ne!(a, b);
    // Fillet: click the first line on +x, the second on +y.
    c.clear_selection();
    c.run_addon_command("CurveFilletCurves").unwrap();
    assert!(c.prompt_text().starts_with("Select first curve to fillet"), "{}", c.prompt_text());
    c.note_session_pick(Some([5.0, 0.0, 0.0]));
    c.select_objects(&[a]).unwrap();
    c.refresh_addon_session();
    assert!(c.prompt_text().starts_with("Select second curve to fillet"), "{}", c.prompt_text());
    c.note_session_pick(Some([0.0, 6.0, 0.0]));
    c.select_objects(&[a, b]).unwrap();
    c.refresh_addon_session();
    assert!(c.addon_session().is_none(), "the fillet is made on the second pick");
    assert_eq!(c.document().entities.len(), 3, "two trimmed lines and the arc");
    // The arc (an OM9 circle with an angle domain): radius 1 around (1, 1).
    let arc = c.document().entities.values().find_map(|e| match &e.geometry {
        Geometry::Curve(Curve::Circle { center, radius, domain, .. }) => Some((*center, *radius, *domain)),
        _ => None,
    });
    let (center, radius, domain) = arc.expect("a fillet arc");
    assert!((center[0] - 1.0).abs() < 1e-6 && (center[1] - 1.0).abs() < 1e-6 && (radius - 1.0).abs() < 1e-9, "{center:?} {radius}");
    assert!((domain[1] - domain[0] - std::f64::consts::FRAC_PI_2).abs() < 1e-6, "a quarter: {domain:?}");
    // Sketch: click, move, click.
    let before = c.document().entities.len();
    c.clear_selection();
    c.run_addon_command("CurveFreeFormSketch").unwrap();
    c.session_point([0.0, 20.0, 0.0]).unwrap();
    for i in 1..60 {
        let x = i as f64 * 0.1;
        c.session_hover([x, 20.0 + (x * 0.8).sin(), 0.0]);
    }
    c.session_point([6.0, 20.0 + (4.8f64).sin(), 0.0]).unwrap();
    assert!(c.addon_session().is_none());
    assert_eq!(c.document().entities.len(), before + 1, "the stroke became a curve");
    fs::remove_dir_all(&tmp).unwrap();
}

/// `brep_data`: a surface with Rhino trim loops reaches the add-on exactly (its surface and
/// its loops run along it). Untrim with KeepTrimObjects gives the whole surface back without
/// the trim record, and leaves the outer loop and the hole as curves on the surface.
#[test]
fn trimmed_surfaces_reach_brep_data_commands_exactly() {
    use om9_core::model::{Entity, TRIM_SURFACE_EXTENSION_SLOT};
    use om9_core::rhino_semantics::{TrimmedSurface, set_trimmed_surface, trimmed_surface_of};
    let tmp = std::env::temp_dir().join(format!("om9-app-untrim-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();
    let feats = features();
    let names: Vec<FeatureName> = feats
        .iter()
        .map(|(id, c)| FeatureName {
            id,
            command: c.as_deref(),
        })
        .collect();
    let limits = Limits::default();
    let surface = om9_core::surface_kernel::plane([0., 0., 0.], [1., 0., 0.], [0., 1., 0.], 10., 10., &limits).unwrap();
    let trim = TrimmedSurface {
        underlying: surface.clone(),
        outer: vec![[0.1, 0.1], [0.9, 0.1], [0.9, 0.9], [0.1, 0.9]],
        inners: vec![vec![[0.4, 0.4], [0.6, 0.4], [0.6, 0.6], [0.4, 0.6]]],
        reversed: false,
    };
    let mut doc = Document::new("untrim");
    let mut patch = Entity::new("trimmed", Geometry::Surface(surface.clone()), doc.current_layer);
    set_trimmed_surface(&mut patch, &trim, &limits).unwrap();
    // A placement: the add-on works in world millimetres, the result goes back to the frame.
    patch.placement = om9_core::geometry::Transform::translation([0.0, 0.0, 2.0]);
    let id = patch.id;
    doc.entities.insert(id, patch);
    let mut c = Controller::new(doc, limits).unwrap();
    c.attach_addons(host(&tmp), &names);
    c.select_objects(&[id]).unwrap();
    c.run_addon_command("SurfaceSurfaceEditToolsUntrim").unwrap();
    assert!(c.prompt_text().contains("1 surfaces"), "{}", c.prompt_text());
    c.submit("KeepTrimObjects").unwrap();
    c.submit("").unwrap();
    assert!(c.addon_session().is_none(), "{}", c.prompt_text());
    let e = &c.document().entities[&id];
    assert_eq!(e.geometry, Geometry::Surface(surface), "the whole surface, in its own frame");
    assert!(!e.extensions.contains_key(TRIM_SURFACE_EXTENSION_SLOT), "no trims left");
    assert!(trimmed_surface_of(e, &limits).unwrap().is_none());
    // The two loops as closed curves at z 2: the outer square 1..9, the hole 4..6.
    let mut spans: Vec<(f64, f64)> = c
        .document()
        .entities
        .values()
        .filter(|x| x.id != id)
        .map(|x| {
            let Geometry::Curve(curve) = &x.geometry else { panic!("{:?}", x.geometry) };
            let pts: Vec<[f64; 3]> = match curve {
                Curve::Nurbs(n) if n.degree == 1 => n.poles.iter().map(|p| x.placement.point(*p).unwrap()).collect(),
                other => panic!("{other:?}"),
            };
            assert!(pts.iter().all(|p| (p[2] - 2.0).abs() < 1e-9), "{pts:?}");
            let lo = pts.iter().map(|p| p[0].min(p[1])).fold(f64::MAX, f64::min);
            let hi = pts.iter().map(|p| p[0].max(p[1])).fold(f64::MIN, f64::max);
            (lo, hi)
        })
        .collect();
    spans.sort_by(|a, b| a.0.total_cmp(&b.0));
    assert_eq!(spans.len(), 2, "{spans:?}");
    assert!((spans[0].0 - 1.0).abs() < 1e-9 && (spans[0].1 - 9.0).abs() < 1e-9, "{spans:?}");
    assert!((spans[1].0 - 4.0).abs() < 1e-9 && (spans[1].1 - 6.0).abs() < 1e-9, "{spans:?}");
    fs::remove_dir_all(&tmp).unwrap();
}

/// An add-on solid (`Geo::Brep`, here ExtrudeCrv Solid=Yes on a circle): with the B-Rep kernel
/// its faces are sewn into one exact solid; a build without the kernel keeps the add-on's
/// display mesh, a closed prism of the right volume.
#[test]
fn addon_solids_arrive_as_one_object() {
    use om9_core::model::Entity;
    let tmp = std::env::temp_dir().join(format!("om9-app-addon-solid-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();
    let feats = features();
    let names: Vec<FeatureName> = feats
        .iter()
        .map(|(id, c)| FeatureName {
            id,
            command: c.as_deref(),
        })
        .collect();
    let mut doc = Document::new("solid");
    let circle = Entity::new(
        "circle",
        Geometry::Curve(Curve::Circle {
            center: [0.0; 3],
            x: [1.0, 0.0, 0.0],
            y: [0.0, 1.0, 0.0],
            radius: 2.0,
            domain: [0.0, std::f64::consts::TAU],
            angle_domain: None,
        }),
        doc.current_layer,
    );
    let id = circle.id;
    doc.entities.insert(id, circle);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.attach_addons(host(&tmp), &names);
    c.select_objects(&[id]).unwrap();
    c.run_addon_command("SurfaceGVDExtrudeAll").unwrap();
    c.submit("Solid").unwrap();
    c.submit("3").unwrap();
    assert!(c.addon_session().is_none(), "{}", c.prompt_text());
    let made: Vec<&Geometry> = c.document().entities.values().filter(|e| e.id != id).map(|e| &e.geometry).collect();
    assert_eq!(made.len(), 1, "one solid");
    match made[0] {
        #[cfg(feature = "occt-brep")]
        Geometry::Brep(brep) => {
            let m = om9_kernel::creation::measure(brep, &Limits::default()).unwrap();
            assert_eq!(m.solids, 1);
            assert!((m.volume_mm3 - std::f64::consts::PI * 12.0).abs() < 1e-6, "{}", m.volume_mm3);
        }
        Geometry::Mesh(mesh) => {
            // Signed volume of the closed triangle mesh (divergence theorem).
            // (Faces are fans: polygons split from their first corner.)
            let v: f64 = mesh
                .faces
                .iter()
                .flat_map(|f| (1..f.len() - 1).map(move |k| [f[0], f[k], f[k + 1]]))
                .map(|t| {
                    let [a, b, c] = t.map(|i| mesh.vertices[i as usize]);
                    (a[0] * (b[1] * c[2] - b[2] * c[1]) - a[1] * (b[0] * c[2] - b[2] * c[0]) + a[2] * (b[0] * c[1] - b[1] * c[0])) / 6.0
                })
                .sum();
            // The display samples the circle at 96 points: within 0.1 % of π r² h.
            let exact = std::f64::consts::PI * 4.0 * 3.0;
            assert!((v.abs() - exact).abs() < exact * 1e-3, "{v} vs {exact}");
        }
        other => panic!("{other:?}"),
    }
    fs::remove_dir_all(&tmp).unwrap();
}

/// What the user does in the viewport, through the controller: each click goes through the
/// same selection call as the viewport (a plain click; inside an add-on object prompt the
/// picks add up). Sweep 2 picked rail, rail, sections; Boolean Difference on two
/// window-selected solids; right-click repeats the last add-on command.
#[test]
fn clicks_build_up_picks_and_right_click_repeats() {
    let tmp = std::env::temp_dir().join(format!("om9-app-clicks-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();
    let feats = features();
    let names: Vec<FeatureName> = feats.iter().map(|(id, c)| FeatureName { id, command: c.as_deref() }).collect();
    let mut c = Controller::new(Document::new("clicks"), Limits::default()).unwrap();
    c.attach_addons(host(&tmp), &names);
    let new_id = |c: &Controller, before: &[om9_core::identity::ObjectId]| {
        *c.document().entities.keys().find(|k| !before.contains(k)).unwrap()
    };
    let polyline = |c: &mut Controller, pts: &[[f64; 3]]| {
        let before: Vec<_> = c.document().entities.keys().copied().collect();
        c.clear_selection();
        c.run_addon_command("CurvePolylinePolyline").unwrap();
        for p in pts {
            c.session_point(*p).unwrap();
        }
        c.submit("").unwrap();
        new_id(c, &before)
    };
    let click = |c: &mut Controller, id| {
        let collecting = c.addon_collects_objects();
        c.select_root(Some(id), collecting).unwrap();
        c.refresh_addon_session();
    };
    let ra = polyline(&mut c, &[[-10.0, -20.0, 0.0], [-12.0, 0.0, 0.0], [-10.0, 20.0, 0.0]]);
    let rb = polyline(&mut c, &[[10.0, -20.0, 0.0], [12.0, 0.0, 0.0], [10.0, 20.0, 0.0]]);
    let p1 = polyline(&mut c, &[[-10.0, -20.0, 0.0], [0.0, -20.0, 5.0], [10.0, -20.0, 0.0]]);
    let p2 = polyline(&mut c, &[[-10.0, 20.0, 0.0], [0.0, 20.0, 3.0], [10.0, 20.0, 0.0]]);
    c.clear_selection();
    c.run_addon_command("SurfaceSweepSweep2Rails").unwrap();
    assert!(c.prompt_text().starts_with("Select first rail"), "{}", c.prompt_text());
    click(&mut c, ra);
    assert!(c.prompt_text().starts_with("Select second rail"), "{}", c.prompt_text());
    click(&mut c, rb);
    assert!(c.prompt_text().starts_with("Select cross section curves"), "{}", c.prompt_text());
    click(&mut c, p1);
    click(&mut c, p2);
    assert!(c.prompt_text().contains("2 selected"), "{}", c.prompt_text());
    c.submit("").unwrap();
    let before: Vec<_> = c.document().entities.keys().copied().collect();
    c.submit("").unwrap();
    assert!(c.addon_session().is_none(), "{}", c.prompt_text());
    assert_eq!(c.document().entities.len(), before.len() + 1, "the sweep surface");
    // Right-click with nothing running (and nothing selected): Sweep 2 again.
    c.clear_selection();
    assert!(c.repeat_addon_command().unwrap());
    assert!(c.prompt_text().starts_with("Select first rail"), "{}", c.prompt_text());
    assert!(c.cancel_addon_session());
    // Two boxes, window-selected, then Difference: cut at once, Enter.
    let boxed = |c: &mut Controller, a: [f64; 3], b: [f64; 3], h: &str| {
        let before: Vec<_> = c.document().entities.keys().copied().collect();
        c.clear_selection();
        c.run_addon_command("SolidBoxCornertoCornerHeight").unwrap();
        c.session_point(a).unwrap();
        c.session_point(b).unwrap();
        c.submit(h).unwrap();
        assert!(c.addon_session().is_none(), "{}", c.prompt_text());
        new_id(c, &before)
    };
    let big = boxed(&mut c, [0.0, 0.0, 0.0], [4.0, 4.0, 0.0], "4");
    let small = boxed(&mut c, [3.0, 3.0, 0.0], [5.0, 5.0, 0.0], "5");
    c.select_objects(&[big, small]).unwrap();
    let count = c.document().entities.len();
    c.run_addon_command("SolidDifference").unwrap();
    c.submit("").unwrap();
    assert!(c.addon_session().is_none(), "{}", c.prompt_text());
    assert_eq!(c.document().entities.len(), count - 1, "the cutter is gone");
    assert!(c.document().entities.contains_key(&big));
    fs::remove_dir_all(&tmp).unwrap();
}

/// The user's case: smooth curves drawn with Curve, the two rails picked before Sweep 2 is
/// typed or clicked, then the cross section; also `Sweep2` typed while Curve still waits.
#[test]
fn sweep2_from_drawn_curves_like_the_user() {
    let tmp = std::env::temp_dir().join(format!("om9-app-user-sweep2-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();
    let feats = features();
    let names: Vec<FeatureName> = feats.iter().map(|(id, c)| FeatureName { id, command: c.as_deref() }).collect();
    let mut c = Controller::new(Document::new("user-sweep2"), Limits::default()).unwrap();
    c.attach_addons(host(&tmp), &names);
    let new_id = |c: &Controller, before: &[om9_core::identity::ObjectId]| {
        *c.document().entities.keys().find(|k| !before.contains(k)).unwrap()
    };
    let curve = |c: &mut Controller, pts: &[[f64; 3]]| {
        let before: Vec<_> = c.document().entities.keys().copied().collect();
        c.clear_selection();
        c.submit("Curve").unwrap();
        eprintln!("curve prompt: {}", c.prompt_text());
        for p in pts {
            c.session_point(*p).unwrap();
        }
        c.submit("").unwrap();
        assert!(c.addon_session().is_none(), "{}", c.prompt_text());
        new_id(c, &before)
    };
    let ra = curve(&mut c, &[[-15.0, -20.0, 0.0], [-5.0, -5.0, 0.0], [-12.0, 8.0, 0.0], [-6.0, 20.0, 0.0]]);
    let rb = curve(&mut c, &[[8.0, -22.0, 0.0], [18.0, -5.0, 0.0], [10.0, 8.0, 0.0], [16.0, 21.0, 0.0]]);
    // The profile touches both rails at their starts.
    let pa = c.document().entities[&ra].geometry.clone();
    eprintln!("rail geometry: {:?}", std::mem::discriminant(&pa));
    let p1 = curve(&mut c, &[[-15.0, -20.0, 0.0], [-3.0, -21.0, 6.0], [8.0, -22.0, 0.0]]);
    for how in ["typed", "icon"] {
        c.clear_selection();
        c.select_objects(&[ra, rb]).unwrap();
        match how {
            "typed" => c.submit("Sweep2").unwrap(),
            _ => {
                let t = c.icon_target("SurfaceSweepSweep2Rails", None).expect("icon target");
                eprintln!("icon runs {} (addon {})", t.command, t.addon);
                c.run_addon_command(&t.command).unwrap();
            }
        }
        c.refresh_addon_session();
        eprintln!("[{how}] after start: {}", c.prompt_text());
        assert!(c.addon_session().is_some(), "{how}: no add-on session: {}", c.prompt_text());
        c.select_root(Some(p1), c.addon_collects_objects()).unwrap();
        c.refresh_addon_session();
        eprintln!("[{how}] after profile: {}", c.prompt_text());
        let before = c.document().entities.len();
        for _ in 0..3 {
            if c.addon_session().is_none() {
                break;
            }
            c.submit("").unwrap();
            eprintln!("[{how}] after enter: {}", c.prompt_text());
        }
        for m in c.take_addon_messages() {
            eprintln!("[{how}] message: {m:?}");
        }
        assert!(c.addon_session().is_none(), "{how}: {}", c.prompt_text());
        assert_eq!(c.document().entities.len(), before + 1, "{how}: the sweep surface");
    }
    // Sweep2 typed while Curve still waits for its first point: Curve closes, Sweep 2 runs.
    c.clear_selection();
    c.submit("Curve").unwrap();
    c.submit("Sweep2").unwrap();
    c.refresh_addon_session();
    assert!(c.prompt_text().starts_with("Select first rail"), "{}", c.prompt_text());
    fs::remove_dir_all(&tmp).unwrap();
}

/// The same through the real window code (`MatrixApp::show`): `Sweep2` typed on the command
/// line with the rails picked, the cross section picked, then right-clicks in the viewport.
#[test]
fn sweep2_through_the_window_with_right_clicks() {
    let tmp = std::env::temp_dir().join(format!("om9-app-ui-sweep2-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();
    let feats = features();
    let names: Vec<FeatureName> = feats.iter().map(|(id, c)| FeatureName { id, command: c.as_deref() }).collect();
    let mut app = om9_app::ui::MatrixApp::new(Document::new("ui-sweep2")).unwrap();
    app.controller.attach_addons(host(&tmp), &names);
    let size = [1400.0f32, 900.0];
    let run = |app: &mut om9_app::ui::MatrixApp, steps: serde_json::Value| -> serde_json::Value {
        om9_app::ui_replay::replay_app(app, size, 1.0, steps.as_array().unwrap()).unwrap()
    };
    let r = run(&mut app, serde_json::json!([{"wait": 2}]));
    let views = r["steps"][0]["views"].clone();
    eprintln!("views: {views}");
    let curve = |app: &mut om9_app::ui::MatrixApp, pts: &[[f64; 3]]| {
        let before: Vec<_> = app.controller.document().entities.keys().copied().collect();
        app.controller.clear_selection();
        app.controller.submit("Curve").unwrap();
        for p in pts {
            app.controller.session_point(*p).unwrap();
        }
        app.controller.submit("").unwrap();
        *app.controller.document().entities.keys().find(|k| !before.contains(k)).unwrap()
    };
    let ra = curve(&mut app, &[[-15.0, -20.0, 0.0], [-5.0, -5.0, 0.0], [-12.0, 8.0, 0.0], [-6.0, 20.0, 0.0]]);
    let rb = curve(&mut app, &[[8.0, -22.0, 0.0], [18.0, -5.0, 0.0], [10.0, 8.0, 0.0], [16.0, 21.0, 0.0]]);
    let p1 = curve(&mut app, &[[-15.0, -20.0, 0.0], [-3.0, -21.0, 6.0], [8.0, -22.0, 0.0]]);
    app.controller.clear_selection();
    app.controller.select_objects(&[ra, rb]).unwrap();
    let r = run(&mut app, serde_json::json!([{"text": "Sweep2"}, {"key": "Enter"}]));
    for s in r["steps"].as_array().unwrap() {
        eprintln!("step {}: prompt={} command={}", s["step"], s["prompt"], s["command"]);
    }
    eprintln!("history:\n{}", app.transcript.history());
    assert!(app.controller.addon_session().is_some(), "Sweep2 did not start: {}", app.controller.prompt_text());
    let collecting = app.controller.addon_collects_objects();
    app.controller.select_root(Some(p1), collecting).unwrap();
    app.controller.refresh_addon_session();
    let persp = views.as_array().unwrap().iter().find(|v| v["view"] == "Perspective").unwrap()["rect"].clone();
    // The bottom right of the view: the options window (top right) does not cover it.
    let at = [persp[0].as_f64().unwrap() + persp[2].as_f64().unwrap() - 40.0, persp[1].as_f64().unwrap() + persp[3].as_f64().unwrap() - 40.0];
    let before = app.controller.document().entities.len();
    let r = run(&mut app, serde_json::json!([{"click": at, "button": "right"}, {"click": at, "button": "right"}]));
    for s in r["steps"].as_array().unwrap() {
        eprintln!("step {}: prompt={}", s["step"], s["prompt"]);
    }
    eprintln!("history:\n{}", app.transcript.history());
    assert!(app.controller.addon_session().is_none(), "{}", app.controller.prompt_text());
    assert_eq!(app.controller.document().entities.len(), before + 1, "the sweep surface");
    fs::remove_dir_all(&tmp).unwrap();
}

/// Rhino 5 / Matrix 9: with the rails and a profile picked, Sweep 2 shows its options window
/// (MaintainHeight, Simplify …); a checkbox changes the option and OK creates the surface.
#[test]
fn sweep2_options_window_ok_creates_the_surface() {
    let tmp = std::env::temp_dir().join(format!("om9-app-ui-dialog-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();
    let feats = features();
    let names: Vec<FeatureName> = feats.iter().map(|(id, c)| FeatureName { id, command: c.as_deref() }).collect();
    let mut app = om9_app::ui::MatrixApp::new(Document::new("ui-dialog")).unwrap();
    // OM9_SHOT_LANG=vi: the same flow in Vietnamese (for the screenshot).
    let lang = std::env::var("OM9_SHOT_LANG").unwrap_or_else(|_| "en".into());
    app.controller.attach_addons(host_lang(&tmp, &lang), &names);
    let size = [1400.0f32, 900.0];
    let run = |app: &mut om9_app::ui::MatrixApp, steps: serde_json::Value| -> serde_json::Value {
        om9_app::ui_replay::replay_app(app, size, 1.0, steps.as_array().unwrap()).unwrap()
    };
    let curve = |app: &mut om9_app::ui::MatrixApp, pts: &[[f64; 3]]| {
        let before: Vec<_> = app.controller.document().entities.keys().copied().collect();
        app.controller.clear_selection();
        app.controller.submit("Curve").unwrap();
        for p in pts {
            app.controller.session_point(*p).unwrap();
        }
        app.controller.submit("").unwrap();
        *app.controller.document().entities.keys().find(|k| !before.contains(k)).unwrap()
    };
    let ra = curve(&mut app, &[[-15.0, -20.0, 0.0], [-5.0, -5.0, 0.0], [-12.0, 8.0, 0.0], [-6.0, 20.0, 0.0]]);
    let rb = curve(&mut app, &[[8.0, -22.0, 0.0], [18.0, -5.0, 0.0], [10.0, 8.0, 0.0], [16.0, 21.0, 0.0]]);
    let p1 = curve(&mut app, &[[-15.0, -20.0, 0.0], [-3.0, -21.0, 6.0], [8.0, -22.0, 0.0]]);
    app.controller.clear_selection();
    app.controller.select_objects(&[ra, rb]).unwrap();
    let r = run(&mut app, serde_json::json!([{"text": "Sweep2"}, {"key": "Enter"}]));
    // Picking the cross section: no window yet.
    assert!(r["steps"][2]["dialog"].is_null(), "{}", r["steps"][2]);
    let collecting = app.controller.addon_collects_objects();
    app.controller.select_root(Some(p1), collecting).unwrap();
    app.controller.refresh_addon_session();
    // A right click (Enter) ends the selection: the options window opens with the preview.
    let views = r["steps"][0]["views"].clone();
    let persp = views.as_array().unwrap().iter().find(|v| v["view"] == "Perspective").unwrap()["rect"].clone();
    let spot = [persp[0].as_f64().unwrap() + 40.0, persp[1].as_f64().unwrap() + persp[3].as_f64().unwrap() - 40.0];
    let r = run(&mut app, serde_json::json!([{"click": spot, "button": "right"}, {"wait": 2}]));
    let last = &r["steps"][2];
    eprintln!("dialog: {}", last["dialog"]);
    let title = if lang == "vi" { "Tuỳ chọn Quét 2 ray" } else { "Sweep 2 Rails Options" };
    assert_eq!(last["dialog"]["title"], title, "{last}");
    let rows = last["dialog"]["rows"].as_array().unwrap();
    for name in ["MaintainHeight", "Simplify", "RebuildPoints", "RefitTolerance", "SweepTolerance"] {
        assert!(rows.iter().any(|r| r["name"] == name), "{name} missing: {rows:?}");
    }
    let at = |r: &serde_json::Value, k: &str| r["controls"][k].clone();
    // OM9_SHOT_DIR=<dir>: the whole window at this moment, for a look at the dialog.
    if let Some(dir) = std::env::var_os("OM9_SHOT_DIR") {
        let out = Path::new(&dir).join("sweep2-options.png");
        om9_app::ui_replay::screenshot(&mut app, size, 1.0, 40, &out).unwrap();
        eprintln!("screenshot: {}", out.display());
    }
    let check = at(last, "Dialog:MaintainHeight");
    assert!(check.is_array(), "no checkbox: {}", last["controls"]);
    let r = run(&mut app, serde_json::json!([{"click": check}, {"wait": 2}]));
    let after = &r["steps"][2];
    let row = after["dialog"]["rows"].as_array().unwrap().iter().find(|r| r["name"] == "MaintainHeight").unwrap().clone();
    assert_eq!(row["value"], "Yes", "{after}");
    let rebuild = at(after, "Dialog:Simplify=Rebuild");
    let r = run(&mut app, serde_json::json!([{"click": rebuild}, {"wait": 2}]));
    let after = &r["steps"][2];
    let row = after["dialog"]["rows"].as_array().unwrap().iter().find(|r| r["name"] == "Simplify").unwrap().clone();
    assert_eq!(row["value"], "Rebuild", "{after}");
    let before = app.controller.document().entities.len();
    let ok = at(after, "Dialog:OK");
    let r = run(&mut app, serde_json::json!([{"click": ok}, {"wait": 2}]));
    eprintln!("history:\n{}", app.transcript.history());
    assert!(app.controller.addon_session().is_none(), "{}", app.controller.prompt_text());
    assert!(r["steps"][2]["dialog"].is_null());
    assert_eq!(app.controller.document().entities.len(), before + 1, "the sweep surface");
    fs::remove_dir_all(&tmp).unwrap();
}

/// The surface commands that have options show their options window once they have
/// something to make (Rhino 5: Surface From Curve Network, Patch Surface Options …).
/// Rhino's Rebuild window: "Point count: (2) [10]", "Degree: (1) [3]", Delete input, Create on
/// current layer, Preserve end tangents, Select Master Curve, the maximum deviation and a
/// Preview button; and the Sweep 2 Rails window in Rhino's groups, with Add Slash.
#[test]
fn rebuild_and_sweep2_windows_follow_rhino() {
    let tmp = std::env::temp_dir().join(format!("om9-app-rebuild-window-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();
    let feats = features();
    let names: Vec<FeatureName> = feats.iter().map(|(id, c)| FeatureName { id, command: c.as_deref() }).collect();
    let mut c = Controller::new(Document::new("rebuild-window"), Limits::default()).unwrap();
    c.attach_addons(host(&tmp), &names);
    let add = |c: &mut Controller, cmd: &str, pts: &[[f64; 3]]| {
        let before: Vec<_> = c.document().entities.keys().copied().collect();
        c.clear_selection();
        c.submit(cmd).unwrap();
        for p in pts {
            c.session_point(*p).unwrap();
        }
        c.submit("").unwrap();
        *c.document().entities.keys().find(|k| !before.contains(k)).unwrap()
    };
    let line = add(&mut c, "Line", &[[0.0, 0.0, 0.0], [10.0, 0.0, 0.0]]);
    c.select_objects(&[line]).unwrap();
    c.submit("Rebuild").unwrap();
    c.refresh_addon_session();
    let d = c.addon_dialog().unwrap_or_else(|| panic!("no Rebuild window: {}", c.prompt_text()));
    let rows: Vec<&str> = d.rows.iter().map(|r| r.name.as_str()).collect();
    for name in ["PointCount", "Degree", "DeleteInput", "CurrentLayer", "PreserveTangents", "SelectMasterCurve", "Preview"] {
        assert!(rows.contains(&name), "{name} missing from {rows:?}");
    }
    assert_eq!(d.notes.get("PointCount").map(String::as_str), Some("(2)"), "{:?}", d.notes);
    assert_eq!(d.notes.get("Degree").map(String::as_str), Some("(1)"));
    assert!(d.footer.iter().any(|f| f.starts_with("Maximum deviation")), "{:?}", d.footer);
    // Preview off: nothing drawn until OK; on again: drawn.
    c.set_addon_dialog_value("Preview", "No").unwrap();
    assert!(c.addon_session().unwrap().reply.preview.is_empty());
    c.set_addon_dialog_value("Preview", "Yes").unwrap();
    assert!(!c.addon_session().unwrap().reply.preview.is_empty());
    c.cancel();

    // Sweep 2 Rails: two rails and two lines, all selected, then the command.
    let ra = add(&mut c, "Curve", &[[0.0, 0.0, 0.0], [3.0, 0.0, 0.0], [6.0, 0.0, 0.0], [9.0, 0.0, 0.0], [12.0, 0.0, 0.0]]);
    let rb = add(&mut c, "Curve", &[[0.0, 5.0, 0.0], [1.0, 5.0, 0.0], [2.0, 5.0, 0.0], [7.0, 5.0, 1.0], [12.0, 5.0, 0.0]]);
    let s1 = add(&mut c, "Line", &[[0.0, 0.0, 0.0], [0.0, 5.0, 0.0]]);
    let s2 = add(&mut c, "Line", &[[12.0, 0.0, 0.0], [12.0, 5.0, 0.0]]);
    c.select_objects(&[ra, rb, s1, s2]).unwrap();
    c.submit("Sweep2").unwrap();
    c.refresh_addon_session();
    let d = c.addon_dialog().unwrap_or_else(|| panic!("no Sweep2 window: {}", c.prompt_text()));
    let titles: Vec<&str> = d.groups.iter().map(|g| g.0.as_str()).collect();
    assert_eq!(titles, ["Rail curve options", "Cross-section curve options"]);
    assert!(d.groups[1].1.contains(&"MaintainHeight".to_owned()));
    assert!(!d.enabled("PreserveFirstShape"), "greyed until the sections are simplified");
    c.set_addon_dialog_value("Simplify", "Rebuild").unwrap();
    assert!(c.addon_dialog().unwrap().enabled("PreserveFirstShape"));
    assert!(c.addon_dialog().unwrap().rows.iter().any(|r| r.name == "AddSlash"));
    let before = c.document().entities.len();
    c.submit("").unwrap();
    assert_eq!(c.document().entities.len(), before + 1, "{}", c.prompt_text());
}

#[test]
fn surface_commands_show_their_options_window() {
    let tmp = std::env::temp_dir().join(format!("om9-app-srf-dialogs-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();
    let feats = features();
    let names: Vec<FeatureName> = feats.iter().map(|(id, c)| FeatureName { id, command: c.as_deref() }).collect();
    let mut c = Controller::new(Document::new("srf-dialogs"), Limits::default()).unwrap();
    c.attach_addons(host(&tmp), &names);
    let curve = |c: &mut Controller, pts: &[[f64; 3]]| {
        let before: Vec<_> = c.document().entities.keys().copied().collect();
        c.clear_selection();
        c.submit("Curve").unwrap();
        for p in pts {
            c.session_point(*p).unwrap();
        }
        c.submit("").unwrap();
        *c.document().entities.keys().find(|k| !before.contains(k)).unwrap()
    };
    // A 2 × 2 network, slightly bent.
    let u0 = curve(&mut c, &[[0.0, 0.0, 0.0], [5.0, 0.0, 1.0], [10.0, 0.0, 0.0]]);
    let u1 = curve(&mut c, &[[0.0, 10.0, 0.0], [5.0, 10.0, 1.0], [10.0, 10.0, 0.0]]);
    let v0 = curve(&mut c, &[[0.0, 0.0, 0.0], [0.0, 5.0, 1.0], [0.0, 10.0, 0.0]]);
    let v1 = curve(&mut c, &[[10.0, 0.0, 0.0], [10.0, 5.0, 1.0], [10.0, 10.0, 0.0]]);
    let mut report = Vec::new();
    for (cmd, sel) in [("NetworkSrf", vec![u0, u1, v0, v1]), ("Patch", vec![u0, u1, v0, v1])] {
        c.clear_selection();
        c.select_objects(&sel).unwrap();
        c.submit(cmd).unwrap();
        c.refresh_addon_session();
        let d = c.addon_dialog();
        report.push(format!("{cmd}: {:?} {:?}", d.as_ref().map(|d| &d.title), d.as_ref().map(|d| d.rows.iter().map(|r| format!("{}={}", r.name, r.value)).collect::<Vec<_>>())));
        let d = d.unwrap_or_else(|| panic!("{cmd}: no options window ({})", c.prompt_text()));
        assert!(d.ready);
        // A setting changed in the window is used at once and remembered.
        if cmd == "NetworkSrf" {
            c.set_addon_dialog_value("EdgeTolerance", "0.002").unwrap();
            let d = c.addon_dialog().unwrap();
            let row = d.rows.iter().find(|r| r.name == "EdgeTolerance").unwrap();
            assert_eq!(row.value, "0.002");
        }
        let before = c.document().entities.len();
        c.submit("").unwrap();
        assert!(c.addon_session().is_none(), "{cmd}: {}", c.prompt_text());
        assert!(c.addon_dialog().is_none());
        assert!(c.document().entities.len() > before, "{cmd} made nothing");
    }
    // Curve Network the other way: the command first, then the curves one click at a time
    // (each click replaces the selection, as in the viewport), Enter, the window, OK.
    c.clear_selection();
    c.submit("NetworkSrf").unwrap();
    assert!(c.addon_dialog().is_none(), "no window before the curves");
    for id in [u0, u1, v0, v1] {
        let collecting = c.addon_collects_objects();
        c.select_root(Some(id), collecting).unwrap();
        c.refresh_addon_session();
    }
    let d = c.addon_dialog().unwrap_or_else(|| panic!("no window after picking: {}", c.prompt_text()));
    report.push(format!("NetworkSrf one by one: {}", d.title));
    let before = c.document().entities.len();
    c.submit("").unwrap();
    if c.addon_session().is_some() {
        // Enter ended the picking; the second Enter is OK.
        c.submit("").unwrap();
    }
    assert!(c.addon_session().is_none(), "{}", c.prompt_text());
    assert_eq!(c.document().entities.len(), before + 1, "network surface from picked curves");
    eprintln!("{}", report.join("\n"));
    fs::remove_dir_all(&tmp).unwrap();
}

/// Text with Create=Solids and FilletRadius through the real builder panel: the letters come
/// out rounded on top. OM9_SHOT_DIR=<dir> also writes a picture of the window.
#[test]
fn text_rounded_solids_in_the_window() {
    let font = if cfg!(windows) { "Arial" } else { "DejaVu Sans" };
    if !cfg!(windows) && !Path::new("/usr/share/fonts/truetype/dejavu").exists() {
        eprintln!("no test font installed; skipped");
        return;
    }
    let tmp = std::env::temp_dir().join(format!("om9-app-text-round-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();
    let feats = features();
    let names: Vec<FeatureName> = feats.iter().map(|(id, c)| FeatureName { id, command: c.as_deref() }).collect();
    let mut app = om9_app::ui::MatrixApp::new(Document::new("text-round")).unwrap();
    app.controller.attach_addons(host_lang(&tmp, "vi"), &names);
    let size = [1400.0f32, 900.0];
    // "Text" typed, as the menu names it, runs the add-on (Windows fonts, panel), not OM9's
    // older built-in Text.
    let r = om9_app::ui_replay::replay_app(&mut app, size, 1.0, &[serde_json::json!({"text": "Text"}), serde_json::json!({"key": "Enter"})]).unwrap();
    eprintln!("after typing: {}", r["steps"][2]["command"]);
    assert!(app.controller.addon_builder().is_some(), "the Text panel: {}", r["steps"][2]["prompt"]);
    use om9_addons::settings::SettingValue as V;
    for (k, v) in [
        ("text_string", V::Str("OM9".into())),
        ("text_font_other", V::Str(font.into())),
        ("text_height", V::Float(5.0)),
        ("text_depth", V::Float(1.5)),
        ("text_fillet", V::Float(0.4)),
    ] {
        app.controller.set_builder_value(k, v).unwrap();
    }
    let before = app.controller.document().entities.len();
    app.controller.session_point([0.0, 0.0, 0.0]).unwrap();
    let after = app.controller.document().entities.len();
    eprintln!("history:\n{}", app.transcript.history());
    assert_eq!(after, before + 3, "three rounded letters");
    if let Some(dir) = std::env::var_os("OM9_SHOT_DIR") {
        app.controller.zoom_extents(1.5).unwrap();
        let out = Path::new(&dir).join("text-rounded.png");
        let tris: usize = app.controller.document().entities.values().map(|e| format!("{:?}", e.geometry).matches("],").count()).sum();
        eprintln!("about {tris} triangles/points");
        om9_app::ui_replay::screenshot(&mut app, size, 1.0, 200, &out).unwrap();
        eprintln!("screenshot: {}", out.display());
    }
    fs::remove_dir_all(&tmp).unwrap();
}

/// The package as the user installs it: the built library loaded by the native runtime
/// (`OM9_NATIVE_ADDON=<path to libom9_matrix.so>`), then the window flow of Sweep 2.
#[test]
#[ignore = "needs OM9_NATIVE_ADDON pointing at a built om9_matrix library"]
fn sweep2_through_the_native_library() {
    let Some(lib) = std::env::var_os("OM9_NATIVE_ADDON") else { return };
    let tmp = std::env::temp_dir().join(format!("om9-app-native-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();
    let catalog = om9_app::addons::catalog().unwrap();
    let mut h = Host::new(HostConfig::new(om9_app::addons::om9_version(), catalog.clone(), tmp.join("config")));
    h.register_runtime(RuntimeKind::Native, Box::new(om9_addons::native::NativeRuntime::new()));
    h.startup(&Startup::default());
    let stage = tmp.join("stage-native");
    fs::create_dir_all(stage.join("icons")).unwrap();
    let addon = repo().join("addons/matrix/addon");
    let entry = Path::new(&lib).file_name().unwrap().to_str().unwrap().to_owned();
    let manifest = fs::read_to_string(addon.join("om9_addon.toml")).unwrap().replace("@PLATFORM@", "linux-x64").replace("@ENTRY@", &entry);
    fs::write(stage.join("om9_addon.toml"), manifest).unwrap();
    fs::copy(&lib, stage.join(&entry)).unwrap();
    for dir in ["icons", "lang"] {
        fs::create_dir_all(stage.join(dir)).unwrap();
        for e in fs::read_dir(addon.join(dir)).unwrap() {
            let e = e.unwrap().path();
            fs::copy(&e, stage.join(dir).join(e.file_name().unwrap())).unwrap();
        }
    }
    let (_, zip) = build_package(&stage, Some(&catalog)).unwrap_or_else(|e| panic!("{e:?}"));
    fs::write(tmp.join("m.zip"), zip).unwrap();
    let r = h.install_zip(&tmp.join("m.zip"), &InstallOptions { enable: true, ..Default::default() }).unwrap_or_else(|e| panic!("{e:?}"));
    assert!(r.enabled, "{:?}", r.warnings);
    h.set_setting("om9_matrix", "language", "en").unwrap();
    let feats = features();
    let names: Vec<FeatureName> = feats.iter().map(|(id, c)| FeatureName { id, command: c.as_deref() }).collect();
    let mut c = Controller::new(Document::new("native"), Limits::default()).unwrap();
    c.attach_addons(Arc::new(Mutex::new(h)), &names);
    let new_id = |c: &Controller, before: &[om9_core::identity::ObjectId]| *c.document().entities.keys().find(|k| !before.contains(k)).unwrap();
    let curve = |c: &mut Controller, pts: &[[f64; 3]]| {
        let before: Vec<_> = c.document().entities.keys().copied().collect();
        c.clear_selection();
        c.submit("Curve").unwrap();
        for p in pts {
            c.session_point(*p).unwrap();
        }
        c.submit("").unwrap();
        new_id(c, &before)
    };
    let ra = curve(&mut c, &[[-15.0, -20.0, 0.0], [-5.0, -5.0, 0.0], [-12.0, 8.0, 0.0], [-6.0, 20.0, 0.0]]);
    let rb = curve(&mut c, &[[8.0, -22.0, 0.0], [18.0, -5.0, 0.0], [10.0, 8.0, 0.0], [16.0, 21.0, 0.0]]);
    let p1 = curve(&mut c, &[[-15.0, -20.0, 0.0], [-3.0, -21.0, 6.0], [8.0, -22.0, 0.0]]);
    c.clear_selection();
    c.select_objects(&[ra, rb]).unwrap();
    c.submit("Sweep2").unwrap();
    eprintln!("native: {}", c.prompt_text());
    c.select_root(Some(p1), c.addon_collects_objects()).unwrap();
    c.refresh_addon_session();
    let before = c.document().entities.len();
    c.submit("").unwrap();
    eprintln!("native: {}", c.prompt_text());
    c.submit("").unwrap();
    for m in c.take_addon_messages() {
        eprintln!("native message: {m:?}");
    }
    assert!(c.addon_session().is_none(), "{}", c.prompt_text());
    assert_eq!(c.document().entities.len(), before + 1, "the sweep surface");
    fs::remove_dir_all(&tmp).unwrap();
}

// ---- Surface and Solid menus end to end -----------------------------------------------

struct Scene {
    c: Controller,
    named: std::collections::BTreeMap<&'static str, om9_core::identity::ObjectId>,
    restore: bool,
}

impl Scene {
    fn new_ids(&self, before: &[om9_core::identity::ObjectId]) -> Vec<om9_core::identity::ObjectId> {
        self.c.document().entities.keys().copied().filter(|k| !before.contains(k)).collect()
    }
    fn keys(&self) -> Vec<om9_core::identity::ObjectId> {
        self.c.document().entities.keys().copied().collect()
    }
    /// Runs a command with `select` preselected, feeding `points`, then Enter until done.
    fn run(&mut self, cmd: &str, select: &[&str], points: &[[f64; 3]]) -> (String, usize, usize, Vec<String>) {
        let before = self.keys();
        let rev = self.c.document().revision;
        self.c.cancel();
        self.c.clear_selection();
        // "name" is selected first; "@name:x,y,z" is a click on that object at that point
        // when the command asks for objects (an edge or a face is found by where it lands).
        let ids: Vec<om9_core::identity::ObjectId> = select.iter().filter(|n| !n.starts_with('@')).map(|n| self.named[n]).collect();
        let mut picks = select.iter().filter_map(|n| n.strip_prefix('@')).map(|n| {
            let (name, at) = n.split_once(':').unwrap();
            let v: Vec<f64> = at.split(',').map(|x| x.parse().unwrap()).collect();
            (self.named[name], [v[0], v[1], v[2]])
        }).collect::<Vec<_>>().into_iter();
        let mut trail = Vec::new();
        for n in select {
            let name = n.trim_start_matches('@').split(':').next().unwrap();
            if !self.c.document().entities.contains_key(&self.named[name]) {
                return (format!("scene object {name} is gone"), 0, 0, trail);
            }
        }
        if !ids.is_empty()
            && let Err(e) = self.c.select_objects(&ids)
        {
            return (format!("select error: {e}"), 0, 0, trail);
        }
        if let Err(e) = self.c.submit(cmd) {
            return (format!("start error: {e}"), 0, 0, trail);
        }
        let mut pts = points.iter();
        let mut saw_dialog = false;
        let mut saw_builder = false;
        for _ in 0..14 {
            let Some(s) = self.c.addon_session() else { break };
            let want = format!("{:?}", s.reply.want);
            let prompt = s.reply.prompt.clone();
            trail.push(format!("{want}:{prompt}"));
            saw_dialog |= self.c.addon_dialog().is_some();
            saw_builder |= self.c.addon_builder().is_some();
            let step = match want.as_str() {
                "Point" => match pts.next() {
                    Some(p) => self.c.session_point(*p),
                    // A click on an edge or a face asked as a point.
                    None => match picks.next() {
                        Some((_, p)) => self.c.session_point(p),
                        None => self.c.submit(""),
                    },
                },
                "Number" => self.c.submit("1"),
                "Objects" => match picks.next() {
                    Some((id, p)) => {
                        self.c.note_session_pick(Some(p));
                        let r = self.c.select_root(Some(id), true);
                        self.c.refresh_addon_session();
                        r
                    }
                    None => self.c.submit(""),
                },
                _ => self.c.submit(""),
            };
            if let Err(e) = step {
                trail.push(format!("error: {e}"));
                break;
            }
        }
        if self.c.addon_builder().is_some() {
            saw_builder = true;
            let r = self.c.accept_addon_builder();
            trail.push(format!("builder accept: {r:?}"));
        }
        let open = self.c.addon_session().map(|s| s.reply.prompt.clone());
        let made = self.new_ids(&before).len();
        let changed = (self.c.document().revision != rev) as usize;
        let mut status = if let Some(p) = open { format!("STILL OPEN: {p}") } else { "done".into() };
        if saw_dialog {
            status.push_str(" [window]");
        }
        if saw_builder {
            status.push_str(" [builder]");
        }
        self.c.cancel();
        // Back to the scene for the next command (a Boolean consumes its inputs …).
        if changed > 0 && self.restore {
            let _ = self.c.undo();
            if self.c.document().revision != rev && self.keys().len() != before.len() {
                let _ = self.c.undo();
            }
        }
        (status, made, changed, trail)
    }
    fn keep(&mut self, name: &'static str, cmd: &str, select: &[&str], points: &[[f64; 3]]) {
        let before = self.keys();
        let r = self.run(cmd, select, points);
        let new = self.new_ids(&before);
        // A command that changes its input in place (Make Hole) keeps the object's id.
        let id = match new.first() {
            Some(id) => *id,
            None if r.2 > 0 => self.named[select.last().unwrap()],
            None => panic!("scene {name} by {cmd}: {r:?}"),
        };
        self.named.insert(name, id);
    }
}

#[test]
fn surface_and_solid_menus_run_end_to_end() {
    let tmp = std::env::temp_dir().join(format!("om9-menu-survey-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();
    let feats = features();
    let names: Vec<FeatureName> = feats.iter().map(|(id, c)| FeatureName { id, command: c.as_deref() }).collect();
    let mut c = Controller::new(Document::new("survey"), Limits::default()).unwrap();
    c.attach_addons(host(&tmp), &names);
    let mut s = Scene { c, named: Default::default(), restore: false };
    // ---- the scene ---------------------------------------------------------------------
    let curve = |s: &mut Scene, name: &'static str, pts: &[[f64; 3]]| s.keep(name, "Curve", &[], pts);
    curve(&mut s, "ra", &[[-15.0, -20.0, 0.0], [-5.0, -5.0, 0.0], [-12.0, 8.0, 0.0], [-6.0, 20.0, 0.0]]);
    curve(&mut s, "rb", &[[8.0, -22.0, 0.0], [18.0, -5.0, 0.0], [10.0, 8.0, 0.0], [16.0, 21.0, 0.0]]);
    curve(&mut s, "p1", &[[-15.0, -20.0, 0.0], [-3.0, -21.0, 6.0], [8.0, -22.0, 0.0]]);
    curve(&mut s, "u0", &[[100.0, 0.0, 0.0], [105.0, 0.0, 1.0], [110.0, 0.0, 0.0]]);
    curve(&mut s, "u1", &[[100.0, 10.0, 0.0], [105.0, 10.0, 1.0], [110.0, 10.0, 0.0]]);
    curve(&mut s, "v0", &[[100.0, 0.0, 0.0], [100.0, 5.0, 1.0], [100.0, 10.0, 0.0]]);
    curve(&mut s, "v1", &[[110.0, 0.0, 0.0], [110.0, 5.0, 1.0], [110.0, 10.0, 0.0]]);
    curve(&mut s, "low", &[[200.0, 35.0, 0.0], [210.0, 35.0, 0.0]]);
    curve(&mut s, "high", &[[200.0, 35.0, 10.0], [210.0, 35.0, 10.0]]);
    s.keep("circ", "Circle", &[], &[[50.0, 50.0, 0.0], [55.0, 50.0, 0.0]]);
    s.keep("hole", "Circle", &[], &[[80.0, 50.0, 0.0], [82.0, 50.0, 0.0]]);
    s.keep("flat", "Plane", &[], &[[200.0, 30.0, 0.0], [210.0, 40.0, 0.0]]);
    s.keep("flat2", "Plane", &[], &[[210.0, 30.0, 0.0], [220.0, 40.0, 0.0]]);
    s.keep("ring", "Circle", &[], &[[50.0, 80.0, 0.0], [55.0, 80.0, 0.0]]);
    s.keep("ringhole", "Circle", &[], &[[50.0, 80.0, 0.0], [52.0, 80.0, 0.0]]);
    s.keep("wall", "Loft", &["low", "high"], &[]);
    s.keep("net", "NetworkSrf", &["u0", "u1", "v0", "v1"], &[]);
    s.keep("disc", "PlanarSrf", &["circ"], &[]);
    s.keep("washer", "PlanarSrf", &["ring", "ringhole"], &[]);
    s.keep("box1", "Box", &[], &[[300.0, 0.0, 0.0], [310.0, 10.0, 0.0], [310.0, 10.0, 10.0]]);
    s.keep("box2", "Box", &[], &[[305.0, 5.0, 0.0], [315.0, 15.0, 0.0], [315.0, 15.0, 10.0]]);
    s.keep("ball", "Sphere", &[], &[[400.0, 0.0, 0.0], [405.0, 0.0, 0.0]]);
    // A ring rail (circle in XZ, as a ring) and four profiles across it.
    s.keep("rr", "Circle", &[], &[[0.0, -150.0, 0.0], [8.0, -150.0, 0.0]]);
    curve(&mut s, "q0", &[[7.0, -150.0, -1.0], [8.0, -150.0, 0.0], [9.0, -150.0, 1.0]]);
    curve(&mut s, "q1", &[[0.0, -143.0, -1.0], [0.0, -142.0, 0.0], [0.0, -141.0, 1.0]]);
    curve(&mut s, "q2", &[[-7.0, -150.0, -1.0], [-8.0, -150.0, 0.0], [-9.0, -150.0, 1.0]]);
    curve(&mut s, "q3", &[[0.0, -157.0, -1.0], [0.0, -158.0, 0.0], [0.0, -159.0, 1.0]]);
    s.keep("c0", "Circle", &[], &[[0.0, -80.0, 0.0], [3.0, -80.0, 0.0]]);
    s.keep("c1", "Circle", &[], &[[0.0, -80.0, 10.0], [3.0, -80.0, 10.0]]);
    s.keep("tube", "Loft", &["c0", "c1"], &[]);
    s.keep("hc", "Circle", &[], &[[355.0, 5.0, 0.0], [356.0, 5.0, 0.0]]);
    s.keep("hb", "Box", &[], &[[350.0, 0.0, 0.0], [360.0, 10.0, 0.0], [360.0, 10.0, 10.0]]);
    s.keep("holed", "SolidSolidEditToolsHolesMakeHole", &["hc", "hb"], &[]);
    // ---- the menus ---------------------------------------------------------------------
    s.restore = true;
    if !cfg!(windows) {
        let host = s.c.addon_host().unwrap().clone();
        host.lock().unwrap().set_setting("om9_matrix", "text_font_other", "DejaVu Sans").unwrap();
    }
    let axis = [[-15.0, -20.0, 0.0], [-15.0, 20.0, 0.0]];
    let plan: Vec<(&str, &str, Vec<&str>, Vec<[f64; 3]>)> = vec![
        ("Surface", "SurfaceSweepSweep1Rail", vec!["ra", "p1"], vec![]),
        ("Surface", "SurfaceSweepSweep2Rails", vec!["ra", "rb", "p1"], vec![]),
        ("Surface", "SurfaceSweepProfileSweep", vec!["ra", "p1"], vec![]),
        ("Surface", "SurfacePlanarCurves", vec!["circ"], vec![]),
        ("Surface", "SurfacePlaneCornertoCorner", vec![], vec![[0.0, 60.0, 0.0], [10.0, 70.0, 0.0]]),
        ("Surface", "OthersSurfaceRebuild", vec!["net"], vec![]),
        ("Surface", "SurfaceBlendSurface", vec!["@flat:205,40,0", "@wall:205,35,10"], vec![]),
        ("Surface", "SurfaceVariableFilletBlendChamferVariableBlendSurfaces", vec!["@flat:205,40,0", "@wall:205,35,10"], vec![]),
        ("Surface", "SurfaceLoft", vec!["u0", "u1"], vec![]),
        ("Surface", "SurfaceCurveNetwork", vec!["u0", "u1", "v0", "v1"], vec![]),
        ("Surface", "SurfaceSurfaceEditToolsShrinkTrimmedSurface", vec!["washer"], vec![]),
        ("Surface", "SurfacePatch", vec!["u0", "u1", "v0", "v1"], vec![]),
        ("Surface", "SurfaceEdgeCurves", vec!["u0", "u1", "v0", "v1"], vec![]),
        ("Surface", "SurfaceSurfaceEditToolsUntrim", vec!["washer"], vec![]),
        ("Surface", "SurfaceGVDExtrudeAll", vec!["circ"], vec![]),
        ("Surface", "SurfaceRevolve", vec!["p1"], axis.to_vec()),
        ("Surface", "SurfaceRailRevolve", vec!["p1", "ra"], vec![[-30.0, -20.0, 0.0], [-30.0, 20.0, 0.0]]),
        ("Surface", "BuilderFourProfileSweep", vec!["rr", "q0", "q1", "q2", "q3"], vec![]),
        ("Surface", "SurfaceSweepSweepMultiRail", vec!["ra", "rb", "p1"], vec![]),
        ("Surface", "SurfaceTweenSurfaces", vec!["net", "wall"], vec![]),
        ("Surface", "SurfaceFilletSurfaces", vec!["@flat:205,33,0", "@wall:205,35,5"], vec![]),
        ("Surface", "SurfaceVariableFilletBlendChamferVariableFilletSurfaces", vec!["@flat:205,33,0", "@wall:205,35,5"], vec![]),
        ("Surface", "SurfaceChamferSurfaces", vec!["@flat:205,33,0", "@wall:205,35,5"], vec![]),
        ("Surface", "SurfaceVariableFilletBlendChamferVariableChamferSurfaces", vec!["@flat:205,33,0", "@wall:205,35,5"], vec![]),
        ("Surface", "SurfaceOffsetSurface", vec!["net"], vec![]),
        ("Surface", "SurfaceVariableOffsetSurface", vec!["net"], vec![]),
        ("Surface", "SurfaceSurfaceEditToolsMerge", vec!["flat", "flat2"], vec![]),
        ("Surface", "SurfaceSurfaceEditToolsMatch", vec!["@wall:205,35,10", "@flat:205,40,0"], vec![]),
        ("Surface", "SurfaceDrape", vec!["net"], vec![[95.0, -5.0, 0.0], [115.0, 15.0, 0.0]]),
        ("Surface", "SurfaceHeightfieldfromImage", vec![], vec![]),
        ("Surface", "SurfaceSurfaceEditToolsSplitatIsocurve", vec!["net"], vec![[105.0, 5.0, 0.8]]),
        ("Surface", "SurfaceExtendSurface", vec!["@flat:205,40,0"], vec![[205.0, 45.0, 0.0]]),
        ("Surface", "SurfaceExtrudeCurveNormalToSurface", vec!["@u0:105,0,1", "@net:105,5,0.8"], vec![[105.0, 0.0, 3.0]]),
        ("Surface", "SurfaceUnrollDevelopableSrf", vec!["wall"], vec![]),
        ("Surface", "SurfaceSmash", vec!["net"], vec![]),
        ("Surface", "SurfaceSurfaceEditToolsSoftEdit", vec!["net"], vec![[105.0, 5.0, 0.8], [105.0, 5.0, 3.0]]),
        ("Surface", "SurfacePlaneCuttingPlane", vec!["box1"], vec![[295.0, 5.0, 0.0], [320.0, 5.0, 0.0]]),
        ("Surface", "SurfaceSurfaceEditToolsAdjustClosedSurfaceSeam", vec!["@tube:0,-80,5"], vec![[3.0, -80.0, 5.0]]),
        ("Surface", "SurfaceSurfaceEditToolsSetSurfaceTangent", vec!["@net:105,5,0.8"], vec![[105.0, 0.0, 1.0], [105.0, 0.0, 1.0], [105.0, -3.0, 2.0]]),
        ("Surface", "SurfaceSurfaceEditToolsRefittoTolerance", vec!["net"], vec![]),
        ("Solid", "SolidUnion", vec!["box1", "box2"], vec![]),
        ("Solid", "SolidDifference", vec!["box1", "box2"], vec![]),
        ("Solid", "SolidIntersection", vec!["box1", "box2"], vec![]),
        ("Solid", "SolidBooleanTwoObjects", vec!["box1", "box2"], vec![]),
        ("Solid", "SolidCapPlanarHoles", vec!["wall"], vec![]),
        ("Solid", "SolidExtractSurface", vec!["box1"], vec![[305.0, 5.0, 10.0]]),
        ("Solid", "SolidFilletEdgeFilletEdge", vec!["box1"], vec![]),
        ("Solid", "SolidText", vec![], vec![[0.0, -60.0, 0.0]]),
        ("Solid", "SolidExtrudePlanarCurveStraight", vec!["circ"], vec![]),
        ("Solid", "SolidGVDAllExtrude", vec!["circ"], vec![]),
        ("Solid", "SolidPipe", vec!["ra"], vec![]),
        ("Solid", "SolidBoxCornertoCornerHeight", vec![], vec![[0.0, 100.0, 0.0], [5.0, 105.0, 0.0], [5.0, 105.0, 5.0]]),
        ("Solid", "SolidSphereCenterRadius", vec![], vec![[20.0, 100.0, 0.0], [23.0, 100.0, 0.0]]),
        ("Solid", "SolidEllipsoidFromCenter", vec![], vec![[40.0, 100.0, 0.0], [44.0, 100.0, 0.0], [40.0, 102.0, 0.0], [40.0, 100.0, 3.0]]),
        ("Solid", "SolidTorus", vec![], vec![[60.0, 100.0, 0.0], [65.0, 100.0, 0.0], [66.0, 100.0, 0.0]]),
        ("Solid", "SolidCylinder", vec![], vec![[80.0, 100.0, 0.0], [83.0, 100.0, 0.0], [80.0, 100.0, 6.0]]),
        ("Solid", "SolidTube", vec![], vec![[100.0, 100.0, 0.0], [104.0, 100.0, 0.0], [103.0, 100.0, 0.0], [100.0, 100.0, 6.0]]),
        ("Solid", "SolidPyramid", vec![], vec![[120.0, 100.0, 0.0], [124.0, 100.0, 0.0], [120.0, 100.0, 6.0]]),
        ("Solid", "SolidCone", vec![], vec![[140.0, 100.0, 0.0], [143.0, 100.0, 0.0], [140.0, 100.0, 6.0]]),
        ("Solid", "SolidTruncatedCone", vec![], vec![[160.0, 100.0, 0.0], [163.0, 100.0, 0.0], [160.0, 100.0, 6.0], [161.0, 100.0, 6.0]]),
        ("Solid", "SolidBoss", vec!["hole", "box1"], vec![]),
        ("Solid", "SolidRib", vec!["@high:205,35,10", "@flat:205,32,0"], vec![]),
        ("Solid", "SolidSlab", vec!["circ"], vec![[50.0, 56.0, 0.0], [50.0, 56.0, 2.0]]),
        ("Solid", "SolidSolidEditToolsHolesMakeHole", vec!["hole", "box1"], vec![]),
        ("Solid", "SolidSolidEditToolsHolesArrayHole", vec!["hole", "box1"], vec![[302.0, 2.0, 10.0], [304.0, 2.0, 10.0], [302.0, 4.0, 10.0]]),
        ("Solid", "SolidSolidEditToolsHolesArrayHolePolar", vec!["hole", "box1"], vec![[305.0, 5.0, 10.0]]),
        ("Solid", "SolidSolidEditToolsHolesMoveHole", vec!["@holed:355,5,10"], vec![[356.0, 5.0, 10.0], [355.0, 5.0, 10.0], [357.0, 6.0, 10.0]]),
        ("Solid", "SolidSolidEditToolsFacesMoveFace", vec!["box1"], vec![[305.0, 5.0, 10.0], [305.0, 5.0, 10.0], [305.0, 5.0, 12.0]]),
        ("Solid", "SolidShell", vec!["box1"], vec![]),
        ("Solid", "SolidPtOn", vec!["ball"], vec![]),
    ];
    let mut rows = vec!["| Menu | Lệnh | Kết quả | Mới | Đổi | Các bước |".to_string(), "|---|---|---|---|---|---|".to_string()];
    let mut bad = Vec::new();
    for (menu, cmd, sel, pts) in &plan {
        let (status, made, changed, trail) = s.run(cmd, sel, pts);
        let label = trail.first().cloned().unwrap_or_default();
        let ok = status.starts_with("done") && (made > 0 || changed > 0);
        if !ok {
            bad.push(format!("{cmd}: {status} | {}", trail.join(" → ")));
        }
        rows.push(format!(
            "| {menu} | {cmd} | {}{status} | {made} | {changed} | {} |",
            if ok { "✅ " } else { "⚠️ " },
            trail.join(" → ").replace('|', "/")
        ));
        let _ = label;
    }
    let table = rows.join("\n");
    if let Some(out) = std::env::var_os("OM9_SURVEY") {
        fs::write(PathBuf::from(out), &table).unwrap();
    }
    eprintln!("{} of {} commands need a look:\n{}", bad.len(), plan.len(), bad.join("\n"));
    // Left out on purpose: Shrink/Untrim need a trimmed surface (stored exactly only with the
    // B-Rep kernel), Heightfield an image file, and the typed 1 makes Shell and Tube too thin
    // here (checked with real sizes elsewhere).
    let expected = ["ShrinkTrimmedSurface", "Untrim", "HeightfieldfromImage", "SolidShell", "SolidTube"];
    let surprising: Vec<&String> = bad.iter().filter(|b| !expected.iter().any(|x| b.contains(x))).collect();
    assert!(surprising.is_empty(), "{surprising:#?}");
    fs::remove_dir_all(&tmp).unwrap();
}

/// OM9_SHOT_DIR=<dir> (and OM9_SHOT_LANG=vi): pictures of the Rebuild window with a polyline
/// selected, for a look against Rhino's (Point count (2) [10], Degree (1) [3] …).
#[test]
fn rebuild_window_picture() {
    let Some(dir) = std::env::var_os("OM9_SHOT_DIR") else { return };
    let tmp = std::env::temp_dir().join(format!("om9-app-rebuild-shot-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();
    let feats = features();
    let names: Vec<FeatureName> = feats.iter().map(|(id, c)| FeatureName { id, command: c.as_deref() }).collect();
    let mut app = om9_app::ui::MatrixApp::new(Document::new("rebuild-shot")).unwrap();
    let lang = std::env::var("OM9_SHOT_LANG").unwrap_or_else(|_| "en".into());
    app.controller.attach_addons(host_lang(&tmp, &lang), &names);
    let size = [1400.0f32, 900.0];
    app.controller.submit("Polyline").unwrap();
    for p in [[-10.0, -10.0, 0.0], [-2.0, 8.0, 0.0], [6.0, -4.0, 0.0], [12.0, 10.0, 0.0]] {
        app.controller.session_point(p).unwrap();
    }
    app.controller.submit("").unwrap();
    let id = *app.controller.document().entities.keys().last().unwrap();
    app.controller.select_objects(&[id]).unwrap();
    om9_app::ui_replay::replay_app(&mut app, size, 1.0, serde_json::json!([{"text": "Rebuild"}, {"key": "Enter"}, {"wait": 2}]).as_array().unwrap()).unwrap();
    let out = Path::new(&dir).join(format!("rebuild-window-{lang}.png"));
    om9_app::ui_replay::screenshot(&mut app, size, 1.0, 40, &out).unwrap();
    eprintln!("screenshot: {} ({})", out.display(), app.controller.prompt_text());
}
