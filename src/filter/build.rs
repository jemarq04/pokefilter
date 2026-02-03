use crate::get_name_strict;
use crate::utils::{cli, enums::LanguageId, helpers};
use clap::error::ErrorKind;
use rustemon::model::pokemon::{Pokemon, PokemonSpecies};
use rustemon::{Follow, client::RustemonClient};
use std::collections::HashMap;
use std::fs::{File, create_dir};
use std::path::Path;

const LAST_SPECIES_ID: i64 = 1025;
// Remaining in previous format:
// Regional Dexes,Past Types,Past Abilities,(Past Stats),
const DEFAULT_HEADER: &str = "pokemon_id,national_dex,pokemon,species,generation,types,abilities,color,egg_groups,held_items,\
  growth_rate,gender_rate,base_stats,EV_yields,hatch_counter,base_EXP,capture_rate,base_happiness,height,weight,\
  stage,evolution_type,is_starter,is_fossil,is_baby,is_mythical,is_legendary,is_UB,is_paradox,category,category_id";

pub async fn build(
  filepath: Option<std::path::PathBuf>,
  force: bool,
  range_opts: cli::BuildOpts,
  keys: &Option<Vec<String>>,
  lang: LanguageId,
  cache_dir: Option<std::path::PathBuf>,
) -> Result<(), clap::Error> {
  let client = helpers::create_client(cache_dir);

  let filepath = match filepath {
    Some(p) => p,
    None => {
      let mut result = String::new();
      if let Some(home) = std::env::home_dir() {
        let dirpath = format!("{}/.{}", home.display(), cli::get_appname());
        let dirpath = Path::new(&dirpath);
        if dirpath.exists() || matches!(create_dir(dirpath), Ok(_)) {
          result = format!("{}/pokeinfo.csv", dirpath.display());
        }
      }
      if result.is_empty() {
        eprintln!("warning: CSV file will be built in working directory");
        result = String::from("pokeinfo.csv");
      }
      result
    }
    .into(),
  };
  let filepath = Path::new(&filepath);
  if !force && filepath.exists() {
    let valid = cli::VALID;
    return Err(cli::error(
      ErrorKind::InvalidValue,
      format!(
        "error: file {} already exists\n\n{valid}tip:{valid:#} to overwrite it, run '{} build --force'",
        filepath.display(),
        cli::get_appname()
      ),
    ));
  }

  let mut outfile = match File::create(&filepath) {
    Ok(file) => file,
    Err(why) => {
      return Err(cli::error(
        ErrorKind::InvalidValue,
        format!("error creating file {}: {}", filepath.display(), why),
      ));
    },
  };

  let keys = match keys {
    None => DEFAULT_HEADER.split(",").collect::<Vec<&str>>(),
    Some(k) => k.into_iter().map(|x| x.as_str()).collect::<Vec<&str>>(),
  };

  helpers::fwriteln(&mut outfile, &filepath.display(), &keys.join(","))?;

  let mut start = 1;
  let mut end = LAST_SPECIES_ID;
  if let Some(pokemon) = &range_opts.pokemon {
    let species = match rustemon::pokemon::pokemon_species::get_by_name(&pokemon, &client).await {
      Ok(obj) => obj,
      Err(_) => {
        return Err(cli::error(
          ErrorKind::InvalidValue,
          format!("API error: could not retrieve species {}", pokemon),
        ));
      },
    };
    start = species.id;
    end = species.id;
  } else if let Some(pokerange) = &range_opts.pokerange {
    let species_start =
      match rustemon::pokemon::pokemon_species::get_by_name(&pokerange[0], &client).await {
        Ok(obj) => obj,
        Err(_) => {
          return Err(cli::error(
            ErrorKind::InvalidValue,
            format!("API error: could not retrieve species {}", pokerange[0]),
          ));
        },
      };
    let species_end =
      match rustemon::pokemon::pokemon_species::get_by_name(&pokerange[1], &client).await {
        Ok(obj) => obj,
        Err(_) => {
          return Err(cli::error(
            ErrorKind::InvalidValue,
            format!("API error: could not retrieve species {}", pokerange[1]),
          ));
        },
      };
    start = species_start.id;
    end = species_end.id;
  } else if let Some(dexnum) = range_opts.dexnum {
    start = dexnum;
    end = dexnum;
  } else if let Some(dexrange) = &range_opts.dexrange {
    start = dexrange[0];
    end = dexrange[1];
  }
  for id in start..=end {
    let species = match rustemon::pokemon::pokemon_species::get_by_id(id, &client).await {
      Ok(obj) => obj,
      Err(_) => {
        return Err(cli::error(
          ErrorKind::InvalidValue,
          format!("API error: could not retrieve species with ID {}", id),
        ));
      },
    };

    for variety in species.varieties.iter() {
      let mon = match variety.pokemon.follow(&client).await {
        Ok(obj) => obj,
        Err(_) => {
          return Err(cli::error(
            ErrorKind::InvalidValue,
            format!(
              "API error: could not retrieve species variety: {}",
              variety.pokemon.name
            ),
          ));
        },
      };
      println!("Building {}...", mon.name);
      helpers::fwriteln(
        &mut outfile,
        &filepath.display(),
        &build_pokemon(&client, &species, &mon, lang, &keys).await?,
      )?;
    }
  }

  Ok(())
}

