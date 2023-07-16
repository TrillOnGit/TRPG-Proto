use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_ecs_ldtk::IntGridCell;
use bevy_ecs_tilemap::tiles::{TilePos, TileStorage};

use crate::SelectedUnit;

use super::{get_attackable_tiles, reachable, UnitRange};

fn mark_reachable_tiles(
    reachable_tiles_param: reachable::ReachableTilesParam,
    mut reachable_info: Query<(&TilePos, &mut ReachableInfo)>,
    unit_ranges: Query<&UnitRange>,
    selected: Res<SelectedUnit>,
) {
    let reachable_tiles = reachable_tiles_param.get(selected.0).unwrap_or_default();
    let attack_movable_tiles = unit_ranges
        .get(selected.0)
        .map(|unit_range| get_attackable_tiles(&reachable_tiles, &unit_range.valid_ranges))
        .unwrap_or_default();
    for (tile_pos, mut reachable_info) in reachable_info.iter_mut() {
        let reachable = reachable_tiles.contains(tile_pos);
        let attack_movable = attack_movable_tiles.contains(tile_pos);
        if reachable_info.reachable != reachable {
            reachable_info.reachable = reachable;
        }
        if reachable_info.attack_movable != attack_movable {
            reachable_info.attack_movable = attack_movable;
        }
    }
}

#[derive(Component, Default, Reflect)]
pub(super) struct LogicTile {
    pub(super) can_move: bool,
    pub(super) move_cost: u32,
}

#[derive(Component, Default, Reflect)]
pub struct ReachableInfo {
    pub reachable: bool,
    pub attack_movable: bool,
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

#[derive(SystemParam)]
pub struct GetTileStorageParam<'w, 's> {
    tile_storages: Query<'w, 's, &'static TileStorage, With<TileType>>,
}

impl<'w, 's> GetTileStorageParam<'w, 's> {
    pub fn get(&self) -> Option<&TileStorage> {
        self.tile_storages.get_single().ok()
    }
}

pub struct TilePlugin;

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(mark_tile_type_storage)
            .add_system(populate_logic_tiles)
            .add_system(mark_reachable_tiles)
            .register_type::<LogicTile>()
            .register_type::<ReachableInfo>();
    }
}
