//! Command-line interface for `rtbgen`.

#[expect(
    clippy::print_stdout,
    reason = "the command-line interface writes its output to stdout"
)]
fn main() {
    let version = env!("CARGO_PKG_VERSION");
    let profiles = rtbgen::Profile::ALL.len();
    let specs = rtbgen::SpecVersion::ALL.len();
    let default_spec = rtbgen::SpecVersion::default();

    println!(
        "rtbgen {version}: {profiles} profiles, {specs} spec versions (default {default_spec})"
    );
}
