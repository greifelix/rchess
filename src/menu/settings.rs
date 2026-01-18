use bevy::prelude::*;

use crate::game_logic::PlayerColor;

use super::{GuiState, NORMAL_BUTTON, Selected, TEXT_COLOR, button_render_system};
use bevy::color::palettes::css::BURLYWOOD;
#[derive(Resource, Clone, Copy, Default, Eq, PartialEq, Debug, Hash)]
pub enum GameMode {
    #[default]
    PVE,
    PVP,
}

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
    pub timer: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        GameSettings {
            player_color: PlayerColor::White,
            game_mode: GameMode::PVE,
            difficulty: 6,
            timer: false,
        }
    }
}

pub fn settings_menu_plugin(app: &mut App) {
    app.add_systems(OnEnter(GuiState::SettingsPage), settings_menu_setup)
        .add_systems(
            Update,
            (button_render_system, settings_menu_action).run_if(in_state(GuiState::SettingsPage)),
        );
}

fn settings_menu_action(
    interaction_query: Query<
        (&Interaction, &SettingsButtonAction, Entity),
        (Changed<Interaction>, With<Button>),
    >,
    selected_difficulty_query: Single<(Entity, &mut BackgroundColor), With<Selected>>,

    mut gui_state: ResMut<NextState<GuiState>>,
    mut settings: ResMut<GameSettings>,
    mut commands: Commands,
) {
    let (previous_button, mut previous_button_color) = selected_difficulty_query.into_inner();

    for (interaction, settings_menu_action, entity) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match settings_menu_action {
                SettingsButtonAction::Ok => {
                    gui_state.set(GuiState::StartPage);
                }
                SettingsButtonAction::Difficulty(val) => {
                    settings.difficulty = *val;
                    *previous_button_color = NORMAL_BUTTON.into();
                    commands.entity(previous_button).remove::<Selected>();
                    commands.entity(entity).insert(Selected);
                }

                _ => {
                    println!("Pressed some unexpected button!");
                }
            }
        }
    }
}

fn settings_menu_setup(mut commands: Commands, settings: Res<GameSettings>) {
    let default_diff = settings.difficulty;
    let default_player = settings.player_color;
    let default_game_mode = settings.game_mode;

    let button_node = Node {
        width: px(200),
        height: px(65),
        margin: UiRect::all(px(20)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_text_style = (
        TextFont {
            font_size: 33.0,
            ..default()
        },
        TextColor(TEXT_COLOR),
    );

    let button_node_clone = button_node.clone();
    commands.spawn((
        DespawnOnExit(GuiState::SettingsPage),
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
            BackgroundColor(BURLYWOOD.into()),
            children![
                (
                    Node {
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    Children::spawn((
                        Spawn((Text::new("Difficulty"), button_text_style.clone())),
                        SpawnWith(move |parent: &mut ChildSpawner| {
                            for difficulty_setting in [4, 6, 8] {
                                let mut entity = parent.spawn((
                                    Button,
                                    Node {
                                        width: px(80),
                                        height: px(40),
                                        ..button_node_clone.clone()
                                    },
                                    BackgroundColor(NORMAL_BUTTON),
                                    SettingsButtonAction::Difficulty(difficulty_setting),
                                ));

                                if difficulty_setting == 4 {
                                    entity.insert(Text::new("Easy"));
                                } else if difficulty_setting == 6 {
                                    entity.insert(Text::new("Medium"));
                                } else {
                                    entity.insert(Text::new("Hard"));
                                }

                                if difficulty_setting == default_diff {
                                    entity.insert(Selected);
                                }
                            }
                        })
                    ))
                ),
                (
                    Button,
                    button_node,
                    BackgroundColor(NORMAL_BUTTON),
                    SettingsButtonAction::Ok,
                    children![(Text::new("Back"), button_text_style)]
                )
            ]
        )],
    ));
}
