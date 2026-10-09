// SPDX-License-Identifier: LGPL-2.1-or-later
pub mod om9_iface_001 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-IFACE-001",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "LayoutAction", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_main_001 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-MAIN-001",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Action", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_f6_001 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-F6-001",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Global", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Pin", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Name", kind: ParameterKind::Text, required: false },
        Parameter { name: "Command", kind: ParameterKind::Text, required: false },
        Parameter { name: "Position", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_viewport_001 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-VIEWPORT-001",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "View", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Maximize", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Gesture", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_buildercore_001 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-BUILDERCORE-001",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "active_document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "ControlValue", kind: ParameterKind::Number, required: false },
        Parameter { name: "Action", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_style_001 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-STYLE-001",
    effect: Effect::ExternalOutput,
    roles: &[
        Role { name: "active_document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "StyleName", kind: ParameterKind::Text, required: false },
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Action", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_info_001 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-INFO-001",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "selected_objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Property", kind: ParameterKind::Text, required: false },
        Parameter { name: "Value", kind: ParameterKind::Text, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_info_002 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-INFO-002",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Scope", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Setting", kind: ParameterKind::Text, required: false },
        Parameter { name: "Value", kind: ParameterKind::Text, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_info_003 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-INFO-003",
    effect: Effect::ReadOnly,
    roles: &[
        Role { name: "selected_objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_info_004 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-INFO-004",
    effect: Effect::ReadOnly,
    roles: &[
        Role { name: "active_document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_info_005 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-INFO-005",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "selected_objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_info_006 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-INFO-006",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "active_document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Text", kind: ParameterKind::Text, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_info_007 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-INFO-007",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "ObjectClass", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_info_008 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-INFO-008",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Visibility", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Align", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_info_009 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-INFO-009",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "selected_objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "SizeX", kind: ParameterKind::Number, required: false },
        Parameter { name: "SizeY", kind: ParameterKind::Number, required: false },
        Parameter { name: "SizeZ", kind: ParameterKind::Number, required: false },
        Parameter { name: "ScaleX", kind: ParameterKind::Number, required: false },
        Parameter { name: "ScaleY", kind: ParameterKind::Number, required: false },
        Parameter { name: "ScaleZ", kind: ParameterKind::Number, required: false },
        Parameter { name: "PositionX", kind: ParameterKind::Number, required: false },
        Parameter { name: "PositionY", kind: ParameterKind::Number, required: false },
        Parameter { name: "PositionZ", kind: ParameterKind::Number, required: false },
        Parameter { name: "RotationX", kind: ParameterKind::Number, required: false },
        Parameter { name: "RotationY", kind: ParameterKind::Number, required: false },
        Parameter { name: "RotationZ", kind: ParameterKind::Number, required: false },
        Parameter { name: "Increment", kind: ParameterKind::Number, required: false },
        Parameter { name: "PivotX", kind: ParameterKind::Choice, required: false },
        Parameter { name: "PivotY", kind: ParameterKind::Choice, required: false },
        Parameter { name: "PivotZ", kind: ParameterKind::Choice, required: false },
        Parameter { name: "CoordinateSpace", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Transform objects individually", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Show Bounding Box", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Copy Objects", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Select Copied Objects", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Action", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_info_010 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-INFO-010",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Viewport", kind: ParameterKind::Reference, required: false },
        Parameter { name: "DisplayMode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "ShowCurves", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Transparency", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_info_011 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-INFO-011",
    effect: Effect::ExternalOutput,
    roles: &[
        Role { name: "active_document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "LibraryRoot", kind: ParameterKind::Text, required: false },
        Parameter { name: "Resource", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Action", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_info_012 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-INFO-012",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "AllowedTypes", kind: ParameterKind::Text, required: false },
        Parameter { name: "Disable", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "OneShot", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_info_013 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-INFO-013",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Enabled", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "CaptureMode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Clear", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_info_014 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-INFO-014",
    effect: Effect::ExternalOutput,
    roles: &[
        Role { name: "active_document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Group Matching Stones", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "StoneFamily", kind: ParameterKind::Choice, required: false },
        Parameter { name: "StoneColor", kind: ParameterKind::Choice, required: false },
        Parameter { name: "StoneQuality", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Sprue", kind: ParameterKind::Number, required: false },
        Parameter { name: "Additional Notes", kind: ParameterKind::Text, required: false },
        Parameter { name: "Stuller Pricing", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "QuoteItem", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Action", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_info_015 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-INFO-015",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Record", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Update", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Lock", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "BrokenHistoryWarning", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_info_016 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-INFO-016",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "selected_objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_info_017 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-INFO-017",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Enabled", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_info_018 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-INFO-018",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Enabled", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_info_019 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-INFO-019",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "active_document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Enabled", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Target", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Handle", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_info_020 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-INFO-020",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "selected_objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Enabled", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Operation", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Axis", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Distance", kind: ParameterKind::Number, required: false },
        Parameter { name: "Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Scale Factor", kind: ParameterKind::Number, required: false },
        Parameter { name: "Copy", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_info_021 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-INFO-021",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "selected_objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Alignment", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_info_022 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-INFO-022",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "selected_objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Origin", kind: ParameterKind::Reference, required: false },
        Parameter { name: "XDirection", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_history_001 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-HISTORY-001",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "active_document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Record", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Update", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Lock", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "BrokenHistoryWarning", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_layer_001 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-LAYER-001",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "active_document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Layer", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Action", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Name", kind: ParameterKind::Text, required: false },
        Parameter { name: "Color", kind: ParameterKind::Text, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_project_001 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-PROJECT-001",
    effect: Effect::ExternalOutput,
    roles: &[
        Role { name: "active_document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "RowName", kind: ParameterKind::Text, required: false },
        Parameter { name: "Bag", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Action", kind: ParameterKind::Choice, required: false },
        Parameter { name: "ObjectTypes", kind: ParameterKind::Text, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_projectdb_001 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-PROJECTDB-001",
    effect: Effect::ExternalOutput,
    roles: &[
        Role { name: "active_document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "StorageRoot", kind: ParameterKind::Text, required: false },
        Parameter { name: "Action", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Text", kind: ParameterKind::Text, required: false },
        Parameter { name: "Jobs", kind: ParameterKind::Choice, required: false },
        Parameter { name: "NumericPredicate", kind: ParameterKind::Text, required: false },
        Parameter { name: "Date", kind: ParameterKind::Text, required: false },
        Parameter { name: "Customer", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Designer", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Tags", kind: ParameterKind::Text, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_snap_001 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SNAP-001",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Enabled", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "MasterOsnap", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_snap_002 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SNAP-002",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Enabled", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "MasterOsnap", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_snap_003 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SNAP-003",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Enabled", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "MasterOsnap", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_snap_004 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SNAP-004",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Enabled", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "MasterOsnap", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_snap_005 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SNAP-005",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Enabled", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "MasterOsnap", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_snap_006 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SNAP-006",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Enabled", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "MasterOsnap", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_snap_007 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SNAP-007",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Enabled", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "MasterOsnap", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_snap_008 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SNAP-008",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Enabled", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "MasterOsnap", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_snap_009 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SNAP-009",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Enabled", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "MasterOsnap", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_snap_010 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SNAP-010",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Enabled", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "MasterOsnap", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_snap_011 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SNAP-011",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Enabled", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "MasterOsnap", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Target", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_snap_012 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SNAP-012",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Enabled", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "MasterOsnap", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Target", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_snap_013 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SNAP-013",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Enabled", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "MasterOsnap", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Increment", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_snap_014 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SNAP-014",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Enabled", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "MasterOsnap", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_snap_015 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SNAP-015",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Enabled", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "MasterOsnap", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_snap_016 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SNAP-016",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Enabled", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "MasterOsnap", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_display_001 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-DISPLAY-001",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Enabled", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_display_002 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-DISPLAY-002",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Enabled", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_display_003 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-DISPLAY-003",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Enabled", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_display_004 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-DISPLAY-004",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Enabled", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_display_005 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-DISPLAY-005",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "gem", kind: InputKind::Gem, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Enabled", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_display_006 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-DISPLAY-006",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Quality", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_display_007 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-DISPLAY-007",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "surface", kind: InputKind::Surface, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Enabled", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_display_008 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-DISPLAY-008",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Enabled", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_top11_001 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOP11-001",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "selected_objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "ControlMode", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_top11_002 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOP11-002",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "selected_objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Vertical", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "In Place", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "From Last Point", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Use Last Distance", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Use Last Direction", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_top11_003 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOP11-003",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "selected_objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "EditMode", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_top11_004 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOP11-004",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "selected_objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "3point", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Copy", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Axis", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_top11_005 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOP11-005",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "selected_objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_top11_006 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOP11-006",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "selected_objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Vertical", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_top11_007 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOP11-007",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "selected_objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Copy", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_top11_008 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOP11-008",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "selected_objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Undo", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_top11_009 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOP11-009",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "selected_objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Direction", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Toggle", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Shrink", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_top11_010 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOP11-010",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "selected_objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Extend Lines", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Use Apparent Intersections", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_top11_011 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOP11-011",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "active_document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Region", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Unit", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Measure", kind: ParameterKind::Choice, required: false },
        Parameter { name: "ExistingRail", kind: ParameterKind::Choice, required: false },
        Parameter { name: "CustomRail", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_file_001 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-FILE-001",
    effect: Effect::ExternalOutput,
    roles: &[
        Role { name: "selected_objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Path", kind: ParameterKind::Text, required: false },
        Parameter { name: "Format", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Save Small", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Save Geometry Only", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Save Textures", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_file_002 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-FILE-002",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "active_document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_file_003 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-FILE-003",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "active_document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Path", kind: ParameterKind::Text, required: false },
        Parameter { name: "Format", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_file_004 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-FILE-004",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "active_document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Path", kind: ParameterKind::Text, required: false },
        Parameter { name: "Format", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_file_005 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-FILE-005",
    effect: Effect::ExternalOutput,
    roles: &[
        Role { name: "active_document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Path", kind: ParameterKind::Text, required: false },
        Parameter { name: "Format", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_file_006 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-FILE-006",
    effect: Effect::ExternalOutput,
    roles: &[
        Role { name: "active_document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Path", kind: ParameterKind::Text, required: false },
        Parameter { name: "Format", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_file_007 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-FILE-007",
    effect: Effect::ExternalOutput,
    roles: &[
        Role { name: "active_document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Path", kind: ParameterKind::Text, required: false },
        Parameter { name: "Format", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_file_008 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-FILE-008",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "active_document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Text", kind: ParameterKind::Text, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_file_009 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-FILE-009",
    effect: Effect::ExternalOutput,
    roles: &[
        Role { name: "active_document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Printer", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Size", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Orientation", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Copies", kind: ParameterKind::Number, required: false },
        Parameter { name: "Print to File", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Output Type", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Output Color", kind: ParameterKind::Choice, required: false },
        Parameter { name: "View", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Multiple Layouts", kind: ParameterKind::Text, required: false },
        Parameter { name: "Print All Layouts", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Scale", kind: ParameterKind::Number, required: false },
        Parameter { name: "MarginUnits", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Margins", kind: ParameterKind::Text, required: false },
        Parameter { name: "Position", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Scale X", kind: ParameterKind::Number, required: false },
        Parameter { name: "Scale Y", kind: ParameterKind::Number, required: false },
        Parameter { name: "Visibility", kind: ParameterKind::Text, required: false },
        Parameter { name: "Notes", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Filename", kind: ParameterKind::Choice, required: false },
        Parameter { name: "LineWidthScale", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_file_010 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-FILE-010",
    effect: Effect::ExternalOutput,
    roles: &[
        Role { name: "active_document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Path", kind: ParameterKind::Text, required: false },
        Parameter { name: "Format", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_file_011 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-FILE-011",
    effect: Effect::ExternalOutput,
    roles: &[
        Role { name: "active_document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_view_001 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-VIEW-001",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "image", kind: InputKind::Image, min: 1, max: Some(1) },
        Role { name: "picked_points", kind: InputKind::Point, min: 2, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "Vertical", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "SelfIllumination", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "EmbedBitmap", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Autoname", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "AlphaTrasnparency", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Length", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_view_002 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-VIEW-002",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_view_003 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-VIEW-003",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_view_004 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-VIEW-004",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_view_005 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-VIEW-005",
    effect: Effect::ExternalOutput,
    roles: &[
        Role { name: "active_viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Path", kind: ParameterKind::Text, required: false },
        Parameter { name: "Format", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_view_006 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-VIEW-006",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_view_007 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-VIEW-007",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "selected_objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_view_008 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-VIEW-008",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_view_009 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-VIEW-009",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Enabled", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_view_010 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-VIEW-010",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Path", kind: ParameterKind::Text, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_view_011 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-VIEW-011",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_view_012 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-VIEW-012",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_view_013 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-VIEW-013",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "MeasuredLength", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_util_001 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-UTIL-001",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "selected_objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "NextMode", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_util_002 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-UTIL-002",
    effect: Effect::ReadOnly,
    roles: &[
        Role { name: "selected_objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_util_003 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-UTIL-003",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_util_004 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-UTIL-004",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "selected_objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "EdgeMode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Edge Color", kind: ParameterKind::Text, required: false },
        Parameter { name: "Zoom", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Mark", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_util_005 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-UTIL-005",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "selected_objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_util_006 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-UTIL-006",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "selected_objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_util_007 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-UTIL-007",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "selected_objects", kind: InputKind::Object, min: 2, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "AcceptedTolerance", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_util_008 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-UTIL-008",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "selected_objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Layout", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Show Tangent Edges", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Create hidden Lines", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Show Viewport Rectangle", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Maintain Source Layers", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "OutputLayers", kind: ParameterKind::Text, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_util_009 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-UTIL-009",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "selected_objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_util_010 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-UTIL-010",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "selected_objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Coordinate System", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Output", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_util_011 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-UTIL-011",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "selected_objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Axes", kind: ParameterKind::Text, required: false },
        Parameter { name: "Action", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Coordinate Space", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_util_012 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-UTIL-012",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "selected_objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Action", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Name", kind: ParameterKind::Text, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_util_013 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-UTIL-013",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "selected_objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_util_014 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-UTIL-014",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Enabled", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_util_015 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-UTIL-015",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "selected_objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_util_016 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-UTIL-016",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "mesh", kind: InputKind::Mesh, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "SelectionMode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Distance To Adjust", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_util_017 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-UTIL-017",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "mesh", kind: InputKind::Mesh, min: 1, max: None },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_util_018 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-UTIL-018",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "mesh", kind: InputKind::Mesh, min: 1, max: None },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_util_019 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-UTIL-019",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "selected_objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Density", kind: ParameterKind::Number, required: false },
        Parameter { name: "Maximum Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Maximum Aspect Ratio", kind: ParameterKind::Number, required: false },
        Parameter { name: "Minimum Edge Length", kind: ParameterKind::Number, required: false },
        Parameter { name: "Maximum Edge Length", kind: ParameterKind::Number, required: false },
        Parameter { name: "Maximum Distance Edge to Surface", kind: ParameterKind::Number, required: false },
        Parameter { name: "Minimum Initial Grid Quads", kind: ParameterKind::Number, required: false },
        Parameter { name: "Refine Mesh", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Jagged Seams", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Simple Planes", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Pack Textures", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Preview", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_util_020 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-UTIL-020",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "mesh", kind: InputKind::Mesh, min: 1, max: Some(1) },
        Role { name: "target_surface", kind: InputKind::Surface, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "BaseMapping", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_util_021 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-UTIL-021",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "mesh", kind: InputKind::Mesh, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "TargetCount", kind: ParameterKind::Number, required: false },
        Parameter { name: "Or Planar Only", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Accuracy", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Preview", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_util_022 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-UTIL-022",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "selected_objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_measure_001 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-MEASURE-001",
    effect: Effect::ReadOnly,
    roles: &[
        Role { name: "picked_points", kind: InputKind::Point, min: 4, max: Some(4) },
    ],
    parameters: &[
        Parameter { name: "TwoObjects", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_measure_002 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-MEASURE-002",
    effect: Effect::ReadOnly,
    roles: &[
        Role { name: "picked_points", kind: InputKind::Point, min: 2, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "Unit", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_measure_003 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-MEASURE-003",
    effect: Effect::ReadOnly,
    roles: &[
        Role { name: "curves_or_edges", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Unit", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_measure_004 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-MEASURE-004",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "picked_points", kind: InputKind::Point, min: 3, max: None },
    ],
    parameters: &[
        Parameter { name: "Style", kind: ParameterKind::Text, required: false },
        Parameter { name: "Object", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Continue", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Alignment", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_measure_005 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-MEASURE-005",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "curves_or_edges", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "SelectCurve", kind: ParameterKind::Reference, required: false },
        Parameter { name: "MarkRadius", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_measure_006 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-MEASURE-006",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "picked_points", kind: InputKind::Point, min: 3, max: None },
    ],
    parameters: &[
        Parameter { name: "Style", kind: ParameterKind::Text, required: false },
        Parameter { name: "Object", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Continue", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Alignment", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_measure_007 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-MEASURE-007",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "curves_or_edges", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Style", kind: ParameterKind::Text, required: false },
        Parameter { name: "Points", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_measure_008 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-MEASURE-008",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "picked_points", kind: InputKind::Point, min: 3, max: None },
    ],
    parameters: &[
        Parameter { name: "Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Style", kind: ParameterKind::Text, required: false },
        Parameter { name: "Object", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Continue", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_measure_009 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-MEASURE-009",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "picked_points", kind: InputKind::Point, min: 3, max: None },
    ],
    parameters: &[
        Parameter { name: "Style", kind: ParameterKind::Text, required: false },
        Parameter { name: "Object", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Continue", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Undo", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_measure_010 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-MEASURE-010",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "curves_or_edges", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Style", kind: ParameterKind::Text, required: false },
        Parameter { name: "PointOnCurve", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_measure_011 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-MEASURE-011",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "curves_or_edges", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Style", kind: ParameterKind::Text, required: false },
        Parameter { name: "PointOnCurve", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_measure_012 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-MEASURE-012",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "picked_points", kind: InputKind::Point, min: 3, max: None },
    ],
    parameters: &[
        Parameter { name: "Text", kind: ParameterKind::Text, required: false },
        Parameter { name: "Style", kind: ParameterKind::Text, required: false },
        Parameter { name: "Undo", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Text Field", kind: ParameterKind::Text, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_measure_013 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-MEASURE-013",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "active_document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Text", kind: ParameterKind::Text, required: false },
        Parameter { name: "Font", kind: ParameterKind::Text, required: false },
        Parameter { name: "Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Mask", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Margin", kind: ParameterKind::Number, required: false },
        Parameter { name: "Layout Scaling", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Bold", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Italic", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "HorizontalAlign", kind: ParameterKind::Choice, required: false },
        Parameter { name: "VerticalAlign", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_measure_014 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-MEASURE-014",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "active_document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Style", kind: ParameterKind::Text, required: false },
        Parameter { name: "Action", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Name", kind: ParameterKind::Text, required: false },
        Parameter { name: "Copy setting from", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Font", kind: ParameterKind::Text, required: false },
        Parameter { name: "Display", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_measure_015 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-MEASURE-015",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "text_object", kind: InputKind::Text, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Text", kind: ParameterKind::Text, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_measure_016 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-MEASURE-016",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "active_document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Precision", kind: ParameterKind::Number, required: false },
        Parameter { name: "Number Format", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Text Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Extension line Extension", kind: ParameterKind::Number, required: false },
        Parameter { name: "Extension line Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Arrow Length", kind: ParameterKind::Number, required: false },
        Parameter { name: "Text Alignment", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Style", kind: ParameterKind::Text, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_measure_017 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-MEASURE-017",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "selected_dimensions", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_001 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-001",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "picked_points", kind: InputKind::Point, min: 2, max: None },
    ],
    parameters: &[
        Parameter { name: "PersistentClose", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Close", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Length", kind: ParameterKind::Number, required: false },
        Parameter { name: "Direction", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Center", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Helpers", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_002 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-002",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "picked_points", kind: InputKind::Point, min: 2, max: None },
    ],
    parameters: &[
        Parameter { name: "BothSides", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Normal", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "IgnoreTrims", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Angled", kind: ParameterKind::Number, required: false },
        Parameter { name: "Vertical", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "FourPoint", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Bisector", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Perpendicular", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Tangent", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "FromFirstPoint", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "2Curves", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Extension", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_003 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-003",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "picked_points", kind: InputKind::Point, min: 2, max: None },
    ],
    parameters: &[
        Parameter { name: "Degree", kind: ParameterKind::Number, required: false },
        Parameter { name: "Knots", kind: ParameterKind::Choice, required: false },
        Parameter { name: "PersistentClose", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Start Tangent", kind: ParameterKind::Reference, required: false },
        Parameter { name: "End Tangent", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Close", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Sharp", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Undo", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_004 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-004",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "picked_points", kind: InputKind::Point, min: 2, max: None },
    ],
    parameters: &[
        Parameter { name: "3Point", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Vertical", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Center", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Rounded", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Corner", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Length", kind: ParameterKind::Number, required: false },
        Parameter { name: "Width", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_005 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-005",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "picked_points", kind: InputKind::Point, min: 2, max: None },
    ],
    parameters: &[
        Parameter { name: "Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Diameter", kind: ParameterKind::Number, required: false },
        Parameter { name: "Circumference", kind: ParameterKind::Number, required: false },
        Parameter { name: "Area", kind: ParameterKind::Number, required: false },
        Parameter { name: "Orientation", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Deformable", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Degree", kind: ParameterKind::Number, required: false },
        Parameter { name: "PointCount", kind: ParameterKind::Number, required: false },
        Parameter { name: "Vertical", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "2Point", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "3Point", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Tangent", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Point", kind: ParameterKind::Reference, required: false },
        Parameter { name: "FromFirstPoint", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "AroundCurve", kind: ParameterKind::Reference, required: false },
        Parameter { name: "FitPoints", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_006 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-006",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "picked_points", kind: InputKind::Point, min: 2, max: None },
    ],
    parameters: &[
        Parameter { name: "Deformable", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Degree", kind: ParameterKind::Number, required: false },
        Parameter { name: "PointCount", kind: ParameterKind::Number, required: false },
        Parameter { name: "Vertical", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Corner", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Diameter", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "FromFoci", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "MarkFoci", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "AroundCurve", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_007 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-007",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "picked_points", kind: InputKind::Point, min: 2, max: None },
    ],
    parameters: &[
        Parameter { name: "Tilted", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Length", kind: ParameterKind::Number, required: false },
        Parameter { name: "Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Deformable", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Degree", kind: ParameterKind::Number, required: false },
        Parameter { name: "PointCount", kind: ParameterKind::Number, required: false },
        Parameter { name: "StartPoint", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Direction", kind: ParameterKind::Reference, required: false },
        Parameter { name: "ThroughPoint", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Center", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Tangent", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Point", kind: ParameterKind::Reference, required: false },
        Parameter { name: "FromFirstPoint", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Extension", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_008 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-008",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "picked_points", kind: InputKind::Point, min: 2, max: None },
    ],
    parameters: &[
        Parameter { name: "Direction", kind: ParameterKind::Reference, required: false },
        Parameter { name: "ThroughPoint", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Center", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_009 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-009",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "input_curves", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "PointCount", kind: ParameterKind::Number, required: false },
        Parameter { name: "Degree", kind: ParameterKind::Number, required: false },
        Parameter { name: "DeleteInput", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Create new object on current layer", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Preview", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_010 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-010",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "input_curves", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Tolerance", kind: ParameterKind::Number, required: false },
        Parameter { name: "DeleteInput", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Degree", kind: ParameterKind::Number, required: false },
        Parameter { name: "OutputLayer", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Angle Tolerance", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_011 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-011",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "input_curves_or_edges", kind: InputKind::Curve, min: 2, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "Perpendicular", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "AtAngle", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Continuity", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_012 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-012",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "input_curves_or_edges", kind: InputKind::Curve, min: 2, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "SelectionMode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Continuity Curve 1", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Continuity Curve 2", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Reset", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Flip 1", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Flip 2", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Trim", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Join", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Show Curvature", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_013 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-013",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "input_curves", kind: InputKind::Curve, min: 2, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "Direction", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_014 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-014",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "input_curves", kind: InputKind::Curve, min: 2, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "Full", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_015 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-015",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "input_curves", kind: InputKind::Curve, min: 1, max: None },
        Role { name: "target", kind: InputKind::Object, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Loose", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "DeleteInput", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "OutputLayer", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_016 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-016",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "input_curves", kind: InputKind::Curve, min: 1, max: None },
        Role { name: "target", kind: InputKind::Object, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Loose", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "DeleteInput", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "OutputLayer", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_017 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-017",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "input_objects", kind: InputKind::Object, min: 2, max: None },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_018 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-018",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "input_curves", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Distance", kind: ParameterKind::Number, required: false },
        Parameter { name: "Corner", kind: ParameterKind::Choice, required: false },
        Parameter { name: "ThroughPoint", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Tolerance", kind: ParameterKind::Number, required: false },
        Parameter { name: "BothSides", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "InCPlane", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Cap", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_019 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-019",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "target_surface", kind: InputKind::Surface, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Direction", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Toggle", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "ExtractAll", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "IgnoreTrims", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_020 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-020",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "input_curves", kind: InputKind::Curve, min: 1, max: None },
        Role { name: "target", kind: InputKind::Object, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Distance", kind: ParameterKind::Number, required: false },
        Parameter { name: "ThroughPoint", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Flip", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Number", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_021 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-021",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "input_curves_or_edges", kind: InputKind::Curve, min: 2, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Join", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Trim", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "ExtendArcsBy", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_022 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-022",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "input_curves_or_edges", kind: InputKind::Curve, min: 2, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "Distances", kind: ParameterKind::Text, required: false },
        Parameter { name: "Join", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Trim", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "ExtendArcsBy", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_023 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-023",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "input_curves", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Radius", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_024 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-024",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "input_curves", kind: InputKind::Curve, min: 1, max: None },
        Role { name: "target", kind: InputKind::Object, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_025 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-025",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "target_surface", kind: InputKind::Surface, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_026 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-026",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "ordered_points", kind: InputKind::Point, min: 2, max: None },
    ],
    parameters: &[
        Parameter { name: "Degree", kind: ParameterKind::Number, required: false },
        Parameter { name: "CurveType", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Knots", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Closed", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_027 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-027",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "input_curves", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Type", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Center", kind: ParameterKind::Reference, required: false },
        Parameter { name: "ToPoint", kind: ParameterKind::Reference, required: false },
        Parameter { name: "ExtensionLength", kind: ParameterKind::Number, required: false },
        Parameter { name: "Trim", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Undo", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_028 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-028",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "picked_points", kind: InputKind::Point, min: 2, max: None },
    ],
    parameters: &[
        Parameter { name: "NumSides", kind: ParameterKind::Number, required: false },
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Vertical", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "AroundCurve", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "InnerRadius", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_029 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-029",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "picked_points", kind: InputKind::Point, min: 2, max: None },
    ],
    parameters: &[
        Parameter { name: "Flat", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Vertical", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "AroundCurve", kind: ParameterKind::Reference, required: false },
        Parameter { name: "SizeMode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Turns", kind: ParameterKind::Number, required: false },
        Parameter { name: "Pitch", kind: ParameterKind::Number, required: false },
        Parameter { name: "ReverseTwist", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "NumPointsPerTurn", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_030 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-030",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "picked_points", kind: InputKind::Point, min: 2, max: None },
    ],
    parameters: &[
        Parameter { name: "Vertical", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "AroundCurve", kind: ParameterKind::Reference, required: false },
        Parameter { name: "SizeMode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Turns", kind: ParameterKind::Number, required: false },
        Parameter { name: "Pitch", kind: ParameterKind::Number, required: false },
        Parameter { name: "ReverseTwist", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "NumPointsPerTurn", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_031 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-031",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "input_curves", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Flip", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Automatic", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Natural", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_032 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-032",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "input_curves", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_033 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-033",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "input_curves", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_034 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-034",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "picked_points", kind: InputKind::Point, min: 1, max: None },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_035 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-035",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "input_curves", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "PersistentClose", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Close", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Sharp", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Undo", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_036 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-036",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "input_curves", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Number", kind: ParameterKind::Number, required: false },
        Parameter { name: "Length", kind: ParameterKind::Number, required: false },
        Parameter { name: "Split", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "MarkEnds", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "GroupOutput", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_037 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-037",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "input_curves", kind: InputKind::Curve, min: 2, max: None },
    ],
    parameters: &[
        Parameter { name: "DeleteInput", kind: ParameterKind::Choice, required: false },
        Parameter { name: "CombineRegions", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_038 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-038",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "input_curves", kind: InputKind::Curve, min: 2, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "Number", kind: ParameterKind::Number, required: false },
        Parameter { name: "MatchMethod", kind: ParameterKind::Choice, required: false },
        Parameter { name: "SampleNumber", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_039 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-039",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "input_objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "OutputLayer", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_040 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-040",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "input_objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "OutputLayer", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_041 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-041",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "input_curves", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "PersistentClose", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Close", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Sharp", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Undo", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_042 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-042",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "picked_points", kind: InputKind::Point, min: 2, max: None },
    ],
    parameters: &[
        Parameter { name: "Degree", kind: ParameterKind::Number, required: false },
        Parameter { name: "PersistentClose", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Close", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Sharp", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Undo", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_043 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-043",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "input_curves", kind: InputKind::Curve, min: 2, max: None },
    ],
    parameters: &[
        Parameter { name: "Closed", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_044 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-044",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "target_surface", kind: InputKind::Surface, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Angle", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_045 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-045",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "picked_points", kind: InputKind::Point, min: 2, max: None },
    ],
    parameters: &[
        Parameter { name: "Closed", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_046 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-046",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "target_surface", kind: InputKind::Surface, min: 1, max: Some(1) },
        Role { name: "picked_points", kind: InputKind::Point, min: 2, max: None },
    ],
    parameters: &[
        Parameter { name: "Closed", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_047 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-047",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "target_surface", kind: InputKind::Surface, min: 1, max: Some(1) },
        Role { name: "picked_points", kind: InputKind::Point, min: 2, max: None },
    ],
    parameters: &[
        Parameter { name: "Undo", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_048 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-048",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "target_surface", kind: InputKind::Surface, min: 1, max: Some(1) },
        Role { name: "picked_points", kind: InputKind::Point, min: 2, max: None },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_049 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-049",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "target_mesh", kind: InputKind::Mesh, min: 1, max: Some(1) },
        Role { name: "stroke_points", kind: InputKind::Point, min: 2, max: None },
    ],
    parameters: &[
        Parameter { name: "Closed", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_050 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-050",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "input_curves", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Output", kind: ParameterKind::Choice, required: false },
        Parameter { name: "SimplifyInput", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "DeleteInput", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "AngleTolerance", kind: ParameterKind::Number, required: false },
        Parameter { name: "Tolerance", kind: ParameterKind::Number, required: false },
        Parameter { name: "MinLength", kind: ParameterKind::Number, required: false },
        Parameter { name: "MaxLength", kind: ParameterKind::Number, required: false },
        Parameter { name: "OutputLayer", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_051 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-051",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "input_objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "AssignLayersBy", kind: ParameterKind::Choice, required: false },
        Parameter { name: "JoinCurves", kind: ParameterKind::Choice, required: false },
        Parameter { name: "GroupObjectsByContourPlane", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "ExtendSection", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_052 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-052",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "input_curves_or_edges", kind: InputKind::Curve, min: 2, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "SurfaceEdge", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Continuity", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Preserve other End", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Perpendicular to Edge", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Average Curves", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Join", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Merge", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_053 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-053",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "input_objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_054 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-054",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "input_objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "OutputLayer", kind: ParameterKind::Choice, required: false },
        Parameter { name: "GroupOutput", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_055 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-055",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "input_curves", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Distance", kind: ParameterKind::Number, required: false },
        Parameter { name: "Copy", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "FixEnds", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_056 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-056",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "input_curves_or_edges", kind: InputKind::Curve, min: 2, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "Ratio", kind: ParameterKind::Number, required: false },
        Parameter { name: "AlternateSolution", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "RadiusDifference", kind: ParameterKind::Number, required: false },
        Parameter { name: "Trim", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Join", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_057 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-057",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "input_curves", kind: InputKind::Curve, min: 1, max: None },
        Role { name: "target", kind: InputKind::Object, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Distance", kind: ParameterKind::Number, required: false },
        Parameter { name: "SetBasePoint", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Flip", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_curve_058 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-058",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "set_a", kind: InputKind::Object, min: 1, max: None },
        Role { name: "set_b", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_surface_001 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SURFACE-001",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "rail", kind: InputKind::Curve, min: 1, max: Some(1) },
        Role { name: "sections", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Style", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Closed Sweep", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Global Shape Blending", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Untrimmed Miters", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Align Shapes", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Cross-Section Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Rebuild PointCount", kind: ParameterKind::Number, required: false },
        Parameter { name: "Refit Tolerance", kind: ParameterKind::Number, required: false },
        Parameter { name: "Simple Sweep", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Refit Rail", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_surface_002 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SURFACE-002",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "rail", kind: InputKind::Curve, min: 1, max: Some(1) },
        Role { name: "sections", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Closed", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Flip", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_surface_003 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SURFACE-003",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "rails", kind: InputKind::Curve, min: 2, max: Some(2) },
        Role { name: "sections", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Cross-Section Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Rebuild PointCount", kind: ParameterKind::Number, required: false },
        Parameter { name: "Refit Tolerance", kind: ParameterKind::Number, required: false },
        Parameter { name: "Preserve First Shape", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Preserve Last Shape", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Maintain Height", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Rail A Continuity", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Rail B Continuity", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Closed Sweep", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Simple Sweep", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Add Slash", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_surface_004 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SURFACE-004",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "rails", kind: InputKind::Curve, min: 2, max: Some(2) },
        Role { name: "sections", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Closed", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Flip", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Maintain Height", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_surface_005 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SURFACE-005",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "profiles", kind: InputKind::Curve, min: 2, max: Some(2) },
        Role { name: "avoid_ring", kind: InputKind::Curve, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Blend Amount", kind: ParameterKind::Number, required: false },
        Parameter { name: "Start Blend", kind: ParameterKind::Number, required: false },
        Parameter { name: "End Blend", kind: ParameterKind::Number, required: false },
        Parameter { name: "Ring Rail", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_surface_006 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SURFACE-006",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "surfaces", kind: InputKind::Surface, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Point Count U", kind: ParameterKind::Number, required: false },
        Parameter { name: "Point Count V", kind: ParameterKind::Number, required: false },
        Parameter { name: "Degree", kind: ParameterKind::Number, required: false },
        Parameter { name: "Delete Input", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Current Layer", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Retrim", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Maximum deviation / Calculate", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Preview", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_surface_007 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SURFACE-007",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "edge_chains", kind: InputKind::Curve, min: 2, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "Lock", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Sliders", kind: ParameterKind::Number, required: false },
        Parameter { name: "Continuity", kind: ParameterKind::Choice, required: false },
        Parameter { name: "AddShapes", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Planar Sections", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Same Height", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Preview", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_surface_008 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SURFACE-008",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "surfaces", kind: InputKind::Surface, min: 2, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "Radius/Distance", kind: ParameterKind::Number, required: false },
        Parameter { name: "LinkHandles", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Rail-Type", kind: ParameterKind::Choice, required: false },
        Parameter { name: "TrimAndJoin", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Preview", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_surface_009 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SURFACE-009",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "sections", kind: InputKind::Curve, min: 2, max: None },
    ],
    parameters: &[
        Parameter { name: "Style", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Closed loft", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Match Start Tangent", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Match End Tangent", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "SplitAtTangents", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Rebuild PointCount", kind: ParameterKind::Number, required: false },
        Parameter { name: "Refit Tolerance", kind: ParameterKind::Number, required: false },
        Parameter { name: "Preview", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_surface_010 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SURFACE-010",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "network", kind: InputKind::Curve, min: 3, max: None },
    ],
    parameters: &[
        Parameter { name: "NoAutoSort", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Edge curves Tolerance", kind: ParameterKind::Number, required: false },
        Parameter { name: "Interior curves Tolerance", kind: ParameterKind::Number, required: false },
        Parameter { name: "Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Edge Matching A", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Edge Matching B", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Edge Matching C", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Edge Matching D", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Preview", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_surface_011 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SURFACE-011",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "samples", kind: InputKind::Object, min: 1, max: None },
        Role { name: "starting_surface", kind: InputKind::Surface, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Sample Point Spacing", kind: ParameterKind::Number, required: false },
        Parameter { name: "Surface U Spans", kind: ParameterKind::Number, required: false },
        Parameter { name: "Surface V Spans", kind: ParameterKind::Number, required: false },
        Parameter { name: "Stiffness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Adjust Tangency", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Automatic Trim", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Starting Surface Pull", kind: ParameterKind::Number, required: false },
        Parameter { name: "Preserve Edges", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Delete Input", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_surface_012 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SURFACE-012",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "trimmed_surfaces", kind: InputKind::Surface, min: 1, max: None },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_surface_013 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SURFACE-013",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "boundaries", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_surface_014 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SURFACE-014",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "edges", kind: InputKind::Curve, min: 2, max: Some(4) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_surface_015 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SURFACE-015",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "definition_points", kind: InputKind::Point, min: 2, max: Some(3) },
    ],
    parameters: &[
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Deformable", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "UDegree", kind: ParameterKind::Number, required: false },
        Parameter { name: "VDegree", kind: ParameterKind::Number, required: false },
        Parameter { name: "UPointCount", kind: ParameterKind::Number, required: false },
        Parameter { name: "VPointCount", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_surface_016 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SURFACE-016",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "profiles", kind: InputKind::Object, min: 1, max: None },
        Role { name: "path", kind: InputKind::Curve, min: 0, max: Some(1) },
        Role { name: "tip", kind: InputKind::Point, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Distance", kind: ParameterKind::Number, required: false },
        Parameter { name: "Direction", kind: ParameterKind::Reference, required: false },
        Parameter { name: "BothSides", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Solid", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "DeleteInput", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "ToBoundary", kind: ParameterKind::Reference, required: false },
        Parameter { name: "SplitAtTangents", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "SetBasePoint", kind: ParameterKind::Reference, required: false },
        Parameter { name: "DraftAngle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Corners", kind: ParameterKind::Choice, required: false },
        Parameter { name: "FlipAngle", kind: ParameterKind::Reference, required: false },
        Parameter { name: "SubCurve", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_surface_017 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SURFACE-017",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "trim_edges", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "KeepTrimObjects", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "AllSimilar", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_surface_018 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SURFACE-018",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "profiles", kind: InputKind::Curve, min: 1, max: None },
        Role { name: "axis_points", kind: InputKind::Point, min: 2, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "StartAngle", kind: ParameterKind::Number, required: false },
        Parameter { name: "RevolutionAngle", kind: ParameterKind::Number, required: false },
        Parameter { name: "DeleteInput", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Deformable", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "PointCount", kind: ParameterKind::Number, required: false },
        Parameter { name: "FullCircle", kind: ParameterKind::Reference, required: false },
        Parameter { name: "AskForStartAngle", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "SplitAtTangents", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_surface_019 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SURFACE-019",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "profile", kind: InputKind::Curve, min: 1, max: Some(1) },
        Role { name: "rail", kind: InputKind::Curve, min: 1, max: Some(1) },
        Role { name: "axis_points", kind: InputKind::Point, min: 2, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "ScaleHeight", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "SplitAtTangents", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_surface_020 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SURFACE-020",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "surfaces", kind: InputKind::Surface, min: 2, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Extend", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Trim", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Split", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_surface_021 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SURFACE-021",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "profiles", kind: InputKind::Curve, min: 2, max: None },
        Role { name: "rails", kind: InputKind::Curve, min: 4, max: Some(4) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_surface_022 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SURFACE-022",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "surfaces", kind: InputKind::Surface, min: 2, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Rail Type", kind: ParameterKind::Choice, required: false },
        Parameter { name: "TrimAndJoin", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Preview", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_surface_023 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SURFACE-023",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "surfaces", kind: InputKind::Surface, min: 2, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "Distances", kind: ParameterKind::Number, required: false },
        Parameter { name: "Extend", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Trim", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_surface_024 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SURFACE-024",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "surfaces", kind: InputKind::Surface, min: 2, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "Distance", kind: ParameterKind::Number, required: false },
        Parameter { name: "TrimAndJoin", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Preview", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_surface_025 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SURFACE-025",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "surfaces", kind: InputKind::Surface, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Distance", kind: ParameterKind::Number, required: false },
        Parameter { name: "Corner", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Solid", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Loose", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Tolerance", kind: ParameterKind::Number, required: false },
        Parameter { name: "Both Sides", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "FlipAll", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_surface_026 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SURFACE-026",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "surface", kind: InputKind::Surface, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Tolerance", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Flip", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "SetAll", kind: ParameterKind::Reference, required: false },
        Parameter { name: "LinkHandles", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "UnlinkHandles", kind: ParameterKind::Reference, required: false },
        Parameter { name: "AddHandle", kind: ParameterKind::Reference, required: false },
        Parameter { name: "SideTangency", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_surface_027 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SURFACE-027",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "surfaces", kind: InputKind::Surface, min: 2, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "NumberOfSurfaces", kind: ParameterKind::Number, required: false },
        Parameter { name: "MatchMethod", kind: ParameterKind::Choice, required: false },
        Parameter { name: "SampleNumber", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_surface_028 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SURFACE-028",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "source_edges", kind: InputKind::Curve, min: 1, max: None },
        Role { name: "target_edges", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Continuity", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Preserve Other End", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Average Surfaces", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Match Edges by Closest Points", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Refine Match", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Distance", kind: ParameterKind::Number, required: false },
        Parameter { name: "Tangency", kind: ParameterKind::Number, required: false },
        Parameter { name: "Curvature", kind: ParameterKind::Number, required: false },
        Parameter { name: "Isocurve Direction Adjustment", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_surface_029 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SURFACE-029",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "surfaces", kind: InputKind::Surface, min: 2, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "Smooth", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Tolerance", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Roundness", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_surface_030 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SURFACE-030",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "objects", kind: InputKind::Object, min: 1, max: None },
        Role { name: "viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "AutoSpacing", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Spacing", kind: ParameterKind::Number, required: false },
        Parameter { name: "AutoDetectMaxDepth", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "MaxDepth", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_surface_031 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SURFACE-031",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "image", kind: InputKind::Image, min: 1, max: Some(1) },
        Role { name: "corners", kind: InputKind::Point, min: 2, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "U Sample Count", kind: ParameterKind::Number, required: false },
        Parameter { name: "V Sample Count", kind: ParameterKind::Number, required: false },
        Parameter { name: "Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Set Image as Texture", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Create Vertex Colors", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Create Object By", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_surface_032 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SURFACE-032",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "rails", kind: InputKind::Curve, min: 2, max: Some(4) },
        Role { name: "profiles", kind: InputKind::Curve, min: 2, max: None },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_surface_033 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SURFACE-033",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "surface_edge", kind: InputKind::Curve, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Distance", kind: ParameterKind::Number, required: false },
        Parameter { name: "Type", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_surface_034 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SURFACE-034",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "surface", kind: InputKind::Surface, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Direction", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Toggle", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Shrink", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_surface_035 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SURFACE-035",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "curve", kind: InputKind::Curve, min: 1, max: Some(1) },
        Role { name: "surface", kind: InputKind::Surface, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Distance", kind: ParameterKind::Number, required: false },
        Parameter { name: "SetBasePoint", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Direction", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_surface_036 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SURFACE-036",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "surfaces", kind: InputKind::Surface, min: 1, max: None },
        Role { name: "attached_curves", kind: InputKind::Curve, min: 0, max: None },
    ],
    parameters: &[
        Parameter { name: "Explode", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Labels", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "KeepProperties", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "RelativeTolerance", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_surface_037 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SURFACE-037",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "surface", kind: InputKind::Surface, min: 1, max: Some(1) },
        Role { name: "move_points", kind: InputKind::Point, min: 2, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "U Distance", kind: ParameterKind::Number, required: false },
        Parameter { name: "V Distance", kind: ParameterKind::Number, required: false },
        Parameter { name: "Copy", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "FixEdges", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "DirConstraint", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_surface_038 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SURFACE-038",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "surface", kind: InputKind::Surface, min: 1, max: Some(1) },
        Role { name: "seam_point", kind: InputKind::Point, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_surface_039 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SURFACE-039",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "targets", kind: InputKind::Object, min: 1, max: None },
        Role { name: "definition_points", kind: InputKind::Point, min: 2, max: None },
    ],
    parameters: &[
        Parameter { name: "3Point", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_surface_040 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SURFACE-040",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "edge", kind: InputKind::Curve, min: 1, max: Some(1) },
        Role { name: "direction_points", kind: InputKind::Point, min: 2, max: Some(2) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_solid_001 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SOLID-001",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "targets", kind: InputKind::Solid, min: 1, max: None },
        Role { name: "cutters", kind: InputKind::Solid, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "DeleteInput", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_solid_002 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SOLID-002",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "first_set", kind: InputKind::Solid, min: 1, max: None },
        Role { name: "second_set", kind: InputKind::Solid, min: 1, max: None },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_solid_003 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SOLID-003",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "solids", kind: InputKind::Solid, min: 2, max: None },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_solid_004 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SOLID-004",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "objects", kind: InputKind::Solid, min: 2, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "Operation", kind: ParameterKind::Choice, required: false },
        Parameter { name: "DeleteInput", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_solid_005 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SOLID-005",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "shells", kind: InputKind::Surface, min: 1, max: None },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_solid_006 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SOLID-006",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "faces", kind: InputKind::Surface, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Copy", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Outputlayer", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_solid_007 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SOLID-007",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "edges", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "ShowRadius", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "NextRadius", kind: ParameterKind::Number, required: false },
        Parameter { name: "CurrentRadius", kind: ParameterKind::Number, required: false },
        Parameter { name: "LinkHandles", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "RailType", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Preview", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "TrimAndJoin", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_solid_008 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SOLID-008",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "placement", kind: InputKind::Point, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Text to Create", kind: ParameterKind::Text, required: false },
        Parameter { name: "Font Name", kind: ParameterKind::Text, required: false },
        Parameter { name: "Bold", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Italic", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Create", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Group Objects", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Allow single-stroke fonts", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Solid Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Lower case as small caps", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Small Caps Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Add spacing", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_solid_009 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SOLID-009",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "profiles", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Direction", kind: ParameterKind::Reference, required: false },
        Parameter { name: "BothSides", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Solid", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "ToBoundary", kind: ParameterKind::Reference, required: false },
        Parameter { name: "DeleteInput", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "SplitAtTangents", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "SetBasePoint", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Distance", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_solid_010 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SOLID-010",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "inputs", kind: InputKind::Object, min: 1, max: None },
        Role { name: "path", kind: InputKind::Curve, min: 0, max: Some(1) },
        Role { name: "tip", kind: InputKind::Point, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Direction", kind: ParameterKind::Reference, required: false },
        Parameter { name: "BothSides", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Solid", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "ToBoundary", kind: ParameterKind::Reference, required: false },
        Parameter { name: "DeleteInput", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "SplitAtTangents", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "SetBasePoint", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Distance", kind: ParameterKind::Number, required: false },
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Draft Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Corners", kind: ParameterKind::Choice, required: false },
        Parameter { name: "FlipAngle", kind: ParameterKind::Reference, required: false },
        Parameter { name: "SubCurve", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_solid_011 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SOLID-011",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "rails", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Diameter/Radius", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Start Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "End Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Thick", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Cap", kind: ParameterKind::Choice, required: false },
        Parameter { name: "FitRail", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "ShapeBlending", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_solid_012 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SOLID-012",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "definition_points", kind: InputKind::Point, min: 2, max: None },
    ],
    parameters: &[
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Length", kind: ParameterKind::Number, required: false },
        Parameter { name: "Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "3Point", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Vertical", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Center", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Cube", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_solid_013 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SOLID-013",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "definition_points", kind: InputKind::Point, min: 3, max: None },
    ],
    parameters: &[
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Axis1", kind: ParameterKind::Number, required: false },
        Parameter { name: "Axis2", kind: ParameterKind::Number, required: false },
        Parameter { name: "Axis3", kind: ParameterKind::Number, required: false },
        Parameter { name: "MarkFoci", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_solid_014 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SOLID-014",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "definition_points", kind: InputKind::Point, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Diameter", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_solid_015 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SOLID-015",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "definition_points", kind: InputKind::Point, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Base Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Major Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Minor Radius", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_solid_016 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SOLID-016",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "definition_points", kind: InputKind::Point, min: 2, max: None },
    ],
    parameters: &[
        Parameter { name: "Direction Constraint", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Base Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Solid", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_solid_017 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SOLID-017",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "definition_points", kind: InputKind::Point, min: 2, max: None },
    ],
    parameters: &[
        Parameter { name: "Direction Constraint", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Base Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Base Size Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Outer Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Inner Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "WallThickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Solid", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "BothSides", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_solid_018 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SOLID-018",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "definition_points", kind: InputKind::Point, min: 2, max: None },
    ],
    parameters: &[
        Parameter { name: "Numsides", kind: ParameterKind::Number, required: false },
        Parameter { name: "Base Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Star Inner Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Solid", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Direction Constraint", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_solid_019 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SOLID-019",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "definition_points", kind: InputKind::Point, min: 2, max: None },
    ],
    parameters: &[
        Parameter { name: "Direction Constraint", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Base Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Base Size Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Solid", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_solid_020 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SOLID-020",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "definition_points", kind: InputKind::Point, min: 2, max: None },
    ],
    parameters: &[
        Parameter { name: "Direction Constraint", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Base Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Base Size Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Base Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Size Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Top Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Solid", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_solid_021 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SOLID-021",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "profiles", kind: InputKind::Curve, min: 1, max: None },
        Role { name: "boundary", kind: InputKind::Surface, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "DraftAngle", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_solid_022 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SOLID-022",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "profiles", kind: InputKind::Curve, min: 1, max: None },
        Role { name: "boundary", kind: InputKind::Surface, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Offset", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Distance", kind: ParameterKind::Number, required: false },
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "DraftAngle", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_solid_023 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SOLID-023",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "profiles", kind: InputKind::Curve, min: 1, max: None },
        Role { name: "target", kind: InputKind::Surface, min: 1, max: Some(1) },
        Role { name: "path", kind: InputKind::Curve, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Direction", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Depth", kind: ParameterKind::Number, required: false },
        Parameter { name: "DeleteInput", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "BothSides", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_solid_024 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SOLID-024",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "curve", kind: InputKind::Curve, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Distance", kind: ParameterKind::Number, required: false },
        Parameter { name: "ThroughPoint", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "BothSides", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "InCPlane", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_solid_025 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SOLID-025",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "hole", kind: InputKind::Selection, min: 1, max: Some(1) },
        Role { name: "base", kind: InputKind::Point, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "ADirection", kind: ParameterKind::Reference, required: false },
        Parameter { name: "ANumber", kind: ParameterKind::Number, required: false },
        Parameter { name: "BNumber", kind: ParameterKind::Number, required: false },
        Parameter { name: "ASpacing", kind: ParameterKind::Number, required: false },
        Parameter { name: "BSpacing", kind: ParameterKind::Number, required: false },
        Parameter { name: "Rectangular", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "UseASpacing", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_solid_026 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SOLID-026",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "hole", kind: InputKind::Selection, min: 1, max: Some(1) },
        Role { name: "center", kind: InputKind::Point, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Number", kind: ParameterKind::Number, required: false },
        Parameter { name: "Angle", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_solid_027 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SOLID-027",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "holes", kind: InputKind::Selection, min: 1, max: None },
        Role { name: "move_points", kind: InputKind::Point, min: 2, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "Copy", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_solid_028 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SOLID-028",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "faces", kind: InputKind::Surface, min: 1, max: None },
        Role { name: "move_points", kind: InputKind::Point, min: 2, max: Some(2) },
        Role { name: "boundary", kind: InputKind::Surface, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Direction Constraint", kind: ParameterKind::Choice, required: false },
        Parameter { name: "ToBoundary", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Delete Boundary", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_solid_029 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SOLID-029",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "solid", kind: InputKind::Solid, min: 1, max: Some(1) },
        Role { name: "removed_faces", kind: InputKind::Surface, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Thickness", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_transform_001 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TRANSFORM-001",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "objects", kind: InputKind::Object, min: 1, max: None },
        Role { name: "origin", kind: InputKind::Point, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Scale Factor", kind: ParameterKind::Number, required: false },
        Parameter { name: "Copy", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_transform_002 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TRANSFORM-002",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "objects", kind: InputKind::Object, min: 1, max: None },
        Role { name: "origin", kind: InputKind::Point, min: 1, max: Some(1) },
        Role { name: "viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Scale Factor", kind: ParameterKind::Number, required: false },
        Parameter { name: "Copy", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_transform_003 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TRANSFORM-003",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "objects", kind: InputKind::Object, min: 1, max: None },
        Role { name: "origin", kind: InputKind::Point, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "X Scale Factor", kind: ParameterKind::Number, required: false },
        Parameter { name: "Y Scale Factor", kind: ParameterKind::Number, required: false },
        Parameter { name: "Z Scale Factor", kind: ParameterKind::Number, required: false },
        Parameter { name: "WorldCoordinates", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Copy", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_transform_004 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TRANSFORM-004",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "objects", kind: InputKind::Object, min: 1, max: None },
        Role { name: "origin", kind: InputKind::Point, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Scale Factor", kind: ParameterKind::Number, required: false },
        Parameter { name: "Copy", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_transform_005 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TRANSFORM-005",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "objects", kind: InputKind::Object, min: 1, max: None },
        Role { name: "reference_points", kind: InputKind::Point, min: 2, max: Some(2) },
        Role { name: "target_points", kind: InputKind::Point, min: 2, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "Copy", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Scale", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_transform_006 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TRANSFORM-006",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "objects", kind: InputKind::Object, min: 1, max: None },
        Role { name: "reference_points", kind: InputKind::Point, min: 3, max: Some(3) },
        Role { name: "target_points", kind: InputKind::Point, min: 3, max: Some(3) },
    ],
    parameters: &[
        Parameter { name: "Copy", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_transform_007 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TRANSFORM-007",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "objects", kind: InputKind::Object, min: 1, max: None },
        Role { name: "spine", kind: InputKind::Point, min: 2, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "Rigid", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "LimitToSpine", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Symmetric", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "PreserveStructure", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_transform_008 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TRANSFORM-008",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "objects", kind: InputKind::Object, min: 1, max: None },
        Role { name: "axis_points", kind: InputKind::Point, min: 2, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Copy", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_transform_009 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TRANSFORM-009",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "objects", kind: InputKind::Object, min: 1, max: None },
        Role { name: "center", kind: InputKind::Point, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Number", kind: ParameterKind::Number, required: false },
        Parameter { name: "Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "StepAngle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Rotate", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "ZOffset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Preview", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_transform_010 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TRANSFORM-010",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "objects", kind: InputKind::Object, min: 1, max: None },
        Role { name: "viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_transform_011 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TRANSFORM-011",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "base_surface", kind: InputKind::Surface, min: 0, max: Some(1) },
        Role { name: "destination_surface", kind: InputKind::Surface, min: 1, max: Some(1) },
        Role { name: "objects", kind: InputKind::Object, min: 0, max: None },
    ],
    parameters: &[
        Parameter { name: "Autobase", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Clear", kind: ParameterKind::Reference, required: false },
        Parameter { name: "BoundingBoxAddX", kind: ParameterKind::Number, required: false },
        Parameter { name: "BoundingBoxAddY", kind: ParameterKind::Number, required: false },
        Parameter { name: "Tolerance", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Around", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_transform_012 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TRANSFORM-012",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "flow_objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Rigid", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_transform_013 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TRANSFORM-013",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "objects", kind: InputKind::Object, min: 1, max: None },
        Role { name: "reference_points", kind: InputKind::Point, min: 2, max: Some(2) },
        Role { name: "surface", kind: InputKind::Surface, min: 1, max: Some(1) },
        Role { name: "target", kind: InputKind::Point, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Scale", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Rotation", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Rigid", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "OnSurface", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Copy", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Flip", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "IgnoreTrims", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_transform_014 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TRANSFORM-014",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "flow_surface", kind: InputKind::Surface, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_transform_015 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TRANSFORM-015",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "flow_surface", kind: InputKind::Surface, min: 1, max: Some(1) },
        Role { name: "curves", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_transform_016 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TRANSFORM-016",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "objects", kind: InputKind::Object, min: 1, max: None },
        Role { name: "source_base", kind: InputKind::Point, min: 1, max: Some(1) },
        Role { name: "curve", kind: InputKind::Curve, min: 1, max: Some(1) },
        Role { name: "target", kind: InputKind::Point, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Copy", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "XFlip", kind: ParameterKind::Reference, required: false },
        Parameter { name: "YFlip", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Rotate", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_transform_017 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TRANSFORM-017",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "objects", kind: InputKind::Object, min: 1, max: None },
        Role { name: "source_base", kind: InputKind::Point, min: 1, max: Some(1) },
        Role { name: "curve", kind: InputKind::Curve, min: 1, max: Some(1) },
        Role { name: "target", kind: InputKind::Point, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Copy", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Perpendicular", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_transform_018 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TRANSFORM-018",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "edge_or_end", kind: InputKind::Selection, min: 1, max: Some(1) },
        Role { name: "plane_points", kind: InputKind::Point, min: 2, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "Continuity", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_transform_019 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TRANSFORM-019",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_transform_020 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TRANSFORM-020",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "captives", kind: InputKind::Object, min: 1, max: None },
        Role { name: "control_object", kind: InputKind::Object, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Control Object", kind: ParameterKind::Choice, required: false },
        Parameter { name: "CoordinateSystem", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Degree", kind: ParameterKind::Number, required: false },
        Parameter { name: "PointCount", kind: ParameterKind::Number, required: false },
        Parameter { name: "UDegree", kind: ParameterKind::Number, required: false },
        Parameter { name: "VDegree", kind: ParameterKind::Number, required: false },
        Parameter { name: "UPointCount", kind: ParameterKind::Number, required: false },
        Parameter { name: "VPointCount", kind: ParameterKind::Number, required: false },
        Parameter { name: "XPointCount", kind: ParameterKind::Number, required: false },
        Parameter { name: "YPointCount", kind: ParameterKind::Number, required: false },
        Parameter { name: "ZPointCount", kind: ParameterKind::Number, required: false },
        Parameter { name: "XDegree", kind: ParameterKind::Number, required: false },
        Parameter { name: "YDegree", kind: ParameterKind::Number, required: false },
        Parameter { name: "ZDegree", kind: ParameterKind::Number, required: false },
        Parameter { name: "Deformation", kind: ParameterKind::Choice, required: false },
        Parameter { name: "PreserveStructure", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Region", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Falloff Distance", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_transform_021 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TRANSFORM-021",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "objects", kind: InputKind::Object, min: 1, max: None },
        Role { name: "path", kind: InputKind::Curve, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Basepoint", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Method", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Spacing Distance", kind: ParameterKind::Number, required: false },
        Parameter { name: "Number", kind: ParameterKind::Number, required: false },
        Parameter { name: "Orientation", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_transform_022 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TRANSFORM-022",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "captives", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_transform_023 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TRANSFORM-023",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "objects", kind: InputKind::Object, min: 1, max: None },
        Role { name: "basepoint", kind: InputKind::Point, min: 1, max: Some(1) },
        Role { name: "surface", kind: InputKind::Surface, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Normal", kind: ParameterKind::Reference, required: false },
        Parameter { name: "U Number", kind: ParameterKind::Number, required: false },
        Parameter { name: "V Number", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_transform_024 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TRANSFORM-024",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "objects", kind: InputKind::Object, min: 1, max: None },
        Role { name: "reference_points", kind: InputKind::Point, min: 2, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "Number", kind: ParameterKind::Number, required: false },
        Parameter { name: "Preview", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_transform_025 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TRANSFORM-025",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "XNumber", kind: ParameterKind::Number, required: false },
        Parameter { name: "YNumber", kind: ParameterKind::Number, required: false },
        Parameter { name: "ZNumber", kind: ParameterKind::Number, required: false },
        Parameter { name: "XSpacing", kind: ParameterKind::Number, required: false },
        Parameter { name: "YSpacing", kind: ParameterKind::Number, required: false },
        Parameter { name: "ZSpacing", kind: ParameterKind::Number, required: false },
        Parameter { name: "Preview", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_transform_026 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TRANSFORM-026",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "objects", kind: InputKind::Object, min: 1, max: None },
        Role { name: "basepoint", kind: InputKind::Point, min: 1, max: Some(1) },
        Role { name: "path", kind: InputKind::Curve, min: 1, max: Some(1) },
        Role { name: "surface", kind: InputKind::Surface, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Divide Number", kind: ParameterKind::Number, required: false },
        Parameter { name: "Multiple Distance", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_transform_027 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TRANSFORM-027",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "objects", kind: InputKind::Object, min: 1, max: None },
        Role { name: "base_curve", kind: InputKind::Curve, min: 1, max: Some(1) },
        Role { name: "target_curve", kind: InputKind::Curve, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Copy", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Rigid", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Line", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Local", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Stretch", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_transform_028 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TRANSFORM-028",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "objects", kind: InputKind::Object, min: 1, max: None },
        Role { name: "base_surface", kind: InputKind::Surface, min: 1, max: Some(1) },
        Role { name: "target_surface", kind: InputKind::Surface, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Copy", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Rigid", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Plane", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_transform_029 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TRANSFORM-029",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "objects", kind: InputKind::Object, min: 1, max: None },
        Role { name: "surface", kind: InputKind::Surface, min: 1, max: Some(1) },
        Role { name: "drop_point", kind: InputKind::Point, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Reference Sphere", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Target Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Rotation Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Copy", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Rigid", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Flip", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_transform_030 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TRANSFORM-030",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "points", kind: InputKind::Selection, min: 1, max: None },
        Role { name: "target", kind: InputKind::Point, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Set X", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Set Y", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Set Z", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Align to", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_transform_031 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TRANSFORM-031",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "objects", kind: InputKind::Object, min: 1, max: None },
        Role { name: "origin", kind: InputKind::Point, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Copy", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Plane", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Plane Scale Factors", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_transform_032 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TRANSFORM-032",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "objects", kind: InputKind::Object, min: 1, max: None },
        Role { name: "axis_points", kind: InputKind::Point, min: 2, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "Copy", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Rigid", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Stretch Factor", kind: ParameterKind::Number, required: false },
        Parameter { name: "Length", kind: ParameterKind::Number, required: false },
        Parameter { name: "Axis Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Pivot Angle", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_transform_033 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TRANSFORM-033",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "objects", kind: InputKind::Object, min: 1, max: None },
        Role { name: "axis_points", kind: InputKind::Point, min: 2, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "Copy", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Rigid", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Start Distance", kind: ParameterKind::Number, required: false },
        Parameter { name: "End Distance", kind: ParameterKind::Number, required: false },
        Parameter { name: "Flat", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Infinite", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "PreserveStructure", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Axis Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Pivot Angle", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_transform_034 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TRANSFORM-034",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "objects", kind: InputKind::Object, min: 1, max: None },
        Role { name: "origin", kind: InputKind::Point, min: 1, max: Some(1) },
        Role { name: "reference", kind: InputKind::Point, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Copy", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_transform_035 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TRANSFORM-035",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "objects", kind: InputKind::Object, min: 1, max: None },
        Role { name: "center", kind: InputKind::Point, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "First Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Second Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Coil Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Circle Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Copy", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Rigid", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_transform_036 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TRANSFORM-036",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "objects", kind: InputKind::Object, min: 1, max: None },
        Role { name: "axis_points", kind: InputKind::Point, min: 2, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "Twist Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Copy", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Rigid", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Infinite", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_transform_037 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TRANSFORM-037",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "object", kind: InputKind::Object, min: 1, max: Some(1) },
        Role { name: "flat_face", kind: InputKind::Surface, min: 1, max: Some(1) },
        Role { name: "viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_transform_038 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TRANSFORM-038",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "objects", kind: InputKind::Object, min: 1, max: None },
        Role { name: "target_viewport", kind: InputKind::Viewport, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Copy", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "CPlane", kind: ParameterKind::Text, required: false },
        Parameter { name: "View", kind: ParameterKind::Text, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_transform_039 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TRANSFORM-039",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "control_points", kind: InputKind::Selection, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "U", kind: ParameterKind::Number, required: false },
        Parameter { name: "V", kind: ParameterKind::Number, required: false },
        Parameter { name: "N", kind: ParameterKind::Number, required: false },
        Parameter { name: "Scale", kind: ParameterKind::Number, required: false },
        Parameter { name: "UV Move Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "U-Symmetry", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "V-Symmetry", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Smoothing", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_transform_040 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TRANSFORM-040",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "objects", kind: InputKind::Object, min: 1, max: None },
        Role { name: "influence_source", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Object Type", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Falloff Distance", kind: ParameterKind::Number, required: false },
        Parameter { name: "Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Anchor", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Vertical", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Copy", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "LinkAll", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_transform_041 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TRANSFORM-041",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "definition_points", kind: InputKind::Point, min: 0, max: None },
        Role { name: "targets_for_bbox", kind: InputKind::Object, min: 0, max: None },
    ],
    parameters: &[
        Parameter { name: "BoundingBox", kind: ParameterKind::Reference, required: false },
        Parameter { name: "CoordinateSystem", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Base Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "XPointCount", kind: ParameterKind::Number, required: false },
        Parameter { name: "YPointCount", kind: ParameterKind::Number, required: false },
        Parameter { name: "ZPointCount", kind: ParameterKind::Number, required: false },
        Parameter { name: "XDegree", kind: ParameterKind::Number, required: false },
        Parameter { name: "YDegree", kind: ParameterKind::Number, required: false },
        Parameter { name: "ZDegree", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_001 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-001",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Enabled", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Soft Manipulate", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Set Soft Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Paint", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Hotkeys", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Selected", kind: ParameterKind::Text, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_002 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-002",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "input", kind: InputKind::Object, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Copy", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "OutputType", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Drag Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Rebuild when increasing degree", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_003 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-003",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "control_model", kind: InputKind::Object, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Copy", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Output Type", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Star Continuity", kind: ParameterKind::Choice, required: false },
        Parameter { name: "History Enabled", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_004 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-004",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "seed_edge", kind: InputKind::Selection, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_005 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-005",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "seed_edge", kind: InputKind::Selection, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_006 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-006",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "control_model", kind: InputKind::Object, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Operation", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Symmetry Type", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Weld", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Tolerance", kind: ParameterKind::Choice, required: false },
        Parameter { name: "3Point", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Segments", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_007 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-007",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "control_model", kind: InputKind::Object, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Isolate", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_008 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-008",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "control_model", kind: InputKind::Object, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_009 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-009",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "control_model", kind: InputKind::Object, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_010 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-010",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "control_model", kind: InputKind::Object, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_011 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-011",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "control_model", kind: InputKind::Object, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_012 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-012",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Speed Mode", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_013 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-013",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "components", kind: InputKind::Selection, min: 0, max: None },
    ],
    parameters: &[
        Parameter { name: "Drag Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Next", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_014 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-014",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "components", kind: InputKind::Selection, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Scale Factor", kind: ParameterKind::Number, required: false },
        Parameter { name: "Handle", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Style", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_015 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-015",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "components", kind: InputKind::Selection, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Translation Amount", kind: ParameterKind::Number, required: false },
        Parameter { name: "Handle", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Style", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_016 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-016",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "components", kind: InputKind::Selection, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Rotation Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Handle", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Style", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_017 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-017",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "pivot", kind: InputKind::Point, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_018 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-018",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_019 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-019",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_020 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-020",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_021 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-021",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "inputs", kind: InputKind::Selection, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Selection Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Transform", kind: ParameterKind::Reference, required: false },
        Parameter { name: "RetopoSnap", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "RetopoOffset", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_022 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-022",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_023 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-023",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "edge_loop_or_ring", kind: InputKind::Selection, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Insertion Type", kind: ParameterKind::Choice, required: false },
        Parameter { name: "RetopoSnap", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "RetopoOffset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Both Sides", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Percentage", kind: ParameterKind::Number, required: false },
        Parameter { name: "Distance", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_024 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-024",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "edge_loop_or_ring", kind: InputKind::Selection, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Insertion Type", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Both Sides", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_025 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-025",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "edges", kind: InputKind::Selection, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Insertion Type", kind: ParameterKind::Choice, required: false },
        Parameter { name: "RetopoSnap", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "RetopoOffset", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_026 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-026",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "edges", kind: InputKind::Selection, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Insertion Type", kind: ParameterKind::Choice, required: false },
        Parameter { name: "RetopoSnap", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "RetopoOffset", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_027 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-027",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "control_model", kind: InputKind::Object, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Direction Type", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Crease Edges", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Copy", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_028 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-028",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "boundary_edge", kind: InputKind::Selection, min: 0, max: Some(1) },
        Role { name: "corners", kind: InputKind::Point, min: 0, max: None },
    ],
    parameters: &[
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "AutoQuad", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "RetopoSnap", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "RetopoOffset", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_029 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-029",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "first_region", kind: InputKind::Selection, min: 1, max: None },
        Role { name: "second_region", kind: InputKind::Selection, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Selection Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Segments", kind: ParameterKind::Number, required: false },
        Parameter { name: "ShowPreview", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Follow Curve", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Rotation", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Twist", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_030 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-030",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "curve_network", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "CurveSplitting", kind: ParameterKind::Number, required: false },
        Parameter { name: "Segments", kind: ParameterKind::Number, required: false },
        Parameter { name: "AddHandle", kind: ParameterKind::Reference, required: false },
        Parameter { name: "RemoveHandle", kind: ParameterKind::Reference, required: false },
        Parameter { name: "SetMultiple", kind: ParameterKind::Reference, required: false },
        Parameter { name: "ClearMultiple", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Preview", kind: ParameterKind::Choice, required: false },
        Parameter { name: "OutputType", kind: ParameterKind::Choice, required: false },
        Parameter { name: "CurrJointSet", kind: ParameterKind::Choice, required: false },
        Parameter { name: "DefromJoints", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "AlwaysIncludeAutojoint", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "ExportJointSet", kind: ParameterKind::Reference, required: false },
        Parameter { name: "ImportJointSet", kind: ParameterKind::Reference, required: false },
        Parameter { name: "AddJointSet", kind: ParameterKind::Reference, required: false },
        Parameter { name: "DeleteJointSet", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_031 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-031",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "holes_or_model", kind: InputKind::Selection, min: 1, max: None },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_032 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-032",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "components", kind: InputKind::Selection, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Insertion Type", kind: ParameterKind::Choice, required: false },
        Parameter { name: "RetopoSnap", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "RetopoOffset", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_033 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-033",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "edges", kind: InputKind::Selection, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Tangency Handles", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_034 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-034",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "components", kind: InputKind::Selection, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Insertion Type", kind: ParameterKind::Choice, required: false },
        Parameter { name: "RetopoSnap", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "RetopoOffset", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_035 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-035",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "first_edges", kind: InputKind::Selection, min: 1, max: None },
        Role { name: "second_edges", kind: InputKind::Selection, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Smooth", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_036 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-036",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "creased_edges", kind: InputKind::Selection, min: 1, max: None },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_037 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-037",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "vertices", kind: InputKind::Selection, min: 1, max: None },
        Role { name: "target", kind: InputKind::Point, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Set X", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Set Y", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Set Z", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Align to", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Copy", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_038 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-038",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "edges", kind: InputKind::Selection, min: 1, max: None },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_039 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-039",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "vertices", kind: InputKind::Selection, min: 2, max: None },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_040 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-040",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "control_model", kind: InputKind::Object, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_041 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-041",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "definition_points", kind: InputKind::Point, min: 2, max: None },
    ],
    parameters: &[
        Parameter { name: "OutputType", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Base Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "XFaces", kind: ParameterKind::Number, required: false },
        Parameter { name: "YFaces", kind: ParameterKind::Number, required: false },
        Parameter { name: "ZFaces", kind: ParameterKind::Number, required: false },
        Parameter { name: "AxialSymmetry", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "XSymmetry", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "YSymmetry", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "ZSymmetry", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_042 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-042",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Primitive", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_043 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-043",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "definition_points", kind: InputKind::Point, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Size Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Diameter", kind: ParameterKind::Number, required: false },
        Parameter { name: "Circumference", kind: ParameterKind::Number, required: false },
        Parameter { name: "Area", kind: ParameterKind::Number, required: false },
        Parameter { name: "Orientation", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Circle Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "VerticalFaces", kind: ParameterKind::Number, required: false },
        Parameter { name: "AroundFaces", kind: ParameterKind::Number, required: false },
        Parameter { name: "OutputType", kind: ParameterKind::Choice, required: false },
        Parameter { name: "AxialSymmetry", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "XSymmetry", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "YSymmetry", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "ZSymmetry", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Symmetry", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Radial Symmetry", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "SymmetrySegments", kind: ParameterKind::Number, required: false },
        Parameter { name: "FacesPerSegment", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_044 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-044",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "definition_points", kind: InputKind::Point, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Size Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Diameter", kind: ParameterKind::Number, required: false },
        Parameter { name: "Circumference", kind: ParameterKind::Number, required: false },
        Parameter { name: "Area", kind: ParameterKind::Number, required: false },
        Parameter { name: "Orientation", kind: ParameterKind::Choice, required: false },
        Parameter { name: "NumEdgeSegments", kind: ParameterKind::Number, required: false },
        Parameter { name: "Circle Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "OutputType", kind: ParameterKind::Choice, required: false },
        Parameter { name: "AxialSymmetry", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "XSymmetry", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "YSymmetry", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "ZSymmetry", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_045 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-045",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "definition_points", kind: InputKind::Point, min: 2, max: None },
    ],
    parameters: &[
        Parameter { name: "Size Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Diameter", kind: ParameterKind::Number, required: false },
        Parameter { name: "Circumference", kind: ParameterKind::Number, required: false },
        Parameter { name: "Area", kind: ParameterKind::Number, required: false },
        Parameter { name: "Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Circle Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "DirectionConstraint", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Solid", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "VerticalFaces", kind: ParameterKind::Number, required: false },
        Parameter { name: "AroundFaces", kind: ParameterKind::Number, required: false },
        Parameter { name: "OutputType", kind: ParameterKind::Choice, required: false },
        Parameter { name: "AxialSymmetry", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "XSymmetry", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "YSymmetry", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "ZSymmetry", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Symmetry", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Radial Symmetry", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "SymmetrySegments", kind: ParameterKind::Number, required: false },
        Parameter { name: "FacesPerSegment", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_046 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-046",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "definition_points", kind: InputKind::Point, min: 2, max: None },
    ],
    parameters: &[
        Parameter { name: "Size Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Diameter", kind: ParameterKind::Number, required: false },
        Parameter { name: "Circumference", kind: ParameterKind::Number, required: false },
        Parameter { name: "Area", kind: ParameterKind::Number, required: false },
        Parameter { name: "Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Circle Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "DirectionConstraint", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Solid", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "VerticalFaces", kind: ParameterKind::Number, required: false },
        Parameter { name: "AroundFaces", kind: ParameterKind::Number, required: false },
        Parameter { name: "OutputType", kind: ParameterKind::Choice, required: false },
        Parameter { name: "AxialSymmetry", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "XSymmetry", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "YSymmetry", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "ZSymmetry", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Symmetry", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Radial Symmetry", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "SymmetrySegments", kind: ParameterKind::Number, required: false },
        Parameter { name: "FacesPerSegment", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_047 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-047",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "definition_points", kind: InputKind::Point, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Size Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Diameter", kind: ParameterKind::Number, required: false },
        Parameter { name: "Circumference", kind: ParameterKind::Number, required: false },
        Parameter { name: "Area", kind: ParameterKind::Number, required: false },
        Parameter { name: "Second Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Circle Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "VerticalFaces", kind: ParameterKind::Number, required: false },
        Parameter { name: "AroundFaces", kind: ParameterKind::Number, required: false },
        Parameter { name: "OutputType", kind: ParameterKind::Choice, required: false },
        Parameter { name: "AxialSymmetry", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "XSymmetry", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "YSymmetry", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "ZSymmetry", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Symmetry", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Radial Symmetry", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "SymmetrySegments", kind: ParameterKind::Number, required: false },
        Parameter { name: "FacesPerSegment", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_048 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-048",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "definition_points", kind: InputKind::Point, min: 2, max: Some(3) },
    ],
    parameters: &[
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "XFaces", kind: ParameterKind::Number, required: false },
        Parameter { name: "YFaces", kind: ParameterKind::Number, required: false },
        Parameter { name: "OutputType", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Axial Symmetry", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "XSymmetry", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "YSymmetry", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_049 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-049",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "rails", kind: InputKind::Curve, min: 2, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "SlashCount", kind: ParameterKind::Number, required: false },
        Parameter { name: "Add Slash", kind: ParameterKind::Reference, required: false },
        Parameter { name: "DeleteSlash", kind: ParameterKind::Reference, required: false },
        Parameter { name: "AddHeight", kind: ParameterKind::Reference, required: false },
        Parameter { name: "DeleteHeight", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Mirror", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Control Points", kind: ParameterKind::Choice, required: false },
        Parameter { name: "CurveClosed", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "EndCaps", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "BackClosed", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_050 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-050",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "gem", kind: InputKind::Gem, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Crease Top", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Crease Seat", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Crease Bottom", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Faces", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Seat Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Seat Length", kind: ParameterKind::Number, required: false },
        Parameter { name: "Seat Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bezel Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Placement", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bezel Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Base Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Dome Height", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_051 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-051",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "gem", kind: InputKind::Gem, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Crease Top", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Crease Seat", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Crease Bottom", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Faces", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Seat Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Seat Length", kind: ParameterKind::Number, required: false },
        Parameter { name: "Seat Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bezel Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Placement", kind: ParameterKind::Number, required: false },
        Parameter { name: "Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Thickness", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_052 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-052",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "ring_rail", kind: InputKind::Curve, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Ring Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Shank Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Depth", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Width Corner", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Depth Corner", kind: ParameterKind::Number, required: false },
        Parameter { name: "Middle Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bottom Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Shank Thickness", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_053 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-053",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "curve_network", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Use File Tolerance", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Tolerance Value", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Display", kind: ParameterKind::Reference, required: false },
        Parameter { name: "MaxAutoFace", kind: ParameterKind::Number, required: false },
        Parameter { name: "MaxManualFace", kind: ParameterKind::Number, required: false },
        Parameter { name: "Face Layout", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Add Creases", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Mark T-Points", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Spans", kind: ParameterKind::Number, required: false },
        Parameter { name: "Chord Length", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Preview", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_054 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-054",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "sections", kind: InputKind::Curve, min: 2, max: None },
    ],
    parameters: &[
        Parameter { name: "Loft Style", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Closed Loft", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Match Start Tangency", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Match End Tangency", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Loft Exactness", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Align Curves", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Preview", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_055 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-055",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "components", kind: InputKind::Selection, min: 1, max: None },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_056 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-056",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "source_edges", kind: InputKind::Selection, min: 1, max: None },
        Role { name: "target", kind: InputKind::Object, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Alignment Add/Delete", kind: ParameterKind::Reference, required: false },
        Parameter { name: "AlignmentType", kind: ParameterKind::Choice, required: false },
        Parameter { name: "FlipAlignment Direction", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Continuity", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Tangent Scale", kind: ParameterKind::Number, required: false },
        Parameter { name: "UseFalloff", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Falloff Distance", kind: ParameterKind::Number, required: false },
        Parameter { name: "Use Refinement", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Preview", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_057 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-057",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "source_points", kind: InputKind::Selection, min: 1, max: None },
        Role { name: "target", kind: InputKind::Object, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "PullType", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_058 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-058",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "components", kind: InputKind::Selection, min: 0, max: None },
    ],
    parameters: &[
        Parameter { name: "Record", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Play", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Delete", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Name", kind: ParameterKind::Text, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_059 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-059",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "components", kind: InputKind::Selection, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Weight", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_060 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-060",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "control_model", kind: InputKind::Object, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "ShowTPoints", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "ShowLPoints", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "ShowStars", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "ShowErrorStars", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "ShowSeashells", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "AutoRepair", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_061 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-061",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "model_asset", kind: InputKind::Path, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Thumbnail Size", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_062 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-062",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "edges", kind: InputKind::Selection, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Segments", kind: ParameterKind::Number, required: false },
        Parameter { name: "Positioning", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Percentage", kind: ParameterKind::Number, required: false },
        Parameter { name: "Distance", kind: ParameterKind::Number, required: false },
        Parameter { name: "KeepOnFace", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Roundness", kind: ParameterKind::Number, required: false },
        Parameter { name: "RetopoSnap", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "RetopoOffset", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_063 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-063",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "control_model", kind: InputKind::Object, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_064 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-064",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "edges", kind: InputKind::Selection, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Positioning", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Percentage", kind: ParameterKind::Number, required: false },
        Parameter { name: "Distance", kind: ParameterKind::Number, required: false },
        Parameter { name: "Roundness", kind: ParameterKind::Number, required: false },
        Parameter { name: "RetopoSnap", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "RetopoOffset", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_065 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-065",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "control_points", kind: InputKind::Selection, min: 4, max: None },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_066 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-066",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "control_model", kind: InputKind::Object, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_067 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-067",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "edges", kind: InputKind::Selection, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "BothSides", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_068 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-068",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "RetopologySnap", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Snapin", kind: ParameterKind::Choice, required: false },
        Parameter { name: "RetopoOffset", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_069 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-069",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "curves", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Use File Tolerance", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Tolerance Value", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_070 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-070",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "control_model", kind: InputKind::Object, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Operation", kind: ParameterKind::Choice, required: false },
        Parameter { name: "FromExisting", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Texture Mesh Method", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Texture Rectangle", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Pinned Points", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_071 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-071",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "surface_or_control_model", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "UseTolerance", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Tolerance", kind: ParameterKind::Choice, required: false },
        Parameter { name: "DivisionsPerFace", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_072 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-072",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "line_network", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "LineSnapping", kind: ParameterKind::Reference, required: false },
        Parameter { name: "UseFileTolerance", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Tolerance", kind: ParameterKind::Choice, required: false },
        Parameter { name: "MaxAutoFace", kind: ParameterKind::Number, required: false },
        Parameter { name: "MaxManualFace", kind: ParameterKind::Number, required: false },
        Parameter { name: "Smooth", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_073 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-073",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "curves", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Distance", kind: ParameterKind::Number, required: false },
        Parameter { name: "Direction", kind: ParameterKind::Reference, required: false },
        Parameter { name: "BothSides", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Solid", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "DeleteInput", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "ToBoundary", kind: ParameterKind::Reference, required: false },
        Parameter { name: "SplitAtTangents", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "SetBasePoint", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_074 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-074",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "components", kind: InputKind::Selection, min: 1, max: None },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_075 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-075",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "components", kind: InputKind::Selection, min: 1, max: None },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_076 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-076",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Settings Category", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Hotkeys", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Rebuild when increasing degree", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Display Settings", kind: ParameterKind::Reference, required: false },
        Parameter { name: "UI Startup", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_077 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-077",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "control_model", kind: InputKind::Object, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tspline_078 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-078",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "control_model", kind: InputKind::Object, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_art_001 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-ART-001",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Curves", kind: InputKind::Curve, min: 1, max: None },
        Role { name: "Bitmap", kind: InputKind::Image, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Resolution", kind: ParameterKind::Number, required: false },
        Parameter { name: "Capped", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Drop Depth", kind: ParameterKind::Number, required: false },
        Parameter { name: "Colors View", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Ghosted View", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Preview View", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_art_002 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-ART-002",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Curves", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Surface Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Profile", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Layer Relation", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_art_003 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-ART-003",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Bitmap", kind: InputKind::Image, min: 1, max: Some(1) },
        Role { name: "Region", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Bitmap Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Size Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Size Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Offset X", kind: ParameterKind::Number, required: false },
        Parameter { name: "Offset Y", kind: ParameterKind::Number, required: false },
        Parameter { name: "Rotate", kind: ParameterKind::Number, required: false },
        Parameter { name: "Maintain Aspect", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Tile Bitmap", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Invert Bitmap", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Trim to Bitmap", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Add Depth", kind: ParameterKind::Number, required: false },
        Parameter { name: "Fill Shape", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Current Layer", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_art_004 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-ART-004",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Bitmap", kind: InputKind::Image, min: 0, max: Some(1) },
        Role { name: "Curves", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_art_005 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-ART-005",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Bitmap", kind: InputKind::Image, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Diameter", kind: ParameterKind::Number, required: false },
        Parameter { name: "Strength", kind: ParameterKind::Number, required: false },
        Parameter { name: "Fall Off", kind: ParameterKind::Number, required: false },
        Parameter { name: "Adjust Contrast", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Blur Amount", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Brightness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Contrast", kind: ParameterKind::Number, required: false },
        Parameter { name: "Blur size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bitmap Invert", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_art_006 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-ART-006",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Relief", kind: InputKind::Mesh, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Diameter", kind: ParameterKind::Number, required: false },
        Parameter { name: "Strength", kind: ParameterKind::Number, required: false },
        Parameter { name: "Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Fall Off", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_art_007 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-ART-007",
    effect: Effect::ExternalOutput,
    roles: &[
        Role { name: "Viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
        Role { name: "Destination", kind: InputKind::Path, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_builder_001 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-BUILDER-001",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "RingRail", kind: InputKind::Curve, min: 1, max: Some(1) },
        Role { name: "Profiles", kind: InputKind::Curve, min: 0, max: None },
        Role { name: "Gems", kind: InputKind::Gem, min: 0, max: None },
    ],
    parameters: &[
        Parameter { name: "Ring Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Channel Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Culet to Finger", kind: ParameterKind::Number, required: false },
        Parameter { name: "Gem Spacing", kind: ParameterKind::Number, required: false },
        Parameter { name: "Draw Band", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Styles", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Channel Depth", kind: ParameterKind::Number, required: false },
        Parameter { name: "Extra Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Edge Profile", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Gem List", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_builder_002 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-BUILDER-002",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "RingRail", kind: InputKind::Curve, min: 1, max: Some(1) },
        Role { name: "TopProfile", kind: InputKind::Curve, min: 0, max: Some(1) },
        Role { name: "EdgeProfile", kind: InputKind::Curve, min: 0, max: Some(1) },
        Role { name: "MiddleProfile", kind: InputKind::Curve, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "SignetType", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Top Profile", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Edge Profile", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Middle Profile", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Top Length", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Shank Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bottom Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Cut Bottom", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Bottom Width Scale", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bottom Length Scale", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bottom Cut Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Middle Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Styles", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_builder_003 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-BUILDER-003",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "RingRail", kind: InputKind::Curve, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Shank Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Depth", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Width Corner", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Depth Corner", kind: ParameterKind::Number, required: false },
        Parameter { name: "Middle Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bottom Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Shank Thickness", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_builder_004 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-BUILDER-004",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "RingRail", kind: InputKind::Curve, min: 1, max: Some(1) },
        Role { name: "Artwork", kind: InputKind::Curve, min: 0, max: None },
        Role { name: "Target", kind: InputKind::Surface, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Shank Profile", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Text Profile", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Text Field", kind: ParameterKind::Text, required: false },
        Parameter { name: "Font", kind: ParameterKind::Text, required: false },
        Parameter { name: "Font Style", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Text Curves", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Raised Band Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Raised Band Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Text Edge Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Text Raised Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Pick Surface", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Center Text", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Show Guide", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_builder_005 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-BUILDER-005",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Signet", kind: InputKind::Solid, min: 1, max: Some(1) },
        Role { name: "Artwork", kind: InputKind::Curve, min: 0, max: None },
    ],
    parameters: &[
        Parameter { name: "Outer Rail", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Inner Rail", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Outer X Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Outer Y Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Outer Z Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Z Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Inner X Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Inner Y Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Inner Z Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Base Profile", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Starburst Count", kind: ParameterKind::Number, required: false },
        Parameter { name: "Inner Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Outer Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bezel", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Bezel Shape", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Bezel Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bezel Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Flutes", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Flute Shape", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Flute Count", kind: ParameterKind::Number, required: false },
        Parameter { name: "Flute Z Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Flute X Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Flute Z Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Flute Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Flute Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Text Profile", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Text Field", kind: ParameterKind::Text, required: false },
        Parameter { name: "Font", kind: ParameterKind::Text, required: false },
        Parameter { name: "Font Style", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Artwork", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Orientation", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Mirror Angle", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Text Angle Mirrored", kind: ParameterKind::Number, required: false },
        Parameter { name: "Text Starting Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Text Ending Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Text Top Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Text Bottom Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Text Left Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Text Right Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Text Height", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_builder_006 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-BUILDER-006",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Pattern", kind: InputKind::Selection, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Knot Size X", kind: ParameterKind::Number, required: false },
        Parameter { name: "Knot Size Y", kind: ParameterKind::Number, required: false },
        Parameter { name: "Knot Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Cap Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Knot Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Corner Scale", kind: ParameterKind::Number, required: false },
        Parameter { name: "Cross Over Knot Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Cross Over Cap Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Cross Over Pass Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Corner Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Nurbs Profile", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Center Knot", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Pattern", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_builder_007 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-BUILDER-007",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Curves", kind: InputKind::Curve, min: 2, max: None },
    ],
    parameters: &[
        Parameter { name: "Input Curves", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Profile Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Profile Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Profile Spacing", kind: ParameterKind::Number, required: false },
        Parameter { name: "Profile Height at Knot", kind: ParameterKind::Number, required: false },
        Parameter { name: "Center Height at Knot", kind: ParameterKind::Number, required: false },
        Parameter { name: "Nurbs Profile", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Keep Curve", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_builder_008 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-BUILDER-008",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "RailProfile", kind: InputKind::Curve, min: 1, max: Some(1) },
        Role { name: "ShapeProfile", kind: InputKind::Curve, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Rail Profile", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Shape Profile", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Rail Profile Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Rail Profile Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Shape Profile Depth", kind: ParameterKind::Number, required: false },
        Parameter { name: "Shape Profile Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Shape Outside/Shape Inside", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_builder_009 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-BUILDER-009",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "PatternA", kind: InputKind::Object, min: 1, max: None },
        Role { name: "PatternB", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Pattern", kind: ParameterKind::Choice, required: false },
        Parameter { name: "X Repeat", kind: ParameterKind::Number, required: false },
        Parameter { name: "Y Repeat", kind: ParameterKind::Number, required: false },
        Parameter { name: "X Scale", kind: ParameterKind::Number, required: false },
        Parameter { name: "Y Scale", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_builder_010 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-BUILDER-010",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Closed", kind: InputKind::Curve, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Amount", kind: ParameterKind::Number, required: false },
        Parameter { name: "Length", kind: ParameterKind::Number, required: false },
        Parameter { name: "Rotation", kind: ParameterKind::Number, required: false },
        Parameter { name: "Flip", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tools_001 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-001",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Document", kind: InputKind::Document, min: 1, max: Some(1) },
        Role { name: "CustomRail", kind: InputKind::Curve, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Region Type", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Custom Ring Size Units", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Custom Ring Size Measure", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Custom Ring Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Custom Rail", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tools_002 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-002",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Rail", kind: InputKind::Curve, min: 1, max: Some(1) },
        Role { name: "Profile", kind: InputKind::Curve, min: 0, max: Some(1) },
        Role { name: "SecondRail", kind: InputKind::Curve, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Edge Profile", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Tilt", kind: ParameterKind::Number, required: false },
        Parameter { name: "Twist", kind: ParameterKind::Number, required: false },
        Parameter { name: "Roll", kind: ParameterKind::Number, required: false },
        Parameter { name: "X Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Y Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Z Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Position On Curve", kind: ParameterKind::Number, required: false },
        Parameter { name: "Mirror Rotate", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Keep Location", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Orient Type", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Second Rail", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Name", kind: ParameterKind::Text, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tools_003 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-003",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "RingRail", kind: InputKind::Curve, min: 1, max: Some(1) },
        Role { name: "RailProfile", kind: InputKind::Curve, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Rail Profile", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Top Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Side Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bottom Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Lock", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tools_004 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-004",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "RingRail", kind: InputKind::Curve, min: 1, max: Some(1) },
        Role { name: "Shape", kind: InputKind::Curve, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Delete Curves", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tools_005 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-005",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Rails", kind: InputKind::Curve, min: 4, max: Some(4) },
        Role { name: "Profile", kind: InputKind::Curve, min: 0, max: Some(1) },
        Role { name: "PlanePoints", kind: InputKind::Point, min: 2, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "Profile", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Rotate Z", kind: ParameterKind::Number, required: false },
        Parameter { name: "Rotate Y", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tools_006 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-006",
    effect: Effect::ReadOnly,
    roles: &[
        Role { name: "Solids", kind: InputKind::Solid, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "GV", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "User", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Use Object Materials", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Material", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tools_007 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-007",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Metal", kind: InputKind::Solid, min: 1, max: None },
        Role { name: "RingRail", kind: InputKind::Curve, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Primary radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Left angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Right angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Sizes", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Symmetrical", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Add Rings", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Save to File", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "STL Options", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tools_008 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-008",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Surface", kind: InputKind::Surface, min: 1, max: Some(1) },
        Role { name: "Boundary", kind: InputKind::Curve, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Drop Depth", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bevel Edge", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Extrude Distance", kind: ParameterKind::Number, required: false },
        Parameter { name: "Insert Only", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Cap", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tools_009 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-009",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Categories", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Thumbnail Slider", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tools_010 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-010",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Categories", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Quality", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Size", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Thumbnail Slider", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tools_011 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-011",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "BlendPoint", kind: InputKind::Point, min: 1, max: Some(1) },
        Role { name: "DirectionPoint", kind: InputKind::Point, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tools_012 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-012",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Targets", kind: InputKind::Object, min: 2, max: Some(2) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tools_013 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-013",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "BlendPoint", kind: InputKind::Object, min: 1, max: Some(1) },
        Role { name: "DirectionPoint", kind: InputKind::Point, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tools_014 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-014",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Curve", kind: InputKind::Curve, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tools_015 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-015",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "Targets", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Smart Target Filter", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tools_016 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-016",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Image", kind: InputKind::Image, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Threshold", kind: ParameterKind::Number, required: false },
        Parameter { name: "Despeckle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Corner Soften", kind: ParameterKind::Number, required: false },
        Parameter { name: "Blur", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Blur Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Outline", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Outline Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Preprocess Method", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Prescale", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Invert", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Group Output", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tools_017 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-017",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Parents", kind: InputKind::Curve, min: 2, max: None },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tools_018 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-018",
    effect: Effect::ReadOnly,
    roles: &[
        Role { name: "Objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tools_019 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-019",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "Document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tools_020 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-020",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Extended Repair", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "DeleteInputObjects", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Mesh Tolerance", kind: ParameterKind::Number, required: false },
        Parameter { name: "Simple Planes", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Optimize Milgrain", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Milgrain Grid Count", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tools_021 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-021",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "Objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Degree", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tools_022 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-022",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Surface", kind: InputKind::Surface, min: 1, max: Some(1) },
        Role { name: "Location", kind: InputKind::Point, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Text", kind: ParameterKind::Text, required: false },
        Parameter { name: "FontSize", kind: ParameterKind::Number, required: false },
        Parameter { name: "Thickness", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tools_023 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-023",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "RingRail", kind: InputKind::Curve, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Text", kind: ParameterKind::Text, required: false },
        Parameter { name: "FontSize", kind: ParameterKind::Number, required: false },
        Parameter { name: "Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "SpanDegree", kind: ParameterKind::Number, required: false },
        Parameter { name: "PlacementStartAngle", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tools_024 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-024",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Ring", kind: InputKind::Object, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "TopLength", kind: ParameterKind::Number, required: false },
        Parameter { name: "TopWidth", kind: ParameterKind::Number, required: false },
        Parameter { name: "BottomLength", kind: ParameterKind::Number, required: false },
        Parameter { name: "BottomWidth", kind: ParameterKind::Number, required: false },
        Parameter { name: "Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "BaseShape", kind: ParameterKind::Choice, required: false },
        Parameter { name: "BaseThickness", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tools_025 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-025",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Model", kind: InputKind::Object, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Shape", kind: ParameterKind::Choice, required: false },
        Parameter { name: "BaseThickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Offset", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tools_026 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-026",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Anchor", kind: InputKind::Point, min: 1, max: Some(1) },
        Role { name: "Tips", kind: InputKind::Point, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Mirror X", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Mirror Y", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "End Tip Diameter", kind: ParameterKind::Number, required: false },
        Parameter { name: "Arc Amount", kind: ParameterKind::Number, required: false },
        Parameter { name: "Sprue Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Sprue Diameter", kind: ParameterKind::Number, required: false },
        Parameter { name: "Sprue Start X Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Sprue Start Y Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Sprue Mid Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Extender", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tools_027 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-027",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Points", kind: InputKind::Point, min: 2, max: None },
    ],
    parameters: &[
        Parameter { name: "Mirror X", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Mirror Y", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Chain Supports", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "ArmLength", kind: ParameterKind::Number, required: false },
        Parameter { name: "BlendTension", kind: ParameterKind::Number, required: false },
        Parameter { name: "StartDiameter", kind: ParameterKind::Number, required: false },
        Parameter { name: "EndDiameter", kind: ParameterKind::Number, required: false },
        Parameter { name: "BlendContinuity", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tools_028 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-028",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Locations", kind: InputKind::Point, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Mirror X", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Mirror Y", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Diameter", kind: ParameterKind::Number, required: false },
        Parameter { name: "Height", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tools_029 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-029",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Tips", kind: InputKind::Point, min: 1, max: None },
        Role { name: "Base", kind: InputKind::Surface, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "AutoProject", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Mirror X", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Mirror Y", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "StartDiameter", kind: ParameterKind::Number, required: false },
        Parameter { name: "EndDiameter", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tools_030 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-030",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Tips", kind: InputKind::Point, min: 1, max: None },
        Role { name: "Base", kind: InputKind::Surface, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "AutoProject", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Mirror X", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Mirror Y", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "StartDiameter", kind: ParameterKind::Number, required: false },
        Parameter { name: "EndDiameter", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tools_031 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-031",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Tips", kind: InputKind::Point, min: 1, max: None },
        Role { name: "Base", kind: InputKind::Surface, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "AutoProject", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Mirror X", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Mirror Y", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "StartDiameter", kind: ParameterKind::Number, required: false },
        Parameter { name: "EndDiameter", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tools_032 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-032",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Curve", kind: InputKind::Curve, min: 0, max: Some(1) },
        Role { name: "Rail", kind: InputKind::Curve, min: 0, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tools_033 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-033",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Profile", kind: InputKind::Curve, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Amount", kind: ParameterKind::Number, required: false },
        Parameter { name: "Flip", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tools_034 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-034",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Profiles", kind: InputKind::Curve, min: 2, max: Some(2) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tools_035 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-035",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "RingRail", kind: InputKind::Curve, min: 1, max: Some(1) },
        Role { name: "Shape", kind: InputKind::Curve, min: 1, max: Some(1) },
        Role { name: "Outside", kind: InputKind::Curve, min: 0, max: Some(1) },
        Role { name: "SideProfile", kind: InputKind::Curve, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Outside Rail", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Side Profile", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Planar", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Mirror", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Rotate", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Rail", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tools_036 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-036",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Curve", kind: InputKind::Curve, min: 1, max: Some(1) },
        Role { name: "Object", kind: InputKind::Object, min: 0, max: None },
        Role { name: "Aim", kind: InputKind::Object, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Object Type", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Start Position", kind: ParameterKind::Number, required: false },
        Parameter { name: "End Position", kind: ParameterKind::Number, required: false },
        Parameter { name: "Spacing", kind: ParameterKind::Number, required: false },
        Parameter { name: "Z Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Roll", kind: ParameterKind::Number, required: false },
        Parameter { name: "Start Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "End Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Mirror", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Orientation", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Flip", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Gem Tops", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Taper Gems", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Taper Spacing", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Packing Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Styles", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tools_037 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-037",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Mesh", kind: InputKind::Mesh, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Percentage", kind: ParameterKind::Number, required: false },
        Parameter { name: "Delete", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Weld", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "WeldAngle", kind: ParameterKind::Number, required: false },
        Parameter { name: "SplitDisjointMeshes", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "ModelUnits", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tools_038 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-038",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Surface", kind: InputKind::Surface, min: 1, max: Some(1) },
        Role { name: "Boundary", kind: InputKind::Curve, min: 1, max: Some(1) },
        Role { name: "DirectionPoints", kind: InputKind::Point, min: 0, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Length", kind: ParameterKind::Number, required: false },
        Parameter { name: "Tolerance", kind: ParameterKind::Number, required: false },
        Parameter { name: "Splits", kind: ParameterKind::Number, required: false },
        Parameter { name: "Surface", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Direction", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tools_039 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-039",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Mesh", kind: InputKind::Mesh, min: 1, max: Some(1) },
        Role { name: "TopLine", kind: InputKind::Curve, min: 1, max: Some(1) },
        Role { name: "BottomLine", kind: InputKind::Curve, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Maintain Z", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tools_040 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-040",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Curve", kind: InputKind::Curve, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Mirror", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Mirror Type", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tools_041 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-041",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Object1", kind: InputKind::Solid, min: 1, max: None },
        Role { name: "Object2", kind: InputKind::Solid, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "OnebyOne", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "ForceBoolean", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Operation", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Keep Cutter", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Auto Scale Object 1", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Auto Scale Object 2", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Auto Scale Factor", kind: ParameterKind::Number, required: false },
        Parameter { name: "Auto Move", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_tools_042 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-042",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Surface", kind: InputKind::Surface, min: 1, max: Some(1) },
        Role { name: "Curve1", kind: InputKind::Curve, min: 1, max: Some(1) },
        Role { name: "Curve2", kind: InputKind::Curve, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_gem_001 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-GEM-001",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Document", kind: InputKind::Document, min: 1, max: Some(1) },
        Role { name: "ExistingGem", kind: InputKind::Gem, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Cut Type", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Gem Shape", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Gem Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Keep Original Size", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Length", kind: ParameterKind::Number, required: false },
        Parameter { name: "Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Depth", kind: ParameterKind::Number, required: false },
        Parameter { name: "Percentage", kind: ParameterKind::Number, required: false },
        Parameter { name: "On Surface", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_gem_002 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-GEM-002",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Curve", kind: InputKind::Curve, min: 1, max: Some(1) },
        Role { name: "Template", kind: InputKind::Gem, min: 0, max: Some(1) },
        Role { name: "Aim", kind: InputKind::Object, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Start Position", kind: ParameterKind::Number, required: false },
        Parameter { name: "End Position", kind: ParameterKind::Number, required: false },
        Parameter { name: "Spacing", kind: ParameterKind::Number, required: false },
        Parameter { name: "Z Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Roll", kind: ParameterKind::Number, required: false },
        Parameter { name: "Start Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "End Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Taper Gems", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Taper Spacing", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Mirror", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Culet", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Object", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Flip", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Gem Tops", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Spacing Method", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Object Type", kind: ParameterKind::Choice, required: false },
        Parameter { name: "SelectGem", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Browser", kind: ParameterKind::Reference, required: false },
        Parameter { name: "SnaptoCorners", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "ShowProngs", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Styles", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_gem_003 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-GEM-003",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Curve", kind: InputKind::Curve, min: 1, max: Some(1) },
        Role { name: "Aim", kind: InputKind::Object, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Start Position", kind: ParameterKind::Number, required: false },
        Parameter { name: "End Position", kind: ParameterKind::Number, required: false },
        Parameter { name: "Start Diameter", kind: ParameterKind::Number, required: false },
        Parameter { name: "End Diameter", kind: ParameterKind::Number, required: false },
        Parameter { name: "Spacing", kind: ParameterKind::Number, required: false },
        Parameter { name: "Start Y Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "End Y Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Start Z Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "End Z Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Tilt", kind: ParameterKind::Number, required: false },
        Parameter { name: "Roll", kind: ParameterKind::Number, required: false },
        Parameter { name: "Gem Shape", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Placement", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Orientation", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Spacing Method", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Pull", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Close", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Mirror", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Flip", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Taper Diameter", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Taper Spacing", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Taper Y Offset", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Taper Z Offset", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Copy", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_gem_004 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-GEM-004",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Curve", kind: InputKind::Curve, min: 1, max: Some(1) },
        Role { name: "Aim", kind: InputKind::Object, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Start Diameter", kind: ParameterKind::Number, required: false },
        Parameter { name: "End Diameter", kind: ParameterKind::Number, required: false },
        Parameter { name: "Spacing", kind: ParameterKind::Number, required: false },
        Parameter { name: "Start Y Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "End Y Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Start Z Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "End Z Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Tilt", kind: ParameterKind::Number, required: false },
        Parameter { name: "Roll", kind: ParameterKind::Number, required: false },
        Parameter { name: "Gem Shape", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Placement", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Orientation", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Spacing Method", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Pull", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Flip", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Taper Diameter", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Taper Spacing", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Taper Y Offset", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Taper Z Offset", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Copy", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Count", kind: ParameterKind::Number, required: false },
        Parameter { name: "Position", kind: ParameterKind::Number, required: false },
        Parameter { name: "Direction", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_gem_005 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-GEM-005",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Curve", kind: InputKind::Curve, min: 1, max: Some(1) },
        Role { name: "GemList", kind: InputKind::Gem, min: 1, max: None },
        Role { name: "Aim", kind: InputKind::Object, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Gem List", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Position", kind: ParameterKind::Number, required: false },
        Parameter { name: "Spacing", kind: ParameterKind::Number, required: false },
        Parameter { name: "Start Y Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "End Y Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Start Z Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "End Z Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Tilt", kind: ParameterKind::Number, required: false },
        Parameter { name: "Roll", kind: ParameterKind::Number, required: false },
        Parameter { name: "Start Diameter", kind: ParameterKind::Number, required: false },
        Parameter { name: "End Diameter", kind: ParameterKind::Number, required: false },
        Parameter { name: "Placement", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Orientation", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Direction", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Flip", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Pull", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Taper Diameter", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Taper Spacing", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Taper Y Offset", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Taper Z Offset", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Copy", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_gem_006 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-GEM-006",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Primary", kind: InputKind::Curve, min: 1, max: Some(1) },
        Role { name: "Secondary", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Primary Curve", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Secondary Curve", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Start Position", kind: ParameterKind::Number, required: false },
        Parameter { name: "End Position", kind: ParameterKind::Number, required: false },
        Parameter { name: "Start Diameter", kind: ParameterKind::Number, required: false },
        Parameter { name: "End Diameter", kind: ParameterKind::Number, required: false },
        Parameter { name: "Spacing", kind: ParameterKind::Number, required: false },
        Parameter { name: "Row Spacing", kind: ParameterKind::Number, required: false },
        Parameter { name: "Z Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Roll", kind: ParameterKind::Number, required: false },
        Parameter { name: "Pattern", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Placement", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Orientation", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Close", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Flip", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Mirror", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Pull", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Taper Diameter", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Taper Spacing", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_gem_007 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-GEM-007",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Gem", kind: InputKind::Gem, min: 1, max: Some(1) },
        Role { name: "Rail", kind: InputKind::Curve, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Tilt", kind: ParameterKind::Number, required: false },
        Parameter { name: "Twist", kind: ParameterKind::Number, required: false },
        Parameter { name: "Roll", kind: ParameterKind::Number, required: false },
        Parameter { name: "X Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Y Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Z Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Edit Sweep", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Select Rail", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_gem_008 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-GEM-008",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Gems", kind: InputKind::Gem, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Culet", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "North", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Information X", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Information Y", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Information Z", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Information Weight", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Bezel Profile", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Prong Points", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Horizontal Prong Lines", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Vertical Prong Lines", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Profile: Offset Amount", kind: ParameterKind::Number, required: false },
        Parameter { name: "Prongs: Horizontal Length", kind: ParameterKind::Number, required: false },
        Parameter { name: "Prongs: Vertical Length", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_gem_009 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-GEM-009",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Gems", kind: InputKind::Gem, min: 1, max: None },
        Role { name: "Objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Show Guide", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Action", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_gem_010 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-GEM-010",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "CenterGem", kind: InputKind::Gem, min: 1, max: Some(1) },
        Role { name: "Profile", kind: InputKind::Curve, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Profile", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Spacing From Center", kind: ParameterKind::Number, required: false },
        Parameter { name: "Gems Height Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Spacing", kind: ParameterKind::Number, required: false },
        Parameter { name: "Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Rotation", kind: ParameterKind::Number, required: false },
        Parameter { name: "Placement", kind: ParameterKind::Number, required: false },
        Parameter { name: "Fix Position", kind: ParameterKind::Number, required: false },
        Parameter { name: "Profile Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Profile Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Inner Channel Wall Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Outer Channel Wall Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Channel Spacing", kind: ParameterKind::Number, required: false },
        Parameter { name: "Channel Wall Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Channel Blend", kind: ParameterKind::Number, required: false },
        Parameter { name: "Prongs- Height Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Prongs-Dome Amount", kind: ParameterKind::Number, required: false },
        Parameter { name: "Prongs-Taper", kind: ParameterKind::Number, required: false },
        Parameter { name: "Edge Nudge", kind: ParameterKind::Number, required: false },
        Parameter { name: "Gem Nudge", kind: ParameterKind::Number, required: false },
        Parameter { name: "Prongs Vertical", kind: ParameterKind::Number, required: false },
        Parameter { name: "Override", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Gem Shape", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Match Shape", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Even", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Prong Meshed", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Styles", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_gem_011 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-GEM-011",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Surface", kind: InputKind::Surface, min: 1, max: Some(1) },
        Role { name: "ExistingGems", kind: InputKind::Gem, min: 0, max: None },
        Role { name: "ConstraintCurve", kind: InputKind::Curve, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Vertical Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Scale", kind: ParameterKind::Number, required: false },
        Parameter { name: "Spacing", kind: ParameterKind::Number, required: false },
        Parameter { name: "Copy", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Snap", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Flip", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Curve", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Gems", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_gem_012 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-GEM-012",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Outline", kind: InputKind::Curve, min: 1, max: Some(1) },
        Role { name: "FacetPoints", kind: InputKind::Point, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Cut Type", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Facet Count", kind: ParameterKind::Number, required: false },
        Parameter { name: "Crown Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Crown Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Girdle Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Pavilion Depth", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bulge Factor", kind: ParameterKind::Number, required: false },
        Parameter { name: "Pavilion Rows", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_gem_013 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-GEM-013",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Gem", kind: InputKind::Gem, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Length", kind: ParameterKind::Number, required: false },
        Parameter { name: "Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Depth", kind: ParameterKind::Number, required: false },
        Parameter { name: "Girdle Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Corner", kind: ParameterKind::Number, required: false },
        Parameter { name: "Move X", kind: ParameterKind::Number, required: false },
        Parameter { name: "Move Y", kind: ParameterKind::Number, required: false },
        Parameter { name: "Move Z", kind: ParameterKind::Number, required: false },
        Parameter { name: "Rotate X", kind: ParameterKind::Number, required: false },
        Parameter { name: "Rotate Y", kind: ParameterKind::Number, required: false },
        Parameter { name: "Rotate Z", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_gem_014 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-GEM-014",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Gem", kind: InputKind::Gem, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Length", kind: ParameterKind::Number, required: false },
        Parameter { name: "Width Bottom", kind: ParameterKind::Number, required: false },
        Parameter { name: "Width Top", kind: ParameterKind::Number, required: false },
        Parameter { name: "Depth", kind: ParameterKind::Number, required: false },
        Parameter { name: "Girdle Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Move X", kind: ParameterKind::Number, required: false },
        Parameter { name: "Move Y", kind: ParameterKind::Number, required: false },
        Parameter { name: "Move Z", kind: ParameterKind::Number, required: false },
        Parameter { name: "Rotate X", kind: ParameterKind::Number, required: false },
        Parameter { name: "Rotate Y", kind: ParameterKind::Number, required: false },
        Parameter { name: "Rotate Z", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_gem_015 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-GEM-015",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Curves", kind: InputKind::Curve, min: 2, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "Size List", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Aspect", kind: ParameterKind::Number, required: false },
        Parameter { name: "Banking", kind: ParameterKind::Number, required: false },
        Parameter { name: "Spacing", kind: ParameterKind::Number, required: false },
        Parameter { name: "Flip", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Middle", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Straight", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_gem_016 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-GEM-016",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Curves", kind: InputKind::Curve, min: 2, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "Spacing", kind: ParameterKind::Number, required: false },
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "AutoSpacing", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "GemTop", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_gem_017 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-GEM-017",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Center", kind: InputKind::Gem, min: 1, max: Some(1) },
        Role { name: "BaseCurve", kind: InputKind::Curve, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Stage", kind: ParameterKind::Choice, required: false },
        Parameter { name: "XY Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Z Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Degree", kind: ParameterKind::Number, required: false },
        Parameter { name: "Edge Gem Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Number of Gems", kind: ParameterKind::Number, required: false },
        Parameter { name: "Use Maximum Gems", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Start", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Edge Profile", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Percentage of Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Diameter", kind: ParameterKind::Number, required: false },
        Parameter { name: "Distance from Girdle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Base Curve", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Bottom Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Nudge", kind: ParameterKind::Number, required: false },
        Parameter { name: "Inside Prong Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Inside Prong Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Outside Prong Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Outside Prong Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Layout", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Use Base Curve", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Dome Prongs", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Add Inner", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Add Outer", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_gem_018 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-GEM-018",
    effect: Effect::ExternalOutput,
    roles: &[
        Role { name: "Gems", kind: InputKind::Gem, min: 1, max: None },
        Role { name: "Destination", kind: InputKind::Path, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "File", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Format", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Path", kind: ParameterKind::Text, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_gem_019 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-GEM-019",
    effect: Effect::ReadOnly,
    roles: &[
        Role { name: "Gems", kind: InputKind::Gem, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Advanced", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Set Materials", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Specific Gravity", kind: ParameterKind::Number, required: false },
        Parameter { name: "Save Report", kind: ParameterKind::Text, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_gem_020 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-GEM-020",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Surface", kind: InputKind::Surface, min: 1, max: Some(1) },
        Role { name: "Start", kind: InputKind::Point, min: 0, max: Some(1) },
        Role { name: "SeedGems", kind: InputKind::Gem, min: 0, max: None },
        Role { name: "Curves", kind: InputKind::Curve, min: 0, max: None },
    ],
    parameters: &[
        Parameter { name: "Start", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Surface", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Seed Gems", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Curve", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Lock Start", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Edge Curve", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Edge Minimum", kind: ParameterKind::Number, required: false },
        Parameter { name: "Edge Maximum", kind: ParameterKind::Number, required: false },
        Parameter { name: "Attractor", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Auto Attractor", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Layout", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Incremental", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Slow", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Edges", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Base Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Min Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Max Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Taper", kind: ParameterKind::Number, required: false },
        Parameter { name: "Sizing Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Tight or Loose", kind: ParameterKind::Number, required: false },
        Parameter { name: "Min Spacing", kind: ParameterKind::Number, required: false },
        Parameter { name: "Pre-Sizing Equalize", kind: ParameterKind::Number, required: false },
        Parameter { name: "Post-Sizing Equalize", kind: ParameterKind::Number, required: false },
        Parameter { name: "Display", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Styles", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Pre-Sizing Perturb", kind: ParameterKind::Number, required: false },
        Parameter { name: "Post-Sizing Perturb", kind: ParameterKind::Number, required: false },
        Parameter { name: "Post-Sizing Perturb Grow", kind: ParameterKind::Number, required: false },
        Parameter { name: "Flip", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_gem_021 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-GEM-021",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Surface", kind: InputKind::Surface, min: 1, max: Some(1) },
        Role { name: "Start", kind: InputKind::Point, min: 1, max: Some(1) },
        Role { name: "GemList", kind: InputKind::Gem, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Add Gems", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Quantity", kind: ParameterKind::Number, required: false },
        Parameter { name: "Get Start Point", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Minimum Spacing", kind: ParameterKind::Number, required: false },
        Parameter { name: "Spacing", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_gem_022 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-GEM-022",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Top", kind: InputKind::Surface, min: 1, max: Some(1) },
        Role { name: "Bottom", kind: InputKind::Surface, min: 1, max: Some(1) },
        Role { name: "Gems", kind: InputKind::Gem, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Top", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Bottom", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Gems", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Hexagonal Toggle", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Azure Depth", kind: ParameterKind::Number, required: false },
        Parameter { name: "Azure Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Through Hole", kind: ParameterKind::Number, required: false },
        Parameter { name: "Seat Angle", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_gem_023 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-GEM-023",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Surface", kind: InputKind::Surface, min: 1, max: Some(1) },
        Role { name: "Gems", kind: InputKind::Gem, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Center Prong Minimum", kind: ParameterKind::Number, required: false },
        Parameter { name: "Center Prong Maximum", kind: ParameterKind::Number, required: false },
        Parameter { name: "Shared Prong Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Unshared Prong Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Drop", kind: ParameterKind::Number, required: false },
        Parameter { name: "Taper", kind: ParameterKind::Number, required: false },
        Parameter { name: "Fillet", kind: ParameterKind::Number, required: false },
        Parameter { name: "Scale", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_gem_024 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-GEM-024",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Gems", kind: InputKind::Gem, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "North Flow Axis", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_gem_025 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-GEM-025",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Surface", kind: InputKind::Surface, min: 1, max: Some(1) },
        Role { name: "Gems", kind: InputKind::Gem, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Action", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Gem", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_gem_026 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-GEM-026",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Gems", kind: InputKind::Gem, min: 1, max: None },
        Role { name: "CuletObject", kind: InputKind::Object, min: 0, max: Some(1) },
        Role { name: "NorthObject", kind: InputKind::Object, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Culet Object", kind: ParameterKind::Reference, required: false },
        Parameter { name: "North Object", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Culet Direction", kind: ParameterKind::Choice, required: false },
        Parameter { name: "North Direction", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_gem_027 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-GEM-027",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Targets", kind: InputKind::Gem, min: 1, max: None },
        Role { name: "Source", kind: InputKind::Gem, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Style Sheets", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_gem_028 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-GEM-028",
    effect: Effect::ExternalOutput,
    roles: &[
        Role { name: "Gem", kind: InputKind::Gem, min: 1, max: Some(1) },
        Role { name: "Destination", kind: InputKind::Path, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Style", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Path", kind: ParameterKind::Text, required: false },
        Parameter { name: "Slot", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_gem_029 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-GEM-029",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Gems", kind: InputKind::Gem, min: 1, max: None },
        Role { name: "Base", kind: InputKind::Object, min: 1, max: Some(1) },
        Role { name: "Target", kind: InputKind::Object, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Base Curve", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Target Curve", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Base Surface", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Target Surface", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Copy", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Rigid", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Local", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Stretch", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_gem_030 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-GEM-030",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Targets", kind: InputKind::Gem, min: 1, max: None },
        Role { name: "File", kind: InputKind::Path, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Path", kind: ParameterKind::Text, required: false },
        Parameter { name: "Style File", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_gem_031 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-GEM-031",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Gem", kind: InputKind::Gem, min: 1, max: Some(1) },
        Role { name: "Surface", kind: InputKind::Surface, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Rigid", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Copy", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_gem_032 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-GEM-032",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Sphere", kind: InputKind::Surface, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Counting Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Count", kind: ParameterKind::Number, required: false },
        Parameter { name: "X", kind: ParameterKind::Number, required: false },
        Parameter { name: "T", kind: ParameterKind::Number, required: false },
        Parameter { name: "Removal", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Gem Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Gem Min Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Sphere Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Relax Amount", kind: ParameterKind::Number, required: false },
        Parameter { name: "Relax Strength", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_gem_033 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-GEM-033",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Gems", kind: InputKind::Gem, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Move X", kind: ParameterKind::Number, required: false },
        Parameter { name: "Move Y", kind: ParameterKind::Number, required: false },
        Parameter { name: "Move Z", kind: ParameterKind::Number, required: false },
        Parameter { name: "Rotate X", kind: ParameterKind::Number, required: false },
        Parameter { name: "Rotate Y", kind: ParameterKind::Number, required: false },
        Parameter { name: "Rotate Z", kind: ParameterKind::Number, required: false },
        Parameter { name: "Scale", kind: ParameterKind::Number, required: false },
        Parameter { name: "Action", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_gem_034 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-GEM-034",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_setting_001 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SETTING-001",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Gem", kind: InputKind::Gem, min: 1, max: Some(1) },
        Role { name: "ProngProfile", kind: InputKind::Curve, min: 0, max: Some(1) },
        Role { name: "RailProfile", kind: InputKind::Curve, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Prong Profile", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Rail Profile", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Control Level", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Prong Layout", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Prong Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Head Drop", kind: ParameterKind::Number, required: false },
        Parameter { name: "Prong Nudge", kind: ParameterKind::Number, required: false },
        Parameter { name: "Prong Dome", kind: ParameterKind::Number, required: false },
        Parameter { name: "Rail Drop", kind: ParameterKind::Number, required: false },
        Parameter { name: "Height Above Girdle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Head Angle X", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Prong X Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Prong Y Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Prong Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Rail X Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Rail Y Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Rail X Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Prong X Size Top", kind: ParameterKind::Number, required: false },
        Parameter { name: "Prong Y Size Top", kind: ParameterKind::Number, required: false },
        Parameter { name: "Prong X Size Base", kind: ParameterKind::Number, required: false },
        Parameter { name: "Prong Y Size Base", kind: ParameterKind::Number, required: false },
        Parameter { name: "Prong Nudge Top", kind: ParameterKind::Number, required: false },
        Parameter { name: "Prong Nudge Base", kind: ParameterKind::Number, required: false },
        Parameter { name: "Rail Count", kind: ParameterKind::Number, required: false },
        Parameter { name: "Shear Rails", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Styles", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Half Bezel Shape Toggle", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_setting_002 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SETTING-002",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Gem", kind: InputKind::Gem, min: 1, max: Some(1) },
        Role { name: "Profile", kind: InputKind::Curve, min: 0, max: Some(1) },
        Role { name: "RingRail", kind: InputKind::Curve, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Edge Profile", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Bezel Profile", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Bezel Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Base Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Dome Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bezel Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Seat Depth", kind: ParameterKind::Number, required: false },
        Parameter { name: "Seat Length", kind: ParameterKind::Number, required: false },
        Parameter { name: "Seat Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bezel Placement", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bezel Scale", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bezel Chamfer X", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bezel Chamfer Z", kind: ParameterKind::Number, required: false },
        Parameter { name: "Boolean", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Styles", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_setting_003 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SETTING-003",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Gem", kind: InputKind::Gem, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Crease Top", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Crease Seat", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Crease Bottom", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Faces", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Bezel Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Base Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Dome Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Seat Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Seat Length", kind: ParameterKind::Number, required: false },
        Parameter { name: "Seat Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bezel Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bezel Placement", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_setting_004 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SETTING-004",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Gem", kind: InputKind::Gem, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Crease Top", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Crease Seat", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Crease Bottom", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Faces", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Seat Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Seat Length", kind: ParameterKind::Number, required: false },
        Parameter { name: "Seat Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bezel Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Placement", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_setting_005 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SETTING-005",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Objects", kind: InputKind::Object, min: 1, max: None },
        Role { name: "RingRail", kind: InputKind::Curve, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_setting_006 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SETTING-006",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "GemOrBezel", kind: InputKind::Object, min: 1, max: Some(1) },
        Role { name: "Target", kind: InputKind::Solid, min: 0, max: Some(1) },
        Role { name: "Profile", kind: InputKind::Curve, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Edge Profile", kind: ParameterKind::Reference, required: false },
        Parameter { name: "X Inside Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Y Inside Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "X Outside Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Y Outside Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Cutter Length", kind: ParameterKind::Number, required: false },
        Parameter { name: "Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Z Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Y Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "X Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Number Of Cutters", kind: ParameterKind::Number, required: false },
        Parameter { name: "Position", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Boolean", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Surface", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Styles", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_setting_007 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SETTING-007",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Gems", kind: InputKind::Gem, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Prong Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Flow Layout", kind: ParameterKind::Choice, required: false },
        Parameter { name: "North Layout", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Large Prong Diameter", kind: ParameterKind::Number, required: false },
        Parameter { name: "Small Prong Diameter", kind: ParameterKind::Number, required: false },
        Parameter { name: "Large Prong Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Small Prong Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Drop", kind: ParameterKind::Number, required: false },
        Parameter { name: "Nudge", kind: ParameterKind::Number, required: false },
        Parameter { name: "Fillet", kind: ParameterKind::Number, required: false },
        Parameter { name: "Taper", kind: ParameterKind::Number, required: false },
        Parameter { name: "Rotate Z", kind: ParameterKind::Number, required: false },
        Parameter { name: "End Prong Layout", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Styles", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_setting_008 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SETTING-008",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Prong", kind: InputKind::Object, min: 1, max: Some(1) },
        Role { name: "Gem", kind: InputKind::Gem, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Diameter", kind: ParameterKind::Number, required: false },
        Parameter { name: "Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Drop", kind: ParameterKind::Number, required: false },
        Parameter { name: "Nudge", kind: ParameterKind::Number, required: false },
        Parameter { name: "Fillet", kind: ParameterKind::Number, required: false },
        Parameter { name: "Taper", kind: ParameterKind::Number, required: false },
        Parameter { name: "Rotate Z", kind: ParameterKind::Number, required: false },
        Parameter { name: "Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Set Gem", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Action", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_setting_009 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SETTING-009",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Surface", kind: InputKind::Surface, min: 1, max: Some(1) },
        Role { name: "Locations", kind: InputKind::Point, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Start Surface", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Diameter", kind: ParameterKind::Number, required: false },
        Parameter { name: "Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Drop", kind: ParameterKind::Number, required: false },
        Parameter { name: "Fillet", kind: ParameterKind::Number, required: false },
        Parameter { name: "Taper", kind: ParameterKind::Number, required: false },
        Parameter { name: "Flip", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Styles", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_setting_010 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SETTING-010",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Surface", kind: InputKind::Surface, min: 1, max: Some(1) },
        Role { name: "Object", kind: InputKind::Object, min: 0, max: Some(1) },
        Role { name: "Curve", kind: InputKind::Curve, min: 0, max: Some(1) },
        Role { name: "Locations", kind: InputKind::Point, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Scale", kind: ParameterKind::Number, required: false },
        Parameter { name: "Vertical Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Object Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Copy", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Flip", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Curve", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_setting_011 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SETTING-011",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Gems", kind: InputKind::Gem, min: 1, max: None },
        Role { name: "Profile", kind: InputKind::Curve, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Top Profile", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Profile Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Gem Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Gem End Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Cap Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Profile Depth", kind: ParameterKind::Number, required: false },
        Parameter { name: "Profile Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Profile End Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bottom Inside Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bottom Outside Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Inside Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Outside Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bottom Inside Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bottom Outside Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Inside Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Outside Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "End Curvature", kind: ParameterKind::Number, required: false },
        Parameter { name: "End Adjustments", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Mirror", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Extension Type", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Tweak Corners", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Corner Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Distance", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Styles", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Profile Corner End Widths: Bottom Inside", kind: ParameterKind::Number, required: false },
        Parameter { name: "Profile Corner End Widths: Bottom Outside", kind: ParameterKind::Number, required: false },
        Parameter { name: "Profile Corner End Widths: Top Inside", kind: ParameterKind::Number, required: false },
        Parameter { name: "Profile Corner End Widths: Top Outside", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_setting_012 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SETTING-012",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Curve", kind: InputKind::Curve, min: 1, max: Some(1) },
        Role { name: "Template", kind: InputKind::Object, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Start Position", kind: ParameterKind::Number, required: false },
        Parameter { name: "End Position", kind: ParameterKind::Number, required: false },
        Parameter { name: "Spacing", kind: ParameterKind::Number, required: false },
        Parameter { name: "Z Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Roll", kind: ParameterKind::Number, required: false },
        Parameter { name: "Start Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "End Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Taper Gems", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Taper Spacing", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Mirror", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Culet", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Object", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Flip", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Gem Tops", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Spacing Method", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Object Type", kind: ParameterKind::Choice, required: false },
        Parameter { name: "SelectGem", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Browser", kind: ParameterKind::Reference, required: false },
        Parameter { name: "SnaptoCorners", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "ShowProngs", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Styles", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Gem Line Dir", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_setting_013 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SETTING-013",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Gems", kind: InputKind::Gem, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Side Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "End Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "V Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Spacing", kind: ParameterKind::Number, required: false },
        Parameter { name: "End", kind: ParameterKind::Number, required: false },
        Parameter { name: "Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Tweak Corners", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Corner Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Type", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_cutter_001 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CUTTER-001",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Gems", kind: InputKind::Gem, min: 1, max: None },
        Role { name: "Target", kind: InputKind::Solid, min: 0, max: Some(1) },
        Role { name: "Direction", kind: InputKind::Point, min: 0, max: Some(1) },
        Role { name: "Profile", kind: InputKind::Curve, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Lower Taper", kind: ParameterKind::Number, required: false },
        Parameter { name: "Lower Depth", kind: ParameterKind::Number, required: false },
        Parameter { name: "Lower Seat Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Girdle Scale", kind: ParameterKind::Number, required: false },
        Parameter { name: "Girdle Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Z Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Edge Profile", kind: ParameterKind::Reference, required: false },
        Parameter { name: "1 Rotation", kind: ParameterKind::Number, required: false },
        Parameter { name: "1 Y Scale", kind: ParameterKind::Number, required: false },
        Parameter { name: "2 Rotation", kind: ParameterKind::Number, required: false },
        Parameter { name: "2 Y Scale", kind: ParameterKind::Number, required: false },
        Parameter { name: "Lower Seat Depth", kind: ParameterKind::Number, required: false },
        Parameter { name: "3 Rotation", kind: ParameterKind::Number, required: false },
        Parameter { name: "3 Y Scale", kind: ParameterKind::Number, required: false },
        Parameter { name: "4 Rotation", kind: ParameterKind::Number, required: false },
        Parameter { name: "4 Y Scale", kind: ParameterKind::Number, required: false },
        Parameter { name: "5 Rotation", kind: ParameterKind::Number, required: false },
        Parameter { name: "5 Y Scale", kind: ParameterKind::Number, required: false },
        Parameter { name: "Upper Seat Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Upper Seat Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Upper Taper", kind: ParameterKind::Number, required: false },
        Parameter { name: "Upper Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Interactive Boolean", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Surface", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Bottom Dir", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Styles", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_cutter_002 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CUTTER-002",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Gems", kind: InputKind::Gem, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Cutter Type", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Scale X and Y", kind: ParameterKind::Number, required: false },
        Parameter { name: "Z Scale", kind: ParameterKind::Number, required: false },
        Parameter { name: "Z Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Draw All", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_cutter_003 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CUTTER-003",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Gems", kind: InputKind::Gem, min: 1, max: None },
        Role { name: "Target", kind: InputKind::Solid, min: 0, max: Some(1) },
        Role { name: "Attach", kind: InputKind::Surface, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Z Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Extension", kind: ParameterKind::Number, required: false },
        Parameter { name: "Cap Length", kind: ParameterKind::Number, required: false },
        Parameter { name: "Cap Scale", kind: ParameterKind::Number, required: false },
        Parameter { name: "Mirror", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Flip AB", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Cap A", kind: ParameterKind::Choice, required: false },
        Parameter { name: "ExtTypeA", kind: ParameterKind::Choice, required: false },
        Parameter { name: "ExtTypeB", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Tweak Corners", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Corner Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Closed", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Middle Profile", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Surface to Attach", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Boolean", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Surface", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Styles", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Cap B", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_cutter_004 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CUTTER-004",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Gems", kind: InputKind::Gem, min: 1, max: None },
        Role { name: "Target", kind: InputKind::Solid, min: 0, max: Some(1) },
        Role { name: "Attach", kind: InputKind::Surface, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Drop", kind: ParameterKind::Number, required: false },
        Parameter { name: "Middle Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Width Adjustment", kind: ParameterKind::Number, required: false },
        Parameter { name: "Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bend", kind: ParameterKind::Number, required: false },
        Parameter { name: "Length", kind: ParameterKind::Number, required: false },
        Parameter { name: "Start", kind: ParameterKind::Number, required: false },
        Parameter { name: "End", kind: ParameterKind::Number, required: false },
        Parameter { name: "Mirror", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Flip AB", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Surface to Attach", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Boolean", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Surface", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Styles", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_cutter_005 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CUTTER-005",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Gems", kind: InputKind::Gem, min: 1, max: None },
        Role { name: "Target", kind: InputKind::Solid, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Spacing", kind: ParameterKind::Number, required: false },
        Parameter { name: "Taper", kind: ParameterKind::Number, required: false },
        Parameter { name: "Depth", kind: ParameterKind::Number, required: false },
        Parameter { name: "Outside", kind: ParameterKind::Number, required: false },
        Parameter { name: "Start", kind: ParameterKind::Number, required: false },
        Parameter { name: "End", kind: ParameterKind::Number, required: false },
        Parameter { name: "Mirror", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Flip AB", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Boolean", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Surface", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Styles", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_cutter_006 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CUTTER-006",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Gem", kind: InputKind::Gem, min: 1, max: Some(1) },
        Role { name: "Target", kind: InputKind::Solid, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Cut Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Depth", kind: ParameterKind::Number, required: false },
        Parameter { name: "Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Length", kind: ParameterKind::Number, required: false },
        Parameter { name: "2D Scale", kind: ParameterKind::Number, required: false },
        Parameter { name: "Boolean", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Surface", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Styles", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_cutter_007 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CUTTER-007",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "GemOrBezel", kind: InputKind::Object, min: 1, max: Some(1) },
        Role { name: "Target", kind: InputKind::Solid, min: 0, max: Some(1) },
        Role { name: "Profile", kind: InputKind::Curve, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Edge Profile", kind: ParameterKind::Reference, required: false },
        Parameter { name: "X Inside Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Y Inside Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "X Outside Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Y Outside Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Cutter Length", kind: ParameterKind::Number, required: false },
        Parameter { name: "Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Z Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Y Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "X Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Number Of Cutters", kind: ParameterKind::Number, required: false },
        Parameter { name: "Position", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Boolean", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Surface", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Styles", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_cutter_008 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CUTTER-008",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Objects", kind: InputKind::Solid, min: 1, max: None },
        Role { name: "RingRail", kind: InputKind::Curve, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_cutter_009 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CUTTER-009",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Cutter Kind", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_cutter_010 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CUTTER-010",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Object1", kind: InputKind::Solid, min: 1, max: None },
        Role { name: "Object2", kind: InputKind::Solid, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Operation", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Keep Cutter", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "One by One", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Force Boolean", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Auto Scale Object 1", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Auto Scale Object 2", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Scale Factor", kind: ParameterKind::Number, required: false },
        Parameter { name: "Auto Move Object 1", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Auto Move Object 2", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Move X", kind: ParameterKind::Number, required: false },
        Parameter { name: "Move Y", kind: ParameterKind::Number, required: false },
        Parameter { name: "Move Z", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_cutter_011 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CUTTER-011",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Object", kind: InputKind::Solid, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Quadrant", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_render_001 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-RENDER-001",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Ground Plane Material", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_render_002 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-RENDER-002",
    effect: Effect::ExternalOutput,
    roles: &[
        Role { name: "Document", kind: InputKind::Document, min: 1, max: Some(1) },
        Role { name: "Viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "View", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Save Image Format", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_render_003 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-RENDER-003",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "Viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Height", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_render_004 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-RENDER-004",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "Viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_render_005 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-RENDER-005",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "Document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Style", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_render_006 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-RENDER-006",
    effect: Effect::ExternalOutput,
    roles: &[
        Role { name: "Document", kind: InputKind::Document, min: 1, max: Some(1) },
        Role { name: "Destination", kind: InputKind::Path, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Package Format", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_render_007 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-RENDER-007",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "Document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_render_008 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-RENDER-008",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "Objects", kind: InputKind::Object, min: 0, max: None },
        Role { name: "Document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Category", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Library", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Material", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Reflection Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Alpha", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Default Ground Plane", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Metal Quality", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Specific Gravity", kind: ParameterKind::Number, required: false },
        Parameter { name: "Color Grid", kind: ParameterKind::Text, required: false },
        Parameter { name: "Color Picker/Hue", kind: ParameterKind::Number, required: false },
        Parameter { name: "Color Picker/RGB", kind: ParameterKind::Text, required: false },
        Parameter { name: "Color Picker/Tint Shade", kind: ParameterKind::Number, required: false },
        Parameter { name: "Reflection Amount", kind: ParameterKind::Number, required: false },
        Parameter { name: "Polish Level", kind: ParameterKind::Number, required: false },
        Parameter { name: "Reflection Quality", kind: ParameterKind::Number, required: false },
        Parameter { name: "Anisotropy", kind: ParameterKind::Number, required: false },
        Parameter { name: "Texture", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Bump Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Map Rotation", kind: ParameterKind::Number, required: false },
        Parameter { name: "Map Tiling U", kind: ParameterKind::Number, required: false },
        Parameter { name: "Map Tiling V", kind: ParameterKind::Number, required: false },
        Parameter { name: "Scene/Override", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Scene/Mapping", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Scene/Image", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Scene/Brightness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Scene/Map Rotation", kind: ParameterKind::Number, required: false },
        Parameter { name: "Scene/Color", kind: ParameterKind::Text, required: false },
        Parameter { name: "Toon", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Toon/Color", kind: ParameterKind::Text, required: false },
        Parameter { name: "Toon/Line Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Toon/Edge Detection", kind: ParameterKind::Number, required: false },
        Parameter { name: "Light/Emissive", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Light/Color", kind: ParameterKind::Text, required: false },
        Parameter { name: "Light/Intensity", kind: ParameterKind::Number, required: false },
        Parameter { name: "Gem/Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Gem/Refraction Image", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Gem/Refraction Multiplier", kind: ParameterKind::Number, required: false },
        Parameter { name: "Gem/Color Image", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Gem/Color Multiplier", kind: ParameterKind::Number, required: false },
        Parameter { name: "Gem/Crop", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Gem/Hue", kind: ParameterKind::Number, required: false },
        Parameter { name: "Gem/Brightness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Gem/Saturation", kind: ParameterKind::Number, required: false },
        Parameter { name: "Gem/RGB", kind: ParameterKind::Text, required: false },
        Parameter { name: "Gem/Reflection Gloss", kind: ParameterKind::Number, required: false },
        Parameter { name: "Gem/Refraction Index", kind: ParameterKind::Number, required: false },
        Parameter { name: "Gem/Refraction Gloss", kind: ParameterKind::Number, required: false },
        Parameter { name: "Gem/Dispersion Enabled", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Gem/Dispersion", kind: ParameterKind::Number, required: false },
        Parameter { name: "Gem/Refraction Depth", kind: ParameterKind::Number, required: false },
        Parameter { name: "Pearl/Body Color", kind: ParameterKind::Text, required: false },
        Parameter { name: "Pearl/Overtone", kind: ParameterKind::Text, required: false },
        Parameter { name: "Pearl/Gradient Transition", kind: ParameterKind::Number, required: false },
        Parameter { name: "Pearl/Overtone Intensity", kind: ParameterKind::Number, required: false },
        Parameter { name: "Pearl/Luster", kind: ParameterKind::Number, required: false },
        Parameter { name: "Pearl/Reflection Gloss", kind: ParameterKind::Number, required: false },
        Parameter { name: "Pearl/Blemish Intensity", kind: ParameterKind::Number, required: false },
        Parameter { name: "Ground Plane/Color Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Ground Plane/Color Map", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Ground Plane/Texture Map", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Environment/Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Environment/Image", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Environment/Color", kind: ParameterKind::Text, required: false },
        Parameter { name: "Environment/Mapping", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Environment/Crop", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Environment/Hue", kind: ParameterKind::Number, required: false },
        Parameter { name: "Environment/Brightness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Environment/Saturation", kind: ParameterKind::Number, required: false },
        Parameter { name: "Environment/Map Rotation U", kind: ParameterKind::Number, required: false },
        Parameter { name: "Environment/Map Rotation V", kind: ParameterKind::Number, required: false },
        Parameter { name: "Lighting/Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Lighting/Image", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Lighting/Color", kind: ParameterKind::Text, required: false },
        Parameter { name: "Lighting/Style Lights", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Caustics", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Max Photon", kind: ParameterKind::Number, required: false },
        Parameter { name: "Multiplier", kind: ParameterKind::Number, required: false },
        Parameter { name: "Search Distance", kind: ParameterKind::Number, required: false },
        Parameter { name: "Show Camera", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Render Curves", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Vignetting", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Camera Target", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Depth of Field", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "F-Number", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bokeh Effect", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Blade Count", kind: ParameterKind::Number, required: false },
        Parameter { name: "Film Grain", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Bloom Effect", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Bloom/Weight", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bloom/Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Glare Effect", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Glare/Weight", kind: ParameterKind::Number, required: false },
        Parameter { name: "Glare/Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Diffraction", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_render_009 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-RENDER-009",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Base", kind: InputKind::Point, min: 1, max: Some(1) },
        Role { name: "Source", kind: InputKind::Point, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Enabled", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Color", kind: ParameterKind::Text, required: false },
        Parameter { name: "Intensity", kind: ParameterKind::Number, required: false },
        Parameter { name: "Shadows", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Shadow Radius", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_render_010 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-RENDER-010",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Target", kind: InputKind::Point, min: 1, max: Some(1) },
        Role { name: "Location", kind: InputKind::Point, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_render_011 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-RENDER-011",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Location", kind: InputKind::Point, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_render_012 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-RENDER-012",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Start", kind: InputKind::Point, min: 1, max: Some(1) },
        Role { name: "LengthPoint", kind: InputKind::Point, min: 1, max: Some(1) },
        Role { name: "WidthPoint", kind: InputKind::Point, min: 1, max: Some(1) },
        Role { name: "Target", kind: InputKind::Point, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Target", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_render_013 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-RENDER-013",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "Spotlight", kind: InputKind::Object, min: 1, max: Some(1) },
        Role { name: "Viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_render_014 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-RENDER-014",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Start", kind: InputKind::Point, min: 1, max: Some(1) },
        Role { name: "End", kind: InputKind::Point, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_render_015 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-RENDER-015",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Prop", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_render_016 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-RENDER-016",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "Document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Render Plugin", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_render_017 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-RENDER-017",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "Objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "On", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Softening", kind: ParameterKind::Number, required: false },
        Parameter { name: "Chamfer", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Faceted", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Pre-reduction", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_render_018 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-RENDER-018",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "Objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Image", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Blend with Object", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Show Isocurve", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Adjust Mesh", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_render_019 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-RENDER-019",
    effect: Effect::ExternalOutput,
    roles: &[
        Role { name: "Document", kind: InputKind::Document, min: 1, max: Some(1) },
        Role { name: "CameraPath", kind: InputKind::Curve, min: 0, max: Some(1) },
        Role { name: "TargetPath", kind: InputKind::Curve, min: 0, max: Some(1) },
        Role { name: "Destination", kind: InputKind::Path, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Direction", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Frame Count", kind: ParameterKind::Number, required: false },
        Parameter { name: "FPS", kind: ParameterKind::Number, required: false },
        Parameter { name: "Filename", kind: ParameterKind::Text, required: false },
        Parameter { name: "Camera Path", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Target Path", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_render_020 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-RENDER-020",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Objects", kind: InputKind::Object, min: 0, max: None },
        Role { name: "CameraPaths", kind: InputKind::Object, min: 0, max: None },
        Role { name: "GeometryCurves", kind: InputKind::Curve, min: 0, max: None },
        Role { name: "Destination", kind: InputKind::Path, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Image Type", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Size Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Size Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Viewport", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Total Frames", kind: ParameterKind::Number, required: false },
        Parameter { name: "Save Name", kind: ParameterKind::Text, required: false },
        Parameter { name: "Save Format", kind: ParameterKind::Choice, required: false },
        Parameter { name: "SaveTo", kind: ParameterKind::Text, required: false },
        Parameter { name: "Record", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Extra Display Modes", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Camera Path", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Target Path", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Start Frame", kind: ParameterKind::Number, required: false },
        Parameter { name: "End Frame", kind: ParameterKind::Number, required: false },
        Parameter { name: "Ease", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Lens Sets/Enable", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Lens Sets", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Lens Tilts/Enable", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Lens Tilts", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Object/Operation", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Move/Enable", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Move/Path", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Move/Pivot Plane", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Move/Orient", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Object/Sets", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Repeat", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Rotate/Enable", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Rotate/Enable X", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Rotate/Enable Y", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Rotate/Enable Z", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Rotate X", kind: ParameterKind::Number, required: false },
        Parameter { name: "Rotate Y", kind: ParameterKind::Number, required: false },
        Parameter { name: "Rotate Z", kind: ParameterKind::Number, required: false },
        Parameter { name: "Rotate/Pivot", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Scale/Enabled", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Scale X", kind: ParameterKind::Number, required: false },
        Parameter { name: "Scale Y", kind: ParameterKind::Number, required: false },
        Parameter { name: "Scale Z", kind: ParameterKind::Number, required: false },
        Parameter { name: "Copies/Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Hide Original", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Copies/Mirror Enable", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Copies/Array Enable", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Mirror/Plane", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Copies/Pivot", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Array/Curve", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Array/Number", kind: ParameterKind::Number, required: false },
        Parameter { name: "Array/Slide", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Array/Spacing", kind: ParameterKind::Number, required: false },
        Parameter { name: "Geometry/Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Geometry/Enable", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Geometry/Material", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Keep", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Sweep/Curve Processing", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Sweep 1 x 1", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Sweep/Smart Trim", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Tween/Enable", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Tween/Profiles", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Unload/Enable", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Unload/Frame", kind: ParameterKind::Number, required: false },
        Parameter { name: "Pipe/Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Pipe/Radius 2", kind: ParameterKind::Number, required: false },
        Parameter { name: "Pipe/Cap", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Loft/Type", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_render_021 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-RENDER-021",
    effect: Effect::ExternalOutput,
    roles: &[
        Role { name: "Frames", kind: InputKind::Image, min: 1, max: None },
        Role { name: "Destination", kind: InputKind::Path, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "FPS", kind: ParameterKind::Number, required: false },
        Parameter { name: "Quality", kind: ParameterKind::Number, required: false },
        Parameter { name: "Movie Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Movie Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Repeat", kind: ParameterKind::Number, required: false },
        Parameter { name: "Dissolve Frames", kind: ParameterKind::Number, required: false },
        Parameter { name: "Watermark Use", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Watermark Image", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Watermark Position", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Watermark Size", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Opacity", kind: ParameterKind::Number, required: false },
        Parameter { name: "Fade In Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Fade In Pic", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Fade In Color", kind: ParameterKind::Text, required: false },
        Parameter { name: "Fade In Frames", kind: ParameterKind::Number, required: false },
        Parameter { name: "Fade Out Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Fade Out Pic", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Fade Out Color", kind: ParameterKind::Text, required: false },
        Parameter { name: "Fade Out Frames", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_render_022 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-RENDER-022",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Document", kind: InputKind::Document, min: 1, max: Some(1) },
        Role { name: "Images", kind: InputKind::Image, min: 0, max: None },
    ],
    parameters: &[
        Parameter { name: "Save Format", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Current Layer", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Layer/Visible", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Layer/Lock", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Text", kind: ParameterKind::Text, required: false },
        Parameter { name: "Font", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Font Style", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Font Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Add from Viewport/Shade Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Add from Viewport/Orientation", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Color/Hue", kind: ParameterKind::Number, required: false },
        Parameter { name: "Color/Saturation", kind: ParameterKind::Number, required: false },
        Parameter { name: "Color/Illumination", kind: ParameterKind::Number, required: false },
        Parameter { name: "Opacity", kind: ParameterKind::Number, required: false },
        Parameter { name: "Color/Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Location/Left", kind: ParameterKind::Number, required: false },
        Parameter { name: "Location/Top", kind: ParameterKind::Number, required: false },
        Parameter { name: "Location/Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Location/Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Location/Rotation", kind: ParameterKind::Number, required: false },
        Parameter { name: "Flip", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Skew X", kind: ParameterKind::Number, required: false },
        Parameter { name: "Skew Y", kind: ParameterKind::Number, required: false },
        Parameter { name: "Tile X", kind: ParameterKind::Number, required: false },
        Parameter { name: "Tile Y", kind: ParameterKind::Number, required: false },
        Parameter { name: "Filter", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Bevel", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Shadow/On", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Shadow/Color", kind: ParameterKind::Text, required: false },
        Parameter { name: "Shadow/Opacity", kind: ParameterKind::Number, required: false },
        Parameter { name: "Shadow/Offset X", kind: ParameterKind::Number, required: false },
        Parameter { name: "Shadow/Offset Y", kind: ParameterKind::Number, required: false },
        Parameter { name: "Shadow/Blur", kind: ParameterKind::Number, required: false },
        Parameter { name: "Wire Composite/Mode", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Wire Composite/Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Wire Composite/Blend", kind: ParameterKind::Number, required: false },
        Parameter { name: "Advanced/Modes", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Advanced/Position", kind: ParameterKind::Number, required: false },
        Parameter { name: "Advanced/Amount", kind: ParameterKind::Number, required: false },
        Parameter { name: "Advanced/Opacity", kind: ParameterKind::Number, required: false },
        Parameter { name: "Advanced/Combine", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Background", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Graphic", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Frame", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Theme", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_render_023 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-RENDER-023",
    effect: Effect::ExternalOutput,
    roles: &[
        Role { name: "Document", kind: InputKind::Document, min: 1, max: Some(1) },
        Role { name: "Destination", kind: InputKind::Path, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Materials", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Views", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_render_024 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-RENDER-024",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Document", kind: InputKind::Document, min: 1, max: Some(1) },
        Role { name: "Images", kind: InputKind::Image, min: 0, max: None },
    ],
    parameters: &[
        Parameter { name: "Layout Name", kind: ParameterKind::Text, required: false },
        Parameter { name: "Printer", kind: ParameterKind::Reference, required: false },
        Parameter { name: "View", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Page Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Page Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Template", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Template/Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Shade Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Detail", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Picture", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Background", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Frame", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Graphic", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_render_025 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-RENDER-025",
    effect: Effect::ExternalOutput,
    roles: &[
        Role { name: "Viewports", kind: InputKind::Viewport, min: 4, max: Some(4) },
        Role { name: "Destination", kind: InputKind::Path, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Capture Type", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Format", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Name", kind: ParameterKind::Text, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_render_026 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-RENDER-026",
    effect: Effect::ExternalOutput,
    roles: &[
        Role { name: "Models", kind: InputKind::Path, min: 1, max: None },
        Role { name: "Destination", kind: InputKind::Path, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Size Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Size Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Format", kind: ParameterKind::Choice, required: false },
        Parameter { name: "SaveTo", kind: ParameterKind::Text, required: false },
        Parameter { name: "View", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Save Channel", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "RunTime", kind: ParameterKind::Text, required: false },
        Parameter { name: "Models", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_render_027 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-RENDER-027",
    effect: Effect::ExternalOutput,
    roles: &[
        Role { name: "PNGs", kind: InputKind::Image, min: 1, max: None },
        Role { name: "Destination", kind: InputKind::Path, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Color", kind: ParameterKind::Text, required: false },
        Parameter { name: "Save Location", kind: ParameterKind::Text, required: false },
        Parameter { name: "Files To Fix", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_render_028 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-RENDER-028",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "Objects", kind: InputKind::Object, min: 1, max: None },
        Role { name: "Texture", kind: InputKind::Image, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "On", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Texture", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Mapping Channel", kind: ParameterKind::Number, required: false },
        Parameter { name: "Black Point", kind: ParameterKind::Number, required: false },
        Parameter { name: "White Point", kind: ParameterKind::Number, required: false },
        Parameter { name: "Initial Quality", kind: ParameterKind::Number, required: false },
        Parameter { name: "Max Faces", kind: ParameterKind::Number, required: false },
        Parameter { name: "Fairing", kind: ParameterKind::Number, required: false },
        Parameter { name: "Post Weld Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Mesh Memory Limit", kind: ParameterKind::Number, required: false },
        Parameter { name: "Refine Steps", kind: ParameterKind::Number, required: false },
        Parameter { name: "Refine Sensitivity", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_render_029 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-RENDER-029",
    effect: Effect::ExternalOutput,
    roles: &[
        Role { name: "RenderBuffer", kind: InputKind::Image, min: 1, max: Some(1) },
        Role { name: "Destination", kind: InputKind::Path, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Format", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Name", kind: ParameterKind::Text, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_001 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-001",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "Viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Edit Mode", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_002 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-002",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Selection", kind: InputKind::Selection, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Gumball Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Value", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_003 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-003",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "Selection", kind: InputKind::Selection, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Selection Mode", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_004 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-004",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "Viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Hot Keys", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Configuration", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Custom Configuration", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_005 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-005",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "Objects", kind: InputKind::Mesh, min: 0, max: None },
    ],
    parameters: &[
        Parameter { name: "PointsOn Surface", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Vertex Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Vertex Over Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Vertex Selected Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Edge Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Edge Over Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Edge Selected Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Over Opacity", kind: ParameterKind::Number, required: false },
        Parameter { name: "Selected Opacity", kind: ParameterKind::Number, required: false },
        Parameter { name: "Vertex Border", kind: ParameterKind::Text, required: false },
        Parameter { name: "Crease Edges", kind: ParameterKind::Text, required: false },
        Parameter { name: "Naked Edges", kind: ParameterKind::Text, required: false },
        Parameter { name: "Over Faces", kind: ParameterKind::Text, required: false },
        Parameter { name: "Selected Faces", kind: ParameterKind::Text, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_006 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-006",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Selection", kind: InputKind::Selection, min: 0, max: Some(1) },
        Role { name: "Location", kind: InputKind::Point, min: 0, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_007 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-007",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "Viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Visualization Mode", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_008 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-008",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Faces", kind: InputKind::Selection, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Distance", kind: ParameterKind::Number, required: false },
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_009 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-009",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Edges", kind: InputKind::Selection, min: 1, max: Some(1) },
        Role { name: "Location", kind: InputKind::Point, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "DeleteCentralEdge", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_010 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-010",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Selection", kind: InputKind::Selection, min: 1, max: Some(1) },
        Role { name: "Curve", kind: InputKind::Curve, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Segments", kind: ParameterKind::Number, required: false },
        Parameter { name: "Taper", kind: ParameterKind::Number, required: false },
        Parameter { name: "Curve", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Flip Curve", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_011 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-011",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Object", kind: InputKind::Mesh, min: 1, max: Some(1) },
        Role { name: "LinePoints", kind: InputKind::Point, min: 0, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "Plane", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Line Points", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_012 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-012",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Faces", kind: InputKind::Selection, min: 1, max: Some(1) },
        Role { name: "Curve", kind: InputKind::Curve, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Segments", kind: ParameterKind::Number, required: false },
        Parameter { name: "Curve", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Flip Curve", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Align", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_013 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-013",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Faces", kind: InputKind::Selection, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Distance", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_014 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-014",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Points", kind: InputKind::Point, min: 0, max: None },
        Role { name: "Edge", kind: InputKind::Selection, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Autoquad", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Weld", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Retopology", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Offset", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_015 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-015",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Curves", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Start Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "End Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Segments", kind: ParameterKind::Number, required: false },
        Parameter { name: "Global Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Global Segments", kind: ParameterKind::Number, required: false },
        Parameter { name: "Edit Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Distance", kind: ParameterKind::Number, required: false },
        Parameter { name: "Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Active Curve", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_016 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-016",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Gem", kind: InputKind::Gem, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Dome Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Seat Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Seat Length", kind: ParameterKind::Number, required: false },
        Parameter { name: "Seat Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Global Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bezel Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bottom Thickness", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_017 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-017",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Region", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Diameter", kind: ParameterKind::Number, required: false },
        Parameter { name: "Segments", kind: ParameterKind::Number, required: false },
        Parameter { name: "Outer Segments", kind: ParameterKind::Number, required: false },
        Parameter { name: "Inner Segments", kind: ParameterKind::Number, required: false },
        Parameter { name: "Left Segments", kind: ParameterKind::Number, required: false },
        Parameter { name: "Right Segments", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Middle Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Middle Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bottom Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bottom Height", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_018 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-018",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Region", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Diameter", kind: ParameterKind::Number, required: false },
        Parameter { name: "Seal Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Seal Depth", kind: ParameterKind::Number, required: false },
        Parameter { name: "Corner Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Corner Depth", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Shoulder Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Shoulder Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Middle Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Middle Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bottom Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bottom Thickness", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_019 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-019",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Selection", kind: InputKind::Selection, min: 1, max: Some(1) },
        Role { name: "PlanePoints", kind: InputKind::Point, min: 0, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "Plane", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Plane Definition", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_020 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-020",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Selection", kind: InputKind::Selection, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "CreaseEdges", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_021 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-021",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Selection", kind: InputKind::Selection, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_022 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-022",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Selection", kind: InputKind::Selection, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_023 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-023",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "SourceEdges", kind: InputKind::Selection, min: 1, max: Some(1) },
        Role { name: "Target", kind: InputKind::Object, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Weld", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Average", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Align", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Flip", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Domain 1", kind: ParameterKind::Number, required: false },
        Parameter { name: "Domain 2", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_024 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-024",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Object", kind: InputKind::Mesh, min: 1, max: Some(1) },
        Role { name: "PlanePoints", kind: InputKind::Point, min: 0, max: Some(2) },
        Role { name: "EdgeLoop", kind: InputKind::Selection, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Plane", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Threshold", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_025 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-025",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Vertices", kind: InputKind::Selection, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Average", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Weld", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Tolerance", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_026 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-026",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Size X", kind: ParameterKind::Number, required: false },
        Parameter { name: "Size Y", kind: ParameterKind::Number, required: false },
        Parameter { name: "Size Z", kind: ParameterKind::Number, required: false },
        Parameter { name: "Segments X", kind: ParameterKind::Number, required: false },
        Parameter { name: "Segments Y", kind: ParameterKind::Number, required: false },
        Parameter { name: "Segments Z", kind: ParameterKind::Number, required: false },
        Parameter { name: "Position", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_027 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-027",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Topology", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Sides", kind: ParameterKind::Number, required: false },
        Parameter { name: "Segments Z", kind: ParameterKind::Number, required: false },
        Parameter { name: "Position", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_028 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-028",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Segments", kind: ParameterKind::Number, required: false },
        Parameter { name: "Segments Z", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Cap", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Bottom Cap", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Position", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_029 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-029",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Segments", kind: ParameterKind::Number, required: false },
        Parameter { name: "Segments Z", kind: ParameterKind::Number, required: false },
        Parameter { name: "Cap", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Top Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Cap", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Position", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_030 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-030",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Size X", kind: ParameterKind::Number, required: false },
        Parameter { name: "Size Y", kind: ParameterKind::Number, required: false },
        Parameter { name: "Segments X", kind: ParameterKind::Number, required: false },
        Parameter { name: "Segments Y", kind: ParameterKind::Number, required: false },
        Parameter { name: "Position", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_031 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-031",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Rail", kind: InputKind::Curve, min: 1, max: Some(1) },
        Role { name: "Profiles", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Segments", kind: ParameterKind::Number, required: false },
        Parameter { name: "Segments Along", kind: ParameterKind::Number, required: false },
        Parameter { name: "Flip Rail", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Align Seams", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Curve Direction", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_032 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-032",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Rails", kind: InputKind::Curve, min: 2, max: Some(2) },
        Role { name: "Profiles", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Segments", kind: ParameterKind::Number, required: false },
        Parameter { name: "Segments Along", kind: ParameterKind::Number, required: false },
        Parameter { name: "Flip Rail", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Align Seams", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Curve Direction", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_033 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-033",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Curves", kind: InputKind::Curve, min: 2, max: None },
    ],
    parameters: &[
        Parameter { name: "Segments", kind: ParameterKind::Number, required: false },
        Parameter { name: "Segments Along", kind: ParameterKind::Number, required: false },
        Parameter { name: "Cap Start", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Cap End", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Direction", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Seam", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_034 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-034",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Wrapper", kind: InputKind::Mesh, min: 1, max: Some(1) },
        Role { name: "Target", kind: InputKind::Surface, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Iterations", kind: ParameterKind::Number, required: false },
        Parameter { name: "Auto Size", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Copy", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_035 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-035",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Curve", kind: InputKind::Curve, min: 1, max: Some(1) },
        Role { name: "AxisPoints", kind: InputKind::Point, min: 0, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "Axis", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Custom Axis", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Angle to Fill", kind: ParameterKind::Number, required: false },
        Parameter { name: "Segments", kind: ParameterKind::Number, required: false },
        Parameter { name: "Segments Along", kind: ParameterKind::Number, required: false },
        Parameter { name: "Cap Start", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Cap End", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_036 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-036",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Curve", kind: InputKind::Curve, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Distance", kind: ParameterKind::Number, required: false },
        Parameter { name: "Segments", kind: ParameterKind::Number, required: false },
        Parameter { name: "Segments Along", kind: ParameterKind::Number, required: false },
        Parameter { name: "Symmetry", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Cap Start", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Cap End", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_037 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-037",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Curves", kind: InputKind::Curve, min: 2, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "Distance", kind: ParameterKind::Number, required: false },
        Parameter { name: "Segments", kind: ParameterKind::Number, required: false },
        Parameter { name: "Segments Along", kind: ParameterKind::Number, required: false },
        Parameter { name: "Symmetry", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Flip Curve", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Cap Start", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Cap End", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_038 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-038",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Faces", kind: InputKind::Selection, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Copy", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_039 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-039",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Selection", kind: InputKind::Selection, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_040 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-040",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Object", kind: InputKind::Mesh, min: 1, max: Some(1) },
        Role { name: "Faces", kind: InputKind::Selection, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Distance", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_041 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-041",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Selection", kind: InputKind::Selection, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_042 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-042",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Surfaces", kind: InputKind::Surface, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Segments", kind: ParameterKind::Number, required: false },
        Parameter { name: "Segments Along", kind: ParameterKind::Number, required: false },
        Parameter { name: "DeleteInput", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_043 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-043",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "TSpline", kind: InputKind::Object, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_044 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-044",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "ClayObjects", kind: InputKind::Mesh, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Delete Original Object", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_045 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-045",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Mesh", kind: InputKind::Mesh, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_046 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-046",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "Selection", kind: InputKind::Selection, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Set Name", kind: ParameterKind::Text, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_047 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-047",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "Edges", kind: InputKind::Selection, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_048 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-048",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "Edges", kind: InputKind::Selection, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_049 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-049",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "Faces", kind: InputKind::Selection, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_050 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-050",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "Faces", kind: InputKind::Selection, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_051 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-051",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "Selection", kind: InputKind::Selection, min: 0, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_052 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-052",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "Document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_053 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-053",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "Document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_054 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-054",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "Document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_055 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-055",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "Selection", kind: InputKind::Selection, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_056 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-056",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "Selection", kind: InputKind::Selection, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_057 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-057",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "Edges", kind: InputKind::Selection, min: 0, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_058 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-058",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "Edges", kind: InputKind::Selection, min: 0, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_059 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-059",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "Edges", kind: InputKind::Selection, min: 0, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_060 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-060",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Thumbnail Size", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_061 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-061",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "Selection", kind: InputKind::Selection, min: 1, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_subd_062 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-062",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "Objects", kind: InputKind::Mesh, min: 0, max: None },
    ],
    parameters: &[
        Parameter { name: "Level", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_emboss_001 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-EMBOSS-001",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Document", kind: InputKind::Document, min: 1, max: Some(1) },
        Role { name: "Operations", kind: InputKind::Object, min: 0, max: None },
    ],
    parameters: &[
        Parameter { name: "Project Type", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Delete Base", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Cap Distance", kind: ParameterKind::Number, required: false },
        Parameter { name: "Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Workbench Position", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Resolution", kind: ParameterKind::Number, required: false },
        Parameter { name: "Display Material", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Show Bounding Box", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Assistant", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_emboss_002 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-EMBOSS-002",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "OperationCurve", kind: InputKind::Curve, min: 1, max: None },
        Role { name: "Profile", kind: InputKind::Curve, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Styles", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Change Color", kind: ParameterKind::Text, required: false },
        Parameter { name: "Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Starting Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Auto Smooth", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Delete Inside Region", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Profile", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_emboss_003 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-EMBOSS-003",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "OperationCurve", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Styles", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Change Color", kind: ParameterKind::Text, required: false },
        Parameter { name: "Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Starting Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Delete Inside Region", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_emboss_004 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-EMBOSS-004",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Geometry", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Styles", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Change Color", kind: ParameterKind::Text, required: false },
        Parameter { name: "Side", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_emboss_005 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-EMBOSS-005",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Profile", kind: InputKind::Curve, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Styles", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Change Color", kind: ParameterKind::Text, required: false },
        Parameter { name: "Side", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_emboss_006 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-EMBOSS-006",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Image", kind: InputKind::Image, min: 1, max: Some(1) },
        Role { name: "OperationCurve", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Styles", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Change Color", kind: ParameterKind::Text, required: false },
        Parameter { name: "Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Starting Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Rotation", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_emboss_007 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-EMBOSS-007",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Stamp", kind: InputKind::Image, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Styles", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Change Color", kind: ParameterKind::Text, required: false },
        Parameter { name: "Height", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_emboss_008 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-EMBOSS-008",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Image", kind: InputKind::Image, min: 1, max: Some(1) },
        Role { name: "OperationCurve", kind: InputKind::Curve, min: 0, max: None },
    ],
    parameters: &[
        Parameter { name: "Styles", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Change Color", kind: ParameterKind::Text, required: false },
        Parameter { name: "Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Starting Height", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_emboss_009 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-EMBOSS-009",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Image", kind: InputKind::Image, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Styles", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Change Color", kind: ParameterKind::Text, required: false },
        Parameter { name: "Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Width", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_emboss_010 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-EMBOSS-010",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Relief", kind: InputKind::Mesh, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Styles", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Change Color", kind: ParameterKind::Text, required: false },
        Parameter { name: "Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Accumulate", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_emboss_011 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-EMBOSS-011",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Relief", kind: InputKind::Mesh, min: 0, max: Some(1) },
        Role { name: "OperationCurve", kind: InputKind::Curve, min: 0, max: None },
    ],
    parameters: &[
        Parameter { name: "Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Change Color", kind: ParameterKind::Text, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_emboss_012 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-EMBOSS-012",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Object", kind: InputKind::Object, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Intensity", kind: ParameterKind::Number, required: false },
        Parameter { name: "Symmetry", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Negative", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Brush Type", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Hot Keys", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Resolution", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_emboss_013 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-EMBOSS-013",
    effect: Effect::ExternalOutput,
    roles: &[
        Role { name: "Geometry", kind: InputKind::Object, min: 1, max: None },
        Role { name: "Destination", kind: InputKind::Path, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Resolution X", kind: ParameterKind::Number, required: false },
        Parameter { name: "Resolution Y", kind: ParameterKind::Number, required: false },
        Parameter { name: "Constrain Proportions", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Side", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Location", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Name", kind: ParameterKind::Text, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_emboss_014 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-EMBOSS-014",
    effect: Effect::ExternalOutput,
    roles: &[
        Role { name: "Resource", kind: InputKind::Path, min: 0, max: Some(1) },
        Role { name: "Profile", kind: InputKind::Curve, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Resource Type", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_emboss_015 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-EMBOSS-015",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Mesh", kind: InputKind::Mesh, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Percentage", kind: ParameterKind::Number, required: false },
        Parameter { name: "Error", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Display", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Set View", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_matrixtools_001 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-MATRIXTOOLS-001",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "target_surface", kind: InputKind::Surface, min: 1, max: Some(1) },
        Role { name: "pattern_objects", kind: InputKind::Object, min: 1, max: None },
        Role { name: "attractor_curves", kind: InputKind::Curve, min: 0, max: None },
    ],
    parameters: &[
        Parameter { name: "PatternType", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Columns", kind: ParameterKind::Number, required: false },
        Parameter { name: "Rows", kind: ParameterKind::Number, required: false },
        Parameter { name: "ZOffset", kind: ParameterKind::Number, required: false },
        Parameter { name: "ScaleX", kind: ParameterKind::Number, required: false },
        Parameter { name: "ScaleY", kind: ParameterKind::Number, required: false },
        Parameter { name: "ScaleZ", kind: ParameterKind::Number, required: false },
        Parameter { name: "RotateX", kind: ParameterKind::Number, required: false },
        Parameter { name: "RotateY", kind: ParameterKind::Number, required: false },
        Parameter { name: "RotateZ", kind: ParameterKind::Number, required: false },
        Parameter { name: "GroupOutput", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "OutputMesh", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Flip", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Towards", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Attractor", kind: ParameterKind::Number, required: false },
        Parameter { name: "SkipColumns", kind: ParameterKind::Number, required: false },
        Parameter { name: "SkipRows", kind: ParameterKind::Number, required: false },
        Parameter { name: "StartColumn", kind: ParameterKind::Number, required: false },
        Parameter { name: "StartRow", kind: ParameterKind::Number, required: false },
        Parameter { name: "PatternScaleU", kind: ParameterKind::Number, required: false },
        Parameter { name: "PatternScaleV", kind: ParameterKind::Number, required: false },
        Parameter { name: "PatternMoveU", kind: ParameterKind::Number, required: false },
        Parameter { name: "PatternMoveV", kind: ParameterKind::Number, required: false },
        Parameter { name: "Distance", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Direction", kind: ParameterKind::Choice, required: false },
        Parameter { name: "AutoTrim", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Relax", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_matrixtools_002 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-MATRIXTOOLS-002",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "target_surface", kind: InputKind::Surface, min: 1, max: Some(1) },
        Role { name: "texture_image", kind: InputKind::Image, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Resolution", kind: ParameterKind::Choice, required: false },
        Parameter { name: "TileX", kind: ParameterKind::Number, required: false },
        Parameter { name: "TileY", kind: ParameterKind::Number, required: false },
        Parameter { name: "Rotation", kind: ParameterKind::Number, required: false },
        Parameter { name: "BlackHeight", kind: ParameterKind::Number, required: false },
        Parameter { name: "WhiteHeight", kind: ParameterKind::Number, required: false },
        Parameter { name: "OffsetX", kind: ParameterKind::Number, required: false },
        Parameter { name: "OffsetY", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_matrixtools_003 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-MATRIXTOOLS-003",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "guide_curve", kind: InputKind::Curve, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "ThreadSize", kind: ParameterKind::Number, required: false },
        Parameter { name: "Threads", kind: ParameterKind::Number, required: false },
        Parameter { name: "Twists", kind: ParameterKind::Number, required: false },
        Parameter { name: "ThreadSpacing", kind: ParameterKind::Number, required: false },
        Parameter { name: "CapType", kind: ParameterKind::Choice, required: false },
        Parameter { name: "GroupOutput", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_matrixtools_004 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-MATRIXTOOLS-004",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "guide_curve", kind: InputKind::Curve, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Text", kind: ParameterKind::Text, required: true },
        Parameter { name: "Font", kind: ParameterKind::Reference, required: true },
        Parameter { name: "Alignment", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Orientation", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Bold", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Italic", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "MirrorLeftRight", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "MirrorTopBottom", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "TextHeight", kind: ParameterKind::Number, required: false },
        Parameter { name: "TextSpacing", kind: ParameterKind::Number, required: false },
        Parameter { name: "Extrusion", kind: ParameterKind::Number, required: false },
        Parameter { name: "Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "BothSides", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Start", kind: ParameterKind::Reference, required: false },
        Parameter { name: "End", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_matrixtools_005 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-MATRIXTOOLS-005",
    effect: Effect::ExternalOutput,
    roles: &[
        Role { name: "builder_session", kind: InputKind::Selection, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Action", kind: ParameterKind::Choice, required: true },
        Parameter { name: "Style", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Category", kind: ParameterKind::Text, required: false },
        Parameter { name: "Description", kind: ParameterKind::Text, required: false },
        Parameter { name: "PreviewImage", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_matrixtools_006 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-MATRIXTOOLS-006",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "library_location", kind: InputKind::Path, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Action", kind: ParameterKind::Choice, required: true },
        Parameter { name: "Model", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Category", kind: ParameterKind::Text, required: false },
        Parameter { name: "ThumbnailSize", kind: ParameterKind::Number, required: false },
        Parameter { name: "GemFilter", kind: ParameterKind::Text, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_matrixtools_007 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-MATRIXTOOLS-007",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "top_image", kind: InputKind::Image, min: 0, max: Some(1) },
        Role { name: "front_image", kind: InputKind::Image, min: 0, max: Some(1) },
        Role { name: "side_image", kind: InputKind::Image, min: 0, max: Some(1) },
        Role { name: "back_image", kind: InputKind::Image, min: 0, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
pub mod om9_matrixtools_008 {
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-MATRIXTOOLS-008",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Action", kind: ParameterKind::Choice, required: true },
        Parameter { name: "Name", kind: ParameterKind::Text, required: false },
        Parameter { name: "SavedView", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
}
