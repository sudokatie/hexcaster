//! Damage calculation and application.

use hecs::{Entity, World};

use crate::ecs::components::{Health, Position};
use crate::hex::Hex;

/// Types of damage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageType {
    Physical,
    Fire,
    Ice,
    Lightning,
    Void,
}

/// A single damage instance.
#[derive(Debug, Clone)]
pub struct DamageInstance {
    pub amount: i32,
    pub damage_type: DamageType,
    pub source: Option<Entity>,
}

impl DamageInstance {
    pub fn new(amount: i32, damage_type: DamageType) -> Self {
        Self {
            amount,
            damage_type,
            source: None,
        }
    }

    pub fn with_source(mut self, source: Entity) -> Self {
        self.source = Some(source);
        self
    }
}

/// Calculate damage after defenses.
pub fn calculate_damage(base: i32, resistance: i32) -> i32 {
    (base - resistance).max(0)
}

/// Result of a melee attack.
#[derive(Debug)]
pub enum AttackResult {
    Hit { damage: i32, killed: bool },
    Miss,
    NoTarget,
}

/// Base melee damage for the player.
pub const BASE_MELEE_DAMAGE: i32 = 15;

/// Cost in AP for a melee attack.
pub const MELEE_AP_COST: i32 = 1;

/// Find an entity at the given position (excluding the attacker).
pub fn entity_at(world: &World, pos: Hex, exclude: Entity) -> Option<Entity> {
    for (entity, position) in world.query::<&Position>().iter() {
        if entity != exclude && position.0 == pos {
            return Some(entity);
        }
    }
    None
}

/// Perform a melee attack against an adjacent target.
pub fn melee_attack(
    world: &mut World,
    _attacker: Entity,
    target: Entity,
    base_damage: i32,
) -> AttackResult {
    // Apply damage to target
    let (killed, actual_damage) = {
        let mut health = match world.get::<&mut Health>(target) {
            Ok(h) => h,
            Err(_) => return AttackResult::NoTarget,
        };

        let damage = calculate_damage(base_damage, 0); // No resistance for now
        health.take_damage(damage);
        (health.is_dead(), damage)
    };

    AttackResult::Hit {
        damage: actual_damage,
        killed,
    }
}

/// Remove all dead entities from the world.
pub fn remove_dead(world: &mut World) -> Vec<Entity> {
    let dead: Vec<Entity> = world
        .query::<&Health>()
        .iter()
        .filter(|(_, h)| h.is_dead())
        .map(|(e, _)| e)
        .collect();

    for entity in &dead {
        let _ = world.despawn(*entity);
    }

    dead
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_damage_instance() {
        let dmg = DamageInstance::new(10, DamageType::Fire);
        assert_eq!(dmg.amount, 10);
        assert_eq!(dmg.damage_type, DamageType::Fire);
    }

    #[test]
    fn test_calculate_damage() {
        assert_eq!(calculate_damage(10, 3), 7);
        assert_eq!(calculate_damage(10, 15), 0); // Can't go negative
    }

    #[test]
    fn test_melee_attack_deals_damage() {
        let mut world = World::new();
        let attacker = world.spawn((Position(Hex::origin()), Health::new(100)));
        let target = world.spawn((Position(Hex::origin().neighbor(0)), Health::new(50)));

        let result = melee_attack(&mut world, attacker, target, 20);

        match result {
            AttackResult::Hit { damage, killed } => {
                assert_eq!(damage, 20);
                assert!(!killed);
                let health = world.get::<&Health>(target).unwrap();
                assert_eq!(health.current, 30);
            }
            _ => panic!("Expected hit"),
        }
    }

    #[test]
    fn test_melee_attack_kills_target() {
        let mut world = World::new();
        let attacker = world.spawn((Position(Hex::origin()), Health::new(100)));
        let target = world.spawn((Position(Hex::origin().neighbor(0)), Health::new(15)));

        let result = melee_attack(&mut world, attacker, target, 20);

        match result {
            AttackResult::Hit { damage: _, killed } => {
                assert!(killed);
            }
            _ => panic!("Expected hit"),
        }
    }

    #[test]
    fn test_remove_dead() {
        let mut world = World::new();
        let alive = world.spawn((Health::new(50),));
        let dead = world.spawn((Health {
            current: 0,
            max: 50,
        },));

        let removed = remove_dead(&mut world);

        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0], dead);
        assert!(world.get::<&Health>(alive).is_ok());
        assert!(world.get::<&Health>(dead).is_err()); // Despawned
    }

    #[test]
    fn test_entity_at() {
        let mut world = World::new();
        let pos = Hex::new(1, 0, -1);
        let entity = world.spawn((Position(pos),));
        let other = world.spawn((Position(Hex::origin()),));

        assert_eq!(entity_at(&world, pos, other), Some(entity));
        assert_eq!(entity_at(&world, Hex::origin(), entity), Some(other));
        assert_eq!(entity_at(&world, Hex::new(5, 0, -5), entity), None);
    }
}