pub async fn build_pokemon(
  client: &RustemonClient,
  species: &PokemonSpecies,
  mon: &Pokemon,
  lang: LanguageId,
  keys: &Vec<&str>,
) -> Result<String, clap::Error> {
  let mut output = Vec::new();
  /*
  const DEFAULT_HEADER: &str = "pokemon_id,national_dex,pokemon,species,generation,types,abilities,color,egg_groups,held_items,\
    growth_rate,gender_rate,base_stats,EV_yields,hatch_counter,base_EXP,capture_rate,base_happiness,height,weight,\
    stage,evolution_type,is_starter,is_fossil,is_baby,is_mythical,is_legendary,is_UB,is_paradox,category,category_id";
  */

  let mut generation = None;
  let mut stage = None;
  let mut evolution_type = None;
  let mut category = None;
  for key in keys.iter() {
    output.push(match *key {
      // Name/ID
      "pokemon_id" => mon.name.to_string(),
      "national_dex" => species.id.to_string(),
      "pokemon" => helpers::get_pokemon_name(&client, &mon, &lang.to_string()).await?,
      "species" => get_name_strict!(species, client, lang.to_string())?,
      // Generation
      "generation" => {
        if let None = generation {
          generation = Some(match species.generation.follow(&client).await {
            Ok(obj) => obj,
            Err(_) => {
              return Err(cli::error(
                ErrorKind::InvalidValue,
                format!(
                  "API error: could not retrieve generation {}",
                  species.generation.name
                ),
              ));
            },
          });
        }
        generation.clone().unwrap().id.to_string()
      },
      // Types
      "types" => {
        let mut types = Vec::new();
        for r_type in mon.types.iter() {
          types.push(get_name_strict!(follow r_type.type_, client, lang.to_string())?);
        }
        types.join(";")
      },
      // Abilities
      "abilities" => {
        let mut abilities = Vec::new();
        for r_ability in mon.abilities.iter() {
          let mut ability: String =
            get_name_strict!(follow r_ability.ability, client, lang.to_string())?;
          if r_ability.is_hidden {
            ability.push_str("*");
          }
          abilities.push(ability);
        }
        abilities.join(";")
      },
      // Color
      "color" => get_name_strict!(follow species.color, client, lang.to_string())?,
      // Egg Groups
      "egg_groups" => {
        let mut egg_groups = Vec::new();
        for r_group in species.egg_groups.iter() {
          egg_groups.push(get_name_strict!(follow r_group, client, lang.to_string())?);
        }
        egg_groups.join(";")
      },
      // Held Items
      "held_items" => {
        let mut held_items = Vec::new();
        for r_item in mon.held_items.iter() {
          held_items.push(get_name_strict!(follow r_item.item, client, lang.to_string())?);
        }
        held_items.join(";")
      },
      // Growth Rate
      "growth_rate" => species.growth_rate.name.clone(),
      // Gender Rate
      "gender_rate" => species.gender_rate.to_string(),
      // Base Stats
      "base_stats" => {
        let mut base_stats: Vec<i64> = mon
          .stats
          .clone()
          .into_iter()
          .map(|stat| stat.base_stat)
          .collect();
        base_stats.push(base_stats.iter().sum());
        base_stats
          .into_iter()
          .map(|stat| stat.to_string())
          .collect::<Vec<String>>()
          .join(";")
      },
      // EV Yields
      "EV_yields" => {
        let mut ev_yields: Vec<i64> = mon
          .stats
          .clone()
          .into_iter()
          .map(|stat| stat.effort)
          .collect();
        ev_yields.push(ev_yields.iter().sum());
        ev_yields
          .into_iter()
          .map(|stat| stat.to_string())
          .collect::<Vec<String>>()
          .join(";")
      },
      // Hatch Counter
      "hatch_counter" => match species.hatch_counter {
        Some(hatch_counter) => hatch_counter.to_string(),
        None => String::from("None"),
      },
      //Base EXP
      "base_EXP" => match mon.base_experience {
        Some(base_experience) => base_experience.to_string(),
        None => String::from("None"),
      },
      // Capture Rate
      "capture_rate" => species.capture_rate.to_string(),
      // Base Happiness (TODO: typo in rustemon)
      "base_happiness" => match species.base_hapiness {
        Some(base_happiness) => base_happiness.to_string(),
        None => String::from("None"),
      },
      // Height/Weight
      "height" => mon.height.to_string(),
      "weight" => mon.weight.to_string(),
      // Stage and Evolution Type
      "stage" => {
        if let None = stage {
          (stage, evolution_type) = get_evo_stage_and_type(&client, &species, &mon).await?;
        }
        stage.unwrap().to_string()
      },
      "evolution_type" => {
        if let None = evolution_type {
          (stage, evolution_type) = get_evo_stage_and_type(&client, &species, &mon).await?;
        }
        evolution_type.unwrap().to_string()
      },
      // Starter
      "is_starter" => match species.id {
        1..10
        | 152..162
        | 252..262
        | 387..397
        | 495..505
        | 650..660
        | 722..732
        | 810..820
        | 906..916 => String::from("1"),
        _ if mon.name == "pikachu-starter" || mon.name == "eevee-starter" => String::from("1"),
        _ => String::from("0"),
      },
      // Fossil
      "is_fossil" => match species.id {
        138..143 | 345..349 | 408..412 | 564..568 | 696..700 | 880..884 => String::from("1"),
        _ => String::from("0"),
      },
      // Baby
      "is_baby" => (species.is_baby as i64).to_string(),
      // Mythical
      "is_mythical" => (species.is_mythical as i64).to_string(),
      // Legendary
      "is_legendary" => (species.is_legendary as i64).to_string(),
      // Ultra Beast
      "is_UB" => match species.id {
        793..800 | 803..807 => String::from("1"),
        _ => String::from("0"),
      },
      // Paradox
      "is_paradox" => match species.id {
        984..996 | 1005..1011 | 1020..1024 => String::from("1"),
        _ => String::from("0"),
      },
      // Category/Category ID (TODO: error check)
      "category" => {
        if let None = generation {
          generation = Some(match species.generation.follow(&client).await {
            Ok(obj) => obj,
            Err(_) => {
              return Err(cli::error(
                ErrorKind::InvalidValue,
                format!(
                  "API error: could not retrieve generation {}",
                  species.generation.name
                ),
              ));
            },
          });
        }
        if let None = category {
          category = Some(
            get_category(
              &client,
              &species,
              &mon,
              &generation.clone().unwrap(),
              &lang.to_string(),
            )
            .await?,
          );
        }
        category.clone().unwrap()
      },
      "category_id" => {
        if let None = category {
          category = Some(
            get_category(
              &client,
              &species,
              &mon,
              &generation.clone().unwrap(),
              &lang.to_string(),
            )
            .await?,
          );
        }
        let category_map = HashMap::from([
          ("Alola", "updated-alola"),
          ("Hisui", "hisui"),
          ("Paldea", "paldea"),
        ]);
        let category_id = match category.clone().unwrap().as_str() {
          "Galar" => {
            let mut result = 0;
            for (&dex, &start) in ["galar", "isle-of-armor", "crown-tundra"]
              .iter()
              .zip([0, 400, 611].iter())
            {
              for r_dex in species.pokedex_numbers.iter() {
                if r_dex.pokedex.name == dex {
                  result = start + r_dex.entry_number;
                }
              }
            }
            result
          },
          "Paldean Expeditions" => {
            let mut result = 0;
            for (&dex, &start) in ["kitakami", "blueberry"].iter().zip([0, 200].iter()) {
              for r_dex in species.pokedex_numbers.iter() {
                if r_dex.pokedex.name == dex {
                  result = start + r_dex.entry_number;
                }
              }
            }
            result
          },
          _ if category_map.contains_key(category.clone().unwrap().as_str()) => {
            let mut result = 0;
            let dex = category_map
              .get(category.clone().unwrap().as_str())
              .unwrap();
            for r_dex in species.pokedex_numbers.iter() {
              if r_dex.pokedex.name == *dex {
                result = r_dex.entry_number;
              }
            }
            result
          },
          _ => species.id,
        };
        if category_id == 0 {
          return Err(cli::error(
            ErrorKind::InvalidValue,
            format!("error: failed to retrieve appropriate pokedex ordering"),
          ));
        }
        category_id.to_string()
      },
      _ => {
        let valid = cli::VALID;
        return Err(cli::error(
          ErrorKind::InvalidValue,
          format!(
            "error: invalid key: {}\n\n{valid}tip:{valid:#} valid options are {}",
            key, DEFAULT_HEADER
          ),
        ));
      },
    });
  }

  Ok(output.join(","))
}

