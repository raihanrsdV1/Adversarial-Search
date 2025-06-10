// This file contains the core data structures and enums that define the game itself.
// Notice that every struct and enum is marked with `pub` so other files can use them.

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
    pub state: CellState,
    pub critical_mass: u32,
    pub is_queued: bool,
}

impl Cell {
    pub fn new(critical_mass: u32) -> Self {
        Cell {
            state: CellState::Empty,
            critical_mass,
            is_queued: false,
        }
    }

    pub fn add_orb(&mut self, player: Player) -> bool {
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
    
    pub fn get_explosion_data(&self) -> Option<(Player, u32)> {
        if let CellState::Occupied { player, orbs } = self.state {
            if orbs >= self.critical_mass {
                return Some((player, orbs));
            }
        }
        None
    }

    pub fn take_over(&mut self, player: Player) {
        let orbs = match self.state {
            CellState::Occupied { orbs, .. } => orbs,
            CellState::Empty => 0,
        };
        self.state = CellState::Occupied { player, orbs: orbs + 1 };
    }
}
