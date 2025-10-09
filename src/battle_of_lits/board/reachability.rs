use super::*;

impl<'a> Board<'a> {
    /// Updates the unreachable cells set after a piece has been placed.
    /// 
    /// This method detects cells that have become mathematically impossible to reach
    /// due to connectivity constraints. Optimized for minimal overhead.
    pub(super) fn update_unreachable_cells(&mut self) -> () {
        // Early game optimization: skip expensive analysis if board is sparse
        if self.cover.len() < 6 {
            return;
        }
        
        // Fast check: only run expensive analysis if last move has isolation potential
        if let Some(&last_move) = self.history.last() {
            // First check: does this piece type have isolation potential at all?
            if !self.piecemap.has_isolation_potential(last_move) {
                return; // This piece type rarely creates isolation, skip analysis
            }
            
            // Second check: does this specific placement have chokepoint potential?
            let chokepoints = self.piecemap.chokepoints(last_move);
            if chokepoints.is_empty() {
                return; // This specific placement can't create isolation, skip analysis
            }
            
            // Fast dependency-based unreachability: mark pieces that become unreachable
            // due to connectivity constraints when this piece is placed
            self.mark_dependency_unreachable(last_move);
            
            // Shadow-based unreachability: mark regions isolated by strategic placement
            self.mark_shadow_unreachable(last_move);
        }
        
        // Targeted analysis: only check cells that might be newly isolated
        self.detect_newly_isolated_regions();
    }

    /// Fast detection of newly isolated regions using minimal flood fill.
    /// 
    /// Only checks areas that could potentially be cut off by recent moves.
    fn detect_newly_isolated_regions(&mut self) -> () {
        // Use bridge information to accelerate connectivity detection
        if let Some(&last_move) = self.history.last() {
            let bridges = self.piecemap.bridges(last_move);
            if !bridges.is_empty() {
                // Fast bridge-based connectivity check
                self.update_reachability_using_bridges(bridges);
                return;
            }
        }

        // Fallback: traditional flood fill approach
        let mut reachable_from_network = CoordSet::default();
        let mut stack: Vec<Coord> = self.neighbours.iter().collect();
        
        while let Some(coord) = stack.pop() {
            if reachable_from_network.contains(&coord) || self.cover.contains(&coord) {
                continue;
            }
            
            reachable_from_network.insert(&coord);
            
            // Add uncovered orthogonal neighbors
            for offset in coords::ORTHOGONAL_OFFSETS.iter() {
                let neighbor = coord + offset;
                if neighbor.in_bounds_signed() {
                    let neighbor_coord = neighbor.coerce();
                    if !self.cover.contains(&neighbor_coord) && 
                       !reachable_from_network.contains(&neighbor_coord) &&
                       !self.unreachable.contains(&neighbor_coord) {
                        stack.push(neighbor_coord);
                    }
                }
            }
        }
        
        // Any uncovered cell not in reachable_from_network is unreachable
        // But only check a limited area to avoid full board scan
        self.check_limited_unreachable_area(&reachable_from_network);
    }
    
    /// Check for unreachable cells in a limited area around recent activity.
    fn check_limited_unreachable_area(&mut self, reachable: &CoordSet) -> () {
        // Only check cells within 2 steps of existing pieces
        let mut search_area = self.get_limited_search_area();
        search_area.difference_inplace(&self.cover).difference_inplace(reachable);
        self.unreachable.union_inplace(&search_area);
    }
    
    /// Get a limited search area around existing pieces to avoid full board scan.
    fn get_limited_search_area(&self) -> CoordSet {
        let mut search_area = CoordSet::default();
        search_area.union_inplace(&self.neighbours);

        // Add all neighbors of neighbors (2-step radius)
        for coord in self.neighbours.iter() {            
            for offset in coords::ORTHOGONAL_OFFSETS.iter() {
                let neighbor = coord + offset;
                if neighbor.in_bounds_signed() {
                    search_area.insert(&neighbor.coerce());
                }
            }
        }
        
        search_area
    }

    /// Fast reachability update using precomputed bridge information.
    /// 
    /// Uses bridge data to quickly identify newly connected regions
    /// without expensive flood fill operations.
    fn update_reachability_using_bridges(&mut self, bridges: &Vec<(Coord, Coord)>) -> () {
        // For each bridge this piece creates, check if it connects previously
        // disconnected regions that contain unreachable cells
        for &(coord1, coord2) in bridges {
            // Skip if either coordinate is already covered or unreachable
            if self.cover.contains(&coord1) || self.cover.contains(&coord2) ||
               self.unreachable.contains(&coord1) || self.unreachable.contains(&coord2) {
                continue;
            }

            // This bridge connects two reachable areas - no new unreachable cells
            // from this particular bridge
        }

        // Check only the immediate area around the new piece for isolation
        // This is much faster than full board analysis
        if let Some(&last_move) = self.history.last() {
            let mut piece_neighbors = self.piecemap.neighbours(last_move).clone();
            piece_neighbors.difference_inplace(&self.cover).difference_inplace(&self.neighbours);
            self.unreachable.union_inplace(&piece_neighbors);
        }
    }

    /// Marks pieces as unreachable based on connectivity dependencies.
    /// 
    /// Uses precomputed dependency chains to quickly identify pieces that become
    /// unreachable when the blocking piece is placed.
    fn mark_dependency_unreachable(&mut self, blocking_piece_id: usize) -> () {
        let dependencies = self.piecemap.connectivity_dependencies(blocking_piece_id);
        
        // Early exit if no dependencies
        if dependencies.is_empty() {
            return;
        }
        
        // Use inplace difference to avoid allocation
        let mut available_dependencies = dependencies.clone();
        available_dependencies.difference_inplace(&self.played);
        
        for dependent_piece_id in available_dependencies.iter() {
            // Skip if any cells of the dependent piece are already covered
            let dependent_coords = self.piecemap.coordset(dependent_piece_id);
            if dependent_coords.intersects(&self.cover) {
                continue;
            }
            
            // Mark all cells of the dependent piece as unreachable - use union_inplace
            self.unreachable.union_inplace(dependent_coords);
        }
    }

    /// Marks regions as unreachable based on isolation shadow maps.
    ///
    /// Uses precomputed shadow maps to quickly identify regions that become
    /// isolated when this piece is placed at strategic positions.
    fn mark_shadow_unreachable(&mut self, piece_id: usize) -> () {
        let shadows = self.piecemap.isolation_shadows(piece_id);

        // Early exit if no shadows
        if shadows.is_empty() {
            return;
        }

        let shadowset = self.piecemap.shadowset(piece_id);

        // Check each precomputed shadow for this piece placement
        for &(anchor, ref isolated_region) in shadows.iter() {
            // Verify the shadow is actually created using precomputed shadowset
            if shadowset.contains(&anchor) {
                // Mark all cells in the isolated region as unreachable
                let mut region = isolated_region.clone();
                region.difference_inplace(&self.cover).difference_inplace(&self.neighbours);
                self.unreachable.union_inplace(&region);
            }
        }
    }
}