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
  gender_rate: f32,
  base_stats: Vec<i64>,
  ev_yields: Vec<i64>,
  hatch_counter: Option<i64>,
  base_exp: Option<i64>,
  capture_rate: i64,
  base_happiness: Option<i64>,
  height: i64,
  weight: i64,
  stage: i64,
  is_branched: bool,
  is_branching: bool,
  is_starter: bool,
  is_fossil: bool,
  is_baby: bool,
  is_mythical: bool,
  is_legendary: bool,
  is_ub: bool,
  is_paradox: bool,
  category: String,
  category_id: i64,
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
