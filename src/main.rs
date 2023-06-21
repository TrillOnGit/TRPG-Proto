use bevy::prelude::*;

#[derive(Component)]
struct Unit {
    initiative: f32,
    max_initiative: f32,
}

#[derive(Component)]
struct GridPosition(IVec2);

const PROGRESS_BAR_WIDTH: f32 = 60.0;
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
                sprite: Sprite {
                    color: Color::rgb(0.5, 0.4, 0.3),
                    custom_size: Some(Vec2::new(50.0, 50.0)),
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
                        custom_size: Some(Vec2::new(PROGRESS_BAR_WIDTH, 10.0)),
                        ..Default::default()
                    },
                    transform: Transform::from_translation(Vec3::new(0.0, -30.0, 1.0)),
                    ..Default::default()
                },
            ));
        });
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    add_unit(commands);
}

fn advance_unit_initiative(mut query: Query<&mut Unit>, time: Res<Time>) {
    for mut unit in &mut query {
        unit.initiative = (unit.initiative + time.delta_seconds()).clamp(0.0, unit.max_initiative);
    }
}

fn update_grid_transform(mut query: Query<(&GridPosition, &mut Transform)>) {
    for (grid_position, mut transform) in query.iter_mut() {
        transform.translation = Vec3::new(
            (grid_position.0.x as f32) * 50.0,
            (grid_position.0.y as f32) * 50.0,
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
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(advance_unit_initiative)
        .add_system(update_grid_transform)
        .add_system(update_initiative_progress_bar)
        .add_system(update_progress_bar_sprite)
        .run();
}
