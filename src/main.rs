mod movable;
mod player;
mod torus;

use bevy::prelude::*;
use movable::*;
use player::*;
use torus::*;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .insert_resource(WindowDescriptor {
            title: "Asteroids".into(),
            width: 800.,
            height: 600.,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(PlayerPlugin)
        .add_plugin(MovablePlugin)
        .add_plugin(TorusConstraintPlugin)
        .add_startup_system(startup_system)
        .run();
}

#[derive(Component)]
struct Viewport {

}

fn startup_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Camera
    commands.spawn_bundle(Camera2dBundle::default());

    // Player
    PlayerRocket::spawn(&mut commands, &mut meshes, &mut materials);
}
