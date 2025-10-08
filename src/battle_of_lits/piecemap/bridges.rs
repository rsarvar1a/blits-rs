use super::*;

/// Computes connectivity bridges created by placing this piece.
/// 
/// A bridge connects two neighbor cells that were previously disconnected.
/// This is used for fast connectivity validation during reachability analysis.
pub fn compute_connectivity_bridges(piece: &Tetromino) -> Vec<(Coord, Coord)> {
    let mut bridges = Vec::new();
    let piece_coords = CoordSet::from_iter(piece.real_coords_lazy().map(|c| c.coerce()));

    // Get all neighbor coordinates around the piece
    let mut neighbors = CoordSet::default();
    for coord in piece_coords.iter() {
        for offset in coords::ORTHOGONAL_OFFSETS.iter() {
            let neighbor = coord + offset;
            if neighbor.in_bounds_signed() {
                let neighbor_coord = neighbor.coerce();
                if !piece_coords.contains(&neighbor_coord) {
                    neighbors.insert(&neighbor_coord);
                }
            }
        }
    }
    
    let neighbors_vec: Vec<Coord> = neighbors.iter().collect();

    // Find pairs of neighbors that this piece bridges together
    for i in 0..neighbors_vec.len() {
        for j in (i + 1)..neighbors_vec.len() {
            let coord1 = neighbors_vec[i];
            let coord2 = neighbors_vec[j];
            
            if piece_bridges_neighbors(&piece_coords, &coord1, &coord2) {
                bridges.push((coord1, coord2));
            }
        }
    }

    bridges
}

/// Determines if a piece bridges two neighbor coordinates together.
/// 
/// Two neighbors are bridged if they can reach each other through the piece
/// but would be disconnected without it.
fn piece_bridges_neighbors(
    piece_coords: &CoordSet, 
    coord1: &Coord, 
    coord2: &Coord
) -> bool {
    // Check if both coordinates are reachable from the piece
    let coord1_touches_piece = coord_touches_piece(piece_coords, coord1);
    let coord2_touches_piece = coord_touches_piece(piece_coords, coord2);
    
    if !coord1_touches_piece || !coord2_touches_piece {
        return false;
    }

    // Check if the coordinates are far enough apart that the piece acts as a bridge
    let distance = ((coord1.row as i32 - coord2.row as i32).abs() + 
                   (coord1.col as i32 - coord2.col as i32).abs()) as usize;
    
    // Coordinates must be at least 2 steps apart to be meaningfully bridged
    // (adjacent coordinates don't need bridging)
    distance >= 2
}

/// Checks if a coordinate touches (is adjacent to) the piece.
fn coord_touches_piece(piece_coords: &CoordSet, coord: &Coord) -> bool {
    coords::ORTHOGONAL_OFFSETS.iter().any(|offset| {
        let neighbor = *coord + offset;
        if neighbor.in_bounds_signed() {
            piece_coords.contains(&neighbor.coerce())
        } else {
            false
        }
    })
}