async fn get_evo_stage_and_type(
  client: &RustemonClient,
  species: &PokemonSpecies,
  mon: &Pokemon,
) -> Result<(Option<i64>, Option<i64>), clap::Error> {
  let mut stage = 0;
  let mut branched = 0;
  let mut branching = 0;
  match mon.name.as_str() {
    "pikachu-starter" => {
      stage = 0;
      branching = 0;
      branched = 0;
    },
    "pikachu-gmax" => {
      stage = 0;
      branching = 0;
      branched = 0;
    },
    "eevee-starter" => {
      stage = 0;
      branching = 0;
      branched = 0;
    },
    "eevee-gmax" => {
      stage = 0;
      branching = 0;
      branched = 0;
    },
    "meowth" => {
      stage = 1;
      branching = 0;
      branched = 0;
    },
    "meowth-alola" => {
      stage = 1;
      branching = 0;
      branched = 0;
    },
    "meowth-galar" => {
      stage = 1;
      branching = 0;
      branched = 0;
    },
    "meowth-gmax" => {
      stage = 1;
      branching = 0;
      branched = 0;
    },
    "persian" => {
      stage = 3;
      branching = 0;
      branched = 0;
    },
    "persian-alola" => {
      stage = 3;
      branching = 0;
      branched = 0;
    },
    "perrserker" => {
      stage = 3;
      branching = 0;
      branched = 0;
    },
    "farfetchd-galar" => {
      stage = 1;
      branching = 0;
      branched = 0;
    },
    "sirfetchd" => {
      stage = 3;
      branching = 0;
      branched = 0;
    },
    "mr-mime" => {
      stage = 3;
      branching = 0;
      branched = 0;
    },
    "mr-mime-galar" => {
      stage = 1;
      branching = 0;
      branched = 0;
    },
    "mr-rime" => {
      stage = 3;
      branching = 0;
      branched = 0;
    },
    "wooper" => {
      stage = 1;
      branching = 0;
      branched = 0;
    },
    "wooper-paldea" => {
      stage = 1;
      branching = 0;
      branched = 0;
    },
    "quagsire" => {
      stage = 3;
      branching = 0;
      branched = 0;
    },
    "clodsire" => {
      stage = 3;
      branching = 0;
      branched = 0;
    },
    "qwilfish" => {
      stage = 0;
      branching = 0;
      branched = 0;
    },
    "qwilfish-hisui" => {
      stage = 1;
      branching = 0;
      branched = 0;
    },
    "overqwil" => {
      stage = 3;
      branching = 0;
      branched = 0;
    },
    "sneasel" => {
      stage = 1;
      branching = 0;
      branched = 0;
    },
    "sneasel-hisui" => {
      stage = 1;
      branching = 0;
      branched = 0;
    },
    "weavile" => {
      stage = 3;
      branching = 0;
      branched = 0;
    },
    "sneasler" => {
      stage = 3;
      branching = 0;
      branched = 0;
    },
    "corsola" => {
      stage = 0;
      branching = 0;
      branched = 0;
    },
    "corsola-galar" => {
      stage = 1;
      branching = 0;
      branched = 0;
    },
    "cursola" => {
      stage = 3;
      branching = 0;
      branched = 0;
    },
    "zigzagoon" => {
      stage = 1;
      branching = 0;
      branched = 0;
    },
    "zigzagoon-galar" => {
      stage = 1;
      branching = 0;
      branched = 0;
    },
    "linoone" => {
      stage = 3;
      branching = 0;
      branched = 0;
    },
    "linoone-galar" => {
      stage = 2;
      branching = 0;
      branched = 0;
    },
    "obstagoon" => {
      stage = 3;
      branching = 0;
      branched = 0;
    },
    "yamask" => {
      stage = 1;
      branching = 0;
      branched = 0;
    },
    "yamask-galar" => {
      stage = 1;
      branching = 0;
      branched = 0;
    },
    "cofagrigus" => {
      stage = 3;
      branching = 0;
      branched = 0;
    },
    "runerigus" => {
      stage = 3;
      branching = 0;
      branched = 0;
    },
    "basculin-red-striped" => {
      stage = 0;
      branching = 0;
      branched = 0;
    },
    "basculin-blue-striped" => {
      stage = 0;
      branching = 0;
      branched = 0;
    },
    "basculin-white-striped" => {
      stage = 1;
      branching = 0;
      branched = 0;
    },
    _ => {
      if let Some(r_chain) = species.evolution_chain.clone() {
        let chain = match r_chain.follow(&client).await {
          Ok(obj) => obj.chain,
          Err(_) => {
            return Err(cli::error(
              ErrorKind::InvalidValue,
              format!(
                "API error: could not retrieve evolution_chain for {}",
                species.name
              ),
            ));
          },
        };
        if chain.evolves_to.len() > 0 {
          let mut found1 = false;
          for ch1 in chain.evolves_to.iter() {
            if stage != 0 {
              break;
            }
            if ch1.evolves_to.len() > 0 {
              let mut found2 = false;
              for ch2 in ch1.evolves_to.iter() {
                if ch2.species.name == species.name {
                  stage = 3;
                  branched = (ch1.evolves_to.len() > 1) as i64;
                  found2 = true;
                  break;
                }
              }
              if !found2 {
                if ch1.species.name == species.name {
                  stage = 2;
                  branching = (ch1.evolves_to.len() > 1) as i64;
                  branched = (chain.evolves_to.len() > 1) as i64;
                  found1 = true;
                  break;
                }
              }
            }
          }
          if !found1 {
            if chain.species.name == species.name {
              stage = 1;
              branching = (chain.evolves_to.len() > 1) as i64;
            }
          }
        }
      }
    },
  }
  Ok((Some(stage), Some(branching + 2 * branched)))
}

