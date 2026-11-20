//! The physical world: terrain, features, visibility and generation.

pub mod encounter;
pub mod feature;
pub mod fov;
pub mod generator;
pub mod harvest;
pub mod map;
pub mod tile;
pub mod wager;

pub use feature::{Cover, Feature};
pub use generator::Biome;
pub use harvest::Harvestable;
pub use map::Map;
pub use tile::Tile;
