// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;
use tauri::{State, AppHandle}; 
use serde::{Deserialize, Serialize};

pub mod game;
pub mod board;
pub mod ai;

use board::Board; 
use game::{Player, CellState};
use ai::{get_ai_move, AIStrategy, Heuristic};

// --- Data Transfer Objects (DTOs) ---
// These DTOs are the contract between Rust and the Svelte frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellData {
    pub player: Option<String>,
    pub orbs: u32,
    pub critical_mass: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameStateData {
    pub board: Vec<Vec<CellData>>,
    pub current_player: String,
    pub game_status: String,
    pub winner: Option<String>,
    pub red_orbs: u32,
    pub blue_orbs: u32,
    pub total_moves: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfigData {
    pub strategy: String,
    pub depth: u32,
    pub heuristics: Vec<String>,
    pub time_limit_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerConfigData {
    pub player_type: String,
    pub name: String,
    pub ai_config: Option<AIConfigData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfigData {
    pub width: u32,
    pub height: u32,
    pub red_player: PlayerConfigData,
    pub blue_player: PlayerConfigData,
}

pub struct GameManager {
    pub board: Option<Board>,
    pub config: Option<GameConfigData>,
}

impl GameManager {
    pub fn new() -> Self {
        GameManager {
            board: None,
            config: None,
        }
    }
}

// Helper function to convert a single Board state to a DTO
fn convert_board_to_state_data(board: &Board) -> GameStateData {
    let board_data = board.cells.iter().map(|row| {
        row.iter().map(|cell| {
            let (player, orbs) = match cell.state {
                game::CellState::Empty => (None, 0),
                game::CellState::Occupied { player, orbs } => (Some(format!("{:?}", player)), orbs),
            };
            CellData { player, orbs, critical_mass: cell.critical_mass }
        }).collect()
    }).collect();
    
    let (game_status, winner) = match board.game_state {
        game::GameState::Ongoing => ("ongoing".to_string(), None),
        game::GameState::Won { winner } => ( "finished".to_string(), Some(format!("{:?}", winner)) )
    };
    
    GameStateData {
        board: board_data,
        current_player: format!("{:?}", board.current_turn),
        game_status,
        winner,
        red_orbs: board.orb_counts.get(&Player::Red).cloned().unwrap_or(0),
        blue_orbs: board.orb_counts.get(&Player::Blue).cloned().unwrap_or(0),
        total_moves: board.total_moves,
    }
}

// --- Tauri Commands ---

#[tauri::command]
fn start_game(config: GameConfigData, state: State<Mutex<GameManager>>) -> Result<GameStateData, String> {
    let mut manager = state.lock().unwrap();
    let log_filename = "../game_log.txt".to_string(); 
    let board = Board::new(config.width, config.height, Player::Red, log_filename);
    let game_state_dto = convert_board_to_state_data(&board);
    manager.board = Some(board);
    manager.config = Some(config);
    Ok(game_state_dto)
}

#[tauri::command]
// FIX: This command now returns the entire animation history to the frontend.
fn make_move(row: usize, col: usize, state: State<Mutex<GameManager>>, _app: AppHandle) -> Result<Vec<GameStateData>, String> {
    let mut manager = state.lock().unwrap();
    let board = manager.board.as_mut().ok_or("Game not initialized")?;
    
    let history_of_boards = board.make_move_and_get_history(row, col).map_err(|e| e.to_string())?;

    // Convert the Vec<Board> into a Vec<GameStateData> for the frontend.
    let history_for_frontend = history_of_boards
        .into_iter()
        .map(|b| convert_board_to_state_data(&b))
        .collect();
    
    Ok(history_for_frontend)
}


#[tauri::command]
fn get_ai_move_command(state: State<Mutex<GameManager>>) -> Result<(usize, usize), String> {
    let manager = state.lock().unwrap();
    let board = manager.board.as_ref().ok_or("Game not initialized")?;
    let config = manager.config.as_ref().ok_or("Game config missing")?;

    let ai_player_color = board.current_turn;
    let ai_player_config = if ai_player_color == Player::Red { &config.red_player } else { &config.blue_player };
    
    if ai_player_config.player_type == "AI" {
        if let Some(ai_conf) = &ai_player_config.ai_config {
            let strategy = match ai_conf.strategy.as_str() {
                "Random" => AIStrategy::Random, "AlphaBeta" => AIStrategy::AlphaBeta,
                _ => AIStrategy::Random,
            };
            let heuristics: Vec<Heuristic> = ai_conf.heuristics.iter().map(|h| match h.as_str() {
                "OrbDifference" => Heuristic::OrbDifference, "PeripheralControl" => Heuristic::PeripheralControl,
                "TerritoryControl" => Heuristic::TerritoryControl, "ChainReactionPotential" => Heuristic::ChainReactionPotential,
                "ConversionPotential" => Heuristic::ConversionPotential, "CascadePotential" => Heuristic::CascadePotential,
                "SafeMobility" => Heuristic::SafeMobility, _ => Heuristic::OrbDifference,
            }).collect();
            
            return Ok(get_ai_move(board, strategy, &heuristics, ai_conf.depth, ai_conf.time_limit_ms));
        }
    }
    Err("Current player is not an AI".to_string())
}

#[tauri::command]
fn get_current_state(state: State<Mutex<GameManager>>) -> Result<GameStateData, String> {
    let manager = state.lock().unwrap();
    let board = manager.board.as_ref().ok_or("Game not initialized")?;
    Ok(convert_board_to_state_data(board))
}

#[tauri::command]
fn recover_from_log(state: State<Mutex<GameManager>>) -> Result<GameStateData, String> {
    use std::fs;
    use std::path::Path;
    
    let mut manager = state.lock().unwrap();
    let config = manager.config.as_ref().ok_or("Game config missing")?;
    
    // Try to read the log file
    let log_path = Path::new("../game_log.txt");
    if !log_path.exists() {
        let alt_path = Path::new("game_log.txt");
        if !alt_path.exists() {
            return Err("Log file not found".to_string());
        }
    }
    
    let log_content = fs::read_to_string(log_path.exists().then(|| log_path).unwrap_or(Path::new("game_log.txt")))
        .map_err(|e| format!("Failed to read log file: {}", e))?;
    
    // Parse the last board state from the log
    let lines: Vec<&str> = log_content.lines().collect();
    if lines.len() < 2 {
        return Err("Log file is empty or corrupted".to_string());
    }
    
    // Find the last "AI Move:" section
    let mut board_lines = Vec::new();
    let mut found_ai_move = false;
    
    for line in lines.iter().rev() {
        if line.starts_with("AI Move:") {
            found_ai_move = true;
            break;
        }
        if found_ai_move {
            board_lines.insert(0, *line);
        }
    }
    
    if !found_ai_move {
        // Get the last section of lines that look like board state
        let mut start_idx = lines.len().saturating_sub(config.height as usize);
        for i in (0..lines.len()).rev() {
            if lines[i].contains("AI Move:") {
                start_idx = i + 1;
                break;
            }
        }
        board_lines = lines[start_idx..].to_vec();
    }
    
    if board_lines.is_empty() || board_lines.len() != config.height as usize {
        return Err("Could not parse board state from log".to_string());
    }
    
    // Create a new board and parse the state
    let mut board = Board::new(config.width, config.height, Player::Red, "../game_log.txt".to_string());
    
    for (row, line) in board_lines.iter().enumerate() {
        let cells: Vec<&str> = line.split_whitespace().collect();
        if cells.len() != config.width as usize {
            return Err(format!("Invalid board row in log: {}", line))?;
        }
        
        for (col, cell_str) in cells.iter().enumerate() {
            if *cell_str == "0" {
                // Empty cell
                continue;
            }
            
            let orbs = cell_str.chars()
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse::<u32>()
                .map_err(|_| format!("Invalid orb count: {}", cell_str))?;
            
            let player = if cell_str.contains('R') {
                Player::Red
            } else if cell_str.contains('B') {
                Player::Blue
            } else {
                return Err(format!("Invalid player in cell: {}", cell_str))?;
            };
            
            // Update cell state using the correct structure
            board.cells[row][col].state = CellState::Occupied { player, orbs };
        }
    }
    
    // Update the current player (this is a guess - you might want to track this in the log too)
    board.current_turn = Player::Red; // Default, could be improved
    
    // Update the manager state
    manager.board = Some(board.clone());
    
    Ok(convert_board_to_state_data(&board))
}

pub fn run() {
    tauri::Builder::default()
        .manage(Mutex::new(GameManager::new()))
        .invoke_handler(tauri::generate_handler![
            start_game,
            make_move,
            get_ai_move_command,
            get_current_state,
            recover_from_log
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
