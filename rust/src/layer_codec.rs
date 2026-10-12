//! Owned, versioned metadata codec. Geometry is absent and unknown fields reject.
use crate::layer_state::*;
use serde::{
    Deserialize, Serialize,
    de::{self, SeqAccess, Visitor},
};
use std::io::{self, Write};
use std::{fmt, marker::PhantomData};

struct LimitedWriter {
    bytes: Vec<u8>,
    limit: usize,
    failed: bool,
}
impl Write for LimitedWriter {
    fn write(&mut self, input: &[u8]) -> io::Result<usize> {
        let needed = self
            .bytes
            .len()
            .checked_add(input.len())
            .filter(|&size| size <= self.limit);
        let Some(needed) = needed else {
            self.failed = true;
            return Err(io::Error::other("metadata byte limit exceeded"));
        };
        if needed > self.bytes.capacity() {
            let capacity = self
                .bytes
                .capacity()
                .saturating_mul(2)
                .max(needed)
                .max(4096)
                .min(self.limit);
            if self
                .bytes
                .try_reserve_exact(capacity - self.bytes.len())
                .is_err()
            {
                self.failed = true;
                return Err(io::Error::other("metadata allocation limit"));
            }
        }
        self.bytes.extend_from_slice(input);
        Ok(input.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
pub(crate) fn encode_metadata<T: Serialize>(value: &T) -> Result<Vec<u8>, LayerError> {
    let mut writer = LimitedWriter {
        bytes: Vec::new(),
        limit: MAX_METADATA_BYTES,
        failed: false,
    };
    if serde_json::to_writer(&mut writer, value).is_err() {
        return Err(if writer.failed {
            LayerError::LimitExceeded
        } else {
            LayerError::InvalidSnapshot
        });
    }
    Ok(writer.bytes)
}
#[cfg(test)]
mod bounded_writer_tests {
    use super::*;
    #[test]
    fn serialization_refuses_before_allocating_or_writing_past_budget() {
        let mut writer = LimitedWriter {
            bytes: Vec::new(),
            limit: 128,
            failed: false,
        };
        assert!(serde_json::to_writer(&mut writer, &"x".repeat(256)).is_err());
        assert!(writer.failed && writer.bytes.len() <= 128 && writer.bytes.capacity() <= 128);
    }
}

// Qt/JSON consumers can round integer numbers through f64. Keep generations
// decimal strings on the wire so stale-state checks retain every uint64 bit.
pub(crate) mod generation {
    use serde::{Deserialize, Deserializer, Serializer, de};
    pub fn serialize<S: Serializer>(value: &u64, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&value.to_string())
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u64, D::Error> {
        let text = String::deserialize(deserializer)?;
        if text.is_empty()
            || text.len() > 20
            || (text.len() > 1 && text.starts_with('0'))
            || !text.bytes().all(|byte| byte.is_ascii_digit())
        {
            return Err(de::Error::custom(
                "generation must be canonical uint64 decimal text",
            ));
        }
        text.parse().map_err(de::Error::custom)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    version: u32,
    snapshot: LayerSnapshotV1,
}
pub fn encode_snapshot(state: &LayerSnapshotV1) -> Result<Vec<u8>, LayerError> {
    validate_layers(state)?;
    #[derive(Serialize)]
    struct Borrowed<'a> {
        version: u32,
        snapshot: &'a LayerSnapshotV1,
    }
    let bytes = serde_json::to_vec(&Borrowed {
        version: 1,
        snapshot: state,
    })
    .map_err(|_| LayerError::InvalidSnapshot)?;
    if bytes.len() > MAX_METADATA_BYTES {
        return Err(LayerError::LimitExceeded);
    }
    Ok(bytes)
}
pub fn decode_snapshot(bytes: &[u8]) -> Result<LayerSnapshotV1, LayerError> {
    if bytes.len() > MAX_METADATA_BYTES {
        return Err(LayerError::LimitExceeded);
    }
    let wire: Envelope = serde_json::from_slice(bytes).map_err(|_| LayerError::InvalidSnapshot)?;
    if wire.version != 1 {
        return Err(LayerError::UnsupportedVersion);
    }
    validate_layers(&wire.snapshot)?;
    Ok(wire.snapshot)
}
trait EntryCharge {
    fn charge(&self) -> usize;
}
impl EntryCharge for LayerRow {
    fn charge(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.source_id.len()
            + self.name.len()
            + self.parent_id.as_ref().map_or(0, String::len)
            + self
                .path_components
                .iter()
                .map(|s| std::mem::size_of::<String>() + s.len())
                .sum::<usize>()
    }
}
impl EntryCharge for ObjectLayerRow {
    fn charge(&self) -> usize {
        std::mem::size_of::<Self>() + self.id.len() + self.layer_id.len()
    }
}
impl EntryCharge for String {
    fn charge(&self) -> usize {
        std::mem::size_of::<Self>() + self.len()
    }
}
struct Bounded<T> {
    limit: usize,
    marker: PhantomData<T>,
}
impl<'de, T: Deserialize<'de> + EntryCharge> Visitor<'de> for Bounded<T> {
    type Value = Vec<T>;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("bounded typed layer metadata array")
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let mut values = Vec::new();
        let mut bytes = 0usize;
        while let Some(value) = seq.next_element::<T>()? {
            if values.len() >= self.limit {
                return Err(de::Error::custom("layer metadata count exceeds bound"));
            }
            bytes = bytes
                .checked_add(value.charge())
                .ok_or_else(|| de::Error::custom("metadata allocation accounting overflow"))?;
            if bytes > MAX_METADATA_BYTES {
                return Err(de::Error::custom("layer metadata allocation exceeds bound"));
            }
            values.push(value);
        }
        Ok(values)
    }
}
pub(crate) fn deserialize_layers<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<LayerRow>, D::Error> {
    deserializer.deserialize_seq(Bounded {
        limit: MAX_LAYERS,
        marker: PhantomData,
    })
}
pub(crate) fn deserialize_objects<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<ObjectLayerRow>, D::Error> {
    deserializer.deserialize_seq(Bounded {
        limit: MAX_OBJECTS,
        marker: PhantomData,
    })
}
pub(crate) fn deserialize_ids<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<String>, D::Error> {
    let values: Vec<String> = deserializer.deserialize_seq(Bounded {
        limit: MAX_OBJECTS,
        marker: PhantomData,
    })?;
    if values
        .iter()
        .any(|s| s.is_empty() || s.len() > 1024 || s.contains('\0'))
    {
        return Err(de::Error::custom("invalid object identity"));
    }
    Ok(values)
}
