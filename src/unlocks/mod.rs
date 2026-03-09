//! Unlockable content system for Hexcaster.
//!
//! Certain runes are locked at the start and must be unlocked
//! through achievements.

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::achievements::{Achievement, AchievementStore};
use crate::magic::{Element, Modifier, Shape};

/// Content that can be unlocked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Unlockable {
    // Advanced elements
    ElementLight,
    ElementDark,
    ElementVoid,
    ElementPoison,
    // Advanced shapes
    ShapeWall,
    ShapeChain,
    ShapeNova,
    // Advanced modifiers
    ModifierHoming,
    ModifierPiercing,
    ModifierVampiric,
    ModifierSilent,
}

impl Unlockable {
    /// Get all unlockables.
    pub fn all() -> Vec<Unlockable> {
        vec![
            Self::ElementLight,
            Self::ElementDark,
            Self::ElementVoid,
            Self::ElementPoison,
            Self::ShapeWall,
            Self::ShapeChain,
            Self::ShapeNova,
            Self::ModifierHoming,
            Self::ModifierPiercing,
            Self::ModifierVampiric,
            Self::ModifierSilent,
        ]
    }

    /// Get the display name for this unlockable.
    pub fn name(&self) -> &'static str {
        match self {
            Self::ElementLight => "Light Element",
            Self::ElementDark => "Dark Element",
            Self::ElementVoid => "Void Element",
            Self::ElementPoison => "Poison Element",
            Self::ShapeWall => "Wall Shape",
            Self::ShapeChain => "Chain Shape",
            Self::ShapeNova => "Nova Shape",
            Self::ModifierHoming => "Homing Modifier",
            Self::ModifierPiercing => "Piercing Modifier",
            Self::ModifierVampiric => "Vampiric Modifier",
            Self::ModifierSilent => "Silent Modifier",
        }
    }

    /// Get the achievement required to unlock this content.
    pub fn required_achievement(&self) -> Achievement {
        match self {
            Self::ElementLight => Achievement::FirstVictory,
            Self::ElementDark => Achievement::BossHunter,
            Self::ElementVoid => Achievement::DeepDiver,
            Self::ElementPoison => Achievement::Slayer,
            Self::ShapeWall => Achievement::Explorer,
            Self::ShapeChain => Achievement::ElementalMaster,
            Self::ShapeNova => Achievement::Archmage,
            Self::ModifierHoming => Achievement::Apprentice,
            Self::ModifierPiercing => Achievement::Untouchable,
            Self::ModifierVampiric => Achievement::Survivor,
            Self::ModifierSilent => Achievement::Pacifist,
        }
    }

    /// Get unlock hint text.
    pub fn hint(&self) -> &'static str {
        match self {
            Self::ElementLight => "Win your first game",
            Self::ElementDark => "Defeat a boss",
            Self::ElementVoid => "Reach floor 3",
            Self::ElementPoison => "Kill 50 enemies total",
            Self::ShapeWall => "Reach floor 2",
            Self::ShapeChain => "Use all 6 base elements in one run",
            Self::ShapeNova => "Have 5 spells at once",
            Self::ModifierHoming => "Craft your first spell",
            Self::ModifierPiercing => "Complete a floor without damage",
            Self::ModifierVampiric => "Win with less than 10 HP",
            Self::ModifierSilent => "Complete a floor without killing",
        }
    }
}

/// Persistent unlock storage.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UnlockStore {
    /// Manually unlocked content (in case we add non-achievement unlocks later).
    pub unlocked: HashSet<Unlockable>,
}

