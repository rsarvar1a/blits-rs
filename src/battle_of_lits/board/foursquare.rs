use crate::battle_of_lits::prelude::*;
use std::sync::OnceLock;

/// Precomputed CoordSets for each foursquare anchor position.
/// Each contains the 4 coords that make up that 2x2 square.
static FOURSQUARE_CELLS: OnceLock<Box<[[CoordSet; BOARD_SIZE - 1]; BOARD_SIZE - 1]>> = OnceLock::new();

/// Precomputed list of affected foursquare anchors for each cell on the board.
/// Maps each cell (row, col) to the list of (anchor_row, anchor_col) pairs that need updating.
static AFFECTED_ANCHORS: OnceLock<Box<[[CoordSet; BOARD_SIZE]; BOARD_SIZE]>> = OnceLock::new();

fn init_foursquare_cells() -> Box<[[CoordSet; BOARD_SIZE - 1]; BOARD_SIZE - 1]> {
    let mut cells = Box::new([[CoordSet::default(); BOARD_SIZE - 1]; BOARD_SIZE - 1]);

    for row in 0..(BOARD_SIZE - 1) {
        for col in 0..(BOARD_SIZE - 1) {
            let mut set = CoordSet::default();
            // 2x2 square with top-left at (row, col)
            set.insert(&Coord { row, col });
            set.insert(&Coord { row, col: col + 1 });
            set.insert(&Coord { row: row + 1, col });
            set.insert(&Coord { row: row + 1, col: col + 1 });
            cells[row][col] = set;
        }
    }

    cells
}

fn init_affected_anchors() -> Box<[[CoordSet; BOARD_SIZE]; BOARD_SIZE]> {
    let mut anchors = Box::new([[CoordSet::default(); BOARD_SIZE]; BOARD_SIZE]);

    for row in 0..BOARD_SIZE {
        for col in 0..BOARD_SIZE {
            let coord = Coord { row, col };
            for offset in coords::ANCHOR_OFFSETS.iter() {
                let anchor = &coord + offset;
                if anchor.in_foursquare_bounds_signed() {
                   anchors[row][col].insert(&anchor.coerce());
                }
            }
        }
    }

    anchors
}

/// A counter for the foursquare rule; no 2X2 box on the board can be fully populated by tiles.
///
/// We keep track of all 81 foursquares using 3 bits each, using 256 bits.
#[derive(Clone, Copy, Debug, Default)]
pub struct FoursquareCounter(pub [[u8; BOARD_SIZE - 1]; BOARD_SIZE - 1]);

impl FoursquareCounter {
    /// Determines if _placing_ the given tile would violate the foursquare rule.
    #[inline]
    pub fn three(&self, coord: &Coord) -> bool {
        self._check_for(coord, 3)
    }

    /// Determines how many tiles are in the foursquare anchored (topleft) at the given coordinate.
    #[inline]
    pub fn count(&self, coord: &Coord) -> u8 {
        unsafe { 
            *self.0.get_unchecked(coord.row).get_unchecked(coord.col)
        }
    }

    /// Updates the foursquare population count on all foursquares touching a given tile, unchecked; engine use only.
    ///
    /// We can think of every cell in the grid as belonging to the four squares that have a topleft corner member to
    /// the topleft foursquare. This function checks which of those foursquares are in bounds; for example, when the
    /// coordinate is (0, 0) then the only valid foursquare is the one anchored at 0,0, not (-1, -1), (-1, 0) or (0, -1).
    #[inline]
    pub fn update_unchecked(&mut self, coord: &Coord, tile: Option<Tile>) -> () {
        let delta: i8 = if tile.is_some() { 1 } else { -1 };
        let anchors = AFFECTED_ANCHORS.get_or_init(init_affected_anchors);

        unsafe {
            for Coord { row: anchor_row, col: anchor_col } in anchors.get_unchecked(coord.row).get_unchecked(coord.col).iter() {
                let el = self.0.get_unchecked_mut(anchor_row).get_unchecked_mut(anchor_col);
                *el = (*el as i8 + delta) as u8;
            }
        }
    }

    fn _check_for(&self, coord: &Coord, v: u8) -> bool {
        coords::ANCHOR_OFFSETS.iter().any(|offset| {
            let anchor = coord + offset;
            if anchor.in_foursquare_bounds_signed() {
                let idx = anchor.coerce();
                self.count(&idx) == v
            } else {
                false
            }
        })
    }

    /// Increments the given counter in-place.
    #[inline]
    pub fn incr_inplace(&mut self, coord: &Coord) -> () {
        let el = unsafe { self.0.get_unchecked_mut(coord.row).get_unchecked_mut(coord.col) };
        *el += 1;
    }

    /// Decrements the given counter in-place.
    #[inline]
    pub fn decr_inplace(&mut self, coord: &Coord) -> () {
        let el = unsafe { self.0.get_unchecked_mut(coord.row).get_unchecked_mut(coord.col) };
        *el -= 1;
    }

    /// Returns a CoordSet of all cells that are protected by foursquare-3.
    /// These are cells where placing a tile would complete a foursquare.
    pub fn protected_cells(&self) -> CoordSet {
        let cells = FOURSQUARE_CELLS.get_or_init(init_foursquare_cells);

        // Directly accumulate union instead of allocating Vec
        let mut result = CoordSet::default();

        for row in 0..(BOARD_SIZE - 1) {
            for col in 0..(BOARD_SIZE - 1) {
                if self.0[row][col] >= 3 {
                    result.union_inplace(&cells[row][col]);
                }
            }
        }

        result
    }
}

/// Checks if placing a piece would violate foursquare.
#[inline]
pub fn violates(piece_coords: &CoordSet, protected: &CoordSet) -> bool {
    protected.intersects(piece_coords)
}
