mod bridges;
mod chokepoints;
mod dependencies;
mod isolation;
mod new;
mod shadows;

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
    associations_specific: Box<[[MoveSet; 3]; NUM_PIECES]>,

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

    /// Isolation potential: boolean flags indicating pieces with high likelihood
    /// of creating isolated regions based on shape and position.
    isolation_potential: Box<[bool; NUM_PIECES]>,

    /// Connectivity dependencies: pieces that become unreachable due to connectivity
    /// constraints when this piece is placed (beyond basic overlap/foursquare).
    connectivity_dependencies: Box<[MoveSet; NUM_PIECES]>,

    /// Isolation shadow maps: precomputed regions that become isolated when this piece
    /// is placed at strategic positions. Maps anchor positions to isolated regions.
    isolation_shadows: Box<[Vec<(Coord, CoordSet)>; NUM_PIECES]>,
}

impl PieceMap {
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

    /// Gets the isolation potential for this piece.
    pub fn has_isolation_potential(&self, id: usize) -> bool {
        unsafe {
            *self.isolation_potential.get_unchecked(id)
        }
    }

    /// Gets the connectivity dependencies for this piece.
    pub fn connectivity_dependencies(&self, id: usize) -> &MoveSet {
        unsafe {
            self.connectivity_dependencies.get_unchecked(id)
        }
    }

    /// Gets the isolation shadow maps for this piece.
    pub fn isolation_shadows(&self, id: usize) -> &Vec<(Coord, CoordSet)> {
        unsafe {
            self.isolation_shadows.get_unchecked(id)
        }
    }

    /// Gets the interaction between two pieces by ID.
    pub fn get_association(&self, i: usize, j: usize) -> Interaction {
        let [r, c] = [i.min(j), i.max(j)];
        unsafe { 
            *self.associations.get_unchecked(r).get_unchecked(c)
        }
    }

    /// Gets the type of a tetromino.
    pub fn get_kind(&self, id: usize) -> Tile {
        unsafe {
            self.forward.get_unchecked(id).kind
        }
    }

    /// Gets a tetromino by ID.
    pub fn get_piece(&self, id: usize) -> Tetromino {
        unsafe {
            *self.forward.get_unchecked(id)
        }
    }

    /// Gets a tetromino ID by its coordinates.
    pub fn try_and_find(&self, coords: &[OffsetCoord; 4]) -> Result<usize> {
        let mut v = coords.clone();
        v.sort();
        
        if let Some(&id) = self.reverse.get(&v) {
            Ok(id)
        } else {
            Err(anyhow!("id {coords:?} out of range"))
        }
    }

    /// Gets the piece neighbours as a coordset.
    pub fn neighbours(&self, id: usize) -> &CoordSet {
        unsafe {
            self.neighbours.get_unchecked(id)
        }
    }

    /// Validates a piece id.
    pub fn get_piece_checked(&self, id: usize) -> Result<Tetromino> {
        if id < NUM_PIECES {
            Ok(unsafe { *self.forward.get_unchecked(id) })
        } else {
            Err(anyhow!("id {id} out of range"))
        }
    }

    /// Notates a piece move.
    pub fn notate(&self, id: usize) -> String {
        match id {
            NULL_MOVE => "swap".into(),
            _         => self.get_piece(id).notate()
        }
    }

    /// Gets the interactions on a piece matching a certain outcome.
    pub fn with_interaction(&self, id: usize, interaction: Interaction) -> &MoveSet {
        unsafe {
            self.associations_specific.get_unchecked(id).get_unchecked(interaction as usize)
        }
    }
}