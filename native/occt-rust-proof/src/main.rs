//! Runs the gate proof and prints one JSON object, so the log is the evidence.
use om9_occt_rust_proof::{box_volume, drilled_volume, reject_invalid_cylinder, version};
use std::f64::consts::PI;

fn main() {
    let version = version();
    let box_volume_value = box_volume(10.0, 10.0, 4.0);
    let drilled = drilled_volume(10.0, 10.0, 4.0, 2.0);
    let invalid = reject_invalid_cylinder(-1.0);
    let finite = |value: Result<f64, _>| match value {
        Ok(number) => format!("{number:.17}"),
        Err(error) => format!("error: {error}"),
    };
    println!(
        "{{\"occt_version\": \"{}\", \"box_volume\": {}, \"drilled_volume\": {}, \"expected_drilled\": {:.17}, \"invalid_input\": \"{}\"}}",
        version,
        finite(box_volume_value),
        finite(drilled),
        400.0 - PI * 4.0 * 4.0,
        match &invalid {
            Ok(()) => "unexpectedly accepted".to_string(),
            Err(error) => error.to_string(),
        }
    );
}
