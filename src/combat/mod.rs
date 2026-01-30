//! Combat system.

pub mod damage;

pub use damage::{
    calculate_damage, entity_at, melee_attack, remove_dead, AttackResult, DamageInstance,
    DamageType, BASE_MELEE_DAMAGE, MELEE_AP_COST,
};
