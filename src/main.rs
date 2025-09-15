use bevy::color::palettes::tailwind::*;
use bevy::picking::pointer::PointerInteraction;
use bevy::prelude::*;

use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

mod utils;
use rchess::game_logic::{self, Figure, PossibleMoves};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MeshPickingPlugin))
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .insert_resource(game_logic::GameState::new())
        .add_systems(Startup, (environment_setup, board_setup).chain())
        .add_systems(Update, (draw_mesh_intersections,).chain())
        .add_systems(Update, figure_picking)
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
                MeshMaterial3d(materials.add(Color::NONE)), // For release version
                // MeshMaterial3d(materials.add(Color::srgba(0.2, 0.5, 0.0, 0.6))),
                SurfaceTile,
            ));
        }
    }
}

// TODO: Vielleicht System vorschalten,
// welches immer checkt, ob es überhaupt noch valid moves gibt und andererseits das Spiel sofort beenden
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
                let x = game_logic::calculate_naive_moves(
                    &game_state.board,
                    (clicked_row, clicked_col),
                );
                let possible_moves =
                    game_state.block_selfchecking_moves((clicked_row, clicked_col), x);
                let move_list_str: Vec<String> = possible_moves
                    .iter()
                    .map(|(r, c)| format!("Tile_{r}_{c}"))
                    .collect();
                // TODO: Replace by nonblockig moves?
                game_state.possible_moves = Some(PossibleMoves {
                    from: (clicked_row, clicked_col),
                    to: possible_moves,
                });

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
    for (e, n, mat) in tile_query {
        if let Some(material) = materials.get_mut(&mat.0) {
            material.base_color = Color::NONE;
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
