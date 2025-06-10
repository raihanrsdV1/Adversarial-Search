use core::f64;
use std::collections::{HashMap, VecDeque};
use std::io::{self, Write}; // For user input and file writing
use std::fs::{File, OpenOptions}; // For creating and opening files
use rand::Rng; // For the random AI

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AIStrategy {
    /// The AI chooses a random valid move.
    Random,
    AlphaBeta,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Heuristic {
    OrbDifference,
    PeripheralControl,
    TerritoryControl,
    ChainReactionPotential,
    ConversionPotential,
    CascadePotential,
    SafeMobility,
}


// --- Enums and Structs ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Player {
    Red,
    Blue,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CellState {
    Empty,
    Occupied { player: Player, orbs: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameState {
    Ongoing,
    Won { winner: Player },
}

#[derive(Debug, Clone, Copy)]
pub struct Cell {
    state: CellState,
    critical_mass: u32,
    is_queued: bool,
}

impl Cell {
    fn new(critical_mass: u32) -> Self {
        Cell {
            state: CellState::Empty,
            critical_mass,
            is_queued: false,
        }
    }

    fn add_orb(&mut self, player: Player) -> bool {
        match self.state {
            CellState::Empty => {
                self.state = CellState::Occupied { player, orbs: 1 };
                true
            }
            CellState::Occupied { player: p, orbs } => {
                if p == player {
                    self.state = CellState::Occupied { player, orbs: orbs + 1 };
                    true
                } else {
                    false
                }
            }
        }
    }
    
    fn get_explosion_data(&self) -> Option<(Player, u32)> {
        if let CellState::Occupied { player, orbs } = self.state {
            if orbs >= self.critical_mass {
                return Some((player, orbs));
            }
        }
        None
    }

    fn take_over(&mut self, player: Player) {
        let orbs = match self.state {
            CellState::Occupied { orbs, .. } => orbs,
            CellState::Empty => 0,
        };
        self.state = CellState::Occupied { player, orbs: orbs + 1 };
    }
}

// --- Board Struct ---
#[derive(Clone)]
pub struct Board {
    width: u32,
    height: u32,
    cells: Vec<Vec<Cell>>,
    pub orb_counts: HashMap<Player, u32>,
    current_turn: Player,
    game_state: GameState,
    total_moves: u32,
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

        // self.log_move(self.current_turn, row, col);

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

    // Helper function to write a move to the log file
    fn log_move(&self, player: Player, row: usize, col: usize) {
        let mut file = OpenOptions::new()
            .append(true)
            .open(&self.log_filename)
            .expect("Cannot open log file.");

        // Format the move string as per the requirement
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
            // println!("Processing explosion at ({}, {})", r, c);
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

                        // --- CORRECTED ORB COUNTING LOGIC ---
                        if let CellState::Occupied { player: prev_player, orbs: prev_orbs } = prev_state {
                            if prev_player != exploding_player {
                                // The new owner gains all the previous owner's orbs + the new one.
                                // The previous owner loses all their orbs.
                                *self.orb_counts.get_mut(&prev_player).unwrap() -= prev_orbs;
                                *self.orb_counts.get_mut(&exploding_player).unwrap() += prev_orbs + 1;
                            } else {
                                // If owner is the same, they just gain one orb.
                                *self.orb_counts.get_mut(&exploding_player).unwrap() += 1;
                            }
                        } else {
                             // If the cell was empty, the new owner just gains one orb.
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

    fn print(&self) {
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
    
    // --- AI LOGIC STRUCTURE ---
    
    /// The main entry point for getting the AI's move.
    pub fn get_ai_move(&self, strategy: AIStrategy, heuristics: &[Heuristic], depth: u32) -> (usize, usize) {
        match strategy {
            AIStrategy::Random => {
                let mut rng = rand::thread_rng();
                loop {
                    let row = rng.gen_range(0..self.height as usize);
                    let col = rng.gen_range(0..self.width as usize);
                    let mut temp_board = self.clone();
                    if temp_board.make_move(row, col).is_ok() {
                        return (row, col);
                    }
                }
            }
            AIStrategy::AlphaBeta => {
                // This is the correct place to call the alpha-beta search.
                // It does NOT call back to get_ai_move.
                self.find_best_move_alphabeta(heuristics, depth)
            }
        }
    }
    
    /// Finds the best move using the alpha-beta algorithm. This is the top-level "manager" function.
    fn find_best_move_alphabeta(&self, heuristics: &[Heuristic], depth: u32) -> (usize, usize) {

        let mut best_move: (usize, usize) = (0, 0);
        let mut best_score = f64::INFINITY;
        let alpha = f64::NEG_INFINITY;
        let beta = f64::INFINITY;
        let possible_moves = self.get_all_valid_moves();
        if possible_moves.is_empty() {
            return (0, 0);
        }

        best_move = possible_moves[0];

        for a_move in possible_moves {
            let mut temp_board = self.clone();
            temp_board.make_move(a_move.0, a_move.1).unwrap();

            let score = self.alphabeta(temp_board, depth - 1, alpha, beta, false, heuristics);

            if(score < best_score) {
                best_score = score;
                best_move = a_move;
            }
        }
        return best_move;

        // unimplemented!("AlphaBeta search not yet implemented.");
    }
    
    /// The core recursive helper function for the alpha-beta algorithm.
    fn alphabeta(&self, board: Board, depth: u32, mut alpha: f64, mut beta: f64, is_maximizing_player: bool, heuristics: &[Heuristic]) -> f64 {

        if depth == 0 || board.game_state != GameState::Ongoing {
            return board.evaluate_board(heuristics);
        }

        let possible_moves = board.get_all_valid_moves();

        if is_maximizing_player {
            let mut max_eval = f64::NEG_INFINITY;
             for a_move in possible_moves {
                let mut child_board = board.clone();
                child_board.make_move(a_move.0, a_move.1).unwrap();

                let eval = self.alphabeta(child_board, depth - 1, alpha, beta, false, heuristics);
                max_eval = max_eval.max(eval);
                alpha = alpha.max(eval);

                if beta <= alpha {
                    break;
                }

                
             }
             return max_eval;
        }
        else {
            let mut min_eval = f64::INFINITY;
            for a_move in possible_moves {
                let mut child_board = board.clone();
                child_board.make_move(a_move.0, a_move.1).unwrap();
                let eval = self.alphabeta(child_board, depth - 1, alpha, beta, true, heuristics);
                min_eval = min_eval.min(eval);
                beta = beta.min(eval);
                if beta <= alpha {
                    break;
                }
            }
            return min_eval;
        }



    }
    
    /// A helper function to get all valid moves for the current player.
    fn get_all_valid_moves(&self) -> Vec<(usize, usize)> {

        let mut valid_moves = Vec::new();

        for r in 0..self.height as usize {
            for c in 0..self.width as usize {
                if let CellState::Occupied { player, .. } = self.cells[r][c].state {
                    if player == self.current_turn {
                        valid_moves.push((r, c));
                    }
                }
                else if self.cells[r][c].state == CellState::Empty {
                    // If the cell is empty, it is a valid move.
                    valid_moves.push((r, c));
                }
            }
        }

        return valid_moves;
        // unimplemented!();
    }

    /// Evaluates the board state from the perspective of the current player.
    fn evaluate_board(&self, heuristics: &[Heuristic]) -> f64 {
        let mut total_score = 0.0;
        let player = self.current_turn;

        for heuristic in heuristics {
            total_score += match heuristic {
                Heuristic::OrbDifference => {

                    let opponent = if player == Player::Red { Player::Blue } else { Player::Red };
                    let my_orbs = self.orb_counts[&player] as f64;
                    let opponent_orbs = self.orb_counts[&opponent] as f64;
                    my_orbs - opponent_orbs
                    // 0.0 // Placeholder
                }
                Heuristic::PeripheralControl => {
                    // corner points would be 3 and peripheral points would be 2.
                    let mut peripheral_score = 0.0;
                    
                    for r in 0..self.height as usize{
                        for c in 0..self.width as usize{
                            let corner_condition = r == 0 && c == 0 ||
                                                      r == 0 && c == self.width as usize - 1 ||
                                                      r == self.height as usize - 1 && c == 0 ||
                                                      r == self.height as usize - 1 && c == self.width as usize - 1;
                            let peripheral_condition = r == 0 || r == self.height as usize - 1 || c == 0 || c == self.width as usize - 1;

                            if let CellState::Occupied { player: cell_player, orbs: _ } = self.cells[r][c].state {
                                if cell_player == player {
                                    if corner_condition {
                                        peripheral_score += 3.0 ;
                                    }
                                    else if peripheral_condition {
                                        peripheral_score += 2.0; 
                                    }
                                    else {
                                        peripheral_score += 1.0; // Inner cells
                                    }
                                }
                                else if cell_player != player {
                                    if corner_condition {
                                        peripheral_score -= 3.0; 
                                    }
                                    else if peripheral_condition {
                                        peripheral_score -= 2.0; 
                                    }
                                    else {
                                        peripheral_score -= 1.0; 
                                    }
                                }

                            }
                            // else if self.cells[r][c].state == CellState::Empty {
                            //     if corner_condition {
                            //         peripheral_score -= 3.0; 
                            //     }
                            //     else if peripheral_condition {
                            //         peripheral_score -= 2.0; 
                            //     }
                            //     else {
                            //         peripheral_score -= 1.0; 
                            //     }
                            // }

                        }
                    }
                    
                    peripheral_score
                }

                Heuristic::TerritoryControl => {
                    let mut territory_score = 0.0;

                    for r in 0..self.height as usize {
                        for c in 0..self.width as usize {
                            if let CellState::Occupied { player: cell_player, orbs: _ } = self.cells[r][c].state {
                                if cell_player == player {
                                    territory_score += 1.0;
                                }
                            }
                        }
                    }

                    territory_score
                }

                Heuristic::ChainReactionPotential => {
                    let mut chain_reaction_score = 0.0;

                    for r in 0..self.height as usize {
                        for c in 0..self.width as usize {
                            if let CellState::Occupied { player: cell_player, orbs } = self.cells[r][c].state {
                                if cell_player == player {
                                    // Check if this cell can cause a chain reaction
                                    if orbs == self.cells[r][c].critical_mass - 1 {
                                        chain_reaction_score += 5.0; 
                                    }
                                } else {
                                    if orbs == self.cells[r][c].critical_mass - 1 {
                                        // it can cause a chain reaction
                                        chain_reaction_score -= 5.0; 
                                    }
                                    
                                }
                            } 
                        }
                    }

                    chain_reaction_score
                }

                Heuristic::ConversionPotential => {
                    let mut conversion_score = 0.0;
                    let neighbors: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
                    // how many opponents orcs can i transform into my own 
                    for r in 0..self.height as usize {
                        for c in 0..self.width as usize {
                            if let CellState::Occupied { player: cell_player, orbs } = self.cells[r][c].state {
                                if cell_player == player && orbs == self.cells[r][c].critical_mass - 1 {
                                    // finding the neighbours of this cell
                                    
                                    for (dr, dc) in neighbors.iter() {
                                        let neighbor_r = r as isize + dr;
                                        let neighbor_c = c as isize + dc;
                                        if neighbor_r >= 0 && neighbor_r < self.height as isize &&
                                           neighbor_c >= 0 && neighbor_c < self.width as isize {
                                            let nr = neighbor_r as usize;
                                            let nc = neighbor_c as usize;
                                            if let CellState::Occupied { player: neighbor_player, orbs: neighbor_orbs } = self.cells[nr][nc].state {
                                                if neighbor_player != player {
                                                    conversion_score += neighbor_orbs as f64; // Potential to convert opponent's cell
                                                }
                                            }
                                         }
                                    }
                                }
                                else if cell_player != player && orbs == self.cells[r][c].critical_mass - 1 {
                                    // finding the neighbours of this cell
                                    // let neighbors: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
                                    for (dr, dc) in neighbors.iter() {
                                        let neighbor_r = r as isize + dr;
                                        let neighbor_c = c as isize + dc;
                                        if neighbor_r >= 0 && neighbor_r < self.height as isize &&
                                           neighbor_c >= 0 && neighbor_c < self.width as isize {
                                            let nr = neighbor_r as usize;
                                            let nc = neighbor_c as usize;
                                            if let CellState::Occupied { player: neighbor_player, orbs: neighbor_orbs } = self.cells[nr][nc].state {
                                                if neighbor_player == player {
                                                    conversion_score -= neighbor_orbs as f64; // Potential to convert my own cell
                                                }
                                            }
                                         }
                                    }
                                }
                            }
                            
                        }
                    }

                    conversion_score
                }

                Heuristic::CascadePotential => {
                    let mut cascade_score = 0.0;
                    let neighbors: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

                    for r in 0..self.height as usize {
                        for c in 0..self.width as usize {
                            if let CellState::Occupied { player: cell_player , orbs} = self.cells[r][c].state {
                                if cell_player == player && orbs == self.cells[r][c].critical_mass - 1 {
                                    // checking neighbors for potential cascading chain reactions 
                                    for (dr, dc) in neighbors.iter() {
                                        let neighbor_r = r as isize + dr;
                                        let neighbor_c = c as isize + dc;
                                        if neighbor_r >= 0 && neighbor_r < self.height as isize &&
                                           neighbor_c >= 0 && neighbor_c < self.width as isize {
                                            let nr = neighbor_r as usize;
                                            let nc = neighbor_c as usize;
                                            if let CellState::Occupied { player: neighbor_player, orbs: neighbor_orbs } = self.cells[nr][nc].state {
                                               
                                                cascade_score += (neighbor_orbs as f64); // Potential to cause a cascade reaction

                                                if neighbor_orbs == self.cells[nr][nc].critical_mass - 1 {
                                                    cascade_score += 3.0; // Stronger potential for a chain reaction
                                                }
                                            }
                                         }
                                    }
                                }

                                else if cell_player != player && orbs == self.cells[r][c].critical_mass - 1 {
                                    // checking neighbors for potential cascading chain reactions 
                                    for (dr, dc) in neighbors.iter() {
                                        let neighbor_r = r as isize + dr;
                                        let neighbor_c = c as isize + dc;
                                        if neighbor_r >= 0 && neighbor_r < self.height as isize &&
                                           neighbor_c >= 0 && neighbor_c < self.width as isize {
                                            let nr = neighbor_r as usize;
                                            let nc = neighbor_c as usize;
                                            if let CellState::Occupied { player: neighbor_player, orbs: neighbor_orbs } = self.cells[nr][nc].state {
                                                // cascade_score -= (neighbor_orbs as f64) / (self.cells[nr][nc].critical_mass as f64) * 3.0; // Potential to cause a cascade reaction
                                                cascade_score -= (neighbor_orbs as f64); // Potential to cause a cascade reaction
                                                if neighbor_orbs == self.cells[nr][nc].critical_mass - 1 {
                                                    cascade_score -= 3.0; // Stronger potential for a chain reaction
                                                }
                                            }
                                         }
                                    }
                                }
                            }
                        }
                    }
                    cascade_score
                }

                Heuristic::SafeMobility => {
                    let mut safe_moves = 0.0;
                    let my_possible_moves = self.get_all_valid_moves();
                    let opponent = if player == Player::Red { Player::Blue } else { Player::Red };
                    for my_move in &my_possible_moves {
                        let mut board_after_my_move = self.clone();
                        board_after_my_move.make_move(my_move.0, my_move.1).unwrap();
                        let mut is_move_safe = true;
                        
                        let mut opponent_board_view = board_after_my_move.clone();
                        opponent_board_view.current_turn = opponent;
                        let opponent_replies = opponent_board_view.get_all_valid_moves();

                        for opp_reply in &opponent_replies {
                            let target_cell = board_after_my_move.cells[opp_reply.0][opp_reply.1];
                            let would_explode = match target_cell.state {
                                CellState::Occupied { orbs, .. } => orbs == target_cell.critical_mass + 1,
                                CellState::Empty => 1 >= target_cell.critical_mass,
                            };

                            if would_explode {
                                is_move_safe = false;
                                break;
                            }
                        }
                        if is_move_safe {
                            safe_moves += 1.0;
                        }
                    }
                    safe_moves
                }

            }
        }
        
        total_score
    }
}


/// The main game loop for a Human vs. AI match.
fn main() {
    let log_filename = "game_log.txt".to_string();
    let mut game_board = Board::new(6, 9, Player::Red, log_filename);
    let human_player = Player::Red;
    let ai_player = Player::Blue;

    // Set this to AlphaBeta to test the smart AI!
    let ai_strategy = AIStrategy::AlphaBeta; 
    let ai_heuristics = vec![
        // Heuristic::OrbDifference, 
        Heuristic::PeripheralControl,
        // Heuristic::ChainReactionPotential,
        // Heuristic::ConversionPotential,
        // Heuristic::SafeMobility,
        // Heuristic::CascadePotential,
    ];
    let search_depth = 3; // A depth of 4-5 is a good starting point.

    println!("You are Player {:?}. The AI is Player {:?}.", human_player, ai_player);

    loop {
        if let GameState::Won { winner } = game_board.game_state {
            println!("\n--- GAME OVER ---");
            println!("Player {:?} has won!", winner);
            game_board.print();
            break;
        }

        game_board.print();
        let current_player = game_board.current_turn;

        if current_player == human_player {
            println!("Your turn (enter 'row col'): ");
            let mut input = String::new();
            io::stdin().read_line(&mut input).expect("Failed to read line");

            let parts: Vec<Result<usize, _>> = input.trim().split_whitespace().map(|s| s.parse()).collect();

            if parts.len() == 2 && parts[0].is_ok() && parts[1].is_ok() {
                let row = *parts[0].as_ref().unwrap();
                let col = *parts[1].as_ref().unwrap();

                // Log the human's move here
                game_board.log_move(current_player, row, col);
                if let Err(e) = game_board.make_move(row, col) {
                    println!("Invalid move: {}", e);
                }
            } else {
                println!("Invalid input. Please use the format 'row col', e.g., '3 4'");
            }
        } else {
            println!("AI ({:?}) is thinking...", ai_player);
            let (row, col) = game_board.get_ai_move(ai_strategy, &ai_heuristics, search_depth);
            println!("AI moves to ({}, {})", row, col);
            // Log the AI's move here
            game_board.log_move(current_player, row, col);
            game_board.make_move(row, col).expect("AI made an invalid move!");
        }
        println!("\n---------------------------\n");
    }
}