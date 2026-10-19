// SPDX-License-Identifier: LGPL-2.1-or-later
//! Owned standalone OM9 foundation. No native host or UI dependency.
#![forbid(unsafe_code)]

pub mod brep_kernel;
pub mod budget;
pub mod brep;
pub mod cage;
pub mod capability;
pub mod changes;
pub mod command_schema;
pub mod coords;
pub mod curve_build;
pub mod error;
pub mod evaluation;
pub mod geometry;
pub mod gumball;
pub mod history;
pub mod history_ops;
pub mod identity;
pub mod jobs;
pub mod layer_ops;
pub mod layers;
pub mod measure;
pub mod model;
pub mod modeling_capability;
pub mod offset_curve;
pub mod provenance;
pub mod request;
pub mod resource;
pub mod result;
pub mod rhino_semantics;
pub mod retained;
mod scope;
pub mod selection;
pub mod surface_kernel;
pub mod three_dm;
pub mod transaction;
pub mod units;

pub use error::{Error, Result};
