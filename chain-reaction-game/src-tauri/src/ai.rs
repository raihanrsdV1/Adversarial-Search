use crate::board::Board;
use crate::game::{Player, GameState, CellState};
use rand::Rng;
use std::time::{Instant, Duration};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AIStrategy {
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

pub fn get_ai_move(board: &Board, strategy: AIStrategy, heuristics: &[Heuristic], max_depth: u32, time_limit_ms: u64) -> (usize, usize) {
    match strategy {
        AIStrategy::Random => {
            let mut rng = rand::thread_rng();
            loop {
                let row = rng.gen_range(0..board.height as usize);
                let col = rng.gen_range(0..board.width as usize);
                let mut temp_board = board.clone();
                if temp_board.make_move_for_simulation(row, col, None).is_ok() {
                    return (row, col);
                }
            }
        }
        AIStrategy::AlphaBeta => {
            let start_time = Instant::now();
            let deadline = start_time + Duration::from_millis(time_limit_ms);

            let possible_moves = board.get_all_valid_moves();
            if possible_moves.is_empty() { return (0, 0); }
            
            let mut best_move_so_far = possible_moves[0];

            for d in 1..=max_depth {
                println!("Searching at depth {}", d);
                if Instant::now() >= deadline {
                    println!("Time limit reached before starting depth {}", d);
                    break; 
                }

                let result = find_best_move_at_depth(board, heuristics, d, &deadline);
                
                if let Some(found_move) = result {
                    best_move_so_far = found_move;
                } else {
                    println!("Search at depth {} timed out. Using best move from previous depth.", d);
                    break;
                }
            }
            
            println!("Final best move: {:?}", best_move_so_far);
            best_move_so_far
        }
    }
}

fn find_best_move_at_depth(board: &Board, heuristics: &[Heuristic], depth: u32, deadline: &Instant) -> Option<(usize, usize)> {
    let mut best_move: (usize, usize);
    let mut best_score = f64::NEG_INFINITY; 

    let mut alpha = f64::NEG_INFINITY;
    let beta = f64::INFINITY;
    
    let possible_moves = board.get_all_valid_moves();
    if possible_moves.is_empty() { return Some((0, 0)); }

    best_move = possible_moves[0];
    
    let player_pov = board.current_turn;

    for a_move in possible_moves {
        if Instant::now() >= *deadline {
            return None; 
        }

        let mut temp_board = board.clone();
        
        if temp_board.make_move_for_simulation(a_move.0, a_move.1, Some(deadline)).is_err() {
            continue; 
        }

        match alphabeta(&temp_board, depth - 1, alpha, beta, false, heuristics, player_pov, deadline) {
            Ok(score) => {
                if score > best_score {
                    best_score = score;
                    best_move = a_move;
                }
                alpha = alpha.max(best_score);
            },
            Err(_) => {
                return None;
            }
        }
    }
    Some(best_move)
}

fn alphabeta(board: &Board, depth: u32, mut alpha: f64, mut beta: f64, is_maximizing_player: bool, heuristics: &[Heuristic], player_for_pov: Player, deadline: &Instant) -> Result<f64, ()> {
    if Instant::now() >= *deadline {
        return Err(());
    }

    if depth == 0 || board.game_state != GameState::Ongoing {
        return Ok(evaluate_board(board, heuristics, player_for_pov));
    }

    let possible_moves = board.get_all_valid_moves();
    if possible_moves.is_empty() {
        return Ok(evaluate_board(board, heuristics, player_for_pov));
    }

    if is_maximizing_player {
        let mut max_eval = f64::NEG_INFINITY;
         for a_move in possible_moves {
            let mut child_board = board.clone();
            // FIX: Convert the Result's error type from &str to () to match the function signature.
            child_board.make_move_for_simulation(a_move.0, a_move.1, Some(deadline)).map_err(|_| ())?;

            let eval = alphabeta(&child_board, depth - 1, alpha, beta, false, heuristics, player_for_pov, deadline)?;
            max_eval = max_eval.max(eval);
            alpha = alpha.max(eval);

            if beta <= alpha {
                break;
            }
         }
         Ok(max_eval)
    }
    else {
        let mut min_eval = f64::INFINITY;
        for a_move in possible_moves {
            let mut child_board = board.clone();
            // FIX: Convert the Result's error type from &str to () to match the function signature.
            child_board.make_move_for_simulation(a_move.0, a_move.1, Some(deadline)).map_err(|_| ())?;

            let eval = alphabeta(&child_board, depth - 1, alpha, beta, true, heuristics, player_for_pov, deadline)?;
            min_eval = min_eval.min(eval);
            beta = beta.min(eval);
            if beta <= alpha {
                break;
            }
        }
        Ok(min_eval)
    }
}

fn evaluate_board(board: &Board, heuristics: &[Heuristic], player_for_pov: Player) -> f64 {
    let mut total_score = 0.0;
    let player = player_for_pov;
    let opponent = if player == Player::Red { Player::Blue } else { Player::Red };

    if let GameState::Won { winner } = board.game_state {
        if winner == player { return f64::INFINITY; }
        if winner == opponent { return f64::NEG_INFINITY; }
    }

    const W_ORB_DIFF: f64 = 1.0;
    const W_PERIPHERAL: f64 = 0.2;
    const W_TERRITORY: f64 = 0.1;
    const W_CHAIN_POTENTIAL: f64 = 0.5;
    const W_CONVERSION: f64 = 0.8;
    const W_CASCADE: f64 = 0.7;
    const W_SAFE_MOBILITY: f64 = 0.4;

    for heuristic in heuristics {
        total_score += match heuristic {
            Heuristic::OrbDifference => {
                let my_orbs = board.orb_counts[&player] as f64;
                let opponent_orbs = board.orb_counts[&opponent] as f64;
                (my_orbs - opponent_orbs) * W_ORB_DIFF
            }
            Heuristic::PeripheralControl => {
                let mut peripheral_score = 0.0;
                for r in 0..board.height as usize{
                    for c in 0..board.width as usize{
                        if let CellState::Occupied { player: cell_player, .. } = board.cells[r][c].state {
                            let is_corner = (r == 0 || r == board.height as usize - 1) && (c == 0 || c == board.width as usize - 1);
                            let is_edge = r == 0 || r == board.height as usize - 1 || c == 0 || c == board.width as usize - 1;
                            let value = if is_corner { 3.0 } else if is_edge { 2.0 } else { 1.0 };
                            if cell_player == player {
                                peripheral_score += value;
                            } else {
                                peripheral_score -= value;
                            }
                        }
                    }
                }
                peripheral_score * W_PERIPHERAL
            }
            Heuristic::TerritoryControl => {
                let mut territory_score = 0.0;
                for r in 0..board.height as usize {
                    for c in 0..board.width as usize {
                        if let CellState::Occupied { player: cell_player, .. } = board.cells[r][c].state {
                            if cell_player == player {
                                territory_score += 1.0;
                            } else {
                                territory_score -= 1.0;
                            }
                        }
                    }
                }
                territory_score * W_TERRITORY
            }
            Heuristic::ChainReactionPotential => {
                let mut chain_reaction_score = 0.0;
                for r in 0..board.height as usize {
                    for c in 0..board.width as usize {
                        if let CellState::Occupied { player: cell_player, orbs } = board.cells[r][c].state {
                            if orbs == board.cells[r][c].critical_mass - 1 {
                                if cell_player == player {
                                    chain_reaction_score += 5.0; 
                                } else {
                                    chain_reaction_score -= 5.0; 
                                }
                            }
                        } 
                    }
                }
                chain_reaction_score * W_CHAIN_POTENTIAL
            }
            // --- REVISED HEURISTIC LOGIC ---
            Heuristic::ConversionPotential => {
                let mut conversion_score = 0.0;
                let neighbors_diff: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

                for r in 0..board.height as usize {
                    for c in 0..board.width as usize {
                        if let CellState::Occupied { player: cell_player, orbs } = board.cells[r][c].state {
                            // Calculate how many orbs are needed for this cell to explode.
                            let orbs_to_explode = (board.cells[r][c].critical_mass - orbs) as f64;

                            // Only consider cells that are not yet at critical mass.
                            if orbs_to_explode > 0.0 {
                                let mut opponent_neighbors = 0;
                                for (dr, dc) in &neighbors_diff {
                                    let nr = r as isize + dr;
                                    let nc = c as isize + dc;

                                    if nr >= 0 && nr < board.height as isize && nc >= 0 && nc < board.width as isize {
                                        if let CellState::Occupied { player: neighbor_player, .. } = board.cells[nr as usize][nc as usize].state {
                                            // Count how many adjacent cells belong to the opponent.
                                            if neighbor_player != cell_player {
                                                opponent_neighbors += 1;
                                            }
                                        }
                                    }
                                }
                                
                                if opponent_neighbors > 0 {
                                    // The potential is the number of opponent cells that would be captured,
                                    // weighted by how close the cell is to exploding.
                                    // A smaller 'orbs_to_explode' value leads to a higher potential score.
                                    let potential = opponent_neighbors as f64 / orbs_to_explode;

                                    if cell_player == player {
                                        conversion_score += potential;
                                    } else {
                                        conversion_score -= potential;
                                    }
                                }
                            }
                        }
                    }
                }
                conversion_score * W_CONVERSION
            }
            Heuristic::SafeMobility => {
                let mut my_safe_moves = 0.0;
                let my_possible_moves = board.get_all_valid_moves();
                for my_move in &my_possible_moves {
                    let mut board_after_my_move = board.clone();
                    // FIX: Pass None for the deadline, as this sub-simulation is not time-critical on its own.
                    if board_after_my_move.make_move_for_simulation(my_move.0, my_move.1, None).is_err() {
                        continue;
                    }
                    let mut is_move_safe = true;
                    let mut opponent_board_view = board_after_my_move.clone();
                    opponent_board_view.current_turn = opponent;
                    let opponent_replies = opponent_board_view.get_all_valid_moves();
                    for opp_reply in &opponent_replies {
                        let mut board_after_opp_reply = opponent_board_view.clone();
                        // FIX: Pass None for the deadline here as well.
                        if board_after_opp_reply.make_move_for_simulation(opp_reply.0, opp_reply.1, None).is_err() {
                            continue;
                        }
                        if board_after_opp_reply.orb_counts[&player] < board.orb_counts[&player] {
                             is_move_safe = false;
                             break;
                        }
                    }
                    if is_move_safe {
                        my_safe_moves += 1.0;
                    }
                }
                my_safe_moves * W_SAFE_MOBILITY
            }
            Heuristic::CascadePotential => {
                let mut cascade_score = 0.0;
                let neighbors_diff: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
                for r in 0..board.height as usize {
                    for c in 0..board.width as usize {
                        if let CellState::Occupied { player: trigger_player, orbs } = board.cells[r][c].state {
                            if orbs == board.cells[r][c].critical_mass - 1 {
                                let mut current_cascade_value = 0.0;
                                for (dr, dc) in &neighbors_diff {
                                    let nr = r as isize + dr;
                                    let nc = c as isize + dc;
                                    if nr >= 0 && nr < board.height as isize && nc >= 0 && nc < board.width as isize {
                                        if let CellState::Occupied { orbs: neighbor_orbs, .. } = board.cells[nr as usize][nc as usize].state {
                                            current_cascade_value += neighbor_orbs as f64;
                                            if neighbor_orbs == board.cells[nr as usize][nc as usize].critical_mass - 1 {
                                                current_cascade_value += 5.0;
                                            }
                                        }
                                    }
                                }
                                if trigger_player == player {
                                    cascade_score += current_cascade_value;
                                } else {
                                    cascade_score -= current_cascade_value;
                                }
                            }
                        }
                    }
                }
                cascade_score * W_CASCADE
            }
        }
    }
    
    total_score
}
