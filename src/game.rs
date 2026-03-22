pub const BOARD_SIZE: usize = 10;

pub const SHIP_LENGTHS: [usize; 5] = [5, 4, 3, 3, 2];

/// Board cell type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cell {
    Empty,
    Ship,
    Hit,
    Miss,
}

/// Game board.
pub struct Board {
    pub cells: [[Cell; BOARD_SIZE]; BOARD_SIZE],
}

impl Board {
    /// New board.
    pub fn new() -> Self {
        Self {
            cells: [[Cell::Empty; BOARD_SIZE]; BOARD_SIZE],
        }
    }

    /// Place a ship.
    pub fn place_ship(
        &mut self,
        x: usize,
        y: usize,
        length: usize,
        horizontal: bool,
    ) -> Result<(), &'static str> {
        // Bounds check.
        if horizontal && x + length > BOARD_SIZE {
            return Err("The ship would stick out the right side of the board!");
        }
        if !horizontal && y + length > BOARD_SIZE {
            return Err("The ship would stick out the bottom of the board!");
        }

        // Overlap check.
        for i in 0..length {
            let (nx, ny) = if horizontal { (x + i, y) } else { (x, y + i) };
            if self.cells[ny][nx] == Cell::Ship {
                return Err("Oops! You're trying to put a ship on top of another one.");
            }
        }

        for i in 0..length {
            let (nx, ny) = if horizontal { (x + i, y) } else { (x, y + i) };
            self.cells[ny][nx] = Cell::Ship;
        }

        Ok(())
    }

    /// Receive attack.
    pub fn receive_attack(&mut self, x: usize, y: usize) -> Cell {
        match self.cells[y][x] {
            Cell::Ship => {
                self.cells[y][x] = Cell::Hit;
                Cell::Hit
            }
            Cell::Empty => {
                self.cells[y][x] = Cell::Miss;
                Cell::Miss
            }
            other => other,
        }
    }

    /// Check if game over.
    pub fn is_all_sunk(&self) -> bool {
        !self.cells.iter().flatten().any(|&cell| cell == Cell::Ship)
    }
}

/// Game state.
pub struct GameState {
    pub my_board: Board,
    pub opponent_board: Board,
    pub my_turn: bool,
}

impl GameState {
    /// New game state.
    pub fn new(is_host: bool) -> Self {
        Self {
            my_board: Board::new(),
            opponent_board: Board::new(),
            my_turn: is_host,
        }
    }

    /// Validate move.
    pub fn handle_my_move(&mut self, x: usize, y: usize) -> Result<(), &'static str> {
        if !self.my_turn {
            return Err("Patience! It's not your turn yet.");
        }

        if self.opponent_board.cells[y][x] != Cell::Empty {
            return Err("You've already fired at this coordinate. Pick a new one!");
        }

        Ok(())
    }

    /// Handle opponent move.
    pub fn handle_opponent_move(&mut self, x: usize, y: usize) -> Result<Cell, &'static str> {
        if self.my_turn {
            return Err("The opponent is trying to move out of turn!");
        }

        let result = self.my_board.receive_attack(x, y);
        self.my_turn = true;
        Ok(result)
    }
}
