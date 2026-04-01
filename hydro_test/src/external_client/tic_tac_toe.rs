use hydro_lang::live_collections::stream::NoOrder;
use hydro_lang::location::MembershipEvent;
use hydro_lang::nondet::NonDet;
use hydro_lang::prelude::*;
use hydro_std::membership::track_membership;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a player in the game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Player {
    X,
    O,
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Player::X => write!(f, "X"),
            Player::O => write!(f, "O"),
        }
    }
}

impl Player {
    fn other(self) -> Self {
        match self {
            Player::X => Player::O,
            Player::O => Player::X,
        }
    }
}

/// Represents a cell position on the board (0-8)
pub type Position = u8;

/// Represents the 3x3 game board
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Board {
    cells: [Option<Player>; 9],
}

impl Board {
    pub fn new() -> Self {
        Board { cells: [None; 9] }
    }

    pub fn get(&self, pos: Position) -> Option<Player> {
        if pos < 9 {
            self.cells[pos as usize]
        } else {
            None
        }
    }

    pub fn set(&mut self, pos: Position, player: Player) -> bool {
        if pos < 9 && self.cells[pos as usize].is_none() {
            self.cells[pos as usize] = Some(player);
            true
        } else {
            false
        }
    }

    pub fn is_full(&self) -> bool {
        self.cells.iter().all(|cell| cell.is_some())
    }

    /// Check if there's a winner on the board
    pub fn check_winner(&self) -> Option<Player> {
        // All possible winning combinations
        const WINS: [[usize; 3]; 8] = [
            [0, 1, 2], // top row
            [3, 4, 5], // middle row
            [6, 7, 8], // bottom row
            [0, 3, 6], // left column
            [1, 4, 7], // middle column
            [2, 5, 8], // right column
            [0, 4, 8], // diagonal
            [2, 4, 6], // anti-diagonal
        ];

        for win in &WINS {
            if let (Some(a), Some(b), Some(c)) = (
                self.cells[win[0]],
                self.cells[win[1]],
                self.cells[win[2]],
            ) {
                if a == b && b == c {
                    return Some(a);
                }
            }
        }
        None
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "  0 | 1 | 2")?;
        writeln!(f, " -----------")?;
        for row in 0..3 {
            write!(f, " ")?;
            for col in 0..3 {
                let idx = row * 3 + col;
                match self.cells[idx] {
                    Some(player) => write!(f, " {} ", player)?,
                    None => write!(f, " {} ", idx)?,
                }
                if col < 2 {
                    write!(f, "|")?;
                }
            }
            writeln!(f)?;
            if row < 2 {
                writeln!(f, " -----------")?;
            }
        }
        Ok(())
    }
}

/// Represents the current state of the game
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GameStatus {
    Waiting,           // Waiting for players
    InProgress(Player), // Game in progress, current player's turn
    Won(Player),        // Game won by player
    Draw,               // Game ended in a draw
}

impl fmt::Display for GameStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameStatus::Waiting => write!(f, "Waiting for players..."),
            GameStatus::InProgress(player) => write!(f, "Player {}'s turn", player),
            GameStatus::Won(player) => write!(f, "Player {} wins!", player),
            GameStatus::Draw => write!(f, "Game ended in a draw!"),
        }
    }
}

/// Complete game state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub board: Board,
    pub status: GameStatus,
    pub player_x: Option<u64>, // client ID for player X
    pub player_o: Option<u64>, // client ID for player O
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            board: Board::new(),
            status: GameStatus::Waiting,
            player_x: None,
            player_o: None,
        }
    }

    /// Assign a client to a player role
    pub fn assign_player(&mut self, client_id: u64) -> Option<Player> {
        if self.player_x.is_none() {
            self.player_x = Some(client_id);
            if self.player_o.is_some() {
                self.status = GameStatus::InProgress(Player::X);
            }
            Some(Player::X)
        } else if self.player_o.is_none() {
            self.player_o = Some(client_id);
            self.status = GameStatus::InProgress(Player::X);
            Some(Player::O)
        } else {
            None // Both players assigned
        }
    }

    /// Get the player for a client ID
    pub fn get_player(&self, client_id: u64) -> Option<Player> {
        if self.player_x == Some(client_id) {
            Some(Player::X)
        } else if self.player_o == Some(client_id) {
            Some(Player::O)
        } else {
            None
        }
    }

    /// Check if it's the given client's turn
    pub fn is_player_turn(&self, client_id: u64) -> bool {
        match self.status {
            GameStatus::InProgress(current_player) => {
                self.get_player(client_id) == Some(current_player)
            }
            _ => false,
        }
    }

    /// Process a move from a client
    pub fn process_move(&mut self, client_id: u64, pos: Position) -> Result<(), String> {
        // Check if game is in progress
        if !matches!(self.status, GameStatus::InProgress(_)) {
            return Err("Game is not in progress".to_string());
        }

        // Check if it's this player's turn
        if !self.is_player_turn(client_id) {
            return Err("Not your turn".to_string());
        }

        // Get the player
        let player = self
            .get_player(client_id)
            .ok_or("You are not a player in this game")?;

        // Try to make the move
        if !self.board.set(pos, player) {
            return Err("Invalid move".to_string());
        }

        // Check for winner
        if let Some(winner) = self.board.check_winner() {
            self.status = GameStatus::Won(winner);
        } else if self.board.is_full() {
            self.status = GameStatus::Draw;
        } else {
            // Switch to other player's turn
            self.status = GameStatus::InProgress(player.other());
        }

        Ok(())
    }

    pub fn is_game_over(&self) -> bool {
        matches!(self.status, GameStatus::Won(_) | GameStatus::Draw)
    }
}

