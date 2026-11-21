//! Argument parsing for the command-line interface.

use std::fmt;
use std::path::PathBuf;

use rtbgen::{Currency, Deals, DegradeRate, Floor, ImpressionCount, Profile, Seed, SpecVersion};

/// A fully parsed command.
pub(crate) enum Command {
    Generate(Generate),
    ListProfiles,
    Corpus(Corpus),
    Help,
    Version,
}

/// The request generation command.
#[expect(
    clippy::field_scoped_visibility_modifiers,
    reason = "the binary modules share these parsed fields"
)]
pub(crate) struct Generate {
    pub(crate) profile: Profile,
    pub(crate) seed: Seed,
    pub(crate) spec: SpecVersion,
    pub(crate) count: u32,
    pub(crate) impressions: Option<ImpressionCount>,
    pub(crate) deals: Option<Deals>,
    pub(crate) floor: Option<Floor>,
    pub(crate) degrade: Option<DegradeRate>,
    pub(crate) out: Option<PathBuf>,
}

/// The corpus subcommand.
#[expect(
    clippy::field_scoped_visibility_modifiers,
    reason = "the binary modules share these parsed fields"
)]
pub(crate) struct Corpus {
    pub(crate) dir: PathBuf,
    pub(crate) write: bool,
}

/// The default corpus directory, relative to the repository root.
const DEFAULT_CORPUS_DIR: &str = "crates/rtbgen/corpus/v1";

impl Command {
    /// Parses the process arguments, without the program name.
    pub(crate) fn parse(mut args: impl Iterator<Item = String>) -> Result<Self, Error> {
        let Some(first) = args.next() else {
            return Ok(Self::Generate(Generate::default()));
        };

        if first == "corpus" {
            parse_corpus(args)
        } else {
            Parser::new(std::iter::once(first).chain(args)).run()
        }
    }
}

impl Default for Generate {
    fn default() -> Self {
        Self {
            profile: Profile::SiteDisplay,
            seed: Seed::new(0),
            spec: SpecVersion::V26,
            count: 1,
            impressions: None,
            deals: None,
            floor: None,
            degrade: None,
            out: None,
        }
    }
}

/// The generator-flag parser. `--floor` and `--currency` are combined at the
/// end, so their order on the command line does not matter.
struct Parser<I> {
    args: I,
    generate: Generate,
    floor_cpm: Option<f64>,
    currency: Currency,
}

impl<I: Iterator<Item = String>> Parser<I> {
    fn new(args: I) -> Self {
        Self { args, generate: Generate::default(), floor_cpm: None, currency: Currency::USD }
    }

    fn run(mut self) -> Result<Command, Error> {
        while let Some(argument) = self.args.next() {
            if let Some(command) = meta(&argument) {
                return Ok(command);
            }
            self.option(&argument)?;
        }

        let mut generate = self.generate;
        generate.floor = self.floor_cpm.map(|cpm| Floor::cpm(cpm, self.currency));

        Ok(Command::Generate(generate))
    }

    fn option(&mut self, flag: &str) -> Result<(), Error> {
        match flag {
            "--profile" => {
                let value = self.value("--profile")?;
                self.generate.profile = Profile::from_name(&value)
                    .map_err(|error| Error::invalid("--profile", value, &error))?;
            }
            "--spec" => {
                let value = self.value("--spec")?;
                self.generate.spec = SpecVersion::from_name(&value)
                    .map_err(|error| Error::invalid("--spec", value, &error))?;
            }
            "--seed" => {
                let value = self.value("--seed")?;
                self.generate.seed = Seed::new(parse(&value, "--seed")?);
            }
            "--count" => {
                let value = self.value("--count")?;
                self.generate.count = parse(&value, "--count")?;
            }
            "--impressions" => {
                let value = self.value("--impressions")?;
                let count: u8 = parse(&value, "--impressions")?;
                self.generate.impressions = Some(ImpressionCount::new(count));
            }
            "--deals" => {
                let value = self.value("--deals")?;
                let count: u8 = parse(&value, "--deals")?;
                self.generate.deals = Some(Deals::new(count));
            }
            "--floor" => {
                let value = self.value("--floor")?;
                self.floor_cpm = Some(parse(&value, "--floor")?);
            }
            "--currency" => {
                let value = self.value("--currency")?;
                self.currency = Currency::from_code(&value)
                    .map_err(|error| Error::invalid("--currency", value, &error))?;
            }
            "--degrade" => {
                let value = self.value("--degrade")?;
                self.generate.degrade = Some(DegradeRate::percent(parse_degrade(&value)?));
            }
            "--out" => {
                self.generate.out = Some(PathBuf::from(self.value("--out")?));
            }
            other => return Err(Error::UnknownArgument(other.to_owned())),
        }

        Ok(())
    }

