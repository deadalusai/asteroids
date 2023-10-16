use bevy::prelude::*;
use super::manager::WorldBoundaries;
use super::FrameStage;

// Components for entities which are moving (basically everything)

#[derive(Resource)]
pub struct MovableGlobalState {
    pub enabled: bool,
}

pub struct MovablePlugin;

impl Plugin for MovablePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MovableGlobalState { enabled: true });
        app.add_systems(
            Update,
            (
                movable_system
                    .in_set(FrameStage::Movement)
                    .after(FrameStage::Input),
                movable_torus_constraint_system
                    .in_set(FrameStage::Movement)
                    .after(FrameStage::Start),
                movable_update_transform_system
                    .in_set(FrameStage::Movement)
                    .after(movable_torus_constraint_system),
            )
            .distributive_run_if(|s: Res<MovableGlobalState>| s.enabled)
        );
    }
}

#[derive(Debug)]
pub enum AcceleratingTo {
    /// No acceleration limit
    Infinite,
    /// Accelerating to Zero (decelerating)
    Zero,
    /// Accelerating to Max (NOTE: encodes an _absolute_ velocity)
    Max(f32)
}

#[derive(Debug)]
pub struct Acceleration<TAcc> {
    pub value: TAcc,
    pub limit: AcceleratingTo,
}

impl<T> Acceleration<T> {
    pub fn new(value: T) -> Self {
        Self { value, limit: AcceleratingTo::Infinite }
    }

    pub fn with_limit(self, limit: AcceleratingTo) -> Self {
        Self { limit, ..self }
    }
}

#[derive(Component, Debug)]
pub struct Movable {
    /// current x,y position in the frame of reference
    pub position: Vec2,
    /// velocity (vector - the movement direction + speed)
    pub velocity: Vec2,
    /// current acceleration (per second per second - a vector representing the current acceleration)
    pub acceleration: Option<Acceleration<Vec2>>,
    /// heading angle in rads (the direction the entity is facing)
    /// 0 = East, PI/2 = North, PI = West, 3(PI/2) = South
    pub heading_angle: f32,
    /// rotational velocity (rads/sec - the speed with which the entity is rotating)
    pub rotational_velocity: f32,
    /// current rotation acceleration (rad/sec - the rate of change of the rotation)
    pub rotational_acceleration: Option<Acceleration<f32>>,
}

impl Movable {
    pub fn heading_normal(&self) -> Vec2 {
        Vec2::from_angle(self.heading_angle)
    }

    fn is_moving_down(&self) -> bool {
        self.velocity.y < 0.
    }

    fn is_moving_up(&self) -> bool {
        self.velocity.y > 0.
    }

    fn is_moving_left(&self) -> bool {
        self.velocity.x < 0.
    }

    fn is_moving_right(&self) -> bool {
        self.velocity.x > 0.
    }
}

fn movable_system(
    time: Res<Time>,
    mut query: Query<&mut Movable>
) {
    use AcceleratingTo::*;
    use std::f32::consts::TAU;

    // Update the position of each moving object
    let t_secs = time.delta_seconds_f64() as f32;
    for mut movable in query.iter_mut() {

        // Update velocity
        if let Some(acc) = &movable.acceleration {
            let new_v = movable.velocity + acc.value * t_secs;
            movable.velocity = match acc.limit {
                Infinite => new_v,
                // Apply "accelerating to n" limit
                Max(max) => {
                    let overspeed = new_v.length() - max;
                    if overspeed > 0. {
                        // Subtract the overspeed
                        new_v - new_v.normalize() * overspeed
                    }
                    else {
                        new_v
                    }
                },
                // Apply "decelerating to zero" limit
                Zero => {
                    // Determine if the new vector lies on the line between ZERO and the original vector.
                    // Convert both vectors to the unit and compare them - if they are (close to) the same, then both vectors are 
                    // on the same side of zero. If they differ, then the second vector is on the other side of zero (i.e. has gone past it)
                    if new_v.normalize().abs_diff_eq(movable.velocity.normalize(), 0.1) {
                        // Same side of zero - we're still decelerating
                        new_v
                    }
                    else {
                        // Other side of zero - we've blown past it
                        Vec2::ZERO
                    }
                },
            };
        }

        // Update rotational velocity
        if let Some(acc) = &movable.rotational_acceleration {
            let new_v = movable.rotational_velocity + acc.value * t_secs;
            movable.rotational_velocity = match acc.limit {
                // No limit
                Infinite => new_v,
                // Apply "accelerating to n" limit
                Max(max) => new_v.clamp(-max, max),
                // Apply "decelerating to zero" limit
                Zero =>
                    if movable.rotational_velocity > 0. && new_v < 0. || movable.rotational_velocity < 0. && new_v > 0. { 0. }
                    else { new_v }
            };
        }

        // Update heading
        let angle_delta = movable.rotational_velocity * t_secs;
        movable.heading_angle += angle_delta;
        movable.heading_angle %= TAU;

        // Update position
        let position_delta = movable.velocity * t_secs;
        movable.position += position_delta;
    }
}

fn movable_update_transform_system(mut query: Query<(&Movable, &mut Transform)>) {
    // Update the translation of each moving object which has one
    for (movable, mut transform) in query.iter_mut() {
        transform.translation.x = movable.position.x;
        transform.translation.y = movable.position.y;
        transform.rotation = Quat::from_rotation_z(movable.heading_angle);
    }
}

// Torus world

#[derive(Component)]
pub struct MovableTorusConstraint {
    /// Radius of the circle used to determine if the entity has "left" the screen
    pub radius: f32,
}

fn movable_torus_constraint_system(
    world_boundaries: Res<WorldBoundaries>,
    mut query: Query<(&MovableTorusConstraint, &mut Movable)>
) {
    // NOTE: 0,0 is in the middle of the window.
    for (torus, mut movable) in query.iter_mut() {
        let right = world_boundaries.right + torus.radius;
        let left = world_boundaries.left - torus.radius;
        let top = world_boundaries.top + torus.radius;
        let bottom = world_boundaries.bottom - torus.radius;
        // Is this Movable leaving the screen?
        // Teleport them to the other side of the Torus
        if movable.position.x > right && movable.is_moving_right() {
            movable.position.x = left;
        }
        if movable.position.x < left && movable.is_moving_left() {
            movable.position.x = right;
        }
        if movable.position.y > top && movable.is_moving_up() {
            movable.position.y = bottom;
        }
        if movable.position.y < bottom && movable.is_moving_down() {
            movable.position.y = top;
        }
    }
}