use core::fmt;

#[cfg(feature = "alloc")]
use alloc::string::String;

use crate::builder::RequestBuilder;
use crate::generate::{self, Config};
use crate::ids::RequestId;
use crate::profile::Profile;
use crate::rng::Entropy;
use crate::seed::Seed;
use crate::version::SpecVersion;
use crate::wire::RequestModel;

/// A generated `OpenRTB` bid request.
///
/// Build it with [`Request::builder`]; the writer is pure, so the same model and
/// spec version always produce the same bytes. With the `arbitrary` feature the
/// type also implements `arbitrary::Arbitrary`; with `serde` it implements
/// `serde::Serialize`.
#[derive(Clone, Copy, Debug)]
pub struct Request {
    profile: Profile,
    seed: Option<Seed>,
    spec: SpecVersion,
    model: RequestModel,
}

impl Request {
    /// Starts a request of the given shape.
    #[must_use]
    pub const fn builder(profile: Profile) -> RequestBuilder {
        RequestBuilder::new(profile)
    }

    pub(crate) const fn from_model(
        profile: Profile,
        seed: Option<Seed>,
        spec: SpecVersion,
        model: &RequestModel,
    ) -> Self {
        Self { profile, seed, spec, model: *model }
    }

    /// Generates a request from an entropy source.
    ///
    /// The request id is drawn first, then the profile body, so the same
    /// entropy sequence always produces byte-identical output. A request built
    /// this way has no seed: [`Request::seed`] returns `None`.
    ///
    /// [`RngEntropy`](crate::RngEntropy) and [`BytesEntropy`](crate::BytesEntropy)
    /// are the sources this crate ships; any deterministic implementation of
    /// [`Entropy`](crate::Entropy) works.
    #[must_use]
    pub fn from_entropy(profile: Profile, spec: SpecVersion, entropy: &mut impl Entropy) -> Self {
        let id = RequestId::from_entropy(entropy);
        let config = Config::new(id, None, None, None);
        let model = generate::build_with(profile, &config, entropy);

        Self::from_model(profile, None, spec, &model)
    }

    /// Returns the profile the request was generated from.
    #[must_use]
    pub const fn profile(&self) -> Profile {
        self.profile
    }

    /// Returns the seed the request was built with.
    ///
    /// A request generated from an entropy source ([`Request::from_entropy`] or
    /// the `arbitrary` adapter) has no seed, so this returns `None`.
    #[must_use]
    pub const fn seed(&self) -> Option<Seed> {
        self.seed
    }

    /// Returns the `OpenRTB` revision the request is written for.
    #[must_use]
    pub const fn spec(&self) -> SpecVersion {
        self.spec
    }

    /// Writes the request as compact JSON into `sink`.
    ///
    /// The writer never allocates and never pretty-prints. Key order is fixed.
    ///
    /// # Errors
    ///
    /// Returns the sink's error. A sink that fails mid-object leaves a partial
    /// document; the writer does not buffer or roll back.
    pub fn write_json(&self, sink: &mut impl fmt::Write) -> fmt::Result {
        crate::write::request(sink, &self.model, self.spec)
    }

    /// Returns the request as a compact JSON string.
    ///
    /// This is the only allocating path; it is available with the default
    /// `alloc` feature. Writing into a `String` cannot fail.
    #[cfg(feature = "alloc")]
    #[must_use]
    pub fn to_json_string(self) -> String {
        let mut out = String::new();
        if self.write_json(&mut out).is_err() {
            out.clear();
        }
        out
    }
}
