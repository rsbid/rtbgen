//! Deterministic `OpenRTB` 2.5/2.6 bid request generator.
//!
//! The crate generates wire JSON, not types, so it works with every `OpenRTB` parser.
//! One seed plus one configuration produces byte-identical output.
//!
//! Both 2.x revisions share one wire model: [`SpecVersion::V25`] drops the fields
//! the 2.6 revision added, and [`SpecVersion::V26`] is the default.
//!
//! This is the M0 skeleton: the profile, seed and spec-version types are in place;
//! the request model, writer and corpus land next.

#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod profile;
mod seed;
mod version;

pub use profile::{Profile, UnknownProfile};
pub use seed::Seed;
pub use version::{SpecVersion, UnknownSpecVersion};
