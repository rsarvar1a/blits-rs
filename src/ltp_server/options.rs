use clap::Parser;
use crate::prelude::*;

#[derive(Clone, Debug, Parser)]
pub struct LTPServerOptions {
    #[arg(short, long)]
    pub log_level: Option<String>,

    #[arg(short, long)]
    pub num_threads: Option<usize>,

    #[arg(short, long, default_value_t = false)]
    pub mcts: bool,

    #[arg(short, long, default_value_t = true)]
    pub ponder: bool,

    #[arg(short, long, default_value_t = false)]
    pub quiescence: bool,

    #[arg(long)]
    pub table_mb: Option<usize>,

    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,

    #[arg(short, long)]
    pub window: Option<usize>,
}

impl LTPServerOptions {
    pub fn agent_config(&self) -> AgentConfig {
        let mut config = AgentConfig::default();

        if let Some(num_threads) = self.num_threads {
            config.parallel_opts = config.parallel_opts.with_num_threads(num_threads);
            config.mcts_opts = config.mcts_opts.with_num_threads(num_threads);
        }
        if self.mcts {
            config.selected = WhichStrategy::MCTS;
        }
        if self.ponder {
            config.parallel_opts = config.parallel_opts.with_background_pondering();
        }
        if self.quiescence {
            config.search_opts = config.search_opts.with_quiescence_search_depth(3);
        }
        if let Some(table_size) = self.table_mb {
            config.search_opts.table_byte_size = table_size.checked_shl(20).unwrap();
        }
        if self.verbose {
            config.search_opts = config.search_opts.verbose();
            config.mcts_opts = config.mcts_opts.verbose();
        }
        if let Some(window_size) = self.window {
            config.search_opts = config.search_opts.with_aspiration_window(window_size as minimax::Evaluation);
        }
        
        config
    }
}
