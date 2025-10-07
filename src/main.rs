#![feature(never_type)]

use std::time::Instant;

use clap::Parser;
use flexi_logger::{AdaptiveFormat, Logger, WriteMode};
use lib_blits::prelude::*;

fn main() -> Result<!> {
    // Initialize program options and environment.
    dotenvy::dotenv()?;
    let options = LTPServerOptions::parse();
    let _logger = Logger::try_with_env_or_str(options.log_level.clone().unwrap_or("info".into()).as_str())?
        .write_mode(WriteMode::BufferAndFlush)
        .log_to_stderr()
        .adaptive_format_for_stderr(
            match cfg!(debug_assertions) {
                true => AdaptiveFormat::WithThread,
                _    => AdaptiveFormat::Default
            })
        .set_palette("b196;208;195;111;67".to_owned())
        .start()?;

    // Serve LTP and the BLITS engine.
    let start_computing_piecemap = Instant::now();
    let piecemap = Box::leak(Box::new(PieceMap::new()));
    log::info!("ready in {:.2}s", (Instant::now() - start_computing_piecemap).as_secs_f64());
    
    let Err(e) = LTPServer::new(options, piecemap).run();
    log::error!("fatal error: {}", e);
    Err(e)
}
