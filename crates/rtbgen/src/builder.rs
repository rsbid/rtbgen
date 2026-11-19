use crate::deals::Deals;
use crate::floor::Floor;
use crate::generate::{self, Config};
use crate::ids::RequestId;
use crate::impressions::ImpressionCount;
use crate::profile::Profile;
use crate::request::Request;
use crate::seed::Seed;
use crate::version::SpecVersion;

/// The typed builder of a generated bid request.
///
/// Every knob is a type, not a bare number or string, and every knob widens the
/// profile instead of conflicting with it: `.deals()` adds a `pmp` object to any
/// profile, `.floor()` overrides the floors of its impressions. That is why
/// [`RequestBuilder::build`] returns a [`Request`] and not a `Result`: there is
/// no combination the builder can express that it has to reject.
///
/// The default configuration is [`Seed::new(0)`](Seed::new), `OpenRTB` 2.6 and
/// no knobs. A knob that is not set changes nothing, so a profile's default
/// bytes are stable.
#[derive(Clone, Copy, Debug)]
pub struct RequestBuilder {
    profile: Profile,
    seed: Seed,
    spec: SpecVersion,
    impressions: Option<ImpressionCount>,
    deals: Option<Deals>,
    floor: Option<Floor>,
}

impl RequestBuilder {
    pub(crate) const fn new(profile: Profile) -> Self {
        Self {
            profile,
            seed: Seed::new(0),
            spec: SpecVersion::V26,
            impressions: None,
            deals: None,
            floor: None,
        }
    }

    /// Sets the seed that determines every generated value.
    #[must_use]
    pub const fn seed(mut self, seed: Seed) -> Self {
        self.seed = seed;
        self
    }

    /// Sets the `OpenRTB` revision.
    ///
    /// 2.6 is the default. 2.5 keeps the profile shape and drops only the
    /// fields the 2.6 revision added, so the same profile works under both.
    #[must_use]
    pub const fn spec(mut self, spec: SpecVersion) -> Self {
        self.spec = spec;
        self
    }

    /// Sets the number of impressions, clamped into `1..=3`.
    #[must_use]
    pub const fn impressions(mut self, impressions: ImpressionCount) -> Self {
        self.impressions = Some(impressions);
        self
    }

    /// Adds a private marketplace with the given number of deals.
    ///
    /// The knob applies to every impression and works on every profile.
    #[must_use]
    pub const fn deals(mut self, deals: Deals) -> Self {
        self.deals = Some(deals);
        self
    }

    /// Overrides the bid floor of every impression.
    #[must_use]
    pub const fn floor(mut self, floor: Floor) -> Self {
        self.floor = Some(floor);
        self
    }

    /// Generates the request.
    ///
    /// The same builder configuration always produces byte-identical output for
    /// the same `rtbgen` version.
    #[must_use]
    pub fn build(self) -> Request {
        let config =
            Config::new(RequestId::from_seed(self.seed), self.impressions, self.deals, self.floor);
        let model = generate::build(self.profile, self.seed, &config);

        Request::from_model(self.profile, self.seed, self.spec, &model)
    }
}

#[cfg(test)]
mod tests {
    use super::RequestBuilder;
    use crate::deals::Deals;
    use crate::floor::Floor;
    use crate::impressions::ImpressionCount;
    use crate::profile::Profile;
    use crate::seed::Seed;
    use crate::version::SpecVersion;

    #[test]
    fn the_default_builder_is_seed_zero_and_2_6() {
        let request = RequestBuilder::new(Profile::SiteDisplay).build();

        assert_eq!(request.seed(), Seed::new(0));
        assert_eq!(request.spec(), SpecVersion::V26);
        assert_eq!(request.profile(), Profile::SiteDisplay);
    }

    #[test]
    fn every_knob_is_kept_by_the_builder() {
        let builder = RequestBuilder::new(Profile::AppRewarded)
            .seed(Seed::new(42))
            .spec(SpecVersion::V25)
            .impressions(ImpressionCount::new(2))
            .deals(Deals::new(3))
            .floor(Floor::cpm(1.5, crate::currency::Currency::USD));

        let request = builder.build();

        assert_eq!(request.seed(), Seed::new(42));
        assert_eq!(request.spec(), SpecVersion::V25);
        assert_eq!(request.profile(), Profile::AppRewarded);
    }
}
