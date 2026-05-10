use crate::{player::Player, state::AppState};
use bevy::{
    app::AppExit,
    prelude::*,
    window::{CursorGrabMode, CursorOptions},
};
use bevy_rapier3d::dynamics::{ExternalImpulse, Velocity};

pub struct MenuPlugin;
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_menu)
            .add_systems(Update, toggle_state)
            .add_systems(OnEnter(AppState::Menu), show_menu)
            .add_systems(OnExit(AppState::Menu), hide_menu)
            .add_systems(Update, handle_buttons.run_if(in_state(AppState::Menu)));
    }
}

#[derive(Component)]
pub struct MenuContainer;

#[derive(Component)]
pub enum MenuAction {
    Quit,
    Resume,
    Respawn,
}

fn setup_menu(mut commands: Commands) {
    let container = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                column_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            Visibility::Hidden,
            MenuContainer,
        ))
        .id();

    let quit_btn = commands
        .spawn((
            Button,
            Node {
                width: Val::Px(200.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.8, 0.2, 0.2)),
            MenuAction::Quit,
        ))
        .with_child((
            Text::new("Quitter"),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ))
        .id();

    let respawn_btn = commands
        .spawn((
            Button,
            Node {
                width: Val::Px(200.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::BLACK),
            MenuAction::Respawn,
        ))
        .with_child((
            Text("Respawn".into()),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ))
        .id();

    commands
        .entity(container)
        .add_children(&[quit_btn, respawn_btn]);
}

fn toggle_state(
    inputs: Res<ButtonInput<KeyCode>>,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if inputs.just_pressed(KeyCode::Escape) {
        next_state.set(match state.get() {
            AppState::InGame => AppState::Menu,
            AppState::Menu => AppState::InGame,
        });
    }
}

fn show_menu(
    mut query: Query<&mut Visibility, With<MenuContainer>>,
    mut cursor_options: Query<&mut CursorOptions>,
) {
    if let Ok(mut vis) = query.single_mut() {
        *vis = Visibility::Visible;
    }
    if let Ok(mut cursor_options) = cursor_options.single_mut() {
        cursor_options.visible = true;
        cursor_options.grab_mode = CursorGrabMode::None;
    }
}

fn hide_menu(
    mut query: Query<&mut Visibility, With<MenuContainer>>,
    mut cursor_options: Query<&mut CursorOptions>,
) {
    if let Ok(mut vis) = query.single_mut() {
        *vis = Visibility::Hidden;
    }
    if let Ok(mut cursor_options) = cursor_options.single_mut() {
        cursor_options.visible = false;
        cursor_options.grab_mode = CursorGrabMode::Locked;
    }
}

fn handle_buttons(
    mut interactions: Query<
        (&Interaction, &MenuAction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut player_query: Query<
        (
            &mut Transform,
            Option<&mut Velocity>,
            Option<&mut ExternalImpulse>,
        ),
        With<Player>,
    >,
    mut app_exit: MessageWriter<AppExit>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (interaction, action, mut color) in &mut interactions {
        match *interaction {
            Interaction::Pressed => {
                *color = color.0.darker(0.3).into();
                match action {
                    MenuAction::Quit => {
                        app_exit.write(AppExit::Success);
                    }
                    MenuAction::Resume => next_state.set(AppState::InGame),
                    MenuAction::Respawn => {
                        if let Ok((mut transform, velocity, impulse)) = player_query.single_mut() {
                            transform.translation = Vec3::new(0.0, 10.0, 0.0);
                            if let Some(mut vel) = velocity {
                                vel.linvel = Vec3::ZERO;
                                vel.angvel = Vec3::ZERO;
                            }
                            if let Some(mut imp) = impulse {
                                imp.impulse = Vec3::ZERO;
                                imp.torque_impulse = Vec3::ZERO;
                            }
                        }
                        next_state.set(AppState::InGame);
                    }
                }
            }
            Interaction::Hovered => {
                *color = color.0.lighter(0.2).into();
            }
            Interaction::None => {
                *color = color.0.darker(0.2).into();
            }
        }
    }
}
