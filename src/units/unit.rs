// use std::{fs, path::Path};

use crate::{
    // CURRENT_SAVE_VERSION, PATH_SAVES,
    direction::Direction,
    map::{
        CurrentMapId, TILE_SIZE,
        coordinates::{GridPosition, tile_coord_to_absolute_coord},
    },
    physics::{
        collision_event::CollisionHistory,
        movement::{DesiredMovement, MovementAccumulator, SpeedStat},
    },
    units::{pathfinding::FlowField, player::Player},
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Default, Serialize, Deserialize)]
pub struct Unit;
impl Unit {
    pub const DEFAULT_REACH: f32 = 1.0;
    pub const DEFAULT_SCALE_MULTIPLIER: f32 = 0.8;
    pub const DEFAULT_SIZE: f32 = TILE_SIZE.x * Unit::DEFAULT_SCALE_MULTIPLIER;
    // pub const DEFAULT_MOVEMENT_SPEED: f32 = TILE_SIZE.x * 5.0;
    pub const DEFAULT_TILE_PER_SECOND_SPEED: f32 = 8.0;
    pub const DEFAULT_LAYER: f32 = 1.0;
}

#[derive(Bundle)]
pub struct UnitBundle {
    pub name: Name,
    pub transform: Transform,
    pub grid_position: GridPosition,
    pub current_map_id: CurrentMapId,
    pub direction: Direction,
    pub speed_stat: SpeedStat,
    pub movement_accumulator: MovementAccumulator,
    pub desired_movement: DesiredMovement,
    pub collision_history: CollisionHistory,
    pub unit: Unit,
}
impl UnitBundle {
    pub fn new(
        name: Name,
        grid_position: GridPosition,
        current_map_id: CurrentMapId,
        speed_stat: SpeedStat,
    ) -> Self {
        let absolute_coordinates = tile_coord_to_absolute_coord(grid_position.0);
        let transform = Transform::from_xyz(
            absolute_coordinates.x,
            absolute_coordinates.y,
            Unit::DEFAULT_LAYER,
        );
        // transform.scale *= Unit::DEFAULT_SCALE_MULTIPLIER;
        Self {
            name,
            transform,
            grid_position,
            current_map_id,
            direction: Direction::East,
            speed_stat,
            movement_accumulator: MovementAccumulator::default(),
            collision_history: CollisionHistory::default(),
            desired_movement: DesiredMovement::default(),
            unit: Unit,
        }
    }
}

pub fn units_follow_field_system(
    mut unit_query: Query<
        (
            &GridPosition,
            &CurrentMapId,
            &mut MovementAccumulator,
            &mut DesiredMovement,
        ),
        (With<Unit>, Without<Player>),
    >,
    flow_field: Res<FlowField>,
) {
    for (grid_position, current_map_id, movement_accumulator, mut desired_movement) in
        unit_query.iter_mut()
    {
        if movement_accumulator.0 < MovementAccumulator::MOVEMENT_COST {
            continue;
        }

        if current_map_id.0 != flow_field.map_id {
            continue;
        }

        // Trouver la prochaine tuile cible depuis le flow field
        if let Some(&next_tile) = flow_field.get_next_tile(&grid_position.0) {
            *desired_movement = DesiredMovement::new(next_tile, current_map_id.0);

            // movement_accumulator.0 = 0.0;
            // movement_accumulator.0 -= MovementAccumulator::MOVEMENT_COST;
        }
    }
}
