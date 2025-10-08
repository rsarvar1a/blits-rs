/*
 *  An implementation of The Battle of LITS in Rust.
 */

pub(crate) mod board;
pub(crate) mod consts;
pub mod coords;
pub mod notation;
pub(crate) mod piecemap;
pub mod sets;
pub(crate) mod tetromino;

pub mod prelude {
    pub(crate) use crate::utils::prelude::*;

    pub use super::{
        board::Board,
        consts::*,
        coords::{self, *},
        notation::*,
        piecemap::{Interaction, PieceMap},
        sets::*,
        tetromino::{Transform, Tetromino}
    };

    pub use super::sets::SetOps;
}
