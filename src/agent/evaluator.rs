use super::LITSGame;

#[derive(Clone, Copy, Debug, Default)]
/// The BLITS evaluator for nonterminal states.
pub struct Evaluator;

impl minimax::Evaluator for Evaluator {
    type G = LITSGame;

    fn evaluate(&self, state: &<Self::G as minimax::Game>::S) -> minimax::Evaluation {
        state.score() * state.player_to_move().perspective()
    }

    fn generate_noisy_moves(
        &self, state: &<Self::G as minimax::Game>::S, moves: &mut Vec<<Self::G as minimax::Game>::M>,
    ) {
        state.noisy_moves(moves);
    }
}
