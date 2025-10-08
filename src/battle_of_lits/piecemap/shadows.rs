use super::*;

/// Computes isolation shadow maps for a piece.
/// 
/// Returns a mapping of strategic anchor positions to the regions they would isolate.
/// Shadow maps represent areas that become disconnected from the main network
/// when a piece is placed at specific positions.
pub fn compute_isolation_shadows(piece: &Tetromino, _piece_id: usize) -> Vec<(Coord, CoordSet)> {
    let mut shadows = Vec::new();
    let piece_coords = CoordSet::from_iter(piece.real_coords_lazy().map(|c| c.coerce()));
    
    // Only compute shadows for pieces with isolation potential
    if !has_shadow_potential(piece) {
        return shadows;
    }
    
    // Analyze strategic positions around the piece that could create isolation
    let strategic_positions = get_strategic_shadow_positions(&piece_coords);
    
    for anchor in strategic_positions {
        if let Some(isolated_region) = compute_shadow_region(piece, &piece_coords, &anchor) {
            if isolated_region.len() >= 4 { // Only meaningful shadows (at least one tetromino)
                shadows.push((anchor, isolated_region));
            }
        }
    }
    
    shadows
}

/// Determines if a piece has the potential to create isolation shadows.
fn has_shadow_potential(piece: &Tetromino) -> bool {
    match piece.kind {
        Tile::I => true,  // I-pieces can create linear barriers
        Tile::L => true,  // L-pieces can create corner isolation
        Tile::T => true,  // T-pieces can block multiple directions
        Tile::S => false, // S-pieces rarely create meaningful isolation
    }
}

/// Gets strategic positions around a piece where shadows might be created.
fn get_strategic_shadow_positions(piece_coords: &CoordSet) -> Vec<Coord> {
    let mut positions = Vec::new();
    
    // Check positions adjacent to the piece that could create barriers
    for coord in piece_coords.iter() {
        for offset in coords::ORTHOGONAL_OFFSETS.iter() {
            let candidate = coord + offset;
            if candidate.in_bounds_signed() {
                let candidate_coord = candidate.coerce();
                
                // Only consider positions that could create strategic barriers
                if is_strategic_shadow_position(piece_coords, &candidate_coord) {
                    positions.push(candidate_coord);
                }
            }
        }
    }
    
    positions
}

/// Determines if a position is strategic for creating isolation shadows.
fn is_strategic_shadow_position(piece_coords: &CoordSet, position: &Coord) -> bool {
    // Count how many different directions this position connects to the piece
    let connection_count = coords::ORTHOGONAL_OFFSETS.iter()
        .filter(|&offset| {
            let neighbor = position + offset;
            if neighbor.in_bounds_signed() {
                piece_coords.contains(&neighbor.coerce())
            } else {
                false
            }
        })
        .count();
    
    // Strategic positions connect to the piece from 1-2 directions
    // (creates potential bottleneck)
    connection_count >= 1 && connection_count <= 2
}

/// Computes the isolated region that would be created by placing something at the anchor.
fn compute_shadow_region(_piece: &Tetromino, piece_coords: &CoordSet, anchor: &Coord) -> Option<CoordSet> {
    // Simulate the barrier effect of placing the piece at this strategic position
    let barrier_region = get_barrier_effect_region(piece_coords, anchor);
    
    // Find regions that would be cut off by this barrier
    let isolated_region = find_isolated_region_from_barrier(&barrier_region, anchor);
    
    if isolated_region.len() > 0 {
        Some(isolated_region)
    } else {
        None
    }
}

/// Gets the region that would act as a barrier when combined with the anchor position.
fn get_barrier_effect_region(piece_coords: &CoordSet, anchor: &Coord) -> CoordSet {
    let mut barrier = piece_coords.clone();
    barrier.insert(anchor);
    
    // Extend barrier to include immediate strategic neighbors that strengthen isolation
    let mut extended_barrier = barrier.clone();
    
    for coord in barrier.iter() {
        for offset in coords::ORTHOGONAL_OFFSETS.iter() {
            let neighbor = coord + offset;
            if neighbor.in_bounds_signed() {
                let neighbor_coord = neighbor.coerce();
                
                // Add neighbors that would strengthen the barrier effect
                if strengthens_barrier(&barrier, &neighbor_coord) {
                    extended_barrier.insert(&neighbor_coord);
                }
            }
        }
    }
    
    extended_barrier
}

