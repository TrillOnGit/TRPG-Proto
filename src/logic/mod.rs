use std::collections::{HashSet, VecDeque};

use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_ecs_ldtk::IntGridCell;
use bevy_ecs_tilemap::{
    helpers::square_grid::neighbors::Neighbors,
    tiles::{TilePos, TileStorage},
};

use crate::SelectedUnit;

#[derive(Component, Reflect)]
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

#[derive(Clone, Copy)]
pub enum UnitAction {
    Wait,
}

#[derive(Clone, Copy)]
pub struct UnitTurn {
    pub unit: Entity,
    pub start_position: IVec2,
    pub end_position: IVec2,
    pub action: UnitAction,
}

#[derive(Clone, Copy, Deref)]
pub struct ValidatedTurn(UnitTurn);

#[derive(SystemParam)]
struct ValidateMovementParam<'w, 's> {
    units: Query<'w, 's, &'static GridPosition>,
    reachable_tiles_param: ReachableTilesParam<'w, 's>,
}

fn validate_movement(
    unit: Entity,
    start: IVec2,
    end: IVec2,
    ValidateMovementParam {
        units,
        reachable_tiles_param,
    }: &ValidateMovementParam,
) -> bool {
    let Ok(pos) = units.get(unit) else {return false};
    if pos.0 != start {
        return false;
    }
    let Some(reachable_tiles) = get_reachable_tiles(reachable_tiles_param, unit) else {return false};
    if !reachable_tiles.contains(&TilePos::new(end.x as u32, end.y as u32)) {
        return false;
    }
    return true;
}

#[derive(SystemParam)]
struct ValidateTurnParam<'w, 's> {
    units: Query<'w, 's, &'static Unit>,
    validate_movement_param: ValidateMovementParam<'w, 's>,
}

fn validate_turn(
    turn: &UnitTurn,
    ValidateTurnParam {
        units,
        validate_movement_param,
    }: &ValidateTurnParam,
) -> Option<ValidatedTurn> {
    let unit = units.get(turn.unit).ok()?;
    if unit.initiative != unit.max_initiative {
        return None;
    }
    if !validate_movement(
        turn.unit,
        turn.start_position,
        turn.end_position,
        validate_movement_param,
    ) {
        return None;
    }
    Some(ValidatedTurn(*turn))
}

fn validate_turns(
    mut turns: EventReader<UnitTurn>,
    mut validated_turns: EventWriter<ValidatedTurn>,
    validate_turn_param: ValidateTurnParam,
) {
    validated_turns.send_batch(
        turns
            .iter()
            .filter_map(|turn| validate_turn(turn, &validate_turn_param)),
    );
}

fn apply_valid_turns(
    mut units: Query<(&mut GridPosition, &mut Unit)>,
    mut turns: EventReader<ValidatedTurn>,
) {
    for turn in turns.iter() {
        let Ok((mut pos, mut unit)) = units.get_mut(turn.unit) else { continue };
        pos.0 = turn.end_position;
        unit.initiative = 0.0;
    }
}

#[derive(SystemParam)]
struct ReachableTilesParam<'w, 's> {
    logical_tiles: Query<'w, 's, &'static LogicTile>,
    tile_storage: Query<'w, 's, &'static TileStorage>,
    units: Query<'w, 's, (&'static GridPosition, &'static UnitSpeed)>,
}

fn get_reachable_tiles(
    ReachableTilesParam {
        logical_tiles,
        tile_storage,
        units,
    }: &ReachableTilesParam,
    unit: Entity,
) -> Option<HashSet<TilePos>> {
    let mut reachable_tiles = HashSet::new();
    let (GridPosition(pos), &UnitSpeed(speed)) = units.get(unit).ok()?;
    let tile_storage = tile_storage.get_single().ok()?;
    let starting_pos = TilePos {
        x: pos.x as u32,
        y: pos.y as u32,
    };
    let mut tiles_to_explore = VecDeque::new();
    tiles_to_explore.push_back((0, starting_pos));
    reachable_tiles.insert(starting_pos);

    while tiles_to_explore.len() > 0 {
        let (cost_so_far, checking_pos) = tiles_to_explore.pop_front().unwrap();

        let neighbors =
            Neighbors::get_square_neighboring_positions(&checking_pos, &tile_storage.size, false);

        for (neighbor_pos, neighbor_tile) in neighbors.iter().filter_map(|neighbor_pos| {
            Some((
                neighbor_pos,
                logical_tiles.get(tile_storage.get(neighbor_pos)?).ok()?,
            ))
        }) {
            let cost = cost_so_far + neighbor_tile.move_cost;
            if cost <= speed && neighbor_tile.can_move {
                tiles_to_explore.push_back((cost, *neighbor_pos));
                reachable_tiles.insert(*neighbor_pos);
            }
        }
        tiles_to_explore
            .make_contiguous()
            .sort_by_key(|(cost, _)| *cost);
    }
    Some(reachable_tiles)
}

fn mark_reachable_tiles(
    reachable_tiles_param: ReachableTilesParam,
    mut reachable_info: Query<(&TilePos, &mut ReachableInfo)>,
    selected: Res<SelectedUnit>,
) {
    let reachable_tiles = get_reachable_tiles(&reachable_tiles_param, selected.0);
    for (tile_pos, mut reachable_info) in reachable_info.iter_mut() {
        let reachable = reachable_tiles
            .as_ref()
            .map_or(false, |r| r.contains(tile_pos));
        if reachable_info.reachable != reachable {
            reachable_info.reachable = reachable;
        }
    }
}

#[derive(Component)]
struct LogicTile {
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
    for (entity, &IntGridCell { value }) in tiles.iter() {
        commands.entity(entity).insert((
            match value {
                2 => LogicTile {
                    can_move: true,
                    move_cost: 1,
                },
                _ => LogicTile {
                    can_move: false,
                    move_cost: 0,
                },
            },
            ReachableInfo { reachable: false },
        ));
    }
}

pub struct LogicPlugin;

impl Plugin for LogicPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(advance_unit_initiative)
            .add_system(validate_turns)
            .add_system(apply_valid_turns)
            .add_system(populate_logic_tiles)
            .add_system(mark_reachable_tiles)
            .add_event::<UnitTurn>()
            .add_event::<ValidatedTurn>()
            .register_type::<ReachableInfo>()
            .register_type::<GridPosition>();
    }
}
