use crate::impl_Display;
use clap::ValueEnum;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, ValueEnum)]
pub enum LanguageId {
  JaHrkt,
  JaRoma,
  Ko,
  ZhHant,
  Fr,
  De,
  Es,
  It,
  En,
  Cs,
  Ja,
  ZhHans,
  PtBr,
}
impl_Display!(LanguageId);
