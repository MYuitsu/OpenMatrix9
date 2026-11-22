// SPDX-License-Identifier: LGPL-2.1-or-later
//! Tool identities used by the workspace layout. Localized captions never identify a panel.
use crate::session::channel_cutters::RowTool;
use crate::session::gem::GemTool;
use crate::session::gem_cluster::ClusterTool;
use crate::session::gem_cutters::CutterTool;
use crate::session::gem_pave::PaveTool;
use crate::session::gem_placement::GemPlacementTool;
use om9_core::builder::BuilderKind;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ToolPanel {
    Gem(GemTool),
    Placement(GemPlacementTool),
    Cluster(ClusterTool),
    Pave(PaveTool),
    Cutter(CutterTool),
    Row(RowTool),
    Builder(BuilderKind),
    HostedBuilder,
    Metal,
    Properties,
    F6,
    CommandHistory,
    GemMap,
    GemReport,
    Health,
    History,
    Styles,
    Profiles,
    Addons,
    ProjectManager,
    ProjectDetails,
    RailShapes,
    SessionOptions,
    StlOptions,
    StlProgress,
    Gumball,
}

impl ToolPanel {
    /// Working space for the Gem builders when first floated from a dock.
    pub(crate) fn preferred_float_size(self) -> Option<[f32; 2]> {
        match self {
            Self::Gem(GemTool::Loader) => Some([320., 525.]),
            Self::Placement(_) => Some([370., 475.]),
            _ => None,
        }
    }

    pub const ALL: &[ToolPanel] = &[
        ToolPanel::Gem(GemTool::Loader),
        ToolPanel::Gem(GemTool::Custom),
        ToolPanel::Gem(GemTool::Emerald),
        ToolPanel::Gem(GemTool::Baguette),
        ToolPanel::Gem(GemTool::Positioner),
        ToolPanel::Placement(GemPlacementTool::OnCurve),
        ToolPanel::Placement(GemPlacementTool::OnSurface),
        ToolPanel::Cluster(ClusterTool::Halo),
        ToolPanel::Cluster(ClusterTool::Cluster),
        ToolPanel::Cluster(ClusterTool::Channel),
        ToolPanel::Cluster(ClusterTool::Tapered),
        ToolPanel::Cluster(ClusterTool::Between),
        ToolPanel::Cluster(ClusterTool::Orient),
        ToolPanel::Pave(PaveTool::Auto),
        ToolPanel::Pave(PaveTool::Manual),
        ToolPanel::Pave(PaveTool::Azure),
        ToolPanel::Pave(PaveTool::Prong),
        ToolPanel::Pave(PaveTool::Sphere),
        ToolPanel::Pave(PaveTool::Springs),
        ToolPanel::Cutter(CutterTool::GemCutter),
        ToolPanel::Cutter(CutterTool::GemCutterLibrary),
        ToolPanel::Row(RowTool::ChannelBuilder),
        ToolPanel::Row(RowTool::MicroProng),
        ToolPanel::Row(RowTool::BrightCutChannel),
        ToolPanel::Row(RowTool::BrightCutter),
        ToolPanel::Builder(BuilderKind::RingRail),
        ToolPanel::Builder(BuilderKind::RailProfile),
        ToolPanel::Builder(BuilderKind::RailSweep),
        ToolPanel::Builder(BuilderKind::OutsideRail),
        ToolPanel::Builder(BuilderKind::RingResizer),
        ToolPanel::Builder(BuilderKind::EternityBand),
        ToolPanel::Builder(BuilderKind::SignetRing),
        ToolPanel::Builder(BuilderKind::RaisedBand),
        ToolPanel::Builder(BuilderKind::AwardRing),
        ToolPanel::HostedBuilder,
        ToolPanel::Metal,
        ToolPanel::Properties,
        ToolPanel::F6,
        ToolPanel::CommandHistory,
        ToolPanel::GemMap,
        ToolPanel::GemReport,
        ToolPanel::Health,
        ToolPanel::History,
        ToolPanel::Styles,
        ToolPanel::Profiles,
        ToolPanel::Addons,
        ToolPanel::ProjectManager,
        ToolPanel::ProjectDetails,
        ToolPanel::RailShapes,
        ToolPanel::SessionOptions,
        ToolPanel::StlOptions,
        ToolPanel::StlProgress,
        ToolPanel::Gumball,
    ];
    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|panel| panel.key() == key)
    }

    pub fn initial_float_size(self) -> Option<[f32; 2]> {
        match self {
            Self::CommandHistory => Some([640., 280.]),
            Self::F6 => {
                let rect = crate::dock_layout::FloatRect::command_search_default();
                Some([rect.width,rect.height])
            }
            Self::ProjectDetails => Some([740., 580.]),
            Self::RailShapes => Some([520., 440.]),
            Self::SessionOptions => Some([350., 320.]),
            Self::StlOptions => Some([360., 215.]),
            Self::StlProgress => Some([300., 125.]),
            Self::ProjectManager => Some([820., 560.]),
            Self::Addons | Self::Health | Self::History => Some([560., 420.]),
            Self::Properties
            | Self::GemMap
            | Self::GemReport
            | Self::Profiles
            | Self::Styles
            | Self::Gumball => Some([360., 320.]),
            _ => None,
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Self::Gem(tool) => tool.command(),
            Self::Placement(tool) => tool.command(),
            Self::Cluster(tool) => tool.command(),
            Self::Pave(tool) => tool.command(),
            Self::Cutter(tool) => tool.feature(),
            Self::Row(tool) => tool.feature(),
            Self::Builder(tool) => tool.command(),
            Self::HostedBuilder => "hosted-builder",
            Self::Metal => "metal-weights",
            Self::F6 => "om9-command-search",
            Self::CommandHistory => "om9-command-history",
            Self::Properties => "om9-gem-properties",
            Self::GemMap => "om9-gem-map",
            Self::GemReport => "om9-gem-report",
            Self::Health => "om9-object-health-report",
            Self::History => "om9-history-report",
            Self::Styles => "om9-builder-styles",
            Self::Profiles => "om9-builder-profile-library",
            Self::Addons => "om9-addons",
            Self::ProjectManager => "project-manager-overview",
            Self::ProjectDetails => "project-metadata",
            Self::RailShapes => "ring-rail-shape-library",
            Self::SessionOptions => "om9-addon-dialog",
            Self::StlOptions => "stl-profile",
            Self::StlProgress => "stl-export-progress",
            Self::Gumball => "gumball-settings",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::Gem(tool) => tool.title(),
            Self::Placement(tool) => tool.title(),
            Self::Cluster(tool) => tool.title(),
            Self::Pave(tool) => tool.title(),
            Self::Cutter(tool) => tool.label(),
            Self::Row(tool) => tool.label(),
            Self::Builder(tool) => tool.title(),
            Self::HostedBuilder => "Builder",
            Self::Metal => "Metal Weights",
            Self::F6 => "F6",
            Self::CommandHistory => "Command History",
            Self::Properties => "Gem Properties",
            Self::GemMap => "Gem Map",
            Self::GemReport => "Gem Reporter",
            Self::Health => "Object Health Report",
            Self::History => "History",
            Self::Styles => "Builder Styles",
            Self::Profiles => "Profile Library",
            Self::Addons => "Add-ons",
            Self::ProjectManager => "Project Manager",
            Self::ProjectDetails => "Project details",
            Self::RailShapes => "Ring Rail shapes",
            Self::SessionOptions => "Command options",
            Self::StlOptions => "STL Export",
            Self::StlProgress => "STL Export Progress",
            Self::Gumball => "Gumball settings",
        }
    }
}
