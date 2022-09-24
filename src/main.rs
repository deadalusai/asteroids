#![feature(drain_filter)]

mod game;
mod splash_screen;
mod pause_screen;
mod game_over_screen;

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    Menu,
    Game,
    Pause,
    GameOver
}

const ASTEROIDS_TITLE: &str = "Asteroids";
const FIXED_WIDTH_HEIGHT: f32 = 200.0;

fn main() {
    let title = ASTEROIDS_TITLE.into();
    let (width, height) = (1600., 1200.);
    App::new()
        // bevy
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .insert_resource(WindowDescriptor {
            title,
            width,
            height,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        // state management
        .add_state(AppState::Menu)
        // bevy_prototype_lyon
        .insert_resource(Msaa { samples: 4 })
        .add_plugin(ShapePlugin)
        .add_plugins(game::GamePluginGroup)
        .add_plugin(splash_screen::SplashScreenPlugin)
        .add_plugin(game_over_screen::GameOverScreenPlugin)
        .add_plugin(pause_screen::PauseScreenPlugin)
        .add_startup_system(startup_system)
        .run();
}

fn startup_system(mut commands: Commands) {
    // Spawn a camera
    // NOTE: Our graphics are small! Tune the projection to keep the size of the "world" known
    // despite window scale
    commands.spawn_bundle(Camera2dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 999.9),
        projection: bevy::render::camera::OrthographicProjection {
            far: 1000.0,
            depth_calculation: bevy::render::camera::DepthCalculation::ZDifference,
            // scaling_mode: bevy::render::camera::ScalingMode::FixedHorizontal(FIXED_VIEWPORT_WIDTH),
            scaling_mode: bevy::render::camera::ScalingMode::Auto { min_width: FIXED_WIDTH_HEIGHT, min_height: FIXED_WIDTH_HEIGHT },
            ..default()
        },
        ..default()
    });
}
