use crate::battle_of_lits::prelude::*;

/// Simple board coordinate; realistically bounded to 10x10.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Coord {
    pub row: usize,
    pub col: usize,
}

impl std::str::FromStr for Coord {
    type Err = Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if s.len() != 2 {
            return Err(anyhow!("expected (0-padded) 2 digit number for Coord; received {s}"));
        }
        let [row, col] = [0, 1]
            .map(|i| s.chars().nth(i).unwrap())
            .map(|x| x.to_string().parse::<usize>());
        let [row, col] = [row?, col?];
        Ok(Coord { row, col })
    }
}

impl Coord {
    /// Determines whether or not the coord is in bounds.
    pub fn in_bounds(&self) -> bool {
        self.row < 10 && self.col < 10
    }

    /// Constructs a new coord.
    pub fn new(row: usize, col: usize) -> Coord {
        Coord { row, col }
    }

    /// The canonical notation of the coord is its linear offset in the grid.
    pub fn notate(&self) -> String {
        format!("{}{}", self.row, self.col)
    }

    /// Gets the squared distance between the two coords.
    pub fn squared_distance(&self, other: &Coord) -> usize {
        let [lhs, rhs] = [OffsetCoord::from(self), OffsetCoord::from(other)];
        let distance = (lhs.rows - rhs.rows).pow(2) + (lhs.cols - rhs.cols).pow(2);
        distance as usize
    }
}

// Simple offset pair that can be used to calculate neighbours.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OffsetCoord {
    pub rows: isize,
    pub cols: isize,
}

/// Offsets that turn a coordinate into one of its orthogonal neighbours.
pub static ORTHOGONAL_OFFSETS: [OffsetCoord; 4] = [
    OffsetCoord { rows: -1, cols: 0 },
    OffsetCoord { rows: 0, cols: -1 },
    OffsetCoord { rows: 0, cols: 1 },
    OffsetCoord { rows: 1, cols: 0 },
];

/// Offsets that denote the topleft anchors of all 2x2 bounding boxes that include a coordinate.
pub static ANCHOR_OFFSETS: [OffsetCoord; 4] = [
    OffsetCoord { rows: -1, cols: -1 },
    OffsetCoord { rows: -1, cols: 0 },
    OffsetCoord { rows: 0, cols: -1 },
    OffsetCoord { rows: 0, cols: 0 },
];

impl OffsetCoord {
    /// Coerces the offset into a coordinate unchecked.
    pub fn coerce(&self) -> Coord {
        Coord {
            row: self.rows as usize,
            col: self.cols as usize,
        }
    }

    /// If the coord is a top-left anchor on a foursquare.
    pub fn in_foursquare_bounds_signed(&self) -> bool {
        0 <= self.rows && self.rows < 9 && 0 <= self.cols && self.cols < 9
    }

    /// Determines whether or not the coord is in bounds.
    pub fn in_bounds_signed(&self) -> bool {
        0 <= self.rows && self.rows < 10 && 0 <= self.cols && self.cols < 10
    }

    /// The taxicab distance between two points.
    pub fn manhattan(&self, other: OffsetCoord) -> usize {
        self.rows.abs_diff(other.rows) + self.cols.abs_diff(other.cols)
    }

    // Whether two coordinates are neighbours.
    pub fn neighbours(&self, other: OffsetCoord) -> bool {
        self.manhattan(other) == 1
    }

    /// Constructs a new offset coord.
    pub fn new(rows: isize, cols: isize) -> OffsetCoord {
        OffsetCoord { rows, cols }
    }
}

// C -> OC

impl From<Coord> for OffsetCoord {
    fn from(value: Coord) -> Self {
        OffsetCoord {
            rows: value.row as isize,
            cols: value.col as isize,
        }
    }
}

impl From<&Coord> for OffsetCoord {
    fn from(value: &Coord) -> Self {
        OffsetCoord {
            rows: value.row as isize,
            cols: value.col as isize,
        }
    }
}

// OC + OC

impl Add<&OffsetCoord> for &OffsetCoord {
    type Output = OffsetCoord;
    fn add(self, rhs: &OffsetCoord) -> Self::Output {
        OffsetCoord {
            rows: self.rows + rhs.rows,
            cols: self.cols + rhs.cols,
        }
    }
}

impl Add<OffsetCoord> for &OffsetCoord {
    type Output = OffsetCoord;
    fn add(self, rhs: OffsetCoord) -> Self::Output {
        self + &rhs
    }
}

impl Add<&OffsetCoord> for OffsetCoord {
    type Output = OffsetCoord;
    fn add(self, rhs: &OffsetCoord) -> Self::Output {
        &self + rhs
    }
}

impl Add<OffsetCoord> for OffsetCoord {
    type Output = OffsetCoord;
    fn add(self, rhs: OffsetCoord) -> Self::Output {
        &self + &rhs
    }
}

// C + OC

impl Add<&OffsetCoord> for &Coord {
    type Output = OffsetCoord;
    fn add(self, rhs: &OffsetCoord) -> Self::Output {
        OffsetCoord::from(self) + rhs
    }
}

impl Add<OffsetCoord> for &Coord {
    type Output = OffsetCoord;
    fn add(self, rhs: OffsetCoord) -> Self::Output {
        self + &rhs
    }
}

impl Add<&OffsetCoord> for Coord {
    type Output = OffsetCoord;
    fn add(self, rhs: &OffsetCoord) -> Self::Output {
        &self + rhs
    }
}

impl Add<OffsetCoord> for Coord {
    type Output = OffsetCoord;
    fn add(self, rhs: OffsetCoord) -> Self::Output {
        &self + &rhs
    }
}

// C - C

impl Sub<&Coord> for &Coord {
    type Output = OffsetCoord;
    fn sub(self, rhs: &Coord) -> Self::Output {
        OffsetCoord::from(self) - OffsetCoord::from(rhs)
    }
}

impl Sub<Coord> for &Coord {
    type Output = OffsetCoord;
    fn sub(self, rhs: Coord) -> Self::Output {
        self - &rhs
    }
}

impl Sub<&Coord> for Coord {
    type Output = OffsetCoord;
    fn sub(self, rhs: &Coord) -> Self::Output {
        &self - rhs
    }
}

impl Sub<Coord> for Coord {
    type Output = OffsetCoord;
    fn sub(self, rhs: Coord) -> Self::Output {
        &self - &rhs
    }
}

// OC - OC

impl Sub<&OffsetCoord> for &OffsetCoord {
    type Output = OffsetCoord;
    fn sub(self, rhs: &OffsetCoord) -> Self::Output {
        OffsetCoord {
            rows: self.rows - rhs.rows,
            cols: self.cols - rhs.cols,
        }
    }
}

impl Sub<OffsetCoord> for &OffsetCoord {
    type Output = OffsetCoord;
    fn sub(self, rhs: OffsetCoord) -> Self::Output {
        self - &rhs
    }
}

impl Sub<OffsetCoord> for OffsetCoord {
    type Output = OffsetCoord;
    fn sub(self, rhs: OffsetCoord) -> Self::Output {
        &self - &rhs
    }
}

impl Sub<&OffsetCoord> for OffsetCoord {
    type Output = OffsetCoord;
    fn sub(self, rhs: &OffsetCoord) -> Self::Output {
        &self - rhs
    }
}
