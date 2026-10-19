// SPDX-License-Identifier: LGPL-2.1-or-later
//! Closed, borrowed Layer CMD routing; labels never invent durable preset identity.
use om9_core::{Error, Result, identity::LayerId, layers::PresetSlot, model::Document};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerChoice {
    None,
    Slot(PresetSlot),
    Existing(LayerId),
}
#[derive(Debug, Clone, Copy)]
pub enum LayerProperty {
    Visible(bool),
    Locked(bool),
}
pub fn group(slot: PresetSlot) -> std::ops::RangeInclusive<u8> {
    let first = if slot.number() == 33 {
        33
    } else {
        ((slot.number() - 1) / 4) * 4 + 1
    };
    first..=if first == 33 { 33 } else { first + 3 }
}
fn same_name(a: &str, b: &str) -> bool {
    a.chars()
        .flat_map(char::to_lowercase)
        .eq(b.chars().flat_map(char::to_lowercase))
}
pub fn parse(text: &str, document: &Document) -> Result<Option<LayerChoice>> {
    let text = text.trim();
    let Some(prefix) = text.get(..5) else {
        return Ok(None);
    };
    if !prefix.eq_ignore_ascii_case("Layer") {
        return Ok(None);
    }
    let tail = &text[5..];
    if !tail.is_empty() && !tail.starts_with('=') && !tail.starts_with(char::is_whitespace) {
        return Ok(None);
    }
    if text.len() > 256 {
        return Err(Error::Budget("Layer command input"));
    }
    if text.chars().any(char::is_control) {
        return Err(Error::Invalid("Layer command controls"));
    }
    let value = tail.trim().strip_prefix('=').unwrap_or(tail.trim()).trim();
    let quoted = value.starts_with('"') || value.ends_with('"');
    let value = if quoted {
        value
            .strip_prefix('"')
            .and_then(|v| v.strip_suffix('"'))
            .ok_or(Error::Invalid("Layer command quotes"))?
    } else {
        value
    };
    if quoted {
        let mut found = None;
        for layer in document.layers.values() {
            if same_name(value, &layer.name) {
                if found.is_some() {
                    return Err(Error::Invalid("ambiguous quoted Layer label"));
                }
                found = Some(layer.id);
            }
        }
        if let Some(id) = found {
            return Ok(Some(LayerChoice::Existing(id)));
        }
        for n in 1..=33 {
            let slot = PresetSlot::new(n)?;
            if same_name(value, slot.name()) {
                return Ok(Some(LayerChoice::Slot(slot)));
            }
        }
        return Err(Error::Invalid("unknown quoted Layer label"));
    }
    if value.eq_ignore_ascii_case("None") || value == "0" {
        return Ok(Some(LayerChoice::None));
    }
    if let Ok(n) = value.parse::<u8>() {
        return Ok(Some(LayerChoice::Slot(PresetSlot::new(n)?)));
    }
    for n in 1..=33 {
        let slot = PresetSlot::new(n)?;
        if same_name(value, slot.name()) {
            return Ok(Some(LayerChoice::Slot(slot)));
        }
    }
    let mut found = None;
    for layer in document.layers.values() {
        if same_name(value, &layer.name) {
            if found.is_some() {
                return Err(Error::Invalid("ambiguous Layer label"));
            }
            found = Some(layer.id);
        }
    }
    found
        .map(LayerChoice::Existing)
        .map(Some)
        .ok_or(Error::Invalid("unknown Layer choice"))
}
