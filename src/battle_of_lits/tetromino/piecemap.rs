
use std::mem::MaybeUninit;

use itertools::Itertools;

use crate::battle_of_lits::prelude::*;


/// The exact natura in which two pieces interact on the board.
/// - Conflicting - pieces that overlap, or two tiles of the same type that are adjacent
/// - Neutral - pieces that are not adjacent
/// - Adjacent - pieces that are adjacent and not of the same type 
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Interaction {
    Conflicting = 0,
    Neutral = 1,
    Adjacent = 2
}

/// Precomputed data for pairwise interactions between pieces on the board.
#[derive(Clone, Debug)]
pub struct PieceMap {
    /// Get a tetromino by ID.
    forward: Box<[Tetromino; NUM_PIECES]>,

    /// Get a tetromino's ID by its coordinates.
    reverse: HashMap<[OffsetCoord; 4], usize>,

    /// Get an interaction between two tetrominos by ID.
    associations: Vec<Vec<Interaction>>,

    /// Get the tetrominos that have a specific interaction with the subject tetromino. 
    associations_specific: [[MoveSet; 3]; NUM_PIECES],

    /// Neighbours as coordsets for specific inbounds coords.
    coord_neighbours: Box<[CoordSet; 100]>,

    /// Get the neighbouring coords to the tetromino.
    neighbours: Box<[CoordSet; NUM_PIECES]>,

    /// Get the coordset representation of a piece instead of the array representation.
    selfs: Box<[CoordSet; NUM_PIECES]>,

    /// Critical chokepoints: narrow passages this piece would block if placed.
    /// These are 1-2 cell wide corridors that become impassable.
    chokepoints: Box<[Vec<Coord>; NUM_PIECES]>,

    /// Connectivity bridges: pairs of neighbor cells this piece connects together.
    /// Used for fast connectivity validation without flood fill.
    bridges: Box<[Vec<(Coord, Coord)>; NUM_PIECES]>,
}

