//! Privacy regulation scenarios: `regs`, TC strings and `GPP`.
//!
//! A scenario fixes the whole privacy shape of a request instead of drawing
//! `regs`, `user.consent` and `regs.gpp` independently, so the emitted fields
//! stay consistent: a TC string always travels with `regs.gdpr=1`, a `GPP`
//! string always carries the same TC string in its TCF EU v2 section, and
//! `coppa=1` never carries identifiers.
//!
//! The generator draws a scenario from the seed. [`Privacy`] lets the builder
//! override it, which is the only way to ask for a specific regulation state.

/// The consent value a TC string carries.
///
/// The string itself is a valid TCF v2.2 TC string: purpose consents are either
/// all granted or all denied, and vendor 1 follows the same value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Consent {
    /// Every purpose is consented to.
    Granted,
    /// No purpose is consented to.
    Denied,
}

/// The GPP string whose TCF EU v2 section is `TCF_CONSENT_DENIED`.
pub(crate) const GPP_DENIED: &str = "DBABMA~CQdWToAQdWToAAAABAENEsEgAAAAAAAAAAYgAAgAAgAA";
/// The GPP string whose TCF EU v2 section is `TCF_CONSENT_GRANTED`.
pub(crate) const GPP_GRANTED: &str = "DBABMA~CQdWToAQdWToAAAABAENEsEsAP_gAAAAAAYgAAoAAgAA";

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
    /// `regs.coppa=0`, `regs.gdpr=0`: GDPR does not apply.
    GdprDisabled,
    /// `regs.coppa=0`, `regs.gdpr=1` and a TCF v2.2 TC string.
    ///
    /// When the builder sets this variant, the matching `regs.gpp` and
    /// `regs.gpp_sid` are emitted as well, so the two transports carry the
    /// same string.
    Gdpr {
        /// The consent value of the emitted TC string.
        consent: Consent,
    },
    /// `regs.coppa=0`, `regs.gdpr=1` and no TC string: the request arrives
    /// without consent, which a bidder must handle.
    GdprWithoutConsent,
}

#[cfg(test)]
mod tests {
    #![expect(clippy::expect_used, reason = "in tests a panic = a failed test")]

    extern crate std;

    use lib_tcstring::TcModelV2;
    use std::vec::Vec;

    use super::{GPP_DENIED, GPP_GRANTED, TCF_CONSENT_DENIED, TCF_CONSENT_GRANTED};

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
