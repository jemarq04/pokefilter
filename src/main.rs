mod filter;
mod utils;

use clap::Parser;
use utils::cli::{Args, SubArgs};

#[tokio::main]
async fn main() {
  let args = Args::parse();

  // Call the appropriate subcommand for results
  match args.command {
    SubArgs::BuildCmd {
      output: filepath,
      force,
      range_opts,
      keys,
      lang,
      cache_dir,
    } => {
      if let Err(e) = filter::build(filepath, force, range_opts, &keys, lang, cache_dir).await {
        e.exit();
      }
    },
  }
}
