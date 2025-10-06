
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
        let covered_component = self.score();
        // TODO: implement the foursquare scoring component
        // TODO: implement the opportunity scoring component
        let score = covered_component;
        score
    }
}
