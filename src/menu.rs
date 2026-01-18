pub mod escape_menu;
pub mod settings;
use bevy::prelude::*;

use bevy::color::palettes::css::NAVY;

use crate::WoodenPiece;
use crate::game_logic::{GameState, minmax_logic};

// UI-State of the game
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum GuiState {
    #[default]
    StartPage,
    EscapePage,
    SettingsPage,
    GameOver,
    InGame,
    Restart,
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

#[derive(Component)]
struct Selected;

/// Resets entire board and stuff
/// TODO: Check if we can get the existing resources instead.
fn reset_system(
    board_query: Query<Entity, With<WoodenPiece>>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GuiState>>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for ent in board_query {
        commands.entity(ent).despawn();
    }

    commands.insert_resource(GameState::new());
    commands.insert_resource(minmax_logic::GeneratedMoves::new());

    // let gltf_handle: Handle<Gltf> = asset_server.load("chess_set.glb");
    // commands.insert_resource(ChessScene(gltf_handle));

    let scene_handle = asset_server.load(GltfAssetLabel::Scene(0).from_asset("chess_set.glb"));
    commands.spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        SceneRoot(scene_handle),
        Name::new("Original Chess Scene"),
        WoodenPiece,
    ));

    next_state.set(GuiState::InGame);
}

fn escape_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GuiState>>,
    state: Res<State<GuiState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        match state.get() {
            GuiState::InGame => next_state.set(GuiState::EscapePage),
            GuiState::EscapePage => next_state.set(GuiState::InGame),
            GuiState::SettingsPage => next_state.set(GuiState::StartPage),
            GuiState::StartPage => (),
            GuiState::Restart => (),
            GuiState::GameOver => next_state.set(GuiState::InGame),
        }
    }
}

// This system handles changing all buttons color based on mouse interaction
fn button_render_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, Option<&Selected>),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut background_color, maybe_selected) in &mut interaction_query {
        *background_color = match (*interaction, maybe_selected) {
            (Interaction::Pressed, _) | (Interaction::None, Some(_)) => PRESSED_BUTTON.into(),
            (Interaction::Hovered, Some(_)) => HOVERED_PRESSED_BUTTON.into(),
            (Interaction::Hovered, None) => HOVERED_BUTTON.into(),
            (Interaction::None, None) => NORMAL_BUTTON.into(),
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
            (button_render_system, menu_action).run_if(in_state(GuiState::StartPage)), //menu_action,
        )
        .add_systems(OnEnter(GuiState::Restart), reset_system)
        .add_systems(Update, escape_system);
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
                    Text::new("Rchess - Bevy fun project"),
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
                        Text::new("To Game"),
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
