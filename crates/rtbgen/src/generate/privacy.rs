//! The privacy scenario the generator draws before the request body.
//!
//! One scenario owns every privacy field of a request: `regs`, `user.consent`,
//! `regs.gpp`, the country of `device.geo` and whether identifiers may appear.
//! Drawing them independently is what made `gdpr=1` arrive with a US location
//! and a TC string that no TCF decoder accepted.

use crate::flag::Flag;
use crate::privacy::{
    Consent, GPP_DENIED, GPP_GRANTED, Privacy, TCF_CONSENT_DENIED, TCF_CONSENT_GRANTED,
};
use crate::rng::{Entropy, Share, Span};

use super::data::{COPPA_GEOS, EU_GEOS, GeoProfile, NON_EU_GEOS};

/// A resolved privacy scenario.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Scenario {
    /// No `regs`, no consent and no `GPP`.
    None,
    /// `coppa=1`, no identifiers and no precise location.
    Coppa,
    /// `coppa=0`, `gdpr=0`.
    GdprDisabled,
    /// `coppa=0`, `gdpr=1` and a TC string.
    Gdpr {
        /// The consent value of the emitted TC string.
        consent: Consent,
        /// Whether the matching `GPP` string is emitted too.
        gpp: bool,
    },
    /// `coppa=0`, `gdpr=1` and no TC string.
    GdprNoConsent,
}

impl Scenario {
    /// Draws the scenario from the seed.
    ///
    /// Every request gets exactly one scenario: 35% without `regs`, 5%
    /// child-directed, 15% outside the GDPR and 45% under the GDPR (25%
    /// granted, 10% denied, 10% without consent). A TC string that is emitted
    /// also appears in `GPP` 40% of the time.
    pub(super) fn draw(rng: &mut impl Entropy) -> Self {
        let roll = rng.u32(Span::from_zero(99));

        if roll < 25 {
            let gpp = rng.bit(Share::percent(40));
            Self::Gdpr { consent: Consent::Granted, gpp }
        } else if roll < 35 {
            let gpp = rng.bit(Share::percent(40));
            Self::Gdpr { consent: Consent::Denied, gpp }
        } else if roll < 45 {
            Self::GdprNoConsent
        } else if roll < 60 {
            Self::GdprDisabled
        } else if roll < 65 {
            Self::Coppa
        } else {
            Self::None
        }
    }

    /// Resolves a builder override into the same shape as a drawn scenario.
    pub(super) const fn from_privacy(privacy: Privacy) -> Self {
        match privacy {
            Privacy::None => Self::None,
            Privacy::Coppa => Self::Coppa,
            Privacy::GdprDisabled => Self::GdprDisabled,
            Privacy::Gdpr { consent } => Self::Gdpr { consent, gpp: true },
            Privacy::GdprWithoutConsent => Self::GdprNoConsent,
        }
    }

