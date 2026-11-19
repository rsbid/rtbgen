//! A snapshot test without another dependency: the golden corpus is the snapshot.

use rtbgen::{Profile, Request, Seed};

fn main() {
    let json = Request::builder(Profile::SiteDisplay).seed(Seed::new(42)).build().to_json_string();

    let expected = include_str!("../corpus/v1/site_display-v26-42.json");
    assert_eq!(json, expected, "the bytes drifted from corpus/v1");
}
