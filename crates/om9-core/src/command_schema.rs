// SPDX-License-Identifier: LGPL-2.1-or-later
//! Typed command option schemas and completion (M01 `OM9-L2-01-02`, wave W6).
//!
//! Ported from the legacy bridge (`rust/src/{command_option_schemas,command_completion,
//! command_options}.rs`). The legacy hard-coded numeric families and the C option layer stay
//! behind; what is portable — a typed schema per option (domain, default, required), the
//! fill/validate step, and the completion ranking — is re-implemented over the target's own
//! `OptionDomain`/`Value`.
use crate::{
    Error, Result,
    budget::{Limits, name},
    request::{OptionDomain, Options, Value},
};

/// One declared command option: its name, its typed domain, an optional default and whether
/// the command needs it.
#[derive(Debug, Clone, PartialEq)]
pub struct OptionSpec {
    pub name: String,
    pub domain: OptionDomain,
    pub default: Option<Value>,
    pub required: bool,
    pub summary: String,
}

impl OptionSpec {
    pub fn new(
        name: impl Into<String>,
        domain: OptionDomain,
        default: Option<Value>,
        required: bool,
    ) -> Self {
        Self {
            name: name.into(),
            domain,
            default,
            required,
            summary: String::new(),
        }
    }
    /// The schema must be well formed before a value is checked: the domain itself and the
    /// default (which must satisfy its own domain).
    pub fn validate(&self, limits: &Limits) -> Result<()> {
        name(&self.name, limits)?;
        if self.name.is_empty() {
            return Err(Error::Invalid("command option name"));
        }
        self.domain.validate()?;
        if let Some(default) = &self.default {
            self.domain.check(default, limits)?;
        }
        if self.required && self.default.is_some() {
            // A required option with a default can never be missing; that is a schema bug,
            // not a value bug.
            return Err(Error::Invalid("command option default/required"));
        }
        Ok(())
    }
}

/// The schema of one command: its names, its options and whether the command repeats.
#[derive(Debug, Clone, PartialEq)]
pub struct CommandSchema {
    pub command: String,
    pub aliases: Vec<String>,
    pub options: Vec<OptionSpec>,
    pub repeat: bool,
}

impl CommandSchema {
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            aliases: Vec::new(),
            options: Vec::new(),
            repeat: true,
        }
    }
    pub fn with_option(mut self, spec: OptionSpec) -> Self {
        self.options.push(spec);
        self
    }
    pub fn with_alias(mut self, alias: impl Into<String>) -> Self {
        self.aliases.push(alias.into());
        self
    }
    pub fn validate(&self, limits: &Limits) -> Result<()> {
        name(&self.command, limits)?;
        if self.command.is_empty() {
            return Err(Error::Invalid("command name"));
        }
        let mut seen = std::collections::BTreeSet::new();
        for alias in &self.aliases {
            name(alias, limits)?;
            if alias.is_empty() || !seen.insert(alias.to_ascii_lowercase()) {
                return Err(Error::Invalid("command alias"));
            }
        }
        let mut names = std::collections::BTreeSet::new();
        for spec in &self.options {
            spec.validate(limits)?;
            if !names.insert(spec.name.to_ascii_lowercase()) {
                return Err(Error::Invalid("command option name"));
            }
        }
        Ok(())
    }
    /// Look an option up by name; command input is case-insensitive.
    pub fn option(&self, name: &str) -> Option<&OptionSpec> {
        self.options
            .iter()
            .find(|spec| spec.name.eq_ignore_ascii_case(name))
    }
    /// The declared defaults, ready to merge under the supplied values.
    pub fn defaults(&self) -> Options {
        let mut options = Options::new();
        for spec in &self.options {
            if let Some(default) = &spec.default {
                options.insert(spec.name.clone(), default.clone());
            }
        }
        options
    }
    /// Fill the defaults under `supplied`, refuse unknown names, require what the schema
    /// requires, and check every final value against its domain.
    pub fn apply(&self, supplied: Options, limits: &Limits) -> Result<Options> {
        self.validate(limits)?;
        for key in supplied.keys() {
            name(key, limits)?;
            if self.option(key).is_none() {
                return Err(Error::Invalid("unknown option"));
            }
        }
        let mut options = self.defaults();
        for (key, value) in supplied {
            // Normalize the spelling to the declared name so downstream code sees one name.
            let declared = self
                .option(&key)
                .ok_or(Error::Invalid("unknown option"))?
                .name
                .clone();
            options.insert(declared, value);
        }
        for spec in &self.options {
            match options.get(&spec.name) {
                Some(value) => spec.domain.check(value, limits)?,
                None if spec.required => return Err(Error::Missing("required option")),
                None => {}
            }
        }
        Ok(options)
    }
    /// Complete option names for the text the user has typed so far.
    pub fn complete(&self, input: &str, limit: usize) -> Vec<String> {
        let names: Vec<&str> = self.options.iter().map(|spec| spec.name.as_str()).collect();
        complete(names, input, limit)
            .into_iter()
            .map(str::to_owned)
            .collect()
    }
}

/// The legacy completion ranking: exact first, then prefix, then case-insensitive prefix,
/// then a case-insensitive substring. `u32::MAX` means "not a candidate".
pub fn rank(candidate: &str, input: &str) -> u32 {
    if candidate == input {
        return 0;
    }
    if candidate.starts_with(input) {
        return 1;
    }
    let candidate_lower = candidate.to_ascii_lowercase();
    let input_lower = input.to_ascii_lowercase();
    if candidate_lower.starts_with(&input_lower) {
        return 2;
    }
    if candidate_lower.contains(&input_lower) {
        return 3;
    }
    u32::MAX
}

/// Complete a candidate list: keep the matches, order by rank then by name, and cut to
/// `limit`. An empty input offers every candidate in name order.
pub fn complete<'a>(
    candidates: impl IntoIterator<Item = &'a str>,
    input: &str,
    limit: usize,
) -> Vec<&'a str> {
    let mut scored: Vec<(u32, &'a str)> = candidates
        .into_iter()
        .filter_map(|candidate| {
            if input.is_empty() {
                Some((0, candidate))
            } else {
                let score = rank(candidate, input);
                (score != u32::MAX).then_some((score, candidate))
            }
        })
        .collect();
    scored.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(right.1)));
    scored
        .into_iter()
        .take(limit)
        .map(|(_, candidate)| candidate)
        .collect()
}

/// Value completion for a choice option: the declared values matching the prefix.
pub fn value_candidates(spec: &OptionSpec, prefix: &str) -> Vec<String> {
    match &spec.domain {
        OptionDomain::Choice(choices) => {
            let lower = prefix.to_ascii_lowercase();
            choices
                .iter()
                .filter(|choice| choice.to_ascii_lowercase().starts_with(&lower))
                .cloned()
                .collect()
        }
        _ => Vec::new(),
    }
}