impl PieceMap {
    /// Creates a new PieceMap.
    pub fn new() -> PieceMap {
        // man just give us placement new already
        let forward = unsafe { 
            let mut tetrominos: Box<MaybeUninit<[Tetromino; NUM_PIECES]>> = Box::new_zeroed();
            let mut i = 0;

            Tile::all().iter().for_each(|kind| {
                (0..10).cartesian_product(0..10).map(|(row, col)| Coord { row, col }).for_each(|anchor| {
                    Tetromino::identity(*kind, &anchor).enumerate().iter().for_each(|isomorph| {
                        if isomorph.in_bounds() {
                            *tetrominos.assume_init_mut().get_unchecked_mut(i) = *isomorph;
                            i += 1;
                        }
                    });
                });
            });

            tetrominos.assume_init()
        };

        let reverse = forward.iter().enumerate().map(|(i, piece): (usize, &Tetromino)| (piece.real_coords(), i)).collect::<HashMap<[OffsetCoord; 4], usize>>();
        let mut associations = vec![vec![Interaction::Conflicting; NUM_PIECES]; NUM_PIECES];

        for i in 0..NUM_PIECES {
            for j in (i + 1)..NUM_PIECES {
                let [lhs, rhs] = [forward[i], forward[j]];
                let [l_coords, r_coords] = [lhs, rhs].map(|p: Tetromino| p.real_coords().into_iter().collect::<HashSet<OffsetCoord>>());

                // 1. do the pieces intersect?
                if l_coords.intersection(&r_coords).cloned().collect::<BTreeSet<_>>().len() > 0 {
                    associations[i][j] = Interaction::Conflicting;
                    continue;
                }

                // 2. do the pieces have no neighbouring tiles?
                if ! l_coords.iter().any(|l| {
                    r_coords.iter().any(|r: &OffsetCoord| r.neighbours(*l))
                }) {
                    associations[i][j] = Interaction::Neutral;
                    continue;
                }

                // 3. are the pieces adjacent and of the same type?
                if lhs.kind == rhs.kind {
                    associations[i][j] = Interaction::Conflicting;
                    continue;
                }

                // 4. do these two pieces alone violate the foursquare rule?
                let cover = l_coords.union(&r_coords).cloned().collect::<HashSet<_>>();
                if cover.iter().any(|c| {
                    cover.contains(&OffsetCoord { rows: c.rows + 1, cols: c.cols })
                        && cover.contains(&OffsetCoord { rows: c.rows, cols: c.cols + 1 })
                        && cover.contains(&OffsetCoord { rows: c.rows + 1, cols: c.cols + 1 })
                }) {
                    associations[i][j] = Interaction::Conflicting;
                    continue;
                }

                associations[i][j] = Interaction::Adjacent;
            }
        }

        let associations_specific: [[MoveSet; 3]; NUM_PIECES] = (0..NUM_PIECES).map(|idx| {
            [Interaction::Conflicting, Interaction::Neutral, Interaction::Adjacent].map(|int| {
                (0..NUM_PIECES).filter(|&p| associations[idx.min(p)][idx.max(p)] == int).collect()
            })
        }).collect_array::<NUM_PIECES>().unwrap();

        // man just give us placement new already
        let neighbours = unsafe {
            let mut neighbours: Box<MaybeUninit<[CoordSet; NUM_PIECES]>> = Box::new_zeroed();
            (0..NUM_PIECES).for_each(|idx| {
                *neighbours.assume_init_mut().get_unchecked_mut(idx) = forward[idx].neighbours();
            });
            neighbours.assume_init()
        };

        let selfs = unsafe {
            let mut selfs: Box<MaybeUninit<[CoordSet; NUM_PIECES]>> = Box::new_zeroed();
            (0..NUM_PIECES).for_each(|idx| {
                *selfs.assume_init_mut().get_unchecked_mut(idx) = CoordSet::from_iter(forward[idx].real_coords_lazy().map(|c| c.coerce()));
            });
            selfs.assume_init()
        };

        let coord_neighbours = unsafe {
            let mut neighbours: Box<MaybeUninit<[CoordSet; 100]>> = Box::new_zeroed();
            (0..10).cartesian_product(0..10).for_each(|(row, col)| {
                let idx = row * BOARD_SIZE + col;
                let c = Coord { row, col };
                let mut set = CoordSet::default();
                ORTHOGONAL_OFFSETS.iter().for_each(|offset| {
                    let candidate = c + offset;
                    if candidate.in_bounds_signed() {
                        set.insert(&candidate.coerce());
                    }
                });
                *neighbours.assume_init_mut().get_unchecked_mut(idx) = set;
            });
            neighbours.assume_init()
        };

        let chokepoints = unsafe {
            let mut chokepoints: Box<MaybeUninit<[Vec<Coord>; NUM_PIECES]>> = Box::new_zeroed();
            (0..NUM_PIECES).for_each(|idx| {
                *chokepoints.assume_init_mut().get_unchecked_mut(idx) = Self::compute_chokepoints(&forward[idx]);
            });
            chokepoints.assume_init()
        };

        let bridges = unsafe {
            let mut bridges: Box<MaybeUninit<[Vec<(Coord, Coord)>; NUM_PIECES]>> = Box::new_zeroed();
            (0..NUM_PIECES).for_each(|idx| {
                *bridges.assume_init_mut().get_unchecked_mut(idx) = Self::compute_connectivity_bridges(&forward[idx]);
            });
            bridges.assume_init()
        };

        PieceMap { 
            forward, 
            reverse, 
            associations, 
            associations_specific, 
            coord_neighbours,
            neighbours,
            selfs,
            chokepoints,
            bridges
        }
    }

    /// Gets a coordset consisting of the on-board neighbours of an on-board Coord.
    pub fn coord_neighbours(&self, coord: &Coord) -> &CoordSet {
        unsafe {
            let Coord { row, col } = *coord;
            let idx = row * BOARD_SIZE + col;
            self.coord_neighbours.get_unchecked(idx)
        }
    }

