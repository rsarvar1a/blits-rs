
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
        
        // Calculate unreachable symbols (guaranteed points)
        // Now enhanced with dependency chains and shadow map detection
        let unreachable_score = self.unreachable.iter()
            .map(|coord| {
                self.get_unchecked(&coord).cell_value().map_or(0, |player| player.perspective())
            })
            .sum::<i16>();
        
        // Calculate potential unreachable regions using piecemap optimizations
        // REMOVED: Was consuming 80% of runtime for 10x performance drop
        let potential_unreachable_score = 0;
        
        let (security, threat, connectivity, constraint) = self.neighbours.iter()
            .map(|coord| {
                let is_protected = self.foursquare_mask.three(&coord);
                let cell_value = self.get_unchecked(&coord).cell_value();
                
                let security_contrib = if is_protected {
                    cell_value.map_or(0, |player| player.perspective())
                } else { 0 };
                
                let threat_contrib = if !is_protected {
                    cell_value.map_or(0, |player| {
                        if player == current_player {
                            0
                        } else {
                            current_player.perspective()
                        }
                    })
                } else { 0 };
                
                let connectivity_contrib = cell_value.map_or(0, |player| {
                    if player == current_player {
                        current_player.perspective()
                    } else { 0 }
                });
                
                let constraint_contrib = if is_protected { 1 } else { 0 };
                
                (security_contrib, threat_contrib, connectivity_contrib, constraint_contrib)
            })
            .fold((0i16, 0i16, 0i16, 0i16), |(s_acc, t_acc, c_acc, ct_acc), (s, t, c, ct)| {
                (s_acc + s, t_acc + t, c_acc + c, ct_acc + ct)
            });
        
        let diversity = {
            let mean = self.piece_bag.iter().sum::<usize>() as f32 / 4.0;
            let variance = self.piece_bag.iter()
                .map(|&count| (count as f32 - mean).powi(2))
                .sum::<f32>() / 4.0;
            -(variance as i16)
        };

        material + 
        500 * unreachable_score +     // Highest weight - these are guaranteed points
        50 * potential_unreachable_score + // Medium weight - likely future unreachable regions
        100 * security + 
        -25 * threat + 
        15 * connectivity + 
        -10 * constraint + 
        5 * diversity
    }

}
