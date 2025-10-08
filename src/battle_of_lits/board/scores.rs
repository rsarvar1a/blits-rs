
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

        // // CoordSet of neighbours for each covered cell.
        // let sets = self.cover.iter().map(|c| {
        //     self.piecemap.coord_neighbours(&c)
        // });

        // // Fastest possible vectorized union over neighboursets.
        // let mut all_neighbours = CoordSet::union_many(sets);
        
        // // All neighbouring cells with foursquare protection are guaranteed on this board.
        // // This unfortunately misses regions of the board that are genuinely unreachable (not neighbours)
        // // in which a move cannot be played, but the cells themselves are not foursquare.
        // let protected = all_neighbours.difference_inplace(&self.cover).iter().filter(|c| {
        //     self.foursquare_mask.three(c)
        // });

        // // Sum up all protected cells in their respective players' favours.
        // let s_protected = protected.map(|c| {
        //     self.get_unchecked(&c).cell_value().map_or(0, |v| v.perspective())
        // }).sum::<i16>();

        // // We care more about protected cells than about onboard score, since the latter
        // // tends to equalize every pair of moves on a competitive board anyways.
        // 100 * s_protected + s_visible_symbols
    }
}