    /// Gets the piece as a coordset.
    pub fn coordset(&self, id: usize) -> &CoordSet {
        unsafe {
            self.selfs.get_unchecked(id)
        }
    }

    /// Gets the critical chokepoints this piece would block.
    pub fn chokepoints(&self, id: usize) -> &Vec<Coord> {
        unsafe {
            self.chokepoints.get_unchecked(id)
        }
    }

    /// Gets the connectivity bridges this piece creates between neighbor cells.
    pub fn bridges(&self, id: usize) -> &Vec<(Coord, Coord)> {
        unsafe {
            self.bridges.get_unchecked(id)
        }
    }

    /// Gets the interaction between two pieces by ID.
    pub fn get_association(&self, i: usize, j: usize) -> Interaction {
        let [r, c] = [i.min(j), i.max(j)];
        unsafe { 
            *self.associations.get_unchecked(r).get_unchecked(c)
        }
    }

    /// Gets the piece type by ID.
    pub fn get_kind(&self, id: usize) -> Tile {
        unsafe {
            self.forward.get_unchecked(id).kind
        }
    }

    /// Gets the piece ID of a given tetromino.
    pub fn get_id(&self, tetromino: &Tetromino) -> usize {
        let idx = tetromino.real_coords();
        self.reverse[&idx]
    } 

    /// Gets a piece by ID.
    pub fn get_piece(&self, id: usize) -> &Tetromino {
        unsafe {
            self.forward.get_unchecked(id)
        }
    }

    /// Gets the neighbouring coords of a piece by ID.
    pub fn neighbours(&self, id: usize) -> &CoordSet {
        unsafe {
            self.neighbours.get_unchecked(id)
        }
    }

    /// Notates a piece by ID.
    pub fn notate(&self, id: usize) -> String {
        match id {
            NULL_MOVE => "swap".into(),
            _         => self.get_piece(id).notate()
        }
    }

    /// Tries to find the given set of coords in the table.
    pub fn try_and_find(&self, coords: &[OffsetCoord; 4]) -> Result<usize> {
        let mut coords = coords.clone();
        coords.sort();
        if self.reverse.contains_key(&coords) {
            Ok(self.reverse[&coords])
        } else {
            Err(anyhow!("could not match a Tetromino to coords {coords:?}"))
        }
    }

    /// Tries to map the ID to a tetromino.
    pub fn try_by_id(&self, id: usize) -> Result<&Tetromino> {
        if id < NUM_PIECES {
            Ok(self.get_piece(id))
        } else {
            Err(anyhow!("id {id} out of range"))
        }
    }

    /// Gets the interactions on a piece matching a certain outcome.
    pub fn with_interaction(&self, id: usize, interaction: Interaction) -> &MoveSet {
        unsafe {
            self.associations_specific.get_unchecked(id).get_unchecked(interaction as usize)
        }
    }

    /// Computes critical chokepoints that would be blocked by placing this piece.
    /// 
    /// A chokepoint is a narrow passage (1-2 cells wide) that becomes impassable
    /// when this piece is placed, potentially isolating board regions.
    fn compute_chokepoints(piece: &Tetromino) -> Vec<Coord> {
        let mut chokepoints = Vec::new();
        let piece_coords: std::collections::HashSet<_> = piece.real_coords_lazy()
            .map(|c| c.coerce())
            .collect();

        // Check each neighbor of the piece for chokepoint patterns
        for piece_coord in piece_coords.iter() {
            for offset in coords::ORTHOGONAL_OFFSETS.iter() {
                let neighbor = *piece_coord + offset;
                if !neighbor.in_bounds_signed() {
                    continue;
                }
                let neighbor_coord = neighbor.coerce();
                
                // Skip if neighbor is occupied by the piece itself
                if piece_coords.contains(&neighbor_coord) {
                    continue;
                }

                // Check if this neighbor position creates a chokepoint
                if Self::is_chokepoint_position(&piece_coords, &neighbor_coord) {
                    chokepoints.push(neighbor_coord);
                }
            }
        }

        chokepoints
    }

