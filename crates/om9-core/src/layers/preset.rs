// SPDX-License-Identifier: LGPL-2.1-or-later
//! Approved local OM9 defaults ported from rust/src/layers.rs, without its FFI.
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
const NAMES: [&str; 32] = [
    "Metal 01", "Metal 02", "Metal 03", "Metal 04", "Gem 01", "Gem 02", "Gem 03", "Gem 04",
    "User 01", "User 02", "User 03", "User 04", "Heads", "Finger", "Cutting", "Creation",
    "User 17", "User 18", "User 19", "User 20", "User 25", "User 26", "User 27", "User 28",
    "User 21", "User 22", "User 23", "User 24", "User 29", "User 30", "User 31", "User 32",
];
const COLORS: [u32; 32] = [
    0x229987, 0x319f49, 0x70c64b, 0xa0cf78, 0x598bc2, 0x4f80cb, 0x7aa7ce, 0xaccae4, 0xe23434,
    0x70c133, 0x3468ff, 0x777777, 0x851ca1, 0xad423c, 0xca7135, 0xdfb126, 0xdf9098, 0xdabb7c,
    0xbd8735, 0xa5c14b, 0x47c7aa, 0x72c6cb, 0x607cab, 0x4d626a, 0x57b397, 0x586076, 0xc6bbb7,
    0x6c5578, 0x57536b, 0x1e555b, 0x0c343f, 0x1b3555,
];
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct PresetSlot(u8);
impl PresetSlot {
    pub fn new(value: u8) -> Result<Self> {
        if (1..=33).contains(&value) {
            Ok(Self(value))
        } else {
            Err(Error::Invalid("OM9 preset slot"))
        }
    }
    pub fn number(self) -> u8 {
        self.0
    }
    pub fn is_custom(self) -> bool {
        (17..=32).contains(&self.0)
    }
    pub fn validate_label(self, name: &str) -> Result<()> {
        if valid_label(self, name) {
            Ok(())
        } else {
            Err(Error::Invalid("OM9 preset label"))
        }
    }
    pub fn name(self) -> &'static str {
        if self.0 == 33 {
            "Lights"
        } else {
            NAMES[(self.0 - 1) as usize]
        }
    }
    pub fn color(self) -> [u8; 4] {
        if self.0 == 33 {
            return [255; 4];
        }
        let rgb = COLORS[(self.0 - 1) as usize];
        [(rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8, 255]
    }
}
impl TryFrom<u8> for PresetSlot {
    type Error = Error;
    fn try_from(value: u8) -> Result<Self> {
        Self::new(value)
    }
}
impl From<PresetSlot> for u8 {
    fn from(value: PresetSlot) -> Self {
        value.0
    }
}

pub(crate) fn valid_label(slot: PresetSlot, name: &str) -> bool {
    if !slot.is_custom() {
        return name == slot.name();
    }
    if name.is_empty()
        || name.len() > 128
        || name.trim() != name
        || name.chars().any(char::is_control)
    {
        return false;
    }
    (1..=33).all(|number| {
        let reserved = PresetSlot(number);
        reserved == slot || !same_name(name, reserved.name())
    })
}
pub(crate) fn same_name(first: &str, second: &str) -> bool {
    first
        .chars()
        .flat_map(char::to_lowercase)
        .eq(second.chars().flat_map(char::to_lowercase))
}
