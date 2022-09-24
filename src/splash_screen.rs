use bevy::prelude::*;

use crate::AppState;

// Plugins

pub struct SplashScreenPlugin;

impl Plugin for SplashScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(AppState::Menu)
                .with_system(setup_menu_system)
        );
        app.add_system_set(
            SystemSet::on_exit(AppState::Menu)
                .with_system(destroy_menu_system)
        );
        app.add_system_set(
            SystemSet::on_update(AppState::Menu)
                .with_system(update_menu_system)
        );
    }
}

// Components

#[derive(Component)]
struct MenuRoot;

// Menu

const COLOR_NORMAL: Color = Color::rgb(0.15, 0.15, 0.15);
const COLOR_HOVERED: Color = Color::rgb(0.25, 0.25, 0.25);
const COLOR_PRESSED: Color = Color::rgb(0.35, 0.75, 0.35);

fn setup_menu_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font_light = asset_server.load("fonts/RedHatMono-Light.ttf");
    let font_bold = asset_server.load("fonts/RedHatMono-Bold.ttf");

    let margin_style = Style {
        margin: UiRect::all(Val::Px(20.0)),
        ..default()
    };

    let title_text_style = TextStyle {
        font: font_bold,
        font_size: 120.0,
        color: Color::WHITE,
    };

    let button_text_style = TextStyle {
        font: font_light,
        font_size: 50.0,
        color: Color::GRAY,
    };

    // Root node
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                flex_direction: FlexDirection::ColumnReverse,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceEvenly,
                ..default()
            },
            color: Color::NONE.into(),
            ..default()
        })
        .insert(MenuRoot)
        .with_children(|parent| {

            // Title
            parent
                .spawn_bundle(
                    TextBundle::from_section("ASTEROIDS", title_text_style)
                        .with_style(margin_style.clone())
                );
                
            // Controls
            parent
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                        margin: margin_style.margin.clone(),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    color: COLOR_NORMAL.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn_bundle(
                            TextBundle::from_section("Play", button_text_style)
                                .with_style(margin_style.clone())
                        );
                });
        });
}

fn update_menu_system(
    mut state: ResMut<State<AppState>>,
    mut interaction_query: Query<
        (&Interaction, &mut UiColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                *color = COLOR_PRESSED.into();
                state.set(AppState::Game).unwrap();
            }
            Interaction::Hovered => {
                *color = COLOR_HOVERED.into();
            }
            Interaction::None => {
                *color = COLOR_NORMAL.into();
            }
        }
    }
}

fn destroy_menu_system(mut commands: Commands, fragments: Query<Entity, With<MenuRoot>>) {
    for entity in fragments.iter() {
        commands
            .entity(entity)
            .despawn_recursive();
    }
}