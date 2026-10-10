//! Geometry-free document command plans. Hosts own transactions and projection.
use crate::{
    layer_exchange::{LayerApplyPlan, choose_active},
    layer_state::*,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};

const DEFAULT_LAYERS: [(&str, u32); 32] = [
    ("Metal 01", 0x229987),
    ("Metal 02", 0x319f49),
    ("Metal 03", 0x70c64b),
    ("Metal 04", 0xa0cf78),
    ("Gem 01", 0x598bc2),
    ("Gem 02", 0x4f80cb),
    ("Gem 03", 0x7aa7ce),
    ("Gem 04", 0xaccae4),
    ("User 01", 0xe23434),
    ("User 02", 0x70c133),
    ("User 03", 0x3468ff),
    ("User 04", 0x777777),
    ("Heads", 0x851ca1),
    ("Finger", 0xad423c),
    ("Cutting", 0xca7135),
    ("Creation", 0xdfb126),
    ("User 17", 0xdf9098),
    ("User 18", 0xdabb7c),
    ("User 19", 0xbd8735),
    ("User 20", 0xa5c14b),
    ("User 25", 0x47c7aa),
    ("User 26", 0x72c6cb),
    ("User 27", 0x607cab),
    ("User 28", 0x4d626a),
    ("User 21", 0x57b397),
    ("User 22", 0x586076),
    ("User 23", 0xc6bbb7),
    ("User 24", 0x6c5578),
    ("User 29", 0x57536b),
    ("User 30", 0x1e555b),
    ("User 31", 0x0c343f),
    ("User 32", 0x1b3555),
];
pub fn default_document(document_id: &str) -> Result<LayerSnapshotV1, LayerError> {
    let mut state = LayerSnapshotV1::empty(document_id);
    state.layers = DEFAULT_LAYERS
        .iter()
        .enumerate()
        .map(|(i, (name, color))| LayerRow {
            source_id: format!("om9-preset-{}", i + 1),
            parent_id: None,
            name: (*name).into(),
            path_components: vec![(*name).into()],
            rgb: [(color >> 16) as u8, (color >> 8) as u8, *color as u8],
            locked: false,
            visible: true,
            persistent_locked: None,
            persistent_visible: None,
        })
        .collect();
    state.active_layer = Some(state.layers[0].source_id.clone());
    validate_layers(&state)?;
    Ok(state)
}

#[derive(Serialize)]
struct PanelRow<'a> {
    id: &'a str,
    name: &'a str,
    path: String,
    rgb: [u8; 3],
    locked: bool,
    visible: bool,
    effective_locked: bool,
    effective_visible: bool,
    can_current: bool,
    current: bool,
    preset_index: Option<usize>,
}
pub fn panel_json(state: &LayerSnapshotV1) -> Result<Vec<u8>, LayerError> {
    let index = LayerIndex::new(state)?;
    let effective = state
        .layers
        .iter()
        .map(|row| index.layer_effective(&row.source_id))
        .collect::<Result<Vec<_>, _>>()?;
    panel_json_cached(state, &effective)
}
pub(crate) fn panel_json_cached(
    state: &LayerSnapshotV1,
    effective: &[EffectiveState],
) -> Result<Vec<u8>, LayerError> {
    if effective.len() != state.layers.len() {
        return Err(LayerError::InvalidSnapshot);
    }
    let layers = state
        .layers
        .iter()
        .zip(effective)
        .map(|(row, flags)| PanelRow {
            id: &row.source_id,
            name: &row.name,
            path: row.path_components.join("::"),
            rgb: row.rgb,
            locked: row.locked,
            visible: row.visible,
            effective_locked: flags.locked,
            effective_visible: flags.visible,
            can_current: flags.selectable(),
            current: state.active_layer.as_ref() == Some(&row.source_id),
            preset_index: DEFAULT_LAYERS.iter().position(|(name, _)| {
                path_key(&[(*name).into()]) == path_key(&row.path_components)
            }),
        })
        .collect::<Vec<_>>();
    #[derive(Serialize)]
    struct Panel<'a> {
        version: u32,
        state_owner: &'static str,
        document_id: &'a str,
        generation: String,
        layers: Vec<PanelRow<'a>>,
    }
    serde_json::to_vec(&Panel {
        version: 1,
        state_owner: "rust",
        document_id: &state.document_id,
        generation: state.generation.to_string(),
        layers,
    })
    .map_err(|_| LayerError::InvalidSnapshot)
}

