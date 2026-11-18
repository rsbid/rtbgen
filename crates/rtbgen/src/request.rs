use core::fmt;

#[cfg(feature = "alloc")]
use alloc::string::String;

use crate::seed::Seed;
use crate::version::SpecVersion;
use crate::wire::RequestModel;

/// A generated `OpenRTB` bid request.
///
/// The writer is pure: the same model and spec version always produce the
/// same bytes. The typed builder is the M1 entry point; this milestone exposes
/// the writer and the internal model.
#[derive(Clone, Copy, Debug)]
pub struct Request {
    spec: SpecVersion,
    model: RequestModel,
}

impl Request {
    #[must_use]
    pub(crate) const fn from_model(spec: SpecVersion, model: &RequestModel) -> Self {
        Self {
            spec,
            model: *model,
        }
    }

    /// Builds the `SiteDisplay` fixture for `seed`.
    #[must_use]
    pub fn site_display(seed: Seed) -> Self {
        crate::generate::site_display(seed)
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
