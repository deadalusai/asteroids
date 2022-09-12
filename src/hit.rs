use bevy::prelude::*;

// Generic messaging for "hits" to allow
// components to handle their own despawning requirements.
//
// Each entity should register a system which listens for these
// events, and respond when an entity they own appears.

pub struct HitEventsPlugin;

impl Plugin for HitEventsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<HitEvent>();
    }
}

pub struct HitEvent(pub Entity);

// Helpers

pub fn distinct_hit_events<'a>(events: &'a mut bevy::prelude::EventReader<crate::hit::HitEvent>) -> impl Iterator<Item=&'a crate::hit::HitEvent> {
    crate::util::distinct_by(events.iter(), |crate::hit::HitEvent(e)| e)
}