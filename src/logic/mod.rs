use bevy::prelude::*;
use bevy_ecs_ldtk::{GridCoords, IntGridCell};

use crate::SelectedUnit;

#[derive(Component)]
pub struct GridPosition(pub IVec2);

#[derive(Component)]
pub struct Unit {
    pub initiative: f32,
    pub max_initiative: f32,
}

#[derive(Component)]
pub struct UnitSpeed(pub u32);

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

fn get_valid_spaces(
    mut tiles: Query<(&LogicalTile, &GridCoords, &mut ReachableInfo)>,
    units: Query<(&GridPosition, &UnitSpeed)>,
    selected: Res<SelectedUnit>,
) {
    if let Ok((GridPosition(pos), &UnitSpeed(speed))) = units.get(selected.0) {
        println!("Our pos: {}, {}", pos.x, pos.y);
        for (logical_tile, GridCoords { x, y }, mut reachable_info) in tiles.iter_mut() {
            reachable_info.reachable = ((pos.x - x).abs() + (pos.y - y).abs()) <= (speed as i32);
        }
    }
}

#[derive(Component)]
struct LogicalTile {
    can_move: bool,
    move_cost: u32,
}

#[derive(Component, Reflect)]
pub struct ReachableInfo {
    pub reachable: bool,
}

fn populate_logic_tiles(
    mut commands: Commands,
    tiles: Query<(Entity, &IntGridCell), Added<IntGridCell>>,
) {
    for (entity, cell) in tiles.iter() {
        commands.entity(entity).insert((
            LogicalTile {
                can_move: true,
                move_cost: 1,
            },
            ReachableInfo { reachable: false },
        ));
    }
}

pub struct LogicPlugin;

impl Plugin for LogicPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(advance_unit_initiative);
        app.add_system(apply_turn);
        app.add_system(populate_logic_tiles);
        app.add_system(get_valid_spaces);
        app.register_type::<ReachableInfo>();
    }
}
