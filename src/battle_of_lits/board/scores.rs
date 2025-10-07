
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
        let s_visible_symbols = {
            self.score()
        };
        s_visible_symbols
        
        // let s_protected = {
        //     let reachable_coords = CoordSet::union_many(self.valid_moves.1.iter().filter(|mv| *mv != NULL_MOVE).map(|mv| self.piecemap.coordset(mv))); // reachable via next move
        //     let unreachable_coords = (!reachable_coords).difference(&self.cover); // not reachable via next move, and not already played

        //     unreachable_coords.iter().filter(|c| { // all unreachable uncovered cells that are actually foursquare-protected
        //         self.foursquare_mask.three(c)
        //     }).map(|c| {
        //         self.get_unchecked(&c).cell_value().map_or(0, |v| v.perspective()) // count for their player on the board
        //     }).sum::<i16>()
        // };

        // 100 * s_protected + s_visible_symbols
    }
}
