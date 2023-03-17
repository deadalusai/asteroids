use bevy::prelude::*;

use crate::AppState;

// Plugins

pub struct SplashScreenPlugin;

impl Plugin for SplashScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems((
            menu_setup_system
                .in_schedule(OnEnter(AppState::Menu)),
            menu_keyboard_system
                .in_set(OnUpdate(AppState::Menu)),
            menu_cleanup_system
                .in_schedule(OnExit(AppState::Menu))
        ));
    }
}

// Components

#[derive(Component)]
struct MenuRoot;

// Menu

fn menu_setup_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font_light = asset_server.load(crate::asset_paths::FONT_MONO_LIGHT);
    let font_bold = asset_server.load(crate::asset_paths::FONT_MONO_BOLD);

    let margin_style = Style {
        margin: UiRect::all(Val::Px(20.0)),
        ..default()
    };

    let title_text_style = TextStyle {
        font: font_bold,
        font_size: 90.0,
        color: Color::WHITE,
    };

    let secondary_text_style = TextStyle {
        font: font_light,
        font_size: 50.0,
        color: Color::GRAY,
    };

    // Root node
    commands
        .spawn((
            MenuRoot,
            NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                background_color: Color::NONE.into(),
                ..default()
            }
        ))
        .with_children(|parent| {
            parent.spawn(
                TextBundle::from_section("ASTEROIDS", title_text_style)
                .with_style(margin_style.clone())
            );
            parent.spawn(
                TextBundle::from_section("Press [space] to begin", secondary_text_style)
                .with_style(margin_style.clone())
            );
        });
}

fn menu_cleanup_system(mut commands: Commands, fragments: Query<Entity, With<MenuRoot>>) {
    for entity in fragments.iter() {
        commands
            .entity(entity)
            .despawn_recursive();
    }
}

fn menu_keyboard_system(
    mut kb: ResMut<Input<KeyCode>>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    if kb.clear_just_released(KeyCode::Space) {
        next_app_state.set(AppState::Game);
    }
}