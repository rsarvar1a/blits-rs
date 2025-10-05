use super::LITSGame;
use crate::utils::prelude::*;

#[derive(Clone, Copy, Debug, Default)]
/// The BLITS evaluator for nonterminal states.
pub struct Evaluator;

impl minimax::Evaluator for Evaluator {
    type G = LITSGame;

    fn evaluate(&self, state: &<Self::G as minimax::Game>::S) -> minimax::Evaluation {
        state.effective_score()
    }

    fn generate_noisy_moves(
            &self, state: &<Self::G as minimax::Game>::S, moves: &mut Vec<<Self::G as minimax::Game>::M>,
        ) {
        let mvs: FastSet = state.noisy_moves();
        moves.reserve(mvs.len());
        moves.extend(mvs.iter());
    }
}
