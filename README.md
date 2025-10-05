# blits-rs

An engine for The Battle of LITS.

## usage

1. clone the project
```sh
git clone git@github.com:rsarvar1a/blits-rs
```

2. build the engine
```sh
cargo build --release
```

3. create an environment file
```sh
touch .env
```

4. run the engine
```sh
target/release/blits --help
```

## stats

`AMD Ryzen AI 9 HX 370; 12c(24t); 32GB@7500MHz`:
- prototype: `5.25 kN/s` (no evaluator)
- best effort: `13.51 MN/s` (no evaluator)
