//! Deterministic `OpenRTB` 2.5/2.6 bid request generator.
//!
//! The crate generates wire JSON, not types, so it works with every `OpenRTB` parser.
//! One seed plus one configuration produces byte-identical output.
//!
//! Both 2.x revisions share one wire model: [`SpecVersion::V25`] drops the fields
//! the 2.6 revision added, and [`SpecVersion::V26`] is the default.
//!
//! This is the M0 vertical slice: the write-only wire model, the byte-stable
//! writer, the crate's own PRNG and the `SiteDisplay` profile. The typed
//! builder, the other profiles, the CLI corpus and the product documentation
//! land in M1.

#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[expect(
    clippy::redundant_pub_crate,
    reason = "the module is crate-private; `pub` would trip `unreachable_pub`"
)]
mod code;
#[expect(
    clippy::redundant_pub_crate,
    reason = "the module is crate-private; `pub` would trip `unreachable_pub`"
)]
mod currency;
#[expect(
    clippy::redundant_pub_crate,
    reason = "the module is crate-private; `pub` would trip `unreachable_pub`"
)]
mod flag;
#[expect(
    clippy::redundant_pub_crate,
    reason = "the module is crate-private; `pub` would trip `unreachable_pub`"
)]
mod generate;
#[expect(
    clippy::redundant_pub_crate,
    reason = "the module is crate-private; `pub` would trip `unreachable_pub`"
)]
mod ids;
#[expect(
    clippy::redundant_pub_crate,
    reason = "the module is crate-private; `pub` would trip `unreachable_pub`"
)]
mod money;
mod profile;
#[expect(
    clippy::redundant_pub_crate,
    reason = "the module is crate-private; `pub` would trip `unreachable_pub`"
)]
mod raw;
mod request;
#[expect(
    clippy::redundant_pub_crate,
    reason = "the module is crate-private; `pub` would trip `unreachable_pub`"
)]
mod rng;
mod seed;
mod version;
#[expect(
    clippy::redundant_pub_crate,
    reason = "the module is crate-private; `pub` would trip `unreachable_pub`"
)]
mod wire;
#[expect(
    clippy::redundant_pub_crate,
    reason = "the module is crate-private; `pub` would trip `unreachable_pub`"
)]
mod write;

pub use profile::{Profile, UnknownProfile};
pub use request::Request;
pub use seed::Seed;
pub use version::{SpecVersion, UnknownSpecVersion};
