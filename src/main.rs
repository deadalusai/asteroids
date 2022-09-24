#![feature(drain_filter)]

mod game;
mod splash_menu;

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    Menu,
    Game,
}

const ASTEROIDS_TITLE: &str = "Asteroids";
const FIXED_VIEWPORT_WIDTH: f32 = 256.0;

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
        .add_plugin(splash_menu::SplashMenuPlugin)
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
            scaling_mode: bevy::render::camera::ScalingMode::FixedHorizontal(FIXED_VIEWPORT_WIDTH),
            ..default()
        },
        ..default()
    });
}
