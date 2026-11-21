use crate::deals::Deals;
use crate::degrade::DegradeRate;
use crate::deprecated::DeprecatedFields;
use crate::floor::Floor;
use crate::generate::{self, Config};
use crate::ids::RequestId;
use crate::impressions::ImpressionCount;
use crate::privacy::Privacy;
use crate::profile::Profile;
use crate::request::Request;
use crate::seed::Seed;
use crate::version::SpecVersion;

/// The typed builder of a generated bid request.
///
/// Every knob is a type, not a bare number or string. A knob either widens the
/// profile (`.deals()` adds a `pmp` object to any profile) or deliberately
/// overrides its values (`.floor()` replaces the floors, `.privacy()` replaces
/// the privacy scenario, `.degrade()` adds one named defect); it never
/// contradicts the profile. That is why [`RequestBuilder::build`] returns a
/// [`Request`] and not a `Result`: there is no combination the builder can
/// express that it has to reject.
///
/// The default configuration is [`Seed::new(0)`](Seed::new), `OpenRTB` 2.6 and
/// no knobs. A knob that is not set changes nothing, so a profile's default
/// bytes are stable. Fields the target revision deprecated are omitted by
/// default and replaced by the revision's own field; see
/// [`RequestBuilder::deprecated_fields`].
#[derive(Clone, Copy, Debug)]
pub struct RequestBuilder {
    profile: Profile,
    seed: Seed,
    spec: SpecVersion,
    impressions: Option<ImpressionCount>,
    deals: Option<Deals>,
    floor: Option<Floor>,
    privacy: Option<Privacy>,
    degrade: DegradeRate,
    deprecated: DeprecatedFields,
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
            privacy: None,
            degrade: DegradeRate::OFF,
            deprecated: DeprecatedFields::Omit,
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

    /// Overrides the profile's default number of impressions, clamped into `1..=3`.
    ///
    /// This is an explicit override, not a hint: `MultiImp` picks `2..=3` by
    /// itself, and this method replaces that distribution with exactly the
    /// given count. Profiles that have no such default get the count as well.
    #[must_use]
    pub const fn impressions_exact(mut self, impressions: ImpressionCount) -> Self {
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

    /// Overrides the privacy scenario of every request.
    ///
    /// The scenario fixes `regs`, `user.consent`, `regs.gpp`, the location of
    /// the device and whether identifiers may appear; see [`Privacy`]. The
    /// override replaces the scenario the seed would have drawn, so it is the
    /// way to ask for a specific regulation state.
    #[must_use]
    pub const fn privacy(mut self, privacy: Privacy) -> Self {
        self.privacy = Some(privacy);
        self
    }

    /// Sets whether the generator emits fields the target revision deprecated.
    ///
    /// The default, [`DeprecatedFields::Omit`], writes the revision's
    /// replacement instead: `OpenRTB` 2.6 video impressions carry
    /// `imp.video.plcmt`, not the deprecated `imp.video.placement`. With
    /// [`DeprecatedFields::Include`] the deprecated field is emitted next to
    /// its replacement, which is the migration pattern for parsers that still
    /// read it. `OpenRTB` 2.5 has no deprecated field in the model, so the
    /// policy does not change its bytes.
    #[must_use]
    pub const fn deprecated_fields(mut self, deprecated: DeprecatedFields) -> Self {
        self.deprecated = deprecated;
        self
    }

    /// Makes a share of requests carry one deliberate `OpenRTB` violation.
    ///
    /// The mutation is drawn after the profile body, so a request without a
    /// mutation is byte-identical to the same request without the knob.
    /// [`DegradeRate::percent`] with `0` is off; with `100` the request always
    /// carries exactly one violation.
    #[must_use]
    pub const fn degrade(mut self, degrade: DegradeRate) -> Self {
        self.degrade = degrade;
        self
    }

    /// Generates the request.
    ///
    /// The same builder configuration always produces byte-identical output for
    /// the same `rtbgen` version.
    #[must_use]
    pub fn build(self) -> Request {
        let mut config =
            Config::new(RequestId::from_seed(self.seed), self.impressions, self.deals, self.floor)
                .degrade(self.degrade)
                .spec(self.spec)
                .deprecated(self.deprecated);

        if let Some(privacy) = self.privacy {
            config = config.privacy_override(privacy);
        }

        let generated = generate::build(self.profile, self.seed, &config);

        Request::from_generated(self.profile, Some(self.seed), self.spec, &generated)
    }
}

#[cfg(test)]
mod tests {
    use super::RequestBuilder;
    use crate::deals::Deals;
    use crate::deprecated::DeprecatedFields;
    use crate::floor::Floor;
    use crate::impressions::ImpressionCount;
    use crate::privacy::Privacy;
    use crate::profile::Profile;
    use crate::seed::Seed;
    use crate::version::SpecVersion;

    #[test]
    fn the_default_builder_is_seed_zero_and_2_6() {
        let request = RequestBuilder::new(Profile::SiteDisplay).build();

        assert_eq!(request.seed(), Some(Seed::new(0)));
        assert_eq!(request.spec(), SpecVersion::V26);
        assert_eq!(request.profile(), Profile::SiteDisplay);
    }

    #[test]
    fn every_knob_is_kept_by_the_builder() {
        let builder = RequestBuilder::new(Profile::AppRewarded)
            .seed(Seed::new(42))
            .spec(SpecVersion::V25)
            .impressions_exact(ImpressionCount::new(2))
            .deals(Deals::new(3))
            .floor(Floor::cpm(1.5, crate::currency::Currency::USD))
            .privacy(Privacy::GdprWithoutConsent)
            .deprecated_fields(DeprecatedFields::Include);

        let request = builder.build();

        assert_eq!(request.seed(), Some(Seed::new(42)));
        assert_eq!(request.spec(), SpecVersion::V25);
        assert_eq!(request.profile(), Profile::AppRewarded);
    }

    #[test]
    fn the_privacy_knob_reaches_the_request() {
        let request = RequestBuilder::new(Profile::SiteDisplay).privacy(Privacy::Coppa).build();
        let json = request.to_json_string();

        assert!(json.contains("\"coppa\":1"), "the COPPA flag was not emitted");
        assert!(!json.contains("\"user\""), "a child-directed request kept the user");
    }

    #[test]
    fn the_deprecated_fields_knob_controls_video_placement() {
        let default =
            RequestBuilder::new(Profile::SiteVideo).seed(Seed::new(7)).build().to_json_string();

        assert!(default.contains("\"plcmt\":"), "2.6 lost the replacement field");
        assert!(!default.contains("\"placement\":"), "2.6 kept the deprecated field");

        let included = RequestBuilder::new(Profile::SiteVideo)
            .seed(Seed::new(7))
            .deprecated_fields(DeprecatedFields::Include)
            .build()
            .to_json_string();

        assert!(included.contains("\"plcmt\":"), "the replacement was dropped");
        assert!(included.contains("\"placement\":"), "the deprecated field was not included");

        let v25 = RequestBuilder::new(Profile::SiteVideo)
            .seed(Seed::new(7))
            .spec(SpecVersion::V25)
            .deprecated_fields(DeprecatedFields::Include)
            .build()
            .to_json_string();

        assert!(v25.contains("\"placement\":"), "2.5 lost its placement field");
        assert!(!v25.contains("\"plcmt\":"), "a 2.6 field leaked into 2.5");
    }
}
