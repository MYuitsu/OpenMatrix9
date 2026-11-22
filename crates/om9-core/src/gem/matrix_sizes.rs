// SPDX-License-Identifier: LGPL-2.1-or-later
//! Factory measurements transcribed from Matrix9 library object metadata.
//! The order is significant, including presets that share a length.

use super::{GemCut, GemShape, GemSize};

enum Ladder {
    Equal(&'static [f64]),
    Pair(&'static [(f64, f64)]),
}

pub(super) fn sizes(cut: GemCut, shape: GemShape) -> Option<(Vec<GemSize>, f64)> {
    use GemCut::*;
    use GemShape::*;

    let (ladder, depth_ratio, weight_factor) = match (cut, shape) {
        (Cabochon, Bullet) => (Ladder::Equal(ROUND_DIAMETERS), 0.5, 0.0022),
        (Cabochon, Cushion) => (Ladder::Pair(OVAL_SIZES), 0.600000016060968, 0.0029),
        (Cabochon, Emerald) => (Ladder::Pair(RECTANGULAR_STEP_SIZES), 0.673056730567302, 0.0032),
        (Cabochon, Heart) => (Ladder::Equal(HEART_SIZES), 0.599999999999959, 0.0025),
        (Cabochon, Marquise) => (Ladder::Pair(MARQUISE_SIZES), 0.6, 0.0024),
        (Cabochon, Oval) => (Ladder::Pair(OVAL_SIZES), 0.642857142857143, 0.0027),
        (Cabochon, Pear) => (Ladder::Pair(PEAR_SIZES), 0.642857142857132, 0.00245),
        (Cabochon, Round) => (Ladder::Equal(ROUND_DIAMETERS), 0.615000009536743, 0.0027),
        (Cabochon, Square) => (Ladder::Equal(SQUARE_STEP_SIZES), 0.6, 0.0034),
        (Diamond, Baguette) => (Ladder::Pair(BAGUETTE_SIZES), 0.640000004768372, 0.003011),
        (Diamond, Calf) => (Ladder::Equal(TRIANGLE_SIZES), 0.490000009536743, 0.00189),
        (Diamond, CushionSquare) => (Ladder::Equal(LARGE_SQUARE_SIZES), 0.68770669464833, 0.002142),
        (Diamond, Cushion) => (Ladder::Pair(OVAL_SIZES), 0.600000010290698, 0.002955),
        (Diamond, EmeraldSquare) => (Ladder::Equal(SQUARE_STEP_SIZES), 0.641110682487488, 0.002297753496487057),
        (Diamond, Emerald) => (Ladder::Pair(RECTANGULAR_STEP_SIZES), 0.647018397099396, 0.0025438128664636286),
        (Diamond, HalfMoon) => (Ladder::Pair(HALF_MOON_SIZES), 0.797505067122021, 0.00177),
        (Diamond, Heart) => (Ladder::Equal(HEART_SIZES), 0.600000011920929, 0.001676),
        (Diamond, Marquise) => (Ladder::Pair(MARQUISE_SIZES), 0.563730965058009, 0.0016433774349006636),
        (Diamond, Octagon) => (Ladder::Equal(SQUARE_STEP_SIZES), 0.692345907140875, 0.002273),
        (Diamond, Oval) => (Ladder::Pair(OVAL_SIZES), 0.612299256471253, 0.0017570835774390167),
        (Diamond, Pear) => (Ladder::Pair(PEAR_SIZES), 0.642857137129335, 0.001634),
        (Diamond, Princess) => (Ladder::Equal(SQUARE_STEP_SIZES), 0.745673082706974, 0.0023862561257931686),
        (Diamond, RadiantSquare) => (Ladder::Equal(LARGE_SQUARE_SIZES), 0.716066526676434, 0.0020221110275488474),
        (Diamond, Radiant) => (Ladder::Pair(RECTANGULAR_STEP_SIZES), 0.661764703392193, 0.002955),
        (Diamond, Round) => (Ladder::Equal(ROUND_DIAMETERS), 0.622338702931999, 0.0017104682553832945),
        (Diamond, Triangle) => (Ladder::Equal(TRIANGLE_SIZES), 0.452925029102734, 0.00175),
        (Diamond, Trillion) => (Ladder::Equal(TRIANGLE_SIZES), 0.499999922503647, 0.001619),
        (Diamond, TrillionStraight) => (Ladder::Equal(TRIANGLE_SIZES), 0.5, 0.001903),
        (Gem, Cushion) => (Ladder::Pair(OVAL_SIZES), 0.599999995645777, 0.00235),
        (Gem, Emerald) => (Ladder::Pair(RECTANGULAR_STEP_SIZES), 0.670000487565995, 0.0027),
        (Gem, Heart) => (Ladder::Equal(HEART_SIZES), 0.600000011920929, 0.00195),
        (Gem, Marquise) => (Ladder::Pair(MARQUISE_SIZES), 0.600000005003972, 0.0019),
        (Gem, Oval) => (Ladder::Pair(OVAL_SIZES), 0.642863859701227, 0.0021),
        (Gem, Pear) => (Ladder::Pair(PEAR_SIZES), 0.642857149438789, 0.002),
        (Gem, Round) => (Ladder::Equal(ROUND_DIAMETERS), 0.61500000100568, 0.002),
        (Gem, Square) => (Ladder::Equal(SQUARE_STEP_SIZES), 0.6000000017063, 0.003),
        (Gem, Triangle) => (Ladder::Equal(TRIANGLE_SIZES), 0.450000009816931, 0.00175),
        (Gem, Trillion) => (Ladder::Equal(TRIANGLE_SIZES), 0.599999972050386, 0.0018),
        (Pearl, Round) => (Ladder::Equal(PEARL_DIAMETERS), 1.0, 0.0026179938800000003),
        (Simple, Calf) => (Ladder::Equal(TRIANGLE_SIZES), 0.490000009536743, 0.00189),
        (Simple, EmeraldSquare) => (Ladder::Equal(SQUARE_STEP_SIZES), 0.67419999797651, 0.002273),
        (Simple, Emerald) => (Ladder::Pair(RECTANGULAR_STEP_SIZES), 0.67299120426178, 0.002841),
        (Simple, Heart) => (Ladder::Equal(HEART_SIZES), 0.6, 0.001676),
        (Simple, Marquise) => (Ladder::Pair(MARQUISE_SIZES), 0.603126563360293, 0.001648),
        (Simple, Oval) => (Ladder::Pair(OVAL_SIZES), 0.642857126557529, 0.001903),
        (Simple, Pear) => (Ladder::Pair(PEAR_SIZES), 0.642857148121782, 0.001634),
        (Simple, Princess) => (Ladder::Equal(SQUARE_STEP_SIZES), 0.60000000144555, 0.003011),
        (Simple, Radiant) => (Ladder::Pair(RECTANGULAR_STEP_SIZES), 0.661764703392193, 0.002955),
        (Simple, Round) => (Ladder::Equal(ROUND_DIAMETERS), 0.614999987313661, 0.001733),
        (Simple, Trillion) => (Ladder::Equal(TRIANGLE_SIZES), 0.499999996358547, 0.001619),
        (Simple, TrillionStraight) => (Ladder::Equal(TRIANGLE_SIZES), 0.500000011835984, 0.001903),
        _ => return None,
    };
    let size = |length_mm, width_mm| GemSize { length_mm, width_mm, depth_mm: width_mm * depth_ratio };
    let sizes = match ladder {
        Ladder::Equal(values) => values.iter().map(|&d| size(d, d)).collect(),
        Ladder::Pair(values) => values.iter().map(|&(l, w)| size(l, w)).collect(),
    };
    Some((sizes, weight_factor))
}

const ROUND_DIAMETERS: &[f64] = &[
    1.0, 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 1.8, 1.9, 2.0, 2.1, 2.2, 2.4,
    2.5, 2.7, 2.8, 2.9, 3.0, 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 3.8, 3.9,
    4.0, 4.1, 4.2, 4.4, 4.6, 4.8, 5.0, 5.2, 5.4, 5.6, 5.8, 6.0, 6.4, 6.5,
    6.6, 6.8, 7.0, 7.2, 7.4, 7.6, 7.8, 8.0, 8.2, 8.4, 8.6, 8.8, 9.2, 9.4,
    9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2,
];

const OVAL_SIZES: &[(f64, f64)] = &[
    (4.0, 2.0), (4.5, 2.5), (5.0, 3.0), (5.3, 3.3), (5.8, 3.8), (6.0, 4.0),
    (6.1, 4.1), (6.2, 4.2), (6.5, 4.5), (6.6, 4.6), (6.9, 4.9), (7.0, 5.0),
    (7.2, 5.2), (7.5, 5.5), (8.0, 6.0), (8.25, 6.25), (8.5, 6.5), (9.0, 7.0),
    (10.0, 8.0), (10.0, 8.5), (11.0, 9.0), (11.0, 9.5), (12.0, 10.0),
];

const RECTANGULAR_STEP_SIZES: &[(f64, f64)] = &[
    (3.0, 1.5), (3.5, 1.5), (3.75, 1.75), (4.0, 2.0), (5.0, 3.0), (5.3, 3.3),
    (5.8, 3.8), (6.0, 4.0), (6.5, 4.5), (7.0, 5.0), (7.5, 5.5), (8.0, 6.0),
    (8.5, 6.5), (9.0, 7.0), (10.0, 8.0), (11.0, 9.0), (12.0, 10.0),
];

const HEART_SIZES: &[f64] = &[
    2.5, 3.0, 3.25, 3.5, 4.0, 4.25, 4.75, 5.0, 5.5, 6.0, 6.1, 6.4, 6.5, 6.75,
    7.0, 8.0, 8.5, 9.0, 9.5, 10.0, 10.5, 11.0,
];

const MARQUISE_SIZES: &[(f64, f64)] = &[
    (2.5, 1.25), (3.0, 1.5), (3.5, 1.75), (4.0, 2.0), (4.25, 2.25), (4.5, 2.5),
    (5.0, 2.5), (5.0, 3.0), (6.0, 3.0), (7.0, 3.5), (8.0, 4.0), (9.0, 4.5),
    (10.0, 5.0), (10.5, 5.25), (11.0, 5.5), (12.0, 6.0), (13.0, 6.5), (14.0, 7.0),
    (14.0, 7.5), (16.0, 8.0), (16.0, 8.5),
];

const PEAR_SIZES: &[(f64, f64)] = &[
    (2.5, 1.3), (3.0, 2.5), (4.0, 2.5), (4.0, 3.0), (4.5, 3.0), (5.0, 3.0),
    (5.5, 3.5), (5.8, 3.8), (6.0, 4.0), (7.0, 5.0), (8.0, 5.0), (8.5, 5.5),
    (9.0, 6.0), (10.0, 7.0), (12.0, 7.0), (12.0, 8.0), (13.0, 8.0), (14.0, 8.0),
    (14.5, 9.0), (15.0, 9.0),
];

const SQUARE_STEP_SIZES: &[f64] = &[
    1.5, 2.0, 2.25, 2.5, 2.75, 3.0, 3.25, 3.5, 3.75, 4.0, 4.3, 4.5, 5.0, 5.5,
    6.0, 6.5, 6.7, 7.0, 7.5, 8.0, 8.5, 9.0, 9.75, 10.5,
];

const BAGUETTE_SIZES: &[(f64, f64)] = &[
    (1.25, 1.0), (1.5, 1.25), (1.5, 1.0), (1.75, 1.0), (1.75, 1.25), (1.75, 1.5),
    (2.0, 1.0), (2.0, 1.25), (2.0, 1.5), (2.0, 1.75), (2.25, 1.0), (2.25, 1.25),
    (2.25, 1.5), (2.25, 1.75), (2.25, 2.0), (2.5, 1.0), (2.5, 1.25), (2.5, 1.5),
    (2.5, 1.75), (2.5, 2.0), (2.75, 1.25), (2.75, 1.5), (2.75, 1.75), (2.75, 2.0),
    (2.75, 2.5), (3.0, 1.0), (3.0, 1.25), (3.0, 1.5), (3.0, 1.75), (3.0, 2.0),
    (3.0, 2.5), (3.25, 1.25), (3.25, 1.5), (3.25, 1.75), (3.25, 2.0), (3.25, 2.5),
    (3.5, 1.5), (3.5, 1.75), (3.5, 2.0), (3.5, 2.5), (3.75, 1.25), (3.75, 1.5),
    (3.75, 1.75), (3.75, 2.0), (3.75, 2.5), (3.75, 3.0), (4.0, 1.5), (4.0, 1.75),
    (4.0, 2.0), (4.0, 2.5), (4.5, 2.0), (5.0, 2.0), (5.0, 2.5), (5.0, 3.0),
    (5.5, 2.5), (6.0, 3.0),
];

const TRIANGLE_SIZES: &[f64] = &[
    3.0, 3.5, 3.75, 4.0, 4.25, 4.5, 5.0, 5.5, 6.0, 6.1, 6.25, 6.5, 6.75, 7.0,
    7.5, 8.0, 9.0, 10.0,
];

const LARGE_SQUARE_SIZES: &[f64] = &[
    3.0, 3.5, 4.0, 4.5, 5.0, 5.5, 6.0, 6.5, 7.0, 7.5, 8.0, 8.5, 9.0, 9.5,
    10.0, 10.5, 11.0, 11.5, 12.0, 12.5, 13.0, 13.5, 14.0, 14.5, 15.0, 15.5, 16.0,
];

const HALF_MOON_SIZES: &[(f64, f64)] = &[
    (1.0, 0.577), (1.1, 0.634), (1.2, 0.692), (1.3, 0.749), (1.4, 0.807), (1.5, 0.865),
    (1.6, 0.922), (1.7, 0.98), (1.8, 1.038), (1.9, 1.095), (2.0, 1.153), (2.1, 1.211),
    (2.2, 1.268), (2.4, 1.384), (2.5, 1.441), (2.7, 1.557), (2.8, 1.614), (2.9, 1.672),
    (3.0, 1.73), (3.1, 1.787), (3.2, 1.845), (3.3, 1.902), (3.4, 1.96), (3.5, 2.018),
    (3.6, 2.075), (3.7, 2.133), (3.8, 2.191), (3.9, 2.248), (4.0, 2.306), (4.1, 2.364),
    (4.2, 2.421), (4.4, 2.537), (4.6, 2.652), (4.8, 2.767), (5.0, 2.883), (5.2, 2.998),
    (5.4, 3.113), (5.6, 3.228), (5.8, 3.344), (6.0, 3.459), (6.4, 3.69), (6.5, 3.747),
    (6.6, 3.805), (6.8, 3.92), (7.0, 4.036), (7.2, 4.151), (7.4, 4.266), (7.6, 4.381),
    (7.8, 4.497), (8.0, 4.612), (8.2, 4.727), (8.4, 4.843), (8.6, 4.958), (8.8, 5.073),
    (9.2, 5.304), (9.4, 5.419), (9.6, 5.534), (9.8, 5.65), (10.0, 5.765), (10.2, 5.88),
    (10.4, 5.996), (10.6, 6.111), (10.8, 6.226), (11.0, 6.342), (11.2, 6.457),
];

const PEARL_DIAMETERS: &[f64] = &[
    1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5, 5.0, 5.5, 6.0, 6.5, 7.0, 7.5,
    8.0, 8.5, 9.0, 9.5, 10.0, 10.5, 11.0, 11.5, 12.0,
];

pub(super) fn corner_to_corner_sizes(cut: GemCut, shape: GemShape) -> Option<(Vec<GemSize>, f64)> {
    use GemCut::*;
    use GemShape::*;

    let (widths, depth_ratio, weight_factor) = match (cut, shape) {
        (Diamond, Triangle) => (TRIANGLE_WIDTHS, 0.452925029102734, 0.00175),
        (Diamond, Trillion) => (CURVED_WIDTHS, 0.500000019694232, 0.001619),
        (Diamond, TrillionStraight) => (STRAIGHT_WIDTHS, 0.5, 0.001903),
        (Gem, Triangle) => (TRIANGLE_WIDTHS, 0.449999996394394, 0.00175),
        (Simple, Trillion) => (CURVED_WIDTHS, 0.499999994807797, 0.001619),
        (Simple, TrillionStraight) => (STRAIGHT_WIDTHS, 0.500000014455153, 0.001903),
        _ => return None,
    };
    let sizes = TRIANGLE_SIZES.iter().zip(widths).map(|(&length_mm, &width_mm)| GemSize {
        length_mm,
        width_mm,
        depth_mm: length_mm * depth_ratio,
    }).collect();
    Some((sizes, weight_factor))
}

const TRIANGLE_WIDTHS: &[f64] = &[
    2.602, 3.036, 3.253, 3.469, 3.686, 3.903, 4.337, 4.77, 5.204,
    5.291, 5.421, 5.638, 5.855, 6.071, 6.505, 6.939, 7.806, 8.674,
];

const CURVED_WIDTHS: &[f64] = &[
    2.89, 3.371, 3.612, 3.853, 4.094, 4.334, 4.816, 5.298, 5.779,
    5.876, 6.02, 6.261, 6.502, 6.742, 7.224, 7.706, 8.669, 9.632,
];

const STRAIGHT_WIDTHS: &[f64] = &[
    2.786, 3.25, 3.482, 3.714, 3.946, 4.178, 4.643, 5.107, 5.571,
    5.664, 5.803, 6.035, 6.267, 6.5, 6.964, 7.428, 8.357, 9.285,
];
