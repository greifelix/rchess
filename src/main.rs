mod game_logic;
mod menu;
mod utils;

use bevy::gltf::{Gltf, GltfExtras, GltfMesh};
use bevy::{camera::Viewport, prelude::*};

use crate::game_logic::movement_logic::{self, MoveBuilder};
use crate::game_logic::{FigType, PlayerColor, minmax_logic};
use crate::menu::escape_menu::escape_menu_plugin;
use crate::menu::settings::{GameMode, GameSettings, settings_menu_plugin};
use crate::menu::{GuiState, menu_plugin};
use crate::minmax_logic::player_vs_minmax_plugin;
fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MeshPickingPlugin))
        .insert_resource(game_logic::GameState::new())
        .insert_resource(game_logic::minmax_logic::GeneratedMoves::new())
        .insert_resource(menu::settings::GameSettings::default())
        .init_state::<GuiState>()
        .add_systems(Startup, (environment_setup, board_setup).chain())
        .add_systems(Update, figure_picking)
        .add_systems(Update, set_camera_viewports)
        .add_plugins(menu_plugin)
        .add_plugins(escape_menu_plugin)
        .add_plugins(settings_menu_plugin)
        .add_plugins(player_vs_minmax_plugin)
        .run();
}

#[derive(Resource)]
pub struct ChessScene(Handle<Gltf>);

/// Use this marker for respawning everything
#[derive(Component)]
struct WoodenPiece;

#[derive(Component)]
struct SurfaceTile;

#[derive(Component)]
struct CameraPosition {
    pos: UVec2,
}

#[derive(Component)]
struct WhiteCamera;

#[derive(Component)]
struct BlackCamera;

fn environment_setup(mut commands: Commands) {
    // I need camera for UI for splitscreen mode
    commands.spawn((
        Camera2d,
        Camera {
            order: 2,
            ..default()
        },
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.5, 0.5).looking_at(Vec3::ZERO, Vec3::Y),
        Camera {
            order: 0,
            ..default()
        },
        CameraPosition {
            pos: UVec2::new(0, 0),
        },
        MeshPickingCamera,
        WhiteCamera,
    ));

    commands
        .spawn(DirectionalLight {
            shadows_enabled: true,
            illuminance: 10000.,
            ..Default::default()
        })
        .insert(
            Transform::from_xyz(0.0, 0.8, 0.0).looking_to(Vec3::new(-0.5, -1.0, -0.5), Vec3::Y),
        );
}

fn queen_spawner(
    commands: &mut Commands,
    chess_scene: &Res<ChessScene>,
    gltf_assets: &Res<Assets<Gltf>>,
    gltf_meshes: &Res<Assets<GltfMesh>>,
    color: PlayerColor,
    queen_name: &str,
    (row, col): (u8, u8),
) {
    // Wait until the scene is loaded
    let Some(gltf) = gltf_assets.get(&chess_scene.0) else {
        return;
    };
    let (mesh_handle, mat_handle) = if color == PlayerColor::White {
        (
            gltf.meshes[30].clone(),
            gltf.named_materials["white pieces"].clone(),
        )
    } else {
        (
            gltf.meshes[31].clone(),
            gltf.named_materials["black pieces"].clone(),
        )
    };

    let (row_offset, col_offset) = utils::idx_to_coordinates(row, col);
    let gltf_mesh = gltf_meshes.get(mesh_handle.id()).unwrap();
    commands.spawn((
        Mesh3d(gltf_mesh.primitives[0].mesh.clone()),
        MeshMaterial3d(mat_handle.clone()),
        Transform::from_xyz(col_offset, 0.047, row_offset),
        gltf_mesh.primitives[0]
            .extras
            .clone()
            .unwrap_or(GltfExtras::default()),
        Name::from(queen_name),
        WoodenPiece,
    ));
}

fn board_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let gltf_handle: Handle<Gltf> = asset_server.load("chess_set.glb");
    commands.insert_resource(ChessScene(gltf_handle));

    let scene_handle = asset_server.load(GltfAssetLabel::Scene(0).from_asset("chess_set.glb"));
    commands.spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        SceneRoot(scene_handle),
        Name::new("Original Chess Scene"),
        WoodenPiece,
    ));

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

fn figure_picking(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut click_events: MessageReader<Pointer<Click>>,
    tile_query: Query<(Entity, &Name, &MeshMaterial3d<StandardMaterial>), With<SurfaceTile>>,
    mut piece_query: Query<(Entity, &Name, &mut Transform)>,
    mut game_state: ResMut<game_logic::GameState>,
    chess_scene: Res<crate::ChessScene>,
    gltf_assets: Res<Assets<Gltf>>,
    gltf_meshes: Res<Assets<GltfMesh>>,
) {
    for click in click_events.read().take(1) {
        if let Ok((_tile_ent, tile_name, tile_mat)) = tile_query.get(click.entity) {
            reset_tile_highlights(&mut materials, &tile_query);
            let clicked_tile = utils::tile_to_indices(tile_name.as_str());

            // Case 1: We previously picked a valid figure and are about to move the figure now
            if let Some((picked_pigure, from_row, from_col)) = game_state.chosen_figure {
                if let Some(chess_move) =
                    game_state.pick_is_valid((from_row, from_col), clicked_tile)
                {
                    game_state.execute_move(
                        &mut commands,
                        picked_pigure.ass_name,
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
                if let Some(fig) = game_state.board[clicked_tile]
                    && fig.player_color == game_state.player_turn
                {
                    game_state.chosen_figure = Some((fig, clicked_tile.0, clicked_tile.1));
                } else {
                    return;
                }

                let mut movelist: Vec<movement_logic::ChessMove> =
                    MoveBuilder::new(clicked_tile, &game_state.board)
                        .calculate_naive_moves(&game_state.board)
                        ._filter_brute_force(&game_state.board)
                        .moveset
                        .into_iter()
                        .collect();

                // Only add the rochade possibility in case we picked the king
                if game_state.chosen_figure.unwrap().0.fig_type == FigType::King {
                    movement_logic::maybe_add_rochade(
                        &game_state.player_turn,
                        &mut movelist,
                        &game_state.board,
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

fn highlight_tiles(
    materials: &mut ResMut<Assets<StandardMaterial>>,
    tile_query: Query<(Entity, &Name, &MeshMaterial3d<StandardMaterial>), With<SurfaceTile>>,
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

fn set_camera_viewports(
    windows: Query<&Window>,
    mut query: Query<(&CameraPosition, &mut Camera)>,
    settings: Res<GameSettings>,
) {
    for window in windows {
        let size = if settings.game_mode == GameMode::PVP {
            UVec2::from((window.physical_size().x / 2, window.physical_size().y))
        } else {
            window.physical_size()
        };

        for (camera_position, mut camera) in &mut query {
            camera.viewport = Some(Viewport {
                physical_position: camera_position.pos * size,
                physical_size: size,
                ..default()
            });
        }
    }
}
