use bevy::prelude::*;

#[derive(Component)]
pub struct GridPosition(pub IVec2);

#[derive(Component)]
pub struct Unit {
    pub initiative: f32,
    pub max_initiative: f32,
}

fn advance_unit_initiative(mut query: Query<&mut Unit>, time: Res<Time>) {
    for mut unit in &mut query {
        unit.initiative = (unit.initiative + time.delta_seconds()).clamp(0.0, unit.max_initiative);
    }
}

pub struct LogicPlugin;

impl Plugin for LogicPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(advance_unit_initiative);
    }
}
