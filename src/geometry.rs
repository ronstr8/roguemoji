//! Grid geometry: positions, directions, lines and rectangles.

/// A cell coordinate on the map grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

impl Pos {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn offset(self, dx: i32, dy: i32) -> Self {
        Self::new(self.x + dx, self.y + dy)
    }

    pub fn step(self, dir: Dir) -> Self {
        let (dx, dy) = dir.delta();
        self.offset(dx, dy)
    }

    /// Chebyshev distance: the number of 8-way moves between two cells.
    pub fn distance(self, other: Pos) -> i32 {
        (self.x - other.x).abs().max((self.y - other.y).abs())
    }

    pub fn neighbors(self) -> impl Iterator<Item = Pos> {
        Dir::ALL.into_iter().map(move |d| self.step(d))
    }
}

/// The eight compass directions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dir {
    N,
    S,
    E,
    W,
    NE,
    NW,
    SE,
    SW,
}

impl Dir {
    pub const ALL: [Dir; 8] = [Dir::N, Dir::S, Dir::E, Dir::W, Dir::NE, Dir::NW, Dir::SE, Dir::SW];

    pub fn delta(self) -> (i32, i32) {
        match self {
            Dir::N => (0, -1),
            Dir::S => (0, 1),
            Dir::E => (1, 0),
            Dir::W => (-1, 0),
            Dir::NE => (1, -1),
            Dir::NW => (-1, -1),
            Dir::SE => (1, 1),
            Dir::SW => (-1, 1),
        }
    }
}

/// Bresenham line from `a` to `b`, inclusive of both ends.
pub fn line(a: Pos, b: Pos) -> Vec<Pos> {
    let mut points = Vec::new();
    let (mut x, mut y) = (a.x, a.y);
    let dx = (b.x - a.x).abs();
    let dy = -(b.y - a.y).abs();
    let sx = if a.x < b.x { 1 } else { -1 };
    let sy = if a.y < b.y { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        points.push(Pos::new(x, y));
        if x == b.x && y == b.y {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
    points
}

/// Axis-aligned rectangle of cells.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h }
    }

    /// True if the rectangles overlap, including a one-cell margin.
    pub fn intersects_padded(&self, other: &Rect) -> bool {
        self.x - 1 <= other.x + other.w
            && self.x + self.w + 1 >= other.x
            && self.y - 1 <= other.y + other.h
            && self.y + self.h + 1 >= other.y
    }

    pub fn cells(&self) -> impl Iterator<Item = Pos> + '_ {
        (self.y..self.y + self.h).flat_map(move |y| (self.x..self.x + self.w).map(move |x| Pos::new(x, y)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_includes_endpoints() {
        let pts = line(Pos::new(0, 0), Pos::new(5, 2));
        assert_eq!(pts.first(), Some(&Pos::new(0, 0)));
        assert_eq!(pts.last(), Some(&Pos::new(5, 2)));
        assert_eq!(pts.len(), 6);
    }

    #[test]
    fn chebyshev_distance() {
        assert_eq!(Pos::new(0, 0).distance(Pos::new(3, -7)), 7);
    }
}
