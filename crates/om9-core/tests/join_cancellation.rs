// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    Error,
    budget::Limits,
    geometry::{Curve, Geometry},
    model::Entity,
    top11::join_curve_entities_guarded,
};

#[test]
fn dense_join_checks_for_cancellation_during_the_endpoint_sweep() {
    let entities: Vec<_> = (0..300)
        .map(|_| {
            Entity::new(
                "line",
                Geometry::Curve(Curve::Line {
                    start: [0.0; 3],
                    end: [1.0, 0.0, 0.0],
                    domain: [0.0, 1.0],
                }),
                None,
            )
        })
        .collect();
    let refs: Vec<_> = entities.iter().collect();
    let original = entities.clone();
    let mut checks = 0;
    let cancellation = om9_core::transaction::Cancellation::default();
    let result = join_curve_entities_guarded(&refs, 0.001, &Limits::default(), &mut || {
        checks += 1;
        if checks == 1120 {
            cancellation.cancel();
        }
        if cancellation.is_canceled() {
            Err(Error::Canceled)
        } else {
            Ok(())
        }
    });
    assert_eq!(result, Err(Error::Canceled));
    assert_eq!(checks, 1120);
    assert_eq!(entities, original);
}
