mod options;

use std::{process::exit, time::Duration};

use itertools::Itertools;
pub use options::LTPServerOptions;

use crate::prelude::*;

pub struct LTPServer {
    agent: BLITSAgent,
    board: Option<Board<'static>>,
    past_boards: Vec<Board<'static>>,
    piecemap: &'static PieceMap,
    #[allow(dead_code)]
    config: LTPServerOptions,
    dirty: bool,
}

impl LTPServer {
    /// Produces a new LTP server with the given BLITS engine configuration.
    pub fn new(options: LTPServerOptions, piecemap: &'static PieceMap) -> LTPServer {
        LTPServer {
            agent: options.agent_config().get_agent(piecemap),
            board: None,
            past_boards: vec![],
            piecemap,
            config: options,
            dirty: true
        }
    }

    /// Runs BLITS in engine mode.
    pub fn run(&mut self) -> Result<!> {
        let a_bit = std::time::Duration::from_secs(2);
        std::thread::sleep(a_bit);

        loop
        {
            let mut cmdstr: String = String::new();
            std::io::stdin().read_line(&mut cmdstr)?;

            let args: Vec<&str> = cmdstr.split_whitespace().filter(|s| !s.is_empty()).collect();
            let cmd = *args.first().unwrap_or(&"");

            self.apply(cmd, &args[1..])?;
        }
    }

    /// Runs a command.
    fn apply(&mut self, cmd: &str, args: &[&str]) -> Result<()> {
        let result = match cmd
        {
            | "" => Ok(()),
            | "bestmove" => self.best_move(args),
            | "info" => self.info(),
            | "newgame" => self.new_game(args),
            | "options" => self.options(args),
            | "play" => self.play_move(args),
            | "pv" => self.principal_variation(args),
            | "quit" => exit(0),
            | "score" => self.score(args),
            | "swap" => self.play_move(&["swap"]),
            | "undo" => self.undo_move(args),
            | "validmoves" => self.valid_moves(args),
            | _ => Err(anyhow!("unrecognized command {cmd}")),
        };

        match result
        {
            Ok(_) => {
                log::debug!("Command completed successfully: {cmd} {}", args.join(" "));
                self.ok()
            },
            Err(err) => {
                log::warn!("encountered recoverable error:\n{err}");
                self.err(&err)
            },
        }
    }

    fn best_move(&mut self, args: &[&str]) -> Result<()> {
        self.ensure_started()?;

        if args.len() >= 2 {
            match args[0] {
                "depth" => {
                    let depth = args[1].parse::<u8>()?;
                    self.agent.set_max_depth(depth);
                },
                "time"  => {
                    let time = self.parse_hhmmss(args[1])?;
                    self.agent.set_max_time(time);
                },
                _       => { return Err(anyhow!("unrecognized search option {}", args[0])); }
            };
        }
        let mv = self.agent.generate_move()?;
        self.dirty = false;
        
        println!("{}", self.piecemap.notate(mv));
        Ok(())
    }

    /// Starts a new game, potentially from an advanced position (i.e. with a move history).
    fn new_game(&mut self, args: &[&str]) -> Result<()> {
        let gamestr = if !args.is_empty() {
            Some(args.join(" ").parse::<GameString>()?)
        } else {
            None
        };

        match gamestr {
            Some(s) => {
                let GameString { setup, moves } = s; {
                    self.board = Some(Board::new(Some(setup.grid), self.piecemap));
                    self.agent.new(Some(setup));
                }

                self.past_boards = vec![];
                for mv in moves {
                    self.past_boards.push(self.get().clone());
                    let MoveString { repr: _, tetromino } = mv;
                    match tetromino {
                        Some(t) => {
                            let index = self.piecemap.try_and_find(&t.real_coords())?;
                            self.get_mut().play(index)?;
                            self.agent.play_move(index)?;
                        },
                        None => {
                            self.get_mut().pass()?;
                            self.agent.play_move(NULL_MOVE)?;
                        }
                    }
                }
            },
            None => {
                self.board = Some(Board::new(None, self.piecemap));
                self.agent.with_board(&self.get().clone());
            }
        };
        self.dirty = true;

        println!("{}", self.get().notate());
        Ok(())
    }

    fn options(&mut self, _args: &[&str]) -> Result<()> {
        Ok(())
    }

    fn play_move(&mut self, args: &[&str]) -> Result<()> {
        self.ensure_started()?;

        if args.is_empty() {
            return Err(anyhow!("no move provided"));
        }

        self.past_boards.push(self.get().clone());

        let MoveString { repr: _, tetromino } = args[0].parse::<MoveString>()?;
        match tetromino {
            Some(t) => {
                let index = self.piecemap.try_and_find(&t.real_coords())?;
                self.get_mut().play(index)?;
                self.agent.play_move(index)?;
            },
            None    => {
                self.get_mut().pass()?;
                self.agent.play_move(NULL_MOVE)?;
            }
        };
        self.dirty = true;

        println!("{}", self.get().notate());
        Ok(())
    }

    fn principal_variation(&mut self, _args: &[&str]) -> Result<()> {
        self.ensure_started()?;

        if self.dirty {
            return Err(anyhow!("board changed since previous engine move"));
        }

        let pv = self.agent.principal_variation();
        let repr = pv.iter().map(|mv| self.piecemap.notate(*mv)).join("; ");
        println!("{}", repr);
        Ok(())
    }

    fn score(&mut self, _args: &[&str]) -> Result<()> {
        self.ensure_started()?;

        let score = self.get().score();
        println!("{}", score);
        Ok(())
    }

    fn undo_move(&mut self, _args: &[&str]) -> Result<()> {
        self.ensure_started()?;

        self.agent.undo_move()?;
        self.board = Some(self.past_boards.pop().unwrap());
        self.dirty = true;

        println!("{}", self.get().notate());
        Ok(())
    }

    fn valid_moves(&mut self, _args: &[&str]) -> Result<()> {
        self.ensure_started()?;
        let moves = self.get().valid_moves_set();
        let movestr = moves.iter().collect::<Vec<usize>>().iter().map(|i| self.piecemap.notate(*i)).join("; ");

        println!("{}", moves.len());
        println!("{}", movestr);
        Ok(())
    }

    // accessors

    fn ensure_started(&mut self) -> Result<&mut Board<'static>> {
        if self.board.is_none() {
            Err(anyhow!("no game in progress"))
        } else {
            Ok(self.get_mut())
        }
    }

    /// Retrieves the board in a shared context.
    fn get(&self) -> & Board<'static> {
        self.board.as_ref().unwrap()
    } 

    /// Retrieves the board in a mutable context.
    fn get_mut(&mut self) -> & mut Board<'static> {
        self.board.as_mut().unwrap()
    }

    // basic printers

    /// Prints the server's ID.
    fn info(&self) -> Result<()>
    {
        println!(
            "id {} v{}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        );
        Ok(())
    }

    /// Prints an error to the UHP stream.
    fn err(&self, err: &Error) -> Result<()>
    {
        println!("err\n{}", err);
        self.ok()
    }

    /// Prints the ok footer to the UHP stream.
    fn ok(&self) -> Result<()>
    {
        println!("ok");
        Ok(())
    }

    // parsers

    fn parse_hhmmss(&self, time: &str) -> Result<Duration> {
        let mut toks = time.split(':');
        let hours = toks.next().unwrap_or("").parse::<u64>()?;
        let minutes = toks.next().unwrap_or("").parse::<u64>()?;
        let seconds = toks.next().unwrap_or("").parse::<u64>()?;
        Ok(Duration::from_secs(hours * 3600 + minutes * 60 + seconds))
    }
}
