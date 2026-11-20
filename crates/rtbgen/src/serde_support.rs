//! The `serde` adapter: serialize a request exactly as the writer emits it.
//!
//! The adapter does not have a second writer. It serializes the request's own
//! JSON string as a [`RawValue`], so key order and number formatting are the
//! bytes of [`Request::to_json_string`](crate::Request::to_json_string).

use serde::ser::{Error as _, Serialize, Serializer};
use serde_json::value::RawValue;

use crate::request::Request;

impl Serialize for Request {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let raw = RawValue::from_string(self.to_json_string()).map_err(S::Error::custom)?;

        raw.serialize(serializer)
    }
}
