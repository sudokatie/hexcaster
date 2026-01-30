//! Spell crafting and targeting.

use super::rune::{Element, Modifier, Rune, RuneKind, Shape};
use crate::hex::{cone as hex_cone, range, ring, Hex};

/// A crafted spell ready to cast.
#[derive(Debug, Clone)]
pub struct Spell {
    pub elements: Vec<Element>,
    pub shape: Shape,
    pub modifiers: Vec<Modifier>,
    pub ap_cost: i32,
    pub base_range: i32,
    pub base_damage: i32,
}

impl Spell {
    /// Get effective range after modifiers.
    pub fn range(&self) -> i32 {
        let mut r = self.base_range;
        for m in &self.modifiers {
            if *m == Modifier::Range {
                r += 2;
            }
        }
        r
    }

    /// Get effective damage after modifiers.
    pub fn damage(&self) -> i32 {
        let mut d = self.base_damage;
        for m in &self.modifiers {
            if *m == Modifier::Power {
                d += 5;
            }
        }
        d
    }

    pub fn name(&self) -> String {
        let element_str = self
            .elements
            .iter()
            .map(|e| e.name())
            .collect::<Vec<_>>()
            .join("/");
        format!("{} {}", element_str, self.shape.name())
    }
}

/// Craft a spell from runes.
/// Requires at least one element rune; shape defaults to Point.
pub fn craft_spell(runes: &[Rune]) -> Option<Spell> {
    let mut elements = Vec::new();
    let mut shape = Shape::Point;
    let mut modifiers = Vec::new();

    for rune in runes {
        match &rune.kind {
            RuneKind::Element(e) => elements.push(*e),
            RuneKind::Shape(s) => shape = *s,
            RuneKind::Modifier(m) => modifiers.push(*m),
        }
    }

    // Must have at least one element
    if elements.is_empty() {
        return None;
    }

    // Calculate costs and stats
    let base_damage = elements.len() as i32 * 10;
    let shape_cost = match shape {
        Shape::Point => 1,
        Shape::Line => 2,
        Shape::Cone => 3,
        Shape::Ring => 3,
        Shape::Burst => 4,
    };
    let modifier_cost: i32 = modifiers.len() as i32;
    let ap_cost = shape_cost + modifier_cost;

    let base_range = match shape {
        Shape::Point => 5,
        Shape::Line => 6,
        Shape::Cone => 4,
        Shape::Ring => 4,
        Shape::Burst => 3,
    };

    Some(Spell {
        elements,
        shape,
        modifiers,
        ap_cost,
        base_range,
        base_damage,
    })
}

/// Get all hex targets for a spell.
pub fn get_targets(spell: &Spell, origin: Hex, target: Hex) -> Vec<Hex> {
    match spell.shape {
        Shape::Point => vec![target],
        Shape::Line => origin.line_to(target),
        Shape::Cone => {
            // Find direction from origin to target
            let direction = find_direction(origin, target);
            hex_cone(origin, direction, spell.range())
        }
        Shape::Ring => ring(target, 2),
        Shape::Burst => range(target, 2),
    }
}

fn find_direction(from: Hex, to: Hex) -> usize {
    use crate::hex::DIRECTIONS;

    let diff = to - from;
    let mut best_dir = 0;
    let mut best_dist = i32::MAX;

    for (i, dir) in DIRECTIONS.iter().enumerate() {
        // Project diff onto direction
        let dist = (diff.x - dir.x).abs() + (diff.y - dir.y).abs() + (diff.z - dir.z).abs();
        if dist < best_dist {
            best_dist = dist;
            best_dir = i;
        }
    }
    best_dir
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_craft_empty_fails() {
        assert!(craft_spell(&[]).is_none());
    }

    #[test]
    fn test_craft_no_element_fails() {
        let runes = vec![Rune::shape(Shape::Line)];
        assert!(craft_spell(&runes).is_none());
    }

    #[test]
    fn test_craft_single_element() {
        let runes = vec![Rune::element(Element::Fire)];
        let spell = craft_spell(&runes).unwrap();
        assert_eq!(spell.elements, vec![Element::Fire]);
        assert_eq!(spell.shape, Shape::Point);
    }

    #[test]
    fn test_craft_with_shape() {
        let runes = vec![Rune::element(Element::Ice), Rune::shape(Shape::Cone)];
        let spell = craft_spell(&runes).unwrap();
        assert_eq!(spell.shape, Shape::Cone);
    }

    #[test]
    fn test_craft_multi_element() {
        let runes = vec![Rune::element(Element::Fire), Rune::element(Element::Ice)];
        let spell = craft_spell(&runes).unwrap();
        assert_eq!(spell.elements.len(), 2);
        assert_eq!(spell.base_damage, 20); // 2 elements * 10
    }

    #[test]
    fn test_power_modifier() {
        let runes = vec![
            Rune::element(Element::Lightning),
            Rune::modifier(Modifier::Power),
        ];
        let spell = craft_spell(&runes).unwrap();
        assert_eq!(spell.damage(), 15); // 10 base + 5 power
    }

    #[test]
    fn test_range_modifier() {
        let runes = vec![
            Rune::element(Element::Void),
            Rune::modifier(Modifier::Range),
        ];
        let spell = craft_spell(&runes).unwrap();
        assert_eq!(spell.range(), 7); // 5 base + 2 range
    }

    #[test]
    fn test_point_targets() {
        let runes = vec![Rune::element(Element::Fire)];
        let spell = craft_spell(&runes).unwrap();
        let targets = get_targets(&spell, Hex::origin(), Hex::new(2, -2, 0));
        assert_eq!(targets.len(), 1);
    }

    #[test]
    fn test_burst_targets() {
        let runes = vec![Rune::element(Element::Fire), Rune::shape(Shape::Burst)];
        let spell = craft_spell(&runes).unwrap();
        let targets = get_targets(&spell, Hex::origin(), Hex::new(2, -2, 0));
        assert!(targets.len() > 1); // Should be filled circle
    }
}