    fn value(&mut self, flag: &'static str) -> Result<String, Error> {
        self.args.next().ok_or(Error::MissingValue(flag))
    }
}

fn meta(argument: &str) -> Option<Command> {
    match argument {
        "--help" | "-h" => Some(Command::Help),
        "--version" | "-V" => Some(Command::Version),
        "--list-profiles" => Some(Command::ListProfiles),
        _ => None,
    }
}

fn parse<T>(value: &str, flag: &'static str) -> Result<T, Error>
where
    T: std::str::FromStr,
    T::Err: fmt::Display,
{
    value.parse::<T>().map_err(|error| Error::invalid(flag, value.to_owned(), &error))
}

/// Parses `5` or `5%` as a degrade percentage.
fn parse_degrade(value: &str) -> Result<u16, Error> {
    let digits = value.strip_suffix('%').unwrap_or(value);
    let percent: u16 =
        digits.parse().map_err(|error| Error::invalid("--degrade", value.to_owned(), &error))?;

    if percent > 100 {
        return Err(Error::invalid(
            "--degrade",
            value.to_owned(),
            &"a percentage between 0 and 100",
        ));
    }

    Ok(percent)
}

fn parse_corpus(mut args: impl Iterator<Item = String>) -> Result<Command, Error> {
    let mut directory = PathBuf::from(DEFAULT_CORPUS_DIR);
    let mut write = false;
    let mut verify = false;

    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--verify" => verify = true,
            "--write" => write = true,
            "--dir" => {
                directory = PathBuf::from(args.next().ok_or(Error::MissingValue("--dir"))?);
            }
            "--help" | "-h" => return Ok(Command::Help),
            other => return Err(Error::UnknownArgument(other.to_owned())),
        }
    }

    if write && verify {
        return Err(Error::ConflictingModes);
    }

    Ok(Command::Corpus(Corpus { dir: directory, write }))
}