impl UnlockStore {
    /// Load unlocks from file.
    pub fn load() -> Self {
        let path = Self::path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
                Err(_) => Self::default(),
            }
        } else {
            Self::default()
        }
    }

    /// Save unlocks to file.
    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)
    }

    /// Check if content is unlocked (checks achievement store too).
    pub fn is_unlocked(&self, unlockable: Unlockable, achievements: &AchievementStore) -> bool {
        // Check manual unlock first
        if self.unlocked.contains(&unlockable) {
            return true;
        }
        // Check if required achievement is unlocked
        achievements.is_unlocked(unlockable.required_achievement())
    }

    /// Get all unlocked content.
    pub fn get_unlocked(&self, achievements: &AchievementStore) -> HashSet<Unlockable> {
        Unlockable::all()
            .into_iter()
            .filter(|u| self.is_unlocked(*u, achievements))
            .collect()
    }

    /// Get all locked content.
    pub fn get_locked(&self, achievements: &AchievementStore) -> Vec<Unlockable> {
        Unlockable::all()
            .into_iter()
            .filter(|u| !self.is_unlocked(*u, achievements))
            .collect()
    }

    /// Check if an element is unlocked.
    pub fn is_element_unlocked(&self, element: Element, achievements: &AchievementStore) -> bool {
        match element {
            // Base elements are always unlocked
            Element::Fire | Element::Ice | Element::Lightning | Element::Earth => true,
            // Advanced elements require unlocks
            Element::Light => self.is_unlocked(Unlockable::ElementLight, achievements),
            Element::Dark => self.is_unlocked(Unlockable::ElementDark, achievements),
            Element::Void => self.is_unlocked(Unlockable::ElementVoid, achievements),
            Element::Poison => self.is_unlocked(Unlockable::ElementPoison, achievements),
        }
    }

    /// Check if a shape is unlocked.
    pub fn is_shape_unlocked(&self, shape: Shape, achievements: &AchievementStore) -> bool {
        match shape {
            // Base shapes are always unlocked
            Shape::Point | Shape::Line | Shape::Cone | Shape::Ring | Shape::Burst => true,
            // Advanced shapes require unlocks
            Shape::Wall => self.is_unlocked(Unlockable::ShapeWall, achievements),
            Shape::Chain => self.is_unlocked(Unlockable::ShapeChain, achievements),
            Shape::Nova => self.is_unlocked(Unlockable::ShapeNova, achievements),
        }
    }

    /// Check if a modifier is unlocked.
    pub fn is_modifier_unlocked(&self, modifier: Modifier, achievements: &AchievementStore) -> bool {
        match modifier {
            // Base modifiers are always unlocked
            Modifier::Power | Modifier::Range | Modifier::Duration | Modifier::Split | Modifier::Echo => true,
            // Advanced modifiers require unlocks
            Modifier::Homing => self.is_unlocked(Unlockable::ModifierHoming, achievements),
            Modifier::Piercing => self.is_unlocked(Unlockable::ModifierPiercing, achievements),
            Modifier::Vampiric => self.is_unlocked(Unlockable::ModifierVampiric, achievements),
            Modifier::Silent => self.is_unlocked(Unlockable::ModifierSilent, achievements),
        }
    }

    /// Get the save file path.
    fn path() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("hexcaster")
            .join("unlocks.json")
    }
}

