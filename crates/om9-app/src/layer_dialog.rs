// SPDX-License-Identifier: LGPL-2.1-or-later
//! Owned modal state; opaque handles never grant document mutation authority.
use om9_core::{
    Result,
    budget::{Limits, Usage},
    identity::{DocumentId, LayerId, SessionId, SourceStamp},
    layers::PresetSlot,
    transaction::OperationContext,
};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerDialogKind {
    Rename,
    Color,
    Show,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayerDialogToken {
    pub(crate) context: OperationContext,
    pub(crate) owner: (DocumentId, SessionId, u64, u64, u64, u64),
    pub(crate) serial: u64,
}
impl LayerDialogToken {
    pub fn authority(self) -> SessionId {
        self.context.authority()
    }
    pub fn serial(self) -> u64 {
        self.serial
    }
}
pub struct HiddenChoice {
    pub(crate) source: SourceStamp,
    pub(crate) label: String,
    pub(crate) selected: bool,
}
impl HiddenChoice {
    pub fn source(&self) -> SourceStamp {
        self.source
    }
    pub fn label(&self) -> &str {
        &self.label
    }
    pub fn selected(&self) -> bool {
        self.selected
    }
}
pub(crate) enum Draft {
    Rename {
        slot: PresetSlot,
        target: Option<LayerId>,
        name: String,
    },
    Color {
        slot: PresetSlot,
        target: Option<LayerId>,
        color: [u8; 4],
    },
    Show {
        entries: Vec<HiddenChoice>,
    },
}
pub struct LayerDialog {
    pub(crate) token: LayerDialogToken,
    pub(crate) draft: Draft,
}
impl LayerDialog {
    pub fn token(&self) -> LayerDialogToken {
        self.token
    }
    pub fn kind(&self) -> LayerDialogKind {
        match self.draft {
            Draft::Rename { .. } => LayerDialogKind::Rename,
            Draft::Color { .. } => LayerDialogKind::Color,
            Draft::Show { .. } => LayerDialogKind::Show,
        }
    }
    pub fn name(&self) -> Option<&str> {
        if let Draft::Rename { name, .. } = &self.draft {
            Some(name)
        } else {
            None
        }
    }
    pub fn color(&self) -> Option<[u8; 4]> {
        if let Draft::Color { color, .. } = self.draft {
            Some(color)
        } else {
            None
        }
    }
    pub fn entries(&self) -> &[HiddenChoice] {
        if let Draft::Show { entries } = &self.draft {
            entries
        } else {
            &[]
        }
    }
    pub(crate) fn usage(&self, limits: &Limits) -> Result<Usage> {
        let mut usage = Usage::array::<Self>(1)?;
        // One bounded UI input copy and a bounded error/formatting reserve.
        usage.add(
            Usage {
                bytes: 1024,
                ..Usage::default()
            },
            limits,
        )?;
        match &self.draft {
            Draft::Rename { name, .. } => usage.add(
                Usage {
                    bytes: name.capacity() as u64,
                    ..Usage::default()
                },
                limits,
            )?,
            Draft::Show { entries } => {
                usage.add(Usage::array::<HiddenChoice>(entries.capacity())?, limits)?;
                for entry in entries {
                    usage.add(
                        Usage {
                            bytes: entry.label.capacity() as u64,
                            ..Usage::default()
                        },
                        limits,
                    )?;
                }
            }
            _ => {}
        }
        Ok(usage)
    }
}
