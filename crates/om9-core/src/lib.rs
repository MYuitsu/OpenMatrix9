// SPDX-License-Identifier: LGPL-2.1-or-later
//! Owned standalone OM9 foundation. No native host or UI dependency.
#![forbid(unsafe_code)]

pub mod brep;
pub mod brep_kernel;
pub mod budget;
pub mod cage;
pub mod capability;
pub mod changes;
pub mod command_schema;
pub mod coords;
pub mod curve_build;
pub mod curve_modify;
pub mod curve_ops;
pub mod curve_surface_ops;
pub mod error;
pub mod evaluation;
pub mod exchange;
pub mod geometry;
pub mod gumball;
pub mod history;
pub mod history_ops;
pub mod identity;
pub mod jobs;
pub mod layer_ops;
pub mod layers;
pub mod measure;
pub mod mesh_csg;
pub mod mesh_repair;
pub mod model;
pub mod modeling_capability;
pub mod object_info;
pub mod offset_curve;
pub mod provenance;
pub mod request;
pub mod resource;
pub mod result;
pub mod retained;
pub mod rhino_semantics;
mod scope;
pub mod selection;
pub mod surface_kernel;
pub mod surface_ops;
pub mod three_dm;
pub mod top11;
pub mod transaction;
pub mod transform_session;
pub mod units;
pub mod utilities;

pub use error::{Error, Result};
