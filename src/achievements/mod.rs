//! Achievement system for Hexcaster.
//!
//! Tracks player accomplishments across runs.

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Achievement definitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Achievement {
    // Victory achievements
    FirstVictory,        // Win your first game
    SpeedrunVictory,     // Win in under 100 turns
    FlawlessVictory,     // Win without taking damage
    
    // Combat achievements
    FirstBlood,          // Kill your first enemy
    Slayer,              // Kill 50 enemies total
    BossHunter,          // Defeat a boss
    BossSlayer,          // Defeat all 3 boss types
    Untouchable,         // Complete a floor without taking damage
    
    // Magic achievements  
    Apprentice,          // Craft your first spell
    Archmage,            // Have 5 spells in grimoire at once
    ElementalMaster,     // Use all 6 elements in one run
    ShapeShifter,        // Use all 6 shapes in one run
    
    // Exploration achievements
    Explorer,            // Reach floor 2
    DeepDiver,           // Reach floor 3
    RuneCollector,       // Collect 10 runes in one run
    
    // Challenge achievements
    DailyPlayer,         // Complete a daily challenge
    DailyStreak3,        // Complete 3 daily challenges
    DailyStreak7,        // Complete 7 daily challenges
    TopScore,            // Get #1 on daily leaderboard
    
    // Miscellaneous
    Survivor,            // Win with less than 10 HP
    Pacifist,            // Complete a floor without killing
    HoarderOfRunes,      // Collect 20 runes in one run
}

impl Achievement {
    /// Get the display name for this achievement.
    pub fn name(&self) -> &'static str {
        match self {
            Self::FirstVictory => "First Victory",
            Self::SpeedrunVictory => "Speedrunner",
            Self::FlawlessVictory => "Flawless",
            Self::FirstBlood => "First Blood",
            Self::Slayer => "Slayer",
            Self::BossHunter => "Boss Hunter",
            Self::BossSlayer => "Boss Slayer",
            Self::Untouchable => "Untouchable",
            Self::Apprentice => "Apprentice",
            Self::Archmage => "Archmage",
            Self::ElementalMaster => "Elemental Master",
            Self::ShapeShifter => "Shape Shifter",
            Self::Explorer => "Explorer",
            Self::DeepDiver => "Deep Diver",
            Self::RuneCollector => "Rune Collector",
            Self::DailyPlayer => "Daily Player",
            Self::DailyStreak3 => "Daily Streak",
            Self::DailyStreak7 => "Weekly Warrior",
            Self::TopScore => "Champion",
            Self::Survivor => "Survivor",
            Self::Pacifist => "Pacifist",
            Self::HoarderOfRunes => "Hoarder",
        }
    }

    /// Get the description for this achievement.
    pub fn description(&self) -> &'static str {
        match self {
            Self::FirstVictory => "Win your first game",
            Self::SpeedrunVictory => "Win in under 100 turns",
            Self::FlawlessVictory => "Win without taking any damage",
            Self::FirstBlood => "Kill your first enemy",
            Self::Slayer => "Kill 50 enemies total",
            Self::BossHunter => "Defeat a boss",
            Self::BossSlayer => "Defeat all 3 boss types",
            Self::Untouchable => "Complete a floor without taking damage",
            Self::Apprentice => "Craft your first spell",
            Self::Archmage => "Have 5 spells at once",
            Self::ElementalMaster => "Use all 6 elements in one run",
            Self::ShapeShifter => "Use all 6 shapes in one run",
            Self::Explorer => "Reach floor 2",
            Self::DeepDiver => "Reach floor 3",
            Self::RuneCollector => "Collect 10 runes in one run",
            Self::DailyPlayer => "Complete a daily challenge",
            Self::DailyStreak3 => "Complete 3 daily challenges",
            Self::DailyStreak7 => "Complete 7 daily challenges",
            Self::TopScore => "Get #1 on daily leaderboard",
            Self::Survivor => "Win with less than 10 HP",
            Self::Pacifist => "Complete a floor without killing",
            Self::HoarderOfRunes => "Collect 20 runes in one run",
        }
    }

    /// Get all achievements.
    pub fn all() -> Vec<Achievement> {
        vec![
            Self::FirstVictory,
            Self::SpeedrunVictory,
            Self::FlawlessVictory,
            Self::FirstBlood,
            Self::Slayer,
            Self::BossHunter,
            Self::BossSlayer,
            Self::Untouchable,
            Self::Apprentice,
            Self::Archmage,
            Self::ElementalMaster,
            Self::ShapeShifter,
            Self::Explorer,
            Self::DeepDiver,
            Self::RuneCollector,
            Self::DailyPlayer,
            Self::DailyStreak3,
            Self::DailyStreak7,
            Self::TopScore,
            Self::Survivor,
            Self::Pacifist,
            Self::HoarderOfRunes,
        ]
    }
}

/// Persistent achievement storage.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AchievementStore {
    /// Unlocked achievements.
    pub unlocked: HashSet<Achievement>,
    /// Total enemies killed across all runs.
    pub total_kills: u32,
    /// Boss types defeated.
    pub bosses_defeated: HashSet<String>,
    /// Daily challenges completed (dates).
    pub daily_completions: Vec<String>,
}

impl AchievementStore {
    /// Load achievements from file.
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

