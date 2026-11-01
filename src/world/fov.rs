//! Field of view by ray casting to the edge of a square radius.

use super::map::Map;
use crate::geometry::{Pos, line};

pub const SIGHT_RADIUS: i32 = 9;

/// Recompute visibility from `origin`, marking cells visible and explored.
pub fn compute(map: &mut Map, origin: Pos, radius: i32) {
    map.clear_visible();
    map.mark_visible(origin);
    let r = radius;
    let mut edge = Vec::with_capacity((8 * r) as usize);
    for i in -r..=r {
        edge.push(origin.offset(i, -r));
        edge.push(origin.offset(i, r));
        edge.push(origin.offset(-r, i));
        edge.push(origin.offset(r, i));
    }
    for target in edge {
        for p in line(origin, target).into_iter().skip(1) {
            if !map.in_bounds(p) {
                break;
            }
            // Round the square into a rough circle.
            let (dx, dy) = (p.x - origin.x, p.y - origin.y);
            if dx * dx + dy * dy > r * r + r {
                break;
            }
            map.mark_visible(p);
            if map.opaque(p) {
                break;
            }
        }
    }
}

/// Clear line of sight/fire between two cells (endpoints excluded).
pub fn clear_line(map: &Map, a: Pos, b: Pos) -> bool {
    let pts = line(a, b);
    pts.iter().skip(1).take(pts.len().saturating_sub(2)).all(|p| !map.opaque(*p))
}
