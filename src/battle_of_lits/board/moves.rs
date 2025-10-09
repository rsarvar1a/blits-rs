use crate::battle_of_lits::prelude::*;
use crate::battle_of_lits::board::foursquare;

const GAME_LENGTH_LOWER_BOUND: usize = 8;

impl<'a> Board<'a> {
    /// Plays a move onto the board unchecked; engine use only.
    pub(super) fn play_unchecked(&mut self, tetromino: &Tetromino, id: usize) -> () {
        { // played piece mutations
            unsafe {
                *self.piece_bag.get_unchecked_mut(tetromino.kind as usize) -= 1;
            }
            tetromino.real_coords_lazy().for_each(|c| {
                self.set_lits_unchecked(&c.coerce(), Some(tetromino.kind));
            });
        }

        { // amortized state calculations
            self.cover._extend(tetromino.real_coords_lazy().map(|c| c.coerce())); // hoist for vectorization, maybe
            self.neighbours
                .union_inplace(self.piecemap.neighbours(id)) // add all the new neighbours
                .difference_inplace(&self.cover); // remove anything conflicting (either in the new neighbours, or from the just-played piece)

            // Update unreachable cells after piece placement
            self.update_unreachable_cells();

            // Update cached protected cells for movegen and evaluator
            self.protected = self.foursquare_mask.protected_cells();
        }

        { // meta information
            self.zobrist_hash ^= self.move_hash(id); // add the move to the hash
            self.history.push(id);
            self.played.insert(id); // O(1) lookup for future operations
            self.next_player();
        }
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
        self.score = -self.score;
        self.swapped = !self.swapped;
        self.next_player();
    }

    pub(super) fn next_player(&mut self) -> () {
        self.player_to_move = -self.player_to_move;
    }

    pub fn _any_valid_move(&self) -> bool {
        match self.history.len() {
            0..=GAME_LENGTH_LOWER_BOUND => {
                return true;
            }
            _     => { /* check manually */},
        };

        let history: MoveSet = self.history.iter().collect();
        let mut valid_moves: MoveSet = MoveSet::default();

        let adjacents = MoveSet::union_many(
            history.iter() // insert adjacencies to current history
                .map(|p| self.piecemap.with_interaction(p, Interaction::Adjacent))
        );
        valid_moves.union_inplace(&adjacents);

        let conflicts = MoveSet::union_many(
            history.iter() // remove conflicts with current history
                .map(|p| self.piecemap.with_interaction(p, Interaction::Conflicting))
        );
        valid_moves.difference_inplace(&conflicts);

        valid_moves.difference_inplace(&history); // remove played moves

        let protected_uncovered = self.protected.difference(&self.cover);

        valid_moves.iter().any(|candidate| {
            let kind = self.piecemap.get_kind(candidate);
            if unsafe { *self.piece_bag.get_unchecked(kind as usize) == 0 } {       // not even one adjacent piece on board
                return false;
            }

            // we also drop pieces that violate foursquare using protected cell check
            !foursquare::violates(self.piecemap.coordset(candidate), &protected_uncovered)
        })
    }

    pub fn valid_moves_set(&self) -> MoveSet {
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

        let history: MoveSet = self.history.iter().collect();
        let mut valid_moves: MoveSet = MoveSet::default();

        let adjacents = MoveSet::union_many(
            history.iter() // insert adjacencies to current history
                .map(|p| self.piecemap.with_interaction(p, Interaction::Adjacent))
        );
        valid_moves.union_inplace(&adjacents);

        let conflicts = MoveSet::union_many(
            history.iter() // remove conflicts with current history
                .map(|p| self.piecemap.with_interaction(p, Interaction::Conflicting))
        );
        valid_moves.difference_inplace(&conflicts);

        valid_moves.difference_inplace(&history); // remove played moves

        // Compute protected cells once for all candidate moves
        let protected_uncovered = self.protected.difference(&self.cover);

        valid_moves
            .iter().filter(|&p| {
                // we drop pieces not in the bag.
                let kind = self.piecemap.get_kind(p);
                if unsafe { *self.piece_bag.get_unchecked(kind as usize) == 0 } {
                    return false;
                }
                // we also drop pieces that violate foursquare using protected cell check
                !foursquare::violates(self.piecemap.coordset(p), &protected_uncovered)
            }).collect()
    }

    pub fn _compute_valid_moves<T: Extend<usize>>(&self, moves: &mut T) {
        match self.history.len() {
            0 => { 
                moves.extend(0..NUM_PIECES);
                return;
            },
            1 => { 
                let mvs = self.piecemap.with_interaction(self.history[0], Interaction::Adjacent);
                moves.extend(mvs.iter());
                if !self.swapped {
                    moves.extend(Some(NULL_MOVE));
                }
                return;
            },
            _ => { /* don't return; compute properly! */ },
        };

        let history: MoveSet = self.history.iter().collect();
        let mut valid_moves: MoveSet = MoveSet::default();

        let adjacents = MoveSet::union_many(
            history.iter() // insert adjacencies to current history
                .map(|p| self.piecemap.with_interaction(p, Interaction::Adjacent))
        );
        valid_moves.union_inplace(&adjacents);

        let conflicts = MoveSet::union_many(
            history.iter() // remove conflicts with current history
                .map(|p| self.piecemap.with_interaction(p, Interaction::Conflicting))
        );
        valid_moves.difference_inplace(&conflicts);

        valid_moves.difference_inplace(&history); // remove played moves

        let protected_uncovered = self.protected.difference(&self.cover);

        valid_moves
            .iter().filter(|&candidate| {
                let kind = self.piecemap.get_kind(candidate);
                if unsafe { *self.piece_bag.get_unchecked(kind as usize) == 0 } {
                    return false;
                }

                !foursquare::violates(self.piecemap.coordset(candidate), &protected_uncovered)
            }).collect_into(moves);
    }

    pub fn _compute_noisy_moves(&self, moves: &mut Vec<usize>) {
        match self.history.len() {
            0 => { 
                let noisy = (0..NUM_PIECES).filter(|mv| {
                    self.noise(*mv) >= 3
                });
                moves.extend(noisy);
                return;
            },
            1 => { 
                let mvs = self.piecemap
                    .with_interaction(self.history[0], Interaction::Adjacent)
                    .iter().filter(|mv| self.noise(*mv) >= 3);
                moves.extend(mvs);
                if !self.swapped {
                    moves.push(NULL_MOVE);
                }
                return;
            },
            _ => { /* don't return; compute properly! */ },
        };

        let history: MoveSet = self.history.iter().collect();
        let mut valid_moves: MoveSet = MoveSet::default();

        history.iter() // insert adjacencies to current history
            .map(|p| self.piecemap.with_interaction(p, Interaction::Adjacent))
            .for_each(|set| { valid_moves.union_inplace(set); });  
        
        history.iter() // remove conflicts with current history
            .map(|p| self.piecemap.with_interaction(p, Interaction::Conflicting))
            .for_each(|set| { valid_moves.difference_inplace(set); });
        
        valid_moves.difference_inplace(&history); // remove played moves

        let protected_uncovered = self.protected.difference(&self.cover);

        valid_moves
            .iter().filter(|&p| {
                // we drop pieces not in the bag.
                let kind = self.piecemap.get_kind(p);
                if unsafe { *self.piece_bag.get_unchecked(kind as usize) == 0 } {
                    return false;
                }

                if self.noise(p) < 3 {
                    return false;
                }

                // we also drop pieces that violate foursquare using protected cell check
                !foursquare::violates(self.piecemap.coordset(p), &protected_uncovered)
            }).collect_into(moves);
    }
}
