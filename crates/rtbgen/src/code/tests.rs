//! Tests for the domain codes.

extern crate std;

use std::string::ToString;

use super::{
    ApiFramework, AuctionType, BlockedBannerType, CompanionType, CreativeAttribute,
    DealAuctionType, DeliveryMethod, DeviceType, ExpandableDirection, FeedType, FinalDecision,
    GppSectionId, Linearity, LocationType, Placement, PlacementPosition, Protocol, SsaiMode,
    VcmMode, VolumeNormalization,
};

#[test]
fn request_auction_codes_are_the_two_openrtb_codes() {
    assert_eq!(AuctionType::FIRST_PRICE.to_string(), "1");
    assert_eq!(AuctionType::SECOND_PRICE_PLUS.to_string(), "2");
}

#[test]
fn deal_auction_codes_are_the_three_openrtb_codes() {
    assert_eq!(DealAuctionType::FIRST_PRICE.to_string(), "1");
    assert_eq!(DealAuctionType::SECOND_PRICE_PLUS.to_string(), "2");
    assert_eq!(DealAuctionType::AGREED_DEAL_PRICE.to_string(), "3");
}

#[test]
fn banner_codes_display_their_wire_value() {
    assert_eq!(BlockedBannerType::IFRAME.to_string(), "4");
    assert_eq!(CreativeAttribute::AUDIO_AUTOPLAY.to_string(), "1");
    assert_eq!(CreativeAttribute::AUDIO_USER_INITIATED.to_string(), "2");
    assert_eq!(ExpandableDirection::LEFT.to_string(), "1");
    assert_eq!(ExpandableDirection::UP.to_string(), "3");
    assert_eq!(ApiFramework::MRAID_1.to_string(), "3");
    assert_eq!(ApiFramework::MRAID_2.to_string(), "5");
    assert_eq!(PlacementPosition::ABOVE_THE_FOLD.to_string(), "1");
    assert_eq!(PlacementPosition::LOCKED.to_string(), "2");
    assert_eq!(PlacementPosition::BELOW_THE_FOLD.to_string(), "3");
    assert_eq!(VcmMode::END_CARD.to_string(), "1");
}

#[test]
fn video_codes_display_their_wire_value() {
    assert_eq!(Protocol::VAST_2_0.to_string(), "2");
    assert_eq!(Protocol::VAST_3_0.to_string(), "3");
    assert_eq!(Placement::IN_STREAM.to_string(), "1");
    assert_eq!(Linearity::LINEAR.to_string(), "1");
    assert_eq!(DeliveryMethod::PROGRESSIVE.to_string(), "2");
    assert_eq!(CompanionType::STATIC.to_string(), "1");
}

#[test]
fn audio_codes_display_their_wire_value() {
    assert_eq!(FeedType::MUSIC_STREAM.to_string(), "1");
    assert_eq!(VolumeNormalization::AVERAGE.to_string(), "1");
}

#[test]
fn context_and_target_codes_display_their_wire_value() {
    assert_eq!(SsaiMode::SERVER_SIDE.to_string(), "3");
    assert_eq!(FinalDecision::UPSTREAM.to_string(), "1");
    assert_eq!(GppSectionId::TCF_EU_V2.to_string(), "2");
    assert_eq!(DeviceType::PERSONAL_COMPUTER.to_string(), "2");
    assert_eq!(DeviceType::PHONE.to_string(), "4");
    assert_eq!(LocationType::IP_ADDRESS.to_string(), "2");
}
