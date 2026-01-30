use crate::get_name_strict;
use crate::utils::{cli, enums::LanguageId, helpers};
use clap::error::ErrorKind;
use rustemon::model::pokemon::{Pokemon, PokemonSpecies};
use rustemon::{Follow, client::RustemonClient};
use std::collections::HashMap;

//TODO: Build command will re-create CSV file containing all filter-able information for pokemon
//      Add customizability for this feature later on, for now build a default spread (names, stats, etc.)

pub async fn build(
  client: &RustemonClient,
  force: bool,
  lang: LanguageId,
) -> Result<Vec<String>, clap::Error> {
  // Check for existing file first...
  //
  // If file not found, continue to building file below

  // Remaining in previous format:
  // Regional Dexes,Past Types,Past Abilities,(Past Stats),
  const HEADER: &str = "pokemon_id,national_dex,pokemon,species,generation,types,abilities,color,egg_groups,held_items,\
    growth_rate,gender_rate,base_stats,EV_yields,hatch_counter,base_EXP,capture_rate,base_happiness,height,weight,\
    stage,evolution_type,is_starter,is_fossil,is_baby,is_mythical,is_legendary,is_UB,is_paradox,category,category_id,";

  let mut result: Vec<String> = vec![String::from(HEADER)];

  let r_all_species = match rustemon::pokemon::pokemon_species::get_all_entries(&client).await {
    Ok(list) => list,
    Err(_) => {
      return Err(cli::error(
        ErrorKind::InvalidValue,
        String::from("API error: could not retrieve all pokemon species"),
      ));
    },
  };

  const START: i64 = 1;
  const END: i64 = 30;
  for r_species in r_all_species.iter() {
    let species = match r_species.follow(&client).await {
      Ok(obj) => obj,
      Err(_) => {
        return Err(cli::error(
          ErrorKind::InvalidValue,
          format!("API error: could not retrieve species {}", r_species.name),
        ));
      },
    };
    match species.id {
      ..START => continue,
      END.. => break,
      _ => {},
    }

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
      result.push(build_pokemon(&client, &species, &mon, lang).await?);
    }
  }

  println!("{:?}", result);
  if let Some(val) = result.first() {
    println!("{}", val);
  }
  if let Some(val) = result.last() {
    println!("{}", val);
  }
  Ok(result)
}

