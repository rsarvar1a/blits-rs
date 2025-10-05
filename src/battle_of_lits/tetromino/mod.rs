pub mod piecemap;
pub mod transform;

use super::prelude::*;

use itertools::Itertools;
pub use piecemap::PieceMap;
pub use transform::Transform;

#[derive(Clone, Copy, Debug, PartialOrd, Ord)]
pub struct Tetromino {
    pub kind: Tile,
    pub anchor: Coord,
    pub points: [OffsetCoord; 4],
    pub real_coords: [OffsetCoord; 4],
    pub transform: Transform,
}

impl Default for Tetromino {
    fn default() -> Self {
        Tetromino { 
            kind: Tile::L, 
            anchor: Coord::new(0, 0), 
            points: [OffsetCoord::new(0, 0); 4],
            real_coords: [OffsetCoord::new(0, 0); 4],
            transform: Transform::Identity__ 
        }
    }
}

impl std::cmp::PartialEq for Tetromino {
    fn eq(&self, other: &Self) -> bool {
        if self.kind != other.kind || self.transform != other.transform { // skip the expensive work if we can
            return false;
        }
        let [lhs, rhs] = [self.real_coords_lazy(), other.real_coords_lazy()];       // if all four real coords are the same, in any order, we are obviously dealing with the same shape
        BTreeSet::from_iter(lhs) == BTreeSet::from_iter(rhs) 
    }
}
impl std::cmp::Eq for Tetromino {}

impl Tetromino {
    /// Produces the tetromino obtained by moving this tetromino to the given anchor.
    pub fn at(&self, coord: &Coord) -> Tetromino {
        Tetromino { 
            kind: self.kind, 
            anchor: *coord, 
            points: self.points, 
            real_coords: self.points.map(|c| coord + c), 
            transform: self.transform 
        }
    }

    /// Gives back all tetrominos that result from canonical transformations on this Tetromino's anchor and type.
    pub fn enumerate(&self) -> Vec<Tetromino> {
        let transforms = Transform::enumerate(&self.kind);
        let iden = self.clone();
        transforms.iter().map(|transform| transform.apply(&iden)).collect()
    }

    /// Constructs the identity tetromino at the given anchor. Makes no guarantees that the tile is in bounds!
    pub fn identity(kind: Tile, anchor: &Coord) -> Tetromino {
        let template = Tetromino::_identity_template(kind);
        Tetromino {
            kind,
            anchor: *anchor,
            points: template,
            real_coords: template.map(|c| anchor + c),
            transform: Transform::Identity__,
        }
    }

    /// Determines whether or not the tetromino is in bounds.
    pub fn in_bounds(&self) -> bool {
        self.real_coords_lazy().all(|c| c.in_bounds_signed())
    }

    /// Gets the base shape corresponding to the tile type as a set of offsets on an anchor point.
    fn _identity_template(kind: Tile) -> [OffsetCoord; 4] {
        match kind {
            Tile::L => [
                OffsetCoord { rows: -1, cols: 0 },
                OffsetCoord { rows: 0, cols: 0 },
                OffsetCoord { rows: 1, cols: 0 },
                OffsetCoord { rows: 1, cols: 1 },
            ],
            Tile::I => [
                OffsetCoord { rows: -1, cols: 0 },
                OffsetCoord { rows: 0, cols: 0 },
                OffsetCoord { rows: 1, cols: 0 },
                OffsetCoord { rows: 2, cols: 0 },
            ],
            Tile::T => [
                OffsetCoord { rows: 0, cols: -1 },
                OffsetCoord { rows: 0, cols: 0 },
                OffsetCoord { rows: 0, cols: 1 },
                OffsetCoord { rows: 1, cols: 0 },
            ],
            Tile::S => [
                OffsetCoord { rows: 0, cols: -1 },
                OffsetCoord { rows: 0, cols: 0 },
                OffsetCoord { rows: 1, cols: 0 },
                OffsetCoord { rows: 1, cols: 1 },
            ],
        }
    }

