
use regex::Regex;

use crate::{prelude::{Coord, Player, Tetromino, Tile, BOARD_SIZE}, battle_of_lits::board::Grid, utils::prelude::*};

/// A segment of a gamestring that represents the board setup
/// (i.e. the placements of the Xs and Os).
#[derive(Clone, Debug)]
pub struct SetupString {
    pub repr: String,
    pub grid: Grid
}

/// Ensures a produced grid is actually valid; i.e. Xs and Os have rotational equivalence.
fn _validate_rotational_symmetry(grid: &Grid) -> std::result::Result<(), Error> {
    for r in 0..BOARD_SIZE {
        for c in 0..BOARD_SIZE {
            let lhs = grid.0[r][c].cell_value();
            let rhs = grid.0[BOARD_SIZE - 1 - r][BOARD_SIZE - 1 - c].cell_value();
            if lhs.map_or(rhs.is_none(), |vl| rhs.is_some_and(|vr| vl == -vr)) { // either both none, or some and inverses
                continue;
            }
            return Err(anyhow!("cells {}{} and {}{} do not match", r, c, BOARD_SIZE - 1 - r, BOARD_SIZE - 1 - c));
        }
    }
    Ok(())
}

/// Parses the 20-character bitstring encoding for the game.
fn _parse_compressed_setup_string(_s: &str) -> std::result::Result<SetupString, Error> {
    let _grid = Grid::default();
    todo!("parse a compressed string... 3 days later, I have no motivation to implement this")
}

/// Parses a 100-character setup string (of the form XO..X.X.O. etc.).
fn _parse_naive_setup_string(s: &str) -> std::result::Result<SetupString, Error> {
    let mut grid = Grid::default();
    for (i, ch) in s.chars().enumerate() {
        let [r, c] = [i / BOARD_SIZE, i % BOARD_SIZE];
        let player = Player::parse(&ch.to_string())?;
        grid.0[r][c] = grid.0[r][c].with_cell(player);
    }
    _validate_rotational_symmetry(&grid)?;
    Ok(SetupString { repr: s.to_owned(), grid })
}

impl std::str::FromStr for SetupString {
    type Err = Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.len() {
            20  => _parse_compressed_setup_string(s),
            100 => _parse_naive_setup_string(s),
            _   => Err(anyhow!("unrecognized setup string {s}"))
        }
    }
}

/// A segment of a gamestring that represents a move (more
/// particularly, a tetromino, since we cannot determine its
/// id without access to the piecemap). If the move represents
/// the swap, then it contains no tetromino.
#[derive(Clone, Debug)]
pub struct MoveString {
    pub repr: String,
    pub tetromino: Option<Tetromino>
}

impl std::str::FromStr for MoveString {
    type Err = Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if s == "swap" {
            return Ok(MoveString { repr: s.to_owned(), tetromino: None });
        }
        
        let pattern = Regex::new("(?<kind>[LITS])\\[(?<coords>[0-9]{2}(,[0-9]{2}){3})\\]")?;
        let Some(matches) = pattern.captures(s) else {
            return Err(anyhow!("could not parse movestring {s}"));
        };

        let kind = matches.name("kind").unwrap().as_str().parse::<Tile>()?;

        let coord_strs = matches.name("coords").unwrap().as_str().split(",").collect::<Vec<&str>>();
        if coord_strs.len() != 4 {
            return Err(anyhow!("expected 4 coordinates, received {}", coord_strs.len()));
        }
        
        let mut coords = [Coord::new(0, 0); 4];
        for (i, coord_str) in coord_strs.iter().enumerate() {
            let coord = coord_str.parse::<Coord>()?;
            coords[i] = coord;
        }
        coords.sort();

        let tetromino = Tetromino::validate(kind, coords)?; // non-canonical but valid, so we can use it to query the piecemap
        Ok(MoveString { repr: s.to_owned(), tetromino: Some(tetromino) })
    }
}

/// A parsed gamestring that resolves to a valid game of LITS.
/// 
/// Caveat: the game need not actually be semantically valid, only syntactically;
/// it is possible to receive a gamestring in which any given move is not a legal
/// continuation of the board state obtained by the gamestring preceding that move.
/// 
/// To ensure a gamestring is actually valid, its moves should be tried 
/// iteratively against Board::play().
#[derive(Clone, Debug)]
pub struct GameString {
    pub setup: SetupString,
    pub moves: Vec<MoveString>
}

impl std::str::FromStr for GameString {
    type Err = Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let parts = s.split(";").collect::<Vec<&str>>();
        let Some((setup_str, movelist)) = parts.split_first() else {
            return Err(anyhow!("gamestring cannot be empty!"));
        };

        let setup = setup_str.trim().parse::<SetupString>()?;
        let mut moves = vec![];
        for move_str in movelist {
            let mv = move_str.trim().parse::<MoveString>()?;
            moves.push(mv);
        }

        Ok(GameString { setup, moves })
    }
}
