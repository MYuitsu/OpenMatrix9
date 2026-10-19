// SPDX-License-Identifier: LGPL-2.1-or-later
//! Trusted build/test receipts govern availability. Archive metadata never
//! registers or enables runtime backends. Receipts are produced by validation.
use crate::{Error, Result, geometry::GeometryKind};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Backend {
    Rust {
        version: String,
    },
    Native {
        library: String,
        version: String,
        abi: String,
        toolkits: Vec<String>,
        closure_verified: bool,
        concurrent: bool,
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReceiptScope {
    SourceAudit,
    Semantic,
    NativeOperation,
    StandaloneRuntime,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Receipt {
    pub target: String,
    pub source_sha256: String,
    pub binary_sha256: String,
    pub suite: String,
    pub scope: ReceiptScope,
    pub passed: bool,
}
impl Receipt {
    fn usable(&self, scope: ReceiptScope) -> bool {
        let hash = |s: &str| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit());
        self.scope == scope
            && self.passed
            && self.target == "OM9Standalone"
            && !self.suite.is_empty()
            && hash(&self.source_sha256)
            && hash(&self.binary_sha256)
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Operation {
    pub id: String,
    pub feature: String,
    /// The other names one command answers to (menu alias, Rhino/Matrix alias).
    /// They resolve to exactly the same descriptor as the id (OM9-L3-01-02-001).
    #[serde(default)]
    pub aliases: Vec<String>,
    /// The question the command asks while it runs ("Start of line: "). Empty
    /// means the command asks nothing (OM9-L3-01-02-002).
    #[serde(default)]
    pub prompt: String,
    /// What the user must supply. An empty list is a command that needs no input.
    pub input_types: Vec<GeometryKind>,
    /// True when the command cannot run without that input: a descriptor that
    /// declares a mandatory input and no type is refused at registration.
    #[serde(default)]
    pub requires_input: bool,
    /// What the command produces. Empty means it modifies or consumes instead.
    #[serde(default)]
    pub output: Vec<GeometryKind>,
    pub options: Vec<String>,
    /// What the command needs from the session before it may be offered
    /// (OM9-L3-01-02-011).
    #[serde(default)]
    pub preconditions: Preconditions,
    pub backend: Backend,
    pub prepare: bool,
    pub preview: bool,
    pub commit: bool,
    pub receipts: Vec<Receipt>,
}

/// What a command needs from the session before it can be offered at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Preconditions {
    #[serde(default)]
    pub needs_document: bool,
    #[serde(default)]
    pub needs_selection: bool,
    #[serde(default)]
    pub needs_active_view: bool,
}

/// The session facts availability is decided against.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ContextState {
    pub has_document: bool,
    pub selection: usize,
    pub has_active_view: bool,
}

impl ContextState {
    /// A session with an empty document and one view: the state at startup.
    pub fn empty_document() -> Self {
        Self {
            has_document: true,
            selection: 0,
            has_active_view: true,
        }
    }
}

impl Operation {
    /// The declared option a user-typed name means, ignoring case, or an explicit
    /// refusal naming the option and the command (OM9-L3-01-02-002).
    pub fn option(&self, name: &str) -> std::result::Result<&str, RegistryError> {
        let wanted = name.trim();
        self.options
            .iter()
            .find(|option| option.eq_ignore_ascii_case(wanted))
            .map(String::as_str)
            .ok_or_else(|| RegistryError::UnknownOption {
                name: trimmed(name),
                command: self.id.clone(),
            })
    }
}

fn trimmed(name: &str) -> String {
    name.trim().to_owned()
}

/// The first precondition the session does not satisfy, in a fixed order so two
/// callers always read the same reason.
fn precondition_failure(
    preconditions: &Preconditions,
    context: &ContextState,
) -> Option<&'static str> {
    if preconditions.needs_document && !context.has_document {
        return Some("No document is open");
    }
    if preconditions.needs_active_view && !context.has_active_view {
        return Some("No active view");
    }
    if preconditions.needs_selection && context.selection == 0 {
        return Some("Select an object first");
    }
    None
}

/// Why a command name could not be registered or resolved. Every variant names
/// the name it refused, so the caller can say which one it was.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryError {
    /// The name is not the id, the feature or an alias of any operation.
    UnknownName { name: String },
    /// Two operations claim the same name.
    DuplicateName {
        name: String,
        first: String,
        second: String,
    },
    /// The name is empty or only whitespace.
    EmptyName { operation: String },
    /// The command does not declare that option (OM9-L3-01-02-002).
    UnknownOption { name: String, command: String },
    /// The id itself is already registered.
    DuplicateId { id: String },
    /// The operation is missing something the registry requires.
    InvalidOperation { id: String, why: &'static str },
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistryError::UnknownName { name } => {
                write!(f, "unknown command '{name}'")
            }
            RegistryError::DuplicateName {
                name,
                first,
                second,
            } => write!(
                f,
                "command name '{name}' is ambiguous: '{first}' and '{second}' both claim it"
            ),
            RegistryError::EmptyName { operation } => {
                write!(f, "command '{operation}' has an empty name")
            }
            RegistryError::UnknownOption { name, command } => {
                write!(f, "command '{command}' has no option '{name}'")
            }
            RegistryError::DuplicateId { id } => write!(f, "command '{id}' is already registered"),
            RegistryError::InvalidOperation { id, why } => {
                write!(f, "command '{id}' is not usable: {why}")
            }
        }
    }
}

