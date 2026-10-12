//! RCORE-04 / OM9-BUILDERCORE-001: owned options, input phases and proposals.
//! A proposal becomes accepted state only after the native consumer approves it.
use std::collections::{BTreeMap, BTreeSet};

pub type OptionKey = String;
pub type OptionError = String;
pub fn command_invocation_line(name: &str) -> Result<String, OptionError> {
    let name = name.trim().trim_start_matches('_');
    if name.is_empty() || name.len() > 128 || name.chars().any(char::is_control) {
        return Err("Invalid command name".into());
    }
    Ok(format!("Command: _{name}"))
}
#[derive(Clone, Debug, PartialEq)]
pub enum OptionValue {
    Toggle(bool),
    Choice(String),
    Number(f64),
}
impl OptionValue {
    pub fn text(&self) -> String {
        match self {
            Self::Toggle(v) => if *v { "Yes" } else { "No" }.into(),
            Self::Choice(v) => v.clone(),
            Self::Number(v) => v.to_string(),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OptionKind {
    Toggle,
    Choice,
    Number,
    Action,
    NativeInput,
}
#[derive(Clone, Debug)]
pub struct Choice {
    pub value: String,
    pub aliases: Vec<String>,
    pub enabled: bool,
    pub reason: String,
    pub child: Option<String>,
}
impl Choice {
    pub fn new(value: &str) -> Self {
        Self {
            value: value.into(),
            aliases: Vec::new(),
            enabled: true,
            reason: String::new(),
            child: None,
        }
    }
    pub fn disabled(mut self, reason: &str) -> Self {
        self.enabled = false;
        self.reason = reason.into();
        self
    }
    pub fn alias(mut self, alias: &str) -> Self {
        self.aliases.push(alias.into());
        self
    }
    pub fn child(mut self, key: &str) -> Self {
        self.child = Some(key.into());
        self
    }
}
#[derive(Clone, Debug)]
pub struct OptionSpec {
    pub key: String,
    pub aliases: Vec<String>,
    pub parent: Option<String>,
    pub kind: OptionKind,
    pub choices: Vec<Choice>,
    pub minimum: f64,
    pub maximum: f64,
    pub unit: String,
    pub integer: bool,
    pub scope: u32,
    pub shown: bool,
    pub enabled: bool,
    pub reason: String,
}
impl OptionSpec {
    fn base(key: &str, kind: OptionKind) -> Self {
        Self {
            key: key.into(),
            aliases: Vec::new(),
            parent: None,
            kind,
            choices: Vec::new(),
            minimum: 0.,
            maximum: 0.,
            unit: String::new(),
            integer: false,
            scope: 3,
            shown: true,
            enabled: true,
            reason: String::new(),
        }
    }
    pub fn toggle(key: &str) -> Self {
        Self::base(key, OptionKind::Toggle)
    }
    pub fn choice(key: &str, choices: Vec<Choice>) -> Self {
        let mut s = Self::base(key, OptionKind::Choice);
        s.choices = choices;
        s
    }
    pub fn number(key: &str, min: f64, max: f64, unit: &str, integer: bool) -> Self {
        let mut s = Self::base(key, OptionKind::Number);
        s.minimum = min;
        s.maximum = max;
        s.unit = unit.into();
        s.integer = integer;
        s
    }
    pub fn action(key: &str) -> Self {
        Self::base(key, OptionKind::Action)
    }
    pub fn native_input(key: &str) -> Self {
        Self::base(key, OptionKind::NativeInput)
    }
    pub fn alias(mut self, value: &str) -> Self {
        self.aliases.push(value.into());
        self
    }
    pub fn parent(mut self, value: &str) -> Self {
        self.parent = Some(value.into());
        self
    }
    pub fn disabled(mut self, reason: &str) -> Self {
        self.enabled = false;
        self.reason = reason.into();
        self
    }
    pub fn scope(mut self, scope: u32) -> Self {
        self.scope = scope;
        self
    }
    pub fn hidden(mut self) -> Self {
        self.shown = false;
        self
    }
}
#[derive(Clone, Debug)]
pub struct OptionContext {
    pub generation: u64,
    pub mm_per_unit: f64,
    pub feature_id: String,
}
impl OptionContext {
    pub fn new(generation: u64, mm_per_unit: f64) -> Self {
        Self {
            generation,
            mm_per_unit,
            feature_id: "RCORE-04".into(),
        }
    }
    pub fn feature(mut self, id: &str) -> Self {
        self.feature_id = id.into();
        self
    }
}
#[derive(Clone, Debug)]
pub enum Constraint {
    GreaterThan {
        higher: String,
        lower: String,
        message: String,
    },
    ForbiddenPair {
        first: String,
        first_value: OptionValue,
        second: String,
        second_value: OptionValue,
        message: String,
    },
}
#[derive(Clone, Debug)]
pub enum OptionEvent {
    Activate(OptionKey),
    Submit(String),
    Set(OptionKey, OptionValue),
    SetText(OptionKey, String),
    Escape,
    Select(i32),
}
#[derive(Clone, Debug)]
pub struct Candidate {
    pub generation: u64,
    pub revision: u64,
    pub input_epoch: u64,
    pub values: BTreeMap<OptionKey, OptionValue>,
    pub changes: Vec<(OptionKey, OptionValue)>,
    pub clear_cursor: bool,
}
#[derive(Clone, Debug)]
pub enum OptionOutcome {
    Unhandled,
    Navigated,
    Candidate(Candidate),
    Action(String),
}
#[derive(Clone, Debug)]
pub struct PromptItem {
    pub key: String,
    pub input: String,
    pub start: usize,
    pub length: usize,
    pub enabled: bool,
    pub reason: String,
    pub mnemonic: String,
}
#[derive(Clone, Debug)]
pub struct OptionSnapshot {
    pub text: String,
    pub question_length: usize,
    pub items: Vec<PromptItem>,
    pub pending: Option<String>,
    pub generation: u64,
    pub revision: u64,
    pub selected: Option<usize>,
}
#[derive(Clone, Debug)]
struct Cursor {
    key: String,
    selected: Option<usize>,
    staged: Option<(String, OptionValue)>,
}
#[derive(Clone, Debug)]
pub struct OptionSession {
    pub schema: Vec<OptionSpec>,
    values: BTreeMap<OptionKey, OptionValue>,
    context: OptionContext,
    revision: u64,
    input_epoch: u64,
    stack: Vec<Cursor>,
    draft: String,
    question: String,
    pub rules: Vec<Constraint>,
}
impl OptionSession {
    pub fn new(
        schema: Vec<OptionSpec>,
        values: BTreeMap<OptionKey, OptionValue>,
        context: OptionContext,
    ) -> Result<Self, OptionError> {
        if context.generation == 0
            || !context.mm_per_unit.is_finite()
            || context.mm_per_unit <= 0.
            || schema.len() > 128
        {
            return Err("Invalid option context/schema".into());
        }
        let keys: BTreeSet<_> = schema.iter().map(|s| s.key.as_str()).collect();
        let canonical_keys: BTreeSet<_> =
            schema.iter().map(|s| s.key.to_ascii_lowercase()).collect();
        if canonical_keys.len() != schema.len()
            || schema.iter().any(|s| {
                s.key.is_empty() || s.key.len() > 128 || s.key.contains(['=', '\n', '\r', '\0'])
            })
        {
            return Err("Invalid or duplicate option identity".into());
        }
        for spec in &schema {
            if spec
                .parent
                .as_ref()
                .is_some_and(|p| !keys.contains(p.as_str()))
            {
                return Err("Unknown parent option".into());
            }
            if spec.kind == OptionKind::Number
                && (!spec.minimum.is_finite()
                    || !spec.maximum.is_finite()
                    || spec.minimum > spec.maximum)
            {
                return Err("Invalid number domain".into());
            }
            if spec.kind == OptionKind::Choice {
                let mut names = BTreeSet::new();
                for c in &spec.choices {
                    if c.value.is_empty()
                        || !names.insert(c.value.to_ascii_lowercase())
                        || c.child.as_ref().is_some_and(|k| !keys.contains(k.as_str()))
                    {
                        return Err("Invalid choice/child identity".into());
                    }
                    if let Some(child) = &c.child {
                        if schema
                            .iter()
                            .find(|s| s.key == *child)
                            .and_then(|s| s.parent.as_deref())
                            != Some(spec.key.as_str())
                        {
                            return Err("Choice child must declare its owning parent".into());
                        }
                    }
                }
            }
            // Bound depth, reject cycles before any UI can enter a child.
            let mut parent = spec.parent.as_deref();
            let mut seen = BTreeSet::from([spec.key.as_str()]);
            while let Some(key) = parent {
                if !seen.insert(key) || seen.len() > 16 {
                    return Err("Cyclic/deep option hierarchy".into());
                }
                parent = schema
                    .iter()
                    .find(|s| s.key == key)
                    .and_then(|s| s.parent.as_deref());
            }
        }
        let session = Self {
            schema,
            values,
            context,
            revision: 0,
            input_epoch: 0,
            stack: Vec::new(),
            draft: String::new(),
            question: String::new(),
            rules: Vec::new(),
        };
        for (key, value) in &session.values {
            session.validate(key, value, false)?;
        }
        Ok(session)
    }
    pub fn values(&self) -> &BTreeMap<OptionKey, OptionValue> {
        &self.values
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn context_changed(&mut self) -> Result<(), OptionError> {
        self.revision = self
            .revision
            .checked_add(1)
            .ok_or("Option revision exhausted")?;
        Ok(())
    }
    pub fn generation(&self) -> u64 {
        self.context.generation
    }
    pub fn feature_id(&self) -> &str {
        &self.context.feature_id
    }
    pub fn unit_scale(&self) -> f64 {
        self.context.mm_per_unit
    }
    pub fn question(&self) -> &str {
        &self.question
    }
    pub fn set_question(&mut self, question: &str) -> Result<(), OptionError> {
        if question.len() > 1024 || question.contains(['\0', '\n', '\r']) {
            return Err("Invalid option question".into());
        }
        if self.question != question {
            self.context_changed()?;
            self.question = question.into();
        }
        Ok(())
    }
    pub fn set_shown(&mut self, key: &str, shown: bool) -> Result<(), OptionError> {
        let spec = self
            .schema
            .iter_mut()
            .find(|s| s.key == key)
            .ok_or("Unknown option")?;
        if spec.shown != shown {
            spec.shown = shown;
            self.context_changed()?;
        }
        Ok(())
    }
    pub fn refresh_presentation(&mut self, mut model: Self) -> Result<(), OptionError> {
        if self.context.generation != model.context.generation
            || self.feature_id() != model.feature_id()
        {
            return Err("Stale option presentation".into());
        }
        model.revision = self
            .revision
            .checked_add(1)
            .ok_or("Option revision exhausted")?;
        model.input_epoch = self
            .input_epoch
            .checked_add(1)
            .ok_or("Option input epoch exhausted")?;
        model.draft = self.draft.clone();
        *self = model;
        Ok(())
    }
    pub fn initial_value(&mut self, key: String, value: OptionValue) {
        self.values.insert(key, value);
    }
    pub fn synchronize(
        &mut self,
        values: BTreeMap<OptionKey, OptionValue>,
    ) -> Result<(), OptionError> {
        for (key, value) in &values {
            self.validate(key, value, false)?;
        }
        self.combination(&values)?;
        if self.values != values {
            self.revision = self
                .revision
                .checked_add(1)
                .ok_or("Option revision exhausted")?;
            self.values = values;
        }
        Ok(())
    }
    pub fn seal(&mut self) -> Result<(), OptionError> {
        let mut checked = Self::new(
            self.schema.clone(),
            self.values.clone(),
            self.context.clone(),
        )?;
        for rule in self.rules.clone() {
            checked.add_rule(rule)?;
        }
        checked.question = self.question.clone();
        *self = checked;
        Ok(())
    }
    pub fn add_rule(&mut self, rule: Constraint) -> Result<(), OptionError> {
        if self.rules.len() >= 128 {
            return Err("Too many option constraints".into());
        }
        match &rule {
            Constraint::GreaterThan { higher, lower, .. } => {
                if self.spec(higher)?.kind != OptionKind::Number
                    || self.spec(lower)?.kind != OptionKind::Number
                {
                    return Err("Numeric rule requires numeric options".into());
                }
            }
            Constraint::ForbiddenPair {
                first,
                first_value,
                second,
                second_value,
                ..
            } => {
                self.validate(first, first_value, false)?;
                self.validate(second, second_value, false)?;
            }
        }
        self.rules.push(rule);
        if let Err(error) = self.combination(&self.values) {
            self.rules.pop();
            return Err(error);
        }
        Ok(())
    }
    pub fn pending_key(&self) -> Option<&str> {
        self.stack.last().map(|c| c.key.as_str())
    }
    pub fn draft(&self) -> &str {
        &self.draft
    }
    pub fn set_draft(&mut self, draft: String) {
        if self.draft != draft {
            // Exhaustion retires every candidate permanently rather than wrapping.
            self.input_epoch = self.input_epoch.saturating_add(1);
        }
        self.draft = draft;
    }
    pub fn set_gate(&mut self, key: &str, enabled: bool, reason: &str) -> Result<(), OptionError> {
        let spec = self
            .schema
            .iter_mut()
            .find(|s| s.key == key)
            .ok_or("Unknown option")?;
        if spec.enabled != enabled || spec.reason != reason {
            self.revision = self
                .revision
                .checked_add(1)
                .ok_or("Option revision exhausted")?;
        }
        spec.enabled = enabled;
        spec.reason = reason.into();
        Ok(())
    }
    fn spec(&self, key: &str) -> Result<&OptionSpec, OptionError> {
        self.schema
            .iter()
            .find(|s| s.key == key)
            .ok_or_else(|| format!("Unknown option: {key}"))
    }
    fn resolve(&self, text: &str) -> Result<Option<String>, OptionError> {
        // Exact original name wins over ambiguous mnemonics.
        if let Some(s) = self
            .schema
            .iter()
            .find(|s| s.key.eq_ignore_ascii_case(text))
        {
            return Ok(Some(s.key.clone()));
        }
        let matches: Vec<_> = self
            .schema
            .iter()
            .filter(|s| s.aliases.iter().any(|a| a.eq_ignore_ascii_case(text)))
            .collect();
        let visible: Vec<_> = matches
            .iter()
            .filter(|s| s.enabled && s.scope & 1 != 0 && self.visible(s))
            .collect();
        match visible.as_slice() {
            [s] => return Ok(Some(s.key.clone())),
            [] => {}
            _ => return Err("Ambiguous option; enter the full name".into()),
        }
        match matches.as_slice() {
            [] => Ok(None),
            [s] => Ok(Some(s.key.clone())),
            _ => Err("Ambiguous option; enter the full name".into()),
        }
    }
    fn validate(&self, key: &str, value: &OptionValue, gate: bool) -> Result<(), OptionError> {
        let s = self.spec(key)?;
        if gate && !s.enabled {
            return Err(format!("{}: {}", s.key, s.reason));
        }
        match (s.kind, value) {
            (OptionKind::Toggle, OptionValue::Toggle(_)) => Ok(()),
            (OptionKind::Choice, OptionValue::Choice(v)) => {
                let c = s
                    .choices
                    .iter()
                    .find(|c| c.value == *v)
                    .ok_or("Unknown choice")?;
                if gate && !c.enabled {
                    Err(c.reason.clone())
                } else {
                    Ok(())
                }
            }
            (OptionKind::Number, OptionValue::Number(v))
                if v.is_finite()
                    && *v >= s.minimum
                    && *v <= s.maximum
                    && (!s.integer || v.fract() == 0.) =>
            {
                Ok(())
            }
            _ => Err(format!(
                "Invalid {} value; expected {:?}{}",
                s.key,
                s.kind,
                if s.unit.is_empty() {
                    String::new()
                } else {
                    format!(" ({})", s.unit)
                }
            )),
        }
    }
    fn proposal(&self, key: &str, value: OptionValue) -> Result<OptionOutcome, OptionError> {
        let completing = self.pending_key() == Some(key);
        self.proposal_from(key, value, completing, completing || self.stack.is_empty())
    }
    fn proposal_from(
        &self,
        key: &str,
        value: OptionValue,
        staged: bool,
        clear_cursor: bool,
    ) -> Result<OptionOutcome, OptionError> {
        self.validate(key, &value, true)?;
        let mut values = self.values.clone();
        let mut changes = Vec::new();
        if staged {
            for c in &self.stack {
                if let Some((key, value)) = &c.staged {
                    self.validate(key, value, true)?;
                    values.insert(key.clone(), value.clone());
                    changes.push((key.clone(), value.clone()));
                }
            }
        }
        values.insert(key.into(), value.clone());
        changes.push((key.into(), value));
        self.combination(&values)?;
        Ok(OptionOutcome::Candidate(Candidate {
            generation: self.context.generation,
            revision: self.revision,
            input_epoch: self.input_epoch,
            values,
            changes,
            clear_cursor,
        }))
    }
    fn combination(&self, values: &BTreeMap<String, OptionValue>) -> Result<(), OptionError> {
        for rule in &self.rules {
            match rule {
                Constraint::GreaterThan {
                    higher,
                    lower,
                    message,
                } => {
                    if let (Some(OptionValue::Number(a)), Some(OptionValue::Number(b))) =
                        (values.get(higher), values.get(lower))
                    {
                        if a <= b {
                            return Err(message.clone());
                        }
                    }
                }
                Constraint::ForbiddenPair {
                    first,
                    first_value,
                    second,
                    second_value,
                    message,
                } => {
                    if values.get(first) == Some(first_value)
                        && values.get(second) == Some(second_value)
                    {
                        return Err(message.clone());
                    }
                }
            }
        }
        Ok(())
    }
    pub fn accept(&mut self, candidate: Candidate) -> Result<(), OptionError> {
        if candidate.generation != self.context.generation
            || candidate.revision != self.revision
            || candidate.input_epoch != self.input_epoch
            || self.input_epoch == u64::MAX
        {
            return Err("Stale option candidate".into());
        }
        let mut expected = self.values.clone();
        for (key, value) in &candidate.changes {
            self.validate(key, value, true)?;
            expected.insert(key.clone(), value.clone());
        }
        if expected != candidate.values {
            return Err("Malformed option candidate".into());
        }
        for (key, value) in &candidate.values {
            self.validate(key, value, candidate.changes.iter().any(|(k, _)| k == key))?;
        }
        self.combination(&candidate.values)?;
        self.input_epoch = self
            .input_epoch
            .checked_add(1)
            .ok_or("Option input epoch exhausted")?;
        if candidate.values != self.values {
            self.revision = self
                .revision
                .checked_add(1)
                .ok_or("Option revision exhausted")?;
        }
        self.values = candidate.values;
        if candidate.clear_cursor {
            self.stack.clear();
            self.draft.clear();
        }
        Ok(())
    }
    fn activate(&mut self, key: &str) -> Result<OptionOutcome, OptionError> {
        let s = self.spec(key)?.clone();
        if s.scope & 1 == 0 {
            return Err("This option is only available in Builder controls".into());
        }
        if !s.enabled {
            return Err(format!("{}: {}", s.key, s.reason));
        }
        match s.kind {
            OptionKind::Toggle => match self.values.get(key) {
                Some(OptionValue::Toggle(value)) => self.proposal(key, OptionValue::Toggle(!value)),
                _ => Err("No current value; enter an explicit Yes/No value".into()),
            },
            OptionKind::Action | OptionKind::NativeInput => Ok(OptionOutcome::Action(key.into())),
            _ => {
                self.stack.clear();
                self.stack.push(Cursor {
                    key: key.into(),
                    selected: None,
                    staged: None,
                });
                self.draft.clear();
                Ok(OptionOutcome::Navigated)
            }
        }
    }
    fn choose(&mut self, key: &str, value: &str) -> Result<OptionOutcome, OptionError> {
        let spec = self.spec(key)?.clone();
        if spec.scope & 1 == 0 {
            return Err("This option is only available in Builder controls".into());
        }
        if !spec.enabled {
            return Err(spec.reason);
        }
        let choice = self
            .resolve_choice(&spec, value)?
            .ok_or("Unknown choice")?
            .clone();
        if let Some(reason) = self.cmd_choice_error(&choice) {
            return Err(reason);
        }
        if let Some(child) = choice.child {
            // A newly named branch replaces navigation; it never inherits an
            // unrelated unfinished branch's staged ancestors.
            if self.pending_key() != Some(key) {
                self.stack.clear();
                self.stack.push(Cursor {
                    key: key.into(),
                    selected: None,
                    staged: None,
                });
            }
            if self.stack.len() >= 16 {
                return Err("Option hierarchy too deep".into());
            }
            let s = self.spec(&child)?;
            if !s.enabled {
                return Err(s.reason.clone());
            }
            if self.stack.iter().any(|c| c.key == child) {
                return Err("Cyclic option choices".into());
            }
            self.stack.last_mut().unwrap().staged =
                Some((key.into(), OptionValue::Choice(choice.value)));
            self.stack.push(Cursor {
                key: child,
                selected: None,
                staged: None,
            });
            self.draft.clear();
            Ok(OptionOutcome::Navigated)
        } else {
            self.proposal(key, OptionValue::Choice(choice.value))
        }
    }
    fn cmd_choice_error(&self, choice: &Choice) -> Option<String> {
        if !choice.enabled {
            return Some(choice.reason.clone());
        }
        if let Some(child) = &choice.child {
            let spec = match self.spec(child) {
                Ok(spec) => spec,
                Err(error) => return Some(error),
            };
            if spec.scope & 1 == 0 {
                return Some("This choice requires a Builder-only option".into());
            }
            if !spec.enabled {
                return Some(spec.reason.clone());
            }
        }
        None
    }
    fn parse(&self, key: &str, text: &str) -> Result<OptionValue, OptionError> {
        self.parse_with_scale(key, text, self.context.mm_per_unit)
    }
    fn parse_with_scale(
        &self,
        key: &str,
        text: &str,
        scale: f64,
    ) -> Result<OptionValue, OptionError> {
        let s = self.spec(key)?;
        let value = match s.kind {
            OptionKind::Toggle => OptionValue::Toggle(match text.to_ascii_lowercase().as_str() {
                "yes" | "y" | "on" | "true" | "1" => true,
                "no" | "n" | "off" | "false" | "0" => false,
                _ => return Err("Enter Yes/No or On/Off".into()),
            }),
            OptionKind::Choice => OptionValue::Choice(
                s.choices
                    .iter()
                    .find(|c| c.value.eq_ignore_ascii_case(text))
                    .ok_or("Unknown choice")?
                    .value
                    .clone(),
            ),
            OptionKind::Number => OptionValue::Number(if s.unit == "mm" {
                crate::units::parse_length(text, scale).map_err(|e| e.to_string())?
            } else {
                text.parse::<f64>().map_err(|_| "Enter a number")?
            }),
            OptionKind::Action | OptionKind::NativeInput => {
                return Err("Action takes no value".into());
            }
        };
        self.validate(key, &value, true)?;
        Ok(value)
    }
    pub fn dispatch(&mut self, event: OptionEvent) -> Result<OptionOutcome, OptionError> {
        // Accepted revision describes parameters; this epoch describes input
        // admission, including cancellation, navigation and rejected input.
        self.input_epoch = self
            .input_epoch
            .checked_add(1)
            .ok_or("Option input epoch exhausted")?;
        match event {
            OptionEvent::Escape => {
                if self.stack.pop().is_none() {
                    return Ok(OptionOutcome::Unhandled);
                }
                if let Some(parent) = self.stack.last_mut() {
                    parent.staged = None;
                    parent.selected = None;
                }
                self.draft.clear();
                Ok(OptionOutcome::Navigated)
            }
            OptionEvent::Set(key, value) => {
                if self.spec(&key)?.scope & 2 == 0 {
                    return Err("This option is only available in CMD".into());
                }
                let drafted = self
                    .draft
                    .split_once('=')
                    .and_then(|(name, _)| self.resolve(name.trim()).ok().flatten());
                if !self.draft.is_empty()
                    && (self.pending_key() == Some(key.as_str())
                        || drafted.as_deref() == Some(key.as_str()))
                {
                    return Err("Accept or cancel the CMD draft first".into());
                }
                self.proposal_from(
                    &key,
                    value,
                    false,
                    self.draft.is_empty()
                        && (self.pending_key().is_none()
                            || self.pending_key() == Some(key.as_str())),
                )
            }
            OptionEvent::SetText(key, text) => {
                let value = self.parse_with_scale(&key, text.trim(), 1.)?;
                self.dispatch(OptionEvent::Set(key, value))
            }
            OptionEvent::Activate(key) => {
                if !self.draft.is_empty() {
                    return Err("Accept or cancel the draft first".into());
                }
                if key.contains('=') {
                    self.submit(&key)
                } else {
                    self.activate(&key)
                }
            }
            OptionEvent::Submit(text) => self.submit(text.trim()),
            OptionEvent::Select(direction) => {
                let Some(cursor) = self.stack.last() else {
                    return Ok(OptionOutcome::Unhandled);
                };
                let spec = self.spec(&cursor.key)?;
                if spec.kind != OptionKind::Choice {
                    return Ok(OptionOutcome::Navigated);
                }
                let valid: Vec<_> = spec
                    .choices
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| self.cmd_choice_error(c).is_none())
                    .map(|(i, _)| i)
                    .collect();
                if valid.is_empty() {
                    return Err("No available choice".into());
                }
                let index = cursor
                    .selected
                    .and_then(|i| valid.iter().position(|v| *v == i));
                let next = match index {
                    None => {
                        if direction < 0 {
                            valid.len() - 1
                        } else {
                            0
                        }
                    }
                    Some(i) => {
                        if direction < 0 {
                            (i + valid.len() - 1) % valid.len()
                        } else {
                            (i + 1) % valid.len()
                        }
                    }
                };
                self.stack.last_mut().unwrap().selected = Some(valid[next]);
                Ok(OptionOutcome::Navigated)
            }
        }
    }
    fn submit(&mut self, text: &str) -> Result<OptionOutcome, OptionError> {
        if text.len() > 1024 || text.contains(['\0', '\n', '\r']) {
            return Err("Invalid/oversize option input".into());
        }
        if self.pending_key().is_some()
            && (text.eq_ignore_ascii_case("Esc") || text.eq_ignore_ascii_case("Cancel"))
        {
            return self.dispatch(OptionEvent::Escape);
        }
        if let Some((name, value)) = text.split_once('=') {
            if let Some(key) = self.resolve(name.trim())? {
                if self.spec(&key)?.scope & 1 == 0 {
                    return Err("This option is only available in Builder controls".into());
                }
                if self.spec(&key)?.kind == OptionKind::NativeInput {
                    if !self.spec(&key)?.enabled {
                        return Err(self.spec(&key)?.reason.clone());
                    }
                    return Ok(OptionOutcome::Action(format!("{key}={}", value.trim())));
                }
                if self.spec(&key)?.kind == OptionKind::Choice {
                    return self.choose(&key, value.trim());
                }
                return self.proposal(&key, self.parse(&key, value.trim())?);
            }
        }
        if let Some(key) = self.pending_key().map(str::to_owned) {
            let spec = self.spec(&key)?;
            if spec.kind == OptionKind::Choice
                && (text.is_empty() || self.resolve_choice(spec, text)?.is_some())
            {
                let chosen = if text.is_empty() {
                    self.stack
                        .last()
                        .and_then(|c| c.selected)
                        .map(|i| spec.choices[i].value.clone())
                        .ok_or("Select a choice first")?
                } else {
                    text.into()
                };
                return self.choose(&key, &chosen);
            }
        }
        if let Some(key) = self.resolve(text)? {
            return self.activate(&key);
        }
        if let Some(key) = self.pending_key().map(str::to_owned) {
            let spec = self.spec(&key)?.clone();
            if spec.scope & 1 == 0 {
                return Err("This option is only available in Builder controls".into());
            }
            if spec.kind == OptionKind::Choice {
                let chosen = if text.is_empty() {
                    self.stack
                        .last()
                        .and_then(|c| c.selected)
                        .map(|i| spec.choices[i].value.clone())
                        .ok_or("Select a choice first")?
                } else {
                    text.into()
                };
                return self.choose(&key, &chosen);
            }
            self.draft = text.into();
            let value = if text.is_empty() {
                self.values
                    .get(&key)
                    .cloned()
                    .ok_or("Enter a value; no default is defined")?
            } else {
                self.parse(&key, text)?
            };
            return self.proposal(&key, value);
        }
        Ok(OptionOutcome::Unhandled)
    }
    fn mnemonic(&self, spec: &OptionSpec) -> String {
        if !spec.enabled {
            return String::new();
        }
        spec.aliases
            .iter()
            .find_map(|alias| {
                let mut chars = alias.chars();
                let character = chars.next()?;
                if chars.next().is_some()
                    || self.resolve(alias).ok().flatten().as_deref() != Some(spec.key.as_str())
                {
                    return None;
                }
                spec.key
                    .chars()
                    .find(|c| c.eq_ignore_ascii_case(&character))
                    .map(|c| c.to_string())
            })
            .unwrap_or_default()
    }
    fn resolve_choice<'a>(
        &self,
        spec: &'a OptionSpec,
        text: &str,
    ) -> Result<Option<&'a Choice>, OptionError> {
        if let Some(choice) = spec
            .choices
            .iter()
            .find(|c| c.value.eq_ignore_ascii_case(text))
        {
            return Ok(Some(choice));
        }
        let matches: Vec<_> = spec
            .choices
            .iter()
            .filter(|c| {
                c.aliases
                    .iter()
                    .any(|alias| alias.eq_ignore_ascii_case(text))
            })
            .collect();
        let enabled: Vec<_> = matches
            .iter()
            .copied()
            .filter(|c| self.cmd_choice_error(c).is_none())
            .collect();
        match enabled.as_slice() {
            [choice] => return Ok(Some(choice)),
            [] => {}
            _ => return Err("Ambiguous choice; enter the full name".into()),
        }
        match matches.as_slice() {
            [] => Ok(None),
            [choice] => Ok(Some(choice)),
            _ => Err("Ambiguous choice; enter the full name".into()),
        }
    }
    fn choice_mnemonic(&self, spec: &OptionSpec, choice: &Choice) -> String {
        if self.cmd_choice_error(choice).is_some() {
            return String::new();
        }
        choice
            .aliases
            .iter()
            .find_map(|alias| {
                let mut chars = alias.chars();
                let character = chars.next()?;
                if chars.next().is_some()
                    || self
                        .resolve_choice(spec, alias)
                        .ok()
                        .flatten()
                        .map(|c| c.value.as_str())
                        != Some(choice.value.as_str())
                {
                    return None;
                }
                choice
                    .value
                    .chars()
                    .find(|c| c.eq_ignore_ascii_case(&character))
                    .map(|c| c.to_string())
            })
            .unwrap_or_default()
    }
    pub fn snapshot(&self) -> OptionSnapshot {
        let pending = self.pending_key().map(str::to_owned);
        let selected = self.stack.last().and_then(|c| c.selected);
        let mut text = String::new();
        let mut items = Vec::new();
        let mut question_length = 0;
        if let Some(key) = &pending {
            if let Ok(s) = self.spec(key) {
                if s.kind == OptionKind::Number {
                    text = format!(
                        "{}{} [{}..{}]{}: ",
                        s.key,
                        if s.unit.is_empty() {
                            String::new()
                        } else {
                            format!(" ({})", s.unit)
                        },
                        s.minimum,
                        s.maximum,
                        self.values
                            .get(key)
                            .map(|v| format!(" <{}>", v.text()))
                            .unwrap_or_default()
                    );
                } else {
                    text = format!("{} ( ", s.key);
                    for c in &s.choices {
                        let start = text.encode_utf16().count();
                        text.push_str(&c.value);
                        let length = c.value.encode_utf16().count();
                        let reason = self.cmd_choice_error(c);
                        if reason.is_some() {
                            text.push_str(" [unsupported]");
                        }
                        text.push(' ');
                        items.push(PromptItem {
                            key: key.clone(),
                            input: format!("{}={}", key, c.value),
                            start,
                            length,
                            enabled: reason.is_none(),
                            reason: reason.unwrap_or_default(),
                            mnemonic: self.choice_mnemonic(s, c),
                        });
                    }
                    text.push_str("): ");
                }
            }
        } else {
            if !self.question.is_empty() {
                text.push_str(&self.question);
                question_length = text.encode_utf16().count();
                text.push(' ');
            }
            let show_options = self
                .schema
                .iter()
                .any(|s| self.visible(s) && s.scope & 1 != 0);
            if show_options || self.question.is_empty() {
                text.push_str("( ");
            }
            for s in self
                .schema
                .iter()
                .filter(|s| self.visible(s) && s.scope & 1 != 0)
            {
                let start = text.encode_utf16().count();
                text.push_str(&s.key);
                let length = s.key.encode_utf16().count();
                if let Some(value) = self.values.get(&s.key) {
                    text.push('=');
                    text.push_str(&value.text());
                }
                if !s.enabled {
                    text.push_str(" [unsupported]");
                }
                text.push(' ');
                items.push(PromptItem {
                    key: s.key.clone(),
                    input: s.key.clone(),
                    start,
                    length,
                    enabled: s.enabled,
                    reason: s.reason.clone(),
                    mnemonic: self.mnemonic(s),
                });
            }
            if show_options || self.question.is_empty() {
                text.push_str("): ");
            } else {
                text.pop();
                text.push_str(": ");
            }
        }
        OptionSnapshot {
            text,
            question_length,
            items,
            pending,
            generation: self.context.generation,
            revision: self.revision,
            selected,
        }
    }
    pub fn visible(&self, spec: &OptionSpec) -> bool {
        if !spec.shown {
            return false;
        }
        match spec.parent.as_deref(){None=>true,Some(key)=>self.spec(key).is_ok_and(|parent|{
            self.visible(parent)&&matches!(self.values.get(key),Some(OptionValue::Choice(value)) if parent.choices.iter().any(|c|c.value==*value&&c.child.as_deref()==Some(spec.key.as_str())))
        })}
    }
}
