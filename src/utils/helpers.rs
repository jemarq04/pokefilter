use crate::get_name;
use futures::future;
use rustemon::Follow;
use rustemon::client::RustemonClient;
use rustemon::pokemon::*;

pub async fn get_pokemon_name(
  client: &RustemonClient,
  pokemon: &rustemon::model::pokemon::Pokemon,
  lang: &str,
) -> String {
  let forms =
    match future::try_join_all(pokemon.forms.iter().map(async |f| f.follow(&client).await)).await {
      Ok(x) => x,
      Err(_) => return pokemon.name.clone(),
    };

  for form in forms.into_iter() {
    if !form.is_default || form.names.len() == 0 {
      continue;
    }
    for n in form.names.iter() {
      if let Ok(item) = n.language.follow(&client).await
        && item.name == lang
      {
        return n.name.clone();
      }
    }
    break;
  }

  get_name!(follow pokemon.species, client, lang)
}

pub async fn get_pokemon_from_chain(
  client: &RustemonClient,
  pokemon: &str,
  recursive: bool,
) -> Result<Vec<rustemon::model::pokemon::Pokemon>, ()> {
  let mut result = Vec::new();
  let pokemon = match pokemon::get_by_name(pokemon, &client).await {
    Ok(x) => x,
    Err(_) => return Err(()),
  };

  if recursive {
    let species = match pokemon.species.follow(&client).await {
      Ok(x) => x,
      Err(_) => return Err(()),
    };
    if let Some(chain) = species.evolution_chain {
      let chain = match chain.follow(&client).await {
        Ok(x) => x.chain,
        Err(_) => return Err(()),
      };
      if let Ok(x) = pokemon_species::get_by_name(&chain.species.name, &client).await {
        if let Ok(y) = future::try_join_all(
          x.varieties
            .iter()
            .map(async |v| v.pokemon.follow(&client).await),
        )
        .await
        {
          y.into_iter().for_each(|mon| result.push(mon));
        }
      }
      for evo1 in chain.evolves_to.iter() {
        if let Ok(x) = pokemon_species::get_by_name(&evo1.species.name, &client).await {
          if let Ok(y) = future::try_join_all(
            x.varieties
              .iter()
              .map(async |v| v.pokemon.follow(&client).await),
          )
          .await
          {
            y.into_iter().for_each(|mon| result.push(mon));
          }
        }
        for evo2 in evo1.evolves_to.iter() {
          if let Ok(x) = pokemon_species::get_by_name(&evo2.species.name, &client).await {
            if let Ok(y) = future::try_join_all(
              x.varieties
                .iter()
                .map(async |v| v.pokemon.follow(&client).await),
            )
            .await
            {
              y.into_iter().for_each(|mon| result.push(mon));
            }
          }
        }
      }
    }
  } else {
    result.push(pokemon);
  }

  Ok(result)
}

pub async fn get_evolution_name(
  client: &RustemonClient,
  species: &rustemon::model::resource::NamedApiResource<rustemon::model::pokemon::PokemonSpecies>,
  lang: &str,
  fast: bool,
) -> String {
  if !fast {
    get_name!(follow species, client, lang)
  } else {
    species.name.clone()
  }
}
