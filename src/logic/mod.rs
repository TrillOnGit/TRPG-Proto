use bevy::{ecs::system::SystemParam, prelude::*};

use bevy_ecs_tilemap::tiles::TilePos;

pub use self::reachable::*;
pub use self::tile::*;

mod reachable;
mod tile;

#[derive(Deref, Component, Reflect)]
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
pub struct UnitRange {
    pub valid_ranges: Vec<u32>,
}

#[derive(Component, Reflect)]
pub struct UnitSpeed(pub u32);

#[derive(Bundle)]
pub struct UnitLogicBundle {
    pub unit: Unit,
    pub unit_stats: UnitStats,
    pub unit_range: UnitRange,
    pub unit_speed: UnitSpeed,
    pub grid_position: GridPosition,
}

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

impl<'w, 's> ValidateMovementParam<'w, 's> {
    fn validate(&self, unit: Entity, start: IVec2, end: IVec2) -> bool {
        let Ok(pos) = self.units.get(unit) else {return false};
        if pos.0 != start {
            return false;
        }
        let Some(reachable_tiles) = self.reachable_tiles_param.get(unit) else {return false};
        if !reachable_tiles.contains(&TilePos::new(end.x as u32, end.y as u32)) {
            return false;
        }
        return true;
    }
}

#[derive(SystemParam)]
struct ValidateTurnParam<'w, 's> {
    units: Query<'w, 's, (&'static Unit, &'static UnitStats)>,
    validate_movement_param: ValidateMovementParam<'w, 's>,
}

impl<'w, 's> ValidateTurnParam<'w, 's> {
    fn validate(&self, turn: &UnitTurn) -> Option<ValidatedTurn> {
        let (unit, unit_stats) = self.units.get(turn.unit).ok()?;
        if unit.initiative != unit_stats.max_initiative {
            return None;
        }
        if !self
            .validate_movement_param
            .validate(turn.unit, turn.start_position, turn.end_position)
        {
            return None;
        }
        Some(ValidatedTurn(*turn))
    }
}

fn validate_turns(
    mut turns: EventReader<UnitTurn>,
    mut validated_turns: EventWriter<ValidatedTurn>,
    validate_turn_param: ValidateTurnParam,
) {
    validated_turns.send_batch(
        turns
            .iter()
            .filter_map(|turn| validate_turn_param.validate(turn)),
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

pub struct LogicPlugin;

impl Plugin for LogicPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(TilePlugin)
            .add_system(advance_unit_initiative)
            .add_system(validate_turns)
            .add_system(apply_valid_turns)
            .add_system(apply_valid_attacks)
            .add_event::<UnitTurn>()
            .add_event::<ValidatedTurn>()
            .register_type::<GridPosition>()
            .register_type::<Unit>()
            .register_type::<UnitStats>()
            .register_type::<UnitSpeed>()
            .register_type::<UnitRange>();
    }
}