async fn get_category(
  client: &RustemonClient,
  species: &PokemonSpecies,
  mon: &Pokemon,
  generation: &rustemon::model::games::Generation,
  lang: &str,
) -> Result<String, clap::Error> {
  let mut category = get_name_strict!(follow generation.main_region, client, lang.to_string())?;
  if mon.name.starts_with("zygarde")
    && ["10", "power-construct", "complete"]
      .iter()
      .any(|name| mon.name.contains(*name))
  {
    category = String::from("Alola");
  } else if let 808..810 = species.id {
    category = String::from("Unknown");
  } else if (mon.name != "ursaluna-bloodmoon" && matches!(species.id, 899..906))
    || ["basculin-white-striped", "dialga-origin", "palkia-origin"]
      .iter()
      .any(|name| mon.name == *name)
  {
    category = String::from("Hisui");
  } else if mon.name == "ursaluna-bloodmoon" || matches!(species.id, 1009..1026) {
    category = String::from("Paldean Expeditions");
  } else {
    for &name in ["Alola", "Galar", "Hisui", "Paldea", "Totem"].iter() {
      if mon.name.contains(&format!("-{}", name.to_lowercase())) && !mon.name.ends_with("-cap") {
        category = String::from(name);
      }
    }
  }
  if ["gmax", "eternamax"]
    .iter()
    .any(|name| mon.name.ends_with(*name))
  {
    category = String::from("Gmax");
  } else if ["mega", "mega-x", "mega-y", "mega-z", "primal"]
    .iter()
    .any(|name| mon.name.ends_with(&format!("-{}", name)))
  {
    category = String::from("Mega");
  }
  Ok(category)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_latest_generation() {
    let client = RustemonClient::default();

    let mut all_species = rustemon::pokemon::pokemon_species::get_all_entries(&client)
      .await
      .unwrap();
    match all_species.pop() {
      Some(r_species) => {
        let species = r_species.follow(&client).await.unwrap();
        assert_eq!(species.id, LAST_SPECIES_ID);
      },
      None => panic!("Could not retrieve species resources"),
    };
  }
}
