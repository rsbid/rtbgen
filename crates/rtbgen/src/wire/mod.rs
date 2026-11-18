//! The crate-private wire model.
//!
//! The model exists to be written, not validated: fields may be absent, empty
//! or mutually exclusive. The typed builder in front of it will keep ordinary
//! callers out of that state.
//!
//! The model is allocation-free so that [`crate::Request::write_json`] builds
//! with `core` alone. Variable-length lists have a fixed inline capacity equal
//! to the largest request this revision can generate.

macro_rules! setters {
    ($type:ty { $($method:ident => $field:ident : $ty:ty),* $(,)? }) => {
        impl $type {
            $(
                pub(crate) const fn $method(&mut self, value: $ty) {
                    self.$field = value;
                }
            )*
        }
    };
}

mod context;
mod imp;
mod media;
mod party;
mod request;

pub(crate) use context::{Deal, DealList, Pmp, Regs, Source};
pub(crate) use imp::{Imp, ImpList};
pub(crate) use media::{Banner, BannerList, Native, Video};
pub(crate) use party::{App, Device, Geo, Publisher, Site, User};
pub(crate) use request::RequestModel;

/// The exchange's impression cap, shared by `MultiImp` and `degrade`.
pub(crate) const MAX_IMPRESSIONS: usize = 3;

/// The largest number of deals a single `pmp` object can carry.
pub(crate) const MAX_DEALS: usize = 8;

/// The largest number of supply-chain nodes in this revision.
pub(crate) const MAX_NODES: usize = 4;

/// The largest number of companion banners on a video impression.
pub(crate) const MAX_COMPANIONS: usize = 2;

#[cfg(test)]
mod tests;
