// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    Error,
    budget::{Limits, Usage},
    model::Document,
    transaction::{TransactionCoordinator, World},
};
#[test]
fn externally_held_startup_is_rejected_before_new_authority_allocation() {
    let world = World::new(vec![Document::new("new held startup")], &Limits::default()).unwrap();
    let limits = Limits {
        max_bytes: world.validate(&Limits::default()).unwrap().bytes + 65536,
        ..Limits::default()
    };
    TransactionCoordinator::new(world.clone(), limits).expect("candidate alone fits");
    assert!(matches!(
        TransactionCoordinator::new_with_held(
            world,
            limits,
            Usage {
                bytes: 1_000_000,
                ..Usage::default()
            }
        ),
        Err(Error::Budget(_))
    ));
}
