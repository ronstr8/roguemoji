//! The physical world: terrain, features, visibility and generation.

pub mod feature;
pub mod fov;
pub mod generator;
pub mod map;
pub mod tile;

pub use feature::{Feature, Harvestable};
pub use generator::Biome;
pub use map::Map;
pub use tile::Tile;
