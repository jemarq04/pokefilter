use crate::get_name_strict;
use crate::utils::{cli, enums::LanguageId, helpers};
use clap::error::ErrorKind;
use rustemon::model::pokemon::{Pokemon, PokemonSpecies};
use rustemon::{Follow, client::RustemonClient};
use std::collections::HashMap;
use std::{fs::File, path::Path};

const LAST_SPECIES_ID: i64 = 1025;
// Remaining in previous format:
// Regional Dexes,Past Types,Past Abilities,(Past Stats),
const HEADER: &str = "identifier,national_dex,pokemon,species,generation,types,abilities,color,egg_groups,held_items,\
  growth_rate,gender_rate,base_stats,ev_yields,hatch_counter,base_exp,capture_rate,base_happiness,height,weight,\
  stage,is_branched,is_branching,is_starter,is_fossil,is_baby,is_mythical,is_legendary,is_ub,is_paradox,category,category_order";

pub async fn build(
  filepath: Option<std::path::PathBuf>,
  force: bool,
  range_opts: cli::BuildOpts,
  lang: LanguageId,
  cache_dir: Option<std::path::PathBuf>,
) -> Result<(), clap::Error> {
  // Create client for API requests
  let client = helpers::create_client(cache_dir);

  // If absent, set filepath to default
  let filepath = match filepath {
    Some(p) => p,
    None => helpers::get_default_cache_file().into(),
  };

  // Check if file already exists
  let filepath = Path::new(&filepath);
  if !force && filepath.exists() {
    let valid = cli::VALID;
    return Err(cli::error(
      ErrorKind::InvalidValue,
      format!(
        "file {} already exists\n\n{valid}tip:{valid:#} to overwrite it, run '{} build --force'",
        filepath.display(),
        cli::get_appname()
      ),
    ));
  }

  // Create output file
  let mut outfile = File::create(filepath).map_err(|why| {
    cli::error(
      ErrorKind::InvalidValue,
      format!("error creating file {}: {}", filepath.display(), why),
    )
  })?;

  // Determine the columns for the output CSV file
  helpers::fwriteln(&mut outfile, &filepath.display(), HEADER)?;

  // Determine appropriate range of pokemon to print
  let mut start = 1;
  let mut end = LAST_SPECIES_ID;
  if let Some(pokemon) = &range_opts.pokemon {
    let Ok(species) = rustemon::pokemon::pokemon_species::get_by_name(pokemon, &client).await
    else {
      return Err(cli::error(
        ErrorKind::InvalidValue,
        format!("API error: could not retrieve species {pokemon}"),
      ));
    };
    start = species.id;
    end = species.id;
  } else if let Some(pokerange) = &range_opts.pokerange {
    let Ok(species_start) =
      rustemon::pokemon::pokemon_species::get_by_name(&pokerange[0], &client).await
    else {
      return Err(cli::error(
        ErrorKind::InvalidValue,
        format!("API error: could not retrieve species {}", pokerange[0]),
      ));
    };
    let Ok(species_end) =
      rustemon::pokemon::pokemon_species::get_by_name(&pokerange[1], &client).await
    else {
      return Err(cli::error(
        ErrorKind::InvalidValue,
        format!("API error: could not retrieve species {}", pokerange[1]),
      ));
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

  // Loop through each ID and print the information for each variety of pokemon species
  for id in start..=end {
    let Ok(species) = rustemon::pokemon::pokemon_species::get_by_id(id, &client).await else {
      return Err(cli::error(
        ErrorKind::InvalidValue,
        format!("API error: could not retrieve species with ID {id}"),
      ));
    };

    for variety in &species.varieties {
      let Ok(mon) = variety.pokemon.follow(&client).await else {
        return Err(cli::error(
          ErrorKind::InvalidValue,
          format!(
            "API error: could not retrieve species variety: {}",
            variety.pokemon.name
          ),
        ));
      };
      println!("Building {}...", mon.name);
      helpers::fwriteln(
        &mut outfile,
        &filepath.display(),
        &build_pokemon(&client, &species, &mon, lang, &HEADER.split(',').collect()).await?,
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

  let mut generation = None;
  let mut stage = None;
  let mut is_branched = None;
  let mut is_branching = None;
  let mut category = None;
  for key in keys {
    output.push(match *key {
      // Name/ID
      "identifier" => mon.name.clone(),
      "national_dex" => species.id.to_string(),
      "pokemon" => helpers::get_pokemon_name(client, mon, &lang.to_string()).await?,
      "species" => get_name_strict!(species, client, lang.to_string())?,
      // Generation
      "generation" => {
        if generation.is_none() {
          generation = Some(species.generation.follow(client).await.map_err(|_| {
            cli::error(
              ErrorKind::InvalidValue,
              format!(
                "API error: could not retrieve generation {}",
                species.generation.name
              ),
            )
          })?);
        }
        generation.clone().unwrap().id.to_string()
      },
      // Types
      "types" => {
        let mut types = Vec::new();
        for r_type in &mon.types {
          types.push(get_name_strict!(follow r_type.type_, client, lang.to_string())?);
        }
        types.join(";")
      },
      // Abilities
      "abilities" => {
        let mut abilities = Vec::new();
        for r_ability in &mon.abilities {
          let mut ability: String =
            get_name_strict!(follow r_ability.ability.clone().unwrap(), client, lang.to_string())?;
          if r_ability.is_hidden {
            ability.push('*');
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
        for r_group in &species.egg_groups {
          egg_groups.push(get_name_strict!(follow r_group, client, lang.to_string())?);
        }
        egg_groups.join(";")
      },
      // Held Items
      "held_items" => {
        let mut held_items = Vec::new();
        for r_item in &mon.held_items {
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
      "ev_yields" => {
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
        None => String::new(),
      },
      //Base EXP
      "base_exp" => match mon.base_experience {
        Some(base_experience) => base_experience.to_string(),
        None => String::new(),
      },
      // Capture Rate
      "capture_rate" => species.capture_rate.to_string(),
      // Base Happiness
      "base_happiness" => match species.base_happiness {
        Some(base_happiness) => base_happiness.to_string(),
        None => String::new(),
      },
      // Height/Weight
      "height" => mon.height.to_string(),
      "weight" => mon.weight.to_string(),
      // Stage and Evolution Type
      "stage" => {
        if stage.is_none() {
          (stage, is_branched, is_branching) = get_evo_stage_and_type(client, species, mon).await?;
        }
        stage.unwrap().to_string()
      },
      "is_branched" => {
        if is_branched.is_none() {
          (stage, is_branched, is_branching) = get_evo_stage_and_type(client, species, mon).await?;
        }
        is_branched.unwrap().to_string()
      },
      "is_branching" => {
        if is_branching.is_none() {
          (stage, is_branched, is_branching) = get_evo_stage_and_type(client, species, mon).await?;
        }
        is_branching.unwrap().to_string()
      },
      // Starter
      "is_starter" => (matches!(species.id, 1..10 | 152..162 | 252..262 | 387..397 | 495..505)
        || matches!(species.id, 650..660 | 722..732 | 810..820 | 906..916)
        || mon.name == "pikachu-starter"
        || mon.name == "eevee-starter")
        .to_string(),
      // Fossil
      "is_fossil" => {
        matches!(species.id, 138..143 | 345..349 | 408..412 | 564..568 | 696..700 | 880..884)
          .to_string()
      },
      // Baby
      "is_baby" => species.is_baby.to_string(),
      // Mythical
      "is_mythical" => species.is_mythical.to_string(),
      // Legendary
      "is_legendary" => species.is_legendary.to_string(),
      // Ultra Beast
      "is_ub" => matches!(species.id, 793..800 | 803..807).to_string(),
      // Paradox
      "is_paradox" => matches!(species.id, 984..996 | 1005..1011 | 1020..1024).to_string(),
      // Category/Category Ordering
      "category" => {
        if generation.is_none() {
          generation = Some(match species.generation.follow(client).await {
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
        if category.is_none() {
          category = Some(
            get_category(
              client,
              species,
              mon,
              &generation.clone().unwrap(),
              &lang.to_string(),
            )
            .await?,
          );
        }
        category.clone().unwrap()
      },
      "category_order" => {
        if category.is_none() {
          category = Some(
            get_category(
              client,
              species,
              mon,
              &generation.clone().unwrap(),
              &lang.to_string(),
            )
            .await?,
          );
        }
        // Handle forms for single-dex regions
        let category_map = HashMap::from([("Alola", "updated-alola"), ("Hisui", "hisui")]);
        let category_order = match category.clone().unwrap().as_str() {
          // Sorted by all regional dexes for forms
          "Galar" => {
            let mut result = 0;
            for (&dex, &start) in ["galar", "isle-of-armor", "crown-tundra"]
              .iter()
              .zip([0, 400, 611].iter())
            {
              if result > 0 {
                break;
              }
              for r_dex in &species.pokedex_numbers {
                if r_dex.pokedex.name == dex {
                  result = start + r_dex.entry_number;
                  break;
                }
              }
            }
            result
          },
          // Sorted by all regional dexes for forms
          "Paldea" => {
            let mut result = 0;
            for (&dex, &start) in ["paldea", "kitakami", "blueberry"]
              .iter()
              .zip([0, 400, 600].iter())
            {
              if result > 0 {
                break;
              }
              for r_dex in &species.pokedex_numbers {
                if r_dex.pokedex.name == dex {
                  result = start + r_dex.entry_number;
                  break;
                }
              }
            }
            result
          },
          // Sorted by the single regional dex for forms
          _ if category_map.contains_key(category.clone().unwrap().as_str()) => {
            let mut result = 0;
            let dex = category_map
              .get(category.clone().unwrap().as_str())
              .unwrap();
            for r_dex in &species.pokedex_numbers {
              if r_dex.pokedex.name == *dex {
                result = r_dex.entry_number;
              }
            }
            result
          },
          // Otherwise, use national dex number
          _ => species.id,
        };
        if category_order == 0 {
          return Err(cli::error(
            ErrorKind::InvalidValue,
            "failed to retrieve appropriate pokedex ordering".to_string(),
          ));
        }
        category_order.to_string()
      },
      _ => {
        let valid = cli::VALID;
        return Err(cli::error(
          ErrorKind::InvalidValue,
          format!("invalid key: {key}\n\n{valid}tip:{valid:#} valid options are {HEADER}"),
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
) -> Result<(Option<i64>, Option<bool>, Option<bool>), clap::Error> {
  // Stage
  //  0: Single-stage pokemon
  //  1: First pokemon in a multi-stage line
  //  2: Middle pokemon in a three-stage line
  //  3: Last pokemon in a multi-stage line
  let mut stage = 0;
  let mut branched = false;
  let mut branching = false;
  match mon.name.as_str() {
    // Handle manual exceptions (limitation of API)
    "pikachu-starter" => {
      stage = 0;
      branching = false;
      branched = false;
    },
    "pikachu-gmax" => {
      stage = 0;
      branching = false;
      branched = false;
    },
    "eevee-starter" => {
      stage = 0;
      branching = false;
      branched = false;
    },
    "eevee-gmax" => {
      stage = 0;
      branching = false;
      branched = false;
    },
    "meowth" => {
      stage = 1;
      branching = false;
      branched = false;
    },
    "meowth-alola" => {
      stage = 1;
      branching = false;
      branched = false;
    },
    "meowth-galar" => {
      stage = 1;
      branching = false;
      branched = false;
    },
    "meowth-gmax" => {
      stage = 1;
      branching = false;
      branched = false;
    },
    "persian" => {
      stage = 3;
      branching = false;
      branched = false;
    },
    "persian-alola" => {
      stage = 3;
      branching = false;
      branched = false;
    },
    "perrserker" => {
      stage = 3;
      branching = false;
      branched = false;
    },
    "farfetchd-galar" => {
      stage = 1;
      branching = false;
      branched = false;
    },
    "sirfetchd" => {
      stage = 3;
      branching = false;
      branched = false;
    },
    "mr-mime" => {
      stage = 3;
      branching = false;
      branched = false;
    },
    "mr-mime-galar" => {
      stage = 2;
      branching = false;
      branched = false;
    },
    "mr-rime" => {
      stage = 3;
      branching = false;
      branched = false;
    },
    "wooper" => {
      stage = 1;
      branching = false;
      branched = false;
    },
    "wooper-paldea" => {
      stage = 1;
      branching = false;
      branched = false;
    },
    "quagsire" => {
      stage = 3;
      branching = false;
      branched = false;
    },
    "clodsire" => {
      stage = 3;
      branching = false;
      branched = false;
    },
    "qwilfish" => {
      stage = 0;
      branching = false;
      branched = false;
    },
    "qwilfish-hisui" => {
      stage = 1;
      branching = false;
      branched = false;
    },
    "overqwil" => {
      stage = 3;
      branching = false;
      branched = false;
    },
    "sneasel" => {
      stage = 1;
      branching = false;
      branched = false;
    },
    "sneasel-hisui" => {
      stage = 1;
      branching = false;
      branched = false;
    },
    "weavile" => {
      stage = 3;
      branching = false;
      branched = false;
    },
    "sneasler" => {
      stage = 3;
      branching = false;
      branched = false;
    },
    "corsola" => {
      stage = 0;
      branching = false;
      branched = false;
    },
    "corsola-galar" => {
      stage = 1;
      branching = false;
      branched = false;
    },
    "cursola" => {
      stage = 3;
      branching = false;
      branched = false;
    },
    "zigzagoon" => {
      stage = 1;
      branching = false;
      branched = false;
    },
    "zigzagoon-galar" => {
      stage = 1;
      branching = false;
      branched = false;
    },
    "linoone" => {
      stage = 3;
      branching = false;
      branched = false;
    },
    "linoone-galar" => {
      stage = 2;
      branching = false;
      branched = false;
    },
    "obstagoon" => {
      stage = 3;
      branching = false;
      branched = false;
    },
    "yamask" => {
      stage = 1;
      branching = false;
      branched = false;
    },
    "yamask-galar" => {
      stage = 1;
      branching = false;
      branched = false;
    },
    "cofagrigus" => {
      stage = 3;
      branching = false;
      branched = false;
    },
    "runerigus" => {
      stage = 3;
      branching = false;
      branched = false;
    },
    "basculin-red-striped" => {
      stage = 0;
      branching = false;
      branched = false;
    },
    "basculin-blue-striped" => {
      stage = 0;
      branching = false;
      branched = false;
    },
    "basculin-white-striped" => {
      stage = 1;
      branching = false;
      branched = false;
    },
    // Determine evolution stage and if it branches
    _ => {
      if let Some(r_chain) = species.evolution_chain.clone() {
        let chain = match r_chain.follow(client).await {
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
        if !chain.evolves_to.is_empty() {
          let mut found1 = false;
          for ch1 in &chain.evolves_to {
            if stage != 0 {
              break;
            }
            if !ch1.evolves_to.is_empty() {
              let mut found2 = false;
              for ch2 in &ch1.evolves_to {
                if ch2.species.name == species.name {
                  stage = 3;
                  branched = ch1.evolves_to.len() > 1;
                  found2 = true;
                  break;
                }
              }
              if !found2 && ch1.species.name == species.name {
                stage = 2;
                branching = ch1.evolves_to.len() > 1;
                branched = chain.evolves_to.len() > 1;
                found1 = true;
                break;
              }
            }
          }
          if !found1 && chain.species.name == species.name {
            stage = 1;
            branching = chain.evolves_to.len() > 1;
          }
        }
      }
    },
  }
  Ok((Some(stage), Some(branched), Some(branching)))
}

async fn get_category(
  client: &RustemonClient,
  species: &PokemonSpecies,
  mon: &Pokemon,
  generation: &rustemon::model::games::Generation,
  lang: &str,
) -> Result<String, clap::Error> {
  let mut category = get_name_strict!(follow generation.main_region, client, lang)?;
  if (mon.name.starts_with("zygarde")
    && ["10", "power-construct", "complete"]
      .iter()
      .any(|name| mon.name.contains(*name)))
    || mon.name.contains("-totem")
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
    category = String::from("Paldea");
  } else {
    for &name in &["Alola", "Galar", "Hisui", "Paldea"] {
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
    .any(|name| mon.name.ends_with(&format!("-{name}")))
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
    }
  }
}
