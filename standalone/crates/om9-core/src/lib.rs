// SPDX-License-Identifier: LGPL-2.1-or-later
//! Owned standalone OM9 foundation. No native host or UI dependency.
#![forbid(unsafe_code)]

pub mod budget;
pub mod capability;
pub mod error;
pub mod evaluation;
pub mod geometry;
pub mod history;
pub mod identity;
pub mod jobs;
pub mod layers;
pub mod model;
pub mod request;
pub mod resource;
pub mod result;
pub mod retained;
mod scope;
pub mod transaction;

pub use error::{Error, Result};
