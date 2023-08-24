use bevy::reflect::TypePath;
use leafwing_input_manager::prelude::*;

#[derive(Actionlike, TypePath, PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum EditorAction {
    Move,
    Rotate,
    Zoom,
    Select,
    SpawnHandle,
    SpawnLine,
}
