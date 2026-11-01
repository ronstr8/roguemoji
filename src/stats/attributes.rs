//! The six core attributes, grouped as physical/mental pairs.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Attribute {
    Brawn,
    Brains,
    Reflexes,
    Wit,
    Fortitude,
    Willpower,
}

/// The baseline score; anything above grants a bonus, below a penalty.
pub const AVERAGE: i32 = 5;
pub const MIN_SCORE: i32 = 1;
pub const MAX_SCORE: i32 = 12;

impl Attribute {
    pub const ALL: [Attribute; 6] = [
        Attribute::Brawn,
        Attribute::Brains,
        Attribute::Reflexes,
        Attribute::Wit,
        Attribute::Fortitude,
        Attribute::Willpower,
    ];

    /// Physical/mental pairs, in display order.
    pub const PAIRS: [(Attribute, Attribute); 3] = [
        (Attribute::Brawn, Attribute::Brains),
        (Attribute::Reflexes, Attribute::Wit),
        (Attribute::Fortitude, Attribute::Willpower),
    ];

    pub fn name(self) -> &'static str {
        match self {
            Attribute::Brawn => "Brawn",
            Attribute::Brains => "Brains",
            Attribute::Reflexes => "Reflexes",
            Attribute::Wit => "Wit",
            Attribute::Fortitude => "Fortitude",
            Attribute::Willpower => "Willpower",
        }
    }

    pub fn abbrev(self) -> &'static str {
        match self {
            Attribute::Brawn => "BRN",
            Attribute::Brains => "BRA",
            Attribute::Reflexes => "REF",
            Attribute::Wit => "WIT",
            Attribute::Fortitude => "FOR",
            Attribute::Willpower => "WIL",
        }
    }

    pub fn is_physical(self) -> bool {
        matches!(self, Attribute::Brawn | Attribute::Reflexes | Attribute::Fortitude)
    }

    fn index(self) -> usize {
        self as usize
    }
}

/// Raw bonus granted by an attribute score.
pub fn bonus(score: i32) -> i32 {
    score - AVERAGE
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attributes {
    scores: [i32; 6],
}

impl Attributes {
    pub fn new(brawn: i32, brains: i32, reflexes: i32, wit: i32, fortitude: i32, willpower: i32) -> Self {
        Self { scores: [brawn, brains, reflexes, wit, fortitude, willpower] }
    }

    pub fn get(&self, a: Attribute) -> i32 {
        self.scores[a.index()]
    }
}
