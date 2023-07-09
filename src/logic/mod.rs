use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_ecs_ldtk::IntGridCell;
use bevy_ecs_tilemap::tiles::{TilePos, TileStorage};

use crate::SelectedUnit;

use self::reachable::get_reachable_tiles;

mod reachable;

#[derive(Component, Reflect)]
pub struct GridPosition(pub IVec2);

#[derive(Component, Reflect)]
pub struct Unit {
    pub initiative: f32,
    pub current_hp: u32,
}

#[derive(Component, Reflect)]
pub struct UnitStats {
    pub max_hp: u32,
    pub max_initiative: f32,
    pub base_atk: u32,
    pub base_armor: u32,
}

#[derive(Component, Reflect)]
pub struct UnitSpeed(pub u32);

fn advance_unit_initiative(mut query: Query<(&mut Unit, &UnitStats)>, time: Res<Time>) {
    for (mut unit, unit_stats) in &mut query {
        unit.initiative =
            (unit.initiative + time.delta_seconds()).clamp(0.0, unit_stats.max_initiative);
    }
}

#[derive(Clone, Copy)]
pub enum UnitAction {
    Wait,
    Attack { target: Entity },
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
    reachable_tiles_param: reachable::ReachableTilesParam<'w, 's>,
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
    units: Query<'w, 's, (&'static Unit, &'static UnitStats)>,
    validate_movement_param: ValidateMovementParam<'w, 's>,
}

fn validate_turn(
    turn: &UnitTurn,
    ValidateTurnParam {
        units,
        validate_movement_param,
    }: &ValidateTurnParam,
) -> Option<ValidatedTurn> {
    let (unit, unit_stats) = units.get(turn.unit).ok()?;
    if unit.initiative != unit_stats.max_initiative {
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

fn apply_valid_attacks(
    mut units: Query<&mut Unit>,
    unit_stats: Query<&UnitStats>,
    mut turns: EventReader<ValidatedTurn>,
) {
    for turn in turns.iter() {
        match turn.action {
            UnitAction::Wait => {}
            UnitAction::Attack { target } => {
                let Ok(mut target_unit) = units.get_mut(target) else { continue };
                let Ok(unit_stats) = unit_stats.get(turn.unit) else { continue };
                target_unit.current_hp = target_unit.current_hp.saturating_sub(unit_stats.base_atk);
            }
        }
    }
}

fn mark_reachable_tiles(
    reachable_tiles_param: reachable::ReachableTilesParam,
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

#[derive(Component, Default, Reflect)]
struct LogicTile {
    can_move: bool,
    move_cost: u32,
}

#[derive(Component, Default, Reflect)]
pub struct ReachableInfo {
    pub reachable: bool,
}

#[derive(Component, Default, Reflect)]
pub struct AttackableInfo {
    pub attackable: bool,
}

#[derive(Component)]
struct TileType;

fn mark_tile_type_storage(
    mut commands: Commands,
    tile_storages: Query<(Entity, &Name), Added<TileStorage>>,
) {
    for (tile_storage, name) in tile_storages.iter() {
        if name.as_str() == "TileType" {
            commands.entity(tile_storage).insert(TileType);
        }
    }
}

#[derive(Bundle, Default)]
struct TileExtraBundle {
    pub logic_tile: LogicTile,
    pub reachable_info: ReachableInfo,
    pub attackable_info: AttackableInfo,
}

fn populate_logic_tiles(
    mut commands: Commands,
    tiles: Query<(Entity, &IntGridCell), Added<IntGridCell>>,
    other_tiles: Query<Entity, (Without<IntGridCell>, Without<LogicTile>)>,
    tile_maps: Query<&TileStorage, With<TileType>>,
) {
    for (entity, &IntGridCell { value }) in tiles.iter() {
        commands.entity(entity).insert(TileExtraBundle {
            logic_tile: match value {
                2 => LogicTile {
                    can_move: true,
                    move_cost: 2,
                },
                _ => LogicTile {
                    can_move: false,
                    move_cost: 0,
                },
            },
            ..Default::default()
        });
    }
    for tile_storage in tile_maps.iter() {
        for &tile in tile_storage.iter().flatten() {
            if !other_tiles.contains(tile) {
                continue;
            }
            commands.entity(tile).insert(TileExtraBundle {
                logic_tile: LogicTile {
                    can_move: true,
                    move_cost: 1,
                },
                ..Default::default()
            });
        }
    }
}

pub struct LogicPlugin;

impl Plugin for LogicPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(advance_unit_initiative)
            .add_system(validate_turns)
            .add_system(apply_valid_turns)
            .add_system(apply_valid_attacks)
            .add_system(mark_tile_type_storage)
            .add_system(populate_logic_tiles)
            .add_system(mark_reachable_tiles)
            .add_event::<UnitTurn>()
            .add_event::<ValidatedTurn>()
            .register_type::<LogicTile>()
            .register_type::<ReachableInfo>()
            .register_type::<GridPosition>()
            .register_type::<Unit>()
            .register_type::<UnitStats>()
            .register_type::<UnitSpeed>();
    }
}
