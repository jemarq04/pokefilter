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
      force,
      output,
      lang,
      options,
      keys,
      cache_dir,
    } => {
      if let Err(e) = filter::build(force, output, &options, &keys, lang, cache_dir).await {
        e.exit();
      }
    },
  }
}
