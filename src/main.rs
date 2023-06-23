use bevy::{core_pipeline::clear_color::ClearColorConfig, prelude::*};
use bevy_ecs_ldtk::{LdtkWorldBundle, LevelSelection};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use cursor::CursorPlugin;
use logic::{GridPosition, LogicPlugin, Unit};

mod cursor;
mod logic;

const PROGRESS_BAR_WIDTH: f32 = 16.0;
const PROGRESS_BAR_HEIGHT: f32 = 4.0;

const GRID_SIZE: f32 = 16.0;
#[derive(Component, Default)]
struct ProgressBar {
    progress: f32,
}

#[derive(Component)]
struct InitiativeProgressBar;

fn add_unit(mut commands: Commands) {
    commands
        .spawn((
            Unit {
                initiative: 0.0,
                max_initiative: 10.0,
            },
            GridPosition(IVec2 { x: 0, y: 0 }),
            SpriteBundle {
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 2.0)),
                sprite: Sprite {
                    color: Color::rgb(0.5, 0.4, 0.3),
                    custom_size: Some(Vec2::new(GRID_SIZE, GRID_SIZE)),
                    ..Default::default()
                },
                ..Default::default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                InitiativeProgressBar,
                ProgressBar::default(),
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::rgb(0.2, 0.7, 0.5),
                        custom_size: Some(Vec2::new(PROGRESS_BAR_WIDTH, PROGRESS_BAR_HEIGHT)),
                        ..Default::default()
                    },
                    transform: Transform::from_translation(Vec3::new(0.0, -4.0, 1.0)),
                    ..Default::default()
                },
            ));
        });
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load("maps/levels.ldtk"),
        ..Default::default()
    });
    commands.spawn(Camera2dBundle {
        camera_2d: Camera2d {
            clear_color: ClearColorConfig::Custom(Color::rgb_u8(15, 10, 25)),
        },
        projection: OrthographicProjection {
            scale: 1.0 / 3.0,
            viewport_origin: Vec2::new(0.0, 0.0),
            ..Default::default()
        },
        ..Default::default()
    });
    add_unit(commands);
}

fn update_grid_transform(mut query: Query<(&GridPosition, &mut Transform)>) {
    for (grid_position, mut transform) in query.iter_mut() {
        transform.translation = Vec3::new(
            (grid_position.0.x as f32 + 0.5) * GRID_SIZE,
            (grid_position.0.y as f32 + 0.5) * GRID_SIZE,
            transform.translation.z,
        );
    }
}

fn update_initiative_progress_bar(
    q_parent: Query<(&Unit, &Children)>,
    mut q_child: Query<&mut ProgressBar, With<InitiativeProgressBar>>,
) {
    for (unit, children) in &q_parent {
        for child in children {
            if let Ok(mut bar) = q_child.get_mut(*child) {
                bar.progress = unit.initiative / unit.max_initiative;
            }
        }
    }
}

fn update_progress_bar_sprite(mut query: Query<(&ProgressBar, &mut Sprite, &mut Transform)>) {
    for (bar, mut sprite, mut transform) in query.iter_mut() {
        let width = PROGRESS_BAR_WIDTH * bar.progress;
        sprite.custom_size = sprite.custom_size.map(|size| Vec2::new(width, size.y));
        transform.translation.x = (PROGRESS_BAR_WIDTH - width) * -0.5;
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugin(bevy_ecs_ldtk::LdtkPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(CursorPlugin)
        .add_plugin(LogicPlugin)
        .add_startup_system(setup)
        .insert_resource(LevelSelection::Index(0))
        .add_system(update_grid_transform)
        .add_system(update_initiative_progress_bar)
        .add_system(update_progress_bar_sprite)
        .run();
}
