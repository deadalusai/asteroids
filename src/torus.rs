use bevy::prelude::*;
use crate::movable::Movable;

pub struct TorusConstraintPlugin;

impl Plugin for TorusConstraintPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::Update, torus_constraint_system);
    }
}

#[derive(Component)]
pub struct TorusConstraint(f32);

impl TorusConstraint {
    pub fn new(gutter_size: f32) -> Self {
        TorusConstraint(gutter_size)
    }
}

fn torus_constraint_system(
    windows: Res<Windows>,
    mut query: Query<(&TorusConstraint, &mut Movable)>
) {
    // NOTE: 0,0 is in the middle of the window.
    let half_width = windows.get_primary().unwrap().width() / 2.;
    let half_height = windows.get_primary().unwrap().height() / 2.;

    for (TorusConstraint(gutter_size), mut movable) in query.iter_mut() {
        // Has this Movable left the screen?
        // Teleport them to the other side of the Torus
        if movable.position.x > half_width + gutter_size || movable.position.x < -half_width - gutter_size {
            movable.position.x = -movable.position.x;
        }
        if movable.position.y > half_height + gutter_size || movable.position.y < -half_height - gutter_size {
            movable.position.y = -movable.position.y;
        }
    }
}