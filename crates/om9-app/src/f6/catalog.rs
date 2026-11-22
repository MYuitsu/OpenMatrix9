// SPDX-License-Identifier: LGPL-2.1-or-later
//! Context groups and command identifiers measured from the installed Matrix90 menu.
use om9_core::{budget::Limits, geometry::Geometry, model::*};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub enum Mode {
    #[default]
    General,
    Curve,
    Gem,
    Surface,
    TSplines,
    User,
    Report,
    Materials,
}

impl Mode {
    pub const ALL: [Self; 8] = [
        Self::General,
        Self::Curve,
        Self::Gem,
        Self::Surface,
        Self::TSplines,
        Self::User,
        Self::Report,
        Self::Materials,
    ];
    pub fn tooltip(self) -> &'static str {
        match self {
            Self::General => "General Modelling",
            Self::Curve => "Curve Mode",
            Self::Gem => "Gem Mode",
            Self::Surface => "Surface Mode",
            Self::TSplines => "Tsplines Mode",
            Self::User => "User Mode",
            Self::Report => "Report Mode",
            Self::Materials => "Material Mode",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::General => "General",
            Self::Curve => "Curve",
            Self::Gem => "Gem",
            Self::Surface => "Surface",
            Self::TSplines => "T-Splines",
            Self::User => "User Defined",
            Self::Report => "Report",
            Self::Materials => "Materials",
        }
    }
}

/// Matrix assigns a category to each menu position. The same command can have a
/// different category when offered on another selection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub enum Category {
    #[default]
    Menu,
    None,
    Label,
    Curves,
    Surface,
    Solid,
    Transform,
    Art,
    Builder,
    Tools,
    Gems,
    Setting,
    Cutters,
    Targets,
    Clayoo,
    Render,
}

impl Category {
    pub const ALL: [Self; 16] = [
        Self::Menu,
        Self::None,
        Self::Label,
        Self::Curves,
        Self::Surface,
        Self::Solid,
        Self::Transform,
        Self::Art,
        Self::Builder,
        Self::Tools,
        Self::Gems,
        Self::Setting,
        Self::Cutters,
        Self::Targets,
        Self::Clayoo,
        Self::Render,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Menu => "Menu",
            Self::None => "None",
            Self::Label => "Label",
            Self::Curves => "Curves",
            Self::Surface => "Surface",
            Self::Solid => "Solid",
            Self::Transform => "Transform",
            Self::Art => "Art",
            Self::Builder => "Builder",
            Self::Tools => "Tools",
            Self::Gems => "Gems",
            Self::Setting => "Setting",
            Self::Cutters => "Cutters",
            Self::Targets => "Targets",
            Self::Clayoo => "Clayoo",
            Self::Render => "Render",
        }
    }
    pub fn marker(self) -> Option<[u8; 3]> {
        Some(match self {
            Self::Menu => [255, 255, 255],
            Self::Curves => [249, 214, 0],
            Self::Surface => [55, 156, 62],
            Self::Solid => [129, 64, 169],
            Self::Transform => [255, 84, 0],
            Self::Art => [217, 217, 15],
            Self::Builder => [216, 45, 51],
            Self::Tools => [47, 24, 201],
            Self::Gems => [25, 117, 226],
            Self::Setting => [176, 18, 127],
            Self::Cutters => [220, 113, 59],
            Self::Targets => [159, 248, 98],
            Self::Clayoo => [175, 56, 44],
            Self::Render => [33, 186, 186],
            Self::None | Self::Label => return None,
        })
    }
}

#[derive(Deserialize)]
pub struct Catalog {
    pub actions: Vec<(String, String)>,
    pub groups: Vec<Group>,
}

pub static CATALOG: LazyLock<Catalog> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../assets/f6_actions.json")).expect("F6 action catalog")
});

#[derive(Deserialize)]
pub struct Group {
    pub categories: Vec<Category>,
    pub mode: Mode,
    pub name: String,
    types: Vec<String>,
    object: Option<String>,
    sub: Option<String>,
    class: Option<String>,
    named: Option<String>,
    pub rows: Vec<usize>,
}

