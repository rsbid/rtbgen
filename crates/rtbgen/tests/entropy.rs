//! Byte goldens for the entropy entry points.
//!
//! The golden corpus pins the seeded builder; these fixtures pin the other
//! public ways to produce bytes: `Request::from_entropy` and, when the feature
//! is on, the `arbitrary` adapter. A change to a byte source, to the draw order
//! or to the id allocation changes the bytes and shows up as a diff here.

#[cfg(feature = "arbitrary")]
use arbitrary::{Arbitrary, Unstructured};
use rtbgen::{BytesEntropy, Profile, Request, RngEntropy, Seed, SpecVersion};

const SITE_DISPLAY_BYTES_V26: &str = r#"{"id":"req-3038287254904252969","imp":[{"id":"1","banner":{"w":300,"h":250,"btype":[4],"battr":[1,2],"pos":1,"mimes":["image/jpeg","image/png"],"topframe":0,"expdir":[1,3],"api":[3,5],"id":"b0"},"bidfloor":0.5,"bidfloorcur":"USD","secure":1}],"site":{"id":"site-0","name":"Example News","domain":"example.com","cat":["IAB1"],"page":"https://example.com/news/42","ref":"https://search.example/","mobile":0,"privacypolicy":1,"publisher":{"id":"pub-0","name":"Example Media","domain":"example.com","cat":["IAB1","IAB2"]},"keywords":"news,sports","ext":{"exchange":"rtbgen"}},"device":{"ua":"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/131.0.0.0 Safari/537.36","ip":"203.0.113.7","geo":{"country":"RU","region":"MOW","city":"Moscow","zip":"101000","type":2,"utcoffset":180},"ifa":"A1B2C3D4-0000-4000-8000-000000000001","dnt":0,"lmt":0,"devicetype":2,"make":"Dell","model":"XPS 13","os":"Windows","osv":"11","language":"en"},"user":{"id":"u0","buyeruid":"buyer-0","consent":"CPX0000000000000000000000000000000000000000000000000000000000000","ext":{"eids":[]}},"at":1,"tmax":800,"cur":["USD"],"bcat":["IAB1","IAB2"],"badv":["bad.example","spam.test"],"regs":{"coppa":0,"gdpr":1,"gpp":"DBABMA~CPX0000000000000000000000000000000000000000000000000000000000000","gpp_sid":[2],"ext":{"us_privacy":"1YNN"}},"ext":{"exchange":"rtbgen"}}"#;

const MAXIMAL_BYTES_V25: &str = r#"{"id":"req-1229782933952336144","imp":[{"id":"1","banner":{"w":300,"h":250,"btype":[4],"battr":[1],"pos":1,"mimes":["image/jpeg"],"topframe":1,"expdir":[3],"api":[5],"id":"b7","vcm":1,"ext":{"k":1}},"video":{"mimes":["video/mp4"],"minduration":5,"maxduration":30,"protocols":[2,3],"w":640,"h":480,"startdelay":-1,"placement":1,"linearity":1,"skip":1,"skipmin":5,"skipafter":5,"api":[5],"battr":[1],"delivery":[2],"pos":1,"companionad":[{"w":300,"h":250,"btype":[4],"battr":[1],"pos":1,"mimes":["image/jpeg"],"topframe":1,"expdir":[3],"api":[5],"id":"b7","vcm":1,"ext":{"k":1}}],"companiontype":[1],"ext":{"k":1}},"audio":{"mimes":["audio/mp4"],"minduration":5,"maxduration":30,"protocols":[2,3],"startdelay":-1,"feed":1,"stitched":1,"nvol":1,"api":[5],"battr":[1],"delivery":[2],"ext":{"k":1}},"native":{"request":"{\"ver\":\"1.2\",\"assets\":[{\"id\":1,\"required\":1,\"title\":{\"len\":80}}]}","ver":"1.2","api":[5],"battr":[1],"ext":{"k":1}},"pmp":{"private_auction":1,"deals":[{"id":"deal-1","bidfloor":1.5,"bidfloorcur":"USD","at":3,"wseat":["seat-1"],"wadomain":["adv.example"],"ext":{"k":1}}],"ext":{"k":1}},"bidfloor":1.5,"bidfloorcur":"USD","secure":1,"ext":{"k":1}}],"site":{"id":"site-1","name":"Example Site","domain":"site.example","cat":["IAB1"],"page":"https://site.example/page","ref":"https://search.example/","mobile":1,"privacypolicy":1,"publisher":{"id":"pub-2","name":"Example Publisher","domain":"publisher.example","cat":["IAB1"],"ext":{"k":1}},"keywords":"news,sports","ext":{"k":1}},"device":{"ua":"Test Agent/1.0","ip":"203.0.113.7","geo":{"country":"RU","region":"MOW","city":"Moscow","zip":"101000","type":2,"utcoffset":-180,"ext":{"k":1}},"ifa":"A1B2C3D4-0000-4000-8000-000000000001","dnt":0,"lmt":0,"devicetype":2,"make":"Apple","model":"iPhone15,2","os":"iOS","osv":"18.1","language":"en","ext":{"k":1}},"user":{"id":"u4","buyeruid":"buyer-5","ext":{"k":1}},"at":2,"tmax":1000,"cur":["USD"],"bcat":["IAB1"],"badv":["bad.example"],"bapp":["bad.app"],"source":{"fd":1,"tid":"tid-1","pchain":"pchain-1","schain":{"ver":"1.0","complete":1,"nodes":[{"asi":"exchange.example","sid":"123","rid":"rid-1","n":1,"hp":1,"ext":{"k":1}}],"ext":{"k":1}},"ext":{"k":1}},"regs":{"coppa":0,"ext":{"k":1}},"ext":{"k":1}}"#;

