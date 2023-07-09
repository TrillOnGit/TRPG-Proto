use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use bevy_ecs_tilemap::helpers::square_grid::neighbors::Neighbors;

use std::cmp::Reverse;
use std::collections::BinaryHeap;

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

struct TileExploreQueueItem {
    cost: u32,
    pos: TilePos,
}

impl TileExploreQueueItem {
    fn new(cost: u32, pos: TilePos) -> Self {
        Self { cost, pos }
    }
}

impl PartialEq for TileExploreQueueItem {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost
    }
}

impl Eq for TileExploreQueueItem {}

impl PartialOrd for TileExploreQueueItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Reverse(self.cost).partial_cmp(&Reverse(other.cost))
    }
}

impl Ord for TileExploreQueueItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        Reverse(self.cost).cmp(&Reverse(other.cost))
    }
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
    let mut tiles_to_explore = BinaryHeap::new();
    tiles_to_explore.push(TileExploreQueueItem::new(0, starting_pos));
    reachable_tiles.insert(starting_pos);

    while let Some(TileExploreQueueItem { cost, pos }) = tiles_to_explore.pop() {
        let neighbors =
            Neighbors::get_square_neighboring_positions(&pos, &tile_storage.size, false);

        for neighbor_pos in neighbors.iter() {
            if reachable_tiles.contains(neighbor_pos) {
                continue;
            }
            let Some(neighbor_tile) = tile_storage.get(neighbor_pos).and_then(|e| logical_tiles.get(e).ok()) else { continue };
            let neighbor_cost = cost + neighbor_tile.move_cost;
            if neighbor_cost <= speed && neighbor_tile.can_move {
                tiles_to_explore.push(TileExploreQueueItem::new(neighbor_cost, *neighbor_pos));
                reachable_tiles.insert(*neighbor_pos);
            }
        }
    }
    Some(reachable_tiles)
}

pub(super) fn get_attackable_tiles(
    reachable_tiles: &HashSet<TilePos>,
    ranges: &[u32],
) -> HashSet<TilePos> {
    let mut attackable_tiles = HashSet::new();
    for range in ranges.iter().cloned() {
        for reachable_tile in reachable_tiles.iter() {
            for offset in get_range_offsets(range) {
                if let Some(pos) = || -> Option<_> {
                    Some(TilePos::new(
                        reachable_tile.x.checked_add_signed(offset.x)?,
                        reachable_tile.y.checked_add_signed(offset.y)?,
                    ))
                }() {
                    attackable_tiles.insert(pos);
                }
            }
        }
    }
    attackable_tiles
}

fn get_range_offsets(range: u32) -> impl Iterator<Item = IVec2> {
    let mut x = -(range as i32);
    let mut y = 0;
    std::iter::from_fn(move || {
        if x > (range as i32) {
            return None;
        }
        let output = Some(IVec2::new(x, y));
        y *= -1;
        if y >= 0 {
            x += 1;
            if x <= 0 {
                y += 1;
            } else {
                y -= 1;
            }
        }
        output
    })
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use bevy::prelude::IVec2;

    use super::get_range_offsets;

    fn test_range_offsets(range: u32) {
        let range_offsets: HashSet<_> = get_range_offsets(range).collect();
        let ri = range as i32;

        let mut expected_offsets = HashSet::new();
        for x in -ri..(ri + 1) {
            for y in -ri..(ri + 1) {
                if x.abs() + y.abs() == ri {
                    expected_offsets.insert(IVec2::new(x, y));
                }
            }
        }
        assert_eq!(range_offsets, expected_offsets);
    }

    #[test]
    fn get_range_offsets_returns_correct_values() {
        for r in 0..5 {
            test_range_offsets(r);
        }
    }
}
