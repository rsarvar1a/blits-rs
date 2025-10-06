use crate::battle_of_lits::prelude::*;

/// A reference counter for a cell on a LITS board. Each u8 is a quartet of three-bit pairs corresponding to each type of tile.
///
/// Each LITSEdgeCount keeps the number of tiles (0 to 4, requiring 3 bits each) of each type bordering the reference tile.
#[derive(Clone, Copy, Debug, Default)]
pub struct LITSEdgeCount(u16);

impl LITSEdgeCount {
    const OFFSET: u16 = 0x03;
    const EXTENT: u16 = 0b0000000000000111;

    /// Returns the count for the given tile at this cell.
    pub fn count(&self, tile: Tile) -> u8 {
        ((self.0 >> (tile as u16 * LITSEdgeCount::OFFSET)) & LITSEdgeCount::EXTENT) as u8
    }

    /// Increments the given counter in-place.
    pub fn incr_inplace(&mut self, tile: Tile) -> &mut Self {
        let new_count = self.count(tile) + 1;
        self.update(tile, new_count);
        self
    }

    /// Decrements the given counter in-place.
    pub fn decr_inplace(&mut self, tile: Tile) -> &mut Self {
        let new_count = self.count(tile) - 1;
        self.update(tile, new_count);
        self
    }

    fn update(&mut self, tile: Tile, value: u8) -> () {
        let shift: u16 = tile as u16 * LITSEdgeCount::OFFSET;
        let mask: u16 = LITSEdgeCount::EXTENT << shift;
        let antimask: u16 = !mask;
        let v: u16 = ((value as u16) << shift) & mask;
        self.0 = (self.0 & antimask) | v;
    }
}

/// An independently-copyable edge counter for a full board of LITS.
///
/// We keep track of each counter with a u16, using 1600 bits.
#[derive(Clone, Copy, Debug, Default)]
pub struct EdgeCounter {
    counters: [[LITSEdgeCount; 10]; 10],
}

impl EdgeCounter {
    /// Updates the tile counts on the neighbours of a given tile unchecked; engine use only.
    pub fn update_unchecked(
        &mut self,
        coord: &Coord,
        tile: Option<Tile>,
        prev: Option<Tile>,
    ) -> () {
        // If the new tile is Some, we are just updating an empty square on the grid to have a tile, so there is just a
        // new value to increment, and there is  never a previous value to decrement.
        //
        // If the new tile is None, then we are just removing a tile from an square on the grid, so there is never a
        // new value to increment, but there is a previous value to decrement.
        let (v, op) = match tile {
            Some(t) => (t, "incr"),
            None => (prev.unwrap(), "decr"),
        };

        coords::ORTHOGONAL_OFFSETS.iter().for_each(|offset| {
            let candidate = coord + offset;
            if candidate.in_bounds_signed() {
                let idx = candidate.coerce();
                match op {
                    "incr" => self.counters[idx.row][idx.col].incr_inplace(v),
                    _ => self.counters[idx.row][idx.col].decr_inplace(v),
                };
            }
        })
    }
}
