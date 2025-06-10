// This file is now the main entry point and "director" of the application.
// It declares the other modules and contains the game loop.

use std::io;


mod game;
mod board;
mod ai;

// --- Bring necessary items into scope ---
use game::{Player, GameState};
use board::Board;
use ai::{AIStrategy, Heuristic, get_ai_move};

/// The main game loop for a Human vs. AI match.
fn main() {
    let log_filename = "game_log.txt".to_string();
    let mut game_board = Board::new(6, 9, Player::Red, log_filename);
    let human_player = Player::Red;
    let ai_player = Player::Blue;

    // --- AI Configuration ---
    let ai_strategy = AIStrategy::AlphaBeta; 
    let ai_heuristics = vec![
        //Heuristic::OrbDifference, 
        Heuristic::PeripheralControl,
        //Heuristic::ChainReactionPotential,
        // Heuristic::ConversionPotential,
        //Heuristic::SafeMobility,
        //Heuristic::CascadePotential,
    ];
    let search_depth = 2; // A depth of 4-5 is a good starting point.

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

                game_board.log_move(current_player, row, col);
                if let Err(e) = game_board.make_move(row, col) {
                    println!("Invalid move: {}", e);
                }
            } else {
                println!("Invalid input. Please use the format 'row col', e.g., '3 4'");
            }
        } else {
            println!("AI ({:?}) is thinking...", ai_player);
            // UPDATED CALL: We now call the free function from the `ai` module.
            let (row, col) = get_ai_move(&game_board, ai_strategy, &ai_heuristics, search_depth);
            println!("AI moves to ({}, {})", row, col);
            game_board.log_move(current_player, row, col);
            game_board.make_move(row, col).expect("AI made an invalid move!");
        }
        println!("\n---------------------------\n");
    }
}
