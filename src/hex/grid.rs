//! Hex grid storage using HashMap.

use std::collections::HashMap;

use super::coord::Hex;

/// A sparse hex grid storing tiles of type T.
#[derive(Debug, Clone)]
pub struct HexGrid<T> {
    tiles: HashMap<Hex, T>,
}

impl<T> Default for HexGrid<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> HexGrid<T> {
    /// Create an empty grid.
    pub fn new() -> Self {
        Self {
            tiles: HashMap::new(),
        }
    }

    /// Get a tile at a position.
    pub fn get(&self, pos: Hex) -> Option<&T> {
        self.tiles.get(&pos)
    }

    /// Get a mutable reference to a tile.
    pub fn get_mut(&mut self, pos: Hex) -> Option<&mut T> {
        self.tiles.get_mut(&pos)
    }

    /// Set a tile at a position.
    pub fn set(&mut self, pos: Hex, tile: T) {
        self.tiles.insert(pos, tile);
    }

    /// Remove a tile at a position.
    pub fn remove(&mut self, pos: Hex) -> Option<T> {
        self.tiles.remove(&pos)
    }

    /// Check if a position has a tile.
    pub fn contains(&self, pos: Hex) -> bool {
        self.tiles.contains_key(&pos)
    }

    /// Number of tiles in the grid.
    pub fn len(&self) -> usize {
        self.tiles.len()
    }

    /// Check if grid is empty.
    pub fn is_empty(&self) -> bool {
        self.tiles.is_empty()
    }

    /// Iterate over all tiles.
    pub fn iter(&self) -> impl Iterator<Item = (Hex, &T)> {
        self.tiles.iter().map(|(k, v)| (*k, v))
    }

    /// Iterate over all tiles mutably.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Hex, &mut T)> {
        self.tiles.iter_mut().map(|(k, v)| (*k, v))
    }

    /// Get all neighboring tiles that exist.
    pub fn neighbors(&self, pos: Hex) -> Vec<(Hex, &T)> {
        pos.neighbors()
            .into_iter()
            .filter_map(|n| self.get(n).map(|t| (n, t)))
            .collect()
    }

    /// Clear all tiles.
    pub fn clear(&mut self) {
        self.tiles.clear();
    }
}

impl<T: Clone> HexGrid<T> {
    /// Fill a region with a tile.
    pub fn fill(&mut self, hexes: impl IntoIterator<Item = Hex>, tile: T) {
        for hex in hexes {
            self.set(hex, tile.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    enum Tile {
        Floor,
        Wall,
    }

    #[test]
    fn test_new_grid() {
        let grid: HexGrid<Tile> = HexGrid::new();
        assert!(grid.is_empty());
    }

    #[test]
    fn test_set_and_get() {
        let mut grid = HexGrid::new();
        let pos = Hex::origin();
        grid.set(pos, Tile::Floor);
        assert_eq!(grid.get(pos), Some(&Tile::Floor));
    }

    #[test]
    fn test_get_nonexistent() {
        let grid: HexGrid<Tile> = HexGrid::new();
        assert_eq!(grid.get(Hex::origin()), None);
    }

    #[test]
    fn test_contains() {
        let mut grid = HexGrid::new();
        let pos = Hex::origin();
        assert!(!grid.contains(pos));
        grid.set(pos, Tile::Wall);
        assert!(grid.contains(pos));
    }

    #[test]
    fn test_remove() {
        let mut grid = HexGrid::new();
        let pos = Hex::origin();
        grid.set(pos, Tile::Floor);
        let removed = grid.remove(pos);
        assert_eq!(removed, Some(Tile::Floor));
        assert!(!grid.contains(pos));
    }

    #[test]
    fn test_len() {
        let mut grid = HexGrid::new();
        assert_eq!(grid.len(), 0);
        grid.set(Hex::origin(), Tile::Floor);
        assert_eq!(grid.len(), 1);
        grid.set(Hex::new(1, -1, 0), Tile::Wall);
        assert_eq!(grid.len(), 2);
    }

    #[test]
    fn test_iter() {
        let mut grid = HexGrid::new();
        grid.set(Hex::origin(), Tile::Floor);
        grid.set(Hex::new(1, -1, 0), Tile::Wall);
        let items: Vec<_> = grid.iter().collect();
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_neighbors() {
        let mut grid = HexGrid::new();
        let center = Hex::origin();
        grid.set(center, Tile::Floor);
        grid.set(center.neighbor(0), Tile::Wall);
        grid.set(center.neighbor(2), Tile::Floor);

        let neighbors = grid.neighbors(center);
        assert_eq!(neighbors.len(), 2);
    }

    #[test]
    fn test_fill() {
        let mut grid = HexGrid::new();
        let hexes = vec![Hex::origin(), Hex::new(1, -1, 0), Hex::new(0, 1, -1)];
        grid.fill(hexes, Tile::Floor);
        assert_eq!(grid.len(), 3);
        assert_eq!(grid.get(Hex::origin()), Some(&Tile::Floor));
    }
}
