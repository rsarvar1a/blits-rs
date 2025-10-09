use super::BoardCell;
use crate::battle_of_lits::prelude::*;

impl<'a> Board<'a> {
    /// Gets the board cell at a given coordinate.
    pub(super) fn get(&self, coord: &Coord) -> Result<BoardCell> {
        if coord.in_bounds() {
            Ok(self.cells.0[coord.row][coord.col])
        } else {
            Err(anyhow!(
                "invalid coordinate ({:02}, {:02})",
                coord.row,
                coord.col
            ))
        }
    }
}

impl<'a> Board<'a> {
    /// Unchecked cell value in the grid; engine use only.
    pub(super) fn cell_unchecked(&self, coord: &Coord) -> Option<Player> {
        self.get_unchecked(coord).cell_value()
    }

    /// Unchecked lits value in the grid; engine use only.
    pub(super) fn lits_unchecked(&self, coord: &Coord) -> Option<Tile> {
        self.get_unchecked(coord).lits_value()
    }

    #[allow(dead_code)]
    /// Unchecked setting of a cell in the grid; engine use only.
    pub(super) fn set_cell_unchecked(
        &mut self,
        coord: &Coord,
        cell: Option<Player>,
    ) -> &mut Self {
        let [prev, new] = {
            let r = self.get_mut_unchecked(coord);
            let prev = r.clone();
            *r = r.with_cell(cell);
            [prev, r.clone()]
        };
        if self.get_unchecked(coord).lits_value().is_none() {  // add this symbol to the score if the new symbol is uncovered
            self.score += cell.map_or(0, |v| v.perspective()); // if covered, do nothing, because the next operation that uncovers it will fix it
        }
        self.zobrist_hash ^= Board::cell_hash(coord.row, coord.col, prev);
        self.zobrist_hash ^= Board::cell_hash(coord.row, coord.col, new);
        self
    }

    /// Unchecked setting of a LITS tile in the grid; engine use only.
    pub(super) fn set_lits_unchecked(
        &mut self,
        coord: &Coord,
        lits: Option<Tile>,
    ) -> &mut Self {
        let [cur, _prev] = {
            let r = self.get_mut_unchecked(coord);
            let prev = r.lits_value();
            *r = r.with_lits(lits);
            [r.lits_value(), prev]
        };
        self.score += self.get_unchecked(coord).cell_value().map_or(0, |v| v.perspective()) * match lits {
            Some(_) => -1, // setting a tile; remove this symbol from score
            None    =>  1, // unsetting a tile; add this symbol to score
        };
        // we ended up never using the edge counter...
        self.foursquare_mask.update_unchecked(coord, cur);
        self
    }

    /// Unchecked accessor into the grid; engine use only.
    pub(super) fn get_unchecked(&self, coord: &Coord) -> &BoardCell {
        unsafe {
            self.cells.0.get_unchecked(coord.row).get_unchecked(coord.col)
        }
    }

    /// Unchecked mutable reference into the grid; engine use only.
    pub(super) fn get_mut_unchecked(&mut self, coord: &Coord) -> &mut BoardCell {
        unsafe {
            self.cells.0.get_unchecked_mut(coord.row).get_unchecked_mut(coord.col)
        }
    }
}
