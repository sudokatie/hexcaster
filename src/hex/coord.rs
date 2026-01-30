//! Hex coordinate system using cube coordinates (x + y + z = 0).

use std::fmt;
use std::ops::{Add, Sub};

/// Six hex directions in cube coordinates.
pub const DIRECTIONS: [Hex; 6] = [
    Hex { x: 1, y: -1, z: 0 }, // East
    Hex { x: 1, y: 0, z: -1 }, // Northeast
    Hex { x: 0, y: 1, z: -1 }, // Northwest
    Hex { x: -1, y: 1, z: 0 }, // West
    Hex { x: -1, y: 0, z: 1 }, // Southwest
    Hex { x: 0, y: -1, z: 1 }, // Southeast
];

/// A hex coordinate using cube coordinates.
/// Invariant: x + y + z = 0
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Hex {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Hex {
    /// Create a new hex coordinate. Panics if x + y + z != 0.
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        assert!(x + y + z == 0, "Hex coordinates must sum to zero");
        Self { x, y, z }
    }

    /// Create a hex from axial coordinates (q, r).
    /// z is computed as -q - r.
    pub fn from_axial(q: i32, r: i32) -> Self {
        Self {
            x: q,
            y: r,
            z: -q - r,
        }
    }

    /// The origin hex (0, 0, 0).
    pub fn origin() -> Self {
        Self { x: 0, y: 0, z: 0 }
    }

    /// Get the neighbor in a given direction (0-5).
    pub fn neighbor(&self, direction: usize) -> Self {
        *self + DIRECTIONS[direction % 6]
    }

    /// Get all six neighbors.
    pub fn neighbors(&self) -> [Hex; 6] {
        [
            self.neighbor(0),
            self.neighbor(1),
            self.neighbor(2),
            self.neighbor(3),
            self.neighbor(4),
            self.neighbor(5),
        ]
    }

    /// Manhattan distance to another hex.
    pub fn distance(&self, other: Hex) -> i32 {
        ((self.x - other.x).abs() + (self.y - other.y).abs() + (self.z - other.z).abs()) / 2
    }

    /// Draw a line from self to other, returning all hexes along the way.
    pub fn line_to(&self, other: Hex) -> Vec<Hex> {
        let n = self.distance(other);
        if n == 0 {
            return vec![*self];
        }

        let mut results = Vec::with_capacity(n as usize + 1);
        for i in 0..=n {
            let t = i as f64 / n as f64;
            results.push(Self::lerp(*self, other, t));
        }
        results
    }

    /// Linear interpolation between two hexes.
    fn lerp(a: Hex, b: Hex, t: f64) -> Hex {
        let x = a.x as f64 + (b.x - a.x) as f64 * t;
        let y = a.y as f64 + (b.y - a.y) as f64 * t;
        let z = a.z as f64 + (b.z - a.z) as f64 * t;
        Self::round(x, y, z)
    }

    /// Round floating point coordinates to nearest valid hex.
    fn round(x: f64, y: f64, z: f64) -> Hex {
        let mut rx = x.round() as i32;
        let mut ry = y.round() as i32;
        let mut rz = z.round() as i32;

        let x_diff = (rx as f64 - x).abs();
        let y_diff = (ry as f64 - y).abs();
        let z_diff = (rz as f64 - z).abs();

        // Reset the component with largest rounding error
        if x_diff > y_diff && x_diff > z_diff {
            rx = -ry - rz;
        } else if y_diff > z_diff {
            ry = -rx - rz;
        } else {
            rz = -rx - ry;
        }

        Hex {
            x: rx,
            y: ry,
            z: rz,
        }
    }
}

impl Add for Hex {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl Sub for Hex {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl fmt::Debug for Hex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hex({}, {}, {})", self.x, self.y, self.z)
    }
}

impl fmt::Display for Hex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_valid() {
        let h = Hex::new(1, -1, 0);
        assert_eq!(h.x, 1);
        assert_eq!(h.y, -1);
        assert_eq!(h.z, 0);
    }

    #[test]
    #[should_panic(expected = "Hex coordinates must sum to zero")]
    fn test_new_invalid() {
        Hex::new(1, 1, 1);
    }

    #[test]
    fn test_from_axial() {
        let h = Hex::from_axial(2, 3);
        assert_eq!(h.x + h.y + h.z, 0);
        assert_eq!(h.x, 2);
        assert_eq!(h.y, 3);
        assert_eq!(h.z, -5);
    }

    #[test]
    fn test_origin() {
        let o = Hex::origin();
        assert_eq!(o.x, 0);
        assert_eq!(o.y, 0);
        assert_eq!(o.z, 0);
    }

    #[test]
    fn test_neighbor() {
        let o = Hex::origin();
        let n = o.neighbor(0); // East
        assert_eq!(n, Hex::new(1, -1, 0));
    }

    #[test]
    fn test_neighbors_count() {
        let o = Hex::origin();
        let neighbors = o.neighbors();
        assert_eq!(neighbors.len(), 6);
    }

    #[test]
    fn test_distance_same() {
        let h = Hex::new(1, -1, 0);
        assert_eq!(h.distance(h), 0);
    }

    #[test]
    fn test_distance_neighbor() {
        let o = Hex::origin();
        let n = o.neighbor(0);
        assert_eq!(o.distance(n), 1);
    }

    #[test]
    fn test_distance_two_away() {
        let a = Hex::origin();
        let b = Hex::new(2, -2, 0);
        assert_eq!(a.distance(b), 2);
    }

    #[test]
    fn test_add() {
        let a = Hex::new(1, -1, 0);
        let b = Hex::new(0, 1, -1);
        let c = a + b;
        assert_eq!(c, Hex::new(1, 0, -1));
    }

    #[test]
    fn test_sub() {
        let a = Hex::new(1, -1, 0);
        let b = Hex::new(0, 1, -1);
        let c = a - b;
        assert_eq!(c, Hex::new(1, -2, 1));
    }

    #[test]
    fn test_line_to_same() {
        let h = Hex::origin();
        let line = h.line_to(h);
        assert_eq!(line, vec![h]);
    }

    #[test]
    fn test_line_to_neighbor() {
        let a = Hex::origin();
        let b = a.neighbor(0);
        let line = a.line_to(b);
        assert_eq!(line.len(), 2);
        assert_eq!(line[0], a);
        assert_eq!(line[1], b);
    }

    #[test]
    fn test_line_to_multiple() {
        let a = Hex::origin();
        let b = Hex::new(3, -3, 0);
        let line = a.line_to(b);
        assert_eq!(line.len(), 4); // 0, 1, 2, 3
    }
}
