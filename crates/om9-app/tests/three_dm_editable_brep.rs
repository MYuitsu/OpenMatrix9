// SPDX-License-Identifier: LGPL-2.1-or-later
//! Rhino 5 editable-BRep end-to-end regression.
//!
//! ```text
//! OM9_EXCHANGE_NATIVE=<om9-exchange-native.exe>
//! OM9_EXCHANGE_NATIVE_SHA256=<64 hex>
//! OM9_THREE_DM_FIXTURE=<Rhino 5 .3dm>
//! cargo test -p om9-app --features occt-brep --test three_dm_editable_brep -- --ignored
//! ```
#![cfg(all(windows, feature = "occt-brep"))]

use om9_app::three_dm_host::ThreeDmHost;
use om9_core::{
    budget::Limits,
    geometry::Geometry,
    model::{Document, Entity},
    transaction::{Cancellation, TransactionCoordinator, World},
};
use std::path::PathBuf;

fn fixture() -> PathBuf {
    PathBuf::from(std::env::var_os("OM9_THREE_DM_FIXTURE").expect("OM9_THREE_DM_FIXTURE"))
}

fn brep_count(entities: impl Iterator<Item = Entity>) -> usize {
    entities
        .filter(|entity| matches!(entity.geometry, Geometry::Brep(_)))
        .count()
}

#[test]
#[ignore = "requires admitted native reader + OCCT runtime + a Rhino 5 fixture"]
fn rhino5_imports_all_breps_as_editable_and_exports_again() {
    let fixture = fixture();
    let host = ThreeDmHost::from_environment().expect("admitted native reader");
    let document = Document::new("Rhino 5 editable BRep");
    let id = document.id;
    let limits = Limits::default();
    let mut coordinator =
        TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
    let receipt = host
        .import(&mut coordinator, id, &fixture, &Cancellation::default())
        .expect("editable import");
    assert_eq!(receipt.breps_editable, 30);
    assert_eq!(receipt.breps_opaque, 0);
    assert_eq!(receipt.layers, 34);
    assert_eq!(receipt.root_objects, 43);
    assert_eq!(receipt.definitions, 1);
    assert_eq!(receipt.definition_members, 1);
    let document = &coordinator.world().documents[&id];
    assert_eq!(brep_count(document.entities.values().cloned()), 29);
    assert_eq!(
        brep_count(
            document
                .definitions
                .values()
                .flat_map(|definition| definition.members.iter().cloned())
        ),
        1
    );
    let target = std::env::temp_dir().join(format!(
        "om9-rhino5-editable-{}.3dm",
        uuid::Uuid::new_v4()
    ));
    host.export(&coordinator, id, &target, &Cancellation::default())
        .expect("editable export");
    assert!(std::fs::metadata(&target).unwrap().len() > 0);
    std::fs::remove_file(target).unwrap();
}
