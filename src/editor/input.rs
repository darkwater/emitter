use leafwing_input_manager::prelude::*;

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum EditorAction {
    Move,
    Rotate,
    Zoom,
    Select,
}
