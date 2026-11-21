//! Domain codes of the wire model.
//!
//! Every code is a newtype over `u16` with a private field, so a code cannot be
//! assigned to a field of another domain and cannot be narrowed to `u8`. The
//! type owns its presentation: the writer formats it with `fmt::Display` and
//! never receives a bare number.
//!
//! A named constant exists only where a profile or the maximal fixture
//! constructs the value. The rest of each spec list lands with the profile that
//! needs it; an unused constant would be dead code.

mod auction;
mod context;
mod media;
mod target;

pub(crate) use auction::{AuctionType, DealAuctionType};
pub(crate) use context::{FinalDecision, GppSectionId, SsaiMode};
pub(crate) use media::{
    ApiFramework, BlockedBannerType, BlockedCreativeAttribute, CompanionType, DeliveryMethod,
    ExpandableDirection, FeedType, Linearity, Placement, PlacementPosition, Plcmt, Protocol,
    VcmMode, VolumeNormalization,
};
pub(crate) use target::{DeviceType, LocationType};

#[cfg(test)]
mod tests;
