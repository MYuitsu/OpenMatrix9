// SPDX-License-Identifier: LGPL-2.1-or-later
//! The Matrix layers palette organised the way the manual describes it
//! (OM9-L3-03-14-001).
//!
//! Matrix8 book 1, pages 54-57: the colours are "grouped into four: Metals, Gems,
//! User colours, and layer colours on which the parts are typically created by
//! Matrix builders (Heads, Finger or ring size, Cutting, and Creation)". The
//! palette therefore has FOUR groups and SEVEN semantic categories, and the plan
//! asks for exactly those seven names in the palette.
//!
//! Identity is the preset slot number (`PresetSlot`, 1..=33) and never a display
//! label: two rows may legitimately show the same text (a custom slot renamed by
//! the user), and that must not make them the same layer.
use super::PresetSlot;
use serde::{Deserialize, Serialize};

/// The four groups the manual names, in its own order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PaletteGroup {
    Metals,
    Gems,
    User,
    Builders,
}

/// The seven semantic categories the palette shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum LayerCategory {
    Metals,
    Gems,
    User,
    Heads,
    Finger,
    Cutting,
    Creation,
}

impl PaletteGroup {
    pub const ALL: [Self; 4] = [Self::Metals, Self::Gems, Self::User, Self::Builders];

    pub fn label(self) -> &'static str {
        match self {
            Self::Metals => "Metals",
            Self::Gems => "Gems",
            Self::User => "User",
            Self::Builders => "Builders",
        }
    }

    /// The categories inside this group, in palette order.
    pub fn categories(self) -> &'static [LayerCategory] {
        match self {
            Self::Metals => &[LayerCategory::Metals],
            Self::Gems => &[LayerCategory::Gems],
            Self::User => &[LayerCategory::User],
            Self::Builders => &[
                LayerCategory::Heads,
                LayerCategory::Finger,
                LayerCategory::Cutting,
                LayerCategory::Creation,
            ],
        }
    }
}

impl LayerCategory {
    /// The seven names the plan asks the palette to have, in palette order.
    pub const ALL: [Self; 7] = [
        Self::Metals,
        Self::Gems,
        Self::User,
        Self::Heads,
        Self::Finger,
        Self::Cutting,
        Self::Creation,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Metals => "Metals",
            Self::Gems => "Gems",
            Self::User => "User",
            Self::Heads => "Heads",
            Self::Finger => "Finger",
            Self::Cutting => "Cutting",
            Self::Creation => "Creation",
        }
    }

    pub fn group(self) -> PaletteGroup {
        match self {
            Self::Metals => PaletteGroup::Metals,
            Self::Gems => PaletteGroup::Gems,
            Self::User => PaletteGroup::User,
            Self::Heads | Self::Finger | Self::Cutting | Self::Creation => PaletteGroup::Builders,
        }
    }

    /// The preset slots this category shows, in number order. Metals, Gems and
    /// User hold four colours each; the four builder categories hold one slot
    /// each, which is why a category is not a fixed-size range.
    pub fn slots(self) -> &'static [u8] {
        match self {
            Self::Metals => &[1, 2, 3, 4],
            Self::Gems => &[5, 6, 7, 8],
            Self::User => &[9, 10, 11, 12],
            Self::Heads => &[13],
            Self::Finger => &[14],
            Self::Cutting => &[15],
            Self::Creation => &[16],
        }
    }
}

impl PresetSlot {
    /// Which palette category this slot belongs to.
    ///
    /// Slots 17..=32 are the extra user colours: they are User colours without a
    /// category of their own. Slot 33 ("Lights") is the light layer, which is not
    /// a colour category at all, so it has none.
    pub fn category(self) -> Option<LayerCategory> {
        match self.number() {
            1..=4 => Some(LayerCategory::Metals),
            5..=8 => Some(LayerCategory::Gems),
            9..=12 | 17..=32 => Some(LayerCategory::User),
            13 => Some(LayerCategory::Heads),
            14 => Some(LayerCategory::Finger),
            15 => Some(LayerCategory::Cutting),
            16 => Some(LayerCategory::Creation),
            33 => None,
            _ => None,
        }
    }

    pub fn palette_group(self) -> Option<PaletteGroup> {
        self.category().map(LayerCategory::group)
    }

    /// The visible label for a palette row: the category it serves.
    pub fn category_label(self) -> &'static str {
        self.category().map_or("Lights", LayerCategory::label)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_palette_has_the_seven_categories_the_plan_names() {
        let labels: Vec<&str> = LayerCategory::ALL.iter().map(|c| c.label()).collect();
        assert_eq!(
            labels,
            vec![
                "Metals", "Gems", "User", "Heads", "Finger", "Cutting", "Creation"
            ]
        );
        // The manual's four groups, with the builder group carrying four.
        let groups: Vec<&str> = PaletteGroup::ALL.iter().map(|g| g.label()).collect();
        assert_eq!(groups, vec!["Metals", "Gems", "User", "Builders"]);
        let builders: Vec<&str> = PaletteGroup::Builders
            .categories()
            .iter()
            .map(|c| c.label())
            .collect();
        assert_eq!(builders, vec!["Heads", "Finger", "Cutting", "Creation"]);
        // Every category is reachable from exactly one group.
        let mut from_groups = Vec::new();
        for group in PaletteGroup::ALL {
            from_groups.extend_from_slice(group.categories());
        }
        let mut expected = LayerCategory::ALL.to_vec();
        from_groups.sort();
        expected.sort();
        assert_eq!(from_groups, expected);
    }

    #[test]
    fn the_slots_map_to_their_documented_categories() {
        for number in 1..=33 {
            let slot = PresetSlot::new(number).expect("slot");
            let expected = match number {
                1..=4 => Some(LayerCategory::Metals),
                5..=8 => Some(LayerCategory::Gems),
                9..=12 | 17..=32 => Some(LayerCategory::User),
                13 => Some(LayerCategory::Heads),
                14 => Some(LayerCategory::Finger),
                15 => Some(LayerCategory::Cutting),
                16 => Some(LayerCategory::Creation),
                _ => None,
            };
            assert_eq!(slot.category(), expected, "slot {number}");
        }
        assert_eq!(
            PresetSlot::new(33).expect("lights").category_label(),
            "Lights",
            "the light layer is labelled, not categorised"
        );
    }

    #[test]
    fn a_category_lists_the_slots_it_shows() {
        for category in LayerCategory::ALL {
            for number in category.slots() {
                let slot = PresetSlot::new(*number).expect("slot");
                assert_eq!(slot.category(), Some(category), "slot {number}");
                assert_eq!(category.group(), slot.palette_group().expect("group"));
            }
        }
        assert_eq!(LayerCategory::Metals.slots(), &[1, 2, 3, 4]);
        assert_eq!(LayerCategory::Creation.slots(), &[16]);
    }

    #[test]
    fn a_display_label_is_not_identity() {
        // Two different slots may carry the same text once a custom slot is
        // renamed; they stay different layers, because the slot number is the id.
        let first = PresetSlot::new(1).expect("metal");
        let second = PresetSlot::new(17).expect("user");
        assert_ne!(first, second);
        assert_ne!(first.name(), second.name());
        // And the core refuses to let a custom slot take a reserved label.
        let custom = PresetSlot::new(17).expect("user");
        assert!(custom.validate_label("Metal 01").is_err());
        assert!(custom.validate_label("My Yellow").is_ok());
    }
}
