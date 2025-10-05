use crate::battle_of_lits::prelude::*;

/// A bitboard for a 10x10 board.
#[derive(Clone, Copy, Debug)]
pub struct Mask([[bool; 10]; 10]);

impl<'a> Board<'a> {
    /// Gets the mask of all covered cells on the board, helpful for scoring.
    pub(super) fn _covered(&self) -> Mask {
        let data = self.cells.0.map(|row| row.map(|v| v.covered()));
        Mask(data)
    }
}
