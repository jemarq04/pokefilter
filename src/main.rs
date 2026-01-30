mod filter;
mod utils;

use clap::Parser;
use rustemon::client::RustemonClient;
use utils::cli::{Args, SubArgs, get_appname};

#[tokio::main]
async fn main() {
  let mut args = Args::parse();

  // Create cache directory for API calls
  if let None = args.cache_dir {
    args.cache_dir = match std::env::home_dir() {
      Some(path) => Some(format!("{}/.cache/{}", path.display(), get_appname()).into()),
      None => None,
    }
  }
  let client = match args.cache_dir {
    Some(path) => {
      match rustemon::client::RustemonClientBuilder::default()
        .with_manager(rustemon::client::CACacheManager::new(path, false))
        .try_build()
      {
        Ok(cl) => cl,
        Err(_) => {
          eprintln!("warning: cache directory set to cache manager default");
          RustemonClient::default()
        },
      }
    },
    None => {
      eprintln!("warning: cache directory set to cache manager default");
      RustemonClient::default()
    },
  };

  // Call the appropriate subcommand for results
  match args.command {
    SubArgs::BuildCmd { force, path, lang } => {
      if let Err(e) = filter::build(&client, force, path, lang).await {
        e.exit();
      }
    },
  }
}
