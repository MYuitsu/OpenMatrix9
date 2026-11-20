// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_app::{session::metal_catalog::{self, UserMetalCatalog}, ui::MatrixApp};
use om9_core::{metal_weight::{DensityUnit, MetalMaterial}, model::Document};

fn material(id: &str, density: f64) -> MetalMaterial {
    MetalMaterial::new(id, id, density, DensityUnit::GramsPerCubicCentimeter, "supplier fixture").unwrap()
}

#[test]
fn user_catalog_persists_and_rejects_invalid_replacement_atomically() {
    let dir = std::env::temp_dir().join(format!("om9-metal-catalog-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&dir).unwrap();
    let path = dir.join("catalog.json");
    let mut app = MatrixApp::new(Document::new("catalog A")).unwrap();
    app.load_user_metal_catalog(&path).unwrap();
    app.add_user_metal(material("alloy_a", 10.)).unwrap();
    app.add_user_metal(material("alloy_b", 12.)).unwrap();
    let before = std::fs::read(&path).unwrap();
    assert!(app.add_user_metal(material("ALLOY_A", 14.)).is_err());
    assert!(app.add_user_metal(material("gold_18k_yellow", 10.)).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), before);
    let mut reopened = MatrixApp::new(Document::new("catalog B")).unwrap();
    reopened.load_user_metal_catalog(&path).unwrap();
    assert_eq!(reopened.controller.metal_registry().get("alloy_a").unwrap().density_g_cm3, 10.);
    assert_eq!(reopened.controller.metal_registry().get("alloy_b").unwrap().density_g_cm3, 12.);
    assert!(!reopened.controller.metal_registry().is_reference("alloy_a"));
    assert!(reopened.controller.metal_registry().is_reference("gold_18k_yellow"));
    let startup_path = dir.join("metal-catalog.json");
    std::fs::copy(&path, &startup_path).unwrap();
    let startup = om9_app::startup::AppConfig { layout_path: Some(dir.join("ui-layout.json")), ..Default::default() };
    let started = MatrixApp::start(&startup).unwrap();
    assert_eq!(started.controller.metal_registry().get("alloy_b").unwrap().density_g_cm3, 12.);
    let invalid = dir.join("invalid.json");
    std::fs::write(&invalid, b"{broken").unwrap();
    assert!(reopened.load_user_metal_catalog(&invalid).is_err());
    assert_eq!(reopened.controller.metal_registry().get("alloy_a").unwrap().density_g_cm3, 10.);
    let test_only = UserMetalCatalog { materials: vec![MetalMaterial::test_only("fixture", 10., "fixture").unwrap()], ..Default::default() };
    assert!(metal_catalog::save(&path, &test_only).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), before);
    std::fs::remove_file(invalid).unwrap();
    std::fs::remove_file(startup_path).unwrap();
    std::fs::remove_file(path).unwrap();
    std::fs::remove_dir(dir).unwrap();
}

#[test]
fn bindings_restore_only_for_the_matching_document_and_material() {
    use om9_core::{exchange::RenderMaterial, identity::MaterialId};
    let dir = std::env::temp_dir().join(format!("om9-metal-bindings-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&dir).unwrap();
    let path = dir.join("catalog.json");
    let render = MaterialId::new();
    let mut document = Document::new("bound document");
    document.exchange.materials.insert(render, RenderMaterial { id: render, name: "render".into(), diffuse: [127; 4] });
    let mut app = MatrixApp::new(document.clone()).unwrap();
    app.load_user_metal_catalog(&path).unwrap();
    app.add_user_metal(material("alloy", 10.)).unwrap();
    app.set_user_metal_binding(render, Some("alloy")).unwrap();
    let saved = std::fs::read(&path).unwrap();
    assert!(app.set_user_metal_binding(render, Some("missing")).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), saved);
    assert_eq!(app.controller.bound_metal_material(render), Some("alloy"));
    assert!(app.delete_user_metal("alloy").is_err());
    assert_eq!(std::fs::read(&path).unwrap(), saved);
    app.update_user_metal(material("alloy", 11.)).unwrap();
    assert_eq!(app.controller.bound_metal_material(render), Some("alloy"));

    let mut reopened = MatrixApp::new(document).unwrap();
    reopened.load_user_metal_catalog(&path).unwrap();
    assert_eq!(reopened.controller.bound_metal_material(render), Some("alloy"));
    assert_eq!(reopened.controller.metal_registry().get("alloy").unwrap().density_g_cm3, 11.);
    let mut other = Document::new("another document");
    other.exchange.materials.insert(render, RenderMaterial { id: render, name: "render".into(), diffuse: [127; 4] });
    let mut other_app = MatrixApp::new(other).unwrap();
    other_app.load_user_metal_catalog(&path).unwrap();
    assert_eq!(other_app.controller.bound_metal_material(render), None);
    reopened.set_user_metal_binding(render, None).unwrap();
    reopened.delete_user_metal("alloy").unwrap();
    let mut reloaded = MatrixApp::new(reopened.controller.document().clone()).unwrap();
    reloaded.load_user_metal_catalog(&path).unwrap();
    assert_eq!(reloaded.controller.bound_metal_material(render), None);
    assert!(reloaded.controller.metal_registry().get("alloy").is_err());
    std::fs::remove_file(path).unwrap();
    std::fs::remove_dir(dir).unwrap();
}

#[test]
fn failed_binding_save_does_not_change_live_binding() {
    use om9_core::{exchange::RenderMaterial, identity::MaterialId};
    let dir = std::env::temp_dir().join(format!("om9-metal-binding-fail-{}", uuid::Uuid::new_v4()));
    let parent = dir.join("settings");
    std::fs::create_dir_all(&parent).unwrap();
    let path = parent.join("catalog.json");
    let render = MaterialId::new();
    let mut document = Document::new("binding failure");
    document.exchange.materials.insert(render, RenderMaterial { id: render, name: "render".into(), diffuse: [128; 4] });
    let mut app = MatrixApp::new(document).unwrap();
    app.load_user_metal_catalog(&path).unwrap();
    app.add_user_metal(material("alloy", 10.)).unwrap();
    std::fs::remove_file(&path).unwrap();
    std::fs::remove_dir(&parent).unwrap();
    std::fs::write(&parent, b"not a directory").unwrap();
    assert!(app.set_user_metal_binding(render, Some("alloy")).is_err());
    assert_eq!(app.controller.bound_metal_material(render), None);
    assert_eq!(app.controller.metal_registry().get("alloy").unwrap().density_g_cm3, 10.);
    std::fs::remove_file(parent).unwrap();
    std::fs::remove_dir(dir).unwrap();
}
