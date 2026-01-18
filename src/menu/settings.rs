use bevy::prelude::*;

#[derive(Resource, Clone, Copy, Default, Eq, PartialEq, Debug, Hash)]
pub enum GameMode {
    #[default]
    PVE,
    PVP,
}
