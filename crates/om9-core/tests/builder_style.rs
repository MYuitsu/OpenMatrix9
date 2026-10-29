// SPDX-License-Identifier: LGPL-2.1-or-later
//! Builder styles (`OM9-L3-03-12-001` … `-006`): versioned snapshots, the browser listing,
//! preview-only apply, user saves, mode filtering and attribute matching.
use om9_core::{
    Error,
    builder::{BuilderKind, BuilderSession, register_reference_producers},
    builder_style::{
        BuilderStyle, MAX_STYLE_NAME_BYTES, STYLE_ATTRIBUTE_SLOTS, STYLE_SCHEMA, StyleFamily,
        StyleLibrary, StyleThumbnail, StyleValue, validate_style_name,
    },
    geometry::{Curve, Geometry},
    model::{Document, Entity, Extension, Layer},
    producers::ProducerRegistry,
};
use std::collections::BTreeMap;

fn registry() -> ProducerRegistry {
    let mut registry = ProducerRegistry::new();
    register_reference_producers(&mut registry).unwrap();
    registry
}

fn rail_curve() -> Geometry {
    Geometry::Curve(Curve::Circle {
        center: [0.0, 0.0, 0.0],
        x: [1.0, 0.0, 0.0],
        y: [0.0, 1.0, 0.0],
        radius: 9.0,
        domain: [0.0, std::f64::consts::TAU],
        angle_domain: None,
    })
}

fn document_with_rail() -> (Document, om9_core::identity::ObjectId) {
    let mut document = Document::new("builder-styles");
    let layer = Layer::new("Rings", [200, 180, 120, 255]);
    let layer_id = layer.id;
    document.layers.insert(layer_id, layer);
    document.layer_order.push(layer_id);
    let rail = Entity::new("Rail", rail_curve(), Some(layer_id));
    let rail_id = rail.id;
    document.entities.insert(rail_id, rail);
    (document, rail_id)
}

fn profile_session(document: &Document, rail: om9_core::identity::ObjectId) -> BuilderSession {
    BuilderSession::new(BuilderKind::RailProfile, document, &[rail]).unwrap()
}

fn style_file(styles: &[BuilderStyle]) -> String {
    serde_json::to_string(&serde_json::json!({
        "schema": STYLE_SCHEMA,
        "styles": styles,
    }))
    .unwrap()
}

fn simple_style(name: &str, width: f64) -> BuilderStyle {
    BuilderStyle {
        name: name.to_string(),
        family: vec![StyleFamily::Band],
        builders: Vec::new(),
        parameters: BTreeMap::from([
            ("width_mm".to_string(), StyleValue::LengthMm(width)),
            (
                "profile_ref".to_string(),
                StyleValue::Choice("circle".into()),
            ),
        ]),
        thumbnail: None,
    }
}

#[test]
fn a_style_roundtrip_keeps_the_values_their_unit_and_the_schema() {
    // `OM9-L3-03-12-001`: the typed snapshot serialises with an explicit version, and the
    // length keeps `length_mm` instead of turning into a bare number.
    let (document, rail) = document_with_rail();
    let session = profile_session(&document, rail);
    let style = BuilderStyle::defaults_of(&session, "typed v1");
    let keys: BTreeMap<_, _> = style
        .parameters
        .iter()
        .map(|(name, value)| (name.clone(), value.json_key()))
        .collect();
    assert_eq!(keys["width_mm"], "length_mm");
    assert_eq!(keys["preset"], "choice");
    // Defaulting is explicit: every field of the mode is recorded, not implied.
    assert_eq!(style.parameters.len(), session.fields().len());

    let text = style_file(std::slice::from_ref(&style));
    let parsed = StyleLibrary::parse_styles(&text).unwrap();
    assert_eq!(parsed, vec![style.clone()]);
    assert_eq!(
        parsed[0].parameters["width_mm"],
        StyleValue::LengthMm(2.0),
        "the length and its unit survive the file"
    );
    assert_eq!(
        parsed[0].parameters["preset"],
        StyleValue::Choice("rectangle".into())
    );
    let document_json: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(document_json["schema"], STYLE_SCHEMA);
    assert_eq!(document_json["styles"][0]["name"], "typed v1");
}

