// SPDX-License-Identifier: LGPL-2.1-or-later
//! Streaming structural preflight before allocating the typed document.
use om9_core::{
    Error, Result,
    budget::{Limits, Usage},
};
use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use std::{collections::BTreeSet, fmt};

struct Budget<'a> {
    limits: &'a Limits,
    usage: Usage,
    failure: Option<Error>,
    legacy_document: bool,
    admit: &'a mut dyn FnMut(u64) -> Result<()>,
}
#[derive(Clone, Copy)]
enum Location {
    Unknown,
    Document,
    Layers,
    Layer,
    Entities,
    Entity,
    Attributes,
    Definitions,
    Definition,
    Members,
}
impl Location {
    fn child(self, key: &str) -> Self {
        match (self, key) {
            (Self::Document, "layers") => Self::Layers,
            (Self::Document, "entities") => Self::Entities,
            (Self::Document, "definitions") => Self::Definitions,
            (Self::Layers, _) => Self::Layer,
            (Self::Entities, _) => Self::Entity,
            (Self::Entity, "attributes") => Self::Attributes,
            (Self::Definitions, _) => Self::Definition,
            (Self::Definition, "members") => Self::Members,
            _ => Self::Unknown,
        }
    }
    fn element(self) -> Self {
        if matches!(self, Self::Members) {
            Self::Entity
        } else {
            Self::Unknown
        }
    }
    /// Fields that a later schema introduced. A payload owned by an earlier
    /// schema must refuse them and travel through an explicit migration instead
    /// of having the additive record admitted implicitly.
    fn legacy_new_field(self, key: &str) -> Option<&'static str> {
        match (self, key) {
            (Self::Attributes, "hidden") | (Self::Layer, "preset_slot") => {
                Some("legacy schema has no preset/Hidden fields")
            }
            (Self::Document, "groups" | "features") | (Self::Entity, "extensions") => {
                Some("legacy schema has no group/feature/extension fields")
            }
            (Self::Document, "tolerance") => Some("legacy schema has no document tolerance field"),
            _ => None,
        }
    }
}
impl Budget<'_> {
    fn reserve<E: de::Error>(&mut self, usage: Usage) -> std::result::Result<(), E> {
        let mut next = self.usage;
        if let Err(error) = next.add(usage, self.limits).and_then(|()| (self.admit)(next.bytes)) {
            self.failure = Some(error);
            return Err(E::custom("JSON admission budget"));
        }
        self.usage = next;
        Ok(())
    }
    fn fail<E: de::Error>(&mut self, error: Error) -> E {
        self.failure = Some(error);
        E::custom("JSON admission")
    }
}
struct Node<'a, 'b> {
    budget: &'a mut Budget<'b>,
    depth: usize,
    field: Option<String>,
    location: Location,
}
// An escaped JSON key is first exposed through the parser scratch. Admit both
// owned key copies before creating the first String; the second belongs to the
// duplicate-key set while the first can remain live in the child visitor.
struct Key<'a, 'b>(&'a mut Budget<'b>);
impl<'de> DeserializeSeed<'de> for Key<'_, '_> {
    type Value = String;
    fn deserialize<D: de::Deserializer<'de>>(self, d: D) -> std::result::Result<String, D::Error> {
        d.deserialize_str(self)
    }
}
impl<'de> Visitor<'de> for Key<'_, '_> {
    type Value = String;
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("bounded JSON key")
    }
    fn visit_str<E: de::Error>(self, value: &str) -> std::result::Result<String, E> {
        if value.len() > self.0.limits.max_name_bytes {
            return Err(self.0.fail(Error::Budget("JSON key bytes")));
        }
        let bytes = (value.len() as u64)
            .checked_mul(2)
            .ok_or_else(|| self.0.fail(Error::Overflow("JSON key copies")))?;
        self.0.reserve::<E>(Usage {
            bytes,
            ..Usage::default()
        })?;
        let mut key = String::new();
        key.try_reserve_exact(value.len())
            .map_err(|_| self.0.fail(Error::Budget("JSON key allocation")))?;
        key.push_str(value);
        Ok(key)
    }
}
impl<'de> DeserializeSeed<'de> for Node<'_, '_> {
    type Value = ();
    fn deserialize<D: de::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> std::result::Result<(), D::Error> {
        if self.depth > self.budget.limits.max_depth {
            return Err(self.budget.fail(Error::Budget("JSON nesting")));
        }
        self.budget.reserve::<D::Error>(Usage {
            bytes: 64,
            nodes: 1,
            ..Usage::default()
        })?;
        deserializer.deserialize_any(self)
    }
}
impl<'de> Visitor<'de> for Node<'_, '_> {
    type Value = ();
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("bounded JSON document")
    }
    fn visit_bool<E: de::Error>(self, _: bool) -> std::result::Result<(), E> {
        Ok(())
    }
    fn visit_i64<E: de::Error>(self, _: i64) -> std::result::Result<(), E> {
        Ok(())
    }
    fn visit_u64<E: de::Error>(self, _: u64) -> std::result::Result<(), E> {
        Ok(())
    }
    fn visit_f64<E: de::Error>(self, value: f64) -> std::result::Result<(), E> {
        if !value.is_finite() {
            return Err(self.budget.fail(Error::Invalid("nonfinite JSON number")));
        }
        Ok(())
    }
    fn visit_unit<E: de::Error>(self) -> std::result::Result<(), E> {
        Ok(())
    }
    fn visit_str<E: de::Error>(self, value: &str) -> std::result::Result<(), E> {
        // A canonical BRep source is an owned OCCT text document, not a name.
        // It is already bounded by the document byte budget; names remain tight.
        let limit = if self.field.as_deref() == Some("source") {
            self.budget.limits.max_bytes
        } else {
            self.budget.limits.max_name_bytes as u64
        };
        if value.len() as u64 > limit {
            return Err(self.budget.fail(Error::Budget("JSON string bytes")));
        }
        self.budget.reserve(Usage {
            bytes: value.len() as u64,
            ..Usage::default()
        })
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut sequence: A) -> std::result::Result<(), A::Error> {
        let point = self.field.as_deref() == Some("Point");
        if point {
            self.budget.reserve::<A::Error>(Usage {
                points: 1,
                ..Usage::default()
            })?;
        }
        let rows = matches!(self.field.as_deref(), Some("poles" | "points" | "vertices"));
        loop {
            let mut row_budget = Usage::default();
            if rows {
                row_budget.points = 1;
            }
            // Seed charges the row before decoding it; the terminator is not a
            // row, so reserve only after SeqAccess confirms a value exists.
            let before = self.budget.usage;
            let value = sequence.next_element_seed(Node {
                budget: self.budget,
                depth: self.depth + 1,
                field: None,
                location: self.location.element(),
            })?;
            if value.is_none() {
                self.budget.usage = before;
                break;
            }
            self.budget.reserve::<A::Error>(row_budget)?;
        }
        Ok(())
    }
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> std::result::Result<(), A::Error> {
        let mut keys = BTreeSet::new();
        let mut tree_bytes = Usage::btree::<String, ()>(0)
            .map_err(|e| self.budget.fail(e))?
            .bytes;
        self.budget.reserve::<A::Error>(Usage {
            bytes: tree_bytes,
            ..Usage::default()
        })?;
        while let Some(key) = map.next_key_seed(Key(self.budget))? {
            if self.budget.legacy_document
                && let Some(message) = self.location.legacy_new_field(&key)
            {
                return Err(self.budget.fail(Error::Invalid(message)));
            }
            let next = keys
                .len()
                .checked_add(1)
                .ok_or_else(|| self.budget.fail(Error::Overflow("JSON keys")))?;
            let required = Usage::btree::<String, ()>(next)
                .map_err(|e| self.budget.fail(e))?
                .bytes;
            self.budget.reserve::<A::Error>(Usage {
                bytes: 64 + required - tree_bytes,
                nodes: 1,
                ..Usage::default()
            })?;
            tree_bytes = required;
            if !keys.insert(key.clone()) {
                return Err(self.budget.fail(Error::Invalid("duplicate JSON key")));
            }
            map.next_value_seed(Node {
                budget: self.budget,
                depth: self.depth + 1,
                location: self.location.child(&key),
                field: Some(key),
            })?;
        }
        Ok(())
    }
}
pub fn check(metadata: &[u8], container_bytes: usize, limits: &Limits) -> Result<()> {
    check_usage(metadata, container_bytes, limits).map(|_| ())
}
pub fn check_usage(metadata: &[u8], container_bytes: usize, limits: &Limits) -> Result<Usage> {
    check_kind(metadata, container_bytes, limits, false, &mut |_| Ok(()))
}
pub(crate) fn check_external_with_admission(metadata: &[u8], limits: &Limits, admit: &mut dyn FnMut(u64) -> Result<()>) -> Result<Usage> {
    check_kind(metadata, metadata.len(), limits, false, admit)
}
pub(crate) fn check_usage_with_admission(metadata: &[u8], container_bytes: usize, limits: &Limits, admit: &mut dyn FnMut(u64) -> Result<()>) -> Result<Usage> {
    let parser = document_parser_limits(metadata, limits)?;
    check_kind(metadata, container_bytes, &parser, false, admit)
}
pub fn check_legacy_document(
    metadata: &[u8],
    container_bytes: usize,
    limits: &Limits,
) -> Result<()> {
    let parser = document_parser_limits(metadata, limits)?;
    check_kind(metadata, container_bytes, &parser, true, &mut |_| Ok(())).map(|_| ())
}

