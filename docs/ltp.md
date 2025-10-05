
# The LITS Text Protocol

A standard for LITS notation across clients and engines.

# Notation 

The rules for notating LITS positions.

## Tiles 

### Individual Representations

The following describes the notation for each individual component of a tile.

```
player X: 'x' | 'X'
player O: 'o' | 'O'
```

```
colour L: 'l' | 'L'
colour I: 'i' | 'I'
colour T: 't' | 'T'
colour S: 's' | 'S'
```

```
blank   : '-' | '_'
```

### Tile State 

The overall state of a tile is an index representation of its symbol.

## Tetrominoes

### Coordinate 

A coordinate of the form (i, j) notates to 'ij'.

Example:

```
(4, 5)
```

notates to 

```
45 
```

### Tetromino 

The overall notation for a tetromino is the tetromino colour followed by a list of coordinates.

Example:

```
L L L - - - - - - -
L - - - - - - - - -
- - - - - - - - - -
- - - - - - - - - -
- - - - - - - - - -
- - - - - - - - - -
- - - - - - - - - -
- - - - - - - - - -
- - - - - - - - - -
- - - - - - - - - -
```

notates to 

```
L[00,01,02,10]
```

## Board Views 

### Board Position

The board state is a hashstring encoding the symbol map. We can represent the symbol map solely by the presence of X,
because Os are computed 180-degree rotations on X; so a board requires 100 bits, or 20 quintets. 

- each quintet on the board can be represented with a base-32 character, which is the set [0-9A-Z]
- the quintets can be read into a 100-bit bitstring
- cell r, c is the (10r+c)th bit of the bitstring, 0-indexed
- so the hashstring of the board is only 20 characters

Example:

```
-- x- -- -- -- x- -- x- -- --
-- -- -- -- -- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --
```

notates to `01000 10100 ...` which is `8J ...`

```
# TODO: hashstring example
```

(newlines added for clarity).

## Game and History 

The game state consists of a base board (the turn-0 setup position) 
followed by each tetromino placed in the move history on sequential 
lines.

Example:

```
-- x- -- -- o- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --

-L xL -L -- o- -- -- -- -- --
-L -- -- -- -- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --
```

notates to 

```
# TODO: hashstring example
L[00,01,02,10]
```

# Commands 

A minimal set of commands that must be supported by a LITS text protocol engine.

```
"cancel-search"  : Cancels an ongoing search request.

"gen-move"       : Requests that the engine find the best move in this position.

"initialize"     : Initializes the backing engine.

"new-game"       : Starts a blank new game.

"play-move"      : Plays the given move into the current position.
  param <piece>       the notation of a tetromino 

"setup-position" : Starts a new game with the given board position. 
  param <board>       the hashstring of a board position

"shutdown"       : Halts the backing engine.

"undo-move"      : Rewinds the position to the previous move, if possible.
```
