// SPDX-License-Identifier: LGPL-2.1-or-later
//! Measurement admission and cooperative cancellation at the Rust/native boundary.
use om9_core::{brep::BrepSourceAccess, budget::Limits, transaction::Cancellation, Error};
use om9_kernel::creation::{self, Primitive};

#[test]
fn canonical_measurement_admits_scratch_and_refuses_canceled_or_denied_work() {
    let limits = Limits::default();
    let body = creation::primitive(Primitive::Box { x_mm: 10., y_mm: 20., z_mm: 30. }, 0.1, &limits).unwrap();
    let source = body.to_canonical();
    let cancel = Cancellation::default();
    let mut calls = 0;
    let measured = creation::measure_canonical_cancellable_admitted(&source, &limits, &cancel, |live, _| {
        calls += 1;
        assert!(live > 0);
        Ok(())
    }).unwrap();
    assert!(calls > 0);
    assert!((measured.volume_mm3.abs() - 6000.).abs() < 1e-6);
    assert!(measured.closed);
    assert!(matches!(creation::measure_canonical_cancellable_admitted(&source, &limits, &cancel,
        |_, _| Err(Error::Budget("fixture admission"))), Err(Error::Budget("fixture admission"))));
    let during = Cancellation::default();
    assert!(matches!(creation::measure_canonical_cancellable_admitted(&source, &limits, &during, |_, _| {
        during.cancel(); Ok(())
    }), Err(Error::Canceled)));
    let before = Cancellation::default();
    before.cancel();
    assert!(matches!(creation::measure_canonical_cancellable_admitted(&source, &limits, &before,
        |_, _| panic!("pre-canceled work must not enter admission")), Err(Error::Canceled)));
}
