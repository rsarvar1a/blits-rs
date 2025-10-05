use crate::battle_of_lits::{prelude::*, tetromino::piecemap::Interaction};

const DEFAULT_ANCHOR: usize = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Move {
    pub tetromino: Tetromino,
    pub score: isize
}

impl<'a> Board<'a> {
    /// Plays a move onto the board unchecked; engine use only.
    pub(super) fn play_unchecked(&mut self, tetromino: &Tetromino, id: usize) -> () {
        self.piece_bag[tetromino.kind as usize] -= 1;
        tetromino.real_coords().iter().for_each(|c| {
            self.set_lits_unchecked(&c.coerce(), Some(tetromino.kind));
        });
        self.zobrist_hash ^= self.move_hash(id); // add the move to the hash
        self.history.push(id);
        self.next_player();

        self._valid_moves_cache[self.history.len()] = Some(self._compute_valid_moves());
    }

    /// Removes a tetromino from the board unchecked; engine use only.
    pub(super) fn undo_unchecked(&mut self, tetromino: &Tetromino, id: usize) -> () {
        self.piece_bag[tetromino.kind as usize] += 1;
        tetromino.real_coords().iter().for_each(|c| {
            self.set_lits_unchecked(&c.coerce(), None);
        });
        self.zobrist_hash ^= self.move_hash(id); // remove the move from the hash
        self.history.pop();
        self.next_player();

        self._valid_moves_cache[self.history.len() + 1] = None;
    }

    /// Swaps the position by:
    /// 1. negating every symbol on the board, and
    /// 2. handing control to the other player
    pub(super) fn swap(&mut self) -> () {
        self.cells.0.iter_mut().enumerate().for_each(|(i, row)| {
            row.iter_mut().enumerate().for_each(|(j, cell)| { 
                self.zobrist_hash ^= Board::cell_hash(i, j, *cell); // remove old cell from the hash
                *cell = cell.negated();
                self.zobrist_hash ^= Board::cell_hash(i, j, *cell); // add new cell to the hash
            });
        });
        self.swapped = !self.swapped;
        self.next_player();

        self._valid_moves_cache[self.history.len()] = Some(self._compute_valid_moves());
    }

    pub(super) fn next_player(&mut self) -> () {
        self.player_to_move = -self.player_to_move;
    }

    pub(super) fn _compute_valid_moves(&self) -> FastSet {
        match self.history.len() {
            0 => { 
                return (0..NUM_PIECES).into_iter().collect(); 
            },
            1 => { 
                let mut mvs = self.piecemap.with_interaction(self.history[0], Interaction::Adjacent).clone();
                if !self.swapped { // need to signal the validity of a pass so the null-move optimization can actually use it
                    mvs.insert(NULL_MOVE); 
                }
                return mvs;
            },
            _ => { /* don't return; compute properly! */ },
        };

        let history: FastSet = self.history.iter().cloned().collect();        

        let initially_valid = {
            if let Some(previously_valid_moves) = &self._valid_moves_cache[self.history.len() - 1] {
                let prev = self.history.last().unwrap();

                let keep_from_prev: FastSet = previously_valid_moves.into_iter().filter(|&mv| {
                    *mv != NULL_MOVE && self.piecemap.get_association(*mv, *prev) != Interaction::Conflicting
                }).collect();
                
                let adjacent_to_prev_and_not_conflicting: FastSet = self.piecemap.with_interaction(*prev, Interaction::Adjacent).into_iter().filter(|&adj| {
                    (!history.contains(adj)) && (!history.iter().any(|&hist| { 
                        self.piecemap.get_association(*adj, hist) == Interaction::Conflicting
                    }))
                }).collect();

                keep_from_prev.union(&adjacent_to_prev_and_not_conflicting)
            } else {            
                let conflicts_with_history: FastSet = history.iter()
                    .flat_map(|&p| self.piecemap.with_interaction(p, Interaction::Conflicting)).collect();
                
                let adjacents_with_history: FastSet = history.iter()
                    .flat_map(|&p| self.piecemap.with_interaction(p, Interaction::Adjacent)).collect();
                
                // we can play any move that is adjacent to the current boardstate but that does not
                // conflict with the boardstate; we also cannot repeat a played move.
                adjacents_with_history.difference(&history.union(&conflicts_with_history))
            }
        };
        initially_valid.into_iter().filter(|&p| {
            // we drop pieces not in the bag.
            let kind = self.piecemap.get_kind(p);
            if self.piece_bag[kind as usize] == 0 {
                return false;
            }
            // we also drop pieces that violate foursquare. to do this, we clone the historical
            // foursquare, simulate the piece, and check all of the refcounts.
            let mut foursquare = self.foursquare_mask.clone();
            let piece = self.piecemap.get_piece(p);
            piece.real_coords().iter().for_each(|c| {
                foursquare.update_unchecked(&c.coerce(), Some(piece.kind));
            });
            !piece.real_coords().iter().any(|c| foursquare.any(&c.coerce()))
        }).collect()
    }
}
