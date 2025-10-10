
use super::*;

impl<'a> Board<'a> {
    /// The heuristic score on the board from X's perspective.
    /// This heuristic takes into account:
    /// 1. the uncovered scoring tiles protected by foursquare
    ///   - (these are "earned" points in that player's favour)
    /// 2. the number of scoring tiles in the immediately reachable set
    ///   - (these are "earnable" points in the opposite player's favour, at a reduced rate)
    ///   - we are basically rewarding a player if they have a breadth of choice in their attack 
    pub(super) fn _true_effective_score(&self) -> i16 {
        self._true_effective_score_impl()
    }
    
    #[allow(dead_code)]
    /// Moving to an impl so I can toggle on/off without commenting out the code.
    pub(super) fn _true_effective_score_impl(&self) -> i16 {
        let material = self.score();
        let current_player = self.player_to_move();

        let mut unreachable_score = 0i16;
        let mut security = 0i16;
        let mut threat = 0i16;
        let mut connectivity = 0i16;
        let mut constraint = 0i16;

        // Unreachable tiles implicated in scoring.
        let unreachable_symbols = self.unreachable.intersect(&self.symbols);

        // Protected by foursquare, and not covered by a piece.
        let protected_uncovered = self.protected.difference(&self.cover);

        // Uncovered neighbours to played pieces that are implicated in scoring.
        let neighbour_symbols = self.neighbours.intersect(&self.symbols);

        for coord in unreachable_symbols.iter() {
            let player = self.get_unchecked(&coord).cell_value().unwrap();
            unreachable_score += player.perspective();
        }

        for coord in neighbour_symbols.iter() {
            let is_protected = protected_uncovered.contains(&coord);
            let player = self.get_unchecked(&coord).cell_value().unwrap();
            let value = player.perspective();

            if is_protected {
                security += value;
                constraint += 1;
            } else if player != current_player {
                threat += current_player.perspective();
            }

            if player == current_player {
                connectivity += current_player.perspective();
            }
        }

        material +
         50 * unreachable_score +
         25 * security +
        -15 * threat +
         10 * connectivity +
         -5 * constraint
    }

}
