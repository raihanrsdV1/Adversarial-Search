// This file contains the Board struct and its core game logic implementation.
// It uses items from the `game` module. The AI logic is now separate.

use std::collections::{HashMap, VecDeque};
use std::fs::{File, OpenOptions};
use std::io::Write;
use crate::game::{Player, Cell, GameState, CellState};

// --- Board Struct ---
#[derive(Clone)]
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

        File::create(&log_filename).expect("Failed to create log file.");

        Board {
            width,
            height,
            cells,
            orb_counts,
            current_turn: first_turn,
            game_state: GameState::Ongoing,
            total_moves: 0,
            log_filename,
        }
    }

    pub fn make_move(&mut self, row: usize, col: usize) -> Result<(), &'static str> {
        if self.game_state != GameState::Ongoing {
            return Err("The game has already been won.");
        }
        if row >= self.height as usize || col >= self.width as usize {
            return Err("Move is out of bounds.");
        }
        if let CellState::Occupied { player, .. } = self.cells[row][col].state {
            if player != self.current_turn {
                return Err("Cannot place orb in a cell occupied by the opponent.");
            }
        }

        self.cells[row][col].add_orb(self.current_turn);
        *self.orb_counts.get_mut(&self.current_turn).unwrap() += 1;

        self.handle_chain_reaction(row, col);
        self.update_game_state();

        if self.game_state == GameState::Ongoing {
            self.current_turn = match self.current_turn {
                Player::Red => Player::Blue,
                Player::Blue => Player::Red,
            };
        }
        
        self.total_moves += 1;
        Ok(())
    }

    pub fn log_move(&self, player: Player, row: usize, col: usize) {
        let mut file = OpenOptions::new()
            .append(true)
            .open(&self.log_filename)
            .expect("Cannot open log file.");
        let move_str = format!("{:?} {} {}\n", player, row, col);
        file.write_all(move_str.as_bytes())
            .expect("Failed to write to log file.");
    }

    fn handle_chain_reaction(&mut self, start_row: usize, start_col: usize) {
        let mut exploding_cells: VecDeque<(usize, usize)> = VecDeque::new();
        if self.cells[start_row][start_col].get_explosion_data().is_some() {
            exploding_cells.push_back((start_row, start_col));
            self.cells[start_row][start_col].is_queued = true;
        }

        while let Some((r, c)) = exploding_cells.pop_front() {
            if let Some((exploding_player, current_orbs)) = self.cells[r][c].get_explosion_data() {
                let crit_mass = self.cells[r][c].critical_mass;
                let remaining_orbs = current_orbs.saturating_sub(crit_mass);
                
                *self.orb_counts.get_mut(&exploding_player).unwrap() -= crit_mass;

                self.cells[r][c].state = if remaining_orbs > 0 {
                    CellState::Occupied { player: exploding_player, orbs: remaining_orbs }
                } else {
                    CellState::Empty
                };
                self.cells[r][c].is_queued = false;

                let neighbors: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
                for (dr, dc) in neighbors.iter() {
                    let neighbor_r = r as isize + dr;
                    let neighbor_c = c as isize + dc;

                    if neighbor_r >= 0 && neighbor_r < self.height as isize &&
                        neighbor_c >= 0 && neighbor_c < self.width as isize {
                        let nr = neighbor_r as usize;
                        let nc = neighbor_c as usize;
                        
                        let prev_state = self.cells[nr][nc].state;
                        self.cells[nr][nc].take_over(exploding_player);

                        if let CellState::Occupied { player: prev_player, orbs: prev_orbs } = prev_state {
                            if prev_player != exploding_player {
                                *self.orb_counts.get_mut(&prev_player).unwrap() -= prev_orbs;
                                *self.orb_counts.get_mut(&exploding_player).unwrap() += prev_orbs + 1;
                            } else {
                                *self.orb_counts.get_mut(&exploding_player).unwrap() += 1;
                            }
                        } else {
                            *self.orb_counts.get_mut(&exploding_player).unwrap() += 1;
                        }

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
            }
        }
    }
    
    fn update_game_state(&mut self) {
        if self.total_moves < 2 { return; }

        let red_orbs = self.orb_counts[&Player::Red];
        let blue_orbs = self.orb_counts[&Player::Blue];

        if red_orbs > 0 && blue_orbs == 0 {
            self.game_state = GameState::Won { winner: Player::Red };
        } else if blue_orbs > 0 && red_orbs == 0 {
            self.game_state = GameState::Won { winner: Player::Blue };
        }
    }

    pub fn print(&self) {
        println!("--- Turn: {:?} | Game: {:?} | Orbs: R-{} B-{} ---", self.current_turn, self.game_state, self.orb_counts[&Player::Red], self.orb_counts[&Player::Blue]);
        for row in &self.cells {
            for cell in row {
                match cell.state {
                    CellState::Empty => print!("[ ] "),
                    CellState::Occupied { player, orbs } => {
                        let symbol = if player == Player::Red { 'R' } else { 'B' };
                        print!("[{}{}] ", orbs, symbol);
                    }
                }
            }
            println!();
        }
    }

    // These two methods remain on Board because they are direct queries about the board's state.
    // The AI module will call them.
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
}
