use std::ops::Neg;
use crate::utils::prelude::*;

pub const BOARD_SIZE: usize = 10;
pub const PIECES_PER_KIND: usize = 5;
pub const NUM_PIECES: usize = 1292;
pub const NULL_MOVE: usize = NUM_PIECES;

// A cell typing.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Player {
    X = 0,
    O = 1,
}

impl Player {
    /// Notates the player.
    pub fn notate(&self) -> String {
        match self {
            Player::X => "X",
            Player::O => "O"
        }.into()
    }

    /// The given player's scoring factor.
    /// Choosing 1 and -1 allows for branchless negamax.
    pub fn perspective(&self) -> i32 {
        match self {
            Player::X => 1,
            Player::O => -1
        }
    }

    /// Parses into a player.
    pub fn parse(s: &str) -> Result<Option<Player>> {
        match s {
            "x" | "X" => Ok(Some(Player::X)),
            "o" | "O" => Ok(Some(Player::O)),
            "_" | "-" | "." => Ok(None),
            _               => Err(anyhow!("invalid notation {s} for player"))
        }
    }
}

impl Neg for Player {
    type Output = Player;
    fn neg(self) -> Self::Output {
        match self {
            Player::X => Player::O,
            Player::O => Player::X
        }
    }
}

impl From<u8> for Player {
    fn from(value: u8) -> Self {
        match value {
            0 => Player::X,
            1 => Player::O,
            _ => panic!("expected CellValue of 0-1, received {value}"),
        }
    }
}

impl std::str::FromStr for Tile {
    type Err = Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "L" | "l" => Ok(Tile::L),
            "I" | "i" => Ok(Tile::I),
            "T" | "t" => Ok(Tile::T),
            "S" | "s" => Ok(Tile::S),
            _         => Err(anyhow!("invalid notation {s} for Tile"))
        }
    }
}

// A tile typing.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tile {
    L = 0,
    I = 1,
    T = 2,
    S = 3,
}

impl From<u8> for Tile {
    fn from(value: u8) -> Self {
        match value {
            0 => Tile::L,
            1 => Tile::I,
            2 => Tile::T,
            3 => Tile::S,
            _ => panic!("expected LITSValue of 0-3, received {value}"),
        }
    }
}

impl Tile {
    /// Gets the LITS tile types in order.
    pub fn all() -> [Tile; 4] {
        [Tile::L, Tile::I, Tile::T, Tile::S]
    }
}