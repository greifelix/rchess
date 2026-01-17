use bevy::color::palettes::css::{CRIMSON, NAVY};
use bevy::gltf::{Gltf, GltfExtras, GltfMesh};
use bevy::prelude::*;
// use bevy_egui::EguiPlugin;
// use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::game_logic::movement_logic::{self, MoveBuilder};

use crate::game_logic::{FigType, PlayerColor, minmax_logic};

mod game_logic;
mod utils;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MeshPickingPlugin))
        // .add_plugins(EguiPlugin::default())
        // .add_plugins(WorldInspectorPlugin::new())
        .insert_resource(game_logic::GameState::new())
        .insert_resource(game_logic::minmax_logic::GeneratedMoves::new())
        .init_state::<GuiState>()
        .init_state::<GameMode>()
        .add_systems(Startup, (environment_setup, board_setup).chain())
        .add_systems(Update, figure_picking)
        .add_plugins(menu_plugin)
        .add_plugins(player_vs_minmax_plugin)
        .run();
}

// UI-State of the game
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum GuiState {
    #[default]
    StartPage,
    EscapePage,
    SettingsPage,
    GameOver,
    InGame,
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum GameMode {
    #[default]
    PVE,
    PVP,
}

// All actions that can be triggered from a button click
#[derive(Component)]
enum MenuButtonAction {
    Play,
    Settings,
    BackToMainMenu,
    BackToSettings,
    Quit,
}

// This system handles changing all buttons color based on mouse interaction
fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut background_color) in &mut interaction_query {
        *background_color = match (*interaction) {
            Interaction::Pressed => PRESSED_BUTTON.into(),
            Interaction::Hovered => HOVERED_BUTTON.into(),
            Interaction::None => NORMAL_BUTTON.into(),
        }
    }
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const HOVERED_PRESSED_BUTTON: Color = Color::srgb(0.25, 0.65, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);

pub fn menu_plugin(app: &mut App) {
    app.add_systems(OnEnter(GuiState::StartPage), menu_setup)
        .add_systems(
            Update,
            (button_system, menu_action).run_if(in_state(GuiState::StartPage)), //menu_action,
        );
}

fn menu_action(
    interaction_query: Query<
        (&Interaction, &MenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut app_exit_writer: MessageWriter<AppExit>,
    mut gui_state: ResMut<NextState<GuiState>>,
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                MenuButtonAction::Quit => {
                    app_exit_writer.write(AppExit::Success);
                }
                MenuButtonAction::Play => {
                    gui_state.set(GuiState::InGame);
                }
                MenuButtonAction::Settings => gui_state.set(GuiState::SettingsPage),
                _ => {
                    println!("Pressed some unexpected button!");
                }
            }
        }
    }
}

fn menu_setup(mut commands: Commands) {
    // Common style for all buttons on the screen
    let button_node = Node {
        width: px(300),
        height: px(65),
        margin: UiRect::all(px(20)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    let button_text_font = TextFont {
        font_size: 33.0,
        ..default()
    };

    commands.spawn((
        DespawnOnExit(GuiState::StartPage),
        Node {
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        children![(
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(NAVY.into()),
            children![
                // Display the game name
                (
                    Text::new("Rchess Menü"),
                    TextFont {
                        font_size: 67.0,
                        ..default()
                    },
                    TextColor(TEXT_COLOR),
                    Node {
                        margin: UiRect::all(px(50)),
                        ..default()
                    },
                ),
                (
                    Button,
                    button_node.clone(),
                    BackgroundColor(NORMAL_BUTTON),
                    MenuButtonAction::Play,
                    children![(
                        Text::new("New Game"),
                        button_text_font.clone(),
                        TextColor(TEXT_COLOR),
                    ),]
                ),
                (
                    Button,
                    button_node.clone(),
                    BackgroundColor(NORMAL_BUTTON),
                    MenuButtonAction::Settings,
                    children![(
                        Text::new("Settings"),
                        button_text_font.clone(),
                        TextColor(TEXT_COLOR),
                    ),]
                ),
                (
                    Button,
                    button_node,
                    BackgroundColor(NORMAL_BUTTON),
                    MenuButtonAction::Quit,
                    children![(Text::new("Quit"), button_text_font, TextColor(TEXT_COLOR),),]
                ),
            ]
        )],
    ));
}

pub fn player_vs_minmax_plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            minmax_logic::spawn_minmax_task,
            minmax_logic::retrieve_and_exec_minmax_result,
        )
            .run_if(in_state(GameMode::PVE)),
    );

    // // When entering the state, spawn everything needed for this screen
    // .add_systems(OnEnter(GameState::Splash), splash_setup)
    // // While in this state, run the `countdown` system
    // .add_systems(Update, countdown.run_if(in_state(GameState::Splash)));
}

#[derive(Resource)]
pub struct ChessScene(Handle<Gltf>);

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
                        ._filter_brute_force_2(&game_state.board)
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
