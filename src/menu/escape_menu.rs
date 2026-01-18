use bevy::prelude::*;

use super::{GuiState, NORMAL_BUTTON, TEXT_COLOR, button_render_system};
use bevy::color::palettes::css::BURLYWOOD;

// All actions that can be triggered from a button click
#[derive(Component)]
enum EscapeButtonAction {
    Back,
    Restart,
    BackToMainMenu,
}

pub fn escape_menu_plugin(app: &mut App) {
    app.add_systems(OnEnter(GuiState::EscapePage), escape_menu_setup)
        .add_systems(
            Update,
            (button_render_system, escape_menu_action).run_if(in_state(GuiState::EscapePage)), //menu_action,
        );
    // .add_systems(Update, escape_system); // Einmal reicht?
}

fn escape_menu_action(
    interaction_query: Query<
        (&Interaction, &EscapeButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut gui_state: ResMut<NextState<GuiState>>,
) {
    for (interaction, escape_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match escape_button_action {
                EscapeButtonAction::Back => {
                    gui_state.set(GuiState::InGame);
                }
                EscapeButtonAction::Restart => gui_state.set(GuiState::Restart),
                EscapeButtonAction::BackToMainMenu => gui_state.set(GuiState::StartPage),
                _ => {
                    println!("Pressed some unexpected button!");
                }
            }
        }
    }
}

fn escape_menu_setup(mut commands: Commands) {
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
        DespawnOnExit(GuiState::EscapePage),
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
                // Display the game name
                (
                    Text::new("Pause"),
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
                    EscapeButtonAction::Back,
                    children![(
                        Text::new("Back"),
                        button_text_font.clone(),
                        TextColor(TEXT_COLOR),
                    ),]
                ),
                (
                    Button,
                    button_node.clone(),
                    BackgroundColor(NORMAL_BUTTON),
                    EscapeButtonAction::Restart,
                    children![(
                        Text::new("Restart"),
                        button_text_font.clone(),
                        TextColor(TEXT_COLOR),
                    ),]
                ),
                (
                    Button,
                    button_node,
                    BackgroundColor(NORMAL_BUTTON),
                    EscapeButtonAction::BackToMainMenu,
                    children![(
                        Text::new("Main menu"),
                        button_text_font,
                        TextColor(TEXT_COLOR),
                    ),]
                ),
            ]
        )],
    ));
}
