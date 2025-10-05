pub(crate) mod board_cell;
pub(crate) mod foursquare;
pub(crate) mod indexing;
pub(crate) mod masks;
pub(crate) mod moves;
pub(crate) mod neighbours;
pub(crate) mod scores;
pub(crate) mod zobrist;

use crate::battle_of_lits::tetromino::piecemap::PieceMap;

use super::prelude::*;

use board_cell::BoardCell;
use foursquare::FoursquareCounter;
use masks::Mask;
use neighbours::EdgeCounter;


/// The grid of cells on a LITS board.
#[derive(Clone, Copy, Debug, Default)]
pub struct Grid(pub [[BoardCell; BOARD_SIZE]; BOARD_SIZE]);

impl Grid {
    pub fn notate(&self, was_swapped: bool) -> String {
        self.0.map(|row| { 
            row.map(|cell| { 
                cell.cell_value().map_or(".".into(), |v| { 
                    if was_swapped { (-v).notate() } else { v.notate() }
                })
            }).join("")
        }).join("")
    }
}

/// A bit-implementation of a board, stored as a 10x10 of u8s.
#[derive(Clone, Debug)]
pub struct Board<'a> {
    /// A grid of squares on the board, each containing an X, O, or neither, and possibly one of the four game tiles.
    cells: Grid,

    /// A reference counter for each colour on the board that tells us how many tiles of a given colour border each cell in the grid.  
    ///
    /// Each value in this mask is a quartet of three-bit trios, each of which counts edges for one of the four LITS tiles.  
    ///
    /// `refcount = (edge_mask[r][c] >> offset_of(tile)) & 0b00000111`
    edge_mask: EdgeCounter,

    /// A population counter for every foursquare on the that tells us whether a given foursquare (indexed by its top-left corner) is full or not.
    ///
    /// Each tile on the grid updates up to 4 foursquares, but the operation is only as large as the size of the piece (compared to edge detection, which
    /// requires checking every distinct neighbour across all cells covered by the played piece).
    foursquare_mask: FoursquareCounter,

    /// The linear history of this game, stored as a list of piece ids.
    /// 
    /// Id-based storage is useful because while linear history operations require a list,
    /// we can quickly obtain a moveset for conflict resolution operations like move validity.
    history: Vec<usize>,

    /// The number of pieces remaining in each type.
    piece_bag: [usize; 4],

    /// A reference to the built piecemap, so we can avoid an RWLock and threadsafe mechanisms that add overhead.
    pub piecemap: &'a PieceMap,

    /// Store the player to move instead of using parity because of the swap rule.
    player_to_move: Player,

    /// Denotes if the game is in the pie rule swap state.
    swapped: bool,

    /// The canonial hash for the gamestate.
    zobrist_hash: u64,

    /// A cache of valid moves for the previous position (i.e. lacking the most recently played move).
    _valid_moves_cache: [Option<FastSet>; PIECES_PER_KIND * 4 + 1]
}

impl<'a> Board<'a> {
    /// Returns a mask of all covered cells on the grid.
    pub fn covered(&self) -> Mask {
        self._covered()
    }

    /// Determines if the gamestate is such that O can swap.
    pub fn can_swap(&self) -> bool {
        self.swapped == false && self.history.len() == 1
    }

    /// Determines the scoring symbol at a given row and column on the board, if any exists.
    pub fn cell(&self, coord: &Coord) -> Result<Option<Player>> {
        self.get(coord).map(|v: BoardCell| v.cell_value())
    }

    /// Determines the "effective score" (i.e. the heuristic score) of the board.
    pub fn effective_score(&self) -> i16 {
        self._true_effective_score() * (self.player_to_move.perspective() as i16)
    }

    /// Determines the tile covering the cell at a given row and column on the board, if any tile exists.
    pub fn lits(&self, coord: &Coord) -> Result<Option<Tile>> {
        self.get(coord).map(|v: BoardCell| v.lits_value())
    }

