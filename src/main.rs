#![feature(extract_if)]

mod asset_paths;
mod game;
mod splash_screen;
mod pause_screen;
mod game_over_screen;

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, States)]
enum AppState {
    #[default]
    Menu,
    Game,
    Pause,
    GameOver
}

const ASTEROIDS_TITLE: &str = "Asteroids";
const FIXED_WIDTH_HEIGHT: f32 = 200.0;

fn main() {
    let title = ASTEROIDS_TITLE.into();
    let (width, height) = (1024., 768.);
    App::new()
        // bevy
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title,
                    resolution: (width, height).into(),
                    ..default()
                }),
                ..default()
            }))
        // state management
        .add_state::<AppState>()
        // bevy_prototype_lyon
        .insert_resource(Msaa::Sample4)
        .add_plugins(ShapePlugin)
        // Game
        .add_plugins((
            game::GamePluginGroup,
            splash_screen::SplashScreenPlugin,
            game_over_screen::GameOverScreenPlugin,
            pause_screen::PauseScreenPlugin
        ))
        .add_systems(Startup, startup_system)
        .run();
}

fn startup_system(mut commands: Commands) {
    // Spawn a camera
    // NOTE: Our graphics are small! Tune the projection to keep the size of the "world" known
    // despite window scale
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 999.9),
        projection: bevy::render::camera::OrthographicProjection {
            far: 1000.0,
            // scaling_mode: bevy::render::camera::ScalingMode::FixedHorizontal(FIXED_VIEWPORT_WIDTH),
            scaling_mode: bevy::render::camera::ScalingMode::AutoMin {
                min_width: FIXED_WIDTH_HEIGHT,
                min_height: FIXED_WIDTH_HEIGHT,
            },
            ..default()
        },
        ..default()
    });
}