    /// Returns the locations this scenario may claim.
    pub(super) const fn geos(self) -> &'static [GeoProfile] {
        match self {
            Self::None | Self::GdprDisabled => NON_EU_GEOS,
            Self::Coppa => COPPA_GEOS,
            Self::Gdpr { .. } | Self::GdprNoConsent => EU_GEOS,
        }
    }

    /// Returns whether the request carries a `regs` object at all.
    pub(super) const fn has_regs(self) -> bool {
        !matches!(self, Self::None)
    }

    /// Returns the `regs.gdpr` value, when the revision has the field.
    pub(super) const fn gdpr(self) -> Option<Flag> {
        match self {
            Self::None | Self::Coppa => None,
            Self::GdprDisabled => Some(Flag::No),
            Self::Gdpr { .. } | Self::GdprNoConsent => Some(Flag::Yes),
        }
    }

    /// Returns the `regs.coppa` value for a request that carries `regs`.
    pub(super) const fn coppa(self) -> Flag {
        if matches!(self, Self::Coppa) { Flag::Yes } else { Flag::No }
    }

    /// Returns the TC string of `user.consent`, when the scenario has one.
    pub(super) const fn consent(self) -> Option<&'static str> {
        match self {
            Self::Gdpr { consent: Consent::Granted, .. } => Some(TCF_CONSENT_GRANTED),
            Self::Gdpr { consent: Consent::Denied, .. } => Some(TCF_CONSENT_DENIED),
            Self::None | Self::Coppa | Self::GdprDisabled | Self::GdprNoConsent => None,
        }
    }

    /// Returns the `regs.gpp` string, when the scenario emits one.
    pub(super) const fn gpp(self) -> Option<&'static str> {
        match self {
            Self::Gdpr { consent: Consent::Granted, gpp: true } => Some(GPP_GRANTED),
            Self::Gdpr { consent: Consent::Denied, gpp: true } => Some(GPP_DENIED),
            Self::None
            | Self::Coppa
            | Self::GdprDisabled
            | Self::GdprNoConsent
            | Self::Gdpr { gpp: false, .. } => None,
        }
    }

    /// Returns whether the request may carry storage identifiers.
    ///
    /// Under GDPR an identifier requires a positive consent signal: a denied
    /// TC string and a request without consent both forbid `device.ifa`,
    /// `user.id` and `user.buyeruid`.
    pub(super) const fn allows_identifiers(self) -> bool {
        !matches!(
            self,
            Self::Coppa | Self::Gdpr { consent: Consent::Denied, .. } | Self::GdprNoConsent
        )
    }

    /// Returns whether the device may carry an address.
    pub(super) const fn allows_ip(self) -> bool {
        !matches!(self, Self::Coppa)
    }

    /// Returns whether `device.geo` may carry region, city, zip and its type.
    pub(super) const fn precise_geo(self) -> bool {
        !matches!(self, Self::Coppa)
    }

    /// Returns whether a non-GDPR US request carries the legacy US Privacy
    /// string in `regs.ext`.
    pub(super) fn has_us_privacy(self, country: &str) -> bool {
        matches!(self, Self::GdprDisabled) && country == "USA"
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::Scenario;
    use crate::flag::Flag;
    use crate::privacy::{Consent, Privacy};
    use crate::rng::RngEntropy;
    use crate::seed::Seed;

    #[test]
    fn every_scenario_is_reachable() {
        let expected = [
            Scenario::Gdpr { consent: Consent::Granted, gpp: false },
            Scenario::Gdpr { consent: Consent::Granted, gpp: true },
            Scenario::Gdpr { consent: Consent::Denied, gpp: false },
            Scenario::Gdpr { consent: Consent::Denied, gpp: true },
            Scenario::GdprNoConsent,
            Scenario::GdprDisabled,
            Scenario::Coppa,
            Scenario::None,
        ];
        let mut seen = std::vec::Vec::new();

        for seed in 0..200 {
            let mut rng = RngEntropy::from_seed(Seed::new(seed));
            let scenario = Scenario::draw(&mut rng);
            if !seen.contains(&scenario) {
                seen.push(scenario);
            }
        }

        for scenario in expected {
            assert!(seen.contains(&scenario), "{scenario:?} was never drawn");
        }
    }

    #[test]
    fn a_coppa_scenario_suppresses_identifiers() {
        let scenario = Scenario::from_privacy(Privacy::Coppa);

        assert!(!scenario.allows_identifiers());
        assert!(!scenario.allows_ip());
        assert!(!scenario.precise_geo());
        assert_eq!(scenario.coppa(), Flag::Yes);
        assert_eq!(scenario.gdpr(), None);
        assert_eq!(scenario.consent(), None);
        assert_eq!(scenario.gpp(), None);
    }

    #[test]
    fn a_gdpr_override_carries_the_matching_gpp() {
        for (consent, tc, gpp) in [
            (Consent::Granted, super::TCF_CONSENT_GRANTED, super::GPP_GRANTED),
            (Consent::Denied, super::TCF_CONSENT_DENIED, super::GPP_DENIED),
        ] {
            let scenario = Scenario::from_privacy(Privacy::Gdpr { consent });

            assert_eq!(scenario.gdpr(), Some(Flag::Yes));
            assert_eq!(scenario.coppa(), Flag::No);
            assert_eq!(scenario.consent(), Some(tc));
            assert_eq!(scenario.gpp(), Some(gpp));
        }
    }

    #[test]
    fn the_us_privacy_string_belongs_to_a_us_non_gdpr_request() {
        let scenario = Scenario::from_privacy(Privacy::GdprDisabled);

        assert!(scenario.has_us_privacy("USA"));
        assert!(!scenario.has_us_privacy("RUS"));
        assert!(!scenario.has_us_privacy("DEU"));
        assert!(!Scenario::from_privacy(Privacy::Coppa).has_us_privacy("USA"));
    }
}
