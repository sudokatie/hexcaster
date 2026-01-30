//! Hex grid algorithms: range, ring, cone, line of sight, pathfinding.

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

use super::coord::{Hex, DIRECTIONS};
use super::grid::HexGrid;

/// Tile trait for pathfinding and line of sight.
pub trait Tile {
    fn is_walkable(&self) -> bool;
    fn is_transparent(&self) -> bool;
}

/// Get all hexes within a given radius (filled circle).
pub fn range(center: Hex, radius: i32) -> Vec<Hex> {
    let mut results = Vec::new();
    for x in -radius..=radius {
        let y_min = (-radius).max(-x - radius);
        let y_max = radius.min(-x + radius);
        for y in y_min..=y_max {
            let z = -x - y;
            results.push(Hex::new(x, y, z) + center);
        }
    }
    results
}

/// Get hexes at exactly a given distance (ring/circle outline).
pub fn ring(center: Hex, radius: i32) -> Vec<Hex> {
    if radius == 0 {
        return vec![center];
    }

    let mut results = Vec::with_capacity(6 * radius as usize);
    let mut hex = center + Hex::new(-radius, radius, 0);

    for direction in 0..6 {
        for _ in 0..radius {
            results.push(hex);
            hex = hex.neighbor(direction);
        }
    }
    results
}

/// Get hexes in a cone shape.
pub fn cone(origin: Hex, direction: usize, cone_range: i32) -> Vec<Hex> {
    let mut results = Vec::new();
    let dir = DIRECTIONS[direction % 6];

    for dist in 1..=cone_range {
        // Walk along the main direction
        let center = origin + Hex::new(dir.x * dist, dir.y * dist, dir.z * dist);
        results.push(center);

        // Add spread on either side proportional to distance
        let spread = dist / 2;
        let left_dir = (direction + 1) % 6;
        let right_dir = (direction + 5) % 6;

        let mut left = center;
        let mut right = center;
        for _ in 0..spread {
            left = left.neighbor(left_dir);
            right = right.neighbor(right_dir);
            results.push(left);
            results.push(right);
        }
    }
    results
}

/// Check if there's line of sight between two hexes.
pub fn line_of_sight<T: Tile>(grid: &HexGrid<T>, from: Hex, to: Hex) -> bool {
    let line = from.line_to(to);
    // Check all hexes except the start and end
    for hex in line.iter().skip(1).take(line.len().saturating_sub(2)) {
        if let Some(tile) = grid.get(*hex) {
            if !tile.is_transparent() {
                return false;
            }
        }
    }
    true
}

/// A* pathfinding between two hexes.
pub fn pathfind<T: Tile>(grid: &HexGrid<T>, from: Hex, to: Hex) -> Option<Vec<Hex>> {
    if from == to {
        return Some(vec![from]);
    }

    // Check if target is walkable
    if let Some(tile) = grid.get(to) {
        if !tile.is_walkable() {
            return None;
        }
    } else {
        return None; // Target doesn't exist
    }

    let mut open_set = BinaryHeap::new();
    let mut came_from: HashMap<Hex, Hex> = HashMap::new();
    let mut g_score: HashMap<Hex, i32> = HashMap::new();

    g_score.insert(from, 0);
    open_set.push(Node {
        hex: from,
        f_score: from.distance(to),
    });

    let mut visited: HashSet<Hex> = HashSet::new();

    while let Some(Node { hex: current, .. }) = open_set.pop() {
        if current == to {
            return Some(reconstruct_path(&came_from, current));
        }

        if visited.contains(&current) {
            continue;
        }
        visited.insert(current);

        let current_g = g_score.get(&current).copied().unwrap_or(i32::MAX);

        for neighbor in current.neighbors() {
            // Check if neighbor is walkable
            let walkable = if let Some(tile) = grid.get(neighbor) {
                tile.is_walkable()
            } else {
                false
            };

            if !walkable {
                continue;
            }

            let tentative_g = current_g.saturating_add(1);

            if tentative_g < g_score.get(&neighbor).copied().unwrap_or(i32::MAX) {
                came_from.insert(neighbor, current);
                g_score.insert(neighbor, tentative_g);
                let f = tentative_g + neighbor.distance(to);
                open_set.push(Node {
                    hex: neighbor,
                    f_score: f,
                });
            }
        }
    }

    None // No path found
}

