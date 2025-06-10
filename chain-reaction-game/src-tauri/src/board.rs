use std::collections::{HashMap, VecDeque};
use std::fs::OpenOptions;
use std::io::Write;
use serde::Serialize;
use std::time::Instant;

// DTOs are no longer needed here as this module is now pure game logic.
use crate::game::{Player, Cell, GameState, CellState};

#[derive(Clone, Serialize)]
pub struct Board {
    pub width: u32,
    pub height: u32,
    pub cells: Vec<Vec<Cell>>,
    pub orb_counts: HashMap<Player, u32>,
    pub current_turn: Player,
    pub game_state: GameState,
    pub total_moves: u32,
    log_filename: String,
}

impl Board {
    // This helper is now in lib.rs, where it belongs.
    
    pub fn new(width: u32, height: u32, first_turn: Player, log_filename: String) -> Self {
        let mut cells = Vec::with_capacity(height as usize);
        for r in 0..height {
            let mut row = Vec::with_capacity(width as usize);
            for c in 0..width {
                let mut neighbours = 4;
                if r == 0 || r == height - 1 { neighbours -= 1; }
                if c == 0 || c == width - 1 { neighbours -= 1; }
                row.push(Cell::new(neighbours));
            }
            cells.push(row);
        }
        let mut orb_counts = HashMap::new();
        orb_counts.insert(Player::Red, 0);
        orb_counts.insert(Player::Blue, 0);

        Board { 
            width, height, cells, orb_counts, 
            current_turn: first_turn, 
            game_state: GameState::Ongoing, 
            total_moves: 0, 
            log_filename 
        }
    }
    
    // This now returns the Vec of board states for the controller to handle.
    pub fn make_move_and_get_history(&mut self, row: usize, col: usize) -> Result<Vec<Board>, &'static str> {
        self.log_move(self.current_turn, row, col);

