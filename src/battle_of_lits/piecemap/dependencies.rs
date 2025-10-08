use super::*;

/// Computes connectivity-based dependencies for a piece.
/// 
/// Returns a MoveSet of pieces that become unreachable due to connectivity
/// constraints when this piece is placed (beyond basic overlap/foursquare).
pub fn compute_connectivity_dependencies(piece: &Tetromino, piece_id: usize, all_pieces: &[Tetromino; NUM_PIECES]) -> MoveSet {
    let mut dependencies = MoveSet::default();
    let piece_coords = CoordSet::from_iter(piece.real_coords_lazy().map(|c| c.coerce()));
    
    // Test each other piece for connectivity dependency
    for (other_id, other_piece) in all_pieces.iter().enumerate() {
        if other_id == piece_id {
            continue;
        }
        
        if is_connectivity_dependent(piece, &piece_coords, other_piece) {
            dependencies.insert(other_id);
        }
    }
    
    dependencies
}

/// Determines if placing the first piece makes the second piece connectivity-unreachable.
/// 
/// This checks for connectivity constraints beyond basic overlap/foursquare violations.
fn is_connectivity_dependent(blocking_piece: &Tetromino, blocking_coords: &CoordSet, dependent_piece: &Tetromino) -> bool {
    // Check if the dependent piece requires connectivity through the blocking piece's area
    if requires_path_through_blocking_area(blocking_coords, dependent_piece) {
        return true;
    }
    
    // Check if the blocking piece cuts off critical connection paths
    if blocks_critical_connection_paths(blocking_piece, blocking_coords, dependent_piece) {
        return true;
    }
    
    // Check for edge-based isolation patterns
    if creates_edge_isolation(blocking_coords, dependent_piece) {
        return true;
    }
    
    false
}

/// Checks if the dependent piece requires a path through the blocking piece's area.
fn requires_path_through_blocking_area(blocking_coords: &CoordSet, dependent_piece: &Tetromino) -> bool {
    let dependent_coords = CoordSet::from_iter(dependent_piece.real_coords_lazy().map(|c| c.coerce()));
    
    // If pieces are far apart, no direct dependency
    if !pieces_are_nearby(&blocking_coords, &dependent_coords) {
        return false;
    }
    
    // Check if dependent piece needs to connect through blocking area
    // This is a simplified heuristic - in a real game, this would depend on existing board state
    let blocking_neighbors = get_neighbor_region(&blocking_coords);
    let dependent_neighbors = get_neighbor_region(&dependent_coords);
    
    // If the neighbor regions overlap significantly, there's likely a dependency
    let overlap = blocking_neighbors.intersect(&dependent_neighbors);
    overlap.len() >= 2
}

/// Checks if the blocking piece cuts off critical connection paths for the dependent piece.
fn blocks_critical_connection_paths(blocking_piece: &Tetromino, blocking_coords: &CoordSet, dependent_piece: &Tetromino) -> bool {
    // Check for corridor blocking - if blocking piece spans across a narrow area
    // that the dependent piece would need to traverse
    
    let dependent_coords = CoordSet::from_iter(dependent_piece.real_coords_lazy().map(|c| c.coerce()));
    
    // Simple heuristic: if blocking piece is linear and positioned between
    // dependent piece and board edges/corners, it may block critical paths
    if is_linear_barrier(blocking_piece, blocking_coords) {
        return blocks_access_to_regions(&blocking_coords, &dependent_coords);
    }
    
    false
}

/// Checks if the blocking piece creates edge-based isolation for the dependent piece.
fn creates_edge_isolation(blocking_coords: &CoordSet, dependent_piece: &Tetromino) -> bool {
    let dependent_coords = CoordSet::from_iter(dependent_piece.real_coords_lazy().map(|c| c.coerce()));
    
    // Check if dependent piece is near board edges and blocking piece cuts off edge access
    let dependent_near_edge = dependent_coords.iter().any(|coord| {
        coord.row <= 1 || coord.row >= BOARD_SIZE - 2 || 
        coord.col <= 1 || coord.col >= BOARD_SIZE - 2
    });
    
    if !dependent_near_edge {
        return false;
    }
    
    // Check if blocking piece is positioned to cut off edge access
    blocking_coords.iter().any(|blocking_coord| {
        dependent_coords.iter().any(|dependent_coord| {
            // If blocking piece is between dependent piece and board edge
            is_between_piece_and_edge(&blocking_coord, &dependent_coord)
        })
    })
}

