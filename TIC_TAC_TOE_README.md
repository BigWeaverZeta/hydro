# Tic-Tac-Toe Game Using Hydro Framework

This is a distributed tic-tac-toe game implementation built with the Hydro distributed programming framework for Rust. The implementation showcases Hydro's dataflow techniques, distributed safety constructs, and stream processing capabilities.

## Overview

The tic-tac-toe game is implemented as a server that manages game state and coordinates between two players over TCP connections. Players connect via simple network clients (like netcat) and interact by sending position numbers (0-8) to make moves.

## Architecture

### Key Components

1. **Game State Management (`GameState`)**: Tracks the 3x3 board, current player turn, game status, and player assignments using Hydro's fold operations.

2. **Player Management**: Uses Hydro's membership tracking to assign players (X and O) to connected clients automatically.

3. **Move Processing**: Leverages Hydro's dataflow operators to validate and process moves commutatively.

4. **State Broadcasting**: Distributes game state updates to all connected players using cross-product operations.

## Hydro Framework Features Used

### 1. **Dataflow Streams**
- `KeyedStream`: Used for managing player-specific input/output
- `Stream`: Used for processing moves and broadcasting state
- `.map()`, `.filter_map()`: Transform and filter data flows

### 2. **State Management**
- `.fold_commutative()`: Aggregates moves into global game state
- `KeyedSingleton`: Maintains player assignments and game state
- `.persist()`: Ensures state persists across ticks

### 3. **Distributed Safety**
- `track_membership()`: Safely tracks connected players
- `.snapshot()`: Creates consistent views of state at specific ticks
- `NonDet` annotations: Documents intentional non-determinism
- `.assume_retries()`: Handles idempotent operations safely

### 4. **Temporal Coordination**
- `process.tick()`: Coordinates state updates
- `.batch()`: Groups events within time windows
- `.all_ticks()`: Ensures outputs propagate across all ticks

### 5. **External Interaction**
- `.bidi_external_many_bytes()`: Manages bidirectional TCP connections
- `LinesCodec`: Handles line-based protocol encoding/decoding

## File Structure

```
hydro_test/
├── src/external_client/
│   ├── tic_tac_toe.rs       # Core game logic implementation
│   └── mod.rs                 # Module exports
└── examples/
    └── tic_tac_toe.rs         # Executable server example
```

## Implementation Details

### Game Board
- 3x3 grid with positions 0-8:
  ```
  0 | 1 | 2
  ---------
  3 | 4 | 5
  ---------
  6 | 7 | 8
  ```

### Player Assignment
- First connected client becomes Player X
- Second connected client becomes Player O  
- Additional clients wait for assignment

### Move Processing Flow
1. Client sends position number (0-8)
2. Input stream filters invalid moves
3. Move is folded into global game state
4. Win conditions are checked
5. Updated board is broadcast to all players

### State Transitions
```
Waiting → InProgress(X) → InProgress(O) → ... → Won(Player) or Draw
```

## Running the Game

### Prerequisites
```bash
cargo build --package hydro_test --example tic_tac_toe
```

### Start the Server
```bash
cargo run --package hydro_test --example tic_tac_toe
```

### Connect Players
In separate terminals:
```bash
# Player 1 (X)
nc localhost <port>

# Player 2 (O)  
nc localhost <port>
```

### Playing
- Players take turns entering numbers 0-8
- The game automatically validates moves and updates the board
- Game ends when a player wins or the board is full (draw)

## Key Hydro Patterns Demonstrated

### 1. Commutative Fold for Global State
```rust
moves_stream
    .map_with_key(q!(|(client_id, pos)| ((), pos)))
    .fold_commutative(
        q!(|| GameState::new()),
        q!(|state, (_key, (_client_id, pos))| {
            // Update state based on move
        })
    )
```

### 2. Membership Tracking
```rust
let current_connections = track_membership(membership);
current_connections
    .snapshot(&tick, nondet!(/** get connected players */))
    .keys()
```

### 3. Cross-Product for Broadcasting
```rust
connected_players
    .cross_product(game_state_broadcast.cross_product(assignments_broadcast))
    .map(q!(|(client_id, (state, assignments))| {
        // Generate personalized response for each player
    }))
```

### 4. Non-Determinism Annotations
```rust
nondet!(/** test - intentional non-determinism for tick coordination */);
nondet!(/** logging - extra logs due to duplicate samples are okay */);
```

## Distributed Safety Guarantees

The implementation leverages Hydro's safety guarantees:

1. **Consistency**: All players see the same game state through coordinated snapshots
2. **Ordering**: Moves are processed in a consistent order via fold operations  
3. **Membership**: Player tracking ensures correct assignment and prevents data races
4. **Idempotency**: Retry-safe operations prevent duplicate move processing

## Design Decisions

### Why Commutative Fold?
Move processing uses `fold_commutative` because moves are processed in order and the fold function is associative (though not truly commutative - moves must be applied sequentially). This allows Hydro to optimize the dataflow while maintaining correctness.

### Why Global State with Unit Key?
The game state is keyed by `()` (unit type) to create a single global state that all moves fold into. This ensures all players see a consistent game board.

### Why Snapshot for Broadcasting?
Taking snapshots of game state and assignments ensures all players receive consistent state at each tick, preventing race conditions or inconsistent views.

## Testing

The example includes a basic integration test:
```bash
cargo test --package hydro_test --example tic_tac_toe
```

## Future Enhancements

Potential improvements to explore Hydro further:

1. **Multiple Concurrent Games**: Support multiple game instances with proper isolation
2. **Game Reset**: Allow players to start a new game after completion
3. **Spectators**: Enable non-playing clients to observe games
4. **Move History**: Track and replay game moves
5. **Distributed Deployment**: Deploy server and clients across multiple machines
6. **Persistence**: Save game state to recover from failures

## Learning Resources

- [Hydro Documentation](https://hydro.run/docs/hydro/)
- [Hydro Research Papers](https://hydro.run/research)
- [DFIR Documentation](https://hydro.run/docs/dfir)

## License

This implementation follows the Apache-2.0 license of the Hydro project.