        let result = self.make_move_internal(row, col, true, None);
        self.print_board_to_file(&self.log_filename);
        result
    }

    // The simulation function remains largely the same.
    pub fn make_move_for_simulation(&mut self, row: usize, col: usize, deadline: Option<&Instant>) -> Result<(), &'static str> {
        self.make_move_internal(row, col, false, deadline).map(|_| ())
    }

    // Returns a history Vec for real moves, and an empty one for simulations.
    fn make_move_internal(&mut self, row: usize, col: usize, is_real_move: bool, deadline: Option<&Instant>) -> Result<Vec<Board>, &'static str> {
        if self.game_state != GameState::Ongoing { return Err("The game has already been won."); }
        if row >= self.height as usize || col >= self.width as usize { return Err("Move is out of bounds."); }
        if let CellState::Occupied { player, .. } = self.cells[row][col].state {
            if player != self.current_turn { return Err("Cannot place orb in a cell occupied by the opponent."); }
        }
        
        let mut history = Vec::new();
        self.cells[row][col].add_orb(self.current_turn);
        
        self.handle_chain_reaction(row, col, is_real_move, deadline, &mut history)?;
        
        self.recalculate_orb_counts();
        self.update_game_state();

        if self.game_state == GameState::Ongoing {
            self.current_turn = match self.current_turn {
                Player::Red => Player::Blue,
                Player::Blue => Player::Red,
            };
        }
        
        self.total_moves += 1;
        
        if is_real_move {
            // Add the final state to the history.
             history.push(self.clone());
        }

        Ok(history)
    }
    
    fn recalculate_orb_counts(&mut self) {
        let mut red_orbs = 0;
        let mut blue_orbs = 0;
        for cell in self.cells.iter().flatten() {
            if let CellState::Occupied { player, orbs } = cell.state {
                match player {
                    Player::Red => red_orbs += orbs,
                    Player::Blue => blue_orbs += orbs,
                }
            }
        }
        self.orb_counts.insert(Player::Red, red_orbs);
        self.orb_counts.insert(Player::Blue, blue_orbs);
    }
    
    pub fn log_move(&self, player: Player, row: usize, col: usize) {
        // Print current working directory for debugging
        if let Ok(current_dir) = std::env::current_dir() {
            println!("Current working directory: {:?}", current_dir);
        }
        println!("Attempting to write to log file: {}", self.log_filename);
        
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_filename) {
            let move_str = format!("{:?} {} {}\n", player, row, col);
            if let Err(e) = file.write_all(move_str.as_bytes()) {
                eprintln!("Warning: Failed to write to log file: {}", e);
            } else {
                // Ensure the data is actually written to disk
                if let Err(e) = file.flush() {
                    eprintln!("Warning: Failed to flush log file: {}", e);
                } else {
                    println!("Successfully logged move: {:?} {} {} to file: {}", player, row, col, self.log_filename);
                }
            }
        } else {
            eprintln!("Warning: Could not open log file: {}", self.log_filename);
        }
    }
    
    // Now only populates a history vec instead of emitting events.
    fn handle_chain_reaction(&mut self, start_row: usize, start_col: usize, is_real_move: bool, deadline: Option<&Instant>, history: &mut Vec<Board>) -> Result<(), &'static str> {
        let mut exploding_cells: VecDeque<(usize, usize)> = VecDeque::new();
        
        if self.cells[start_row][start_col].get_explosion_data().is_some() {
            exploding_cells.push_back((start_row, start_col));
            self.cells[start_row][start_col].is_queued = true;
        }

        while let Some((r, c)) = exploding_cells.pop_front() {
            println!("Processing explosion at ({}, {})", r, c);

            if let Some(d) = deadline {
                println!("Checking deadline: {:?}", d);
                if Instant::now() >= *d {
                    return Err("Chain reaction timed out during simulation.");
                }
            }

            if let Some((exploding_player, current_orbs)) = self.cells[r][c].get_explosion_data() {
                let crit_mass = self.cells[r][c].critical_mass;
                let remaining_orbs = current_orbs.saturating_sub(crit_mass);
                self.cells[r][c].state = if remaining_orbs > 0 { CellState::Occupied { player: exploding_player, orbs: remaining_orbs } } else { CellState::Empty };
                self.cells[r][c].is_queued = false;

                let neighbors: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
                for (dr, dc) in neighbors.iter() {
                    let neighbor_r = r as isize + dr;
                    let neighbor_c = c as isize + dc;
                    if neighbor_r >= 0 && neighbor_r < self.height as isize && neighbor_c >= 0 && neighbor_c < self.width as isize {
                        let nr = neighbor_r as usize;
                        let nc = neighbor_c as usize;
                        self.cells[nr][nc].take_over(exploding_player);
                        let neighbor_cell = &mut self.cells[nr][nc];
                        if neighbor_cell.get_explosion_data().is_some() && !neighbor_cell.is_queued {
                            exploding_cells.push_back((nr, nc));
                            neighbor_cell.is_queued = true;
                        }
                    }
                }
                
                let cell_after_explosion = &mut self.cells[r][c];
                if cell_after_explosion.get_explosion_data().is_some() && !cell_after_explosion.is_queued {
                    exploding_cells.push_back((r, c));
                    cell_after_explosion.is_queued = true;
                }
                
                self.recalculate_orb_counts();
                
                // If it's a real move, save the intermediate state for animation.
                if is_real_move {
                    history.push(self.clone());
                }
                
                self.update_game_state();
                if self.game_state != GameState::Ongoing {
                    break; 
                }
            }
        }
        Ok(())
    }
    
    fn update_game_state(&mut self) {
        if self.total_moves < 2 { return; }
        
        let red_orbs = self.orb_counts.get(&Player::Red).cloned().unwrap_or(0);
        let blue_orbs = self.orb_counts.get(&Player::Blue).cloned().unwrap_or(0);

        if red_orbs > 0 && blue_orbs == 0 {
            self.game_state = GameState::Won { winner: Player::Red };
        } else if blue_orbs > 0 && red_orbs == 0 {
            self.game_state = GameState::Won { winner: Player::Blue };
        }
    }

    pub fn get_all_valid_moves(&self) -> Vec<(usize, usize)> {
        let mut valid_moves = Vec::new();
        for r in 0..self.height as usize {
            for c in 0..self.width as usize {
                match self.cells[r][c].state {
                    CellState::Empty => {
                        valid_moves.push((r, c));
                    }
                    CellState::Occupied { player, .. } => {
                        if player == self.current_turn {
                            valid_moves.push((r, c));
                        }
                    }
                }
            }
        }
        valid_moves
    }

    // print the board on the file descibed in the file path. 
    pub fn print_board_to_file(&self, file_path: &str) {
        use std::fs::File;
        use std::io::Write;
        
        let mut file = File::create(file_path).expect("Could not open file");
        
        // Write header based on current player
        let move_type = match self.current_turn {
            Player::Red => "Human Move",
            Player::Blue => "AI Move",
        };
        writeln!(file, "{}:", move_type).expect("Failed to write");
        
        // Write board state
        for row in &self.cells {
            let mut row_parts = Vec::new();
            for cell in row {
                match cell.state {
                    CellState::Empty => row_parts.push("0".to_string()),
                    CellState::Occupied { player, orbs } => {
                        let player_char = match player {
                            Player::Red => 'R',
                            Player::Blue => 'B',
                        };
                        row_parts.push(format!("{}{}", orbs, player_char));
                    }
                }
            }
            writeln!(file, "{}", row_parts.join(" ")).expect("Failed to write");
        }
    }

}