    /// The on-board neighbours of the points in this piece. They are:
    /// 1. compute the set of neighbours of each point
    /// 2. keep each one that's in-bounds
    /// 3. discard any that's also a coordinate on the piece
    pub fn neighbours(&self) -> CoordSet {
        let inside = self.real_coords_lazy().filter_map(|oc| {
            if oc.in_bounds_signed() { 
                Some(oc.coerce()) 
            } else { 
                None 
            }
        }).collect::<CoordSet>();

        inside.iter().flat_map(|c| {
            ORTHOGONAL_OFFSETS.iter().map(move |offset| {
                c + offset
            }).filter_map(|c| {
                if c.in_bounds_signed() && !inside.contains(&c.coerce()) {
                    Some(c.coerce())
                } else {
                    None
                }
            }).collect::<CoordSet>()
        }).collect()
    }

    /// The canonical notation for the piece; must be in bounds!
    pub fn notate(&self) -> String {
        let arr = self.real_coords_lazy().map(|c| c.coerce().notate()).join(",");
        format!("{:?}[{}]", self.kind, arr)
    }

    /// Gets the real board coordinates of the move by adding the anchor to the offsets.
    pub fn real_coords(&self) -> [OffsetCoord; 4] {
        let mut coords = self.points.map(|p| self.anchor + p);
        coords.sort();
        coords
    }

    /// Gets an iterator to the real coords of a canonical Tetromino.
    pub fn real_coords_lazy<'a>(&'a self) -> impl Iterator<Item = &'a OffsetCoord> {
        self.real_coords.iter()
    }

    /// Reanchors a tetromino at the given point by:
    /// 1. picking one of the offset coordinates to zero out
    /// 2. adding that offset to the anchor, moving the anchor to it
    /// 3. subtracting that offset from every point on the tetromino (which makes the newly chosen reference point (0, 0)
    pub fn reanchor(&self, i: usize) -> Tetromino {
        let new_focus = self.points[i];
        Tetromino {
            kind: self.kind,
            anchor: (self.anchor + new_focus).coerce(),
            points: self.points.map(|p| p - new_focus),
            real_coords: self.real_coords,
            transform: self.transform
        }
    }

    /// Recontextualizes a template tetromino by:
    /// 1. picking one of the offset coordinates in the template to be the anchor point
    /// 2. adding that offset to the existing anchor
    /// 3. subtracing that offset from every point on the tetromino
    /// 
    /// This method is useful for enumerating the tetrominos that touch a certain point
    /// on the board, especially for move generation. For example, the T is illustrative:
    ///
    /// `0 1 2`  
    /// 
    /// `. 3 .`
    ///
    /// Some interesting callouts on how this affects transformations:
    /// - the 1-recontextualized T is just the default T
    /// - the default T spins on its center, while the other three spin wide 
    pub fn recontextualize(kind: Tile, anchor: Coord, i: u8) -> Tetromino {
        let iden = Tetromino::identity(kind, &anchor);
        let new_focus = iden.points[i as usize];
        Tetromino { 
            kind, 
            anchor: (anchor + new_focus).coerce(), 
            points: iden.points.map(|p| p - new_focus),
            real_coords: iden.real_coords,
            transform: Transform::Identity__
        } 
    }

    /// Determines whether the given coords are a tetromino; if so, returns a tetromino representing those coords.
    /// 
    /// Note that the returned tetromino is not guaranteed to be in standard form (i.e. a tetromino in the piecemap);
    /// in fact, it is _likely_ to be nonstandard as it is quite difficult to find the correct recontextualization
    /// due to possible transformations.
    pub fn validate(kind: Tile, coords: [Coord; 4]) -> Result<Tetromino> {
        let distances: [usize; 16] = coords.iter().cartesian_product(
            coords.iter()).map(|(lhs, rhs)| {
                lhs.squared_distance(rhs)
            }).sorted().collect_array::<16>().unwrap();

        let real_kind = match distances {
            [0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 2, 2, 4, 4, 5, 5] => Tile::L,
            [0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 4, 4, 4, 4, 9, 9] => Tile::I,
            [0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 4, 4] => Tile::T,
            [0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 5, 5] => Tile::S,
            _                                                => { return Err(anyhow!("coords {coords:?} are not a valid tetromino!")); }
        };
        if real_kind != kind {
            return Err(anyhow!("given Tile {kind:?}, but this Tetromino is of type {real_kind:?}"));
        }

        Ok(Tetromino {
            kind, 
            anchor: Coord::new(0, 0), 
            points: coords.map(|c| c.into()), 
            real_coords: coords.map(|c| c.into()),
            transform: Transform::Identity__ 
        })
    }
}
