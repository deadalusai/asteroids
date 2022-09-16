use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_prototype_lyon::prelude::*;
use crate::movable::{Movable, MovableTorusConstraint};
use crate::svg::simple_svg_to_path;
use crate::util::*;

// Explosions

static EXPLOSION_Z: f32 = 30.0;

pub struct ExplosionPlugin;

impl Plugin for ExplosionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(explosion_system);
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

pub struct ExplosionAssets {
    explosion_part_shapes: HashMap<ExplosionAssetId, Path>,
}

pub fn create_explosion_assets() -> ExplosionAssets {
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

    ExplosionAssets { explosion_part_shapes }
}

// Entity

#[derive(Component)]
pub struct Explosion {
    despawn_timer: Timer,
}

/// Marker component which indicates that an entity should be considered for explosion collisions
#[derive(Component)]
pub struct ExplosionCollidable;

// Spawning

pub struct SpawnExplosion {
    pub shape_id: ExplosionAssetId,
    pub shape_scale: f32,
    pub position: Vec2,
    pub velocity: Vec2,
    pub heading_angle: f32,
    pub rotational_velocity: f32,
    pub despawn_after_secs: f32,
}

const LINE_WIDTH: f32 = 2.0;

pub fn spawn_explosions(
    commands: &mut Commands,
    assets: &ExplosionAssets,
    spawns: &[SpawnExplosion]
) {
    for spawn in spawns {

        let explosion_color = Color::rgba(0.8, 0.8, 0.8, 1.0);
        let explosion_draw_mode = DrawMode::Stroke(StrokeMode::new(explosion_color, LINE_WIDTH / spawn.shape_scale));
    
        // Transform
        let transform = Transform::default()
            .with_translation(Vec3::new(spawn.position.x, spawn.position.y, EXPLOSION_Z))
            .with_rotation(heading_angle_to_transform_rotation(spawn.heading_angle))
            .with_scale(Vec3::splat(spawn.shape_scale));

        let shape = assets.explosion_part_shapes.get(&spawn.shape_id).unwrap();

        commands
            .spawn()
            .insert(Explosion {
                despawn_timer: Timer::from_seconds(spawn.despawn_after_secs, false),
            })
            .insert(Movable {
                position: spawn.position,
                velocity: spawn.velocity,
                acceleration: None,
                heading_angle: spawn.heading_angle,
                rotational_velocity: spawn.rotational_velocity,
                rotational_acceleration: None,
            })
            .insert(MovableTorusConstraint)
            // Rendering
            .insert_bundle(GeometryBuilder::build_as(
                shape,
                explosion_draw_mode,
                transform
            ));
    }
}

fn explosion_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Explosion, &mut DrawMode)>
) {
    for (entity, mut explosion, mut draw_mode) in query.iter_mut() {
        // Update
        explosion.despawn_timer.tick(time.delta());
        // Despawn?
        if explosion.despawn_timer.finished() {
            commands.entity(entity).despawn();
            continue;
        }
        // Slowly fade to transparent
        let percent_left = explosion.despawn_timer.elapsed_secs(); // 1.0 -> 0.0
        update_drawmode_alpha(&mut draw_mode, percent_left);
    }
}
