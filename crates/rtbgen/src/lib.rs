//! Deterministic `OpenRTB` 2.5/2.6 bid request generator for tests, fuzzing and
//! load runs.
//!
//! The crate generates wire JSON, not types, so it works with every `OpenRTB`
//! parser: one seed plus one configuration produces byte-identical output.
//!
//! ```rust
//! use rtbgen::{Currency, Deals, Floor, ImpressionCount, Profile, Request, Seed};
//!
//! let req = Request::builder(Profile::AppRewarded)
//!     .seed(Seed::new(42))
//!     .impressions_exact(ImpressionCount::new(2))
//!     .deals(Deals::new(3))
//!     .floor(Floor::cpm(1.50, Currency::USD))
//!     .build();
//!
//! let json: String = req.to_json_string();
//! assert!(json.starts_with('{'), "a request is a JSON object");
//! ```
//!
//! ## Which call do I need?
//!
//! | You want | Do this |
//! |---|---|
//! | A valid request for a test | [`Request::builder`] with the matching [`Profile`] |
//! | An `OpenRTB` 2.5 request | add `.spec(SpecVersion::V25)`; 2.6 is the default |
//! | Extra impressions, a marketplace or a floor | [`ImpressionCount`], [`Deals`], [`Floor`] |
//! | An exact bug reproduction | keep the [`Seed`] in the test, not the JSON |
//! | Profiles as data | [`Profile::from_name`] and [`Profile::ALL`] |
//! | Fixtures without the API | the `rtbgen` command-line interface |
//! | Fuzzing where every byte matters | the `arbitrary` feature: `Request: arbitrary::Arbitrary` |
//! | Property tests that shrink | the `proptest` feature: `rtbgen::proptest::request()` |
//! | Serializing through `serde` | the `serde` feature: `serde_json::to_string(&request)` |
//! | Your own deterministic source | [`Request::from_entropy`] and the [`Entropy`] trait |
//!
//! Both 2.x revisions share one wire model: [`SpecVersion::V25`] drops the
//! fields the 2.6 revision added, and [`SpecVersion::V26`] is the default.
//! `OpenRTB` 3.0 is a different object model and is planned for a future version
//! of this project; it will not change 2.x output.
//!
//! ## Features
//!
//! | Feature | Adds | Default |
//! |---|---|---|
//! | `alloc` | `Request::to_json_string()` | yes |
//! | `arbitrary` | `arbitrary::Arbitrary` for [`Request`] | no |
//! | `proptest` | `rtbgen::proptest::request()` | no |
//! | `serde` | `serde::Serialize` for [`Request`] (for `serde_json`) | no |
//!
//! No feature changes the bytes of an existing `(spec, profile, seed, knobs)`
//! tuple: the adapters only choose values, the writer is always the same.
//! The `serde` adapter embeds the writer's JSON as a raw value, so it is
//! meaningful only with `serde_json`; another serializer sees the raw-value
//! marker instead of the request object.
//!
//! The core is `no_std` and has no unconditional runtime dependencies.

#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "arbitrary")]
mod arbitrary_support;
mod builder;
#[expect(
    clippy::redundant_pub_crate,
    reason = "the module is crate-private; `pub` would trip `unreachable_pub`"
)]
mod code;
pub mod corpus;
mod currency;
mod deals;
#[expect(
    clippy::redundant_pub_crate,
    reason = "the module is crate-private; `pub` would trip `unreachable_pub`"
)]
mod flag;
mod floor;
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
mod impressions;
#[expect(
    clippy::redundant_pub_crate,
    reason = "the module is crate-private; `pub` would trip `unreachable_pub`"
)]
mod money;
mod profile;
#[cfg(feature = "proptest")]
pub mod proptest;
#[expect(
    clippy::redundant_pub_crate,
    reason = "the module is crate-private; `pub` would trip `unreachable_pub`"
)]
mod raw;
mod request;
mod rng;
mod seed;
#[cfg(feature = "serde")]
mod serde_support;
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

pub use builder::RequestBuilder;
pub use currency::{Currency, UnknownCurrency};
pub use deals::Deals;
pub use floor::Floor;
pub use impressions::ImpressionCount;
pub use profile::{Profile, UnknownProfile};
pub use request::Request;
pub use rng::{BytesEntropy, Entropy, RngEntropy, Share, Span};
pub use seed::Seed;
pub use version::{SpecVersion, UnknownSpecVersion};