/// The registry answers "which command does this name mean" with one descriptor.
impl CapabilityRegistry {
    /// Register an operation and every name it answers to.
    ///
    /// A name is the id, the feature or one of the aliases; each name belongs to
    /// exactly one operation, compared without case (`Line` and `line` are the
    /// same command).
    pub fn try_register(&mut self, operation: Operation) -> std::result::Result<(), RegistryError> {
        let id = operation.id.clone();
        if id.trim().is_empty() {
            return Err(RegistryError::InvalidOperation {
                id,
                why: "the id is empty",
            });
        }
        if operation.feature.trim().is_empty() {
            return Err(RegistryError::InvalidOperation {
                id,
                why: "the feature is empty",
            });
        }
        if operation.requires_input && operation.input_types.is_empty() {
            return Err(RegistryError::InvalidOperation {
                id,
                why: "a mandatory input has no declared type",
            });
        }
        if operation.input_types.is_empty()
            && operation.output.is_empty()
            && operation.prompt.is_empty()
        {
            // A command with no input, no output and no question is not a
            // descriptor: it would be offered and then do nothing it declares.
            return Err(RegistryError::InvalidOperation {
                id,
                why: "the descriptor declares no input, no output and no prompt",
            });
        }
        if self.operations.contains_key(&id) {
            return Err(RegistryError::DuplicateId { id });
        }
        let mut seen = BTreeSet::new();
        if operation.options.iter().any(|s| {
            // An option the user can never type is not an option, and the parser
            // reads options without case, so 'Length' and 'length' are one name.
            s.trim().is_empty() || !seen.insert(s.trim().to_ascii_lowercase())
        }) {
            return Err(RegistryError::InvalidOperation {
                id,
                why: "an option is empty or repeated",
            });
        }
        // The names this command answers to: the stable id and its aliases. The
        // feature is a plan identifier, not a command name, so two commands may
        // share one. One command may spell its own name twice under different
        // case — that is the built-in menu name next to the stable id, and it
        // still means one descriptor — but two commands may never share a name.
        let mut names: Vec<String> = Vec::with_capacity(1 + operation.aliases.len());
        names.push(id.clone());
        names.extend(operation.aliases.iter().cloned());
        let mut claimed = BTreeSet::new();
        for name in &names {
            let key = name.trim().to_ascii_lowercase();
            if key.is_empty() {
                return Err(RegistryError::EmptyName { operation: id });
            }
            claimed.insert(key);
        }
        for other in self.operations.values() {
            let mut other_names = vec![other.id.clone()];
            other_names.extend(other.aliases.iter().cloned());
            for other_name in other_names {
                let other_key = other_name.trim().to_ascii_lowercase();
                if let Some(clash) = claimed.get(&other_key) {
                    return Err(RegistryError::DuplicateName {
                        name: clash.clone(),
                        first: other.id.clone(),
                        second: id,
                    });
                }
            }
        }
        self.operations.insert(id.clone(), operation);
        Ok(())
    }

