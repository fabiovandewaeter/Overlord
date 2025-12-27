use std::collections::HashSet;

use bevy::{prelude::*, sprite_render::TilemapChunk};
use serde::{Deserialize, Serialize};

use crate::{
    map::{
        CurrentMapId, MapId, MultiMapManager, StructureLayerManager,
        coordinates::{GridPosition, TileCoordinates, tile_coord_to_absolute_coord},
    },
    time::GameTime,
    units::Unit,
};

#[derive(Component)]
pub struct Passable;

#[derive(Component, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SpeedStat(pub f32);
impl SpeedStat {
    pub fn from_tiles_per_second(tiles_per_second: f32) -> Self {
        // to move 1 tile per tick
        let cost_per_tile = MovementAccumulator::MOVEMENT_COST;
        let ticks_per_second = GameTime::UPS_TARGET as f32;
        let speed_stat_per_tick = (tiles_per_second * cost_per_tile) / ticks_per_second;
        Self(speed_stat_per_tick)
    }
}
impl Default for SpeedStat {
    fn default() -> Self {
        // Self(Unit::DEFAULT_MOVEMENT_SPEED)
        Self::from_tiles_per_second(Unit::DEFAULT_TILE_PER_SECOND_SPEED)
    }
}

/// add SpeedStat every tick until reached MOVEMENT_COST, then unit can move one time
#[derive(Component, Debug)]
pub struct MovementAccumulator(pub f32);
impl MovementAccumulator {
    pub const MOVEMENT_COST: f32 = 100.0;
}
impl Default for MovementAccumulator {
    fn default() -> Self {
        Self(Self::MOVEMENT_COST)
    }
}

pub fn update_units_movement_accumulators_system(
    mut unit_query: Query<(&mut MovementAccumulator, &SpeedStat), With<Unit>>,
) {
    for (mut movement_accumulator, speed_stat) in unit_query.iter_mut() {
        if movement_accumulator.0 < MovementAccumulator::MOVEMENT_COST {
            movement_accumulator.0 += speed_stat.0;
        }
    }
}

#[derive(Component)]
pub struct DesiredMovement {
    pub tile: Option<TileCoordinates>,
    pub map_id: Option<MapId>,
}
impl DesiredMovement {
    pub fn new(tile: TileCoordinates, map_id: MapId) -> Self {
        Self {
            tile: Some(tile),
            map_id: Some(map_id),
        }
    }
}
impl Default for DesiredMovement {
    fn default() -> Self {
        Self {
            tile: None,
            map_id: None,
        }
    }
}

pub fn apply_desired_movement_system(
    mut query: Query<
        (
            &mut GridPosition,
            &mut CurrentMapId,
            &mut MovementAccumulator,
            &mut DesiredMovement,
        ),
        With<Unit>,
    >,
    chunk_query: Query<&StructureLayerManager, With<TilemapChunk>>,
    multi_map_manager: Res<MultiMapManager>,
) {
    let mut occupied: HashSet<(TileCoordinates, MapId)> = HashSet::new();
    for (grid_pos, map_id, _, _) in query.iter() {
        occupied.insert((grid_pos.0, map_id.0));
    }

    for (mut grid_pos, mut current_map_id, mut movement_accumulator, mut desired_movement) in
        query.iter_mut()
    {
        let Some(target_tile) = desired_movement.tile else {
            continue;
        };
        let Some(target_map_id) = desired_movement.map_id else {
            panic!()
        };

        if movement_accumulator.0 < MovementAccumulator::MOVEMENT_COST {
            desired_movement.tile = None;
            continue;
        }

        let map_manager = multi_map_manager.maps.get(&current_map_id.0).unwrap();

        let is_tile_occupied_by_unit = occupied.contains(&(target_tile, target_map_id));

        if map_manager.is_tile_walkable(target_tile, &chunk_query) && !is_tile_occupied_by_unit {
            occupied.remove(&(grid_pos.0, current_map_id.0));

            grid_pos.0 = target_tile;
            current_map_id.0 = target_map_id;

            movement_accumulator.0 -= MovementAccumulator::MOVEMENT_COST;

            desired_movement.tile = None;
            desired_movement.map_id = None;

            occupied.insert((target_tile, target_map_id));
        }
    }
}

// TODO: move that elsewhere
pub fn sync_grid_pos_to_transform(
    mut query: Query<(&GridPosition, &mut Transform)>,
    time: Res<Time>,
) {
    for (grid_pos, mut transform) in query.iter_mut() {
        let target = tile_coord_to_absolute_coord(grid_pos.0);
        let target_vec3 = Vec3::new(target.x, target.y, transform.translation.z);

        // Lerp rapide (15.0 de speed) pour glisser vers la case
        transform.translation = transform
            .translation
            .lerp(target_vec3, time.delta_secs() * 15.0);
    }
}
