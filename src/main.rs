use std::process::CommandArgs;

use bevy::color::palettes::tailwind::*;
use bevy::ecs::system::command::send_event;
use bevy::gltf::{Gltf, GltfMaterialName, GltfMesh};
use bevy::picking::pointer::PointerInteraction;
use bevy::prelude::*;
use bevy::state::commands;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

mod utils;
use rchess::game_logic::{self, FigType, Figure};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MeshPickingPlugin))
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .insert_resource(game_logic::GameState::new())
        .add_systems(Startup, (environment_setup, board_setup).chain())
        .add_systems(Update, (draw_mesh_intersections,).chain())
        .add_systems(Update, surface_picking_system)
        .run();
}

#[derive(Component)]
struct SurfaceTile;

#[derive(Component)]
struct ChessBoard;

#[derive(Resource)]
struct MainHandle {
    handle: Handle<Scene>,
}

#[derive(Resource)]
struct WhiteMaterial(Handle<StandardMaterial>);

#[derive(Resource)]
struct BlackMaterial(Handle<StandardMaterial>);

fn environment_setup(mut commands: Commands) {
    // Setup Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.5, 0.5).looking_at(Vec3::ZERO, Vec3::Y),
        MeshPickingCamera,
    ));

    // Setup Lighting
    commands
        .spawn(DirectionalLight {
            shadows_enabled: true,
            illuminance: 5000., // Adjusted for a more reasonable value
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
    // Note Felix: This is not needed yet, I just want to check if and how this works if at all
    let scene_handle = asset_server.load(GltfAssetLabel::Scene(0).from_asset("chess_set.glb"));
    commands.insert_resource(MainHandle {
        handle: scene_handle.clone(),
    });

    // Spawns and also loads assets into the respective c
    commands.spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        SceneRoot(scene_handle),
        ChessBoard,
    ));

    // Parameters
    let square_size = 0.05;

    // Spawn the pickable coponents
    for (row, _row_c) in ('1'..='8').enumerate() {
        for (col, _col_c) in ('a'..='h').enumerate() {
            let (row_offset, col_offset) = utils::idx_to_coordinates(row, col);

            commands.spawn((
                Mesh3d(meshes.add(Plane3d::default().mesh().size(square_size, square_size))),
                Transform::from_xyz(col_offset, 0.01, row_offset),
                Name::new(format!("Tile_{}_{}", row, col)),
                // MeshMaterial3d(materials.add(Color::NONE)),// For release version
                MeshMaterial3d(materials.add(Color::srgba(0.2, 0.5, 0.0, 0.6))),
                SurfaceTile,
            ));
        }
    }
}

fn surface_picking_system(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut click_events: EventReader<Pointer<Click>>,
    tile_query: Query<(Entity, &Name, &MeshMaterial3d<StandardMaterial>), With<SurfaceTile>>,
    mut piece_query: Query<(Entity, &Name, &mut Transform)>,
    mut game_state: ResMut<game_logic::GameState>,
    mut prev_pick: Local<Option<(Figure, usize, usize)>>,
) {
    for click in click_events.read().take(1) {
        if let Ok((tile_ent, tile_name, tile_mat)) = tile_query.get(click.target) {
            reset_tile_highlights(&mut materials, &tile_query);
            let (to_row, to_col) = utils::tile_to_indices(tile_name.as_str());

            // Case 1: We previously picked a valid figure and are about to move the figure now
            if let Some((picked_pigure, from_row, from_col)) = *prev_pick {
                game_state.move_figure_and_asset(
                    &mut commands,
                    picked_pigure.ass_name,
                    (from_row, from_col),
                    (to_row, to_col),
                    &mut piece_query,
                );
                *prev_pick = None;
            }
            // Case 2: We did not yet pick a valid figure and will pick the figure to be moved now
            else {
                //1. Highlight picked filed
                if let Some(material) = materials.get_mut(&tile_mat.0) {
                    material.base_color = Color::srgba(1.0, 0.0, 0.0, 0.6); // modifies existing
                }

                // 2. Highlight the valid fields the piece can move to in case a figure was picked (in another color)
                // (Use the tile query here, to filter by the names of the files we need / write indices to tile name function maybe)

                // // Dummy
                // for (e, n, m) in tile_query {
                //     if n.as_str() == "Tile_1_0" || n.as_str() == "Tile_7_7" {
                //         if let Some(material) = materials.get_mut(&m.0) {
                //             material.base_color = Color::srgba(1.0, 0.0, 0.0, 0.6); // modifies existing
                //         }
                //     }
                // }
                // // End dummy

                let maybe_picked_figure = game_state.get_fig_on_tile(to_row, to_col);
                if let Some(fig) = maybe_picked_figure {
                    *prev_pick = Some((fig, to_row, to_col));
                }
            }
        }
    }
}

fn reset_tile_highlights(
    materials: &mut ResMut<Assets<StandardMaterial>>,
    tile_query: &Query<(Entity, &Name, &MeshMaterial3d<StandardMaterial>), With<SurfaceTile>>,
) {
    for (e, n, mat) in tile_query {
        if let Some(material) = materials.get_mut(&mat.0) {
            material.base_color = Color::srgba(0.2, 0.5, 0.0, 0.6); // modifies existing
        }
    }
}

fn draw_mesh_intersections(pointers: Query<&PointerInteraction>, mut gizmos: Gizmos) {
    for (point, normal) in pointers
        .iter()
        .filter_map(|interaction| interaction.get_nearest_hit())
        .filter_map(|(_entity, hit)| hit.position.zip(hit.normal))
    {
        gizmos.sphere(point, 0.05, RED_500);
        gizmos.arrow(point, point + normal.normalize() * 0.5, PINK_100);
    }
}
