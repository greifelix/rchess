use bevy::{
    color::palettes::css::{BLACK, WHITE},
    prelude::*,
};

use crate::menu::{SelectedColor, SelectedDifficulty, SelectedMode};
use crate::utils::type_utils::{BlackCamera, CameraPosition, PlayerColor, WhiteCamera};

use super::{GuiState, NORMAL_BUTTON, Selected, TEXT_COLOR, button_render_system};
use bevy::color::palettes::css::BURLYWOOD;

#[derive(Resource, Clone, Copy, Default, Eq, PartialEq, Debug, Hash)]
pub enum GameMode {
    #[default]
    PVE,
    PVP,
}

enum ScreenMessage {
    OnePlayerBlack,
    OnePlayerWhite,
    TwoPlayer,
}

#[derive(Message)]
pub struct SwitchScreenMessage(ScreenMessage);

#[derive(Component)]
enum SettingsButtonAction {
    Ok,
    Difficulty(u8),
    PlayerToggle(PlayerColor),
    GameModeToggle(GameMode),
}

#[derive(Resource)]
pub struct GameSettings {
    pub player_color: PlayerColor,
    pub game_mode: GameMode,
    pub difficulty: u8,
}

impl Default for GameSettings {
    fn default() -> Self {
        GameSettings {
            player_color: PlayerColor::White,
            game_mode: GameMode::PVE,
            difficulty: 6,
        }
    }
}

pub fn settings_menu_plugin(app: &mut App) {
    app.add_message::<SwitchScreenMessage>()
        .add_systems(OnEnter(GuiState::SettingsPage), flatter_settings)
        .add_systems(
            Update,
            (
                button_render_system,
                settings_menu_action,
                switch_screen_mode,
            )
                .run_if(in_state(GuiState::SettingsPage)),
        );
}

fn settings_menu_action(
    interaction_query: Query<
        (&Interaction, &SettingsButtonAction, Entity),
        (Changed<Interaction>, With<Button>),
    >,
    // For future reference: Darauf achten, dass mutable queries mututally exclusive sind - sonst runtime panic!! --> Filter nutzen
    selected_difficulty: Single<
        (Entity, &mut BackgroundColor),
        (
            With<SelectedDifficulty>,
            Without<SelectedMode>,
            Without<SelectedColor>,
        ),
    >,
    selected_game_mode: Single<
        (Entity, &mut BackgroundColor),
        (
            With<SelectedMode>,
            Without<SelectedDifficulty>,
            Without<SelectedColor>,
        ),
    >,
    selected_player: Single<
        (Entity, &mut BackgroundColor),
        (
            With<SelectedColor>,
            Without<SelectedDifficulty>,
            Without<SelectedMode>,
        ),
    >,
    mut message_writer: MessageWriter<SwitchScreenMessage>,
    mut gui_state: ResMut<NextState<GuiState>>,
    mut settings: ResMut<GameSettings>,
    mut commands: Commands,
    // black_camera: Query<Entity, With<BlackCamera>>,
) {
    let (previous_diff_button, mut previous_diff_color) = selected_difficulty.into_inner();
    let (previous_mode_button, mut previous_mode_color) = selected_game_mode.into_inner();
    let (previous_player_button, mut previous_player_color) = selected_player.into_inner();

    for (interaction, settings_menu_action, entity) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match settings_menu_action {
                SettingsButtonAction::Ok => {
                    gui_state.set(GuiState::StartPage);
                }
                SettingsButtonAction::Difficulty(val) => {
                    settings.difficulty = *val;
                    *previous_diff_color = NORMAL_BUTTON.into();
                    commands
                        .entity(previous_diff_button)
                        .remove::<(Selected, SelectedDifficulty)>();
                    commands
                        .entity(entity)
                        .insert((Selected, SelectedDifficulty));
                }
                SettingsButtonAction::GameModeToggle(mode) => {
                    settings.game_mode = *mode;
                    *previous_mode_color = NORMAL_BUTTON.into();
                    commands
                        .entity(previous_mode_button)
                        .remove::<(Selected, SelectedMode)>();

                    commands.entity(entity).insert((Selected, SelectedMode));

                    if *mode == GameMode::PVE {
                        if settings.player_color == PlayerColor::White {
                            message_writer
                                .write(SwitchScreenMessage(ScreenMessage::OnePlayerWhite));
                        } else {
                            message_writer
                                .write(SwitchScreenMessage(ScreenMessage::OnePlayerBlack));
                        }
                    } else {
                        message_writer.write(SwitchScreenMessage(ScreenMessage::TwoPlayer));
                    }
                }
                SettingsButtonAction::PlayerToggle(player) => {
                    settings.player_color = *player;
                    *previous_player_color = if *player == PlayerColor::Black {
                        WHITE.into()
                    } else {
                        BLACK.into()
                    };
                    commands
                        .entity(previous_player_button)
                        .remove::<(Selected, SelectedColor)>();
                    commands.entity(entity).insert((Selected, SelectedColor));

                    if settings.game_mode == GameMode::PVE {
                        if *player == PlayerColor::White {
                            message_writer
                                .write(SwitchScreenMessage(ScreenMessage::OnePlayerWhite));
                        } else {
                            message_writer
                                .write(SwitchScreenMessage(ScreenMessage::OnePlayerBlack));
                        }
                    }
                }
            }
        }
    }
}

