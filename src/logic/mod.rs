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

pub enum UnitAction {
    Wait,
}

#[derive(Resource)]
pub struct UnitTurn {
    pub unit: Entity,
    pub start_position: IVec2,
    pub end_position: IVec2,
    pub action: UnitAction,
}

pub fn apply_turn(
    mut units: Query<(&mut GridPosition, &mut Unit)>,
    turn: Option<Res<UnitTurn>>,
    mut commands: Commands,
) {
    if let Some(turn) = turn {
        if let Ok((mut pos, mut unit)) = units.get_mut(turn.unit) {
            pos.0 = turn.end_position;
            unit.initiative = 0.0;
            commands.remove_resource::<UnitTurn>()
        }
    }
}

pub struct LogicPlugin;

impl Plugin for LogicPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(advance_unit_initiative);
        app.add_system(apply_turn);
    }
}
