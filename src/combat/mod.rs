//! Combat system.

pub mod damage;

pub use damage::{
    calculate_damage, entity_at, melee_attack, ranged_attack, remove_dead, AttackResult,
    DamageInstance, DamageType, BASE_MELEE_DAMAGE, BASE_RANGED_DAMAGE, MELEE_AP_COST,
    RANGED_OPTIMAL_DISTANCE,
};
