//! Wire fields that may carry a deliberately invalid value.
//!
//! A generated request holds validated values only, but `degrade` has to put a
//! raw literal into the same key: `"bidfloor":"1.50"` where the typed model
//! holds a number, `"cur":"USD"` where it holds an array. Each field is
//! therefore a union of the typed value and a static literal. The typed variant
//! writes exactly the bytes it wrote before the union existed, so a request
//! without a mutation is byte-identical.

use core::fmt;

use crate::currency::Currency;
use crate::money::Money;
use crate::raw::RawJson;
use crate::write::{
    Object, field_id, field_number, field_raw, field_string, option_currency_array,
};

/// An identifier that is either a generated value or a raw string.
#[derive(Clone, Copy, Debug)]
pub(crate) enum WireId<T> {
    /// The generated identifier.
    Id(T),
    /// A literal that is not a valid identifier, such as `""` or `" "`.
    Literal(&'static str),
}

impl<T: Default> Default for WireId<T> {
    fn default() -> Self {
        Self::Id(T::default())
    }
}

impl<T: Copy + fmt::Display> WireId<T> {
    /// Builds an identifier from its typed value.
    pub(crate) const fn id(value: T) -> Self {
        Self::Id(value)
    }

    /// Writes the key and its value.
    pub(crate) fn write_field<W: fmt::Write>(
        &self,
        f: &mut Object<'_, W>,
        name: &str,
    ) -> fmt::Result {
        match *self {
            Self::Id(value) => field_id(f, name, value),
            Self::Literal(text) => field_string(f, name, text),
        }
    }
}

/// A money field that is either a number or a JSON string.
#[derive(Clone, Copy, Debug)]
pub(crate) enum WireNumber {
    /// A fixed-scale amount, the only valid form.
    Number(Money),
    /// A literal such as `"1.50"`, which a money-typing parser must reject.
    Text(&'static str),
}

impl WireNumber {
    /// Builds a numeric amount.
    pub(crate) const fn number(value: Money) -> Self {
        Self::Number(value)
    }

    /// Writes the key and its value.
    pub(crate) fn write_field<W: fmt::Write>(
        self,
        f: &mut Object<'_, W>,
        name: &str,
    ) -> fmt::Result {
        match self {
            Self::Number(value) => field_number(f, name, value),
            Self::Text(text) => field_string(f, name, text),
        }
    }
}

/// A currency field that is either a validated code or a raw string.
#[derive(Clone, Copy, Debug)]
pub(crate) enum WireCurrency {
    /// A three-letter uppercase code.
    Code(Currency),
    /// A literal such as `"usd"` or `""`.
    Text(&'static str),
}

impl WireCurrency {
    /// Builds a validated code.
    pub(crate) const fn code(value: Currency) -> Self {
        Self::Code(value)
    }

    /// Writes the key and its value.
    pub(crate) fn write_field<W: fmt::Write>(
        self,
        f: &mut Object<'_, W>,
        name: &str,
    ) -> fmt::Result {
        match self {
            Self::Code(value) => crate::write::field_currency(f, name, value),
            Self::Text(text) => field_string(f, name, text),
        }
    }
}

/// The `cur` field: a code array, or a literal of the wrong shape.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) enum Currencies {
    /// No currency list, which omits the key.
    #[default]
    Empty,
    /// The validated array every profile emits.
    Codes(&'static [Currency]),
    /// A JSON string instead of an array, such as `"USD"`.
    Text(&'static str),
    /// A literal array or object, such as `["US1"]`.
    Raw(RawJson),
}

impl Currencies {
    /// Builds a validated code array.
    pub(crate) const fn codes(values: &'static [Currency]) -> Self {
        Self::Codes(values)
    }

    /// Writes the key and its value, omitting an empty list.
    pub(crate) fn write_field<W: fmt::Write>(
        &self,
        f: &mut Object<'_, W>,
        name: &str,
    ) -> fmt::Result {
        match *self {
            Self::Empty => Ok(()),
            Self::Codes(values) => option_currency_array(f, name, values),
            Self::Text(text) => field_string(f, name, text),
            Self::Raw(raw) => field_raw(f, name, raw),
        }
    }
}
