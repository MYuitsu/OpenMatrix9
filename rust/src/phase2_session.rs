#![forbid(unsafe_code)]
use super::phase2_curve::{Basis, CurveError};
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Witness {
    pub document: u64,
    pub object: u64,
    pub generation: u64,
    pub signature: String,
}
impl Witness {
    pub fn validate(&self) -> Result<(), CurveError> {
        if self.document == 0
            || self.object == 0
            || self.signature.is_empty()
            || self.signature.len() > 512
        {
            Err("Invalid native identity witness".into())
        } else {
            Ok(())
        }
    }
}
pub struct CurveSession {
    witness: Witness,
    draft: Basis,
    canceled: bool,
}
impl CurveSession {
    pub fn new(witness: Witness, basis: Basis) -> Result<Self, CurveError> {
        witness.validate()?;
        basis.validate()?;
        Ok(Self {
            witness,
            draft: basis,
            canceled: false,
        })
    }
    pub fn replace_draft(&mut self, basis: Basis) -> Result<(), CurveError> {
        if self.canceled {
            return Err("Curve session canceled".into());
        }
        basis.validate()?;
        self.draft = basis;
        Ok(())
    }
    pub fn commit_request(&self, current: &Witness) -> Result<Basis, CurveError> {
        if self.canceled || current != &self.witness {
            return Err("Curve or placement changed; reopen the CV editor".into());
        }
        Ok(self.draft.clone())
    }
    pub fn cancel(&mut self) {
        self.canceled = true;
    }
    pub fn draft(&self) -> Result<&Basis, CurveError> {
        if self.canceled {
            Err("Curve session canceled".into())
        } else {
            Ok(&self.draft)
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RebuildOptions {
    pub degree: usize,
    pub point_count: usize,
    pub delete_input: bool,
}
impl RebuildOptions {
    pub fn validate(&self) -> Result<(), CurveError> {
        if !(1..=11).contains(&self.degree)
            || self.point_count <= self.degree
            || self.point_count > 256
        {
            Err("Rebuild requires Degree1..11 and PointCount>Degree within256".into())
        } else {
            Ok(())
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RebuildInput {
    pub witness: Witness,
    pub has_dependents: bool,
}
pub struct RebuildSession {
    inputs: Vec<RebuildInput>,
    options: RebuildOptions,
    canceled: bool,
}
pub fn validate_rebuild_count(count: usize) -> Result<(), CurveError> {
    if !(1..=16).contains(&count) {
        Err("Rebuild needs1..16 curves".into())
    } else {
        Ok(())
    }
}
impl RebuildSession {
    pub fn new(inputs: Vec<RebuildInput>, options: RebuildOptions) -> Result<Self, CurveError> {
        options.validate()?;
        validate_rebuild_count(inputs.len())?;
        let mut unique = std::collections::HashSet::new();
        for input in &inputs {
            input.witness.validate()?;
            if !unique.insert((input.witness.document, input.witness.object)) {
                return Err("Duplicate Rebuild curve".into());
            }
        }
        Ok(Self {
            inputs,
            options,
            canceled: false,
        })
    }
    pub fn replace_options(&mut self, options: RebuildOptions) -> Result<(), CurveError> {
        if self.canceled {
            return Err("Rebuild canceled".into());
        }
        options.validate()?;
        self.options = options;
        Ok(())
    }
    pub fn commit_request(&self, current: &[RebuildInput]) -> Result<RebuildOptions, CurveError> {
        if self.canceled
            || current.len() != self.inputs.len()
            || current
                .iter()
                .zip(&self.inputs)
                .any(|(a, b)| a.witness != b.witness)
        {
            return Err("Rebuild inputs changed; restart command".into());
        }
        if self.options.delete_input && current.iter().any(|x| x.has_dependents) {
            return Err("Cannot delete an input with dependent objects; use DeleteInput=No".into());
        }
        Ok(self.options.clone())
    }
    pub fn cancel(&mut self) {
        self.canceled = true;
    }
    pub fn options(&self) -> Result<&RebuildOptions, CurveError> {
        if self.canceled {
            Err("Rebuild canceled".into())
        } else {
            Ok(&self.options)
        }
    }
}