#[test]
fn an_unknown_schema_version_or_an_invalid_numeric_is_refused() {
    // `OM9-L3-03-12-001` negative fixture.
    let unknown = serde_json::json!({
        "schema": "om9.builder_style.v2",
        "styles": [],
    })
    .to_string();
    assert_eq!(
        StyleLibrary::parse_styles(&unknown).err().unwrap(),
        Error::Invalid("style schema")
    );

    let negative = serde_json::json!({
        "schema": STYLE_SCHEMA,
        "styles": [{
            "name": "bad width",
            "family": ["band"],
            "parameters": {"width_mm": {"length_mm": -1.0}},
        }],
    })
    .to_string();
    assert_eq!(
        StyleLibrary::parse_styles(&negative).err().unwrap(),
        Error::Invalid("style numeric")
    );

    let not_a_number = serde_json::json!({
        "schema": STYLE_SCHEMA,
        "styles": [{
            "name": "text width",
            "family": ["band"],
            "parameters": {"width_mm": {"length_mm": "wide"}},
        }],
    })
    .to_string();
    assert_eq!(
        StyleLibrary::parse_styles(&not_a_number).err().unwrap(),
        Error::Invalid("style schema")
    );
}

#[test]
fn the_browser_lists_every_style_for_the_mode_and_disables_the_incompatible() {
    // `OM9-L3-03-12-002`/`-005`: the ring mode sees the band and signet styles enabled and the
    // gem style listed but disabled.
    let library = StyleLibrary::builtin();
    let rows = library.styles_for(BuilderKind::RingRail);
    assert_eq!(rows.len(), 3);
    let enabled: Vec<_> = rows
        .iter()
        .filter(|row| row.enabled)
        .map(|row| row.style.name.as_str())
        .collect();
    assert_eq!(enabled, vec!["Classic 2.2", "Signet 3.0"]);
    let gem = rows
        .iter()
        .find(|row| row.style.name == "Brilliant Gem 1.9")
        .unwrap();
    assert!(!gem.enabled);
    assert!(!gem.style.compatible(BuilderKind::RingRail));
    assert!(
        rows.iter()
            .all(|row| matches!(row.thumbnail.icon, "RingRail" | "RailProfile"))
    );
}

#[test]
fn a_missing_or_corrupt_thumbnail_falls_back_without_dropping_the_style() {
    // `OM9-L3-03-12-002` negative fixture: the entry still lists, with a placeholder.
    let mut style = simple_style("Broken thumb", 2.4);
    style.thumbnail = Some("not-an-icon.png".to_string());
    let placeholder = StyleThumbnail::for_style(&style);
    assert!(placeholder.fallback);
    assert_eq!(placeholder.icon, "RingRail");

    style.thumbnail = None;
    assert!(StyleThumbnail::for_style(&style).fallback);

    style.thumbnail = Some("RailProfile".to_string());
    let real = StyleThumbnail::for_style(&style);
    assert!(!real.fallback);
    assert_eq!(real.icon, "RailProfile");

    let mut library = StyleLibrary::builtin();
    // A saved style without a thumbnail resource keeps listing with the placeholder.
    library.save_user(simple_style("Thumb 2.4", 2.4)).unwrap();
    let rows = library.styles_for(BuilderKind::RingRail);
    assert_eq!(rows.len(), 4);
    assert_eq!(
        rows.iter().filter(|row| row.thumbnail.fallback).count(),
        1,
        "only the entry without a valid thumbnail falls back"
    );
}

