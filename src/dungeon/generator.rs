//! Procedural dungeon generation.

use crate::hex::{range, Hex, HexGrid};
use rand::Rng;

/// Tile types for the dungeon.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DungeonTile {
    Floor,
    Wall,
    Stairs,
    BossFloor, // Special floor for boss room
}

impl crate::hex::Tile for DungeonTile {
    fn is_walkable(&self) -> bool {
        matches!(self, DungeonTile::Floor | DungeonTile::Stairs | DungeonTile::BossFloor)
    }

    fn is_transparent(&self) -> bool {
        matches!(self, DungeonTile::Floor | DungeonTile::Stairs | DungeonTile::BossFloor)
    }
}

/// Configuration for dungeon generation.
#[derive(Debug, Clone)]
pub struct DungeonConfig {
    pub width: i32,
    pub height: i32,
    pub num_rooms: usize,
    pub floor_number: i32,
    /// Whether to generate a boss room at the end.
    pub has_boss: bool,
}

impl Default for DungeonConfig {
    fn default() -> Self {
        Self {
            width: 40,
            height: 30,
            num_rooms: 6,
            floor_number: 1,
            has_boss: true,
        }
    }
}

/// Result of dungeon generation.
#[derive(Debug)]
pub struct DungeonResult {
    pub grid: HexGrid<DungeonTile>,
    pub player_start: Hex,
    pub boss_spawn: Option<Hex>,
}

/// Generate a dungeon.
pub fn generate(config: &DungeonConfig, rng: &mut impl Rng) -> DungeonResult {
    let mut grid = HexGrid::new();

    // Fill with walls first
    for x in -(config.width / 2)..=(config.width / 2) {
        for y in -(config.height / 2)..=(config.height / 2) {
            if let Some(hex) = try_create_hex(x, y) {
                grid.set(hex, DungeonTile::Wall);
            }
        }
    }

    // Generate rooms (fewer regular rooms if boss room needed)
    let regular_rooms = if config.has_boss {
        config.num_rooms.saturating_sub(1)
    } else {
        config.num_rooms
    };
    
    let mut rooms: Vec<(Hex, i32)> = Vec::new();
    let mut attempts = 0;
    while rooms.len() < regular_rooms && attempts < 100 {
        let x = rng.gen_range(-(config.width / 3)..(config.width / 3));
        let y = rng.gen_range(-(config.height / 3)..(config.height / 3));
        let radius = rng.gen_range(2..=4);

        if let Some(center) = try_create_hex(x, y) {
            // Check overlap
            let overlaps = rooms
                .iter()
                .any(|(rc, rr)| center.distance(*rc) < radius + rr + 2);
            if !overlaps {
                // Carve room
                for hex in range(center, radius) {
                    grid.set(hex, DungeonTile::Floor);
                }
                rooms.push((center, radius));
            }
        }
        attempts += 1;
    }

    // Generate boss room if needed (larger than regular rooms)
    let boss_spawn = if config.has_boss {
        let boss_room = generate_boss_room(&mut grid, &rooms, config, rng);
        if let Some((center, radius)) = boss_room {
            rooms.push((center, radius));
            Some(center)
        } else {
            None
        }
    } else {
        None
    };

    // Connect rooms with corridors
    for i in 0..rooms.len().saturating_sub(1) {
        let (from, _) = rooms[i];
        let (to, _) = rooms[i + 1];
        for hex in from.line_to(to) {
            grid.set(hex, DungeonTile::Floor);
            // Widen corridor slightly
            for neighbor in hex.neighbors() {
                if rng.gen_bool(0.3) {
                    grid.set(neighbor, DungeonTile::Floor);
                }
            }
        }
    }

    // Place stairs in boss room or last room
    if let Some((last_room, _)) = rooms.last() {
        grid.set(*last_room, DungeonTile::Stairs);
    }

    // Player starts in first room
    let player_start = rooms.first().map(|(c, _)| *c).unwrap_or(Hex::origin());

    DungeonResult {
        grid,
        player_start,
        boss_spawn,
    }
}

/// Generate a boss room (larger than regular rooms).
fn generate_boss_room(
    grid: &mut HexGrid<DungeonTile>,
    existing_rooms: &[(Hex, i32)],
    config: &DungeonConfig,
    rng: &mut impl Rng,
) -> Option<(Hex, i32)> {
    let boss_radius = 5; // Boss rooms are larger
    
    for _ in 0..50 {
        // Try to place boss room at edge of map
        let x = rng.gen_range(-(config.width / 4)..(config.width / 4));
        let y = rng.gen_range(-(config.height / 4)..(config.height / 4));
        
        if let Some(center) = try_create_hex(x, y) {
            // Check overlap with existing rooms (need more space for boss)
            let overlaps = existing_rooms
                .iter()
                .any(|(rc, rr)| center.distance(*rc) < boss_radius + rr + 3);
            
            if !overlaps {
                // Carve boss room with special floor
                for hex in range(center, boss_radius) {
                    grid.set(hex, DungeonTile::BossFloor);
                }
                return Some((center, boss_radius));
            }
        }
    }
    None
}

fn try_create_hex(x: i32, y: i32) -> Option<Hex> {
    let z = -x - y;
    Some(Hex::new(x, y, z))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn test_generate_has_floor() {
        let config = DungeonConfig::default();
        let mut rng = StdRng::seed_from_u64(42);
        let result = generate(&config, &mut rng);

        let has_floor = result.grid.iter().any(|(_, t)| *t == DungeonTile::Floor);
        assert!(has_floor);
    }

    #[test]
    fn test_generate_has_stairs() {
        let config = DungeonConfig::default();
        let mut rng = StdRng::seed_from_u64(42);
        let result = generate(&config, &mut rng);

        let has_stairs = result.grid.iter().any(|(_, t)| *t == DungeonTile::Stairs);
        assert!(has_stairs);
    }

    #[test]
    fn test_player_start_is_floor() {
        let config = DungeonConfig::default();
        let mut rng = StdRng::seed_from_u64(42);
        let result = generate(&config, &mut rng);

        let tile = result.grid.get(result.player_start);
        assert!(tile == Some(&DungeonTile::Floor) || tile == Some(&DungeonTile::BossFloor));
    }
    
    #[test]
    fn test_generate_boss_room() {
        let config = DungeonConfig {
            has_boss: true,
            ..Default::default()
        };
        let mut rng = StdRng::seed_from_u64(42);
        let result = generate(&config, &mut rng);
        
        // Should have boss spawn location
        assert!(result.boss_spawn.is_some());
        
        // Should have boss floor tiles
        let has_boss_floor = result.grid.iter().any(|(_, t)| *t == DungeonTile::BossFloor);
        assert!(has_boss_floor);
    }
    
    #[test]
    fn test_no_boss_room() {
        let config = DungeonConfig {
            has_boss: false,
            ..Default::default()
        };
        let mut rng = StdRng::seed_from_u64(42);
        let result = generate(&config, &mut rng);
        
        // Should not have boss spawn location
        assert!(result.boss_spawn.is_none());
        
        // Should not have boss floor tiles
        let has_boss_floor = result.grid.iter().any(|(_, t)| *t == DungeonTile::BossFloor);
        assert!(!has_boss_floor);
    }
}
