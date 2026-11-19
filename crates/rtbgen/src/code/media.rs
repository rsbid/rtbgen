//! Codes of the media objects: banners, video and audio.

use core::fmt;

/// A blocked banner ad type: an element of `Banner.btype`.
///
/// `btype` is a blocklist: its values name the ad types the exchange refuses,
/// not the type of the banner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BlockedBannerType(u16);

impl BlockedBannerType {
    /// The `iframe` banner ad type.
    ///
    /// Emitting it in `btype` blocks `iframe` banner ads.
    pub(crate) const IFRAME: Self = Self(4);
}

impl fmt::Display for BlockedBannerType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A blocked creative attribute: an element of `battr`.
///
/// Values `1..=23` come from `AdCOM`'s creative-attributes list;
/// vendor-specific codes are `500` and greater.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CreativeAttribute(u16);

impl CreativeAttribute {
    /// A blocked audio ad that starts playing automatically.
    pub(crate) const AUDIO_AUTOPLAY: Self = Self(1);

    /// A blocked audio ad that starts only after a user action.
    pub(crate) const AUDIO_USER_INITIATED: Self = Self(2);
}

impl fmt::Display for CreativeAttribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A direction a banner may expand in: an element of `Banner.expdir`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ExpandableDirection(u16);

impl ExpandableDirection {
    /// The banner expands to the left.
    pub(crate) const LEFT: Self = Self(1);

    /// The banner expands upwards.
    pub(crate) const UP: Self = Self(3);
}

impl fmt::Display for ExpandableDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A supported API framework: an element of `Banner.api`, `Video.api`,
/// `Audio.api` or `Native.api`.
///
/// Values `1..=9` come from `AdCOM`'s API-frameworks list; vendor-specific
/// codes are `500` and greater.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ApiFramework(u16);

impl ApiFramework {
    /// `MRAID` 1.0.
    pub(crate) const MRAID_1: Self = Self(3);

    /// `MRAID` 2.0.
    pub(crate) const MRAID_2: Self = Self(5);
}

impl fmt::Display for ApiFramework {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// An ad position on screen: `Banner.pos` and `Video.pos`.
///
/// Values `0..=17` come from `AdCOM`'s placement-positions list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PlacementPosition(u16);

impl PlacementPosition {
    /// Above the fold.
    pub(crate) const ABOVE_THE_FOLD: Self = Self(1);

    /// A locked, fixed position.
    pub(crate) const LOCKED: Self = Self(2);

    /// Below the fold.
    pub(crate) const BELOW_THE_FOLD: Self = Self(3);
}

impl fmt::Display for PlacementPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A video or audio protocol: an element of `Video.protocols` and
/// `Audio.protocols`.
///
/// Values `1..=16` come from `AdCOM`'s audio/video creative-subtypes list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Protocol(u16);

impl Protocol {
    /// `VAST` 2.0.
    pub(crate) const VAST_2_0: Self = Self(2);

    /// `VAST` 3.0.
    pub(crate) const VAST_3_0: Self = Self(3);

    /// `VAST` 4.0.
    pub(crate) const VAST_4_0: Self = Self(7);
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The placement of a video or audio impression: `Video.placement` and
/// `Audio.placement`.
///
/// The field is deprecated in `OpenRTB` 2.6 in favour of `plcmt`, but the model
/// keeps it because 2.5 still defines it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Placement(u16);

impl Placement {
    /// In-stream: pre-roll, mid-roll or post-roll.
    pub(crate) const IN_STREAM: Self = Self(1);

    /// In-article.
    pub(crate) const IN_ARTICLE: Self = Self(3);

    /// In-feed.
    pub(crate) const IN_FEED: Self = Self(4);

    /// Interstitial.
    pub(crate) const INTERSTITIAL: Self = Self(5);
}

impl fmt::Display for Placement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The required linearity of a video impression: `Video.linearity`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Linearity(u16);

impl Linearity {
    /// Only linear creatives are allowed; used by the maximal fixture.
    pub(crate) const LINEAR: Self = Self(1);
}

impl fmt::Display for Linearity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The companion banner rendering mode: `Banner.vcm`.
///
/// `0` renders the banner concurrently with the video, `1` as an end-card.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct VcmMode(u16);

impl VcmMode {
    /// The banner renders as an end-card; used by the maximal fixture.
    pub(crate) const END_CARD: Self = Self(1);
}

impl fmt::Display for VcmMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The type of an audio feed: `Audio.feed`.
///
/// Values `1..=7` come from `AdCOM`'s feed-types list; vendor-specific codes
/// are `500` and greater.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct FeedType(u16);

impl FeedType {
    /// A music streaming service; used by the maximal fixture.
    pub(crate) const MUSIC_STREAM: Self = Self(1);
}

impl fmt::Display for FeedType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// An audio volume normalization mode: `Audio.nvol`.
///
/// Values `0..=4` come from `AdCOM`'s volume-normalization list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct VolumeNormalization(u16);

impl VolumeNormalization {
    /// Normalized to the average content volume; used by the maximal fixture.
    pub(crate) const AVERAGE: Self = Self(1);
}

impl fmt::Display for VolumeNormalization {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A delivery method: an element of `Video.delivery` and `Audio.delivery`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct DeliveryMethod(u16);

impl DeliveryMethod {
    /// Progressive download.
    pub(crate) const PROGRESSIVE: Self = Self(2);
}

impl fmt::Display for DeliveryMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A companion ad type: an element of `Video.companiontype`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CompanionType(u16);

impl CompanionType {
    /// A static image resource; used by the maximal fixture.
    pub(crate) const STATIC: Self = Self(1);
}

impl fmt::Display for CompanionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
