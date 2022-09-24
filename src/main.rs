#![feature(drain_filter)]

mod game;

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    Menu,
    Game,
}

const ASTEROIDS_TITLE: &str = "Asteroids";

fn main() {
    let title = ASTEROIDS_TITLE.into();
    let (width, height) = (1600., 1200.);
    App::new()
        // bevy
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .insert_resource(WindowDescriptor {
            title,
            width,
            height,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        // state management
        .add_state(AppState::Menu)
        .add_system_set(SystemSet::on_enter(AppState::Menu).with_system(setup_menu_system))
        .add_system_set(SystemSet::on_update(AppState::Menu).with_system(update_menu_system))
        .add_system_set(SystemSet::on_exit(AppState::Menu).with_system(destroy_menu_system))
        // bevy_prototype_lyon
        .insert_resource(Msaa { samples: 4 })
        .add_plugin(ShapePlugin)
        .add_plugins(game::GamePluginGroup)
        .add_startup_system(startup_system)
        .run();
}

// Startup

const FIXED_VIEWPORT_WIDTH: f32 = 256.0;

fn startup_system(mut commands: Commands) {
    // Spawn a camera
    // NOTE: Our graphics are small! Tune the projection to keep the size of the "world" known
    // despite window scale
    commands.spawn_bundle(Camera2dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 999.9),
        projection: bevy::render::camera::OrthographicProjection {
            far: 1000.0,
            depth_calculation: bevy::render::camera::DepthCalculation::ZDifference,
            scaling_mode: bevy::render::camera::ScalingMode::FixedHorizontal(FIXED_VIEWPORT_WIDTH),
            ..default()
        },
        ..default()
    });
}

// Menu

struct MenuData {
    button_entity: Entity,
}

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

fn setup_menu_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    let button_entity = commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                margin: UiRect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            color: NORMAL_BUTTON.into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle::from_section(
                "Play",
                TextStyle {
                    font: asset_server.load("fonts/RedHatMono-Light.ttf"),
                    font_size: 40.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                },
            ));
        })
        .id();
    commands.insert_resource(MenuData { button_entity });
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
                *color = PRESSED_BUTTON.into();
                state.set(AppState::Game).unwrap();
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

fn destroy_menu_system(mut commands: Commands, menu_data: Res<MenuData>) {
    commands
        .entity(menu_data.button_entity)
        .despawn_recursive();
}