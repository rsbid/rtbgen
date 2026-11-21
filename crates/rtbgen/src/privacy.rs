//! Privacy regulation scenarios: `regs`, TC strings and `GPP`.
//!
//! A scenario fixes the whole privacy shape of a request instead of drawing
//! `regs`, `user.consent` and `regs.gpp` independently, so the emitted fields
//! stay consistent: a TC string always travels with `regs.gdpr=1`, a `GPP`
//! string always carries the same TC string in its TCF EU v2 section, and
//! `coppa=1` never carries identifiers.
//!
//! The profiles that draw a request body draw a scenario from the seed; the
//! `Sparse` and `Maximal` profiles use a fixed privacy shape. [`Privacy`] lets
//! the builder override the drawn or fixed scenario on any profile.

use core::fmt;

/// The consent value a TC string carries.
///
/// The string itself is a syntactically valid TCF v2.2 TC string: purpose
/// consents are either all granted or all denied, and vendor 1 follows the same
/// value. It is a synthetic fixture, not a registered CMP's output.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Consent {
    /// Every purpose is consented to.
    Granted,
    /// No purpose is consented to.
    Denied,
}

/// The GPP string whose TCF EU v2 section is `TCF_CONSENT_DENIED`.
///
/// `DBABM` is the canonical GPP header for version 1 with section id 2 (TCF EU
/// v2): type 3, version 1 and section 2, padded to 30 bits. Do not add padding.
pub(crate) const GPP_DENIED: &str = "DBABM~CQdWToAQdWToAAAABAENEsEgAAAAAAAAAAYgAAgAAgAA";
/// The GPP string whose TCF EU v2 section is `TCF_CONSENT_GRANTED`.
pub(crate) const GPP_GRANTED: &str = "DBABM~CQdWToAQdWToAAAABAENEsEsAP_gAAAAAAYgAAoAAgAA";

/// The TCF v2.2 TC string of a denied consent.
///
/// Version 2, a single day-level timestamp in `Created` and `LastUpdated`,
/// `TcfPolicyVersion=4`, `IsServiceSpecific=1`, no vendor and no purpose
/// consented to, and a terminated vendor section. The strings are synthetic
/// fixtures, not a CMP's output: `CmpId` is 0 and the timestamp is fixed, so
/// the bytes of a request never depend on the day it is built.
pub(crate) const TCF_CONSENT_DENIED: &str = "CQdWToAQdWToAAAABAENEsEgAAAAAAAAAAYgAAgAAgAA";
/// The TCF v2.2 TC string of a granted consent: every defined purpose 1..=11
/// and special features 1..=2, vendor 1, the same fixed metadata.
pub(crate) const TCF_CONSENT_GRANTED: &str = "CQdWToAQdWToAAAABAENEsEsAP_gAAAAAAYgAAoAAgAA";

/// The privacy regulation scenario of one request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Privacy {
    /// No `regs` object, no consent and no `GPP`.
    None,
    /// `coppa=1`: no `user`, no `device.ifa`, no `device.ip` and no precise
    /// location, which is how a child-directed request is shaped.
    Coppa,
    /// `regs.coppa=0`, `regs.gdpr=0`: GDPR does not apply. Under `OpenRTB`
    /// 2.5 the writer omits `regs.gdpr`, so only `regs.coppa=0` remains.
    GdprDisabled,
    /// `regs.coppa=0`, `regs.gdpr=1` and a syntactically valid TCF v2.2 TC
    /// string.
    ///
    /// Under `OpenRTB` 2.6 the matching `regs.gpp` and `regs.gpp_sid` are
    /// emitted as well, so the two transports carry the same string. 2.5 has
    /// no `user.consent`, `regs.gdpr`, `regs.gpp` or `regs.gpp_sid`, so the
    /// writer only changes the request location and emits `regs.coppa=0`.
    Gdpr {
        /// The consent value of the emitted TC string.
        consent: Consent,
    },
    /// `regs.coppa=0`, `regs.gdpr=1` and no TC string: the request arrives
    /// without consent, which a bidder must handle. Under `OpenRTB` 2.5 the
    /// writer omits `regs.gdpr`, so only `regs.coppa=0` remains.
    GdprWithoutConsent,
}

impl Privacy {
    /// Every privacy scenario the builder can express.
    ///
    /// The drawn path additionally produces a TC string without `GPP` and a
    /// `GdprDisabled` US request with `regs.ext.us_privacy`; the builder states
    /// cover the scenarios a caller can pin deterministically.
    pub const ALL: [Self; 6] = [
        Self::None,
        Self::Coppa,
        Self::GdprDisabled,
        Self::Gdpr { consent: Consent::Granted },
        Self::Gdpr { consent: Consent::Denied },
        Self::GdprWithoutConsent,
    ];

    /// The stable `snake_case` name accepted by [`Privacy::from_name`].
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Coppa => "coppa",
            Self::GdprDisabled => "gdpr_disabled",
            Self::Gdpr { consent: Consent::Granted } => "gdpr_granted",
            Self::Gdpr { consent: Consent::Denied } => "gdpr_denied",
            Self::GdprWithoutConsent => "gdpr_no_consent",
        }
    }

    /// Parses the exact `snake_case` name of a privacy scenario.
    ///
    /// The match is case-sensitive; there are no aliases. The names come from
    /// [`Privacy::ALL`], so a new variant is parsed as soon as it is listed.
    ///
    /// # Errors
    ///
    /// Returns [`UnknownPrivacy`] when `name` is not one of [`Privacy::ALL`]'s
    /// names.
    pub fn from_name(name: &str) -> Result<Self, UnknownPrivacy> {
        Self::ALL.into_iter().find(|privacy| privacy.name() == name).ok_or(UnknownPrivacy)
    }
}

