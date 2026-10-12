//! Native getters expose desired state, not the presence of an explicit setting.
//! A bounded witness preserves presence on unchanged OM9 exports. Native edits
//! win over stale witnesses; malformed supplied witnesses fail closed.
use crate::layer_state::LayerError;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PersistentFields {
    pub locked: Option<bool>,
    pub visible: Option<bool>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Witness {
    version: u32,
    child: bool,
    locked: bool,
    visible: bool,
    persistent_locked: Option<bool>,
    persistent_visible: Option<bool>,
}
pub fn encode_native_persistent(
    child: bool,
    locked: bool,
    visible: bool,
    fields: PersistentFields,
) -> Result<Vec<u8>, LayerError> {
    // The canonical local fields are desired own state, not inherited state.
    // Contradictory child desired flags cannot be faithfully represented by
    // ON_Layer::Write, which writes PersistentLocking/Visibility as local flags.
    if child
        && (fields.locked.is_some_and(|v| v != locked)
            || fields.visible.is_some_and(|v| v != visible))
    {
        return Err(LayerError::InvalidSnapshot);
    }
    serde_json::to_vec(&Witness {
        version: 1,
        child,
        locked,
        visible,
        persistent_locked: fields.locked,
        persistent_visible: fields.visible,
    })
    .map_err(|_| LayerError::InvalidSnapshot)
}
pub fn decode_native_persistent(
    child: bool,
    locked: bool,
    visible: bool,
    bytes: &[u8],
) -> Result<PersistentFields, LayerError> {
    let native = PersistentFields {
        locked: child.then_some(locked),
        visible: child.then_some(visible),
    };
    if bytes.is_empty() {
        return Ok(native);
    }
    if bytes.len() > 256 {
        return Err(LayerError::LimitExceeded);
    }
    let witness: Witness =
        serde_json::from_slice(bytes).map_err(|_| LayerError::InvalidSnapshot)?;
    if witness.version != 1 {
        return Err(LayerError::UnsupportedVersion);
    }
    // Revalidate a supplied marker rather than trusting any serialized fields.
    let fields = PersistentFields {
        locked: witness.persistent_locked,
        visible: witness.persistent_visible,
    };
    encode_native_persistent(witness.child, witness.locked, witness.visible, fields)?;
    if witness.child != child {
        return Ok(native);
    }
    Ok(PersistentFields {
        locked: if witness.locked == locked {
            fields.locked
        } else {
            native.locked
        },
        visible: if witness.visible == visible {
            fields.visible
        } else {
            native.visible
        },
    })
}
