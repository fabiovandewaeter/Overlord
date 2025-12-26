use bevy::{prelude::*, sprite_render::TilemapChunk};

use crate::map::{
    CurrentMapId, MultiMapManager, StructureLayerManager,
    coordinates::{GridPosition, TileCoordinates},
};

#[derive(Component)]
pub struct Passable;

#[derive(Component)]
pub struct DesiredMovement {
    pub target: Option<TileCoordinates>,
}
impl Default for DesiredMovement {
    fn default() -> Self {
        Self { target: None }
    }
}

pub fn apply_desired_movement(
    mut query: Query<(&mut GridPosition, &mut DesiredMovement, &CurrentMapId)>,
    other_query: Query<(&GridPosition, &DesiredMovement, &CurrentMapId)>,
    chunk_query: &Query<&StructureLayerManager, With<TilemapChunk>>,
    multi_map_manager: Res<MultiMapManager>,
) {
    for (mut grid_pos, mut desired_movement, current_map_id) in query.iter_mut() {
        let Some(target_tile) = desired_movement.target else {
            return;
        };

        let map_manager = multi_map_manager.maps.get(&current_map_id.0).unwrap();

        let is_tile_occupied_by_unit =
            other_query.iter().any(|(other_grid_pos, _, other_map_id)| {
                other_map_id.0 == current_map_id.0 && other_grid_pos.0 == target_tile
            });
        if !map_manager.is_tile_walkable(target_tile, chunk_query) && !is_tile_occupied_by_unit {
            grid_pos.0 = target_tile;
            desired_movement.target = None;
        }
    }
}