fn text_path(text: &str) -> Result<Vec<String>, LayerError> {
    let path = if text.starts_with('"') {
        serde_json::from_str::<String>(text).map_err(|_| LayerError::InvalidPath)?
    } else {
        text.to_string()
    };
    let parts = path.split("::").map(String::from).collect::<Vec<_>>();
    if parts.is_empty() || parts.iter().any(|s| s.is_empty()) {
        return Err(LayerError::InvalidPath);
    }
    Ok(parts)
}
pub fn plan_text(
    state: &LayerSnapshotV1,
    text: &str,
    selected: &[ObjectId],
) -> Result<LayerApplyPlan, LayerError> {
    if text.len() > 32768 || selected.len() > MAX_OBJECTS {
        return Err(LayerError::LimitExceeded);
    }
    validate_layers(state)?;
    let text = text.trim();
    let (prefix, rest) = text
        .split_once(char::is_whitespace)
        .ok_or(LayerError::InvalidSnapshot)?;
    if !prefix.eq_ignore_ascii_case("Layer") {
        return Err(LayerError::InvalidSnapshot);
    }
    let (action, argument) = rest
        .trim_start()
        .split_once(char::is_whitespace)
        .ok_or(LayerError::InvalidSnapshot)?;
    let action = action.to_ascii_lowercase();
    let argument = argument.trim();
    let (path, value) = match action.as_str() {
        "lock" | "visible" | "color" => argument
            .rsplit_once(char::is_whitespace)
            .map(|(p, v)| (p.trim(), v.trim()))
            .ok_or(LayerError::InvalidSnapshot)?,
        "current" | "assign" => (argument, ""),
        _ => return Err(LayerError::InvalidSnapshot),
    };
    let key = path_key(&text_path(path)?);
    let row = state
        .layers
        .iter()
        .find(|row| path_key(&row.path_components) == key)
        .ok_or(LayerError::MissingLayer)?;
    let layer_id = row.source_id.clone();
    let boolean = |own: bool| match value.to_ascii_lowercase().as_str() {
        "on" => Ok(true),
        "off" => Ok(false),
        "toggle" => Ok(!own),
        _ => Err(LayerError::InvalidSnapshot),
    };
    let command = match action.as_str() {
        "current" => LayerCommand::SetCurrent { layer_id },
        "assign" => LayerCommand::Assign {
            layer_id,
            object_ids: selected.to_vec(),
        },
        "lock" => LayerCommand::SetLocked {
            layer_id,
            locked: boolean(row.locked)?,
        },
        "visible" => LayerCommand::SetVisible {
            layer_id,
            visible: boolean(row.visible)?,
        },
        "color" => {
            let hex = value.strip_prefix('#').ok_or(LayerError::InvalidSnapshot)?;
            if hex.len() != 6 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Err(LayerError::InvalidSnapshot);
            }
            let n = u32::from_str_radix(hex, 16).map_err(|_| LayerError::InvalidSnapshot)?;
            LayerCommand::SetColor {
                layer_id,
                rgb: [(n >> 16) as u8, (n >> 8) as u8, n as u8],
            }
        }
        _ => return Err(LayerError::InvalidSnapshot),
    };
    plan_command(state, command)
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub enum LayerCommand {
    SetCurrent {
        layer_id: LayerId,
    },
    Assign {
        layer_id: LayerId,
        #[serde(deserialize_with = "crate::layer_codec::deserialize_ids")]
        object_ids: Vec<ObjectId>,
    },
    SetLocked {
        layer_id: LayerId,
        locked: bool,
    },
    SetVisible {
        layer_id: LayerId,
        visible: bool,
    },
    SetColor {
        layer_id: LayerId,
        rgb: [u8; 3],
    },
}