#[test]
fn applying_a_style_changes_the_preview_only_and_cancel_keeps_the_baseline() {
    // `OM9-L3-03-12-003`: apply loads the parameters into the session (the preview source);
    // the document is untouched, so Cancel has nothing to revert.
    let (document, rail) = document_with_rail();
    let mut session = profile_session(&document, rail);
    let baseline = session.field("width_mm").unwrap().numeric().unwrap();
    let entities = document.entities.len();

    let style_a = simple_style("Style A", 2.2);
    let style_b = simple_style("Style B", 3.0);
    let report = style_a.apply(&mut session).unwrap();
    assert_eq!(report.applied, vec!["preset", "width_mm"]);
    assert_eq!(session.field("width_mm").unwrap().numeric(), Some(2.2));

    style_b.apply(&mut session).unwrap();
    assert_eq!(session.field("width_mm").unwrap().numeric(), Some(3.0));

    // Preview only: no entity, no recipe, no history entry.
    session.refresh(&document, &registry()).unwrap();
    assert_eq!(document.entities.len(), entities);
    assert!(document.history.recipes.is_empty());
    assert_eq!(baseline, 2.0);

    // Cancel drops the whole session: the baseline is what a new session starts from.
    let fresh = profile_session(&document, rail);
    assert_eq!(fresh.field("width_mm").unwrap().numeric(), Some(baseline));
}

#[test]
fn a_style_missing_the_mandatory_profile_input_is_refused() {
    // `OM9-L3-03-12-003` negative fixture.
    let (document, rail) = document_with_rail();
    let mut session = profile_session(&document, rail);
    let mut style = simple_style("No profile", 2.5);
    style.parameters.remove("profile_ref");
    assert_eq!(
        style.apply(&mut session).err().unwrap(),
        Error::Missing("style profile")
    );
    // The refusal left the session at its defaults.
    assert_eq!(session.field("width_mm").unwrap().numeric(), Some(2.0));
}

#[test]
fn a_gem_style_cannot_be_applied_to_a_ring_builder() {
    // `OM9-L3-03-12-005` negative fixture: a style tagged for the gem builder is refused by
    // a ring-only mode even though it is listed.
    let (document, rail) = document_with_rail();
    let mut session = profile_session(&document, rail);
    let mut style = simple_style("Gem only", 2.0);
    style.family = vec![StyleFamily::Gem];
    assert!(!style.compatible(BuilderKind::RingRail));
    assert_eq!(
        style.apply(&mut session).err().unwrap(),
        Error::Invalid("style mode")
    );
}

#[test]
fn saving_a_user_style_leaves_the_defaults_untouched_and_refuses_a_duplicate_name() {
    // `OM9-L3-03-12-004`.
    let mut library = StyleLibrary::builtin();
    let defaults: Vec<_> = library.builtins().to_vec();
    library.save_user(simple_style("Classic 2.5", 2.5)).unwrap();
    assert_eq!(library.user().len(), 1);
    assert_eq!(library.builtins(), defaults.as_slice());
    assert_eq!(library.user()[0].name, "Classic 2.5");

    // A duplicate is refused, regardless of case; so is a built-in name.
    assert_eq!(
        library
            .save_user(simple_style("classic 2.5", 2.7))
            .err()
            .unwrap(),
        Error::Invalid("style exists")
    );
    assert_eq!(
        library
            .save_user(simple_style("Classic 2.2", 2.7))
            .err()
            .unwrap(),
        Error::Invalid("style exists")
    );
    // Built-ins are read-only.
    assert_eq!(
        library.remove_user("Classic 2.2").err().unwrap(),
        Error::Permission("style builtin")
    );
    assert!(library.remove_user("Classic 2.5").unwrap());
    assert!(library.user().is_empty());
}

#[test]
fn a_style_name_cannot_be_a_path_or_control_text() {
    // `OM9-L3-03-12-004` negative fixture: traversal, separators and control text.
    for name in ["../escape", "a/b", "a\\b", ".hidden", "a\0b", "a\nb"] {
        assert!(
            validate_style_name(name).is_err(),
            "{name:?} must be refused"
        );
    }
    assert!(validate_style_name("").is_err());
    assert!(validate_style_name(" padded").is_err());
    assert!(validate_style_name(&"x".repeat(MAX_STYLE_NAME_BYTES + 1)).is_err());
    assert!(validate_style_name("Classic 2.5").is_ok());
}

