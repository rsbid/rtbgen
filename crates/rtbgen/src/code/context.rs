//! Context codes: privacy, supply chain and server-side ad insertion.

use core::fmt;

/// How server-side ad insertion affects asset and tracker retrieval:
/// `Imp.ssai`.
///
/// Unlike a flag this code has four values: `0` unknown, `1` client-side,
/// `2` assets stitched with client-side pixels, `3` all server-side.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SsaiMode(u16);

impl SsaiMode {
    /// All assets and tracking are server-side; used by the maximal fixture.
    pub(crate) const SERVER_SIDE: Self = Self(3);
}

impl fmt::Display for SsaiMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The entity responsible for the final impression sale decision: `Source.fd`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct FinalDecision(u16);

impl FinalDecision {
    /// The upstream source decides; used by the maximal fixture.
    pub(crate) const UPSTREAM: Self = Self(1);
}

impl fmt::Display for FinalDecision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// One `GPP` section id: an element of `Regs.gpp_sid`.
///
/// Section ids are assigned by the `GPP` specification and grow over time, so
/// the type keeps the numeric id and names only the sections this crate emits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GppSectionId(u16);

impl GppSectionId {
    /// The `TCF` EU v2 section (`GPP` section 2).
    pub(crate) const TCF_EU_V2: Self = Self(2);
}

impl fmt::Display for GppSectionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