/// JSON scalar/key tokens are not model nodes. The input itself bounds the
/// number of tokens, and the existing byte budget charges every visited token.
/// Typed document validation separately enforces the original node limit.
fn document_parser_limits(metadata: &[u8], limits: &Limits) -> Result<Limits> {
    Ok(Limits {
        max_nodes: u64::try_from(metadata.len()).map_err(|_| Error::Overflow("JSON token limit"))?,
        ..*limits
    })
}
fn check_kind(
    metadata: &[u8],
    container_bytes: usize,
    limits: &Limits,
    legacy_document: bool,
    admit: &mut dyn FnMut(u64) -> Result<()>,
) -> Result<Usage> {
    // Account the input/container copy and geometrically growing UTF-unescape
    // scratch before walking any values. A late escape after a long prefix can
    // make parser capacity nearly twice the JSON input length. The extra term
    // applies to metadata rather than unrelated embedded binary payloads.
    let bytes = (container_bytes as u64)
        .checked_mul(2)
        .and_then(|n| n.checked_add(metadata.len() as u64))
        .ok_or(Error::Overflow("JSON input peak"))?;
    Usage { bytes, ..Usage::default() }.check(limits)?;
    admit(bytes)?;
    let mut budget = Budget {
        legacy_document,
        limits,
        usage: Usage {
            bytes,
            ..Usage::default()
        },
        failure: None,
        admit,
    };
    budget.usage.check(limits)?;
    let mut deserializer = serde_json::Deserializer::from_slice(metadata);
    let result = Node {
        budget: &mut budget,
        depth: 0,
        field: None,
        location: if legacy_document {
            Location::Document
        } else {
            Location::Unknown
        },
    }
    .deserialize(&mut deserializer)
    .and_then(|_| deserializer.end());
    if result.is_err() {
        return Err(budget.failure.unwrap_or(Error::Invalid("JSON syntax")));
    }
    Ok(budget.usage)
}

