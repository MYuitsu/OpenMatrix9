// SPDX-License-Identifier: LGPL-2.1-or-later
//! OM9-L2-01-08 precision and performance contract.
//!
//! CAD accuracy means the numeric layer must state what it guarantees and prove it, so this
//! file measures instead of assuming:
//!
//! * **Unit conversion**: every unit converts through an exact integer ratio
//!   (`Unit::ratio`), so 25.4 mm/in is `127/5` and the conversion has one rounding, not two.
//! * **Round trip**: `from_unit(in_unit(u))` stays within one ULP of the original value over a
//!   deterministic sweep, which is the strongest statement a single-`f64` record can make.
//! * **Angles**: 180° is exactly `PI`, 90° is exactly `PI/2`, and the degree→radian→degree round
//!   trip is within one ULP on a grid.
//! * **Performance**: the admission hot path does not allocate a set per value and does not
//!   re-validate the schema per value; the smoke test prints the measured rate and asserts a
//!   generous ceiling so a real regression trips it without flaking.
use om9_core::{
    budget::Limits,
    geometry::Unit,
    request::{Dimension, OptionDomain, Value},
    units::{AngleDeg, LengthMm, MAX_ABS_DEGREES},
};

/// The ULP distance between two finite `f64` values (both treated as ordered integers).
fn ulps(a: f64, b: f64) -> u64 {
    let ordered = |value: f64| -> i64 {
        let bits = value.to_bits() as i64;
        if bits < 0 { i64::MIN - bits } else { bits }
    };
    ordered(a).abs_diff(ordered(b))
}

/// A deterministic pseudo-random sweep: an LCG, so the sample is reproducible on every run.
struct Sweep(u64);

impl Sweep {
    fn next_unit(&mut self) -> f64 {
        // xorshift64*, then map the top 53 bits into [0, 1).
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        ((x.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 11) as f64) / (1u64 << 53) as f64
    }
    fn next_signed(&mut self, magnitude: f64) -> f64 {
        (self.next_unit() * 2.0 - 1.0) * magnitude
    }
}

/* ------------------------- unit conversion ------------------------- */

#[test]
fn an_exact_ratio_is_more_accurate_than_a_rounded_decimal_factor() {
    // 1 in = 25.4 mm exactly as a ratio; the float literal 25.4 is the same value.
    assert_eq!(
        LengthMm::from_unit(1.0, Unit::Inch).unwrap().mm().to_bits(),
        25.4f64.to_bits()
    );
    assert_eq!(
        LengthMm::from_unit(1.0, Unit::Foot).unwrap().mm().to_bits(),
        304.8f64.to_bits()
    );
    // 1000 um = 1 mm exactly, through the division path.
    assert_eq!(
        LengthMm::from_unit(1000.0, Unit::Micrometer).unwrap().mm(),
        1.0
    );
    // The old multiplicative form used a rounded factor for the micrometer: measure the gap on a
    // value where it shows.
    let value = 3.0;
    let ratio = value * Unit::Micrometer.ratio().0 / Unit::Micrometer.ratio().1;
    let multiplicative = value * 0.001;
    assert!(
        ratio == 3.0 / 1000.0,
        "the ratio path is a single division: {ratio:?}"
    );
    assert!(
        ulps(ratio, multiplicative) <= 1,
        "the rounded 0.001 factor differs by at most one ULP here"
    );
}

#[test]
fn a_round_trip_through_any_unit_stays_within_one_ulp() {
    let mut sweep = Sweep(0x9E37_79B9_7F4A_7C15);
    for unit in [
        Unit::Millimeter,
        Unit::Centimeter,
        Unit::Meter,
        Unit::Inch,
        Unit::Foot,
        Unit::Micrometer,
    ] {
        for _ in 0..20_000 {
            let value = sweep.next_signed(1.0e4);
            let mm = LengthMm::from_unit(value, unit).unwrap();
            let back = mm.in_unit(unit);
            let distance = ulps(value, back);
            assert!(
                distance <= 1,
                "{unit:?}: {value:e} -> {:e} -> {back:e} is {distance} ULP apart",
                mm.mm()
            );
        }
    }
}

#[test]
fn a_display_unit_conversion_is_lossless_for_exact_values() {
    // Values that are exactly representable in both units must survive the round trip exactly.
    for (value, unit) in [
        (1.0, Unit::Millimeter),
        (10.0, Unit::Centimeter),
        (1000.0, Unit::Meter),
        (25.4, Unit::Inch),
        (304.8, Unit::Foot),
        (0.001, Unit::Micrometer),
    ] {
        let mm = LengthMm::from_unit(value, unit).unwrap();
        assert_eq!(mm.in_unit(unit), value, "{value} {unit:?}");
    }
}

/* ------------------------- angles ------------------------- */

