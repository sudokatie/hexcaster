//! Rune types for spell crafting.

/// Elemental damage types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Element {
    Fire,
    Ice,
    Lightning,
    Earth,
    Void,
}

impl Element {
    pub fn glyph(&self) -> char {
        match self {
            Element::Fire => '🔥',
            Element::Ice => '❄',
            Element::Lightning => '⚡',
            Element::Earth => '🗿',
            Element::Void => '◯',
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Element::Fire => "Fire",
            Element::Ice => "Ice",
            Element::Lightning => "Lightning",
            Element::Earth => "Earth",
            Element::Void => "Void",
        }
    }
}

/// Spell shapes (area of effect).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    Point, // Single target
    Line,  // Line from caster
    Cone,  // Cone in direction
    Ring,  // Circle around target
    Burst, // Filled circle around target
}

impl Shape {
    pub fn name(&self) -> &'static str {
        match self {
            Shape::Point => "Point",
            Shape::Line => "Line",
            Shape::Cone => "Cone",
            Shape::Ring => "Ring",
            Shape::Burst => "Burst",
        }
    }
}

/// Spell modifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modifier {
    Power,    // +damage
    Range,    // +range
    Duration, // +status duration
    Split,    // Multiple projectiles
    Echo,     // Cast twice
}

impl Modifier {
    pub fn name(&self) -> &'static str {
        match self {
            Modifier::Power => "Power",
            Modifier::Range => "Range",
            Modifier::Duration => "Duration",
            Modifier::Split => "Split",
            Modifier::Echo => "Echo",
        }
    }
}

/// A rune that can be combined into spells.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rune {
    pub kind: RuneKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuneKind {
    Element(Element),
    Shape(Shape),
    Modifier(Modifier),
}

impl Rune {
    pub fn element(e: Element) -> Self {
        Self {
            kind: RuneKind::Element(e),
        }
    }

    pub fn shape(s: Shape) -> Self {
        Self {
            kind: RuneKind::Shape(s),
        }
    }

    pub fn modifier(m: Modifier) -> Self {
        Self {
            kind: RuneKind::Modifier(m),
        }
    }

    pub fn name(&self) -> &'static str {
        match &self.kind {
            RuneKind::Element(e) => e.name(),
            RuneKind::Shape(s) => s.name(),
            RuneKind::Modifier(m) => m.name(),
        }
    }
}