const SPARSE_RNG_V26: &str =
    r#"{"id":"req-1546998761530907014","imp":[{"id":"1","banner":{"w":300,"h":250}}]}"#;

#[cfg(feature = "arbitrary")]
const ARBITRARY_5A_V26: &str = r#"{"id":"req-6510615551131933273","imp":[{"id":"1","banner":{"w":320,"h":50,"btype":[4],"pos":2,"mimes":["image/jpeg","image/png"],"topframe":0,"api":[3,5],"id":"b0"},"bidfloor":0.5,"bidfloorcur":"USD","secure":1}],"site":{"id":"site-0","name":"Example Daily","domain":"news.example","cat":["IAB1"],"page":"https://news.example/weather/today","ref":"https://search.example/","mobile":0,"privacypolicy":1,"publisher":{"id":"pub-0","name":"Example Media","domain":"news.example","cat":["IAB1","IAB2"]},"keywords":"weather,local","ext":{"exchange":"rtbgen"}},"device":{"ua":"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/131.0.0.0 Safari/537.36","ip":"203.0.113.7","geo":{"country":"RU","region":"MOW","city":"Moscow","zip":"101000","type":2,"utcoffset":180},"ifa":"A1B2C3D4-0000-4000-8000-000000000001","dnt":0,"lmt":0,"devicetype":2,"make":"Dell","model":"XPS 13","os":"Windows","osv":"11","language":"en"},"user":{"id":"u0","buyeruid":"buyer-0","ext":{"eids":[]}},"at":0,"tmax":800,"cur":["USD"],"bcat":["IAB1","IAB2"],"badv":["bad.example","spam.test"],"regs":{"coppa":0,"ext":{"us_privacy":"1YNN"}},"ext":{"exchange":"rtbgen"}}"#;

#[test]
fn bytes_entropy_output_is_frozen() {
    let request = Request::from_entropy(
        Profile::SiteDisplay,
        SpecVersion::V26,
        &mut BytesEntropy::new(&[0x2A; 64]),
    );

    assert_eq!(request.to_json_string(), SITE_DISPLAY_BYTES_V26);
}

#[test]
fn a_short_byte_input_is_frozen_too() {
    let request = Request::from_entropy(
        Profile::Maximal,
        SpecVersion::V25,
        &mut BytesEntropy::new(&[0x11; 40]),
    );

    assert_eq!(request.to_json_string(), MAXIMAL_BYTES_V25);
}

#[test]
fn rng_entropy_output_is_frozen() {
    let request = Request::from_entropy(
        Profile::Sparse,
        SpecVersion::V26,
        &mut RngEntropy::from_seed(Seed::new(42)),
    );

    assert_eq!(request.to_json_string(), SPARSE_RNG_V26);
}

#[cfg(feature = "arbitrary")]
#[test]
#[expect(clippy::expect_used, reason = "in tests a panic = a failed test")]
fn arbitrary_output_is_frozen() {
    let request = Request::arbitrary(&mut Unstructured::new(&[0x5A; 96]))
        .expect("the adapter is total over every byte slice");

    assert!(request.mutation().is_some(), "the frozen input selects a mutation");
    assert_eq!(request.to_json_string(), ARBITRARY_5A_V26);
}
