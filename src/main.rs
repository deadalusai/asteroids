mod movable;
mod torus;
mod player;
mod asteroid;

use bevy::prelude::*;
use movable::*;
use torus::*;
use player::*;
use asteroid::*;

fn main() {
    App::new()
        .add_event::<SpawnAsteroidEvent>()
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .insert_resource(WindowDescriptor {
            title: "Asteroids".into(),
            width: 800.,
            height: 600.,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(MovablePlugin)
        .add_plugin(TorusConstraintPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(AsteroidPlugin)
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
    mut spawn_events: EventWriter<SpawnAsteroidEvent>,
) {
    // Camera
    commands.spawn_bundle(Camera2dBundle::default());

    // Player
    PlayerRocket::spawn(&mut commands, &mut meshes, &mut materials);

    spawn_events.send(SpawnAsteroidEvent(AsteroidSize::Large));
    spawn_events.send(SpawnAsteroidEvent(AsteroidSize::Medium));
    spawn_events.send(SpawnAsteroidEvent(AsteroidSize::Small));
}
