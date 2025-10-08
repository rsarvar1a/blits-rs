# Battle of LITS Evaluator - Architectural Decision Record

## Status
Proposed - Implementation phase

## Context

The Battle of LITS engine requires an evaluation function to assess non-terminal board positions for minimax search. The game is a territorial control abstract strategy game where players place tetromino pieces (L, I, T, S) on a 10x10 grid containing pre-placed scoring symbols (X and O). The objective is to maximize your score by covering opponent symbols while protecting your own.

### Game Mechanics Analysis

**Core Rules:**
- Players alternate placing tetromino pieces on a 10x10 board
- Each piece must touch at least one previously placed piece (connectivity)
- No 2x2 foursquare can be completely filled with pieces (foursquare constraint)
- Each player has 5 pieces of each type (L, I, T, S)
- Game ends when no valid moves remain
- Score = visible X symbols - visible O symbols

**Strategic Elements:**
- **Territorial Control**: Covering opponent symbols increases relative score
- **Protection**: Preventing opponent access to your symbols
- **Connectivity**: Maintaining piece networks for future placement options
- **Resource Management**: Limited pieces of each type
- **Constraint Navigation**: Working around foursquare restrictions

### Available Board State Information

1. **Grid**: 10x10 array of cells containing LITS tiles and scoring symbols
2. **Cover Set**: CoordSet of all cells covered by pieces
3. **Neighbors Set**: CoordSet of all cells adjacent to placed pieces (potential placement locations)
4. **Score**: Current visible score (X count - O count)
5. **Foursquare Counter**: Tracks occupancy of all 2x2 regions (3 bits per foursquare)
6. **Edge Counter**: Tracks which tile colors border each cell (3 bits per color per cell)
7. **Piece Bags**: Count of remaining pieces per type
8. **Player to Move**: Current player

## Decision

### Evaluation Function Design

Based on research into abstract strategy game evaluation and tetromino placement heuristics, we propose a multi-component evaluator that balances immediate material advantage with positional considerations:

#### Component 1: Material Balance (Weight: 1.0)
```
material_score = current_visible_score
```
- Direct measurement of current scoring advantage
- Foundation metric that all other components modify

#### Component 2: Territorial Security (Weight: 100.0)
```
security_score = Σ(foursquare_protected_symbols)
```
- Counts scoring symbols in cells with foursquare protection (3+ adjacent pieces)
- These are "secure" points unlikely to be covered by opponent
- High weight reflects that protected symbols are extremely valuable

#### Component 3: Territorial Threat (Weight: -25.0)
```
threat_score = Σ(accessible_opponent_symbols / distance_factor)
```
- Evaluates opponent symbols that could be reached and covered
- Applies distance decay to prioritize immediate threats
- Negative weight as opponent accessible symbols are disadvantageous

#### Component 4: Connectivity Potential (Weight: 15.0)
```
connectivity_score = neighbor_cells_with_own_symbols
```
- Counts cells in neighbor set containing own scoring symbols
- Measures future placement potential for extending territorial control
- Encourages maintaining board presence for future opportunities

#### Component 5: Constraint Pressure (Weight: -10.0)
```
constraint_score = cells_with_high_foursquare_pressure
```
- Counts cells where foursquare constraint creates placement restrictions
- Penalizes positions with limited future mobility
- Encourages maintaining placement flexibility

#### Component 6: Piece Diversity (Weight: 5.0)
```
diversity_score = -variance(piece_bag_counts)
```
- Penalizes uneven piece usage
- Encourages balanced piece consumption to maintain strategic options
- Lower weight as it's a secondary consideration

### Final Evaluation Formula
```
evaluation = material_score + 
             100.0 * security_score + 
             -25.0 * threat_score + 
             15.0 * connectivity_score + 
             -10.0 * constraint_score + 
             5.0 * diversity_score
```

## Rationale

### Weight Selection Justification

**Material (1.0)**: Base unit - all other weights are relative to material advantage

**Security (100.0)**: Very high weight because protected symbols are essentially guaranteed points. Research shows territorial games heavily favor secure positions.

**Threat (-25.0)**: Moderate negative weight. While important, threats can sometimes be defended, so not as critical as actual security.

**Connectivity (15.0)**: Medium weight. Maintains board presence but shouldn't override immediate tactical concerns.

**Constraint (-10.0)**: Low-medium weight. Important for long-term planning but not immediately critical.

**Diversity (5.0)**: Low weight. Piece balance matters but shouldn't drive major decisions.

### Performance Considerations

**Computational Complexity**: All components can be computed efficiently using existing board state:
- Material: O(1) - already computed
- Security: O(k) where k = neighbor set size (typically small)
- Threat: O(k) - same traversal as security
- Connectivity: O(k) - single pass through neighbors
- Constraint: O(k) - single pass checking foursquare counts
- Diversity: O(1) - simple variance calculation

**Cache Efficiency**: Components reuse the same data structures (neighbor set, foursquare counter) minimizing memory access patterns.

### Comparison to Traditional Approaches

**vs Pure Material**: Accounts for positional factors that pure material count misses
**vs Territorial Go-style**: Adapts territorial concepts to tetromino placement constraints
**vs Tetris Heuristics**: Borrows height/hole concepts but adapts for competitive territorial control

## Implementation Strategy

1. **Compute efficiently in single pass through neighbor set**
2. **Use existing optimized data structures (CoordSet, foursquare counter)**
3. **Avoid expensive operations like move generation**
4. **Leverage piece adjacency patterns for threat assessment**

## Alternative Approaches Considered

**Pure Greedy**: Only material score - too simplistic for strategic depth
**Monte Carlo Evaluation**: Computationally expensive, breaks performance requirements
**Pattern Recognition**: Complex to implement, unclear benefit over heuristic approach
**Machine Learning**: Requires training data, adds complexity without clear advantage

## Future Enhancements

1. **Adaptive Weights**: Adjust weights based on game phase (early/mid/endgame)
2. **Threat Depth**: Multi-move threat analysis for deeper tactical awareness
3. **Pattern Recognition**: Identify recurring strategic patterns for bonus scoring
4. **Opponent Modeling**: Adapt evaluation based on opponent playing style

## Success Metrics

1. **Performance**: Evaluation must complete in < 1ms for typical board positions
2. **Playing Strength**: Engine should demonstrate improved tactical and positional play
3. **Search Efficiency**: Better evaluation should reduce search depth requirements
4. **Stability**: Evaluation should be consistent across similar positions

---

**Author**: BLITS Engine Team  
**Date**: 2025-01-08  
**Version**: 1.0  
**Status**: Ready for Implementation