use crate::impl_Display;
use clap::ValueEnum;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, ValueEnum)]
pub enum LanguageId {
  #[value(alias = "ja-Hrkt")]
  JaHrkt,
  Roomaji,
  Ko,
  #[value(alias = "zh-Hant")]
  ZhHant,
  Fr,
  De,
  Es,
  It,
  En,
  Cs,
  Ja,
  #[value(alias = "zh-Hans")]
  ZhHans,
  #[value(alias = "pt-BR")]
  PtBR,
}
impl_Display!(LanguageId);
