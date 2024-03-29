use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_prototype_lyon::prelude::*;
use crate::AppState;
use super::manager::GameCleanup;
use super::movable::Movable;
use super::svg::simple_svg_to_path;
use super::util::*;

// Explosions

static EXPLOSION_Z: f32 = 30.0;

pub struct ExplosionPlugin;

impl Plugin for ExplosionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, 
            explosion_system
                .run_if(in_state(AppState::Game))
        );
        app.add_systems(GameCleanup, destroy_explosions_system);
    }
}

// Setup

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExplosionShapeId {
    RocketDebris,
    AsteroidDebris,
    UfoDebris,
}

pub struct ExplosionPart {
    direction: Vec2,
    shape: Path,
}

pub struct ExplosionAssets {
    explosion_parts: HashMap<ExplosionShapeId, Vec<ExplosionPart>>,
}

pub fn create_explosion_assets() -> ExplosionAssets {
    fn unit(x: f32, y: f32) -> Vec2 {
        Vec2::new(x, y).normalize()
    }
    use ExplosionShapeId::*;
    // See: https://yqnn.github.io/svg-path-editor/
    let explosion_part_directions_and_shapes = vec![
        // id, diameter, path
        // See: https://yqnn.github.io/svg-path-editor/
        (RocketDebris, unit( 1., -1.), "M 3 0 L 1.6 -0.8 M 3 0 L -2 2 M -1 1.6 L -0.9 0.6"),
        (RocketDebris, unit(-1.,  1.), "M -2 -2 L 1.6 -0.5 M -1 -1.6 L -1 0.6"),
        // Asteroid Debris
        (AsteroidDebris, unit(-1.,  1.), "M 1 -5 L -2 -5 L -5 -2 L -5 0 L -3 0"),
        (AsteroidDebris, unit( 4.,  2.), "M 2 2 L 5 0 L 4 -2 L 1 -5"),
        (AsteroidDebris, unit(-1., -2.), "M -2 0 L -5 2 L -2 5 L 3 4 L 2 2"),
        // Ufo Debris
        (UfoDebris, unit(0.,  1.), "M 5 0 C 2 -2 -2 -2 -5 0 L -5 1 M 3 -1 Q 0 -5 -3 -1"),
        (UfoDebris, unit(0., -1.), "M -5 1 C -2 3 2 3 5 1 L 5 0"),
    ];
    let mut explosion_parts = HashMap::new();
    for (explosion_id, direction, svg) in explosion_part_directions_and_shapes.into_iter() {
        let shape = simple_svg_to_path(svg);
        explosion_parts
            .entry(explosion_id).or_insert(Vec::new())
            .push(ExplosionPart { direction, shape });
    }
    ExplosionAssets { explosion_parts }
}

// Teardown

fn destroy_explosions_system(mut commands: Commands, query: Query<Entity, With<Explosion>>) {
    for entity in query.iter() {
        commands
            .entity(entity)
            .despawn_recursive();
    }
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

#[derive(Clone)]
pub struct SpawnExplosion {
    pub shape_id: ExplosionShapeId,
    pub shape_scale: f32,
    pub position: Vec2,
    pub velocity: Vec2,
    pub heading_angle: f32,
    pub rotational_velocity: f32,
    pub despawn_after_secs: f32,
}

const LINE_WIDTH: f32 = 0.2;
const EXPLOSION_PART_MIN_ADD_SPEED: f32 = 10.0;
const EXPLOSION_PART_MAX_ADD_SPEED: f32 = 25.0;

pub fn spawn_explosion(
    commands: &mut Commands,
    rng: &mut rand::rngs::ThreadRng,
    assets: &ExplosionAssets,
    spawn: SpawnExplosion
) {
    let explosion_color = Color::rgba(0.8, 0.8, 0.8, 1.0);
    let explosion_stroke = Stroke::new(explosion_color, LINE_WIDTH / spawn.shape_scale);
    let explosion_part_speed = EXPLOSION_PART_MIN_ADD_SPEED + rng.random_f32() * (EXPLOSION_PART_MAX_ADD_SPEED - EXPLOSION_PART_MIN_ADD_SPEED);

    let parts = assets.explosion_parts.get(&spawn.shape_id).unwrap();
    for part in parts.iter() {
        let rotation = Vec2::from_angle(spawn.heading_angle);
        let position = spawn.position;
        let velocity = spawn.velocity + (part.direction.rotate(rotation) * explosion_part_speed);
        let transform = Transform::default()
            .with_translation(Vec3::new(position.x, position.y, EXPLOSION_Z))
            .with_rotation(Quat::from_rotation_z(spawn.heading_angle))
            .with_scale(Vec3::splat(spawn.shape_scale));

        commands
            .spawn((
                Explosion {
                    despawn_timer: Timer::from_seconds(spawn.despawn_after_secs, TimerMode::Once),
                },
                Movable {
                    position,
                    velocity,
                    acceleration: None,
                    heading_angle: spawn.heading_angle,
                    rotational_velocity: spawn.rotational_velocity,
                    rotational_acceleration: None,
                },
                // Rendering
                ShapeBundle {
                    transform,
                    path: Path(part.shape.0.clone()),
                    ..default()
                },
                explosion_stroke
            ));
    }
}

fn explosion_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Explosion, &mut Stroke)>
) {
    for (entity, mut explosion, mut stroke) in query.iter_mut() {
        // Update
        explosion.despawn_timer.tick(time.delta());
        // Despawn?
        if explosion.despawn_timer.finished() {
            commands.entity(entity).despawn();
            continue;
        }
        // Slowly fade to transparent
        let percent_left = explosion.despawn_timer.percent_left(); // 1.0 -> 0.0
        stroke.color.set_a(percent_left);
    }
}