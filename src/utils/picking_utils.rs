use super::{
    ChessScene, FigType, SurfaceTile,
    board_utils::{highlight_tiles, reset_tile_highlights},
    tile_to_indices,
};
use crate::game_logic::movement_logic::{ChessMove, MoveBuilder, maybe_add_rochade};
use crate::game_logic::state_logic::GameState;
use bevy::gltf::{Gltf, GltfMesh};
use bevy::prelude::*;

pub fn figure_picking(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut click_events: MessageReader<Pointer<Click>>,
    tile_query: Query<(Entity, &Name, &MeshMaterial3d<StandardMaterial>), With<SurfaceTile>>,
    mut piece_query: Query<(Entity, &Name, &mut Transform)>,
    mut game_state: ResMut<GameState>,
    chess_scene: Res<ChessScene>,
    gltf_assets: Res<Assets<Gltf>>,
    gltf_meshes: Res<Assets<GltfMesh>>,
) {
    for click in click_events.read().take(1) {
        if let Ok((_tile_ent, tile_name, tile_mat)) = tile_query.get(click.entity) {
            reset_tile_highlights(&mut materials, &tile_query);
            let clicked_tile = tile_to_indices(tile_name.as_str());

            // Case 1: We previously picked a valid figure and are about to move the figure now
            if let Some((_picked_pigure, from_row, from_col)) = game_state.chosen_figure {
                if let Some(chess_move) =
                    game_state.pick_is_valid((from_row, from_col), clicked_tile)
                {
                    let picked_asset_name = game_state.fat_board[(from_row, from_col)].unwrap();
                    game_state.execute_move(
                        &mut commands,
                        picked_asset_name,
                        &chess_move,
                        &mut piece_query,
                        &chess_scene,
                        &gltf_assets,
                        &gltf_meshes,
                    );
                }
                game_state.chosen_figure = None;
                game_state.possible_moves = None;
                game_state.move_number += 1;
            }
            // Case 2: We did not yet pick a valid figure and will pick the figure to be moved now
            else {
                if let Some(fig) = game_state.fat_board.board[clicked_tile]
                    && fig.player_color == game_state.player_turn
                {
                    game_state.chosen_figure = Some((fig, clicked_tile.0, clicked_tile.1));
                } else {
                    return;
                }

                let mut movelist: Vec<ChessMove> =
                    MoveBuilder::new(clicked_tile, &game_state.fat_board.board)
                        .calculate_naive_moves(&game_state.fat_board.board)
                        .filter_brute_force(&game_state.fat_board.board)
                        .moveset
                        .into_iter()
                        .collect();

                // Only add the rochade possibility in case we picked the king
                if game_state.chosen_figure.unwrap().0.fig_type == FigType::King {
                    maybe_add_rochade(
                        &game_state.player_turn,
                        &mut movelist,
                        &game_state.fat_board.board,
                    );
                }

                let move_list_str: Vec<String> = movelist
                    .iter()
                    .map(|cm| {
                        let (r, c) = cm.to_tile;
                        format!("Tile_{r}_{c}")
                    })
                    .collect();
                game_state.possible_moves = Some(movelist);

                highlight_tiles(&mut materials, tile_query, move_list_str, tile_mat);
            }
        }
    }
}
