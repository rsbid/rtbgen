//! Wire identifiers.
//!
//! Every identifier owns its prefix and its width, so a call site cannot write
//! a publisher id into a site id and cannot forget the format. [`fmt::Display`]
//! yields the string contents; the writer adds the JSON quotes.
//!
//! Every id type defaults to `0`: the allocation-free model needs a placeholder
//! before a profile fills it in, and `0` is a representable wire value.

use core::fmt;

use crate::seed::Seed;

macro_rules! id_type {
    ($(#[$doc:meta])* $name:ident, $format:literal) => {
        $(#[$doc])*
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
        pub(crate) struct $name(u64);

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, $format, self.0)
            }
        }
    };
}

id_type!(
    /// The id of a bid request, formatted as `req-` plus six zero-padded digits.
    RequestId,
    "req-{:06}"
);

impl RequestId {
    /// Derives the request id from the seed that produced the request.
    ///
    /// The seed is consumed as an id here, at the boundary between the two
    /// types; the generator never takes the raw seed value.
    #[must_use]
    pub(crate) const fn from_seed(seed: Seed) -> Self {
        Self(seed.into_inner())
    }
}

id_type!(
    /// The id of one impression, written as its decimal value.
    ImpId,
    "{}"
);

impl ImpId {
    /// Wraps a numeric impression id.
    #[must_use]
    pub(crate) fn new(value: u32) -> Self {
        Self(u64::from(value))
    }
}

id_type!(
    /// The id of a private marketplace deal, formatted as `deal-` plus the value.
    DealId,
    "deal-{}"
);

#[cfg(test)]
impl DealId {
    /// Wraps a numeric deal id; used by the maximal fixture.
    #[must_use]
    pub(crate) const fn new(value: u64) -> Self {
        Self(value)
    }
}

id_type!(
    /// The id of a banner, formatted as `b` plus the value.
    BannerId,
    "b{}"
);

impl BannerId {
    /// Wraps a numeric banner id.
    #[must_use]
    pub(crate) const fn new(value: u64) -> Self {
        Self(value)
    }
}

id_type!(
    /// The id of a site, formatted as `site-` plus the value.
    SiteId,
    "site-{}"
);

impl SiteId {
    /// Wraps a numeric site id.
    #[must_use]
    pub(crate) const fn new(value: u64) -> Self {
        Self(value)
    }
}

id_type!(
    /// The id of an app, formatted as `a` plus the value.
    AppId,
    "a{}"
);

#[cfg(test)]
impl AppId {
    /// Wraps a numeric app id; used by the maximal fixture.
    #[must_use]
    pub(crate) const fn new(value: u64) -> Self {
        Self(value)
    }
}

id_type!(
    /// The id of a publisher, formatted as `pub-` plus the value.
    PublisherId,
    "pub-{}"
);

impl PublisherId {
    /// Wraps a numeric publisher id.
    #[must_use]
    pub(crate) const fn new(value: u64) -> Self {
        Self(value)
    }
}

id_type!(
    /// The id of a user, formatted as `u` plus the value.
    UserId,
    "u{}"
);

impl UserId {
    /// Wraps a numeric user id.
    #[must_use]
    pub(crate) const fn new(value: u64) -> Self {
        Self(value)
    }
}

id_type!(
    /// The buyer's user id, formatted as `buyer-` plus the value.
    BuyerUserId,
    "buyer-{}"
);

impl BuyerUserId {
    /// Wraps a numeric buyer user id.
    #[must_use]
    pub(crate) const fn new(value: u64) -> Self {
        Self(value)
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use std::string::ToString;

    use super::{
        AppId, BannerId, BuyerUserId, DealId, ImpId, PublisherId, RequestId, SiteId, UserId,
    };
    use crate::seed::Seed;

    #[test]
    fn every_identifier_owns_its_prefix() {
        let cases = [
            (
                RequestId::from_seed(Seed::new(42)).to_string(),
                "req-000042",
            ),
            (
                RequestId::from_seed(Seed::new(u64::MAX)).to_string(),
                "req-18446744073709551615",
            ),
            (ImpId::new(7).to_string(), "7"),
            (DealId::new(1).to_string(), "deal-1"),
            (BannerId::new(7).to_string(), "b7"),
            (SiteId::new(496).to_string(), "site-496"),
            (AppId::new(3).to_string(), "a3"),
            (PublisherId::new(654).to_string(), "pub-654"),
            (UserId::new(913_513).to_string(), "u913513"),
            (BuyerUserId::new(987_883).to_string(), "buyer-987883"),
        ];

        for (actual, expected) in cases {
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn defaults_are_the_model_placeholders() {
        assert_eq!(RequestId::default().to_string(), "req-000000");
        assert_eq!(ImpId::default().to_string(), "0");
    }
}
