//! Procedural dungeon generation.

use crate::hex::{range, Hex, HexGrid};
use rand::Rng;

/// Tile types for the dungeon.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DungeonTile {
    Floor,
    Wall,
    Stairs,
}

impl crate::hex::Tile for DungeonTile {
    fn is_walkable(&self) -> bool {
        matches!(self, DungeonTile::Floor | DungeonTile::Stairs)
    }

    fn is_transparent(&self) -> bool {
        matches!(self, DungeonTile::Floor | DungeonTile::Stairs)
    }
}

/// Configuration for dungeon generation.
#[derive(Debug, Clone)]
pub struct DungeonConfig {
    pub width: i32,
    pub height: i32,
    pub num_rooms: usize,
    pub floor_number: i32,
}

impl Default for DungeonConfig {
    fn default() -> Self {
        Self {
            width: 40,
            height: 30,
            num_rooms: 6,
            floor_number: 1,
        }
    }
}

/// Generate a dungeon.
pub fn generate(config: &DungeonConfig, rng: &mut impl Rng) -> (HexGrid<DungeonTile>, Hex) {
    let mut grid = HexGrid::new();

    // Fill with walls first
    for x in -(config.width / 2)..=(config.width / 2) {
        for y in -(config.height / 2)..=(config.height / 2) {
            if let Some(hex) = try_create_hex(x, y) {
                grid.set(hex, DungeonTile::Wall);
            }
        }
    }

    // Generate rooms
    let mut rooms: Vec<(Hex, i32)> = Vec::new();
    let mut attempts = 0;
    while rooms.len() < config.num_rooms && attempts < 100 {
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

    // Place stairs in last room
    if let Some((last_room, _)) = rooms.last() {
        grid.set(*last_room, DungeonTile::Stairs);
    }

    // Player starts in first room
    let player_start = rooms.first().map(|(c, _)| *c).unwrap_or(Hex::origin());

    (grid, player_start)
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
        let (grid, _) = generate(&config, &mut rng);

        let has_floor = grid.iter().any(|(_, t)| *t == DungeonTile::Floor);
        assert!(has_floor);
    }

    #[test]
    fn test_generate_has_stairs() {
        let config = DungeonConfig::default();
        let mut rng = StdRng::seed_from_u64(42);
        let (grid, _) = generate(&config, &mut rng);

        let has_stairs = grid.iter().any(|(_, t)| *t == DungeonTile::Stairs);
        assert!(has_stairs);
    }

    #[test]
    fn test_player_start_is_floor() {
        let config = DungeonConfig::default();
        let mut rng = StdRng::seed_from_u64(42);
        let (grid, start) = generate(&config, &mut rng);

        assert_eq!(grid.get(start), Some(&DungeonTile::Floor));
    }
}
