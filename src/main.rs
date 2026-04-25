mod game_logic;
mod menu;
mod utils;

use crate::game_logic::minmax_logic;
use crate::menu::{
    GuiState, escape_menu::escape_menu_plugin, menu_plugin, settings::settings_menu_plugin,
};
use crate::minmax_logic::player_vs_minmax_plugin;
use crate::utils::{
    board_graphics_utils::set_camera_viewports,
    picking_utils::figure_picking,
    setup_utils::{board_setup, environment_setup},
};
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MeshPickingPlugin))
        .insert_resource(game_logic::state_logic::GameState::new())
        .insert_resource(game_logic::minmax_logic::GeneratedMoves::new())
        .insert_resource(menu::settings::GameSettings::default())
        .init_state::<GuiState>()
        .add_systems(Startup, (environment_setup, board_setup).chain())
        .add_systems(Update, set_camera_viewports)
        .add_systems(Update, figure_picking)
        .add_plugins(menu_plugin)
        .add_plugins(escape_menu_plugin)
        .add_plugins(settings_menu_plugin)
        .add_plugins(player_vs_minmax_plugin)
        .run();
}
