
use std::{hash::{DefaultHasher, Hasher}, sync::OnceLock};

use crate::{prelude::{Player, BOARD_SIZE, NUM_PIECES}, battle_of_lits::board::{board_cell::BoardCell, Grid}};

use super::Board;

const NUM_CELLS: usize = BOARD_SIZE * BOARD_SIZE * 3;

static ZOBRIST_CELL_TABLE: OnceLock<[u64; NUM_CELLS]> = OnceLock::new();
static ZOBRIST_MOVE_TABLE: OnceLock<[u64; NUM_PIECES]> = OnceLock::new();

impl<'a> Board<'a> {
    /// Gets the hash for a given Player on a board tile. This hash is _always_ updated as a part of a mutable operation on the grid.
    /// We set it in the indexing method except for during init (where we might have been passed a grid pre-formed) and during swap, where it's
    /// way easier to just negate the cells in-place. 
    pub(super) fn cell_hash(i: usize, j: usize, c: BoardCell) -> u64 {
        let table = ZOBRIST_CELL_TABLE.get_or_init(|| {
            let mut table: [u64; NUM_CELLS] = [0; NUM_CELLS];
            let mut hasher = DefaultHasher::new();
            for (i, entry) in table.iter_mut().enumerate() {
                hasher.write_usize(i + NUM_PIECES);
                *entry = hasher.finish();
            } 
            table
        });
        let offset = c.cell_value().map_or(2, |v| match v { Player::X => 0, _ => 1 });
        table[offset * BOARD_SIZE * BOARD_SIZE + (i * BOARD_SIZE) + j]
    }
    
    /// Instead of hashing the LITS tiles into the hash each move, we can just hash the move, which distinctly identifies a collection of tiles on the board.
    /// We don't use the zobrist to find individual subtiles on the pieces anyways, since that's a needless abstraction.
    pub(super) fn move_hash(&self, mv: usize) -> u64 {
        let table = ZOBRIST_MOVE_TABLE.get_or_init(|| {
            let mut table: [u64; NUM_PIECES] = [0; NUM_PIECES];
            let mut hasher = DefaultHasher::new();
            for (i, entry) in table.iter_mut().enumerate() {
                hasher.write_usize(i);
                *entry = hasher.finish();
            }     
            table
        });
        table[mv]
    }

    /// Given an initial grid, calculates the zobrist hash for the board as if no pieces have been played.
    pub(super) fn initial_zobrist_hash(cells: &Grid) -> u64 {
        let mut h = 0;
        for (i, row) in cells.0.iter().enumerate() {
            for (j, cell) in row.iter().enumerate() {
                h ^= Board::cell_hash(i, j, *cell);
            }
        }
        h
    }
}