#[test]
fn the_user_library_roundtrips_through_a_file_and_reports_a_refused_write() {
    // `OM9-L3-03-12-004`: the user list lives in its own file; the built-ins stay in code.
    let directory = std::env::temp_dir().join(format!("om9-styles-{}", std::process::id()));
    let path = directory.join("styles.json");
    let _ = std::fs::remove_dir_all(&directory);

    let mut library = StyleLibrary::builtin();
    library.save_user(simple_style("Classic 2.5", 2.5)).unwrap();
    library.save_to(&path).unwrap();

    let mut loaded = StyleLibrary::builtin();
    assert_eq!(loaded.load_from(&path).unwrap(), 1);
    assert_eq!(loaded.user(), library.user());
    assert_eq!(loaded.builtins(), library.builtins());
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains(STYLE_SCHEMA));
    assert!(
        !text.contains("Classic 2.2"),
        "defaults are not copied into the user file"
    );

    // A missing file is an empty user list, not an error.
    let missing = directory.join("absent.json");
    let mut fresh = StyleLibrary::builtin();
    assert_eq!(fresh.load_from(&missing).unwrap(), 0);

    // A directory is not a style file: the write is refused explicitly.
    assert_eq!(
        library.save_to(&directory).err().unwrap(),
        Error::Permission("style library")
    );

    // A corrupt user file is refused instead of silently dropping the styles.
    std::fs::write(&path, "{ not json").unwrap();
    assert_eq!(
        loaded.load_from(&path).err().unwrap(),
        Error::Invalid("style schema")
    );
    let _ = std::fs::remove_dir_all(&directory);
}

#[test]
fn matching_attributes_copies_only_the_declared_attributes_and_keeps_the_identity() {
    // `OM9-L3-03-12-006` positive fixture: width=3, material=synthetic, height=2.
    let mut document = Document::new("style-source");
    let mut entity = Entity::new("Source ring", rail_curve(), None);
    let payload = serde_json::json!({
        "width_mm": 3.0,
        "height_mm": 2.0,
        "material": "synthetic",
        "unknown_member": 7,
    });
    entity.extensions.insert(
        STYLE_ATTRIBUTE_SLOTS[0].to_string(),
        Extension::new(1, payload.to_string()).unwrap(),
    );
    let id = entity.id;
    let extensions = entity.extensions.clone();
    document.entities.insert(id, entity);

    let mut style = simple_style("Matched", 2.0);
    let copied = style.match_attributes(&document.entities[&id]).unwrap();
    assert_eq!(copied, vec!["width_mm", "height_mm", "material"]);
    assert_eq!(
        style.parameters["width_mm"],
        StyleValue::LengthMm(3.0),
        "the copied width keeps the length type"
    );
    assert_eq!(style.parameters["height_mm"], StyleValue::LengthMm(2.0));
    assert_eq!(
        style.parameters["material"],
        StyleValue::Choice("synthetic".into())
    );
    // The source entity is not rewritten: same id, same slots.
    let after = &document.entities[&id];
    assert_eq!(after.id, id);
    assert_eq!(after.extensions, extensions);
}

