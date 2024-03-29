use bevy::prelude::*;

use crate::AppState;
use crate::game::manager::{GameManager, GameCleanup};

// Plugins

pub struct SplashScreenPlugin;

impl Plugin for SplashScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::Menu),
            (
                menu_setup_system,
                game_cleanup_system,
            )
        );
        app.add_systems(
            OnExit(AppState::Menu),
            menu_cleanup_system,
        );
        app.add_systems(
            Update, 
            menu_keyboard_system
                .run_if(in_state(AppState::Menu))
        );
    }
}

// Components

#[derive(Component)]
struct MenuRoot;

// Menu

fn menu_setup_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font_light = asset_server.load(crate::asset_paths::FONT_MONO_LIGHT);
    let font_bold = asset_server.load(crate::asset_paths::FONT_MONO_BOLD);

    let margin_style = Style {
        margin: UiRect::all(Val::Px(20.0)),
        ..default()
    };

    let title_text_style = TextStyle {
        font: font_bold,
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
            MenuRoot,
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                background_color: Color::NONE.into(),
                ..default()
            }
        ))
        .with_children(|parent| {
            parent.spawn(
                TextBundle::from_section("ASTEROIDS", title_text_style)
                .with_style(margin_style.clone())
            );
            parent.spawn(
                TextBundle::from_section("Press [space] to begin", secondary_text_style)
                .with_style(margin_style.clone())
            );
        });
}

fn menu_cleanup_system(mut commands: Commands, fragments: Query<Entity, With<MenuRoot>>) {
    for entity in fragments.iter() {
        commands
            .entity(entity)
            .despawn_recursive();
    }
}

fn menu_keyboard_system(
    mut commands: Commands,
    mut kb: ResMut<Input<KeyCode>>,
    mut next_app_state: ResMut<NextState<AppState>>
) {
    if kb.clear_just_released(KeyCode::Space) {
        crate::game::manager::game_create(&mut commands);
        next_app_state.set(AppState::Game);
    }
}

fn game_cleanup_system(
    world: &mut World
)
{
    // If the user enters the main menu, clean up any running game state
    let is_game_initialized = world.get_resource::<GameManager>().is_some();
    if is_game_initialized {
        world.run_schedule(GameCleanup);
    }
}