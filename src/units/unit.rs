// use std::{fs, path::Path};

use crate::{
    // CURRENT_SAVE_VERSION, PATH_SAVES,
    direction::Direction,
    map::{
        TILE_SIZE,
        coordinates::{
            AbsoluteCoordinates, absolute_coord_to_tile_coord, tile_coord_to_absolute_coord,
        },
    },
    movement::{LinearVelocity, collision::Collider},
    units::{pathfinding::FlowField, player::Player},
};
use bevy::prelude::*;
use bevy_transform_interpolation::prelude::TranslationInterpolation;
use serde::{Deserialize, Serialize};

pub const UNIT_REACH: f32 = 1.0;
pub const UNIT_DEFAULT_SIZE: f32 = TILE_SIZE.x * 0.8;
pub const UNIT_DEFAULT_MOVEMENT_SPEED: f32 = TILE_SIZE.x * 3.0;
pub const UNIT_LAYER: f32 = 1.0;

#[derive(Component, Debug, Default, Serialize, Deserialize)]
pub struct Unit;
#[derive(Bundle)]
pub struct UnitBundle {
    pub name: Name,
    pub transform: Transform,
    pub direction: Direction,
    pub speed_stat: SpeedStat,
    pub collider: Collider,
    pub linear_velocity: LinearVelocity,
    pub translation_interpolation: TranslationInterpolation,
    pub unit: Unit,
}
impl UnitBundle {
    pub fn new(name: Name, transform: Transform, speed_stat: SpeedStat) -> Self {
        Self {
            name,
            transform,
            direction: Direction::East,
            speed_stat,
            collider: Collider::circle(UNIT_DEFAULT_SIZE / 2.0),
            linear_velocity: LinearVelocity::ZERO,
            translation_interpolation: TranslationInterpolation,
            unit: Unit,
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SpeedStat(pub f32);
impl Default for SpeedStat {
    fn default() -> Self {
        Self(UNIT_DEFAULT_MOVEMENT_SPEED)
    }
}

pub fn units_follow_field_system(
    mut unit_query: Query<
        (&mut LinearVelocity, &Transform, &SpeedStat),
        (With<Unit>, Without<Player>),
    >,
    flow_field: Res<FlowField>,
) {
    for (mut linear_velocity, transform, speed_stat) in unit_query.iter_mut() {
        let current_pos_world = transform.translation.xy();
        let current_pos_abs = AbsoluteCoordinates {
            x: current_pos_world.x,
            y: current_pos_world.y,
        };
        let current_tile = absolute_coord_to_tile_coord(current_pos_abs);

        // Trouver la prochaine tuile cible depuis le flow field
        if let Some(&next_tile) = flow_field.0.get(&current_tile) {
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
