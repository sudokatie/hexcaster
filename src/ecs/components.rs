//! ECS components for game entities.

use crate::hex::Hex;
use crate::magic::Rune;

/// Position in the hex grid.
#[derive(Debug, Clone, Copy)]
pub struct Position(pub Hex);

/// Health points.
#[derive(Debug, Clone)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

impl Health {
    pub fn new(max: i32) -> Self {
        Self { current: max, max }
    }

    pub fn take_damage(&mut self, amount: i32) {
        self.current = (self.current - amount).max(0);
    }

    pub fn heal(&mut self, amount: i32) {
        self.current = (self.current + amount).min(self.max);
    }

    pub fn is_dead(&self) -> bool {
        self.current <= 0
    }
}

/// Action points for movement and spells.
#[derive(Debug, Clone)]
pub struct ActionPoints {
    pub current: i32,
    pub max: i32,
}

impl ActionPoints {
    pub fn new(max: i32) -> Self {
        Self { current: max, max }
    }

    pub fn spend(&mut self, amount: i32) -> bool {
        if self.current >= amount {
            self.current -= amount;
            true
        } else {
            false
        }
    }

    pub fn restore(&mut self) {
        self.current = self.max;
    }
}

/// Marker component for the player.
#[derive(Debug, Clone, Copy)]
pub struct Player;

/// AI type for enemies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AIType {
    Melee,  // Approaches and attacks
    Ranged, // Keeps distance, uses spells
    Patrol, // Wanders until sees player
}

/// Enemy component with AI behavior.
#[derive(Debug, Clone)]
pub struct Enemy {
    pub ai_type: AIType,
    pub aggro_range: i32,
}

/// Rune inventory for spell crafting.
#[derive(Debug, Clone, Default)]
pub struct Inventory {
    pub runes: Vec<Rune>,
}

/// A rune on the floor that can be picked up.
#[derive(Debug, Clone)]
pub struct RunePickup {
    pub rune: Rune,
}

/// Active status effects.
#[derive(Debug, Clone)]
pub struct StatusEffect {
    pub kind: StatusKind,
    pub duration: i32,
    pub intensity: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatusKind {
    Burning,
    Frozen,
    Shocked,
    Poisoned,
}

#[derive(Debug, Clone, Default)]
pub struct Status {
    pub effects: Vec<StatusEffect>,
}

impl Status {
    pub fn add(&mut self, effect: StatusEffect) {
        self.effects.push(effect);
    }

    pub fn tick(&mut self) {
        for effect in &mut self.effects {
            effect.duration -= 1;
        }
        self.effects.retain(|e| e.duration > 0);
    }
}

/// Display info for rendering.
#[derive(Debug, Clone)]
pub struct Display {
    pub glyph: char,
    pub color: (u8, u8, u8),
}
