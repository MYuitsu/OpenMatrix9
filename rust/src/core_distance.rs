//! OM9-MEASURE-002: read-only distance between two world points.
pub fn command(icon: &str) -> Option<&'static str> {
    (icon == "AnalyzeDistance").then_some("Distance")
}
#[derive(Clone, Copy)]
pub enum Unit {
    Millimeter,
    Centimeter,
    Meter,
    Inch,
    Foot,
}

pub fn distance(first: [f64; 3], second: [f64; 3], unit: Unit) -> Option<f64> {
    if first
        .iter()
        .chain(second.iter())
        .any(|v| !v.is_finite() || v.abs() > 1e9)
    {
        return None;
    }
    let millimeters = (second[0] - first[0])
        .hypot(second[1] - first[1])
        .hypot(second[2] - first[2]);
    let divisor = match unit {
        Unit::Millimeter => 1.,
        Unit::Centimeter => 10.,
        Unit::Meter => 1000.,
        Unit::Inch => 25.4,
        Unit::Foot => 304.8,
    };
    Some(millimeters / divisor)
}

/// Unit codes: 0 mm, 1 cm, 2 m, 3 in, 4 ft. Invalid inputs return NaN.
#[unsafe(no_mangle)]
pub extern "C" fn om9_measure_distance(
    ax: f64,
    ay: f64,
    az: f64,
    bx: f64,
    by: f64,
    bz: f64,
    unit: u32,
) -> f64 {
    let unit = match unit {
        0 => Unit::Millimeter,
        1 => Unit::Centimeter,
        2 => Unit::Meter,
        3 => Unit::Inch,
        4 => Unit::Foot,
        _ => return f64::NAN,
    };
    distance([ax, ay, az], [bx, by, bz], unit).unwrap_or(f64::NAN)
}