pub async fn build_pokemon(
  client: &RustemonClient,
  species: &PokemonSpecies,
  mon: &Pokemon,
  lang: LanguageId,
) -> Result<String, clap::Error> {
  let mut output = String::new();

  // Name/ID
  output.push_str(&format!(
    "{},{},{},{},",
    mon.name,
    species.id,
    helpers::get_pokemon_name(&client, &mon, &lang.to_string()).await?,
    get_name_strict!(species, client, lang.to_string())?,
  ));

  // Generation
  let generation = match species.generation.follow(&client).await {
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
  };
  output.push_str(&format!("{},", generation.id));

  // Types
  let mut types = Vec::new();
  for r_type in mon.types.iter() {
    types.push(get_name_strict!(follow r_type.type_, client, lang.to_string())?);
  }
  output.push_str(&format!("{},", types.join(";")));

  // Abilities
  let mut abilities = Vec::new();
  for r_ability in mon.abilities.iter() {
    let mut ability: String = get_name_strict!(follow r_ability.ability, client, lang.to_string())?;
    if r_ability.is_hidden {
      ability.push_str("*");
    }
    abilities.push(ability);
  }
  output.push_str(&format!("{},", abilities.join(";")));

  // Color
  output.push_str(&format!(
    "{},",
    get_name_strict!(follow species.color, client, lang.to_string())?
  ));

  // Egg Groups
  let mut egg_groups = Vec::new();
  for r_group in species.egg_groups.iter() {
    egg_groups.push(get_name_strict!(follow r_group, client, lang.to_string())?);
  }
  output.push_str(&format!("{},", egg_groups.join(";")));

  // Held Items
  let mut held_items = Vec::new();
  for r_item in mon.held_items.iter() {
    held_items.push(get_name_strict!(follow r_item.item, client, lang.to_string())?);
  }
  output.push_str(&format!("{},", held_items.join(";")));

  // Growth Rate
  output.push_str(&format!("{},", species.growth_rate.name));

  // Gender Rate
  output.push_str(&format!("{},", species.gender_rate));

  // Base Stats
  let mut base_stats: Vec<i64> = mon
    .stats
    .clone()
    .into_iter()
    .map(|stat| stat.base_stat)
    .collect();
  base_stats.push(base_stats.iter().sum());
  output.push_str(&format!(
    "{},",
    base_stats
      .into_iter()
      .map(|stat| stat.to_string())
      .collect::<Vec<String>>()
      .join(";")
  ));

  // EV Yields
  let mut ev_yields: Vec<i64> = mon
    .stats
    .clone()
    .into_iter()
    .map(|stat| stat.effort)
    .collect();
  ev_yields.push(ev_yields.iter().sum());
  output.push_str(&format!(
    "{},",
    ev_yields
      .into_iter()
      .map(|stat| stat.to_string())
      .collect::<Vec<String>>()
      .join(";")
  ));

  // Hatch Counter
  output.push_str(&format!(
    "{},",
    match species.hatch_counter {
      Some(hatch_counter) => hatch_counter.to_string(),
      None => String::from("None"),
    }
  ));

  // Base EXP
  output.push_str(&format!(
    "{},",
    match mon.base_experience {
      Some(base_experience) => base_experience.to_string(),
      None => String::from("None"),
    }
  ));

  // Capture Rate
  output.push_str(&format!("{},", species.capture_rate));

  // Base Happiness
  //TODO: typo in rustemon?
  output.push_str(&format!(
    "{},",
    match species.base_hapiness {
      Some(base_hapiness) => base_hapiness.to_string(),
      None => String::from("None"),
    }
  ));

  // Height
  output.push_str(&format!("{},", mon.height));

  // Weight
  output.push_str(&format!("{},", mon.weight));

  // Stage and Evolution Type
  // TODO: handle exceptions for these values
  let mut stage: i64 = 0;
  let (mut branched, mut branching) = (0, 0);
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
  output.push_str(&format!("{},{},", stage, branching + 2 * branched));

  // Starter
  output.push_str(&format!(
    "{},",
    match species.id {
      1..10
      | 152..162
      | 252..262
      | 387..397
      | 495..505
      | 650..660
      | 722..732
      | 810..820
      | 906..916 => 1,
      _ if mon.name == "pikachu-starter" || mon.name == "eevee-starter" => 1,
      _ => 0,
    }
  ));

  // Fossil
  output.push_str(&format!(
    "{},",
    match species.id {
      138..143 | 345..349 | 408..412 | 564..568 | 696..700 | 880..884 => 1,
      _ => 0,
    }
  ));

  // Baby
  output.push_str(&format!("{},", species.is_baby as i64));

  // Mythical
  output.push_str(&format!("{},", species.is_mythical as i64));

  // Legendary
  output.push_str(&format!("{},", species.is_legendary as i64));

  // Ultra Beast
  output.push_str(&format!(
    "{},",
    match species.id {
      793..800 | 803..807 => 1,
      _ => 0,
    }
  ));

  // Paradox
  output.push_str(&format!(
    "{},",
    match species.id {
      984..996 | 1005..1011 | 1020..1024 => 1,
      _ => 0,
    }
  ));

  // Category
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
  output.push_str(&format!("{},", category));

  // Category ID
  let category_map = HashMap::from([
    ("Alola", "updated-alola"),
    ("Hisui", "hisui"),
    ("Paldea", "paldea"),
  ]);
  let category_id = match category.as_str() {
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
    _ if category_map.contains_key(category.as_str()) => {
      let mut result = 0;
      let dex = category_map.get(category.as_str()).unwrap();
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
  output.push_str(&format!("{},", category_id));

  Ok(output)
}

#[cfg(test)]
mod tests {
  use super::*;
}
