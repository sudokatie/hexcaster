//! Rune types for spell crafting.

/// Elemental damage types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Element {
    Fire,
    Ice,
    Lightning,
    Earth,
    Void,
    Light,
    Dark,
    Poison,
}

impl Element {
    pub fn glyph(&self) -> char {
        match self {
            Element::Fire => '🔥',
            Element::Ice => '❄',
            Element::Lightning => '⚡',
            Element::Earth => '🗿',
            Element::Void => '◯',
            Element::Light => '☀',
            Element::Dark => '🌑',
            Element::Poison => '☠',
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Element::Fire => "Fire",
            Element::Ice => "Ice",
            Element::Lightning => "Lightning",
            Element::Earth => "Earth",
            Element::Void => "Void",
            Element::Light => "Light",
            Element::Dark => "Dark",
            Element::Poison => "Poison",
        }
    }

    /// Get the elemental opposition (for resistance/weakness).
    pub fn opposed(&self) -> Option<Element> {
        match self {
            Element::Fire => Some(Element::Ice),
            Element::Ice => Some(Element::Fire),
            Element::Lightning => Some(Element::Earth),
            Element::Earth => Some(Element::Lightning),
            Element::Light => Some(Element::Dark),
            Element::Dark => Some(Element::Light),
            _ => None,
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
    Wall,  // Line perpendicular to aim direction
    Chain, // Jumps between nearby targets
    Nova,  // Ring around caster
}

impl Shape {
    pub fn name(&self) -> &'static str {
        match self {
            Shape::Point => "Point",
            Shape::Line => "Line",
            Shape::Cone => "Cone",
            Shape::Ring => "Ring",
            Shape::Burst => "Burst",
            Shape::Wall => "Wall",
            Shape::Chain => "Chain",
            Shape::Nova => "Nova",
        }
    }

    /// Get the base range for this shape.
    pub fn base_range(&self) -> i32 {
        match self {
            Shape::Point => 5,
            Shape::Line => 4,
            Shape::Cone => 3,
            Shape::Ring => 4,
            Shape::Burst => 3,
            Shape::Wall => 4,
            Shape::Chain => 6,
            Shape::Nova => 0, // centered on caster
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
    Homing,   // Seeks targets
    Piercing, // Ignores armor/obstacles
    Vampiric, // Heals caster
    Silent,   // No aggro generation
}

impl Modifier {
    pub fn name(&self) -> &'static str {
        match self {
            Modifier::Power => "Power",
            Modifier::Range => "Range",
            Modifier::Duration => "Duration",
            Modifier::Split => "Split",
            Modifier::Echo => "Echo",
            Modifier::Homing => "Homing",
            Modifier::Piercing => "Piercing",
            Modifier::Vampiric => "Vampiric",
            Modifier::Silent => "Silent",
        }
    }

    /// Get the mana cost multiplier for this modifier.
    pub fn cost_multiplier(&self) -> f32 {
        match self {
            Modifier::Power => 1.5,
            Modifier::Range => 1.2,
            Modifier::Duration => 1.3,
            Modifier::Split => 1.8,
            Modifier::Echo => 2.0,
            Modifier::Homing => 1.4,
            Modifier::Piercing => 1.6,
            Modifier::Vampiric => 1.7,
            Modifier::Silent => 1.1,
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
