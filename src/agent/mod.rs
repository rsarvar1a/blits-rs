mod evaluator;
mod game;

use std::time::Duration;

use crate::battle_of_lits::prelude::*;

pub use evaluator::Evaluator;
pub use game::LITSGame;
use minimax::{strategies::mcts, IterativeOptions, MCTSOptions, ParallelOptions, Strategy};

/// An implementation of the actual blits engine.
pub struct BLITSAgent {
    board: Board<'static>,
    strategy: Box<dyn Strategy<LITSGame>>,
    piecemap: &'static PieceMap,
    past: Vec<usize>,
    past_boards: Vec<Board<'static>>,
    future: Vec<usize>
}

impl BLITSAgent {
    /// Creates a new board. If a symbol map is provided, initializes that board, otherwise generates one.
    /// 
    /// This method does _NOT_ handle the entire game string. That's because any user of the agent needs to
    /// synchronize the board states across all of its players, so it holds the responsibility of:
    /// 1. initializing each board with the setup string
    /// 2. playing each move in order to allow the agents to build their linear histories
    pub fn new(&mut self, setup_str: Option<SetupString>) {
        self.board = Board::new(setup_str.map(|v| v.grid), self.piecemap);
        [self.past, self.future] = [vec![], vec![]];
        self.past_boards = vec![];
    }

    /// Plays a move on the board if it is legal. If the move is a redo, then just redo it and maintain the future history.
    pub fn play_move(&mut self, mv: usize) -> Result<()> {
        if self.future.last().is_some_and(|&next| next == mv) {
            self.redo_move()
        } else {
            self.past_boards.push(self.board.clone());
            match mv {
                NULL_MOVE => self.board.pass()?,
                _         => self.board.play(mv)?
            };
            self.past.push(mv);
            self.future.clear(); // if the move didn't match the redo, then the future is wiped
            Ok(())
        }
    }

    /// Redo a move, if any - this maintains the linear history.
    pub fn redo_move(&mut self) -> Result<()> {
        if let Some(mv) = self.future.pop() {
            self.past_boards.push(self.board.clone());
            match mv {
                NULL_MOVE => self.board.pass()?,
                _         => self.board.play(mv)?
            };
            self.past.push(mv);
            Ok(())
        } else {
            Err(anyhow!("no move to redo"))
        }
    }

    /// Swaps on the board, if possible. If redoing the swap, then just redo it manually and maintain the future history.
    pub fn swap(&mut self) -> Result<()> {
        if self.future.last().is_some_and(|&next| next == NULL_MOVE) {
            self.redo_move()
        } else {
            self.past_boards.push(self.board.clone());
            self.board.pass()?;
            self.past.push(NULL_MOVE);
            self.future.clear();
            Ok(())
        }
    }

    /// Undoes a move on the board if it is legal.
    pub fn undo_move(&mut self) -> Result<usize> {
        if let Some(mv) = self.past.pop() {
            self.board = self.past_boards.pop().unwrap();
            self.future.push(mv);
            Ok(mv)
        } else {
            Err(anyhow!("no move to undo"))
        }
    }

    /// Generates the best move in the current position.
    pub fn generate_move(&mut self) -> Result<usize> {
        self.strategy.choose_move(&self.board).ok_or(
            anyhow!("failed to generate a move")
        )
    }

    /// Gets the principal variation.
    pub fn principal_variation(&self) -> Vec<usize> {
        self.strategy.principal_variation()
    }

    /// Configures the max depth on the search.
    pub fn set_max_depth(&mut self, depth: u8) {
        self.strategy.set_max_depth(depth);
    }

    /// Configures the timeout on the search.
    pub fn set_max_time(&mut self, time: Duration) {
        self.strategy.set_timeout(time);
    }

    pub fn with_board(&mut self, board: &Board<'static>) {
        self.board = board.clone();
        [self.past, self.future] = [vec![], vec![]];
        self.past_boards = vec![];
    }
}

pub enum WhichStrategy {
    MCTS,
    Negamax
}

pub struct AgentConfig {
    pub search_opts: minimax::IterativeOptions,
    pub parallel_opts: minimax::ParallelOptions,
    pub mcts_opts: minimax::MCTSOptions,
    pub selected: WhichStrategy,
}

impl Default for AgentConfig {
    fn default() -> Self {
        AgentConfig { 
            search_opts: IterativeOptions::new()
                .with_countermoves()
                .with_countermove_history()
                .with_table_byte_size(200 << 20),
            parallel_opts: ParallelOptions::new()
                .with_num_threads(std::thread::available_parallelism().map_or(1, |v| v.into())),
            mcts_opts: MCTSOptions::default()
                .with_num_threads(std::thread::available_parallelism().map_or(1, |v| v.into())),
            selected: WhichStrategy::Negamax
        }
    }
}

impl AgentConfig {
    /// Gets the default configuration for BLITS.
    pub fn new() -> AgentConfig {
        AgentConfig::default()
    }

    /// Produces an agent.
    pub fn get_agent(&self, piecemap: &'static PieceMap) -> BLITSAgent {
        match self.selected {
            WhichStrategy::Negamax => BLITSAgent { 
                board: Board::new(None, piecemap), 
                strategy: Box::new(minimax::ParallelSearch::new(Evaluator::default(), self.search_opts, self.parallel_opts)),
                piecemap,
                past: vec![],
                past_boards: vec![],
                future: vec![] 
            },
            WhichStrategy::MCTS => BLITSAgent { 
                board: Board::new(None, piecemap), 
                strategy: Box::new(mcts::MonteCarloTreeSearch::new(self.mcts_opts.clone())), 
                piecemap, 
                past: vec![], 
                past_boards: vec![], 
                future: vec![]
            }
        }
    }
}
