use std::fmt;
use std::io::{self, Write};

#[derive(Debug, Clone, Copy, PartialEq)]
enum Player {
    X,
    O,
}

impl Player {
    fn other(&self) -> Player {
        match self {
            Player::X => Player::O,
            Player::O => Player::X,
        }
    }
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Player::X => write!(f, "X"),
            Player::O => write!(f, "O"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Cell {
    Empty,
    Occupied(Player),
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Cell::Empty => write!(f, " "),
            Cell::Occupied(player) => write!(f, "{}", player),
        }
    }
}

struct Board {
    cells: [[Cell; 3]; 3],
}

impl Board {
    fn new() -> Self {
        Board {
            cells: [[Cell::Empty; 3]; 3],
        }
    }

    fn display(&self) {
        println!("\nCurrent Board:");
        println!("  1   2   3");
        for (row_idx, row) in self.cells.iter().enumerate() {
            print!("{} ", row_idx + 1);
            for (col_idx, cell) in row.iter().enumerate() {
                print!("{}", cell);
                if col_idx < 2 {
                    print!(" | ");
                }
            }
            println!();
            if row_idx < 2 {
                println!(" -----------");
            }
        }
        println!();
    }

    fn is_valid_move(&self, row: usize, col: usize) -> bool {
        if row >= 3 || col >= 3 {
            return false;
        }
        self.cells[row][col] == Cell::Empty
    }

    fn make_move(&mut self, row: usize, col: usize, player: Player) -> bool {
        if self.is_valid_move(row, col) {
            self.cells[row][col] = Cell::Occupied(player);
            true
        } else {
            false
        }
    }

    fn check_win(&self, player: Player) -> bool {
        let target = Cell::Occupied(player);

        // Check rows
        for row in &self.cells {
            if row.iter().all(|&cell| cell == target) {
                return true;
            }
        }

        // Check columns
        for col in 0..3 {
            if (0..3).all(|row| self.cells[row][col] == target) {
                return true;
            }
        }

        // Check diagonals
        if (0..3).all(|i| self.cells[i][i] == target) {
            return true;
        }
        if (0..3).all(|i| self.cells[i][2 - i] == target) {
            return true;
        }

        false
    }

    fn is_full(&self) -> bool {
        self.cells
            .iter()
            .all(|row| row.iter().all(|&cell| cell != Cell::Empty))
    }
}

fn get_move() -> Result<(usize, usize), String> {
    print!("Enter row (1-3): ");
    io::stdout().flush().unwrap();
    
    let mut row_input = String::new();
    io::stdin()
        .read_line(&mut row_input)
        .map_err(|e| format!("Failed to read input: {}", e))?;
    
    let row = row_input
        .trim()
        .parse::<usize>()
        .map_err(|_| "Invalid row number".to_string())?;
    
    print!("Enter column (1-3): ");
    io::stdout().flush().unwrap();
    
    let mut col_input = String::new();
    io::stdin()
        .read_line(&mut col_input)
        .map_err(|e| format!("Failed to read input: {}", e))?;
    
    let col = col_input
        .trim()
        .parse::<usize>()
        .map_err(|_| "Invalid column number".to_string())?;
    
    if row < 1 || row > 3 || col < 1 || col > 3 {
        return Err("Row and column must be between 1 and 3".to_string());
    }
    
    Ok((row - 1, col - 1))
}

fn play_game() {
    println!("=================================");
    println!("   Welcome to Tic-Tac-Toe!      ");
    println!("=================================");
    println!("\nPlayer 1: X");
    println!("Player 2: O");
    println!("\nEnter positions as row and column numbers (1-3)");
    
    let mut board = Board::new();
    let mut current_player = Player::X;
    
    loop {
        board.display();
        println!("Player {}'s turn", current_player);
        
        match get_move() {
            Ok((row, col)) => {
                if board.make_move(row, col, current_player) {
                    // Check for win
                    if board.check_win(current_player) {
                        board.display();
                        println!("=================================");
                        println!("   Player {} wins! 🎉", current_player);
                        println!("=================================");
                        break;
                    }
                    
                    // Check for draw
                    if board.is_full() {
                        board.display();
                        println!("=================================");
                        println!("   It's a draw! 🤝");
                        println!("=================================");
                        break;
                    }
                    
                    // Switch player
                    current_player = current_player.other();
                } else {
                    println!("❌ Invalid move! That position is already occupied.");
                    println!("Please try again.\n");
                }
            }
            Err(e) => {
                println!("❌ Error: {}", e);
                println!("Please try again.\n");
            }
        }
    }
}

fn main() {
    play_game();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_board_is_empty() {
        let board = Board::new();
        for row in &board.cells {
            for cell in row {
                assert_eq!(*cell, Cell::Empty);
            }
        }
    }

    #[test]
    fn test_valid_move() {
        let mut board = Board::new();
        assert!(board.make_move(0, 0, Player::X));
        assert_eq!(board.cells[0][0], Cell::Occupied(Player::X));
    }

    #[test]
    fn test_invalid_move_occupied() {
        let mut board = Board::new();
        board.make_move(0, 0, Player::X);
        assert!(!board.make_move(0, 0, Player::O));
    }

    #[test]
    fn test_horizontal_win() {
        let mut board = Board::new();
        board.make_move(0, 0, Player::X);
        board.make_move(0, 1, Player::X);
        board.make_move(0, 2, Player::X);
        assert!(board.check_win(Player::X));
    }

    #[test]
    fn test_vertical_win() {
        let mut board = Board::new();
        board.make_move(0, 0, Player::O);
        board.make_move(1, 0, Player::O);
        board.make_move(2, 0, Player::O);
        assert!(board.check_win(Player::O));
    }

    #[test]
    fn test_diagonal_win() {
        let mut board = Board::new();
        board.make_move(0, 0, Player::X);
        board.make_move(1, 1, Player::X);
        board.make_move(2, 2, Player::X);
        assert!(board.check_win(Player::X));
    }

    #[test]
    fn test_anti_diagonal_win() {
        let mut board = Board::new();
        board.make_move(0, 2, Player::O);
        board.make_move(1, 1, Player::O);
        board.make_move(2, 0, Player::O);
        assert!(board.check_win(Player::O));
    }

    #[test]
    fn test_draw_condition() {
        let mut board = Board::new();
        // Fill board with no winner
        // X O X
        // O X X
        // O X O
        board.make_move(0, 0, Player::X);
        board.make_move(0, 1, Player::O);
        board.make_move(0, 2, Player::X);
        board.make_move(1, 0, Player::O);
        board.make_move(1, 1, Player::X);
        board.make_move(1, 2, Player::X);
        board.make_move(2, 0, Player::O);
        board.make_move(2, 1, Player::X);
        board.make_move(2, 2, Player::O);
        
        assert!(board.is_full());
        assert!(!board.check_win(Player::X));
        assert!(!board.check_win(Player::O));
    }

    #[test]
    fn test_player_switching() {
        let player = Player::X;
        assert_eq!(player.other(), Player::O);
        assert_eq!(player.other().other(), Player::X);
    }

    #[test]
    fn test_invalid_position_out_of_bounds() {
        let board = Board::new();
        assert!(!board.is_valid_move(3, 0));
        assert!(!board.is_valid_move(0, 3));
        assert!(!board.is_valid_move(5, 5));
    }
}
