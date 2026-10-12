#![forbid(unsafe_code)]
use std::collections::HashSet;
pub type CurveError = String;
#[derive(Clone, Debug, PartialEq)]
pub struct Basis {
    pub degree: usize,
    pub periodic: bool,
    pub poles: Vec<[f64; 3]>,
    pub weights: Vec<f64>,
    pub knots: Vec<f64>,
    pub multiplicities: Vec<usize>,
    pub first: f64,
    pub last: f64,
}
pub fn finite(value: f64) -> bool {
    value.is_finite() && value.abs() <= 1e9
}
impl Basis {
    pub fn validate(&self) -> Result<(), CurveError> {
        let n = self.poles.len();
        let k = self.knots.len();
        if !(1..=25).contains(&self.degree)
            || !(2..=4096).contains(&n)
            || n <= self.degree
            || !(2..=4096).contains(&k)
            || self.weights.len() != n
            || self.multiplicities.len() != k
        {
            return Err("Invalid curve degree or basis array sizes".into());
        }
        if self.poles.iter().flatten().any(|x| !finite(*x))
            || self.weights.iter().any(|x| !finite(*x) || *x <= 0.)
            || self.knots.iter().any(|x| !finite(*x))
            || !finite(self.first)
            || !finite(self.last)
        {
            return Err("Curve values must be finite within 1e9 and weights positive".into());
        }
        if self.knots.windows(2).any(|w| w[1] <= w[0])
            || self
                .multiplicities
                .iter()
                .any(|m| *m == 0 || *m > self.degree + 1)
        {
            return Err("Knots must strictly increase with valid integer multiplicities".into());
        }
        let sum: usize = self.multiplicities.iter().sum();
        let (lo, hi) = if self.periodic {
            if self.multiplicities[0] != self.multiplicities[k - 1]
                || self.multiplicities[0] > self.degree
                || sum - self.multiplicities[0] != n
            {
                return Err("Periodic basis pole/multiplicity relation is invalid".into());
            }
            (self.knots[0], self.knots[k - 1])
        } else {
            if sum != n + self.degree + 1
                || self.multiplicities[1..k - 1]
                    .iter()
                    .any(|m| *m > self.degree)
            {
                return Err("Basis pole/multiplicity relation is invalid".into());
            }
            let expanded = |index: usize| {
                let mut at = 0;
                for (u, m) in self.knots.iter().zip(&self.multiplicities) {
                    at += m;
                    if index < at {
                        return *u;
                    }
                }
                self.knots[k - 1]
            };
            (expanded(self.degree), expanded(n))
        };
        if self.last <= self.first || self.first < lo - 1e-12 || self.last > hi + 1e-12 {
            return Err("Curve trim domain is outside the edited basis".into());
        }
        Ok(())
    }
}
pub fn validate_wire(
    vertex_degrees: &[usize],
    ordered_edges: &[u64],
    expected_edges: usize,
) -> Result<(), CurveError> {
    if expected_edges == 0
        || ordered_edges.len() != expected_edges
        || vertex_degrees.is_empty()
        || vertex_degrees.iter().any(|n| *n == 0 || *n > 2)
        || ordered_edges.contains(&0)
        || ordered_edges.iter().copied().collect::<HashSet<_>>().len() != expected_edges
    {
        return Err(
            "Curve requires complete unique traversal of one nonbranching chain or cycle".into(),
        );
    }
    Ok(())
}
pub fn join_options(inputs: usize, edges: usize) -> Result<(), CurveError> {
    if !(2..=16).contains(&inputs) || !(1..=64).contains(&edges) {
        Err("Join requires 2..16 distinct open curves within 64 edges".into())
    } else {
        Ok(())
    }
}
