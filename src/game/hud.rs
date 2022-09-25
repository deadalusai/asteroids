use bevy::prelude::*;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, Diagnostics};

use crate::AppState;
use super::manager::GameManager;

// Plugin

pub struct HeadsUpDisplayPlugin;

impl Plugin for HeadsUpDisplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(FrameTimeDiagnosticsPlugin::default());
        app.add_system_set(
            SystemSet::on_enter(AppState::Game)
                .with_system(setup_system)
        );
        app.add_system_set(
            SystemSet::on_update(AppState::Game)
                .with_system(status_text_update_system)
                .with_system(debug_text_update_system)
        );
        app.add_system_set(
            SystemSet::on_exit(AppState::Game)
                .with_system(destroy_system)
        );
    }
}

// HUD

#[derive(Component)]
struct HudPart;

#[derive(Component)]
struct StatusText;

#[derive(Component)]
struct DebugText;

fn setup_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    
    let font_light = asset_server.load(crate::asset_paths::FONT_MONO_LIGHT);

    let status_text_bundle =
        TextBundle::from_sections([
            TextSection::new(
                "POINTS: ",
                TextStyle {
                    font: font_light.clone(),
                    font_size: 30.0,
                    color: Color::WHITE,
                },
            ),
            TextSection::from_style(TextStyle {
                font: font_light.clone(),
                font_size: 30.0,
                color: Color::GOLD,
            }),
            TextSection::new(
                " LIVES: ",
                TextStyle {
                    font: font_light.clone(),
                    font_size: 30.0,
                    color: Color::WHITE,
                },
            ),
            TextSection::from_style(TextStyle {
                font: font_light.clone(),
                font_size: 30.0,
                color: Color::GOLD,
            }),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                top: Val::Px(15.0),
                left: Val::Px(15.0),
                ..default()
            },
            justify_content: JustifyContent::FlexStart,
            ..default()
        });

    commands
        .spawn()
        .insert(StatusText)
        .insert(HudPart)
        .insert_bundle(status_text_bundle);

    let debug_text_bundle =
        TextBundle::from_sections([
            TextSection::new(
                "",
                TextStyle {
                    font: font_light.clone(),
                    font_size: 15.0,
                    color: Color::WHITE,
                },
            ),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                bottom: Val::Px(15.0),
                right: Val::Px(15.0),
                ..default()
            },
            justify_content: JustifyContent::FlexEnd,
            ..default()
        });

    commands
        .spawn()
        .insert(DebugText)
        .insert(HudPart)
        .insert_bundle(debug_text_bundle);
}

fn destroy_system(mut commands: Commands, query: Query<Entity, With<HudPart>>) {
    for entity in query.iter() {
        commands
            .entity(entity)
            .despawn_recursive();
    }
}

fn status_text_update_system(
    game: Res<GameManager>,
    mut status_text: Query<&mut Text, With<StatusText>>
) {
    let mut status_text = status_text.get_single_mut().unwrap();
    status_text.sections[1].value = format!("{}", game.player_points);
    status_text.sections[3].value = format!("{}", game.player_lives_remaining);
}

fn debug_text_update_system(
    game: Res<GameManager>,
    diag: Res<Diagnostics>,
    mut debug_text: Query<&mut Text, With<DebugText>>
) {
    let mut debug_text = debug_text.get_single_mut().unwrap();
    let debug_s = &mut debug_text.sections[0].value;
    debug_s.clear();
    write_debug_info_text(&diag, &game, debug_s).unwrap();
}

fn write_debug_info_text(
    diag: &Diagnostics,
    game: &GameManager,
    w: &mut impl std::fmt::Write
) -> Result<(), std::fmt::Error> {
    // FPS
    if let Some(fps) = diag.get(FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(avg) = fps.average() {
            writeln!(w, "fps: {avg:.2}")?;
        }
    };
    // Game state
    writeln!(w, "asteroids on screen: {}", game.debug_asteroid_count_on_screen)?;
    writeln!(w, "asteroids pending spawn: {}", game.scheduled_asteroid_spawns.len())?;
    writeln!(w, "player state: {:?}", game.player_state)?;
    writeln!(w, "alien state: {:?}", game.alien_state)?;
    Ok(())
}