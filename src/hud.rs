use bevy::prelude::*;

use crate::game::Game;

// Plugin

pub struct HeadsUpDisplayPlugin;

impl Plugin for HeadsUpDisplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_system);
        app.add_system(text_update_system);
        app.add_system(text_color_system);
    }
}

// HUD

#[derive(Component)]
struct StatusText;

#[derive(Component)]
struct ColorText;

fn setup_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    
    let font_dotgothic16 = asset_server.load("fonts/RedHatMono-Light.ttf");
    
    let color_text_bundle =
        TextBundle::from_section(
            "ASTEROIDS",
            TextStyle {
                font: font_dotgothic16.clone(),
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

    let status_text_bundle =
        TextBundle::from_sections([
            TextSection::new(
                "POINTS: ",
                TextStyle {
                    font: font_dotgothic16.clone(),
                    font_size: 60.0,
                    color: Color::WHITE,
                },
            ),
            TextSection::from_style(TextStyle {
                font: font_dotgothic16.clone(),
                font_size: 60.0,
                color: Color::GOLD,
            }),
            TextSection::new(
                " LIVES: ",
                TextStyle {
                    font: font_dotgothic16.clone(),
                    font_size: 60.0,
                    color: Color::WHITE,
                },
            ),
            TextSection::from_style(TextStyle {
                font: font_dotgothic16,
                font_size: 60.0,
                color: Color::GOLD,
            }),
        ])
        .with_style(Style {
            align_self: AlignSelf::FlexEnd,
            position: UiRect {
                top: Val::Px(5.0),
                left: Val::Px(15.0),
                ..default()
            },
            ..default()
        });
    
    commands
        .spawn()
        .insert(StatusText)
        .insert_bundle(status_text_bundle);
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

fn text_update_system(
    game: Res<Game>,
    mut query: Query<&mut Text, With<StatusText>>
) {
    for mut text in &mut query {
        text.sections[1].value = format!("{}", game.player_points);
        text.sections[3].value = format!("{}", game.player_lives_remaining);
    }
}