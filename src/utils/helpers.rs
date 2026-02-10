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
  let Ok(forms) =
    future::try_join_all(pokemon.forms.iter().map(async |f| f.follow(client).await)).await
  else {
    return Err(cli::error(
      ErrorKind::InvalidValue,
      format!(
        "API error: could not retrieve pokemon forms for {}",
        pokemon.name
      ),
    ));
  };

  for form in forms {
    if !form.is_default || form.names.is_empty() {
      continue;
    }
    for n in &form.names {
      if let Ok(item) = n.language.follow(client).await
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
  if let Err(why) = outfile.write_all(format!("{content}\n").as_bytes()) {
    return Err(cli::error(
      ErrorKind::InvalidValue,
      format!("error writing to file {display}: {why}"),
    ));
  }
  Ok(())
}

pub fn create_client(mut cache_dir: Option<std::path::PathBuf>) -> RustemonClient {
  if cache_dir.is_none() {
    cache_dir = {
      let mut result = None;
      if let Some(home) = std::env::home_dir() {
        let dirpath = format!("{}/.cache", home.display());
        let dirpath = Path::new(&dirpath);
        if dirpath.exists() || create_dir(dirpath).is_ok() {
          result = Some(format!("{}/{}", dirpath.display(), cli::get_appname()).into());
        }
      }
      result
    };
  }
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

pub fn get_default_cache_file() -> String {
  let mut result = String::new();
  let filename = "pokeinfo.csv";
  if let Some(home) = std::env::home_dir() {
    let dirpath = format!("{}/.{}", home.display(), cli::get_appname());
    let dirpath = Path::new(&dirpath);
    if dirpath.exists() || create_dir(dirpath).is_ok() {
      result = format!("{}/{}", dirpath.display(), filename);
    }
  }
  if result.is_empty() {
    eprintln!("warning: CSV file will be built in working directory");
    result = String::from(filename);
  }
  result
}
