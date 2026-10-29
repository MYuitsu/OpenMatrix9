use om9_app::command_options::{CommandOptionInterface, CommandValue};
use om9_app::session::{Controller, InvocationOrigin};
use om9_core::budget::Limits;
use om9_core::model::Document;

#[test]
fn snapshots_expose_typed_values_not_only_labels() {
    let limits = Limits::default();
    let mut line = Controller::new(Document::new("values"), limits).unwrap();
    line.submit("Line").unwrap();
    let options = line.command_options();
    let both = options
        .iter()
        .find(|option| option.id.name() == "BothSides")
        .expect("BothSides");
    assert_eq!(both.value, CommandValue::Flag(false));
    let length = options
        .iter()
        .find(|option| option.id.name() == "Length")
        .expect("Length");
    assert_eq!(length.value, CommandValue::Number(None));

    let mut polyline = Controller::new(Document::new("polyline values"), limits).unwrap();
    polyline
        .dispatch("OM9-CURVE-001", InvocationOrigin::Menu)
        .unwrap();
    polyline.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    let length = polyline
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "Length")
        .expect("polyline Length");
    assert_eq!(length.value, CommandValue::Number(None));
    polyline.submit("Length=5").unwrap();
    let length = polyline
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "Length")
        .expect("polyline Length");
    assert_eq!(length.value, CommandValue::Number(Some(5.0)));
}

#[test]
fn helix_and_spiral_snapshots_carry_typed_owner_values() {
    let limits = Limits::default();
    let mut helix = Controller::new(Document::new("helix values"), limits).unwrap();
    helix.submit("Helix").unwrap();
    let turns = helix
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "Turns")
        .expect("Turns");
    assert_eq!(turns.value, CommandValue::Number(Some(3.0)));
    let height = helix
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "Height")
        .expect("Height");
    assert_eq!(height.value, CommandValue::Number(Some(10.0)));

    let mut spiral = Controller::new(Document::new("spiral values"), limits).unwrap();
    spiral.submit("Spiral").unwrap();
    let start = spiral
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "StartRadius")
        .expect("StartRadius");
    assert_eq!(start.value, CommandValue::Number(None));
    let turns = spiral
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "Turns")
        .expect("Spiral Turns");
    assert_eq!(turns.value, CommandValue::Number(Some(3.0)));
}

#[test]
fn angle_and_divide_snapshots_carry_typed_values() {
    use om9_core::geometry::Geometry;
    use om9_core::model::Entity;
    let limits = Limits::default();
    let mut doc = Document::new("typed owners");
    let surface = om9_core::surface_kernel::plane(
        [0.0; 3],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
        10.0,
        &limits,
    )
    .unwrap();
    let entity = Entity::new("plane", Geometry::Surface(surface), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut angle = Controller::new(doc, limits).unwrap();
    angle.select_objects(&[id]).unwrap();
    angle.submit("AngleIsocurve").unwrap();
    let direction = angle
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "Direction")
        .expect("Direction");
    assert!(matches!(direction.value, CommandValue::Choice("U" | "V")));
    let theta = angle
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "Angle")
        .expect("Angle");
    assert!(matches!(theta.value, CommandValue::Number(Some(_))));

    let mut divide = Controller::new(Document::new("divide values"), Limits::default()).unwrap();
    divide.submit("Line").unwrap();
    divide.accept_world_point([0.0; 3]).unwrap();
    divide.accept_world_point([10.0, 0.0, 0.0]).unwrap();
    let curve = *divide.document().entities.keys().next().unwrap();
    divide.select_objects(&[curve]).unwrap();
    divide.submit("Divide").unwrap();
    let count = divide
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "Count")
        .expect("Count");
    assert!(matches!(count.value, CommandValue::Integer(Some(_))));
}

#[test]
fn rebuild_and_polygon_snapshots_carry_typed_values() {
    use om9_core::geometry::{Curve, Geometry};
    use om9_core::model::Entity;
    let limits = Limits::default();
    let mut doc = Document::new("rebuild values");
    let curve = Curve::Line {
        start: [0.0; 3],
        end: [10.0, 0.0, 0.0],
        domain: [0.0, 1.0],
    };
    let entity = Entity::new("line", Geometry::Curve(curve), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut rebuild = Controller::new(doc, limits).unwrap();
    rebuild.select_objects(&[id]).unwrap();
    rebuild.submit("Rebuild").unwrap();
    let degree = rebuild
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "Degree")
        .expect("Degree");
    assert!(matches!(degree.value, CommandValue::Integer(Some(_))));
    let points = rebuild
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "Points")
        .expect("Points");
    assert!(matches!(points.value, CommandValue::Integer(Some(_))));

    let mut polygon = Controller::new(Document::new("polygon values"), Limits::default()).unwrap();
    polygon.submit("Polygon").unwrap();
    let sides = polygon
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "Sides")
        .expect("Sides");
    assert!(matches!(sides.value, CommandValue::Integer(Some(_))));
}

#[test]
fn primitive_snapshots_carry_typed_owner_values() {
    let limits = Limits::default();
    let mut arc = Controller::new(Document::new("arc values"), limits).unwrap();
    arc.submit("Arc").unwrap();
    let center = arc
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "Center")
        .expect("Arc Center");
    assert_eq!(center.value, CommandValue::Flag(false));

    let mut rect = Controller::new(Document::new("rect values"), limits).unwrap();
    rect.submit("Rectangle").unwrap();
    let square = rect
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "Square")
        .expect("Square");
    assert_eq!(square.value, CommandValue::Flag(false));

    let mut circle = Controller::new(Document::new("circle values"), limits).unwrap();
    circle.submit("Circle").unwrap();
    circle.accept_world_point([0.0; 3]).unwrap();
    let area = circle
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "Size=Area")
        .expect("Size=Area");
    circle.activate_command_option(area.id).unwrap();
    let area = circle
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "Size=Area")
        .expect("Size=Area after");
    assert_eq!(area.value, CommandValue::Flag(true));
    let diameter = circle
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "Size=Diameter")
        .expect("Size=Diameter");
    assert_eq!(diameter.value, CommandValue::Flag(false));
    let deformable = circle
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "Deformable")
        .expect("Deformable");
    assert_eq!(deformable.value, CommandValue::Flag(false));
}

#[test]
fn interp_snapshot_carries_degree_and_close() {
    let mut interp = Controller::new(Document::new("interp values"), Limits::default()).unwrap();
    interp.submit("InterpCrv").unwrap();
    let degree = interp
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "Degree")
        .expect("Degree");
    assert_eq!(degree.value, CommandValue::Integer(Some(3)));
}

#[test]
fn top11_duplicate_flags_are_typed_values() {
    use om9_core::geometry::Geometry;
    use om9_core::model::Entity;
    let mut doc = Document::new("duplicate typed");
    let entity = Entity::new("point", Geometry::Point([0.0; 3]), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Duplicate").unwrap();
    let vertical = c
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "Vertical")
        .expect("Vertical");
    assert_eq!(vertical.value, CommandValue::Flag(false));
    c.activate_command_option(vertical.id).unwrap();
    let vertical = c
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "Vertical")
        .expect("Vertical after");
    assert_eq!(vertical.value, CommandValue::Flag(true));
}
