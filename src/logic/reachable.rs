use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use bevy_ecs_tilemap::helpers::square_grid::neighbors::Neighbors;

use std::collections::VecDeque;

use bevy_ecs_tilemap::tiles::TilePos;

use std::collections::HashSet;

use super::TileType;
use super::UnitSpeed;

use super::GridPosition;

use bevy_ecs_tilemap::tiles::TileStorage;

use super::LogicTile;

#[derive(SystemParam)]
pub(super) struct ReachableTilesParam<'w, 's> {
    pub(super) logical_tiles: Query<'w, 's, &'static LogicTile>,
    pub(super) tile_storage: Query<'w, 's, &'static TileStorage, With<TileType>>,
    pub(super) units: Query<'w, 's, (&'static GridPosition, &'static UnitSpeed)>,
}

pub(super) fn get_reachable_tiles(
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

        for neighbor_pos in neighbors.iter() {
            if reachable_tiles.contains(neighbor_pos) {
                continue;
            }
            let Some(neighbor_tile) = tile_storage.get(neighbor_pos).and_then(|e| logical_tiles.get(e).ok()) else { continue };
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
