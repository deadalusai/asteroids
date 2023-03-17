use bevy::prelude::*;

use crate::AppState;
use super::FrameStage;
use super::movable::*;

// Component for entities which may collide (basically everything)

pub struct CollidablePlugin;

impl Plugin for CollidablePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            collidable_update_system
                .in_set(OnUpdate(AppState::Game))
                .after(FrameStage::Movement)
                .before(FrameStage::Collision)
        );
        
    }
}

#[derive(Component)]
pub struct Collidable {
    pub collider: Collider,
}

impl Collidable {
    pub fn test_collision_with(&self, other: &Collidable) -> bool {
        sepax2d::sat_overlap(self.shape(), other.shape())
    }

    fn shape(&self) -> &dyn sepax2d::Shape {
        match self.collider {
            Collider::Circle(ref circle) => circle,
            Collider::Capsule(ref capsule) => capsule,
        }
    }
}

fn collidable_update_system(mut query: Query<(&Movable, &mut Collidable)>) {
    // Update all movable colliders with their new positions
    for (movable, mut collidable) in query.iter_mut() {
        collidable.collider.set_position(movable.position);
    }
}

pub enum Collider {
    Circle(sepax2d::circle::Circle),
    Capsule(sepax2d::capsule::Capsule),
}

impl Collider {
    pub fn circle(position: Vec2, radius: f32) -> Collider {
        let circle = sepax2d::circle::Circle::new(position.into(), radius);
        Collider::Circle(circle)
    }
    
    pub fn capsule(position: Vec2, arm: Vec2, radius: f32) -> Collider {
        let capsule = sepax2d::capsule::Capsule::new(position.into(), arm.into(), radius);
        Collider::Capsule(capsule)
    }

    fn set_position(&mut self, position: Vec2) {
        let pos = match self {
            Collider::Circle(ref mut circle) => &mut circle.position,
            Collider::Capsule(ref mut capsule) => &mut capsule.position,
        };
        *pos = position.into();
    }
}
