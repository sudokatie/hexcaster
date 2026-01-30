//! Magic system: runes and spell crafting.

pub mod rune;
pub mod spell;

pub use rune::{Element, Modifier, Rune, RuneKind, Shape};
pub use spell::{craft_spell, get_targets, Spell};