    /// Save achievements to file.
    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)
    }

    /// Unlock an achievement, returning true if newly unlocked.
    pub fn unlock(&mut self, achievement: Achievement) -> bool {
        self.unlocked.insert(achievement)
    }

    /// Check if an achievement is unlocked.
    pub fn is_unlocked(&self, achievement: Achievement) -> bool {
        self.unlocked.contains(&achievement)
    }

    /// Get count of unlocked achievements.
    pub fn unlocked_count(&self) -> usize {
        self.unlocked.len()
    }

    /// Get total achievement count.
    pub fn total_count(&self) -> usize {
        Achievement::all().len()
    }

    /// Record an enemy kill, unlocking achievements if needed.
    pub fn record_kill(&mut self) -> Vec<Achievement> {
        let mut newly_unlocked = Vec::new();
        
        self.total_kills += 1;
        
        if self.total_kills == 1 && self.unlock(Achievement::FirstBlood) {
            newly_unlocked.push(Achievement::FirstBlood);
        }
        if self.total_kills >= 50 && self.unlock(Achievement::Slayer) {
            newly_unlocked.push(Achievement::Slayer);
        }
        
        newly_unlocked
    }

    /// Record a boss defeat.
    pub fn record_boss_defeat(&mut self, boss_type: &str) -> Vec<Achievement> {
        let mut newly_unlocked = Vec::new();
        
        self.bosses_defeated.insert(boss_type.to_string());
        
        if self.unlock(Achievement::BossHunter) {
            newly_unlocked.push(Achievement::BossHunter);
        }
        
        // Check if all 3 boss types defeated
        let all_bosses = ["FireLord", "FrostQueen", "VoidWraith"];
        if all_bosses.iter().all(|b| self.bosses_defeated.contains(*b)) {
            if self.unlock(Achievement::BossSlayer) {
                newly_unlocked.push(Achievement::BossSlayer);
            }
        }
        
        newly_unlocked
    }

    /// Record a daily challenge completion.
    pub fn record_daily_completion(&mut self, date: &str) -> Vec<Achievement> {
        let mut newly_unlocked = Vec::new();
        
        if !self.daily_completions.contains(&date.to_string()) {
            self.daily_completions.push(date.to_string());
        }
        
        if self.unlock(Achievement::DailyPlayer) {
            newly_unlocked.push(Achievement::DailyPlayer);
        }
        
        if self.daily_completions.len() >= 3 && self.unlock(Achievement::DailyStreak3) {
            newly_unlocked.push(Achievement::DailyStreak3);
        }
        
        if self.daily_completions.len() >= 7 && self.unlock(Achievement::DailyStreak7) {
            newly_unlocked.push(Achievement::DailyStreak7);
        }
        
        newly_unlocked
    }

    /// Get the save file path.
    fn path() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("hexcaster")
            .join("achievements.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_achievement_all() {
        let all = Achievement::all();
        assert_eq!(all.len(), 22);
    }

    #[test]
    fn test_achievement_name() {
        assert_eq!(Achievement::FirstVictory.name(), "First Victory");
        assert_eq!(Achievement::BossSlayer.name(), "Boss Slayer");
    }

    #[test]
    fn test_achievement_description() {
        let desc = Achievement::FirstVictory.description();
        assert!(desc.contains("first game"));
    }

    #[test]
    fn test_store_unlock() {
        let mut store = AchievementStore::default();
        assert!(!store.is_unlocked(Achievement::FirstVictory));
        
        let newly = store.unlock(Achievement::FirstVictory);
        assert!(newly);
        assert!(store.is_unlocked(Achievement::FirstVictory));
        
        // Second unlock returns false
        let newly = store.unlock(Achievement::FirstVictory);
        assert!(!newly);
    }

    #[test]
    fn test_store_counts() {
        let mut store = AchievementStore::default();
        assert_eq!(store.unlocked_count(), 0);
        assert_eq!(store.total_count(), 22);
        
        store.unlock(Achievement::FirstBlood);
        store.unlock(Achievement::Explorer);
        assert_eq!(store.unlocked_count(), 2);
    }

    #[test]
    fn test_record_kill() {
        let mut store = AchievementStore::default();
        
        let unlocked = store.record_kill();
        assert_eq!(unlocked.len(), 1);
        assert_eq!(unlocked[0], Achievement::FirstBlood);
        
        // Second kill doesn't unlock again
        let unlocked = store.record_kill();
        assert!(unlocked.is_empty());
    }

    #[test]
    fn test_record_kill_slayer() {
        let mut store = AchievementStore::default();
        store.total_kills = 49;
        
        let unlocked = store.record_kill();
        assert!(unlocked.contains(&Achievement::Slayer));
    }

    #[test]
    fn test_record_boss_defeat() {
        let mut store = AchievementStore::default();
        
        let unlocked = store.record_boss_defeat("FireLord");
        assert!(unlocked.contains(&Achievement::BossHunter));
        
        store.record_boss_defeat("FrostQueen");
        let unlocked = store.record_boss_defeat("VoidWraith");
        assert!(unlocked.contains(&Achievement::BossSlayer));
    }

    #[test]
    fn test_record_daily() {
        let mut store = AchievementStore::default();
        
        let unlocked = store.record_daily_completion("2026-03-01");
        assert!(unlocked.contains(&Achievement::DailyPlayer));
        
        store.record_daily_completion("2026-03-02");
        let unlocked = store.record_daily_completion("2026-03-03");
        assert!(unlocked.contains(&Achievement::DailyStreak3));
    }
}
