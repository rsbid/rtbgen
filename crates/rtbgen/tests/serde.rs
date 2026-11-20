//! The `serde` adapter: serializing a request equals writing it.
#![cfg(feature = "serde")]
#![expect(clippy::expect_used, reason = "in tests a panic = a failed test")]

use rtbgen::corpus;
use serde_json::Value;

#[test]
fn serde_output_matches_the_writer_for_every_corpus_entry() {
    for entry in corpus::ALL {
        let request = entry.request();
        let written = request.to_json_string();
        let serialized = serde_json::to_string(&request).expect("a request always serializes");

        assert_eq!(serialized, written, "{} serialized differently", entry.name());
    }
}

#[test]
fn to_value_is_a_tree_for_inspection() {
    for entry in corpus::ALL {
        let request = entry.request();
        let written = request.to_json_string();
        let from_writer: Value =
            serde_json::from_str(&written).expect("the writer emits valid JSON");
        let from_serde: Value = serde_json::to_value(request).expect("a request always serializes");

        assert_eq!(from_serde, from_writer, "{} changed shape", entry.name());
    }
}