impl fmt::Display for Privacy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// The error returned by [`Privacy::from_name`] for an unknown scenario.
#[derive(Debug, Clone, Copy)]
pub struct UnknownPrivacy;

impl fmt::Display for UnknownPrivacy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("unknown privacy scenario; expected one of:")?;

        let mut names = Privacy::ALL.iter();
        if let Some(first) = names.next() {
            write!(f, " {first}")?;
        }
        for privacy in names {
            write!(f, ", {privacy}")?;
        }

        Ok(())
    }
}

impl core::error::Error for UnknownPrivacy {}

#[cfg(test)]
mod tests {
    #![expect(clippy::expect_used, reason = "in tests a panic = a failed test")]

    extern crate std;

    use lib_tcstring::TcModelV2;
    use std::vec::Vec;

    use super::{
        GPP_DENIED, GPP_GRANTED, Privacy, TCF_CONSENT_DENIED, TCF_CONSENT_GRANTED, UnknownPrivacy,
    };

    fn decode(string: &str) -> TcModelV2 {
        TcModelV2::try_from(string).expect("a valid TC string")
    }

    fn check_metadata(model: &TcModelV2) {
        assert_eq!(model.created_at, model.updated_at, "Created must equal LastUpdated");
        assert_eq!(model.tcf_policy_version, 4, "TCF v2.2 uses policy version 4");
        assert!(model.is_service_specific, "IsServiceSpecific must be 1");
        assert!(!model.use_non_standard_stacks);
        assert_eq!(model.consent_language, "EN");
        assert_eq!(model.publisher_country_code, "DE");
        assert_eq!(
            model.purposes_li_transparency,
            Vec::<u8>::new(),
            "no purpose may be set on legitimate interest"
        );
        assert_eq!(model.publisher_restrictions, Vec::<lib_tcstring::PublisherRestriction>::new());
        assert_eq!(model.vendors_li_consent, Vec::<u16>::new());
    }

    fn check_denied(model: &TcModelV2) {
        assert_eq!(model.special_feature_opt_ins, Vec::<u8>::new());
        assert_eq!(model.purposes_consent, Vec::<u8>::new());
        assert_eq!(model.vendors_consent, Vec::<u16>::new());
    }

    fn check_granted(model: &TcModelV2) {
        assert_eq!(model.special_feature_opt_ins, [1, 2]);
        assert_eq!(model.purposes_consent, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
        assert_eq!(model.vendors_consent, [1]);
    }

    #[test]
    fn the_denied_tc_string_decodes_as_tcf_v2_2() {
        let model = decode(TCF_CONSENT_DENIED);

        check_metadata(&model);
        check_denied(&model);
    }

    #[test]
    fn the_granted_tc_string_decodes_as_tcf_v2_2() {
        let model = decode(TCF_CONSENT_GRANTED);

        check_metadata(&model);
        check_granted(&model);
    }

    #[test]
    fn the_gpp_strings_carry_the_same_tc_string() {
        assert_eq!(
            GPP_DENIED.split_once('~').map(|(_, section)| section),
            Some(TCF_CONSENT_DENIED)
        );
        assert_eq!(
            GPP_GRANTED.split_once('~').map(|(_, section)| section),
            Some(TCF_CONSENT_GRANTED)
        );
    }

    #[test]
    fn the_gpp_header_is_the_canonical_tcf_eu_v2_header() {
        // The IAB GPP header for version 1 with section id 2 is `DBABM`.
        // An extra zero group (`DBABMA`) is not the canonical encoding.
        for string in [GPP_DENIED, GPP_GRANTED] {
            assert_eq!(
                string.split_once('~').map(|(header, _)| header),
                Some("DBABM"),
                "{string} has the wrong GPP header"
            );
        }
    }

    #[test]
    fn every_privacy_round_trips_through_its_name() {
        for privacy in Privacy::ALL {
            assert!(matches!(
                Privacy::from_name(privacy.name()),
                Ok(found) if found == privacy
            ));
        }
    }

    #[test]
    fn privacy_names_are_unique_snake_case() {
        let mut names = Vec::new();
        for privacy in Privacy::ALL {
            let name = privacy.name();
            assert!(
                name.bytes().all(|byte| byte.is_ascii_lowercase() || byte == b'_'),
                "{name} is not snake_case"
            );
            assert!(names.iter().all(|seen| *seen != name), "{name} is a duplicate");
            names.push(name);
        }
    }

    #[test]
    fn unknown_privacy_names_are_rejected() {
        assert!(matches!(Privacy::from_name("Coppa"), Err(UnknownPrivacy)));
        assert!(matches!(Privacy::from_name("gdpr-granted"), Err(UnknownPrivacy)));
        assert!(matches!(Privacy::from_name(""), Err(UnknownPrivacy)));
    }

    #[test]
    fn unknown_privacy_lists_every_name() {
        let error = UnknownPrivacy;
        let message = std::format!("{error}");

        for privacy in Privacy::ALL {
            assert!(message.contains(privacy.name()), "{message} does not list {}", privacy.name());
        }
    }

    #[test]
    fn the_tc_strings_use_the_url_safe_base64_alphabet_without_padding() {
        for string in [TCF_CONSENT_DENIED, TCF_CONSENT_GRANTED] {
            assert!(
                string
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_'),
                "{string} is not URL-safe base64"
            );
            assert_eq!(string.len(), 44, "{string} is not byte-aligned");
        }
    }
}