/// Helper: Check if two piece regions are nearby (within 3 cells).
fn pieces_are_nearby(coords1: &CoordSet, coords2: &CoordSet) -> bool {
    coords1.iter().any(|c1| {
        coords2.iter().any(|c2| {
            let distance = (c1.row as i32 - c2.row as i32).abs() + 
                          (c1.col as i32 - c2.col as i32).abs();
            distance <= 3
        })
    })
}

/// Helper: Get the neighbor region around a set of coordinates.
fn get_neighbor_region(coords: &CoordSet) -> CoordSet {
    let mut neighbors = CoordSet::default();
    
    for coord in coords.iter() {
        for offset in coords::ORTHOGONAL_OFFSETS.iter() {
            let neighbor = coord + offset;
            if neighbor.in_bounds_signed() {
                neighbors.insert(&neighbor.coerce());
            }
        }
    }
    
    neighbors
}

/// Helper: Check if a piece forms a linear barrier.
fn is_linear_barrier(piece: &Tetromino, coords: &CoordSet) -> bool {
    // Check if piece is primarily linear (I-pieces, straight L-pieces, etc.)
    piece.kind == Tile::I || has_linear_structure(coords)
}

/// Helper: Check if coordinates form a linear structure.
fn has_linear_structure(coords: &CoordSet) -> bool {
    let coords_vec: Vec<_> = coords.iter().collect();
    if coords_vec.len() < 3 {
        return false;
    }
    
    // Check if all pieces are in same row or column
    let same_row = coords_vec.iter().all(|c| c.row == coords_vec[0].row);
    let same_col = coords_vec.iter().all(|c| c.col == coords_vec[0].col);
    
    same_row || same_col
}

/// Helper: Check if blocking piece blocks access to board regions for dependent piece.
fn blocks_access_to_regions(blocking_coords: &CoordSet, dependent_coords: &CoordSet) -> bool {
    // Simplified check: if blocking piece is positioned such that it could
    // interfere with dependent piece's connection to board regions
    
    blocking_coords.iter().any(|blocking_coord| {
        dependent_coords.iter().any(|dependent_coord| {
            // Check if blocking coordinate is in a strategic position relative to dependent
            let row_diff = (blocking_coord.row as i32 - dependent_coord.row as i32).abs();
            let col_diff = (blocking_coord.col as i32 - dependent_coord.col as i32).abs();
            
            // If blocking piece is adjacent and forms a barrier pattern
            (row_diff <= 1 && col_diff <= 2) || (row_diff <= 2 && col_diff <= 1)
        })
    })
}

/// Helper: Check if blocking coordinate is between dependent coordinate and board edge.
fn is_between_piece_and_edge(blocking_coord: &Coord, dependent_coord: &Coord) -> bool {
    // Check if blocking piece intercepts path to nearest edge
    let to_top_edge = dependent_coord.row;
    let to_bottom_edge = BOARD_SIZE - 1 - dependent_coord.row;
    let to_left_edge = dependent_coord.col;
    let to_right_edge = BOARD_SIZE - 1 - dependent_coord.col;
    
    let min_edge_distance = [to_top_edge, to_bottom_edge, to_left_edge, to_right_edge].into_iter().min().unwrap();
    
    // Simple heuristic: blocking piece is between dependent and nearest edge
    let blocking_distance = (blocking_coord.row as i32 - dependent_coord.row as i32).abs() + 
                           (blocking_coord.col as i32 - dependent_coord.col as i32).abs();
    
    blocking_distance <= (min_edge_distance as i32) && blocking_distance <= 2
}