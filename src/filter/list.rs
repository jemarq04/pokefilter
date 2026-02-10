use crate::utils::{cli, helpers};
use clap::error::ErrorKind;
use serde::{self, Deserialize, de};
use std::{fs::File, path::Path};

#[derive(Clone, Debug, Deserialize)]
struct Record {
  identifier: String,
  national_dex: i64,
  pokemon: String,
  species: String,
  generation: i64,
  #[serde(deserialize_with = "deserialize_list")]
  types: Vec<String>,
  #[serde(deserialize_with = "deserialize_list")]
  abilities: Vec<String>,
  color: String,
  #[serde(deserialize_with = "deserialize_list")]
  egg_groups: Vec<String>,
  #[serde(deserialize_with = "deserialize_list")]
  held_items: Vec<String>,
  growth_rate: String,
  gender_rate: f32,
  #[serde(deserialize_with = "deserialize_list")]
  base_stats: Vec<i64>,
  #[serde(deserialize_with = "deserialize_list")]
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
  category_order: i64,
}

fn deserialize_list<'de, D, V>(deserializer: D) -> Result<Vec<V>, D::Error>
where
  D: de::Deserializer<'de>,
  V: std::str::FromStr,
  <V as std::str::FromStr>::Err: std::fmt::Debug,
{
  let s = String::deserialize(deserializer)?;
  let mut result = Vec::new();
  for item in s.split(';') {
    result.push(item.parse::<V>().unwrap());
  }
  Ok(result)
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
  let infile = File::open(filepath).map_err(|why| {
    cli::error(
      ErrorKind::InvalidValue,
      format!("error opening file {}: {}", filepath.display(), why),
    )
  })?;

  let mut reader = csv::Reader::from_reader(infile);
  //for result in reader.deserialize() {
  if let Some(result) = reader.deserialize().next() {
    let record: Record =
      result.map_err(|err| cli::error(ErrorKind::InvalidValue, format!("{err}")))?;

    println!("record: {record:?}");
  }

  Ok(())
}