    /// The one descriptor a name means, whatever the user typed: the stable id or
    /// any alias, compared without case.
    pub fn resolve(&self, name: &str) -> std::result::Result<&Operation, RegistryError> {
        let wanted = name.trim().to_ascii_lowercase();
        self.operations
            .values()
            .find(|operation| {
                operation.id.eq_ignore_ascii_case(&wanted)
                    || operation
                        .aliases
                        .iter()
                        .any(|alias| alias.trim().eq_ignore_ascii_case(&wanted))
            })
            .ok_or(RegistryError::UnknownName {
                name: name.trim().to_owned(),
            })
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Prepare,
    Preview,
    Commit,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Availability {
    pub enabled: bool,
    pub reason: String,
}
#[derive(Debug, Default)]
pub struct CapabilityRegistry {
    operations: BTreeMap<String, Operation>,
}
impl CapabilityRegistry {
    pub fn register(&mut self, operation: Operation) -> Result<()> {
        // The old total entry point keeps its signature: a caller that only wants
        // "is this operation usable at all" is not forced to read a diagnostic.
        self.try_register(operation)
            .map_err(|_| Error::Invalid("operation registration"))
    }
    /// Availability from the recorded receipts alone: what the command can do.
    pub fn availability(&self, id: &str) -> Availability {
        self.availability_with(id, &ContextState::default())
    }

    /// Availability from the receipts **and** the session state: a command whose
    /// preconditions are not met is offered as disabled with the reason, never
    /// hidden and never offered as if it could run (OM9-L3-01-02-011).
    pub fn availability_with(&self, id: &str, context: &ContextState) -> Availability {
        let Some(operation) = self.operations.get(id) else {
            return Availability {
                enabled: false,
                reason: "Operation has no standalone implementation".into(),
            };
        };
        if let Some(reason) = precondition_failure(&operation.preconditions, context) {
            return Availability {
                enabled: false,
                reason: reason.into(),
            };
        }
        let has = |scope| operation.receipts.iter().any(|r| r.usable(scope));
        let failure = match &operation.backend {
            Backend::Rust { version } => {
                if version.is_empty()
                    || !has(ReceiptScope::Semantic) && !has(ReceiptScope::StandaloneRuntime)
                {
                    Some("Rust semantic receipt is missing")
                } else {
                    None
                }
            }
            Backend::Native {
                library,
                version,
                abi,
                toolkits,
                closure_verified,
                ..
            } => {
                if library.is_empty()
                    || version.is_empty()
                    || abi.is_empty()
                    || toolkits.is_empty()
                    || !closure_verified
                {
                    Some("Native ABI/dependency closure has not been verified")
                } else if !has(ReceiptScope::NativeOperation) {
                    Some("Native operation receipt is missing")
                } else {
                    None
                }
            }
        };
        let failure = failure.or_else(|| {
            if operation.commit && !has(ReceiptScope::StandaloneRuntime) {
                Some("Standalone commit/runtime receipt is missing")
            } else {
                None
            }
        });
        match failure {
            Some(reason) => Availability {
                enabled: false,
                reason: reason.into(),
            },
            None => Availability {
                enabled: true,
                reason: String::new(),
            },
        }
    }
    pub fn admit(
        &self,
        id: &str,
        kind: GeometryKind,
        options: &[&str],
        phase: Phase,
    ) -> Result<&Operation> {
        // A caller that brings no session context cannot claim the preconditions
        // are met, so a command that needs one is refused here.
        self.admit_with(id, kind, options, phase, &ContextState::default())
    }

    /// The gate a running command passes: evidence, input, options, phase **and**
    /// the session preconditions. A refused command has not touched anything.
    pub fn admit_with(
        &self,
        id: &str,
        kind: GeometryKind,
        options: &[&str],
        phase: Phase,
        context: &ContextState,
    ) -> Result<&Operation> {
        if !self.availability_with(id, context).enabled {
            return Err(Error::Unsupported("operation evidence/preconditions"));
        }
        let operation = self.operations.get(id).ok_or(Error::Missing("operation"))?;
        if !operation.input_types.contains(&kind) {
            return Err(Error::Unsupported("operation representation"));
        }
        // The descriptor declares option names without case (Operation::option),
        // so the gate must read them the same way.
        if let Some(unknown) = options.iter().find(|name| operation.option(name).is_err()) {
            let _ = unknown;
            return Err(Error::Unsupported("operation options"));
        }
        if !match phase {
            Phase::Prepare => operation.prepare,
            Phase::Preview => operation.preview,
            Phase::Commit => operation.commit,
        } {
            return Err(Error::Unsupported("operation phase"));
        }
        Ok(operation)
    }
}
