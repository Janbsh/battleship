pub const BOARD_SIZE: usize = 10;

pub const SHIP_LENGTHS: [usize; 5] = [5, 4, 3, 3, 2];

/// Defines the possible states for any single coordinate on the grid.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cell {
    Empty,
    Ship,
    Hit,
    Miss,
}

/// Represents a player's 10x10 grid and the ships or shots placed upon it.
pub struct Board {
    pub cells: [[Cell; BOARD_SIZE]; BOARD_SIZE],
}

impl Board {
    /// Creates a fresh board where every cell is initialized to empty.
    pub fn new() -> Self {
        Self {
            cells: [[Cell::Empty; BOARD_SIZE]; BOARD_SIZE],
        }
    }

    /// Attempts to place a ship of a specific length at the given coordinates.
    pub fn place_ship(
        &mut self,
        x: usize,
        y: usize,
        length: usize,
        horizontal: bool,
    ) -> Result<(), &'static str> {
        // ensure the ship doesn't extend beyond the board boundaries.
        if horizontal && x + length > BOARD_SIZE {
            return Err("The ship would stick out the right side of the board!");
        }
        if !horizontal && y + length > BOARD_SIZE {
            return Err("The ship would stick out the bottom of the board!");
        }

        // check if any of the target cells are already occupied by another ship.
        for i in 0..length {
            let (nx, ny) = if horizontal { (x + i, y) } else { (x, y + i) };
            if self.cells[ny][nx] == Cell::Ship {
                return Err("Oops! You're trying to put a ship on top of another one.");
            }
        }

        // mark the validated cells as part of a ship.
        for i in 0..length {
            let (nx, ny) = if horizontal { (x + i, y) } else { (x, y + i) };
            self.cells[ny][nx] = Cell::Ship;
        }

        Ok(())
    }

    /// Updates the board state based on an incoming shot.
    pub fn receive_attack(&mut self, x: usize, y: usize) -> Cell {
        match self.cells[y][x] {
            Cell::Ship => {
                // record a hit if a ship was at these coordinates.
                self.cells[y][x] = Cell::Hit;
                Cell::Hit
            }
            Cell::Empty => {
                // record a miss if the water was empty.
                self.cells[y][x] = Cell::Miss;
                Cell::Miss
            }
            other => other,
        }
    }

    /// Determines if any ship cells remain on the board.
    pub fn is_all_sunk(&self) -> bool {
        // flatten the 2D array and check if any 'Ship' variant still exists.
        !self.cells.iter().flatten().any(|&cell| cell == Cell::Ship)
    }
}

/// Tracks both boards and manages the turn-based flow of the game.
pub struct GameState {
    pub my_board: Board,
    pub opponent_board: Board,
    pub my_turn: bool,
}

impl GameState {
    /// Initializes a new game state and determines who moves first.
    pub fn new(is_host: bool) -> Self {
        Self {
            my_board: Board::new(),
            opponent_board: Board::new(),
            my_turn: is_host,
        }
    }

    /// Checks if a local move is legal before sending it over the network.
    pub fn handle_my_move(&mut self, x: usize, y: usize) -> Result<(), &'static str> {
        // prevent moving if the turn indicator isn't currently active.
        if !self.my_turn {
            return Err("Patience! It's not your turn yet.");
        }

        // prevent redundant shots on already revealed cells.
        if self.opponent_board.cells[y][x] != Cell::Empty {
            return Err("You've already fired at this coordinate. Pick a new one!");
        }

        Ok(())
    }

    /// Processes an incoming attack from the opponent and flips the turn.
    pub fn handle_opponent_move(&mut self, x: usize, y: usize) -> Result<Cell, &'static str> {
        // validate that the opponent is actually allowed to move.
        if self.my_turn {
            return Err("The opponent is trying to move out of turn!");
        }

        let result = self.my_board.receive_attack(x, y);

        // return control to the local player after the opponent's shot.
        self.my_turn = true;
        Ok(result)
    }
}