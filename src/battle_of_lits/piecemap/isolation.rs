use super::*;

/// Computes isolation potential for a piece.
/// 
/// Returns true if this piece has high likelihood of creating isolated regions
/// based on its shape, size, and typical placement patterns.
pub fn compute_isolation_potential(piece: &Tetromino) -> bool {
    // Criteria for high isolation potential:
    
    // 1. Long straight pieces (I-pieces) can create walls
    if piece.kind == Tile::I && is_straight_piece(piece) {
        return true;
    }
    
    // 2. Corner pieces (L-pieces) can block corner access
    if piece.kind == Tile::L && creates_corner_blockage(piece) {
        return true;
    }
    
    // 3. Large/sprawling pieces that cover significant area
    if has_wide_footprint(piece) {
        return true;
    }
    
    // 4. Pieces near board edges have higher isolation potential
    if is_near_board_edge(piece) {
        return true;
    }
    
    false
}

/// Checks if this is a straight I-piece (horizontal or vertical line).
fn is_straight_piece(piece: &Tetromino) -> bool {
    let coords: Vec<_> = piece.real_coords_lazy().map(|c| c.coerce()).collect();
    
    // Check if all pieces are in same row (horizontal line)
    let same_row = coords.iter().all(|c| c.row == coords[0].row);
    
    // Check if all pieces are in same column (vertical line) 
    let same_col = coords.iter().all(|c| c.col == coords[0].col);
    
    same_row || same_col
}

/// Checks if L-piece creates corner blockage patterns.
fn creates_corner_blockage(piece: &Tetromino) -> bool {
    let coords: Vec<_> = piece.real_coords_lazy().map(|c| c.coerce()).collect();
    
    // L-pieces have a characteristic corner shape
    // Count orthogonal connections between cells
    let mut connections = 0;
    for i in 0..coords.len() {
        for j in (i + 1)..coords.len() {
            let coord1 = coords[i];
            let coord2 = coords[j];
            
            // Check if coordinates are orthogonally adjacent
            let distance = (coord1.row as i32 - coord2.row as i32).abs() + 
                          (coord1.col as i32 - coord2.col as i32).abs();
            if distance == 1 {
                connections += 1;
            }
        }
    }
    
    // L-pieces typically have 3 connections in their corner structure
    connections == 3
}

/// Checks if piece has wide footprint that can create barriers.
fn has_wide_footprint(piece: &Tetromino) -> bool {
    let coords: Vec<_> = piece.real_coords_lazy().map(|c| c.coerce()).collect();
    
    let min_row = coords.iter().map(|c| c.row).min().unwrap_or(0);
    let max_row = coords.iter().map(|c| c.row).max().unwrap_or(0);
    let min_col = coords.iter().map(|c| c.col).min().unwrap_or(0);
    let max_col = coords.iter().map(|c| c.col).max().unwrap_or(0);
    
    let width = max_col - min_col + 1;
    let height = max_row - min_row + 1;
    
    // Pieces that span 3+ cells in any direction have higher isolation potential
    width >= 3 || height >= 3
}

/// Checks if piece is positioned near board edges where isolation is more likely.
fn is_near_board_edge(piece: &Tetromino) -> bool {
    piece.real_coords_lazy().any(|coord| {
        let c = coord.coerce();
        // Within 2 cells of any board edge
        c.row <= 1 || c.row >= BOARD_SIZE - 2 || 
        c.col <= 1 || c.col >= BOARD_SIZE - 2
    })
}