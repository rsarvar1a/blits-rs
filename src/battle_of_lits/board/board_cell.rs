use crate::battle_of_lits::prelude::*;

/// A cell on a LITS board.
/// bits:
///     [00, 01]: LITS value
///     [02, 02]: occupied by tile
///     [03, 03]: XO value
///     [04, 04]: occupied by scorer
#[derive(Clone, Copy, Debug, Default)]
pub struct BoardCell(u8);

impl BoardCell {
    const LITS_VALUE_OFFSET: usize = 0x00;
    const LITS_VALUE_EXTENT: usize = 0b11; // L I T S
    const LITS_PRESENCE_OFFSET: usize = 0x02;
    const LITS_PRESENCE_EXTENT: usize = 0b01; // Some None
    const CELL_VALUE_OFFSET: usize = 0x03;
    const CELL_VALUE_EXTENT: usize = 0b01; // X O
    const CELL_PRESENCE_OFFSET: usize = 0x04;
    const CELL_PRESENCE_EXTENT: usize = 0b01; // Some None

    /// Determines whether or not there is a scoring symbol in this cell.
    pub fn covered(&self) -> bool {
        let v = self._extract(
            BoardCell::LITS_PRESENCE_OFFSET,
            BoardCell::LITS_PRESENCE_EXTENT,
        );
        v == 1
    }

    /// Determines the scorer at this cell, if any.
    pub fn cell_value(&self) -> Option<Player> {
        if self._cell_present() {
            let v = self._extract(BoardCell::CELL_VALUE_OFFSET, BoardCell::CELL_VALUE_EXTENT);
            Some(Player::from(v))
        } else {
            None
        }
    }

    /// Determines the tile covering this cell, if any.
    pub fn lits_value(&self) -> Option<Tile> {
        if self.covered() {
            let v = self._extract(BoardCell::LITS_VALUE_OFFSET, BoardCell::LITS_VALUE_EXTENT);
            Some(Tile::from(v))
        } else {
            None
        }
    }

    /// Produces a new board cell with the player negated. This operation is extremely useful for swap.
    pub fn negated(&self) -> BoardCell {
        let Some(v) = self.cell_value() else {
            return *self;
        };
        self.with_cell(Some(-v))
    }

    /// Produces a new board cell with the given cell value.
    pub fn with_cell(&self, cell: Option<Player>) -> BoardCell {
        if let Some(value) = cell {
            self._with(
                BoardCell::CELL_PRESENCE_OFFSET,
                BoardCell::CELL_PRESENCE_EXTENT,
                1,
            )
            ._with(
                BoardCell::CELL_VALUE_OFFSET,
                BoardCell::CELL_VALUE_EXTENT,
                value as u8,
            )
        } else {
            self._with(
                BoardCell::CELL_PRESENCE_OFFSET,
                BoardCell::CELL_PRESENCE_EXTENT,
                0,
            )
        }
    }

    /// Produces a new board cell with the given cell value.
    pub fn with_lits(&self, lits: Option<Tile>) -> BoardCell {
        if let Some(value) = lits {
            self._with(
                BoardCell::LITS_PRESENCE_OFFSET,
                BoardCell::LITS_PRESENCE_EXTENT,
                1,
            )
            ._with(
                BoardCell::LITS_VALUE_OFFSET,
                BoardCell::LITS_VALUE_EXTENT,
                value as u8,
            )
        } else {
            self._with(
                BoardCell::LITS_PRESENCE_OFFSET,
                BoardCell::LITS_PRESENCE_EXTENT,
                0,
            )
        }
    }

    /// Determines whether or not there is a cell value in this cell.
    fn _cell_present(&self) -> bool {
        let v = self._extract(
            BoardCell::CELL_PRESENCE_OFFSET,
            BoardCell::CELL_PRESENCE_EXTENT,
        );
        v == 1
    }

    /// Produces the value stored in the bits corresponding to a given offset and extent.
    fn _extract(&self, offset: usize, extent: usize) -> u8 {
        (self.0 >> offset) & extent as u8
    }

    /// Produces a new BoardCell with the given value placed into the bits corresponding to the given offset and extent.
    fn _with(&self, offset: usize, extent: usize, value: u8) -> BoardCell {
        let mask: u8 = (extent << offset) as u8; // e.g. if value is 0b11 at 0x01, then this is 0b00000110
        let antimask = !mask; // then this antimask is 0b11111001, so keep everything but the value
        let v = (value << offset) & mask; // then move value to the region 0b00000110
        BoardCell((self.0 & antimask) | v) // tl;dr: mask out the value being replaced, then include the shifted bits of the new value
    }
}

impl std::fmt::Display for BoardCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.covered() {
            write!(f, "{}", self.lits_value().unwrap())
        } else {
            write!(f, "{}", Player::repr(self.cell_value()))
        }
    }
}
