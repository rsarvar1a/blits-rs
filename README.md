# blits-rs

An engine for The Battle of LITS.

## usage

```sh
git clone git@github.com:rsarvar1a/blits-rs # clone the project
cargo build --release                       # build the BLITS engine and LTP server
touch .env                                  # create an empty .env (oops)
target/release/blits --help                 # see engine options
target/release/blits                        # serve LTP
```

See [docs/commands.md](docs/commands.md) for more information on interacting with the engine.

## milestones

### benchmarking conditions

- cpu: AMD Ryzen AI 9 HX 370 ( 12c / 24t ) @ 5.1 GHz
- ram: 32 GB LPDDR5X @ 7497 MHz
- BLITS settings: 
    - `--num-threads 24`
    - `--verbose`

### implementations

- gametree prototype:
    - $5.251\times10^3$ nodes per second
    - evaluator: none
        - returns 0 in all non-terminal states

- current best effort: 
    - $2.204\times10^7$ nodes per second
    - evaluator: connectivity
        - applies a multifaceted heuristic approach, including:
            - graph connectivity (the threat on the board in terms of region draining)
            - protected cells (guaranteed scoring)
            - unreachability (regions that are mathematically impossible to reach for the rest of the game)