/// What went wrong before a command could run.
#[derive(Debug)]
pub(crate) enum Error {
    /// A flag was not recognised.
    UnknownArgument(String),
    /// A flag that takes a value was the last argument.
    MissingValue(&'static str),
    /// A flag value could not be parsed.
    InvalidValue { flag: &'static str, value: Box<str>, reason: Box<str> },
    /// An argument was not valid UTF-8.
    NonUnicode,
    /// `--write` and `--verify` were given together.
    ConflictingModes,
    /// The corpus on disk differs from the generator.
    CorpusMismatch(u32),
    /// The filesystem or a stream failed.
    Io(std::io::Error),
}

impl Error {
    fn invalid(flag: &'static str, value: String, reason: &impl fmt::Display) -> Self {
        Self::InvalidValue {
            flag,
            value: value.into_boxed_str(),
            reason: reason.to_string().into_boxed_str(),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownArgument(argument) => write!(f, "unexpected argument `{argument}`"),
            Self::MissingValue(flag) => write!(f, "`{flag}` needs a value"),
            Self::InvalidValue { flag, value, reason } => {
                write!(f, "invalid value `{value}` for `{flag}`: {reason}")
            }
            Self::NonUnicode => f.write_str("arguments must be valid UTF-8"),
            Self::ConflictingModes => f.write_str("`--write` and `--verify` cannot be combined"),
            Self::CorpusMismatch(count) => {
                write!(f, "{count} corpus fixtures differ from the generator")
            }
            Self::Io(error) => write!(f, "{error}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use rtbgen::{DegradeRate, Profile, SpecVersion};

    use super::{Command, Error};

    fn parse_command(args: &[&str]) -> Result<Command, Error> {
        Command::parse(args.iter().copied().map(String::from))
    }

    #[test]
    fn the_empty_command_generates_one_site_display() {
        let command = parse_command(&[]).ok();

        assert!(matches!(
            command,
            Some(Command::Generate(generate))
                if generate.profile == Profile::SiteDisplay
                    && generate.seed.into_inner() == 0
                    && generate.spec == SpecVersion::V26
                    && generate.count == 1
        ));
    }

    #[test]
    fn the_profile_and_spec_are_parsed() {
        let command = parse_command(&["--profile", "app_rewarded", "--spec", "2.5"]).ok();

        assert!(matches!(
            command,
            Some(Command::Generate(generate))
                if generate.profile == Profile::AppRewarded
                    && generate.spec == SpecVersion::V25
        ));
    }

    #[test]
    fn the_floor_and_currency_can_come_in_any_order() {
        let first = parse_command(&["--floor", "1.5", "--currency", "EUR"]).ok();
        let second = parse_command(&["--currency", "EUR", "--floor", "1.5"]).ok();

        let floor = |command: Option<Command>| match command {
            Some(Command::Generate(generate)) => generate.floor,
            _ => None,
        };

        let first = floor(first);
        let second = floor(second);

        assert_eq!(first, second);
        assert!(second.is_some());
    }

    #[test]
    fn the_degrade_rate_accepts_percent_and_a_bare_number() {
        let rate = |command: Option<Command>| match command {
            Some(Command::Generate(generate)) => generate.degrade.map(DegradeRate::basis_points),
            _ => None,
        };

        assert_eq!(rate(parse_command(&["--degrade", "5%"]).ok()), Some(500));
        assert_eq!(rate(parse_command(&["--degrade", "100"]).ok()), Some(10_000));
        assert_eq!(rate(parse_command(&["--degrade", "0"]).ok()), Some(0));
    }

    #[test]
    fn a_bad_degrade_rate_is_rejected() {
        for value in ["101", "abc", "5%%", "-1", ""] {
            assert!(
                matches!(parse_command(&["--degrade", value]), Err(Error::InvalidValue { .. })),
                "{value:?} was accepted"
            );
        }
    }

    #[test]
    fn unknown_flags_and_bad_values_are_errors() {
        assert!(matches!(parse_command(&["--nonsense"]), Err(Error::UnknownArgument(_))));
        assert!(matches!(parse_command(&["--seed"]), Err(Error::MissingValue("--seed"))));
        assert!(matches!(
            parse_command(&["--profile", "AppRewarded"]),
            Err(Error::InvalidValue { .. })
        ));
    }

    #[test]
    fn the_corpus_subcommand_parses() {
        assert!(matches!(
            parse_command(&["corpus", "--verify"]),
            Ok(Command::Corpus(corpus)) if !corpus.write
        ));
        assert!(matches!(
            parse_command(&["corpus", "--write", "--dir", "x"]),
            Ok(Command::Corpus(corpus)) if corpus.write
        ));
        assert!(matches!(
            parse_command(&["corpus", "--write", "--verify"]),
            Err(Error::ConflictingModes)
        ));
    }

    #[test]
    fn the_help_and_list_flags_short_circuit() {
        assert!(matches!(parse_command(&["--help"]), Ok(Command::Help)));
        assert!(matches!(
            parse_command(&["--profile", "pmp_deals", "--list-profiles"]),
            Ok(Command::ListProfiles)
        ));
    }
}
