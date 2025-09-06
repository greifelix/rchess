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
use rchess::GameLogic::{self, FigType, Figure};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MeshPickingPlugin))
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .insert_resource(GameLogic::GameState::new())
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

/// Kills target asset
fn kill_helper(
    commands: &mut Commands,
    target_name: &str,
    piece_query: &mut Query<(Entity, &Name, &mut Transform)>,
) {
    for (e, n, _t) in piece_query {
        if n.as_str() == target_name {
            commands.entity(e).despawn();
        }
    }
}

fn surface_picking_system(
    mut commands: Commands, // commands.entity(entity).despawn();
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut click_events: EventReader<Pointer<Click>>,
    tile_query: Query<(Entity, &Name, &MeshMaterial3d<StandardMaterial>), With<SurfaceTile>>,
    mut piece_query: Query<(Entity, &Name, &mut Transform)>,
    mut game_state: ResMut<GameLogic::GameState>,
    mut prev_pick: Local<Option<(Figure, usize, usize)>>,
) {
    for click in click_events.read().take(1) {
        if let Ok((ent, name, mat)) = tile_query.get(click.target) {
            reset_tile_highlights(&mut materials, &tile_query);
            let (row, col) = utils::tile_to_indices(name.as_str());

            // Case 1: We previously picked a valid figure and are about to move the figure now
            if let Some((p_fig, p_row, p_col)) = *prev_pick {
                if game_state.move_is_valid((p_row, p_col), (row, col)) {
                    // If we move to an occupied field we kill
                    if let Some(target_name) = game_state.get_figure_name(row, col) {
                        kill_helper(&mut commands, target_name, &mut piece_query);
                    }
                    // game_state.move_figure_and_asset()

                    // game_state.despawn_entity_on_field(commands, piece_query, row, col);

                    *prev_pick = None;

                    // Add move logig
                }
            }
            // Case 2: We did not yet pick a valid figure and will pick the figure to be moved now
            else {
                if let Some(material) = materials.get_mut(&mat.0) {
                    material.base_color = Color::srgba(1.0, 0.0, 0.0, 0.6); // modifies existing
                }

                if game_state.pick_is_valid(row, col) {
                    let maybe_picked_figure = game_state.get_fig_on_tile(row, col);
                    if let Some(fig) = maybe_picked_figure {
                        *prev_pick = Some((fig, row, col));
                    }
                }
            }

            // println!("Clicked tile {}", name.as_str());

            // if let Some(selected_figure) = game_state.get_figure_name(row, col) {
            //     println!("Figure on tile is{}", selected_figure);

            //     piece_query
            //         .iter_mut()
            //         .filter(|(e, q_name, pos)| q_name.as_str() == selected_figure)
            //         .for_each(|(_, _, mut pos)| pos.translation += Vec3::new(0.00, 0.00, 0.05));
            // }
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

fn draw_mesh_intersections(
    pointers: Query<&PointerInteraction>,
    mut gizmos: Gizmos,
    
) {
    for (point, normal) in pointers
        .iter()
        .filter_map(|interaction| interaction.get_nearest_hit())
        .filter_map(|(_entity, hit)| hit.position.zip(hit.normal))
    {
        gizmos.sphere(point, 0.05, RED_500);
        gizmos.arrow(point, point + normal.normalize() * 0.5, PINK_100);
    }

}
