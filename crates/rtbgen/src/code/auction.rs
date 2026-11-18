//! Auction types, as defined by `OpenRTB` 2.6.

use core::fmt;

/// The auction type of a bid request: `BidRequest.at`.
///
/// `OpenRTB` 2.6 defines `1` (first price) and `2` (second price plus, the
/// default), and allows exchange-specific values of `500` and greater. `3` is
/// a deal-only code and lives in [`DealAuctionType`]. This crate names only the
/// two `OpenRTB` codes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct AuctionType(u16);

impl AuctionType {
    /// `1`: first-price auction.
    pub(crate) const FIRST_PRICE: Self = Self(1);

    /// `2`: second-price-plus auction, the wire default.
    pub(crate) const SECOND_PRICE_PLUS: Self = Self(2);
}

impl fmt::Display for AuctionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The auction type override of one deal: `Deal.at`.
///
/// `OpenRTB` 2.6 defines `1` (first price), `2` (second price plus) and `3`
/// (the value passed in `bidfloor` is the agreed deal price), and lets the
/// exchange define further auction types. This crate names only the three
/// `OpenRTB` codes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct DealAuctionType(u16);

impl DealAuctionType {
    /// `1`: first-price auction.
    pub(crate) const FIRST_PRICE: Self = Self(1);

    /// `2`: second-price-plus auction.
    pub(crate) const SECOND_PRICE_PLUS: Self = Self(2);

    /// `3`: the floor is the agreed deal price; used by the maximal fixture.
    pub(crate) const AGREED_DEAL_PRICE: Self = Self(3);
}

impl fmt::Display for DealAuctionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