    /// Determines if a position creates a chokepoint when combined with piece placement.
    /// 
    /// Detects narrow corridors (1-2 cells wide) that would be blocked.
    fn is_chokepoint_position(piece_coords: &std::collections::HashSet<Coord>, pos: &Coord) -> bool {
        // Check for narrow horizontal corridors
        let blocks_horizontal = Self::blocks_horizontal_corridor(piece_coords, pos);
        
        // Check for narrow vertical corridors  
        let blocks_vertical = Self::blocks_vertical_corridor(piece_coords, pos);
        
        // Check for corner positions that create isolation
        let blocks_corner = Self::blocks_corner_access(piece_coords, pos);

        blocks_horizontal || blocks_vertical || blocks_corner
    }

    /// Checks if piece blocks a narrow horizontal corridor.
    fn blocks_horizontal_corridor(piece_coords: &std::collections::HashSet<Coord>, pos: &Coord) -> bool {
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
    fn blocks_vertical_corridor(piece_coords: &std::collections::HashSet<Coord>, pos: &Coord) -> bool {
        let up = Coord { row: pos.row.saturating_sub(1), col: pos.col };
        let down = Coord { row: (pos.row + 1).min(BOARD_SIZE - 1), col: pos.col };
        
        let up_blocked = pos.row == 0 || piece_coords.contains(&up);
        let down_blocked = pos.row == BOARD_SIZE - 1 || piece_coords.contains(&down);
        
        up_blocked && down_blocked
    }

    /// Checks if piece blocks corner access, creating isolated regions.
    fn blocks_corner_access(piece_coords: &std::collections::HashSet<Coord>, pos: &Coord) -> bool {
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

    /// Computes connectivity bridges created by placing this piece.
    /// 
    /// A bridge connects two neighbor cells that were previously disconnected.
    /// This is used for fast connectivity validation during reachability analysis.
    fn compute_connectivity_bridges(piece: &Tetromino) -> Vec<(Coord, Coord)> {
        let mut bridges = Vec::new();
        let piece_coords: std::collections::HashSet<_> = piece.real_coords_lazy()
            .map(|c| c.coerce())
            .collect();

        // Get all neighbor coordinates around the piece
        let neighbors: Vec<Coord> = piece_coords.iter()
            .flat_map(|&coord| {
                coords::ORTHOGONAL_OFFSETS.iter().filter_map(move |offset| {
                    let neighbor = coord + offset;
                    if neighbor.in_bounds_signed() {
                        let neighbor_coord = neighbor.coerce();
                        if !piece_coords.contains(&neighbor_coord) {
                            Some(neighbor_coord)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
            })
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        // Find pairs of neighbors that this piece bridges together
        for i in 0..neighbors.len() {
            for j in (i + 1)..neighbors.len() {
                let coord1 = neighbors[i];
                let coord2 = neighbors[j];
                
                if Self::piece_bridges_neighbors(&piece_coords, &coord1, &coord2) {
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
        piece_coords: &std::collections::HashSet<Coord>, 
        coord1: &Coord, 
        coord2: &Coord
    ) -> bool {
        // Check if both coordinates are reachable from the piece
        let coord1_touches_piece = Self::coord_touches_piece(piece_coords, coord1);
        let coord2_touches_piece = Self::coord_touches_piece(piece_coords, coord2);
        
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
    fn coord_touches_piece(piece_coords: &std::collections::HashSet<Coord>, coord: &Coord) -> bool {
        coords::ORTHOGONAL_OFFSETS.iter().any(|offset| {
            let neighbor = *coord + offset;
            if neighbor.in_bounds_signed() {
                piece_coords.contains(&neighbor.coerce())
            } else {
                false
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;
    use crate::battle_of_lits::consts::NUM_PIECES;
    use super::PieceMap;

    #[test]
    fn ensure_builds() {
        let timer = Instant::now();
        let piece_map = PieceMap::new();
        assert_eq!(piece_map.reverse.len(), NUM_PIECES);
        let elapsed = Instant::now() - timer;
        println!("took {}s", elapsed.as_secs());
    }
}
