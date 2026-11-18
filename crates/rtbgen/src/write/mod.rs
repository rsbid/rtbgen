//! Hand-written JSON writer.
//!
//! Every object is written by a fixed sequence of key/value calls; there is no
//! map, no sorting and no allocation. Key order is part of the public byte
//! contract.

use core::fmt;

use crate::currency::Currency;
use crate::money::Money;
use crate::raw::RawJson;
use crate::version::SpecVersion;
use crate::wire::RequestModel;

/// Writes the complete bid request.
pub(crate) fn request<W: fmt::Write>(
    out: &mut W,
    model: &RequestModel,
    spec: SpecVersion,
) -> fmt::Result {
    model.write(out, spec)
}

/// The cursor of one JSON object. It tracks whether a comma is needed.
pub(crate) struct Object<'sink, W: fmt::Write> {
    out: &'sink mut W,
    first: bool,
}

impl<'sink, W: fmt::Write> Object<'sink, W> {
    pub(crate) const fn new(out: &'sink mut W) -> Self {
        Self { out, first: true }
    }

    pub(crate) fn key(&mut self, name: &str) -> fmt::Result {
        if self.first {
            self.first = false;
        } else {
            self.out.write_char(',')?;
        }
        write!(self.out, "\"{name}\":")
    }

    pub(crate) fn string(&mut self, value: &str) -> fmt::Result {
        json_string(self.out, value)
    }

    /// Gives a nested writer access to the underlying sink.
    pub(crate) const fn out(&mut self) -> &mut W {
        self.out
    }
}

