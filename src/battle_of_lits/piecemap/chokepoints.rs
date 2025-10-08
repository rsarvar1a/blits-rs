use super::*;

/// Computes critical chokepoints that would be blocked by placing this piece.
/// 
/// A chokepoint is a narrow passage (1-2 cells wide) that becomes impassable
/// when this piece is placed, potentially isolating board regions.
pub fn compute_chokepoints(piece: &Tetromino) -> Vec<Coord> {
    let mut chokepoints = Vec::new();
    let piece_coords = CoordSet::from_iter(piece.real_coords_lazy().map(|c| c.coerce()));

    // Check each neighbor of the piece for chokepoint patterns
    for piece_coord in piece_coords.iter() {
        for offset in coords::ORTHOGONAL_OFFSETS.iter() {
            let neighbor = piece_coord + offset;
            if !neighbor.in_bounds_signed() {
                continue;
            }
            let neighbor_coord = neighbor.coerce();
            
            // Skip if neighbor is occupied by the piece itself
            if piece_coords.contains(&neighbor_coord) {
                continue;
            }

            // Check if this neighbor position creates a chokepoint
            if is_chokepoint_position(&piece_coords, &neighbor_coord) {
                chokepoints.push(neighbor_coord);
            }
        }
    }

    chokepoints
}

/// Determines if a position creates a chokepoint when combined with piece placement.
/// 
/// Detects narrow corridors (1-2 cells wide) that would be blocked.
fn is_chokepoint_position(piece_coords: &CoordSet, pos: &Coord) -> bool {
    // Check for narrow horizontal corridors
    let blocks_horizontal = blocks_horizontal_corridor(piece_coords, pos);
    
    // Check for narrow vertical corridors  
    let blocks_vertical = blocks_vertical_corridor(piece_coords, pos);
    
    // Check for corner positions that create isolation
    let blocks_corner = blocks_corner_access(piece_coords, pos);

    blocks_horizontal || blocks_vertical || blocks_corner
}

/// Checks if piece blocks a narrow horizontal corridor.
fn blocks_horizontal_corridor(piece_coords: &CoordSet, pos: &Coord) -> bool {
    // Look for patterns like: wall-empty-empty-wall (2-wide corridor)
    // or: wall-empty-wall (1-wide corridor)
    let left = Coord { row: pos.row, col: pos.col.saturating_sub(1) };
    let right = Coord { row: pos.row, col: (pos.col + 1).min(BOARD_SIZE - 1) };
    
    // Check if we're creating a blockage in a 1-2 cell wide horizontal passage
    let left_blocked = pos.col == 0 || piece_coords.contains(&left);
    let right_blocked = pos.col == BOARD_SIZE - 1 || piece_coords.contains(&right);
    
    left_blocked && right_blocked
}

/// Checks if piece blocks a narrow vertical corridor.
fn blocks_vertical_corridor(piece_coords: &CoordSet, pos: &Coord) -> bool {
    let up = Coord { row: pos.row.saturating_sub(1), col: pos.col };
    let down = Coord { row: (pos.row + 1).min(BOARD_SIZE - 1), col: pos.col };
    
    let up_blocked = pos.row == 0 || piece_coords.contains(&up);
    let down_blocked = pos.row == BOARD_SIZE - 1 || piece_coords.contains(&down);
    
    up_blocked && down_blocked
}

/// Checks if piece blocks corner access, creating isolated regions.
fn blocks_corner_access(piece_coords: &CoordSet, pos: &Coord) -> bool {
    // Check if we're blocking access to corners or edge regions
    let is_near_edge = pos.row <= 1 || pos.row >= BOARD_SIZE - 2 || 
                      pos.col <= 1 || pos.col >= BOARD_SIZE - 2;
    
    if !is_near_edge {
        return false;
    }

    // Count how many orthogonal directions are blocked
    let mut blocked_directions = 0;
    for offset in coords::ORTHOGONAL_OFFSETS.iter() {
        let neighbor = *pos + offset;
        if !neighbor.in_bounds_signed() || piece_coords.contains(&neighbor.coerce()) {
            blocked_directions += 1;
        }
    }

    // If 3+ directions are blocked, this likely creates isolation
    blocked_directions >= 3
}