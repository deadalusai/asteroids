use bevy::prelude::*;

pub struct ViewportPlugin;

impl Plugin for ViewportPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Viewport { width: 0., height: 0. });
        app.add_system_to_stage(CoreStage::PreUpdate, viewport_update_system);
    }
}

// Viewport resource

pub struct Viewport {
    pub width: f32,
    pub height: f32,
}

fn viewport_update_system(
    mut resize_events: EventReader<bevy::window::WindowResized>,
    mut viewport: ResMut<Viewport>
) {
    for ev in resize_events.iter() {
        viewport.width = ev.width;
        viewport.height = ev.height;
    }
}