use bevy::prelude::*;

use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::game_logic::movement_logic::MoveBuilder;

use crate::game_logic::minmax_logic;

mod game_logic;
mod utils;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MeshPickingPlugin))
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .insert_resource(game_logic::GameState::new())
        .insert_resource(game_logic::minmax_logic::GeneratedMoves::new())
        .add_systems(Startup, (environment_setup, board_setup).chain())
        .add_systems(Update, figure_picking)
        .add_systems(
            Update,
            (
                minmax_logic::spawn_minmax_task,
                minmax_logic::retrieve_and_exec_minmax_result,
            ),
        )
        .run();
}

#[derive(Component)]
struct SurfaceTile;

fn environment_setup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.5, 0.5).looking_at(Vec3::ZERO, Vec3::Y),
        MeshPickingCamera,
    ));

    commands
        .spawn(DirectionalLight {
            shadows_enabled: true,
            illuminance: 5000.,
            ..Default::default()
        })
        .insert(
            Transform::from_xyz(0.0, 0.0, 0.0).looking_to(Vec3::new(-0.5, -1.0, -0.5), Vec3::Y),
        );
}

fn board_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let scene_handle = asset_server.load(GltfAssetLabel::Scene(0).from_asset("chess_set.glb"));

    commands.spawn((Transform::from_xyz(0.0, 0.0, 0.0), SceneRoot(scene_handle)));

    let square_size = 0.05;

    for (row, _row_c) in (0..8).zip('1'..='8') {
        for (col, _col_c) in (0..8).zip('a'..='h') {
            let (row_offset, col_offset) = utils::idx_to_coordinates(row, col);

            commands.spawn((
                Mesh3d(meshes.add(Plane3d::default().mesh().size(square_size, square_size))),
                Transform::from_xyz(col_offset, 0.01, row_offset),
                Name::new(format!("Tile_{}_{}", row, col)),
                MeshMaterial3d(materials.add(Color::NONE)),
                SurfaceTile,
            ));
        }
    }
}

// ToDo: Disable figure picking for one colour entirely in Multiplayer
fn figure_picking(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut click_events: EventReader<Pointer<Click>>,
    tile_query: Query<(Entity, &Name, &MeshMaterial3d<StandardMaterial>), With<SurfaceTile>>,
    mut piece_query: Query<(Entity, &Name, &mut Transform)>,
    mut game_state: ResMut<game_logic::GameState>,
) {
    for click in click_events.read().take(1) {
        if let Ok((_tile_ent, tile_name, tile_mat)) = tile_query.get(click.target) {
            reset_tile_highlights(&mut materials, &tile_query);
            let clicked_tile = utils::tile_to_indices(tile_name.as_str());

            // Case 1: We previously picked a valid figure and are about to move the figure now
            if let Some((picked_pigure, from_row,from_col)) = game_state.chosen_figure {
                if let Some(chess_move) = game_state.pick_is_valid((from_row,from_col),clicked_tile) {
                    game_state.execute_move(
                        &mut commands,
                        picked_pigure.ass_name,
                        &chess_move,
                        &mut piece_query,
                    );
                }
        game_state.chosen_figure = None;
        game_state.possible_moves = None;
        game_state.move_number += 1;
            }
            // Case 2: We did not yet pick a valid figure and will pick the figure to be moved now
            else {
                if let Some(fig) = game_state.board[clicked_tile]
                    && fig.player_color == game_state.player_turn
                {
                    game_state.chosen_figure = Some((fig, clicked_tile.0,clicked_tile.1));
                } else {
                    return;
                }

                let movelist = MoveBuilder::new(clicked_tile, &game_state.board)
                    .calculate_naive_moves(&game_state.board)
                    ._filter_brute_force_2(&game_state.board)
                    .moveset;

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

fn highlight_tiles(
    materials: &mut ResMut<'_, Assets<StandardMaterial>>,
    tile_query: Query<
        '_,
        '_,
        (Entity, &Name, &MeshMaterial3d<StandardMaterial>),
        With<SurfaceTile>,
    >,
    move_list_str: Vec<String>,
    tile_mat: &MeshMaterial3d<StandardMaterial>,
) {
    //1. Highlight fields
    if let Some(material) = materials.get_mut(&tile_mat.0) {
        material.base_color = Color::srgba(1.0, 0.0, 0.0, 0.6);
    }
    for (_e, n, m) in tile_query {
        if move_list_str.contains(&n.as_str().to_string()) {
            if let Some(material) = materials.get_mut(&m.0) {
                material.base_color = Color::srgba(0.0, 1.0, 0.0, 0.6); // modifies existing
            }
        }
    }
}

fn reset_tile_highlights(
    materials: &mut ResMut<Assets<StandardMaterial>>,
    tile_query: &Query<(Entity, &Name, &MeshMaterial3d<StandardMaterial>), With<SurfaceTile>>,
) {
    for (_e, _n, mat) in tile_query {
        if let Some(material) = materials.get_mut(&mat.0) {
            material.base_color = Color::NONE;
        }
    }
}