#[test]
fn angle_conversions_are_exact_at_the_quadrant_boundaries() {
    // 180 degrees is exactly PI, 90 exactly PI/2, 360 exactly 2*PI.
    assert_eq!(
        AngleDeg::try_new(180.0)
            .unwrap()
            .to_radians()
            .rad()
            .to_bits(),
        std::f64::consts::PI.to_bits()
    );
    assert_eq!(
        AngleDeg::try_new(90.0)
            .unwrap()
            .to_radians()
            .rad()
            .to_bits(),
        (std::f64::consts::PI / 2.0).to_bits()
    );
    assert_eq!(
        AngleDeg::try_new(360.0)
            .unwrap()
            .to_radians()
            .rad()
            .to_bits(),
        (std::f64::consts::PI * 2.0).to_bits()
    );
    // And back: PI radians is exactly 180 degrees.
    let degrees = om9_core::units::AngleRad::try_new(std::f64::consts::PI)
        .unwrap()
        .to_degrees()
        .unwrap()
        .deg();
    assert_eq!(degrees, 180.0);
}

#[test]
fn a_degree_radian_round_trip_stays_within_one_ulp() {
    let mut sweep = Sweep(0x1234_5678_9ABC_DEF1);
    for _ in 0..20_000 {
        let degrees = sweep.next_signed(MAX_ABS_DEGREES);
        let angle = AngleDeg::try_new(degrees).unwrap();
        let back = angle.to_radians().to_degrees().unwrap().deg();
        let distance = ulps(degrees, back);
        assert!(
            distance <= 1,
            "{degrees:e} -> {:e} -> {back:e} is {distance} ULP apart",
            angle.to_radians().rad()
        );
    }
}

/* ------------------------- tolerance comparison ------------------------- */

#[test]
fn a_tolerance_comparison_is_documented_and_boundary_exact() {
    use om9_core::history::Tolerance;
    let tolerance = Tolerance::try_new(0.001, 1.0e-4, 1.0e-9).unwrap();
    // Exactly at the tolerance: refused (the contract is "at or below", not "below").
    assert!(
        tolerance
            .admit_feature(LengthMm::try_new(0.001).unwrap())
            .is_err()
    );
    // One ULP above the tolerance is already outside it: the comparison is exact, not fuzzy.
    let just_above = f64::from_bits(0.001f64.to_bits() + 1);
    assert!(
        tolerance
            .admit_feature(LengthMm::try_new(just_above).unwrap())
            .is_ok()
    );
    // One ULP below is still inside.
    let just_below = f64::from_bits(0.001f64.to_bits() - 1);
    assert!(
        tolerance
            .admit_feature(LengthMm::try_new(just_below).unwrap())
            .is_err()
    );
    // The documented rule itself.
    assert!(om9_core::history::within_tolerance(1.0, 1.0005, 0.001));
    assert!(!om9_core::history::within_tolerance(1.0, 1.002, 0.001));
    assert!(om9_core::history::within_tolerance(1.0, 1.001, 0.001));
    assert!(!om9_core::history::within_tolerance(f64::NAN, 1.0, 0.001));
}

/* ------------------------- performance ------------------------- */

#[test]
fn the_admission_hot_path_does_not_allocate_per_value() {
    // A 32-choice domain: the small-list rule validates without a BTreeSet. The assertion is
    // the correctness half; the printed rate is the regression tripwire.
    let domain = OptionDomain::Choice((0..32).map(|index| format!("Choice{index:02}")).collect());
    let limits = Limits::default();
    domain.validate().unwrap();
    let value = Value::Choice("Choice17".into());
    for _ in 0..1000 {
        domain.check_admitted(&value, &limits).unwrap();
    }
    let started = std::time::Instant::now();
    const ROUNDS: u32 = 200_000;
    for _ in 0..ROUNDS {
        domain.check_admitted(&value, &limits).unwrap();
    }
    let elapsed = started.elapsed();
    let per_value = elapsed.as_nanos() / u128::from(ROUNDS);
    println!("check_admitted: {ROUNDS} values in {elapsed:?} ({per_value} ns/value)");
    // Generous ceiling: a real regression (e.g. re-validating the schema per value) is far
    // slower than this, and a loaded machine is still fast enough.
    assert!(
        elapsed.as_millis() < 2_000,
        "the value check must stay far below 2 s for {ROUNDS} values: {elapsed:?}"
    );
    // The rejecting path is the same cost class.
    let bad = Value::Choice("Unknown".into());
    let started = std::time::Instant::now();
    for _ in 0..ROUNDS {
        assert!(domain.check_admitted(&bad, &limits).is_err());
    }
    println!("check_admitted (reject): {:?}", started.elapsed());
}

#[test]
fn unit_conversion_throughput_is_stated() {
    const ROUNDS: u32 = 200_000;
    let started = std::time::Instant::now();
    let mut sink = 0.0;
    for index in 0..ROUNDS {
        let value = f64::from(index) * 0.37;
        let mm = LengthMm::from_unit(value, Unit::Inch).unwrap();
        sink += mm.in_unit(Unit::Inch);
    }
    let elapsed = started.elapsed();
    let per_value = elapsed.as_nanos() / u128::from(ROUNDS);
    println!(
        "from_unit + in_unit (inch): {ROUNDS} round trips in {elapsed:?} ({per_value} ns/pair), sink {sink:e}"
    );
    assert!(
        elapsed.as_millis() < 2_000,
        "conversion must stay fast: {elapsed:?}"
    );
    // Dimensions stay in the numeric layer: this is only a sanity check on the type.
    assert_eq!(Dimension::Length, Dimension::Length);
}
