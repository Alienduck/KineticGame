use bevy::{
    app::{Plugin, Startup},
    camera::visibility::Visibility,
    color::{Color, Srgba},
    ecs::{
        component::Component,
        query::With,
        system::{Commands, Query},
    },
    input::{keyboard::KeyCode, ButtonInput},
    prelude::*,
    ui::{BackgroundColor, Node, Val},
    utils::default,
    window::{CursorGrabMode, CursorOptions},
};

use crate::player::Player;

#[derive(Component)]
pub struct CloseButton;

#[derive(Component)]
pub struct MenuContainer;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, (toggle_menu, close_game));
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Node {
            left: Val::Percent(0.0),
            top: Val::Percent(0.0),
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            align_content: AlignContent::Center,
            ..default()
        },
        BackgroundColor(Color::Srgba(Srgba::new(0.0, 0.0, 0.0, 0.5))),
        Visibility::Hidden,
        MenuContainer,
        children![(
            Node {
                width: Val::Percent(20.0),
                height: Val::Percent(5.0),
                ..default()
            },
            Text("Close game".into()),
            TextColor(Color::srgb(0.0, 0.0, 0.0)),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            CloseButton,
            BackgroundColor(Color::Srgba(Srgba {
                red: 0.2,
                green: 0.2,
                blue: 0.2,
                alpha: 0.5
            })),
            Button
        )],
    ));
}

fn toggle_menu(
    inputs: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut Player>,
    mut query: Query<&mut Visibility, With<MenuContainer>>,
    mut cursor_options: Single<&mut CursorOptions>,
) {
    let Ok(mut player) = player_query.single_mut() else {
        return;
    };
    if inputs.just_pressed(KeyCode::Escape) {
        if let Ok(mut visibility) = query.single_mut() {
            *visibility = match *visibility {
                Visibility::Hidden => {
                    player.state = crate::player::PlayerState::InMenu;
                    cursor_options.grab_mode = CursorGrabMode::None;
                    cursor_options.visible = true;
                    Visibility::Visible
                }
                _ => {
                    player.state = crate::player::PlayerState::InGame;
                    cursor_options.grab_mode = CursorGrabMode::Locked;
                    cursor_options.visible = false;
                    Visibility::Hidden
                }
            };
        }
    }
}

fn close_game(
    mut interaction_query: Query<&Interaction, (With<CloseButton>, Changed<Interaction>)>,
    mut exit: MessageWriter<AppExit>,
) {
    for interaction in &mut interaction_query {
        match interaction {
            Interaction::Pressed => {
                exit.write(AppExit::Success);
            }
            _ => {}
        };
    }
}
