use std::collections::HashSet;

use bevy::{prelude::*, sprite_render::TilemapChunk};
use serde::{Deserialize, Serialize};

use crate::{
    map::{
        CurrentMapId, MapId, MultiMapManager, StructureLayerManager,
        coordinates::{GridPosition, TileCoordinates, tile_coord_to_absolute_coord},
        structure::Structure,
    },
    physics::collision_event::{Collision, CollisionHistory},
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
    mut unit_query: Query<
        (
            Entity,
            &mut GridPosition,
            &mut CurrentMapId,
            &mut MovementAccumulator,
            &mut DesiredMovement,
            &mut CollisionHistory,
        ),
        With<Unit>,
    >,
    structure_query: Query<Has<Passable>, With<Structure>>,
    chunk_query: Query<&StructureLayerManager, With<TilemapChunk>>,
    multi_map_manager: Res<MultiMapManager>,
    mut commands: Commands,
) {
    let mut occupied_by_unit: HashSet<(TileCoordinates, MapId)> = HashSet::new();
    for (_, grid_pos, map_id, _, _, _) in unit_query.iter() {
        occupied_by_unit.insert((grid_pos.0, map_id.0));
    }

    for (
        unit_entity,
        mut grid_pos,
        mut current_map_id,
        mut movement_accumulator,
        mut desired_movement,
        mut collision_history,
    ) in unit_query.iter_mut()
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

        let is_tile_occupied_by_unit = occupied_by_unit.contains(&(target_tile, target_map_id));

        let map_manager = multi_map_manager.maps.get(&current_map_id.0).unwrap();

        if map_manager.can_move_between(grid_pos.0, target_tile, &structure_query, &chunk_query)
            && !is_tile_occupied_by_unit
        {
            // moves the unit
            grid_pos.0 = target_tile;
            current_map_id.0 = target_map_id;
            movement_accumulator.0 -= MovementAccumulator::MOVEMENT_COST;
            desired_movement.tile = None;
            desired_movement.map_id = None;

            // update occupied_by_unit
            occupied_by_unit.remove(&(grid_pos.0, current_map_id.0));
            occupied_by_unit.insert((target_tile, target_map_id));

            // clear collision_history
            collision_history.clear();

            // trigger collision because it's either an empty tile or a passable structure
            if let Some(structure_entity) = map_manager.get_structure(target_tile, &chunk_query) {
                commands.trigger(Collision {
                    entity: structure_entity,
                    source: unit_entity,
                })
            }
        } else {
            // so unit doesn't get stuck
            desired_movement.tile = None;
            desired_movement.map_id = None;

            // trigger collision if it's not a passable structure because it means it hits a wall for example
            // collisions with passable structure are only triggered if the movement succeded
            if let Some(structure_entity) = map_manager.get_structure(target_tile, &chunk_query) {
                let is_passable = structure_query.get(structure_entity).unwrap();
                if !is_passable {
                    commands.trigger(Collision {
                        entity: structure_entity,
                        source: unit_entity,
                    })
                }
            }
        }
    }
}

// TODO: move sync_grid_pos_to_transform_system() elsewhere
pub fn sync_grid_pos_to_transform_system(
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
