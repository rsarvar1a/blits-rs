
## meta

### `info`

returns some information about the BLITS engine

### `options`

TODO: gets and sets engine options 

### `quit`

quits the program

## mutations

### `newgame [gamestr]`

creates a new game of LITS

- `[gamestr]`: a game string (optional)
    - see [ltp.md](ltp.md) for more information on valid notation
    - if not provided, a random board is generated

### `play <movestr>`

applies a move to the current position, if legal

- `<movestr>`: a move string
    - see [ltp.md](ltp.md) for more information on valid notation

### `swap`

swaps X and O in the current position, if legal

### `undo`

reverts the most recent move in the current position, if one exists

## queries

### `bestmove <depth <int> | time <hh:mm:ss>>`

queries the engine for the best move in the current position

- `<depth ...>`: instructs the engine to search up to this depth
- `<time  ...>`: allots a maximum duration for this search

### `pv`

displays the principal variation
- requires that the board has not changed since the last `bestmove` operation

### `score`

returns the score on the board in X's perspective

### `validmoves`

returns all valid moves in the current position
