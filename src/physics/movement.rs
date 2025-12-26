use std::collections::HashSet;

use bevy::{prelude::*, sprite_render::TilemapChunk};

use crate::{
    map::{
        CurrentMapId, MapId, MultiMapManager, StructureLayerManager,
        coordinates::{GridPosition, TileCoordinates, tile_coord_to_absolute_coord},
    },
    units::Unit,
};

#[derive(Component)]
pub struct Passable;

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
    mut query: Query<(&mut GridPosition, &mut DesiredMovement, &mut CurrentMapId), With<Unit>>,
    chunk_query: Query<&StructureLayerManager, With<TilemapChunk>>,
    multi_map_manager: Res<MultiMapManager>,
) {
    let mut occupied: HashSet<(TileCoordinates, MapId)> = HashSet::new();
    for (grid_pos, _, map_id) in query.iter() {
        occupied.insert((grid_pos.0, map_id.0));
    }

    for (mut grid_pos, mut desired_movement, mut current_map_id) in query.iter_mut() {
        let Some(target_tile) = desired_movement.tile else {
            continue;
        };
        let Some(target_map_id) = desired_movement.map_id else {
            panic!()
        };

        let map_manager = multi_map_manager.maps.get(&current_map_id.0).unwrap();

        let is_tile_occupied_by_unit = occupied.contains(&(target_tile, target_map_id));

        if map_manager.is_tile_walkable(target_tile, &chunk_query) && !is_tile_occupied_by_unit {
            occupied.remove(&(grid_pos.0, current_map_id.0));

            grid_pos.0 = target_tile;
            current_map_id.0 = target_map_id;

            desired_movement.tile = None;
            desired_movement.map_id = None;

            occupied.insert((target_tile, target_map_id));
        }
    }
}

// TODO: move that elsewhere
pub fn sync_grid_pos_to_transform(
    mut query: Query<(&GridPosition, &mut Transform), Changed<GridPosition>>,
) {
    for (grid_pos, mut transform) in query.iter_mut() {
        let new_absolute_coords = tile_coord_to_absolute_coord(grid_pos.0);
        transform.translation.x = new_absolute_coords.x;
        transform.translation.y = new_absolute_coords.y;
    }
}
