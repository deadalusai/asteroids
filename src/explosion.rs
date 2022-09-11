use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_prototype_lyon::prelude::*;
use crate::util::*;
use crate::movable::*;
use crate::torus::*;
use crate::svg::*;

// Explosions

static EXPLOSION_Z: f32 = 30.0;

pub struct ExplosionPlugin;

impl Plugin for ExplosionPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(asset_initialisation_system);
        app.add_system(explosion_spawn_system);
        app.add_system(explosion_system);
        app.add_event::<SpawnExplosionEvent>();
    }
}

// Setup

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExplosionAssetId {
    RocketDebrisA,
    RocketDebrisB,
    AsteroidDebrisA,
    AsteroidDebrisB,
    AsteroidDebrisC,
}

struct ExplosionAssets {
    explosion_part_shapes: HashMap<ExplosionAssetId, Path>,
}

fn asset_initialisation_system(
    mut commands: Commands
) {
    use ExplosionAssetId::*;
    // See: https://yqnn.github.io/svg-path-editor/
    let explosion_part_shapes = vec![
        // id, diameter, path
        // See: https://yqnn.github.io/svg-path-editor/
        (RocketDebrisA, "M 0 -3 L 2 2 M 0 -3 L -0.8 -2.2 M 0.5 0.8 L 1.6 1"),
        (RocketDebrisB, "M -0.2 -1.8 L -2 2 M -1.53 1 L 0.4 1"),
        (AsteroidDebrisA, "M -2 -5 L -5 -2 L -5 0 L -2 0"),
        (AsteroidDebrisB, "M 2 2 L 5 0 L 4 -2 L 1 -5"),
        (AsteroidDebrisC, "M -2 0 L -5 2 L -2 5 L 3 4 "),
    ];
    let explosion_part_shapes = explosion_part_shapes.into_iter()
        .map(|(id, svg)| (id, simple_svg_to_path(svg)))
        .collect();

    commands.insert_resource(ExplosionAssets { explosion_part_shapes });
}

// Entity

#[derive(Component)]
pub struct Explosion {
    start_secs: f32,
    despawn_after_secs: f32,
}

/// Marker component which indicates that an entity should be considered for explosion collisions
#[derive(Component)]
pub struct ExplosionCollidable;

// Spawning

pub struct SpawnExplosionEvent {
    pub mesh_id: ExplosionAssetId,
    pub mesh_scale: f32,
    pub position: Vec2,
    pub velocity: Vec2,
    pub heading_angle: f32,
    pub rotational_velocity: f32,
    pub despawn_after_secs: f32,
}

fn explosion_spawn_system(
    time: Res<Time>,
    assets: Res<ExplosionAssets>,
    mut spawn_events: EventReader<SpawnExplosionEvent>,
    mut commands: Commands
) {
    for ev in spawn_events.iter() {
        spawn_explosion(&time, &assets, &mut commands, ev);
    }
}

const LINE_WIDTH: f32 = 2.0;

fn spawn_explosion(
    time: &Res<Time>,
    assets: &Res<ExplosionAssets>,
    commands: &mut Commands,
    spawn: &SpawnExplosionEvent
) {
    let explosion_color = Color::rgba(0.8, 0.8, 0.8, 1.0);
    let explosion_draw_mode = DrawMode::Stroke(StrokeMode::new(explosion_color, LINE_WIDTH / spawn.mesh_scale));

    // Transform
    let transform = Transform::default()
        .with_translation(Vec3::new(spawn.position.x, spawn.position.y, EXPLOSION_Z))
        .with_rotation(heading_angle_to_transform_rotation(spawn.heading_angle))
        .with_scale(Vec3::splat(spawn.mesh_scale));

    let shape = assets.explosion_part_shapes.get(&spawn.mesh_id).unwrap();

    commands
        .spawn()
        .insert(Explosion {
            start_secs: time.seconds_since_startup() as f32,
            despawn_after_secs: spawn.despawn_after_secs,
        })
        .insert(Movable {
            position: spawn.position,
            velocity: spawn.velocity,
            acceleration: None,
            heading_angle: spawn.heading_angle,
            rotational_velocity: spawn.rotational_velocity,
            rotational_acceleration: None,
        })
        .insert(TorusConstraint)
        // Rendering
        .insert_bundle(GeometryBuilder::build_as(
            shape,
            explosion_draw_mode,
            transform
        ));
}

fn explosion_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &Explosion, &mut DrawMode)>
) {
    let time_secs = time.seconds_since_startup() as f32;
    for (entity, explosion, mut draw_mode) in query.iter_mut() {
        let time_passed_secs = time_secs - explosion.start_secs;
        // Despawn
        if time_passed_secs >= explosion.despawn_after_secs {
            commands.entity(entity).despawn();
            continue;
        }
        // Slowly fade to black
        let t = time_passed_secs / explosion.despawn_after_secs; // t is 0.0 to 1.0;
        let new_alpha = 1.0 - t;
        try_update_stroke_alpha(&mut draw_mode, new_alpha);
    }
}