struct Pick<'a> {
    kind: &'static str,
    class: &'static str,
    object: &'static str,
    sub: &'static str,
    name: &'a str,
}

pub(crate) fn object_kind(document: &Document, entity: &Entity) -> &'static str {
    if entity.extension(GEM_EXTENSION_SLOT).is_some()
        || matches!(entity.geometry, Geometry::Instance(_))
            && om9_core::gem::gem_record_in(document, entity.id).is_some()
    {
        "Gem"
    } else if entity.extension(FOUR_RAIL_PROFILE_EXTENSION_SLOT).is_some() {
        "FourRailProfile"
    } else if om9_core::model::is_tspline_box_cage(entity) || entity.extension(om9_core::model::TSPLINE_FROM_LINES_SLOT).is_some() {
        "TSpline"
    } else {
        match &entity.geometry {
            Geometry::Curve(curve) => {
                if document.tolerance.as_ref().is_some_and(|t| {
                    om9_core::rhino_semantics::classify_curve(
                        curve,
                        t.absolute_mm / document.page_unit.factor(),
                        &Limits::default(),
                    )
                    .is_ok_and(|shape| shape.closed && shape.planar)
                }) {
                    "PlanarClosedCurve"
                } else {
                    "Curve"
                }
            }
            Geometry::Surface(_) => "Surface",
            Geometry::Brep(brep) if brep.topology().faces.len() == 1 => "Surface",
            Geometry::Brep(_) | Geometry::NativeBrep(_) => "Polysurface",
            Geometry::Mesh(_) => "Mesh",
            _ => "Other",
        }
    }
}

impl<'a> Pick<'a> {
    fn from_entity(document: &Document, entity: &'a Entity) -> Self {
        let kind = object_kind(document, entity);
        let recipe = document
            .history
            .owner(entity.id)
            .and_then(|id| document.history.recipes.get(&id));
        let (object, sub) = if entity
            .extension(crate::session::gem_placement::CURVE_ROW_SLOT)
            .is_some()
        {
            ("ObjectOnCrv", "GemOnCrv")
        } else if entity.extension(SETTINGS_BEZEL_SLOT).is_some() {
            ("Bezel", "")
        } else if entity.extension(SETTINGS_PRONG_SLOT).is_some() {
            ("Prong", "")
        } else if entity.extension(HEAD_EXTENSION_SLOT).is_some()
            || entity.extension(SETTINGS_HEAD_SLOT).is_some()
        {
            ("Head", "")
        } else if entity.extension(CUTTER_EXTENSION_SLOT).is_some() {
            ("Cutter", "")
        } else {
            match recipe.map(|r| r.command.as_str()) {
                Some("RingRail") => ("RingRail", "Round"),
                Some("OutsideRingRail") => ("OutsideRingRail", ""),
                Some("Signet" | om9_core::ring_builder::SIGNET_RING_COMMAND) => ("Signet", ""),
                _ => ("", ""),
            }
        };
        Self {
            kind,
            object,
            sub,
            class: if om9_core::cage::is_cage(entity) {
                "CRhinoMorphControl"
            } else if kind == "TSpline" {
                "CTMeshObject"
            } else {
                ""
            },
            name: &entity.label,
        }
    }
}

