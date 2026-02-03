use crate::get_name_strict;
use crate::utils::cli;
use clap::error::ErrorKind;
use futures::future;
use rustemon::Follow;
use rustemon::client::RustemonClient;
use std::fs::{File, create_dir};
use std::io::prelude::*;
use std::path::{Display, Path};

pub async fn get_pokemon_name(
  client: &RustemonClient,
  pokemon: &rustemon::model::pokemon::Pokemon,
  lang: &str,
) -> Result<String, clap::Error> {
  let forms =
    match future::try_join_all(pokemon.forms.iter().map(async |f| f.follow(&client).await)).await {
      Ok(x) => x,
      Err(_) => {
        return Err(cli::error(
          ErrorKind::InvalidValue,
          format!(
            "API error: could not retrieve pokemon forms for {}",
            pokemon.name
          ),
        ));
      },
    };

  for form in forms.into_iter() {
    if !form.is_default || form.names.len() == 0 {
      continue;
    }
    for n in form.names.iter() {
      if let Ok(item) = n.language.follow(&client).await
        && item.name == lang
      {
        return Ok(n.name.clone());
      }
    }
    break;
  }

  get_name_strict!(follow pokemon.species, client, lang)
}

pub fn fwriteln(outfile: &mut File, display: &Display, content: &str) -> Result<(), clap::Error> {
  if let Err(why) = outfile.write_all(format!("{}\n", content).as_bytes()) {
    return Err(cli::error(
      ErrorKind::InvalidValue,
      format!("error writing to file {}: {}", display, why),
    ));
  }
  Ok(())
}

pub fn create_client(cache_dir: Option<std::path::PathBuf>) -> RustemonClient {
  // Create cache directory for API calls
  let cache_dir = match cache_dir {
    Some(p) => Some(p),
    None => {
      let mut result = None;
      if let Some(home) = std::env::home_dir() {
        let dirpath = format!("{}/.cache", home.display());
        let dirpath = Path::new(&dirpath);
        if dirpath.exists() || matches!(create_dir(dirpath), Ok(_)) {
          result = Some(format!("{}/{}", dirpath.display(), cli::get_appname()).into());
        }
      }
      result
    },
  };
  if let Some(path) = cache_dir
    && let Ok(client) = rustemon::client::RustemonClientBuilder::default()
      .with_manager(rustemon::client::CACacheManager::new(path, false))
      .try_build()
  {
    client
  } else {
    eprintln!("warning: cache directory set to cache manager default");
    RustemonClient::default()
  }
}