/// Switches between splitscreen and regular based on incoming messages.
fn switch_screen_mode(
    mut commands: Commands,
    mut message_reader: MessageReader<SwitchScreenMessage>,
    black_camera_query: Query<(Entity, &BlackCamera), Without<WhiteCamera>>,
    white_camera_query: Query<(Entity, &WhiteCamera), Without<BlackCamera>>,
) {
    let Some(msg) = message_reader.read().next() else {
        return;
    };

    for (e, _) in black_camera_query {
        commands.entity(e).despawn();
    }
    for (e, _) in white_camera_query {
        commands.entity(e).despawn();
    }

    match msg.0 {
        ScreenMessage::OnePlayerBlack => {
            commands.spawn((
                Camera3d::default(),
                Transform::from_xyz(0.0, 0.5, -0.5).looking_at(Vec3::ZERO, Vec3::Y),
                Camera {
                    order: 0,
                    ..default()
                },
                CameraPosition {
                    pos: UVec2::new(0, 0),
                },
                MeshPickingCamera,
                BlackCamera,
            ));
        }
        ScreenMessage::OnePlayerWhite => {
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
        }
        ScreenMessage::TwoPlayer => {
            commands.spawn((
                Camera3d::default(),
                Transform::from_xyz(0.0, 0.6, 0.6).looking_at(Vec3::ZERO, Vec3::Y),
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

            commands.spawn((
                Camera3d::default(),
                Transform::from_xyz(0.0, 0.6, -0.6).looking_at(Vec3::ZERO, Vec3::Y),
                Camera {
                    order: 1,
                    ..default()
                },
                CameraPosition {
                    pos: UVec2::new(1, 0),
                },
                MeshPickingCamera,
                BlackCamera,
            ));
        }
    }
}

fn flatter_settings(mut commands: Commands, settings: Res<GameSettings>) {
    // Big 100 % window
    let parent_id = commands
        .spawn((
            DespawnOnExit(GuiState::SettingsPage),
            Node {
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .id();

    // Smaller windows, adapts to size of children (Buttons and text)
    let child_id = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(px(20.0)),
                row_gap: px(20),
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(BURLYWOOD.into()),
        ))
        // All the buttons - pretty wild right now but yeah
        .with_children(|builder| {
            _game_mode_helper(builder, &settings);
            _difficulty_helper(builder, &settings);
            _player_toggle_helper(builder, &settings);
            _back_helper(builder);
        })
        .id();

    commands.entity(parent_id).add_child(child_id);
}

fn _back_helper(builder: &mut ChildSpawnerCommands) {
    builder.spawn((
        Button,
        Node {
            width: px(160),
            height: px(40),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(NORMAL_BUTTON),
        SettingsButtonAction::Ok,
        children![(
            Text::new("Back"),
            TextFont {
                font_size: 33.0,
                ..default()
            },
            TextColor(TEXT_COLOR),
        )],
    ));
}

fn _difficulty_helper(builder: &mut ChildSpawnerCommands, settings: &GameSettings) {
    builder
        .spawn(Node {
            flex_direction: FlexDirection::Row, // Stack buttons horizontally
            column_gap: px(10.0),               // Space between each button
            ..default()
        })
        .with_children(|row_builder| {
            for difficulty_setting in [4, 6, 8] {
                let mut entity = row_builder.spawn((
                    Button,
                    Node {
                        width: px(80),
                        height: px(40),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(NORMAL_BUTTON),
                    SettingsButtonAction::Difficulty(difficulty_setting),
                ));

                if difficulty_setting == 4 {
                    entity.insert((Text::new("Easy"),));
                } else if difficulty_setting == 6 {
                    entity.insert(Text::new("Medium"));
                } else {
                    entity.insert(Text::new("Hard"));
                }

                if difficulty_setting == settings.difficulty {
                    entity.insert((Selected, SelectedDifficulty));
                };
            }
        });
}

fn _game_mode_helper(builder: &mut ChildSpawnerCommands, settings: &GameSettings) {
    builder
        .spawn(Node {
            flex_direction: FlexDirection::Row, // Stack buttons horizontally
            column_gap: px(20.0),               // Space between each button
            ..default()
        })
        .with_children(|row_builder| {
            let mut pve = row_builder.spawn((
                Button,
                Node {
                    width: px(240),
                    height: px(40),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(NORMAL_BUTTON),
                SettingsButtonAction::GameModeToggle(GameMode::PVE),
                Text::new("Player vs Computer"),
            ));

            if settings.game_mode == GameMode::PVE {
                pve.insert((Selected, SelectedMode));
            }

            let mut pvp = row_builder.spawn((
                Button,
                Node {
                    width: px(240),
                    height: px(40),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(NORMAL_BUTTON),
                SettingsButtonAction::GameModeToggle(GameMode::PVP),
                Text::new("Player vs Player"),
            ));

            if settings.game_mode == GameMode::PVP {
                pvp.insert((Selected, SelectedMode));
            }
        });
}

fn _player_toggle_helper(builder: &mut ChildSpawnerCommands, settings: &GameSettings) {
    builder
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: px(0.0),
            ..default()
        })
        .with_children(|row_builder| {
            let mut white_player = row_builder.spawn((
                Button,
                Node {
                    width: px(100),
                    height: px(100),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(WHITE.into()),
                SettingsButtonAction::PlayerToggle(PlayerColor::White),
                Text::new("White"),
                TextColor::BLACK,
            ));

            if settings.player_color == PlayerColor::White {
                white_player.insert((Selected, SelectedColor));
            }

            let mut black_player = row_builder.spawn((
                Button,
                Node {
                    width: px(100),
                    height: px(100),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(BLACK.into()),
                SettingsButtonAction::PlayerToggle(PlayerColor::Black),
                Text::new("Black"),
                TextColor::WHITE,
            ));

            if settings.player_color == PlayerColor::Black {
                black_player.insert((Selected, SelectedColor));
            }
        });
}
