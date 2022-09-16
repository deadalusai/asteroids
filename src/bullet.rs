use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use crate::SystemLabel;
use crate::assets::GameAssets;
use crate::hit::*;
use crate::movable::*;
use crate::collidable::*;
use crate::svg::simple_svg_to_path;
use crate::util::*;

// Bullets

static BULLET_SCALE: f32 = 5.0;
static BULLET_Z: f32 = 5.0;

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            bullet_controller_system
                .after(SystemLabel::Input)
        );
        app.add_system(
            bullet_collision_system
                .label(SystemLabel::Collision)
                .after(SystemLabel::Movement)
        );
        app.add_system_to_stage(CoreStage::PostUpdate, bullet_despawn_system);
    }
}

// Setup

pub struct BulletAssets {
    bullet_dimension: f32, // h of the bullet shape
    bullet_shape: Path,
}

pub fn create_bullet_assets() -> BulletAssets {
    // See: https://yqnn.github.io/svg-path-editor/
    let bullet_dimension = 2.0;
    let bullet_path = "M 0 1 L 0 -1";

    BulletAssets {
        bullet_dimension,
        bullet_shape: simple_svg_to_path(bullet_path),
    }
}

// Entity

#[derive(Component)]
pub struct Bullet {
    despawn_timer: Timer,
}

/// Marker component which indicates that an entity should be considered for bullet collisions
#[derive(Component)]
pub struct BulletCollidable;

// Spawning

pub struct BulletSpawn {
    pub position: Vec2,
    pub velocity: Vec2,
    pub heading_angle: f32,
    pub despawn_after_secs: f32,
}

const LINE_WIDTH: f32 = 2.0;

pub fn spawn_bullet(
    commands: &mut Commands,
    assets: &BulletAssets,
    spawn: BulletSpawn
) {
    let scale = BULLET_SCALE;
    let bullet_color = Color::rgba(0.8, 0.8, 0.8, 1.0);
    let bullet_draw_mode = DrawMode::Stroke(StrokeMode::new(bullet_color, LINE_WIDTH / scale));

    // Transform
    let transform = Transform::default()
        .with_translation(Vec3::new(spawn.position.x, spawn.position.y, BULLET_Z))
        .with_rotation(heading_angle_to_transform_rotation(spawn.heading_angle))
        .with_scale(Vec3::splat(scale));

    // collision detection
    let collider = Collider::circle(spawn.position.into(), scale * assets.bullet_dimension / 2.);

    commands
        .spawn()
        .insert(Bullet {
            despawn_timer: Timer::from_seconds(spawn.despawn_after_secs, false),
        })
        .insert(Movable {
            position: spawn.position,
            velocity: spawn.velocity,
            acceleration: None,
            heading_angle: spawn.heading_angle,
            rotational_velocity: 0.,
            rotational_acceleration: None,
        })
        .insert(MovableTorusConstraint)
        // Rendering
        .insert_bundle(GeometryBuilder::build_as(
            &assets.bullet_shape,
            bullet_draw_mode,
            transform
        ))
        // Collision detection
        .insert(Collidable { collider });
}

fn bullet_despawn_system(
    mut commands: Commands,
    mut hit_events: EventReader<HitEvent>,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Bullet)>
) {
    // Despawn bullets which have hit something
    for &HitEvent(entity) in distinct_hit_events(&mut hit_events) {
        if let Ok(_) = query.get(entity) {
            commands.entity(entity).despawn();
        }
    }

    // Despawn bullets which have expired
    for (entity, mut bullet) in query.iter_mut() {
        bullet.despawn_timer.tick(time.delta());
        if bullet.despawn_timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

// Fire control

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UpdateResult {
    None,
    FireBullet,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum BulletControllerState {
    None,
    Firing,
    Cooldown,
}

#[derive(Component)]
pub struct BulletController {
    timer: Timer,
    state: BulletControllerState,
    fire_count: i32,
    bullet_start_offset: f32,
    bullet_speed: f32,
    bullet_despawn_after_secs: f32,
}

impl BulletController {
    pub fn new(fire_rate: f32, bullet_start_offset: f32, bullet_speed: f32, bullet_despawn_after_secs: f32) -> Self {
        Self {
            state: BulletControllerState::None,
            fire_count: 0,
            timer: Timer::from_seconds(1.0 / fire_rate, true),
            bullet_start_offset,
            bullet_speed,
            bullet_despawn_after_secs,
        }
    }

    pub fn try_set_firing_state(&mut self, firing: bool) {
        if firing && self.state == BulletControllerState::None {
            // Start firing
            self.state = BulletControllerState::Firing;
            self.fire_count = 0;
            self.timer.reset();
        }
        else if !firing && self.state == BulletControllerState::Firing {
            // Start cooling down
            self.state = BulletControllerState::Cooldown;
        }
    }

    fn update(&mut self, time: &Time) -> UpdateResult {
        self.timer.tick(time.delta());
        match self.state {
            BulletControllerState::None => UpdateResult::None,
            BulletControllerState::Firing => {
                // Fire immediately for the first bullet, and therafter
                // on the cadence set by the timer
                let should_fire = self.fire_count == 0 || self.timer.just_finished();
                if should_fire {
                    self.fire_count += 1;
                    UpdateResult::FireBullet
                }
                else {
                    UpdateResult::None
                }
            },
            BulletControllerState::Cooldown => {
                // Cooldown completes after one "firing period" as completed
                // This prevents button mashing from firing faster than the configured fire rate
                let cooldown_complete = self.timer.just_finished();
                if cooldown_complete {
                    self.state = BulletControllerState::None;
                }
                UpdateResult::None
            },
        }
    }
}

fn bullet_controller_system(
    time: Res<Time>,
    assets: Res<GameAssets>,
    mut commands: Commands,
    mut query: Query<(&Movable, &mut BulletController)>
) {
    for (movable, mut controller) in query.iter_mut() {
        if controller.update(&time) == UpdateResult::FireBullet {
            spawn_bullet(&mut commands, &assets.bullet_assets, BulletSpawn {
                position: movable.position + movable.heading_normal() * controller.bullet_start_offset,
                velocity: movable.velocity + movable.heading_normal() * controller.bullet_speed,
                heading_angle: movable.heading_angle,
                despawn_after_secs: controller.bullet_despawn_after_secs,
            });
        }
    }
}

// Collision detection

fn  bullet_collision_system(
    bullets: Query<(Entity, &Collidable), With<Bullet>>,
    collidables: Query<(Entity, &Collidable), With<BulletCollidable>>,
    mut hit_events: EventWriter<HitEvent>
)
{
    for (bullet, b_collidable) in bullets.iter() {
        for (other, o_collidable) in collidables.iter() {
            if b_collidable.test_collision_with(&o_collidable) {
                // Collision!
                hit_events.send(HitEvent(bullet));
                hit_events.send(HitEvent(other));
            }
        }
    }
}