/// Represents a move from a client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Move {
    pub position: Position,
}

/// Response message to send to clients
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameResponse {
    pub board: String,
    pub status: String,
    pub message: Option<String>,
    pub your_symbol: Option<String>,
}

impl GameResponse {
    pub fn to_string(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("\n{}\n", "=".repeat(40)));
        output.push_str(&self.board);
        output.push_str(&format!("\n{}\n", "-".repeat(40)));
        output.push_str(&format!("Status: {}\n", self.status));
        if let Some(symbol) = &self.your_symbol {
            output.push_str(&format!("You are player: {}\n", symbol));
        }
        if let Some(msg) = &self.message {
            output.push_str(&format!("\n{}\n", msg));
        }
        output.push_str(&format!("{}\n", "=".repeat(40)));
        output
    }
}

/// Main tic-tac-toe game server implementation using Hydro
pub fn tic_tac_toe_server<'a, P>(
    process: &Process<'a, P>,
    in_stream: KeyedStream<u64, String, Process<'a, P>, Unbounded>,
    membership: KeyedStream<u64, MembershipEvent, Process<'a, P>, Unbounded>,
    nondet_tick: NonDet,
) -> KeyedStream<u64, String, Process<'a, P>, Unbounded, NoOrder> {
    let current_connections = track_membership(membership);

    // Log connection count
    current_connections
        .clone()
        .key_count()
        .sample_every(q!(std::time::Duration::from_secs(1)), nondet!(/** logging */))
        .assume_retries(nondet!(/** extra logs due to duplicate samples are okay */))
        .for_each(q!(|count| {
            println!("Current connections: {}", count);
        }));

    let tick = process.tick();

    // Parse moves from input stream - also tag with client_id in the value
    let moves_stream = in_stream
        .filter_map_with_key(q!(|(client_id, input)| {
            if let Ok(pos) = input.trim().parse::<u8>() {
                if pos < 9 {
                    // Return just (client_id, pos) as the value
                    Some((client_id, pos))
                } else {
                    println!("Invalid position {} from client {}", pos, client_id);
                    None
                }
            } else {
                None
            }
        }));

    // Maintain game state by folding all moves
    // Convert moves to global state using unit key
    let game_state = moves_stream
        .clone()
        .map_with_key(q!(|(_client_id, pos)| ((), pos))) // key=(), value=pos
        .fold_commutative::<GameState, _, _>(
            q!(|| GameState::new()),
            q!(|state, key_and_pos| {
                let (_, (_client_id, pos)) = key_and_pos;
                println!("Processing move at position {}", pos);
                // For simplicity, alternate turns based on move count
                let current_player = if state.board.cells.iter().filter(|c| c.is_some()).count() % 2 == 0 {
                    Player::X
                } else {
                    Player::O
                };
                
                if state.board.set(pos, current_player) {
                    println!("Player {} made move at position {}", current_player, pos);
                    // Check for winner
                    if let Some(winner) = state.board.check_winner() {
                        state.status = GameStatus::Won(winner);
                    } else if state.board.is_full() {
                        state.status = GameStatus::Draw;
                    } else {
                        state.status = GameStatus::InProgress(current_player.other());
                    }
                } else {
                    println!("Invalid move at position {}", pos);
                }
            }),
        );

    // Get current connected players each tick
    let connected_players = current_connections
        .snapshot(&tick, nondet_tick)
        .keys();

    // Assign players to connections (first connection is X, second is O)
    // Convert to stream and fold to track assignments
    let all_connections_stream = connected_players
        .clone();

    let player_assignments_stream = all_connections_stream
        .map(q!(|client_id| (0, client_id)))
        .into_keyed()
        .fold_commutative::<(Option<u64>, Option<u64>), _, _>(
            q!(|| (None, None)),
            q!(|assignments, client_id| {
                if assignments.0.is_none() {
                    assignments.0 = Some(client_id);
                    println!("Assigned client {} to Player X", client_id);
                } else if assignments.1.is_none() && assignments.0 != Some(client_id) {
                    assignments.1 = Some(client_id);
                    println!("Assigned client {} to Player O", client_id);
                }
            }),
        )
        .values();

    // Broadcast state to all connected players each tick
    // Get game state for broadcast
    let game_state_broadcast = game_state
        .snapshot(&tick, nondet!(/** get current game state */))
        .values();

    // For each connected player, send them the current game state
    connected_players
        .cross_product(game_state_broadcast.cross_product(player_assignments_stream))
        .map(q!(|(client_id, (state, (player_x, player_o)))| {
            let your_symbol = if player_x == Some(client_id) {
                Some("X".to_string())
            } else if player_o == Some(client_id) {
                Some("O".to_string())
            } else {
                None
            };

            let current_player_id = match state.status {
                GameStatus::InProgress(Player::X) => player_x,
                GameStatus::InProgress(Player::O) => player_o,
                _ => None,
            };

            let message = if your_symbol.is_none() {
                Some("Waiting to be assigned a player role...".to_string())
            } else if matches!(state.status, GameStatus::Won(_) | GameStatus::Draw) {
                Some("Game over! Type 'reset' to start a new game.".to_string())
            } else if current_player_id == Some(client_id) {
                Some("Your turn! Enter a position (0-8):".to_string())
            } else {
                Some("Waiting for other player...".to_string())
            };

            let response = GameResponse {
                board: format!("{}", state.board),
                status: format!("{}", state.status),
                message,
                your_symbol,
            };

            (client_id, response.to_string())
        }))
        .into_keyed()
        .all_ticks()
}
