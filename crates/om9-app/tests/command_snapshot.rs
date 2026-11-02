use om9_app::session::{CmdPhase, Controller};
use om9_core::budget::Limits;
use om9_core::model::Document;

#[test]
fn active_input_snapshot_is_owner_run_and_phase_aware() {
    let mut c = Controller::new(Document::new("snapshot"), Limits::default()).unwrap();
    let idle = c.active_command_input_snapshot();
    assert_eq!(idle.owner, None);
    assert_eq!(idle.phase, CmdPhase::Idle);
    assert!(idle.options.is_empty());

    c.submit("Line").unwrap();
    let started = c.active_command_input_snapshot();
    assert_eq!(started.owner, Some("line"));
    assert_eq!(started.phase, CmdPhase::AwaitPoint);
    assert_eq!(started.options.len(), 2);
    assert!(started.run > idle.run);
    assert_eq!(started.options[0].id.name(), "BothSides");

    c.accept_world_point([0.0; 3]).unwrap();
    let end = c.active_command_input_snapshot();
    assert_eq!(end.owner, Some("line"));
    assert_eq!(end.phase, CmdPhase::AwaitEnd);
    assert_eq!(end.run, started.run);
    c.accept_world_point([10.0, 0.0, 0.0]).unwrap();
    let done = c.active_command_input_snapshot();
    assert_eq!(done.owner, None);
    assert_eq!(done.phase, CmdPhase::Idle);
    assert!(done.options.is_empty());
}

#[test]
fn positional_top11_owner_is_visible_without_a_fake_option_row() {
    use om9_core::geometry::{Curve, Geometry};
    use om9_core::model::Entity;
    let mut doc = Document::new("positional snapshot");
    let curve = Curve::Line {
        start: [0.0; 3],
        end: [10.0, 0.0, 0.0],
        domain: [0.0, 1.0],
    };
    let entity = Entity::new("line", Geometry::Curve(curve), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Move").unwrap();
    let snapshot = c.active_command_input_snapshot();
    assert_eq!(snapshot.owner, Some("top11"));
    assert_eq!(snapshot.phase, CmdPhase::AwaitPoint);
    assert!(snapshot.options.is_empty());
}

#[test]
fn polyline_snapshot_owner_and_stage_options_match_the_owner() {
    use om9_app::session::InvocationOrigin;
    let mut c = Controller::new(Document::new("polyline snapshot"), Limits::default()).unwrap();
    c.dispatch("OM9-CURVE-001", InvocationOrigin::Menu).unwrap();
    let start = c.active_command_input_snapshot();
    assert_eq!(start.owner, Some("polyline"));
    assert_eq!(start.phase, CmdPhase::AwaitPoint);
    assert!(
        start
            .options
            .iter()
            .any(|option| option.id.name() == "PersistentClose")
    );
    assert!(
        !start
            .options
            .iter()
            .any(|option| option.id.name() == "Length")
    );
    c.accept_world_point([0.0; 3]).unwrap();
    let next = c.active_command_input_snapshot();
    assert_eq!(next.owner, Some("polyline"));
    assert!(
        next.options
            .iter()
            .any(|option| option.id.name() == "Length")
    );
    assert_eq!(next.run, start.run);
}
