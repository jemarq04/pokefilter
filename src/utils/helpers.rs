use crate::get_name_strict;
use crate::utils::cli;
use clap::error::ErrorKind;
use futures::future;
use rustemon::Follow;
use rustemon::client::RustemonClient;

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
