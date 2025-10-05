use crate::battle_of_lits::prelude::*;

/// A counter for the foursquare rule; no 2X2 box on the board can be fully populated by tiles.
///
/// We keep track of all 81 foursquares using 3 bits each, using 256 bits.
#[derive(Clone, Copy, Debug, Default)]
pub struct FoursquareCounter([[u8; BOARD_SIZE - 1]; BOARD_SIZE - 1]);

impl FoursquareCounter {
    /// Determines if the given tile violates the foursquare rule.
    pub fn any(&self, coord: &Coord) -> bool {
        self._check_for(coord, 4)
    }

    /// Determines if _placing_ the given tile would violate the foursquare rule.
    pub fn three(&self, coord: &Coord) -> bool {
        self._check_for(coord, 3)
    }

    /// Determines how many tiles are in the foursquare anchored (topleft) at the given coordinate.
    pub fn count(&self, coord: &Coord) -> u8 {
        self.0[coord.row][coord.col]
    }

    /// Updates the foursquare population count on all foursquares touching a given tile.
    pub fn update(&mut self, coord: &Coord, tile: Option<Tile>) -> Result<()> {
        if coord.in_bounds() {
            self.update_unchecked(coord, tile);
            Ok(())
        } else {
            Err(anyhow!(
                "invalid coordinate ({:02}, {:02})",
                coord.row,
                coord.col
            ))
        }
    }

    /// Updates the foursquare population count on all foursquares touching a given tile, unchecked; engine use only.
    ///
    /// We can think of every cell in the grid as belonging to the four squares that have a topleft corner member to
    /// the topleft foursquare. This function checks which of those foursquares are in bounds; for example, when the
    /// coordinate is (0, 0) then the only valid foursquare is the one anchored at 0,0, not (-1, -1), (-1, 0) or (0, -1).
    pub fn update_unchecked(&mut self, coord: &Coord, tile: Option<Tile>) -> () {
        let op = match tile {
            Some(_) => "incr",
            None => "decr",
        };

        coords::ANCHOR_OFFSETS.iter().for_each(|offset| {
            let anchor = coord + offset;
            if anchor.in_foursquare_bounds_signed() {
                let idx = anchor.coerce();
                match op {
                    "incr" => self.incr_inplace(&idx),
                    _ => self.decr_inplace(&idx),
                };
            }
        });
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
    fn incr_inplace(&mut self, coord: &Coord) -> () {
        let new_count = self.count(coord) + 1;
        self.write(coord, new_count);
    }

    /// Decrements the given counter in-place.
    fn decr_inplace(&mut self, coord: &Coord) -> () {
        let new_count = self.count(coord) - 1;
        self.write(coord, new_count);
    }

    fn write(&mut self, coord: &Coord, value: u8) -> () {
        self.0[coord.row][coord.col] = value;
    }
}