pub fn decode_command(bytes: &[u8]) -> Result<LayerCommand, LayerError> {
    if bytes.len() > MAX_METADATA_BYTES {
        return Err(LayerError::LimitExceeded);
    }
    let command: LayerCommand =
        serde_json::from_slice(bytes).map_err(|_| LayerError::InvalidSnapshot)?;
    let id = match &command {
        LayerCommand::SetCurrent { layer_id }
        | LayerCommand::Assign { layer_id, .. }
        | LayerCommand::SetLocked { layer_id, .. }
        | LayerCommand::SetVisible { layer_id, .. }
        | LayerCommand::SetColor { layer_id, .. } => layer_id,
    };
    if id.is_empty() || id.len() > 1024 || id.contains('\0') {
        return Err(LayerError::InvalidSnapshot);
    }
    Ok(command)
}

pub fn plan_command(
    state: &LayerSnapshotV1,
    command: LayerCommand,
) -> Result<LayerApplyPlan, LayerError> {
    let index = LayerIndex::new(state)?;
    let id = match &command {
        LayerCommand::SetCurrent { layer_id }
        | LayerCommand::Assign { layer_id, .. }
        | LayerCommand::SetLocked { layer_id, .. }
        | LayerCommand::SetVisible { layer_id, .. }
        | LayerCommand::SetColor { layer_id, .. } => layer_id,
    };
    let layer_state = index.layer_effective(id)?;
    match &command {
        LayerCommand::SetCurrent { .. } => {
            if layer_state.locked {
                return Err(LayerError::Locked);
            }
            if !layer_state.visible {
                return Err(LayerError::Hidden);
            }
        }
        LayerCommand::Assign {
            layer_id,
            object_ids,
        } => {
            if object_ids.len() > MAX_OBJECTS {
                return Err(LayerError::LimitExceeded);
            }
            check_mutation(
                object_ids,
                &LayerOperation::AssignLayer(layer_id.clone()),
                |id| index.effective(id),
                |id| index.layer_effective(id),
            )?;
        }
        _ => {}
    }
    let layer_position = state
        .layers
        .iter()
        .position(|row| &row.source_id == id)
        .ok_or(LayerError::MissingLayer)?;
    drop(index);
    let mut after = state.clone();
    match command {
        LayerCommand::SetCurrent { layer_id } => after.active_layer = Some(layer_id),
        LayerCommand::Assign {
            layer_id,
            object_ids,
        } => {
            let selected: HashSet<_> = object_ids.into_iter().collect();
            for row in &mut after.objects {
                if selected.contains(&row.id) {
                    row.layer_id = layer_id.clone();
                }
            }
        }
        LayerCommand::SetLocked { locked, .. } => {
            let row = &mut after.layers[layer_position];
            row.locked = locked;
            if row.parent_id.is_some() {
                row.persistent_locked = Some(locked);
            }
            choose_active(&mut after, None)?;
        }
        LayerCommand::SetVisible { visible, .. } => {
            let row = &mut after.layers[layer_position];
            row.visible = visible;
            if row.parent_id.is_some() {
                row.persistent_visible = Some(visible);
            }
            choose_active(&mut after, None)?;
        }
        LayerCommand::SetColor { rgb, .. } => after.layers[layer_position].rgb = rgb,
    }
    if &after != state {
        after.generation = state
            .generation
            .checked_add(1)
            .ok_or(LayerError::LimitExceeded)?;
    }
    validate_layers(&after)?;
    Ok(LayerApplyPlan {
        before: state.clone(),
        after,
        layer_mapping: BTreeMap::new(),
        object_mapping: BTreeMap::new(),
    })
}
