use leafwing_input_manager::prelude::*;

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum PlayerAction {
    Move,
    Aim,
    Shoot,
}
