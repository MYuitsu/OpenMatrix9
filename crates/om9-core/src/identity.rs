// SPDX-License-Identifier: LGPL-2.1-or-later
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Typed UUID maps admit each key before its value and never use serde's
/// last-record-wins map behavior. Only actual typed ID tables use this hook;
/// arbitrary command option names and generic JSON objects remain strings.
pub(crate) fn unique_uuid_map<'de, D, K, V>(
    deserializer: D,
) -> Result<std::collections::BTreeMap<K, V>, D::Error>
where
    D: serde::Deserializer<'de>,
    K: serde::Deserialize<'de> + Ord + std::fmt::Display,
    V: serde::Deserialize<'de>,
{
    struct Unique<K, V>(std::marker::PhantomData<(K, V)>);
    impl<'de, K, V> serde::de::Visitor<'de> for Unique<K, V>
    where
        K: serde::Deserialize<'de> + Ord + std::fmt::Display,
        V: serde::Deserialize<'de>,
    {
        type Value = std::collections::BTreeMap<K, V>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("unique canonical UUID table")
        }
        fn visit_map<A: serde::de::MapAccess<'de>>(
            self,
            mut map: A,
        ) -> Result<Self::Value, A::Error> {
            let mut out = std::collections::BTreeMap::new();
            while let Some(key) = map.next_key::<String>()? {
                let id = K::deserialize(serde::de::value::StrDeserializer::<A::Error>::new(&key))?;
                if key != id.to_string() {
                    return Err(serde::de::Error::custom("noncanonical UUID map key"));
                }
                if out.contains_key(&id) {
                    return Err(serde::de::Error::custom("duplicate typed UUID map key"));
                }
                out.insert(id, map.next_value::<V>()?);
            }
            Ok(out)
        }
    }
    deserializer.deserialize_map(Unique(std::marker::PhantomData))
}

/// Extension slots use the same admit-each-key discipline as typed UUID tables,
/// so a duplicated slot name in stored metadata is refused instead of silently
/// overwriting the earlier record.
pub(crate) fn unique_named_slot_map<'de, D, V>(
    deserializer: D,
) -> Result<std::collections::BTreeMap<String, V>, D::Error>
where
    D: serde::Deserializer<'de>,
    V: serde::Deserialize<'de>,
{
    struct Unique<V>(std::marker::PhantomData<V>);
    impl<'de, V> serde::de::Visitor<'de> for Unique<V>
    where
        V: serde::Deserialize<'de>,
    {
        type Value = std::collections::BTreeMap<String, V>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("unique named extension slot table")
        }
        fn visit_map<A: serde::de::MapAccess<'de>>(
            self,
            mut map: A,
        ) -> Result<Self::Value, A::Error> {
            let mut out = std::collections::BTreeMap::new();
            while let Some(key) = map.next_key::<String>()? {
                if out.contains_key(&key) {
                    return Err(serde::de::Error::custom("duplicate extension slot"));
                }
                out.insert(key, map.next_value::<V>()?);
            }
            Ok(out)
        }
    }
    deserializer.deserialize_map(Unique(std::marker::PhantomData))
}

macro_rules! identifier {
    ($($name:ident),+ $(,)?) => {$(
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub Uuid);
        impl $name {
            pub fn new() -> Self { Self(Uuid::new_v4()) }
            /// The nil identity is only valid as a *recorded marker* (an absent input, a
            /// legacy sentinel) and is refused wherever a live identity is required.
            pub fn nil() -> Self { Self(Uuid::nil()) }
            pub fn is_nil(self) -> bool { self.0.is_nil() }
        }
        impl Default for $name { fn default() -> Self { Self::new() } }
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { self.0.fmt(f) }
        }
    )+};
}
identifier!(
    DocumentId,
    ObjectId,
    DefinitionId,
    OccurrenceId,
    LayerId,
    MaterialId,
    LinePatternId,
    ResourceId,
    SourceId,
    ImportNamespaceId,
    HistoryId,
    ReferenceId,
    TransactionId,
    JobId,
    FeatureId,
    GroupId,
    ViewportId,
    TextStyleId,
    DimStyleId,
    HatchPatternId,
    LightId
);
identifier!(SessionId);
impl SessionId {
    /// Schema-1 files written before owner sessions existed use this sentinel
    /// only during format admission. Runtime restore always issues a new owner.
    pub fn legacy_nil() -> Self {
        Self(Uuid::nil())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceStamp {
    pub document: DocumentId,
    pub document_epoch: u64,
    pub session_generation: u64,
    #[serde(default = "SessionId::legacy_nil")]
    pub owner_session: SessionId,
    pub context_revision: u64,
    pub object: ObjectId,
    pub generation: u64,
    pub geometry_revision: u64,
    pub attributes_revision: u64,
}
