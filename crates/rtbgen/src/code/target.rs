//! Targeting codes: the device and its location.

use core::fmt;

/// The general type of a device: `Device.devicetype`.
///
/// Values come from `AdCOM`'s device-types list; `1..=8` are standard and
/// exchange-specific values are `500` and greater.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct DeviceType(u16);

impl DeviceType {
    /// A desktop or laptop personal computer.
    pub(crate) const PERSONAL_COMPUTER: Self = Self(2);

    /// A mobile phone.
    pub(crate) const PHONE: Self = Self(4);
}

impl fmt::Display for DeviceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The source of a location: `Geo.type`.
///
/// Values come from `AdCOM`'s location-types list; `1..=3` are standard and
/// exchange-specific values are `500` and greater.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LocationType(u16);

impl LocationType {
    /// The location was derived from the IP address.
    pub(crate) const IP_ADDRESS: Self = Self(2);
}

impl fmt::Display for LocationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