/// Writes one object, including its braces.
pub(crate) fn object<W, F>(out: &mut W, write: F) -> fmt::Result
where
    W: fmt::Write,
    F: FnOnce(&mut Object<'_, W>) -> fmt::Result,
{
    out.write_char('{')?;
    let mut object = Object::new(out);
    write(&mut object)?;
    object.out.write_char('}')
}

/// Writes a key and a nested object.
pub(crate) fn object_field<W, F>(f: &mut Object<'_, W>, name: &str, write: F) -> fmt::Result
where
    W: fmt::Write,
    F: FnOnce(&mut Object<'_, W>) -> fmt::Result,
{
    f.key(name)?;
    object(f.out(), write)
}

/// Writes a key and a number.
pub(crate) fn field_number<W, T>(f: &mut Object<'_, W>, name: &str, value: T) -> fmt::Result
where
    W: fmt::Write,
    T: fmt::Display,
{
    f.key(name)?;
    write!(f.out, "{value}")
}

/// Writes an optional number, omitting the key when absent.
pub(crate) fn option_number<W, T>(
    f: &mut Object<'_, W>,
    name: &str,
    value: Option<T>,
) -> fmt::Result
where
    W: fmt::Write,
    T: fmt::Display,
{
    if let Some(value) = value {
        field_number(f, name, value)?;
    }
    Ok(())
}

/// Writes a key and an escaped JSON string.
pub(crate) fn field_string<W: fmt::Write>(
    f: &mut Object<'_, W>,
    name: &str,
    value: &str,
) -> fmt::Result {
    f.key(name)?;
    f.string(value)
}

/// Writes a key and an identifier as a JSON string.
///
/// Identifier contents are produced by this crate: ASCII letters, digits and
/// the `-` separator, so they need no escaping.
pub(crate) fn field_id<W, T>(f: &mut Object<'_, W>, name: &str, value: T) -> fmt::Result
where
    W: fmt::Write,
    T: fmt::Display,
{
    f.key(name)?;
    f.out().write_char('"')?;
    write!(f.out(), "{value}")?;
    f.out().write_char('"')
}

/// Writes an optional identifier, omitting the key when absent.
pub(crate) fn option_id<W, T>(f: &mut Object<'_, W>, name: &str, value: Option<T>) -> fmt::Result
where
    W: fmt::Write,
    T: fmt::Display,
{
    if let Some(value) = value {
        field_id(f, name, value)?;
    }
    Ok(())
}

/// Writes an optional escaped JSON string.
pub(crate) fn option_string<W: fmt::Write>(
    f: &mut Object<'_, W>,
    name: &str,
    value: Option<&str>,
) -> fmt::Result {
    if let Some(value) = value {
        field_string(f, name, value)?;
    }
    Ok(())
}

/// Writes a key and a fixed-scale money amount.
pub(crate) fn field_money<W: fmt::Write>(
    f: &mut Object<'_, W>,
    name: &str,
    value: Money,
) -> fmt::Result {
    field_number(f, name, value)
}

/// Writes an optional money amount.
pub(crate) fn option_money<W: fmt::Write>(
    f: &mut Object<'_, W>,
    name: &str,
    value: Option<Money>,
) -> fmt::Result {
    if let Some(value) = value {
        field_money(f, name, value)?;
    }
    Ok(())
}

/// Writes a key and a currency code.
pub(crate) fn field_currency<W: fmt::Write>(
    f: &mut Object<'_, W>,
    name: &str,
    value: Currency,
) -> fmt::Result {
    f.key(name)?;
    value.write_literal(f.out())
}

/// Writes an optional currency code.
pub(crate) fn option_currency<W: fmt::Write>(
    f: &mut Object<'_, W>,
    name: &str,
    value: Option<Currency>,
) -> fmt::Result {
    if let Some(value) = value {
        field_currency(f, name, value)?;
    }
    Ok(())
}

/// Writes a key and a trusted raw JSON literal.
pub(crate) fn field_raw<W: fmt::Write>(
    f: &mut Object<'_, W>,
    name: &str,
    value: RawJson,
) -> fmt::Result {
    f.key(name)?;
    f.out.write_str(value.as_str())
}

/// Writes an optional raw JSON literal.
pub(crate) fn option_raw<W: fmt::Write>(
    f: &mut Object<'_, W>,
    name: &str,
    value: Option<RawJson>,
) -> fmt::Result {
    if let Some(value) = value {
        field_raw(f, name, value)?;
    }
    Ok(())
}

/// Writes a JSON string, escaping exactly the characters the contract names.
pub(crate) fn json_string<W: fmt::Write>(out: &mut W, value: &str) -> fmt::Result {
    out.write_char('"')?;

    for character in value.chars() {
        match character {
            '"' => out.write_str("\\\"")?,
            '\\' => out.write_str("\\\\")?,
            '\u{08}' => out.write_str("\\b")?,
            '\u{0C}' => out.write_str("\\f")?,
            '\n' => out.write_str("\\n")?,
            '\r' => out.write_str("\\r")?,
            '\t' => out.write_str("\\t")?,
            control if u32::from(control) < 0x20 => {
                write!(out, "\\u{:04x}", u32::from(control))?;
            }
            character => out.write_char(character)?,
        }
    }

    out.write_char('"')
}

/// Writes an escaped string array.
pub(crate) fn string_array<W: fmt::Write>(out: &mut W, values: &[&str]) -> fmt::Result {
    out.write_char('[')?;

    let mut first = true;
    for value in values {
        if first {
            first = false;
        } else {
            out.write_char(',')?;
        }
        json_string(out, value)?;
    }

    out.write_char(']')
}

/// Writes a numeric array.
pub(crate) fn number_array<W: fmt::Write, T: fmt::Display>(
    out: &mut W,
    values: &[T],
) -> fmt::Result {
    out.write_char('[')?;

    let mut first = true;
    for value in values {
        if first {
            first = false;
        } else {
            out.write_char(',')?;
        }
        write!(out, "{value}")?;
    }

    out.write_char(']')
}

/// Writes a currency array.
pub(crate) fn currency_array<W: fmt::Write>(out: &mut W, values: &[Currency]) -> fmt::Result {
    out.write_char('[')?;

    let mut first = true;
    for value in values {
        if first {
            first = false;
        } else {
            out.write_char(',')?;
        }
        value.write_literal(out)?;
    }

    out.write_char(']')
}

/// Writes an optional string array, omitting an empty one.
pub(crate) fn option_string_array<W: fmt::Write>(
    f: &mut Object<'_, W>,
    name: &str,
    values: &[&str],
) -> fmt::Result {
    if values.is_empty() {
        return Ok(());
    }
    f.key(name)?;
    string_array(f.out(), values)
}

/// Writes an optional numeric array, omitting an empty one.
pub(crate) fn option_number_array<W: fmt::Write, T: fmt::Display>(
    f: &mut Object<'_, W>,
    name: &str,
    values: &[T],
) -> fmt::Result {
    if values.is_empty() {
        return Ok(());
    }
    f.key(name)?;
    number_array(f.out(), values)
}

/// Writes an optional currency array, omitting an empty one.
pub(crate) fn option_currency_array<W: fmt::Write>(
    f: &mut Object<'_, W>,
    name: &str,
    values: &[Currency],
) -> fmt::Result {
    if values.is_empty() {
        return Ok(());
    }
    f.key(name)?;
    currency_array(f.out(), values)
}

#[cfg(test)]
mod tests;
