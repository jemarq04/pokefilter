use crate::utils::enums::*;
use clap::builder::styling::{AnsiColor, Effects, Style, Styles};
use clap::{CommandFactory, Parser, Subcommand};

pub const HEADER: Style = AnsiColor::Green.on_default().effects(Effects::BOLD);
pub const USAGE: Style = AnsiColor::Green.on_default().effects(Effects::BOLD);
pub const LITERAL: Style = AnsiColor::Cyan.on_default().effects(Effects::BOLD);
pub const PLACEHOLDER: Style = AnsiColor::Cyan.on_default();
pub const ERROR: Style = AnsiColor::Red.on_default().effects(Effects::BOLD);
pub const VALID: Style = AnsiColor::Cyan.on_default().effects(Effects::BOLD);
pub const INVALID: Style = AnsiColor::Yellow.on_default().effects(Effects::BOLD);

/// Cargo's color style
/// [source](https://github.com/crate-ci/clap-cargo/blob/master/src/style.rs)
const CARGO_STYLING: Styles = Styles::styled()
  .header(HEADER)
  .usage(USAGE)
  .literal(LITERAL)
  .placeholder(PLACEHOLDER)
  .error(ERROR)
  .valid(VALID)
  .invalid(INVALID);

/// Placeholder
#[derive(Parser, Debug)]
#[command(version, long_about, styles=CARGO_STYLING)]
pub struct Args {
  #[command(subcommand)]
  pub command: SubArgs,
}

#[derive(Subcommand, Debug)]
pub enum SubArgs {
  /// Build cached CSV file for quick searches.
  #[command(name = "build", long_about)]
  Build {
    #[arg(short, long, help = "overwrite file, if one exists")]
    force: bool,

    #[command(flatten)]
    range_opts: BuildOpts,

    #[arg(
      short,
      long,
      value_name = "PATH",
      help = "output file path for CSV file"
    )]
    output: Option<std::path::PathBuf>,

    #[arg(value_enum,
      short = 'L',
      long,
      value_name = "LANGUAGE",
      default_value_t = LanguageId::En,
      hide_possible_values=true,
      help = "language ID for API requests for formatted names"
    )]
    lang: LanguageId,

    #[arg(
      long,
      value_name = "DIR",
      help = "cache directory for API calls (default: ~/.cache/pokefilter/)"
    )]
    cache_dir: Option<std::path::PathBuf>,
  },
}

#[derive(clap::Args, Debug)]
#[group(required = true, multiple = false)]
pub struct BuildOpts {
  #[arg(
    short,
    long,
    help_heading = "Range",
    help = "print information for all Pokemon"
  )]
  pub all: bool,

  #[arg(
    short,
    long,
    help_heading = "Range",
    help = "print information for specified Pokemon"
  )]
  pub pokemon: Option<String>,

  #[arg(
    short = 'P',
    long,
    num_args = 2,
    help_heading = "Range",
    help = "print information for all between the specified Pokemon"
  )]
  pub pokerange: Option<Vec<String>>,

  #[arg(
    short = 'n',
    long,
    help_heading = "Range",
    help = "print information for Pokemon with the specified national Pokedex number"
  )]
  pub dexnum: Option<i64>,

  #[arg(
    short = 'N',
    long,
    num_args = 2,
    help_heading = "Range",
    help = "print information for all between the specified national Pokedex numbers"
  )]
  pub dexrange: Option<Vec<i64>>,
}

pub fn get_appname() -> String {
  String::from(Args::command().get_name())
}

pub fn error(kind: clap::error::ErrorKind, message: String) -> clap::Error {
  Args::command().error(kind, message)
}