#[derive(Eq, PartialEq)]
struct Node {
    hex: Hex,
    f_score: i32,
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f_score.cmp(&self.f_score) // Reverse for min-heap
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn reconstruct_path(came_from: &HashMap<Hex, Hex>, mut current: Hex) -> Vec<Hex> {
    let mut path = vec![current];
    while let Some(&prev) = came_from.get(&current) {
        path.push(prev);
        current = prev;
    }
    path.reverse();
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct TestTile {
        walkable: bool,
        transparent: bool,
    }

    impl Tile for TestTile {
        fn is_walkable(&self) -> bool {
            self.walkable
        }
        fn is_transparent(&self) -> bool {
            self.transparent
        }
    }

    #[test]
    fn test_range_zero() {
        let hexes = range(Hex::origin(), 0);
        assert_eq!(hexes.len(), 1);
        assert_eq!(hexes[0], Hex::origin());
    }

    #[test]
    fn test_range_one() {
        let hexes = range(Hex::origin(), 1);
        assert_eq!(hexes.len(), 7); // center + 6 neighbors
    }

    #[test]
    fn test_range_two() {
        let hexes = range(Hex::origin(), 2);
        assert_eq!(hexes.len(), 19); // 1 + 6 + 12
    }

    #[test]
    fn test_ring_zero() {
        let hexes = ring(Hex::origin(), 0);
        assert_eq!(hexes.len(), 1);
    }

    #[test]
    fn test_ring_one() {
        let hexes = ring(Hex::origin(), 1);
        assert_eq!(hexes.len(), 6);
    }

    #[test]
    fn test_ring_two() {
        let hexes = ring(Hex::origin(), 2);
        assert_eq!(hexes.len(), 12);
    }

    #[test]
    fn test_cone() {
        let hexes = cone(Hex::origin(), 0, 3);
        assert!(!hexes.is_empty());
        // All hexes should be reachable from origin in direction 0
    }

    #[test]
    fn test_line_of_sight_clear() {
        let mut grid = HexGrid::new();
        let floor = TestTile {
            walkable: true,
            transparent: true,
        };
        for hex in range(Hex::origin(), 5) {
            grid.set(hex, floor.clone());
        }

        let from = Hex::origin();
        let to = Hex::new(3, -3, 0);
        assert!(line_of_sight(&grid, from, to));
    }

    #[test]
    fn test_line_of_sight_blocked() {
        let mut grid = HexGrid::new();
        let floor = TestTile {
            walkable: true,
            transparent: true,
        };
        let wall = TestTile {
            walkable: false,
            transparent: false,
        };

        for hex in range(Hex::origin(), 5) {
            grid.set(hex, floor.clone());
        }
        // Put wall between origin and target
        grid.set(Hex::new(1, -1, 0), wall);

        let from = Hex::origin();
        let to = Hex::new(3, -3, 0);
        assert!(!line_of_sight(&grid, from, to));
    }

    #[test]
    fn test_pathfind_same_hex() {
        let mut grid = HexGrid::new();
        let floor = TestTile {
            walkable: true,
            transparent: true,
        };
        grid.set(Hex::origin(), floor);

        let path = pathfind(&grid, Hex::origin(), Hex::origin());
        assert!(path.is_some());
        assert_eq!(path.unwrap().len(), 1);
    }

    #[test]
    fn test_pathfind_neighbor() {
        let mut grid = HexGrid::new();
        let floor = TestTile {
            walkable: true,
            transparent: true,
        };
        let from = Hex::origin();
        let to = from.neighbor(0);
        grid.set(from, floor.clone());
        grid.set(to, floor);

        let path = pathfind(&grid, from, to);
        assert!(path.is_some());
        assert_eq!(path.unwrap().len(), 2);
    }

    #[test]
    fn test_pathfind_around_wall() {
        let mut grid = HexGrid::new();
        let floor = TestTile {
            walkable: true,
            transparent: true,
        };
        let wall = TestTile {
            walkable: false,
            transparent: false,
        };

        // Fill a 3x3 area with floor
        for hex in range(Hex::origin(), 3) {
            grid.set(hex, floor.clone());
        }

        // Put wall directly between start and target
        grid.set(Hex::new(1, -1, 0), wall);

        let from = Hex::origin();
        let to = Hex::new(2, -2, 0);
        let path = pathfind(&grid, from, to);

        assert!(path.is_some());
        let path = path.unwrap();
        // Path should go around the wall
        assert!(path.len() > 3);
    }

    #[test]
    fn test_pathfind_no_path() {
        let mut grid = HexGrid::new();
        let floor = TestTile {
            walkable: true,
            transparent: true,
        };
        let wall = TestTile {
            walkable: false,
            transparent: false,
        };

        grid.set(Hex::origin(), floor);
        grid.set(Hex::new(2, -2, 0), wall); // Target is unwalkable

        let path = pathfind(&grid, Hex::origin(), Hex::new(2, -2, 0));
        assert!(path.is_none());
    }
}
