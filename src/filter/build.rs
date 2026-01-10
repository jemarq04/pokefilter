use crate::utils::enums::LanguageId;
use rustemon::client::RustemonClient;

pub async fn build(
  client: &RustemonClient,
  force: bool,
  lang: LanguageId,
) -> Result<Vec<String>, clap::Error> {
  Ok(vec![])
}

pub async fn build_pokemon(
  client: &RustemonClient,
  lang: LanguageId,
) -> Result<String, clap::Error> {
  Ok(String::new())
}

#[cfg(test)]
mod tests {
  use super::*;
}
