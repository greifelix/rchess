use bevy::prelude::*;

use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::game_logic::movement_logic::MoveBuilder;

mod game_logic;
mod utils;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MeshPickingPlugin))
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .insert_resource(game_logic::GameState::new())
        .add_systems(Startup, (environment_setup, board_setup).chain())
        .add_systems(Update, figure_picking)
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

    for (row, _row_c) in ('1'..='8').enumerate() {
        for (col, _col_c) in ('a'..='h').enumerate() {
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
            let (clicked_row, clicked_col) = utils::tile_to_indices(tile_name.as_str());

            // Case 1: We previously picked a valid figure and are about to move the figure now
            if let Some((picked_pigure, from_row, from_col)) = game_state.chosen_figure {
                game_state.execute_move(
                    &mut commands,
                    picked_pigure.ass_name,
                    (from_row, from_col),
                    (clicked_row, clicked_col),
                    &mut piece_query,
                );
            }
            // Case 2: We did not yet pick a valid figure and will pick the figure to be moved now
            else {
                if let Some(fig) = game_state.get_fig_on_tile(clicked_row, clicked_col)
                    && fig.player_color == game_state.player_turn
                {
                    game_state.chosen_figure = Some((fig, clicked_row, clicked_col));
                } else {
                    return;
                }

                let movelist =
                    MoveBuilder::new((clicked_row, clicked_col), game_state.board.clone())
                        .calculate_naive_moves()
                        .filter_moves_in_check(game_state.under_attack)
                        .block_selfchecking_moves()
                        .extract();

                let move_list_str: Vec<String> = movelist
                    .to
                    .iter()
                    .map(|(r, c)| format!("Tile_{r}_{c}"))
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
