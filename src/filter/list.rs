use crate::utils::{cli, helpers};
use clap::error::ErrorKind;
use std::{fs::File, path::Path};

struct Record {
  identifier: String,
  dex: i64,
  pokemon: String,
  species: String,
  generation: i64,
  types: Vec<String>,
  abilities: Vec<String>,
  color: String,
  egg_groups: Vec<String>,
  held_items: Vec<String>,
  growth_rate: String,
}

pub fn list(filepath: Option<std::path::PathBuf>) -> Result<(), clap::Error> {
  // If absent, set filepath to default
  let filepath = match filepath {
    Some(p) => p,
    None => helpers::get_default_cache_file().into(),
  };

  // Check if file exists
  let filepath = Path::new(&filepath);
  if !filepath.exists() {
    return Err(cli::error(
      ErrorKind::InvalidValue,
      format!("invalid file: {}", filepath.display()),
    ));
  }

  Ok(())
}
