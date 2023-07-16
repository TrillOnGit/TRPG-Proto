use bevy::prelude::*;

use crate::{
    logic::{Unit, UnitStats},
    TRPGState,
};

const PROGRESS_BAR_WIDTH: f32 = 16.0;
const PROGRESS_BAR_HEIGHT: f32 = 4.0;

pub fn initiative_progress_bar_bundle() -> impl Bundle {
    (
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
    )
}

#[derive(Component, Default)]
struct ProgressBar {
    progress: f32,
}

#[derive(Component)]
struct InitiativeProgressBar;

fn update_initiative_progress_bar(
    q_parent: Query<(&Unit, &UnitStats, &Children)>,
    mut q_child: Query<&mut ProgressBar, With<InitiativeProgressBar>>,
) {
    for (unit, unit_stats, children) in &q_parent {
        for child in children {
            if let Ok(mut bar) = q_child.get_mut(*child) {
                bar.progress = unit.initiative / unit_stats.max_initiative;
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

pub struct ProgressBarPlugin;

impl Plugin for ProgressBarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            (update_initiative_progress_bar, update_progress_bar_sprite)
                .in_set(OnUpdate(TRPGState::Battle)),
        );
    }
}
