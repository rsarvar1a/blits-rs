
use crate::battle_of_lits::prelude::*;

pub struct LITSGame;

impl minimax::Game for LITSGame {
    type S = Board<'static>;
    type M = usize;

    fn apply(state: &mut Self::S, m: Self::M) -> Option<Self::S> {
        let mut state = state.clone();
        if m == NULL_MOVE {
            state.pass_unchecked_engine();
        } else {
            state.play_unchecked_engine(m);
        }
        Some(state)
    }

    fn generate_moves(state: &Self::S, moves: &mut Vec<Self::M>) {

        state._compute_valid_moves(moves);
    }

    fn get_winner(state: &Self::S) -> Option<minimax::Winner> {
        if !state.is_terminal() {
            return None; 
        }

        let score = state.score() * state.player_to_move().perspective();
        let winner = match score.signum() {
             1 => minimax::Winner::PlayerToMove,
            -1 => minimax::Winner::PlayerJustMoved,
             0 => minimax::Winner::Draw,
             _ => unreachable!()
        };
        Some(winner)
    }

    fn null_move(state: &Self::S) -> Option<Self::M> {
        if state.can_swap() {
            Some(NULL_MOVE)
        } else {
            None
        }
    }

    fn max_table_index() -> u16 {
        (NUM_PIECES - 1) as u16
    }

    fn notation(state: &Self::S, mv: Self::M) -> Option<String> {
        Some(state.piecemap.notate(mv))
    }

    fn table_index(m: Self::M) -> u16 {
        m as u16
    }

    fn zobrist_hash(state: &Self::S) -> u64 {
        state.zobrist()
    }
}
