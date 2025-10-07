
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

    /// Get the neighbouring coords to the tetromino.
    neighbours: Box<[CoordSet; NUM_PIECES]>
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

        PieceMap { 
            forward, 
            reverse, 
            associations, 
            associations_specific, 
            neighbours 
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
