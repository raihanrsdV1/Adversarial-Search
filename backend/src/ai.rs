// This file contains all the AI-related logic as free-standing functions.
// They operate on a `Board` but are not part of the Board's implementation.

use crate::board::Board;
use crate::game::{Player, GameState, CellState};
use rand::Rng;

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


/// The main entry point for getting the AI's move.
pub fn get_ai_move(board: &Board, strategy: AIStrategy, heuristics: &[Heuristic], depth: u32) -> (usize, usize) {
    match strategy {
        AIStrategy::Random => {
            let mut rng = rand::thread_rng();
            loop {
                let row = rng.gen_range(0..board.height as usize);
                let col = rng.gen_range(0..board.width as usize);
                let mut temp_board = board.clone();
                if temp_board.make_move(row, col).is_ok() {
                    return (row, col);
                }
            }
        }
        AIStrategy::AlphaBeta => {
            find_best_move_alphabeta(board, heuristics, depth)
        }
    }
}

/// Finds the best move using the alpha-beta algorithm. This is the top-level "manager" function.
fn find_best_move_alphabeta(board: &Board, heuristics: &[Heuristic], depth: u32) -> (usize, usize) {
    let mut best_move: (usize, usize) = (0, 0);
    let mut best_score = f64::NEG_INFINITY; 

    let alpha = f64::NEG_INFINITY;
    let beta = f64::INFINITY;
    
    let possible_moves = board.get_all_valid_moves();
    if possible_moves.is_empty() {
        return (0, 0);
    }

    best_move = possible_moves[0];
    
    // The player whose turn it is at the root of the search. This is our consistent Point of View.
    let player_pov = board.current_turn;

    for a_move in possible_moves {
        let mut temp_board = board.clone();
        temp_board.make_move(a_move.0, a_move.1).unwrap();

        // We are the maximizing player, so the next turn is the minimizing player (is_maximizing_player = false)
        let score = alphabeta(&temp_board, depth - 1, alpha, beta, false, heuristics, player_pov);

        // We want the move that results in the HIGHEST score from our Point of View.
        if score > best_score {
            best_score = score;
            best_move = a_move;
        }
    }
    return best_move;
}

/// The core recursive helper function for the alpha-beta algorithm.
fn alphabeta(board: &Board, depth: u32, mut alpha: f64, mut beta: f64, is_maximizing_player: bool, heuristics: &[Heuristic], player_for_pov: Player) -> f64 {
    if depth == 0 || board.game_state != GameState::Ongoing {
        return evaluate_board(&board, heuristics, player_for_pov);
    }

    let possible_moves = board.get_all_valid_moves();
    if possible_moves.is_empty() {
        return evaluate_board(&board, heuristics, player_for_pov);
    }

    if is_maximizing_player {
        let mut max_eval = f64::NEG_INFINITY;
         for a_move in possible_moves {
            let mut child_board = board.clone();
            child_board.make_move(a_move.0, a_move.1).unwrap();

            let eval = alphabeta(&child_board, depth - 1, alpha, beta, false, heuristics, player_for_pov);
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
            let eval = alphabeta(&child_board, depth - 1, alpha, beta, true, heuristics, player_for_pov);
            min_eval = min_eval.min(eval);
            beta = beta.min(eval);
            if beta <= alpha {
                break;
            }
        }
        return min_eval;
    }
}

/// Evaluates the board state from the perspective of a consistent player (the one who started the search).
fn evaluate_board(board: &Board, heuristics: &[Heuristic], player_for_pov: Player) -> f64 {
    let mut total_score = 0.0;
    // The player for point-of-view (pov) is passed in, ensuring a consistent evaluation.
    let player = player_for_pov;
    let opponent = if player == Player::Red { Player::Blue } else { Player::Red };

    // Handle game-ending states with the highest/lowest possible scores
    if let GameState::Won { winner } = board.game_state {
        if winner == player { return f64::INFINITY; }
        if winner == opponent { return f64::NEG_INFINITY; }
    }

    for heuristic in heuristics {
        total_score += match heuristic {
            Heuristic::OrbDifference => {
                let my_orbs = board.orb_counts[&player] as f64;
                let opponent_orbs = board.orb_counts[&opponent] as f64;
                my_orbs - opponent_orbs
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
                peripheral_score
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
                territory_score
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
                chain_reaction_score
            }
            Heuristic::ConversionPotential => {
                let mut conversion_score = 0.0;
                let neighbors_diff: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
                for r in 0..board.height as usize {
                    for c in 0..board.width as usize {
                        if let CellState::Occupied { player: trigger_player, orbs } = board.cells[r][c].state {
                            if orbs == board.cells[r][c].critical_mass - 1 {
                                for (dr, dc) in &neighbors_diff {
                                    let nr = r as isize + dr;
                                    let nc = c as isize + dc;
                                    if nr >= 0 && nr < board.height as isize && nc >= 0 && nc < board.width as isize {
                                        if let CellState::Occupied { player: neighbor_player, orbs: neighbor_orbs } = board.cells[nr as usize][nc as usize].state {
                                            if trigger_player == player && neighbor_player == opponent {
                                                conversion_score += neighbor_orbs as f64;
                                            }
                                            else if trigger_player == opponent && neighbor_player == player {
                                                conversion_score -= neighbor_orbs as f64;
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
            Heuristic::SafeMobility => {
                let mut my_safe_moves = 0.0;
                let my_possible_moves = board.get_all_valid_moves();
                for my_move in &my_possible_moves {
                    let mut board_after_my_move = board.clone();
                    board_after_my_move.make_move(my_move.0, my_move.1).unwrap();
                    let mut is_move_safe = true;
                    
                    let mut opponent_board_view = board_after_my_move.clone();
                    opponent_board_view.current_turn = opponent;
                    let opponent_replies = opponent_board_view.get_all_valid_moves();

                    for opp_reply in &opponent_replies {
                        let target_cell = board_after_my_move.cells[opp_reply.0][opp_reply.1];
                        let would_explode = match target_cell.state {
                            CellState::Occupied { orbs, .. } => orbs + 1 == target_cell.critical_mass,
                            CellState::Empty => 1 == target_cell.critical_mass,
                        };
                        if would_explode {
                            is_move_safe = false;
                            break;
                        }
                    }
                    if is_move_safe {
                        my_safe_moves += 1.0;
                    }
                }
                my_safe_moves
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
                                                current_cascade_value += 10.0;
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
                cascade_score
            }
        }
    }
    
    total_score
}
