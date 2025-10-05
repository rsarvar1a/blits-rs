use crate::battle_of_lits::prelude::*;
use std::collections::BTreeSet;

/// An enum that represents the 8 possible transforms on the cartesian tetrominoes.
///
/// Identity refers to the null transformation, while Reflect refers to reflecting
/// the tetromino in a mirror parallel to the y-axis (i.e. a horizontal reflection).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Transform {
    Identity__,
    Rot90_____,
    Rot180____,
    Rot270____,
    Reflect___,
    ReflRot90_,
    ReflRot180,
    ReflRot270,
}

impl Add for &Transform {
    type Output = Transform;
    fn add(self, rhs: Self) -> Self::Output {
        match rhs {
            Transform::Identity__ => *self,
            Transform::Rot90_____ => self.rotate(),
            Transform::Rot180____ => self.rotate().rotate(),
            Transform::Rot270____ => self.rotate().rotate().rotate(),
            Transform::Reflect___ => self.reflect(),
            Transform::ReflRot90_ => self.reflect().rotate(),
            Transform::ReflRot180 => self.reflect().rotate().rotate(),
            Transform::ReflRot270 => self.reflect().rotate().rotate().rotate(),
        }
    }
}

impl Transform {
    /// Gets all transforms in canonical order.
    pub fn all() -> [Transform; 8] {
        [
            Transform::Identity__,
            Transform::Rot90_____,
            Transform::Rot180____,
            Transform::Rot270____,
            Transform::Reflect___,
            Transform::ReflRot90_,
            Transform::ReflRot180,
            Transform::ReflRot270,
        ]
    }

    /// Applies a transformation to a tetromino.
    pub fn apply(&self, tetromino: &Tetromino) -> Tetromino {
        let points = tetromino
            .points
            .clone()
            .map(|p| self.canonicalize(&tetromino.kind).apply_one(&p));
        let new_transform = (&tetromino.transform + self).canonicalize(&tetromino.kind);
        Tetromino {
            kind: tetromino.kind,
            anchor: tetromino.anchor,
            points,
            transform: new_transform,
        }
    }

    /// Applies a transformation to an offset point.
    pub fn apply_one(&self, offset: &OffsetCoord) -> OffsetCoord {
        let OffsetCoord { rows: r, cols: c } = *offset;
        match self {
            Transform::Identity__ => OffsetCoord::new(r, c),
            Transform::Rot90_____ => OffsetCoord::new(c, -r),
            Transform::Rot180____ => OffsetCoord::new(-r, -c),
            Transform::Rot270____ => OffsetCoord::new(-c, r),
            Transform::Reflect___ => OffsetCoord::new(-r, c),
            Transform::ReflRot90_ => OffsetCoord::new(c, r),
            Transform::ReflRot180 => OffsetCoord::new(r, -c),
            Transform::ReflRot270 => OffsetCoord::new(-c, -r),
        }
    }

    /// Returns the canonical (most direct) transform for this transform and the given tile type.
    pub fn canonicalize(&self, lits: &Tile) -> Transform {
        match lits {
            Tile::L => *self,
            Tile::I => match self {
                Transform::Rot180____ | Transform::Reflect___ | Transform::ReflRot180 => {
                    Transform::Identity__
                }
                Transform::Rot270____ | Transform::ReflRot90_ | Transform::ReflRot270 => {
                    Transform::Rot90_____
                }
                _ => *self,
            },
            Tile::T => match self {
                Transform::Reflect___ => Transform::Identity__,
                Transform::ReflRot270 => Transform::Rot90_____,
                Transform::ReflRot180 => Transform::Rot180____,
                Transform::ReflRot90_ => Transform::Rot270____,
                _ => *self,
            },
            Tile::S => match self {
                Transform::Rot180____ => Transform::Identity__,
                Transform::Rot270____ => Transform::Rot90_____,
                Transform::ReflRot180 => Transform::Reflect___,
                Transform::ReflRot270 => Transform::ReflRot90_,
                _ => *self,
            },
        }
    }

    /// Returns an in-order list of all transformations applicable to the given tile type.
    pub fn enumerate(kind: &Tile) -> Vec<Transform> {
        let mut set: BTreeSet<Transform> = BTreeSet::new();
        for transform in Transform::all() {
            set.insert(transform.canonicalize(kind));
        }
        set.into_iter().collect()
    }

    /// Returns the transform given by reflecting this transform.
    pub fn reflect(&self) -> Transform {
        match self {
            Transform::Identity__ => Transform::Reflect___,
            Transform::Rot90_____ => Transform::ReflRot270,
            Transform::Rot180____ => Transform::ReflRot180,
            Transform::Rot270____ => Transform::ReflRot90_,
            Transform::Reflect___ => Transform::Identity__,
            Transform::ReflRot90_ => Transform::Rot270____,
            Transform::ReflRot180 => Transform::Rot180____,
            Transform::ReflRot270 => Transform::Rot90_____,
        }
    }

    /// Returns the transform given by rotating this transform by 90 degrees.
    pub fn rotate(&self) -> Transform {
        match self {
            Transform::Identity__ => Transform::Rot90_____,
            Transform::Rot90_____ => Transform::Rot180____,
            Transform::Rot180____ => Transform::Rot270____,
            Transform::Rot270____ => Transform::Identity__,
            Transform::Reflect___ => Transform::ReflRot90_,
            Transform::ReflRot90_ => Transform::ReflRot180,
            Transform::ReflRot180 => Transform::ReflRot270,
            Transform::ReflRot270 => Transform::Reflect___,
        }
    }
}