impl Group {
    pub(super) fn legacy_fields_match(
        &self,
        fields: &std::collections::BTreeMap<&str, &str>,
    ) -> bool {
        for (key, expected) in [
            ("ID", self.object.as_deref()),
            ("subID", self.sub.as_deref()),
            ("name", self.named.as_deref()),
        ] {
            if fields.get(key).copied() != expected {
                return false;
            }
        }
        for (i, key) in ["type1", "type2", "type3"].iter().enumerate() {
            let expected = self.types.get(i).map(String::as_str).unwrap_or("None");
            let raw = fields.get(key).copied().unwrap_or("ObjNone");
            if expected == "ClassName" {
                if self.class.as_deref() != Some(raw) {
                    return false;
                }
            } else if !expected.eq_ignore_ascii_case(raw.strip_prefix("Obj").unwrap_or(raw)) {
                return false;
            }
        }
        true
    }
    pub fn scope_key(&self) -> String {
        format!(
            "{}/{}/{}/{}/{}/{}",
            self.name,
            self.types.join("+"),
            self.object.as_deref().unwrap_or(""),
            self.sub.as_deref().unwrap_or(""),
            self.class.as_deref().unwrap_or(""),
            self.named.as_deref().unwrap_or("")
        )
    }
    fn matches(&self, picks: &[&Pick<'_>]) -> bool {
        if let Some(name) = &self.named {
            return picks.iter().any(|p| name == p.name);
        }
        if let Some(id) = &self.object {
            return picks
                .first()
                .is_some_and(|p| p.object == id && self.sub.as_deref().is_none_or(|s| s == p.sub))
                && (picks.len() == 1 || self.types.iter().any(|t| t == "All"));
        }
        if self.types.is_empty() {
            return picks.is_empty() && self.name == "Empty";
        }
        if let Some(t @ ("GemsAll" | "GemsAll3")) = self.types.first().map(String::as_str) {
            // In the compiled FindMenuGroup the two-gem branch selects the
            // same factory group as three or more gems.
            return t == "GemsAll3" && picks.len() >= 2 && picks.iter().all(|p| p.kind == "Gem");
        }
        for (index, expected) in self.types.iter().enumerate() {
            if expected == "All" {
                return picks.len() > index
                    || (index == 1 && self.class.is_some() && picks.len() == 1);
            }
            let Some(pick) = picks.get(index) else {
                return false;
            };
            if expected == "ClassName" {
                if self.class.as_deref() != Some(pick.class) {
                    return false;
                }
            } else if expected != pick.kind {
                return false;
            }
        }
        self.types.len() == picks.len() || (self.types.len() == 3 && picks.len() > 3)
    }
}

impl Catalog {
    pub fn tspline_component_group(&self, mode: Mode, kind: om9_scene::scene::PickKind) -> Option<&Group> {
        use om9_scene::scene::PickKind;
        let tag = match kind {
            PickKind::Vertex => "GripTSplineVertex",
            PickKind::Edge => "GripTSplineEdge",
            PickKind::Face => "GripTSplineFace",
            PickKind::Object => return None,
        };
        self.groups.iter().find(|g| g.mode == mode && g.types.first().is_some_and(|t| t == tag))
    }
    pub fn grip_group(&self, mode: Mode, class: &str) -> Option<&Group> {
        self.groups
            .iter()
            .find(|g| g.mode == mode && g.class.as_deref() == Some(class))
    }
    pub fn group(
        &self,
        mode: Mode,
        document: &Document,
        selected: &[om9_core::identity::ObjectId],
    ) -> Option<&Group> {
        let groups = || self.groups.iter().filter(|g| g.mode == mode);
        if mode == Mode::User {
            return groups().next();
        }
        let picks: Vec<_> = selected
            .iter()
            .filter_map(|id| document.entities.get(id))
            .map(|entity| Pick::from_entity(document, entity))
            .collect();
        let mut ordered: Vec<_> = picks.iter().collect();
        if let Some(group) = groups().find(|g| g.matches(&ordered)) {
            return Some(group);
        }
        if let [p] = ordered.as_slice()
            && let Some(group) = groups().find(|g| {
                g.types.first().is_some_and(|t| t == p.kind)
                    && g.types.get(1).is_some_and(|t| t == "All")
            })
        {
            return Some(group);
        }
        let mut seen = std::collections::BTreeSet::new();
        ordered.retain(|p| seen.insert(p.kind));
        groups()
            .find(|g| g.matches(&ordered))
            .or_else(|| groups().find(|g| g.name == "Default"))
    }
}
