use bevy::prelude::*;

use crate::{AppState, game::movable::MovableGlobalState};

// Plugins

pub struct PauseScreenPlugin;

impl Plugin for PauseScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::Pause),
            pause_setup_system
        );
        app.add_systems(
            OnExit(AppState::Pause),
            pause_cleanup_system
        );
        app.add_systems(
            Update, 
            pause_keyboard_system
                .run_if(in_state(AppState::Pause))
        );
    }
}

// Components

#[derive(Component)]
struct PauseRoot;

fn pause_setup_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut movable_state: ResMut<MovableGlobalState>,
) {
    let font_light = asset_server.load(crate::asset_paths::FONT_MONO_LIGHT);

    let margin_style = Style {
        margin: UiRect::all(Val::Px(20.0)),
        ..default()
    };

    let title_text_style = TextStyle {
        font: font_light.clone(),
        font_size: 90.0,
        color: Color::WHITE,
    };

    let secondary_text_style = TextStyle {
        font: font_light,
        font_size: 50.0,
        color: Color::GRAY,
    };

    // Root node
    commands
        .spawn((
            PauseRoot,
            NodeBundle {
                style: Style {
                    height: Val::Percent(100.0),
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                background_color: Color::NONE.into(),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn(
                TextBundle::from_section("PAUSE", title_text_style)
                .with_style(margin_style.clone())
            );
            parent.spawn(
                TextBundle::from_section("Press [esc] to continue", secondary_text_style)
                .with_style(margin_style.clone())
            );
        });

    // Disable movables
    movable_state.enabled = false;
}

fn pause_cleanup_system(
    mut commands: Commands,
    fragments: Query<Entity, With<PauseRoot>>,
    mut movable_state: ResMut<MovableGlobalState>,
) {
    for entity in fragments.iter() {
        commands
            .entity(entity)
            .despawn_recursive();
    }

    // Re-enable movables
    movable_state.enabled = true;
}

fn pause_keyboard_system(
    mut kb: ResMut<Input<KeyCode>>,
    mut next_app_state: ResMut<NextState<AppState>>
) {
    if kb.clear_just_released(KeyCode::Escape) {
        next_app_state.set(AppState::Game);
    }
}