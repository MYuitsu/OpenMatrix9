// SPDX-License-Identifier: LGPL-2.1-or-later
//! Portable whole-document snapshot: the crate's own container encoding together
//! with the document identity it was captured from. A snapshot never restores
//! into another document and never publishes a partially restored one.
use crate::container;
use om9_core::{
    Error, Result,
    budget::{Limits, Usage},
    identity::DocumentId,
    model::Document,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const FORMAT: &str = "om9.document.snapshot";
const VERSION: u32 = 1;
const HEX: [u8; 16] = *b"0123456789abcdef";
/// Version refusal reuses the container's recorded schema message so a caller
/// cannot mistake an unknown snapshot epoch for a restorable one.
const VERSION_REFUSAL: &str = "OM9 schema version; no implicit migration";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    format: String,
    version: u32,
    document_id: DocumentId,
    epoch: u64,
    sha256: String,
    /// Hex text of the complete container encoding. Text keeps the snapshot a
    /// plain JSON document without a second binary codec.
    payload: String,
}
/// Captured document plus the identity, epoch and digest needed to refuse
/// restoring anything else.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentSnapshot {
    version: u32,
    document_id: DocumentId,
    epoch: u64,
    sha256: String,
    payload: Vec<u8>,
}
fn hex(bytes: &[u8]) -> Result<String> {
    let len = bytes
        .len()
        .checked_mul(2)
        .ok_or(Error::Overflow("snapshot hex"))?;
    let mut text = String::new();
    text.try_reserve_exact(len)
        .map_err(|_| Error::Budget("snapshot hex allocation"))?;
    for byte in bytes {
        text.push(HEX[usize::from(*byte >> 4)] as char);
        text.push(HEX[usize::from(*byte & 0x0f)] as char);
    }
    Ok(text)
}
fn digit(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}
/// Decode one complete hex payload. Truncated text (odd length) and non-hex
/// character data are both `Invalid`, never a silently shorter payload.
fn unhex(text: &str) -> Result<Vec<u8>> {
    let bytes = text.as_bytes();
    if !bytes.len().is_multiple_of(2) {
        return Err(Error::Invalid("snapshot payload hex"));
    }
    let mut out = Vec::new();
    out.try_reserve_exact(bytes.len() / 2)
        .map_err(|_| Error::Budget("snapshot payload allocation"))?;
    for pair in bytes.as_chunks::<2>().0 {
        let high = digit(pair[0]).ok_or(Error::Invalid("snapshot payload hex"))?;
        let low = digit(pair[1]).ok_or(Error::Invalid("snapshot payload hex"))?;
        out.push((high << 4) | low);
    }
    Ok(out)
}
/// The snapshot retains the encoded payload and the hex text carrying it. Both
/// copies are admitted against the caller's limits before either is returned.
fn retain(payload_len: usize, limits: &Limits) -> Result<()> {
    let hex_len = payload_len
        .checked_mul(2)
        .ok_or(Error::Overflow("snapshot hex"))?;
    let mut usage = Usage {
        bytes: payload_len as u64,
        ..Usage::default()
    };
    usage.add(
        Usage {
            bytes: hex_len as u64,
            ..Usage::default()
        },
        limits,
    )
}
impl DocumentSnapshot {
    /// Encode the document with the archive container codec and record the
    /// identity, epoch and digest that restoration rechecks.
    pub fn capture(document: &Document, limits: &Limits) -> Result<Self> {
        let payload = container::encode(document, limits)?;
        let sha256 = hex(&Sha256::digest(&payload))?;
        retain(payload.len(), limits)?;
        Ok(Self {
            version: VERSION,
            document_id: document.id,
            epoch: document.epoch,
            sha256,
            payload,
        })
    }
    /// Restore in one atomic step: version tag, payload integrity, document
    /// identity, then typed validation. No partial document is ever returned.
    pub fn restore(&self, limits: &Limits) -> Result<Document> {
        if self.version != VERSION {
            return Err(Error::Unsupported(VERSION_REFUSAL));
        }
        // Integrity is established before the payload is interpreted so that an
        // edited snapshot is always reported as `snapshot checksum` rather than
        // as whatever downstream decoder error its bytes happen to produce.
        if self.digest()? != self.sha256 {
            return Err(Error::Invalid("snapshot checksum"));
        }
        let document = container::decode(&self.payload, limits)?;
        if document.id != self.document_id {
            return Err(Error::Invalid("snapshot document"));
        }
        let mut usage = Usage {
            bytes: self.payload.len() as u64,
            ..Usage::default()
        };
        usage.add(document.validate(limits)?, limits)?;
        Ok(document)
    }
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(&Envelope {
            format: FORMAT.to_string(),
            version: self.version,
            document_id: self.document_id,
            epoch: self.epoch,
            sha256: self.sha256.clone(),
            payload: hex(&self.payload)?,
        })
        .map_err(|error| Error::Io(error.to_string()))
    }
    pub fn from_json(text: &str, limits: &Limits) -> Result<Self> {
        let envelope: Envelope =
            serde_json::from_str(text).map_err(|_| Error::Invalid("snapshot format"))?;
        if envelope.format != FORMAT {
            return Err(Error::Unsupported(VERSION_REFUSAL));
        }
        // Text that is not even a complete hex encoding of the recorded bytes
        // cannot match the recorded digest either, so tampering is reported as
        // the checksum failure rather than as a low-level hex error.
        let payload = match unhex(&envelope.payload) {
            Ok(payload) => payload,
            Err(Error::Invalid(_)) => return Err(Error::Invalid("snapshot checksum")),
            Err(error) => return Err(error),
        };
        if hex(&Sha256::digest(&payload))? != envelope.sha256 {
            return Err(Error::Invalid("snapshot checksum"));
        }
        retain(payload.len(), limits)?;
        Ok(Self {
            version: envelope.version,
            document_id: envelope.document_id,
            epoch: envelope.epoch,
            sha256: envelope.sha256,
            payload,
        })
    }
    fn digest(&self) -> Result<String> {
        hex(&Sha256::digest(&self.payload))
    }
    pub fn document_id(&self) -> DocumentId {
        self.document_id
    }
    pub fn epoch(&self) -> u64 {
        self.epoch
    }
    pub fn sha256(&self) -> &str {
        &self.sha256
    }
}
