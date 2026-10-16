//! A PLACE HOLDS A TURN AND A SHIFT, NOTHING ELSE: a component stands rigidly in its group. A file's transform may also
//! scale, shear or mirror - a glTF node, a 3MF component - and such a transform is split here into the turn nearest to
//! it and what is left over, which the reader bakes into the geometry under it. So a model its author scaled keeps its
//! tree, and only its numbers change.

/// A linear map of 3D, row by row, taking column vectors: `m[r][c]`.
pub(crate) type Lin = [[f64; 3]; 3];

pub(crate) const ONE: Lin = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

pub(crate) fn det(m: &Lin) -> f64 {
    m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1]) - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0]) + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
}

pub(crate) fn mul(a: &Lin, b: &Lin) -> Lin {
    std::array::from_fn(|r| std::array::from_fn(|c| (0..3).map(|k| a[r][k] * b[k][c]).sum()))
}

fn transpose(m: &Lin) -> Lin {
    std::array::from_fn(|r| std::array::from_fn(|c| m[c][r]))
}

/// Whether `m` is a turn: its columns of unit length and square to each other (to 1e-9), and no mirror.
fn is_turn(m: &Lin) -> bool {
    let p = mul(&transpose(m), m);
    (0..3).all(|r| (0..3).all(|c| (p[r][c] - if r == c { 1.0 } else { 0.0 }).abs() < 1e-9)) && det(m) > 0.0
}

/// `m` AS A TURN TIMES WHAT IS LEFT: `m = turn * rest`, the turn proper and the nearest one to `m` - the polar split,
/// found by Newton's iteration `u <- (u + u^-T) / 2`, which a 3x3 settles in a handful of steps. A turn is kept to the
/// bit, so a file of rigid transforms reads as it did. A mirror stays in `rest`: the turn is flipped about its first
/// axis to be proper, and whatever takes `rest` into its geometry turns its triangles over. `None` for a map that
/// flattens space (its determinant under 1e-12 of its size cubed): no geometry can be taken through it.
pub(crate) fn split(m: &Lin) -> Option<(Lin, Lin)> {
    if is_turn(m) {
        return Some((*m, ONE));
    }
    let size = m.iter().flatten().map(|v| v * v).sum::<f64>().sqrt();
    if det(m).abs() < 1e-12 * size.powi(3) {
        return None;
    }
    let mut u = *m;
    for _ in 0..100 {
        let du = det(&u);
        // u^-T: the cofactors over the determinant, each read off the two rows and columns that follow it cyclically
        let it: Lin = std::array::from_fn(|r| {
            std::array::from_fn(|c| {
                let (r1, r2, c1, c2) = ((r + 1) % 3, (r + 2) % 3, (c + 1) % 3, (c + 2) % 3);
                (u[r1][c1] * u[r2][c2] - u[r1][c2] * u[r2][c1]) / du
            })
        });
        let next: Lin = std::array::from_fn(|r| std::array::from_fn(|c| (u[r][c] + it[r][c]) / 2.0));
        let moved = (0..9).map(|k| (next[k / 3][k % 3] - u[k / 3][k % 3]).abs()).fold(0.0, f64::max);
        u = next;
        if moved < 1e-15 {
            break;
        }
    }
    if det(&u) < 0.0 {
        for row in &mut u {
            row[0] = -row[0];
        }
    }
    Some((u, mul(&transpose(&u), m)))
}
