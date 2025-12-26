// use std::{fs, path::Path};

use crate::{
    // CURRENT_SAVE_VERSION, PATH_SAVES,
    direction::Direction,
    map::{
        CurrentMapId, TILE_SIZE,
        coordinates::{
            AbsoluteCoordinates, GridPosition, absolute_coord_to_tile_coord,
            tile_coord_to_absolute_coord,
        },
    },
    physics::{collision_event::CollisionHistory, movement::DesiredMovement},
    units::{pathfinding::FlowField, player::Player},
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Default, Serialize, Deserialize)]
pub struct Unit;
impl Unit {
    pub const DEFAULT8REACH: f32 = 1.0;
    pub const DEFAULT_SCALE_MULTIPLIER: f32 = 0.8;
    pub const DEFAULT_SIZE: f32 = TILE_SIZE.x * Unit::DEFAULT_SCALE_MULTIPLIER;
    pub const DEFAULT_MOVEMENT_SPEED: f32 = TILE_SIZE.x * 5.0;
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
        let mut transform = Transform::from_xyz(
            absolute_coordinates.x,
            absolute_coordinates.y,
            Unit::DEFAULT_LAYER,
        );
        transform.scale *= Unit::DEFAULT_SCALE_MULTIPLIER;
        Self {
            name,
            transform,
            grid_position,
            current_map_id,
            direction: Direction::East,
            speed_stat,
            collision_history: CollisionHistory::default(),
            desired_movement: DesiredMovement::default(),
            unit: Unit,
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SpeedStat(pub f32);
impl Default for SpeedStat {
    fn default() -> Self {
        Self(Unit::DEFAULT_MOVEMENT_SPEED)
    }
}

pub fn units_follow_field_system(
    mut unit_query: Query<
        (&mut LinearVelocity, &Transform, &SpeedStat, &CurrentMapId),
        (With<Unit>, Without<Player>),
    >,
    flow_field: Res<FlowField>,
) {
    for (mut linear_velocity, transform, speed_stat, current_map_id) in unit_query.iter_mut() {
        if current_map_id.0 != flow_field.map_id {
            continue;
        }

        let current_pos_world = transform.translation.xy();
        let current_pos_abs = AbsoluteCoordinates {
            x: current_pos_world.x,
            y: current_pos_world.y,
        };
        let current_tile = absolute_coord_to_tile_coord(current_pos_abs);

        // Trouver la prochaine tuile cible depuis le flow field
        if let Some(&next_tile) = flow_field.get_next_tile(&current_tile) {
            // Calculer la position cible (le centre de la prochaine tuile)
            let target_pos_abs = tile_coord_to_absolute_coord(next_tile);
            let target_pos_world: Vec2 = target_pos_abs.into();

            // Calculer la direction et la force vers la cible
            let to_target_vec = target_pos_world - current_pos_world;
            let direction_to_target = to_target_vec.normalize_or_zero();

            // Appliquer la force
            linear_velocity.x = direction_to_target.x * speed_stat.0;
            linear_velocity.y = direction_to_target.y * speed_stat.0;
        } else {
            // pas de chemin -> on arrête
            linear_velocity.x = 0.0;
            linear_velocity.y = 0.0;
        }
    }
}