/// Check for newly unlocked content after an achievement unlock.
/// Returns the list of content that was just unlocked.
pub fn check_new_unlocks(
    achievements: &AchievementStore,
    previous_unlocks: &HashSet<Unlockable>,
) -> Vec<Unlockable> {
    let unlock_store = UnlockStore::default();
    let current_unlocks = unlock_store.get_unlocked(achievements);
    
    current_unlocks
        .difference(previous_unlocks)
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unlockable_all() {
        let all = Unlockable::all();
        assert_eq!(all.len(), 11);
    }

    #[test]
    fn test_unlockable_names() {
        assert_eq!(Unlockable::ElementLight.name(), "Light Element");
        assert_eq!(Unlockable::ShapeChain.name(), "Chain Shape");
        assert_eq!(Unlockable::ModifierVampiric.name(), "Vampiric Modifier");
    }

    #[test]
    fn test_required_achievements() {
        assert_eq!(Unlockable::ElementLight.required_achievement(), Achievement::FirstVictory);
        assert_eq!(Unlockable::ElementPoison.required_achievement(), Achievement::Slayer);
        assert_eq!(Unlockable::ShapeWall.required_achievement(), Achievement::Explorer);
    }

    #[test]
    fn test_unlock_store_default() {
        let store = UnlockStore::default();
        assert!(store.unlocked.is_empty());
    }

    #[test]
    fn test_base_elements_always_unlocked() {
        let store = UnlockStore::default();
        let achievements = AchievementStore::default();
        
        assert!(store.is_element_unlocked(Element::Fire, &achievements));
        assert!(store.is_element_unlocked(Element::Ice, &achievements));
        assert!(store.is_element_unlocked(Element::Lightning, &achievements));
        assert!(store.is_element_unlocked(Element::Earth, &achievements));
    }

    #[test]
    fn test_advanced_elements_locked_by_default() {
        let store = UnlockStore::default();
        let achievements = AchievementStore::default();
        
        assert!(!store.is_element_unlocked(Element::Light, &achievements));
        assert!(!store.is_element_unlocked(Element::Dark, &achievements));
        assert!(!store.is_element_unlocked(Element::Void, &achievements));
        assert!(!store.is_element_unlocked(Element::Poison, &achievements));
    }

    #[test]
    fn test_unlock_via_achievement() {
        let store = UnlockStore::default();
        let mut achievements = AchievementStore::default();
        
        // Light element is locked
        assert!(!store.is_element_unlocked(Element::Light, &achievements));
        
        // Unlock FirstVictory achievement
        achievements.unlock(Achievement::FirstVictory);
        
        // Now Light element is unlocked
        assert!(store.is_element_unlocked(Element::Light, &achievements));
    }

    #[test]
    fn test_base_shapes_always_unlocked() {
        let store = UnlockStore::default();
        let achievements = AchievementStore::default();
        
        assert!(store.is_shape_unlocked(Shape::Point, &achievements));
        assert!(store.is_shape_unlocked(Shape::Line, &achievements));
        assert!(store.is_shape_unlocked(Shape::Cone, &achievements));
        assert!(store.is_shape_unlocked(Shape::Ring, &achievements));
        assert!(store.is_shape_unlocked(Shape::Burst, &achievements));
    }

    #[test]
    fn test_advanced_shapes_locked_by_default() {
        let store = UnlockStore::default();
        let achievements = AchievementStore::default();
        
        assert!(!store.is_shape_unlocked(Shape::Wall, &achievements));
        assert!(!store.is_shape_unlocked(Shape::Chain, &achievements));
        assert!(!store.is_shape_unlocked(Shape::Nova, &achievements));
    }

    #[test]
    fn test_base_modifiers_always_unlocked() {
        let store = UnlockStore::default();
        let achievements = AchievementStore::default();
        
        assert!(store.is_modifier_unlocked(Modifier::Power, &achievements));
        assert!(store.is_modifier_unlocked(Modifier::Range, &achievements));
        assert!(store.is_modifier_unlocked(Modifier::Duration, &achievements));
        assert!(store.is_modifier_unlocked(Modifier::Split, &achievements));
        assert!(store.is_modifier_unlocked(Modifier::Echo, &achievements));
    }

    #[test]
    fn test_advanced_modifiers_locked_by_default() {
        let store = UnlockStore::default();
        let achievements = AchievementStore::default();
        
        assert!(!store.is_modifier_unlocked(Modifier::Homing, &achievements));
        assert!(!store.is_modifier_unlocked(Modifier::Piercing, &achievements));
        assert!(!store.is_modifier_unlocked(Modifier::Vampiric, &achievements));
        assert!(!store.is_modifier_unlocked(Modifier::Silent, &achievements));
    }

    #[test]
    fn test_get_locked() {
        let store = UnlockStore::default();
        let achievements = AchievementStore::default();
        
        let locked = store.get_locked(&achievements);
        assert_eq!(locked.len(), 11); // All 11 advanced items locked
    }

    #[test]
    fn test_get_unlocked() {
        let store = UnlockStore::default();
        let mut achievements = AchievementStore::default();
        
        let unlocked = store.get_unlocked(&achievements);
        assert!(unlocked.is_empty());
        
        // Unlock some achievements
        achievements.unlock(Achievement::FirstVictory);
        achievements.unlock(Achievement::Explorer);
        
        let unlocked = store.get_unlocked(&achievements);
        assert_eq!(unlocked.len(), 2);
        assert!(unlocked.contains(&Unlockable::ElementLight));
        assert!(unlocked.contains(&Unlockable::ShapeWall));
    }

    #[test]
    fn test_check_new_unlocks() {
        let mut achievements = AchievementStore::default();
        let previous = HashSet::new();
        
        // No achievements = no unlocks
        let new = check_new_unlocks(&achievements, &previous);
        assert!(new.is_empty());
        
        // Unlock FirstVictory
        achievements.unlock(Achievement::FirstVictory);
        let new = check_new_unlocks(&achievements, &previous);
        assert_eq!(new.len(), 1);
        assert!(new.contains(&Unlockable::ElementLight));
    }

    #[test]
    fn test_hint_text() {
        let hint = Unlockable::ElementPoison.hint();
        assert!(hint.contains("50 enemies"));
        
        let hint = Unlockable::ShapeNova.hint();
        assert!(hint.contains("5 spells"));
    }
}