#[cfg(test)]
mod external_admission_tests {
    use super::*;

    #[test]
    fn incremental_external_admission_preserves_limits_and_propagates_refusal() {
        let input = br#"{"data":[1,2,3],"name":"part"}"#;
        let limits = Limits::default();
        let mut requests = Vec::new();
        let usage = check_external_with_admission(input, &limits, &mut |bytes| {
            requests.push(bytes);
            Ok(())
        }).unwrap();
        assert!(requests.len() > 2);
        assert!(requests.windows(2).all(|pair| pair[0] <= pair[1]));
        assert_eq!(*requests.last().unwrap(), usage.bytes);
        let threshold = requests[1];
        let mut calls = 0;
        let refused = check_external_with_admission(input, &limits, &mut |bytes| {
            calls += 1;
            if bytes > threshold { Err(Error::Budget("shared JSON preflight")) } else { Ok(()) }
        });
        assert!(matches!(refused, Err(Error::Budget("shared JSON preflight"))));
        assert!(calls < requests.len());
        let tight = Limits { max_nodes: 2, ..limits };
        assert!(matches!(check_external_with_admission(input, &tight, &mut |_| Ok(())), Err(Error::Budget(_))));
        assert!(matches!(check_external_with_admission(br#"{"x":1,"x":2}"#, &limits, &mut |_| Ok(())), Err(Error::Invalid("duplicate JSON key"))));
        assert!(matches!(check_external_with_admission(br#"{"x": [}"#, &limits, &mut |_| Ok(())), Err(Error::Invalid(_))));
    }
}
