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

/// Boss types with unique mechanics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BossType {
    /// Fire Elemental - AOE fire attacks, enrages at low health
    FireLord,
    /// Ice Elemental - Slows player, summons ice walls
    FrostQueen,
    /// Shadow creature - teleports, creates duplicates
    VoidWraith,
}

impl BossType {
    /// Get the boss name for display.
    pub fn name(&self) -> &'static str {
        match self {
            BossType::FireLord => "Fire Lord",
            BossType::FrostQueen => "Frost Queen",
            BossType::VoidWraith => "Void Wraith",
        }
    }
    
    /// Get the boss glyph.
    pub fn glyph(&self) -> char {
        match self {
            BossType::FireLord => 'F',
            BossType::FrostQueen => 'Q',
            BossType::VoidWraith => 'V',
        }
    }
    
    /// Get the boss color.
    pub fn color(&self) -> (u8, u8, u8) {
        match self {
            BossType::FireLord => (255, 100, 50),     // Orange-red
            BossType::FrostQueen => (150, 200, 255),  // Ice blue
            BossType::VoidWraith => (180, 50, 255),   // Purple
        }
    }
    
    /// Get base health for this boss (scales with floor).
    pub fn base_health(&self) -> i32 {
        match self {
            BossType::FireLord => 100,
            BossType::FrostQueen => 80,
            BossType::VoidWraith => 60,
        }
    }
    
    /// Get the boss for a given floor number.
    pub fn for_floor(floor: i32) -> Self {
        match floor % 3 {
            1 => BossType::FireLord,
            2 => BossType::FrostQueen,
            _ => BossType::VoidWraith,
        }
    }
}

/// Boss component for boss enemies.
#[derive(Debug, Clone)]
pub struct Boss {
    pub boss_type: BossType,
    /// Current phase (bosses may have multiple phases).
    pub phase: u8,
    /// Whether the boss is enraged (low health).
    pub enraged: bool,
}

impl Boss {
    pub fn new(boss_type: BossType) -> Self {
        Self {
            boss_type,
            phase: 1,
            enraged: false,
        }
    }
    
    /// Check if boss should enrage based on health.
    pub fn check_enrage(&mut self, health_percent: f32) {
        if health_percent <= 0.3 && !self.enraged {
            self.enraged = true;
        }
    }
}

/// Loot drop quality.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LootQuality {
    Common,
    Rare,
    Epic,
}

/// Special loot from bosses.
#[derive(Debug, Clone)]
pub struct BossLoot {
    pub quality: LootQuality,
    /// Number of runes dropped.
    pub rune_count: usize,
}

impl BossLoot {
    pub fn from_boss(boss_type: BossType, floor: i32) -> Self {
        let quality = if floor >= 3 {
            LootQuality::Epic
        } else if floor >= 2 {
            LootQuality::Rare
        } else {
            LootQuality::Common
        };
        
        let rune_count = match quality {
            LootQuality::Common => 2,
            LootQuality::Rare => 3,
            LootQuality::Epic => 5,
        };
        
        // Void Wraith drops extra loot
        let rune_count = if boss_type == BossType::VoidWraith {
            rune_count + 1
        } else {
            rune_count
        };
        
        Self { quality, rune_count }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_boss_type_for_floor() {
        assert_eq!(BossType::for_floor(1), BossType::FireLord);
        assert_eq!(BossType::for_floor(2), BossType::FrostQueen);
        assert_eq!(BossType::for_floor(3), BossType::VoidWraith);
        assert_eq!(BossType::for_floor(4), BossType::FireLord); // Cycles
    }
    
    #[test]
    fn test_boss_enrage() {
        let mut boss = Boss::new(BossType::FireLord);
        assert!(!boss.enraged);
        
        boss.check_enrage(0.5);
        assert!(!boss.enraged);
        
        boss.check_enrage(0.25);
        assert!(boss.enraged);
    }
    
    #[test]
    fn test_boss_loot_quality() {
        let loot1 = BossLoot::from_boss(BossType::FireLord, 1);
        assert_eq!(loot1.quality, LootQuality::Common);
        assert_eq!(loot1.rune_count, 2);
        
        let loot3 = BossLoot::from_boss(BossType::FireLord, 3);
        assert_eq!(loot3.quality, LootQuality::Epic);
        assert_eq!(loot3.rune_count, 5);
    }
    
    #[test]
    fn test_void_wraith_extra_loot() {
        let void_loot = BossLoot::from_boss(BossType::VoidWraith, 1);
        let fire_loot = BossLoot::from_boss(BossType::FireLord, 1);
        assert_eq!(void_loot.rune_count, fire_loot.rune_count + 1);
    }
    
    #[test]
    fn test_boss_display() {
        let boss = BossType::FireLord;
        assert_eq!(boss.name(), "Fire Lord");
        assert_eq!(boss.glyph(), 'F');
        assert_eq!(boss.color(), (255, 100, 50));
    }
}