/// Determines if a position would strengthen the isolation barrier.
fn strengthens_barrier(barrier: &CoordSet, position: &Coord) -> bool {
    // Position strengthens barrier if it connects multiple barrier segments
    let barrier_connections = coords::ORTHOGONAL_OFFSETS.iter()
        .filter(|&offset| {
            let neighbor = position + offset;
            if neighbor.in_bounds_signed() {
                barrier.contains(&neighbor.coerce())
            } else {
                false
            }
        })
        .count();
    
    barrier_connections >= 2
}

/// Finds the region that would be isolated by the given barrier.
fn find_isolated_region_from_barrier(barrier: &CoordSet, anchor: &Coord) -> CoordSet {
    let mut isolated_region = CoordSet::default();
    
    // Start flood fill from positions that might be cut off
    let potential_seeds = get_potential_isolation_seeds(barrier, anchor);
    
    for seed in potential_seeds {
        if isolated_region.contains(&seed) || barrier.contains(&seed) {
            continue;
        }
        
        // Flood fill to find connected component
        let component = flood_fill_component(&seed, barrier);
        
        // Check if this component is isolated (can't reach board edges)
        if is_component_isolated(&component, barrier) {
            for coord in component.iter() {
                isolated_region.insert(&coord);
            }
        }
    }
    
    isolated_region
}

/// Gets potential starting points for isolation detection.
fn get_potential_isolation_seeds(barrier: &CoordSet, _anchor: &Coord) -> Vec<Coord> {
    let mut seeds = Vec::new();
    
    // Check areas adjacent to the barrier that might be cut off
    for coord in barrier.iter() {
        for offset in coords::ORTHOGONAL_OFFSETS.iter() {
            let candidate = coord + offset;
            if candidate.in_bounds_signed() {
                let candidate_coord = candidate.coerce();
                
                if !barrier.contains(&candidate_coord) {
                    seeds.push(candidate_coord);
                }
            }
        }
    }
    
    seeds
}

/// Performs flood fill to find a connected component.
fn flood_fill_component(start: &Coord, barrier: &CoordSet) -> CoordSet {
    let mut component = CoordSet::default();
    let mut stack = vec![*start];
    
    while let Some(coord) = stack.pop() {
        if component.contains(&coord) || barrier.contains(&coord) {
            continue;
        }
        
        component.insert(&coord);
        
        // Add uncovered orthogonal neighbors
        for offset in coords::ORTHOGONAL_OFFSETS.iter() {
            let neighbor = coord + offset;
            if neighbor.in_bounds_signed() {
                let neighbor_coord = neighbor.coerce();
                if !component.contains(&neighbor_coord) && !barrier.contains(&neighbor_coord) {
                    stack.push(neighbor_coord);
                }
            }
        }
    }
    
    component
}

/// Determines if a component is isolated (cannot reach board edges).
fn is_component_isolated(component: &CoordSet, barrier: &CoordSet) -> bool {
    // Simple heuristic: if component contains cells near board edges, it's not isolated
    let has_edge_access = component.iter().any(|coord| {
        coord.row <= 1 || coord.row >= BOARD_SIZE - 2 || 
        coord.col <= 1 || coord.col >= BOARD_SIZE - 2
    });
    
    if has_edge_access {
        return false;
    }
    
    // More sophisticated check: can we reach edges without crossing the barrier?
    let mut can_reach_edge = false;
    let mut visited = CoordSet::default();
    let mut stack = vec![];
    
    // Start from component cells
    for coord in component.iter() {
        stack.push(coord);
    }
    
    while let Some(coord) = stack.pop() {
        if visited.contains(&coord) || barrier.contains(&coord) {
            continue;
        }
        
        visited.insert(&coord);
        
        // Check if we reached a board edge
        if coord.row == 0 || coord.row == BOARD_SIZE - 1 || 
           coord.col == 0 || coord.col == BOARD_SIZE - 1 {
            can_reach_edge = true;
            break;
        }
        
        // Add neighbors for continued exploration
        for offset in coords::ORTHOGONAL_OFFSETS.iter() {
            let neighbor = coord + offset;
            if neighbor.in_bounds_signed() {
                let neighbor_coord = neighbor.coerce();
                if !visited.contains(&neighbor_coord) && !barrier.contains(&neighbor_coord) {
                    stack.push(neighbor_coord);
                }
            }
        }
    }
    
    !can_reach_edge
}