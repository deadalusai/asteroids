use bevy::diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

// Plugin

struct HeadsUpDisplayPlugin;

impl Plugin for HeadsUpDisplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(FrameTimeDiagnosticsPlugin::default());
        app.add_startup_system(setup_system);
        app.add_system(text_update_system);
        app.add_system(text_color_system);
    }
}

// HUD

#[derive(Component)]
struct FpsText;

#[derive(Component)]
struct ColorText;

fn setup_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    
    // Create a TextBundle that has a Text with a single section.
    let color_text_bundle =
        TextBundle::from_section(
            "hello\nbevy!",
            TextStyle {
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: 100.0,
                color: Color::WHITE,
            },
        )
        .with_text_alignment(TextAlignment::TOP_CENTER)
        .with_style(Style {
            align_self: AlignSelf::FlexEnd,
            position_type: PositionType::Absolute,
            position: UiRect {
                bottom: Val::Px(5.0),
                right: Val::Px(15.0),
                ..default()
            },
            ..default()
        });

    commands
        .spawn()
        .insert(ColorText)
        .insert_bundle(color_text_bundle);

    let fps_text_bundle =
        TextBundle::from_sections([
            TextSection::new(
                "FPS: ",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 60.0,
                    color: Color::WHITE,
                },
            ),
            TextSection::from_style(TextStyle {
                font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                font_size: 60.0,
                color: Color::GOLD,
            }),
        ])
        .with_style(Style {
            align_self: AlignSelf::FlexEnd,
            ..default()
        });
    
    commands
        .spawn()
        .insert(FpsText)
        .insert_bundle(fps_text_bundle);
}

fn text_color_system(time: Res<Time>, mut query: Query<&mut Text, With<ColorText>>) {
    for mut text in &mut query {
        let seconds = time.seconds_since_startup() as f32;

        // Update the color of the first and only section.
        text.sections[0].style.color = Color::Rgba {
            red: (1.25 * seconds).sin() / 2.0 + 0.5,
            green: (0.75 * seconds).sin() / 2.0 + 0.5,
            blue: (0.50 * seconds).sin() / 2.0 + 0.5,
            alpha: 1.0,
        };
    }
}

fn text_update_system(diagnostics: Res<Diagnostics>, mut query: Query<&mut Text, With<FpsText>>) {
    for mut text in &mut query {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(average) = fps.average() {
                // Update the value of the second section
                text.sections[1].value = format!("{average:.2}");
            }
        }
    }
}