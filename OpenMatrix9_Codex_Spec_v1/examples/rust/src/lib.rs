extern crate self as om9_spec_examples;
// SPDX-License-Identifier: LGPL-2.1-or-later
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputKind { Point, Curve, Surface, Solid, Mesh, Gem, Object, Path, Image, Text, Document, Viewport, Selection }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParameterKind { Number, Boolean, Choice, Text, Reference }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect { ReadOnly, ViewState, DocumentWrite, ExternalOutput, InteractivePreview }
#[derive(Clone, Debug, PartialEq)]
pub struct Input { pub role: String, pub kind: InputKind, pub key: String, pub point: Option<[f64; 3]> }
#[derive(Clone, Debug, PartialEq)]
pub enum Value { Number(f64), Boolean(bool), Choice(String), Text(String), Reference(String) }
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Request { pub inputs: Vec<Input>, pub options: BTreeMap<String, Value>, pub document_revision: Option<u64> }
#[derive(Clone, Copy, Debug)]
pub struct Role { pub name: &'static str, pub kind: InputKind, pub min: usize, pub max: Option<usize> }
#[derive(Clone, Copy, Debug)]
pub struct Parameter { pub name: &'static str, pub kind: ParameterKind, pub required: bool }
#[derive(Clone, Copy, Debug)]
pub struct FeatureContract { pub id: &'static str, pub effect: Effect, pub roles: &'static [Role], pub parameters: &'static [Parameter] }
#[derive(Clone, Debug, PartialEq)]
pub struct OperationPlan { pub feature_id: &'static str, pub effect: Effect, pub request: Request }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlanError { InvalidContract, MissingRole(String), TooManyInputs(String), UnknownRole(String), WrongInputKind(String), EmptyIdentity, InvalidPoint, DuplicateInput, UnknownOption(String), MissingOption(String), WrongOptionKind(String), InvalidNumber(String), EmptyOption(String), MissingDocumentRevision, StaleDocument }

pub fn validate_request(contract: &FeatureContract, request: &Request) -> Result<OperationPlan, PlanError> {
    if contract.id.trim().is_empty() { return Err(PlanError::InvalidContract); }
    let mut role_names=BTreeSet::new();
    for role in contract.roles {
        if role.name.trim().is_empty() || !role_names.insert(role.name) || role.max.is_some_and(|m|m<role.min) {
            return Err(PlanError::InvalidContract);
        }
        let count=request.inputs.iter().filter(|input|input.role==role.name).count();
        if count<role.min { return Err(PlanError::MissingRole(role.name.into())); }
        if role.max.is_some_and(|m|count>m) { return Err(PlanError::TooManyInputs(role.name.into())); }
    }
    let mut identities=BTreeSet::new();
    for input in &request.inputs {
        let role=contract.roles.iter().find(|r|r.name==input.role)
            .ok_or_else(||PlanError::UnknownRole(input.role.clone()))?;
        if input.kind!=role.kind { return Err(PlanError::WrongInputKind(input.role.clone())); }
        if input.key.trim().is_empty() { return Err(PlanError::EmptyIdentity); }
        if !identities.insert((&input.role,&input.key)) { return Err(PlanError::DuplicateInput); }
        if input.kind==InputKind::Point && !input.point.is_some_and(|p|p.iter().all(|v|v.is_finite())) {
            return Err(PlanError::InvalidPoint);
        }
    }
    let mut names=BTreeSet::new();
    for parameter in contract.parameters {
        if parameter.name.trim().is_empty() || !names.insert(parameter.name) { return Err(PlanError::InvalidContract); }
        if parameter.required && !request.options.contains_key(parameter.name) {
            return Err(PlanError::MissingOption(parameter.name.into()));
        }
    }
    for (name,value) in &request.options {
        let parameter=contract.parameters.iter().find(|p|p.name==name)
            .ok_or_else(||PlanError::UnknownOption(name.clone()))?;
        let kind=match value { Value::Number(_)=>ParameterKind::Number,Value::Boolean(_)=>ParameterKind::Boolean,
            Value::Choice(_)=>ParameterKind::Choice,Value::Text(_)=>ParameterKind::Text,Value::Reference(_)=>ParameterKind::Reference };
        if parameter.kind!=kind { return Err(PlanError::WrongOptionKind(name.clone())); }
        match value {
            Value::Number(v) if !v.is_finite()=>return Err(PlanError::InvalidNumber(name.clone())),
            Value::Choice(v)|Value::Reference(v) if v.trim().is_empty()=>return Err(PlanError::EmptyOption(name.clone())),
            _=>{}
        }
    }
    if matches!(contract.effect,Effect::DocumentWrite|Effect::InteractivePreview) && request.document_revision.is_none() {
        return Err(PlanError::MissingDocumentRevision);
    }
    Ok(OperationPlan { feature_id: contract.id, effect: contract.effect, request: request.clone() })
}
impl OperationPlan {
    pub fn check_revision(&self, current: Option<u64>) -> Result<(), PlanError> {
        if self.request.document_revision.is_some() && self.request.document_revision!=current {
            return Err(PlanError::StaleDocument);
        }
        Ok(())
    }
}

#[allow(unused_imports)]
pub mod features;
