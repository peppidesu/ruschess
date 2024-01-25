use crate::state::GameState;
use crate::moves::Move;
pub trait Player {
    fn get_move(&self, state: &GameState) -> Move;    
}