    /// Returns a new board. If a symbol map is provided, use it, otherwise generate one.
    /// 
    /// This method does NOT handle gamestrings with moves, by design. This is because any user of a board
    /// is keeping a linear history, and must populate it by parsing and playing each piece, so the board
    /// will always receive the necessary (in-order) calls to Board::play().
    pub fn new<'p>(symbols: Option<Grid>, piecemap: &'p PieceMap) -> Board<'p> {
        let cells = {
            if let Some(grid) = symbols {
                grid // we delegated this parsing to the notation module :)
            } else {
                Grid(<[[BoardCell; BOARD_SIZE]; BOARD_SIZE]>::default()) // TODO(soft): generate instead
            }
        };
        
        let mut b = Board { 
            cells, 
            piecemap,
            edge_mask: EdgeCounter::default(),
            foursquare_mask: FoursquareCounter::default(),
            piece_bag: [PIECES_PER_KIND; 4],
            history: vec![],
            swapped: false,
            player_to_move: Player::X,
            zobrist_hash: Board::initial_zobrist_hash(&cells),
            _valid_moves_cache: [const { None }; PIECES_PER_KIND * 4 + 1]
        };

        b._valid_moves_cache[0] = Some(FastSet::from_iter(0..NUM_PIECES)); 
        b
    }

    /// Gets the greedy evaluation of this move. The greedy evaluation is the difference in enemy vs. self tiles covered, plus 
    /// the difference in self vs. enemy tiles protected by foursquare.
    pub fn noise(&self, mv: usize) -> i32 {
        if mv == NULL_MOVE { 
            return 6; // the swap is always noisy, just because we want to encourage exploring it
        }
        let piece = self.piecemap.get_piece(mv);
        let true_coverage = piece.real_coords().iter().map(|c| {
            let Coord { row, col } = c.coerce();
            self.cells.0[row][col].cell_value().map_or(0, |v| -v.perspective()) // covering a player's tile is scoring for the opposite player
        }).sum::<i32>();
        let true_protection = {
            let mut foursquare = self.foursquare_mask.clone();
            for coord in piece.real_coords() {
                foursquare.update_unchecked(&coord.coerce(), Some(piece.kind));
            }
            piece.neighbours().iter().map(|c| { // the on-board neighbours of this piece
                if self.lits_unchecked(c).is_some() { // this is covered by a different tile, so it's not protected 
                    return 0;
                }
                // uncovered tile scores in favour of the owning player, obviously
                foursquare.three(c) as i32 * self.cell_unchecked(c).map_or(0, |v| v.perspective())
            }).sum::<i32>()
        };
        let score = (true_coverage + true_protection) * self.player_to_move.perspective();
        score
    }

    /// Picks the noisy moves; i.e. those moves that are greedy score swings for the current player.
    /// 
    /// Greedy moves are pieces that cover & protect extremely favourably for the current player.
    /// 
    /// If the swap is available, always returns it.
    pub fn noisy_moves(&self) -> FastSet {
        self.valid_moves().into_iter().filter(|&mv| {
            self.noise(*mv) >= 3
        }).collect()
    }

    /// Returns the full gamestring for this board. If a swap was played, the gamestring is mindful of this fact,
    /// and the starting positional fragment is a negation of the current visible board.
    pub fn notate(&self) -> String {
        let mut fragments: Vec<String> = vec![];
        fragments.push(self.cells.notate(self.swapped)); // 

        for (i, mv) in self.history.iter().enumerate() {
            fragments.push(self.piecemap.notate(*mv));
            if i == 0 && self.swapped {
                fragments.push("swap".into());
            }
        }
        fragments.join(";")
    }

    /// Implements the swap rule.
    /// 
    /// In this engine, we cannot change the colour assigned to a playing agent, so we must instead recontextualize the board to support the worldview of the swap.
    /// 1. negate the symbols on the board, so that O is now "playing as if they were X", which is normally what would happen
    /// 2. give control back to X by flipping the player to move
    /// 
    /// As a neat consequence, the swap operation is symmetric - to unswap, we need to re-negate the board and hand control back to O.  
    pub fn pass(&mut self) -> Result<()> {
        if self.history.len() == 1 {
            self.swap();
            Ok(())
        } else {
            Err(anyhow!("passes are only legal on the first turn"))
        }
    }

    /// Passes unchecked; engine use only.
    pub fn pass_unchecked_engine(&mut self) -> () {
        self.swap();
    }

    /// Plays a move on this board, if valid.
    pub fn play(&mut self, mv: usize) -> Result<()> {
        if self.valid_moves().contains(&mv) {
            self.play_unchecked(&self.piecemap.get_piece(mv), mv);
            Ok(())
        } else {
            Err(anyhow!("move {mv} is not valid in this position"))
        }
    }

    /// Plays a piece with no checks onto the board; engine only.
    pub fn play_unchecked_engine(&mut self, mv: usize) -> () {
        self.play_unchecked(&self.piecemap.get_piece(mv), mv);
    }

    /// Determines the current player to move. X is the player when the number of played moves is even,
    /// since they start the game off at 0 moves on board.
    pub fn player_to_move(&self) -> Player {
        self.player_to_move
    }

    /// Gets the naive score on the board in X's perspective.
    pub fn score(&self) -> i32 {
        self.cells.0.iter().map(|row| {
            row.iter().map(|cell| { 
                if cell.covered() { return 0; };
                cell.cell_value().map_or(0, |v| v.perspective())
            }).sum::<i32>()
        }).sum::<i32>()
    }

    /// Sets the value of the cell at a given row and column on the board.
    pub fn set_cell(&mut self, coord: &Coord, cell: Option<Player>) -> Result<&mut Self> {
        let r = self.get_mut(coord)?;
        *r = r.with_cell(cell);
        Ok(self)
    }

    /// Sets the tile covering the cell at a given row and column on the board.
    pub fn set_lits(&mut self, coord: &Coord, lits: Option<Tile>) -> Result<&mut Self> {
        let [cur, prev] = {
            let r = self.get_mut(coord)?;
            let prev = r.lits_value();
            *r = r.with_lits(lits);
            [r.lits_value(), prev]
        };
        self.edge_mask.update(coord, cur, prev)?;
        self.foursquare_mask.update(coord, cur)?;
        Ok(self)
    }

    /// Undoes a move on the board. Pretty much all that's necessary here is:
    /// 1. the tetromino is in bounds, and 
    /// 2. all cells covered by this tetromino are of the expected type
    /// 
    /// Condition 2 holds true because if the board is otherwise consistent, all cells of the matching type
    /// can only flood-fill into the desired shape of the tile, otherwise the board would be in 
    /// violation of the rule prohibiting tiles of the same type from sharing an edge.
    pub fn undo(&mut self, mv: usize) -> Result<()> {
        if self.history.last().map_or(false, |&v| v == mv) {
            self.undo_unchecked(&self.piecemap.get_piece(mv), mv);
            Ok(())
        } else {
            let real_prev = self.history.last();
            Err(anyhow!("move {mv} is not the last piece in this position, {real_prev:?} is"))
        }
    }

    // Undoes the passing operation.
    pub fn unpass(&mut self) -> Result<()> {
        if self.swapped && self.history.len() == 1 {
            self.swap();
            Ok(())
        } else {
            Err(anyhow!("can only unpass if the last move was to swap"))
        }
    }

    /// Unpasses; engine use only.
    pub fn unpass_unchecked_engine(&mut self) -> () {
        self.swap();
    }

    /// Undoes a move with no checks; engine use only.
    pub fn undo_unchecked_engine(&mut self, mv: usize) -> () {
        self.undo_unchecked(&self.piecemap.get_piece(mv), mv);
    }

    /// Returns a set of valid moves in the current position. Does so using _m a g i c_, computing 99% of
    /// validity checks in constant time and saving n-piece foursquare detection for last.
    pub fn valid_moves(&self) -> &FastSet {
        self._valid_moves_cache[self.history.len()].as_ref().unwrap()
    }

    /// Gets a hash for the position. Since the searcher maintains an instance over
    /// multiple games, we need both the symbol zobrist and the move zobrist.
    /// Associativity of XOR makes it pretty easy to write; each bit of the output hash
    /// is set if and only if an odd number of component hashes are set at that bit.
    pub fn zobrist(&self) -> u64 {
        self.zobrist_hash
    }
}