#[test]
fn a_source_without_a_declared_slot_or_with_a_wrong_type_is_refused() {
    // `OM9-L3-03-12-006` negative fixture.
    let mut document = Document::new("style-source");
    let mut style = simple_style("Matched", 2.0);

    // No declared slot at all.
    let bare = Entity::new("Bare", rail_curve(), None);
    let bare_id = bare.id;
    document.entities.insert(bare_id, bare);
    assert_eq!(
        style
            .match_attributes(&document.entities[&bare_id])
            .err()
            .unwrap(),
        Error::Missing("style source")
    );

    // A slot whose schema does not match the declared types.
    let mut broken = Entity::new("Broken", rail_curve(), None);
    broken.extensions.insert(
        STYLE_ATTRIBUTE_SLOTS[0].to_string(),
        Extension::new(1, r#"{"width_mm": "wide"}"#.to_string()).unwrap(),
    );
    let broken_id = broken.id;
    document.entities.insert(broken_id, broken);
    assert_eq!(
        style
            .match_attributes(&document.entities[&broken_id])
            .err()
            .unwrap(),
        Error::Invalid("style attribute")
    );
}

#[test]
fn the_builder_tags_filter_a_style_to_one_mode() {
    // `OM9-L3-03-12-005`: a style may declare a single builder; the others disable it.
    let mut style = simple_style("Profile only", 2.6);
    style.builders = vec!["rail_profile".to_string()];
    assert!(style.targets(BuilderKind::RailProfile));
    assert!(!style.targets(BuilderKind::RingRail));
    let library = StyleLibrary::builtin();
    let rail_rows = library.styles_for(BuilderKind::RailProfile);
    assert!(rail_rows.iter().any(|row| row.style.name == "Signet 3.0"));
    // An unknown tag is refused at validation time instead of being guessed.
    let mut bogus = simple_style("Bogus", 2.0);
    bogus.builders = vec!["gem_builder".to_string()];
    assert_eq!(
        bogus.validate().err().unwrap(),
        Error::Invalid("style builder")
    );
}

#[test]
fn a_profile_reference_name_is_translated_to_the_builder_preset() {
    // `OM9-L3-03-12-001` fixture: `profile_ref=basic_round` is the design name of the circle
    // preset, so the style applies without the caller translating it.
    let (document, rail) = document_with_rail();
    let mut session = profile_session(&document, rail);
    let mut style = simple_style("Round 2.2", 2.2);
    style.parameters.insert(
        "profile_ref".to_string(),
        StyleValue::Choice("basic_round".into()),
    );
    style.apply(&mut session).unwrap();
    assert_eq!(
        session.field("preset").unwrap().selected_choice(),
        Some("circle")
    );
    // An unknown reference is refused, not coerced to the default shape.
    let mut unknown = simple_style("Bad ref", 2.2);
    unknown.parameters.insert(
        "profile_ref".to_string(),
        StyleValue::Choice("triangular".into()),
    );
    let mut fresh = profile_session(&document, rail);
    assert_eq!(
        unknown.apply(&mut fresh).err().unwrap(),
        Error::Invalid("style profile")
    );
}

#[test]
fn a_style_ignores_parameters_of_another_mode_and_reports_them() {
    // `OM9-L3-03-12-005`: parameters that do not exist in this mode are reported, and a
    // style whose parameters are all foreign is refused.
    let (document, rail) = document_with_rail();
    let mut session = profile_session(&document, rail);
    let mut style = simple_style("Sweep params", 2.0);
    style
        .parameters
        .insert("thickness_mm".to_string(), StyleValue::LengthMm(1.5));
    let report = style.apply(&mut session).unwrap();
    assert_eq!(report.applied, vec!["preset", "width_mm"]);
    assert_eq!(report.ignored, vec!["thickness_mm"]);
    assert!(report.summary().contains("ignored thickness_mm"));

    let mut foreign = BuilderStyle {
        name: "Foreign".to_string(),
        family: vec![StyleFamily::Band],
        builders: Vec::new(),
        parameters: BTreeMap::from([("thickness_mm".to_string(), StyleValue::LengthMm(1.5))]),
        thumbnail: None,
    };
    // The ring rail mode has no required profile, so the refusal here is the mode mismatch:
    // not one parameter of the style exists in this mode.
    let mut ring = BuilderSession::new(BuilderKind::RingRail, &document, &[]).unwrap();
    assert_eq!(
        foreign.apply(&mut ring).err().unwrap(),
        Error::Invalid("style mode")
    );
    // The ring session was not touched by the refused style.
    assert_eq!(ring.field("diameter_mm").unwrap().numeric(), Some(18.0));
    assert!(foreign.parameters.remove("thickness_mm").is_some());
}
