use crate::{
    FixedSet,
    loading::LoadingState,
    map::{
        CurrentMapId, MapId, MultiMapManager, StructureLayerManager,
        coordinates::{GridPosition, TileCoordinates},
        structure::Structure,
    },
    physics::movement::Passable,
    units::{Player, player_control_system},
};
use bevy::{prelude::*, sprite_render::TilemapChunk};
use pathfinding::prelude::dijkstra_all;
use std::collections::HashMap;

const FLOWFIELD_RADIUS: i32 = 50; // radius in tile

pub struct PathfindingPlugin;
impl Plugin for PathfindingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FlowField::default())
            .add_message::<RecalculateFlowField>()
            .add_systems(
                FixedUpdate,
                calculate_flow_field_system
                    .in_set(FixedSet::Process)
                    .before(player_control_system)
                    .run_if(in_state(LoadingState::Ready)),
            );
    }
}

#[derive(Resource, Default)]
// pub struct FlowField(pub HashMap<TileCoordinates, Vec2>);
pub struct FlowField {
    flow_field: HashMap<TileCoordinates, TileCoordinates>,
    pub map_id: MapId,
}
impl FlowField {
    pub fn get_next_tile(&self, current_tile_coords: &TileCoordinates) -> Option<&TileCoordinates> {
        self.flow_field.get(&current_tile_coords)
    }

    pub fn set_flow_field(
        &mut self,
        player_tile: TileCoordinates,
        pathing_result: HashMap<TileCoordinates, (TileCoordinates, i32)>,
        map_id: MapId,
    ) {
        self.flow_field.clear();

        for (tile, (parent, _cost)) in pathing_result {
            if tile != player_tile {
                self.flow_field.insert(tile, parent);
            }
        }
        self.map_id = map_id;
    }

    pub fn clear(&mut self, new_map_id: MapId) {
        self.flow_field.clear();
        self.map_id = new_map_id;
    }

    pub fn insert(
        &mut self,
        current_tile: TileCoordinates,
        next_tile: TileCoordinates,
        new_map_id: MapId,
    ) {
        self.flow_field.insert(current_tile, next_tile);
        self.map_id = new_map_id;
    }
}

#[derive(Message, Default)]
pub struct RecalculateFlowField;

pub fn calculate_flow_field_system(
    mut message_recalculate: MessageReader<RecalculateFlowField>,
    mut flow_field: ResMut<FlowField>,
    structure_query: Query<(), (With<Passable>, With<Structure>)>,
    multi_map_manager: Res<MultiMapManager>,
    player_query: Query<(&GridPosition, &CurrentMapId), With<Player>>,
    chunk_query: Query<&StructureLayerManager, With<TilemapChunk>>,
) {
    if message_recalculate.is_empty() {
        return;
    }
    message_recalculate.clear();

    let Ok((grid_pos, current_map_id)) = player_query.single() else {
        return;
    };
    let Some(map_manager) = multi_map_manager.maps.get(&current_map_id.0) else {
        panic!("Map manager not found for current map");
    };
    let player_tile = grid_pos.0;

    // TODO: regarder si on devrait utiliser dijkstra_partial ou dijkstra_reach
    let pathing_result = dijkstra_all(&player_tile, |&tile| {
        let mut neighbors = Vec::with_capacity(8);

        for y in -1..=1 {
            for x in -1..=1 {
                if x == 0 && y == 0 {
                    continue;
                }

                let neighbor_tile = TileCoordinates {
                    x: tile.x + x,
                    y: tile.y + y,
                };

                // Vérifier que le voisin est dans le rayon ET praticable
                let dx_dist = (neighbor_tile.x - player_tile.x).abs();
                let dy_dist = (neighbor_tile.y - player_tile.y).abs();

                if dx_dist > FLOWFIELD_RADIUS || dy_dist > FLOWFIELD_RADIUS {
                    continue;
                }

                if map_manager.can_move_between(tile, neighbor_tile, &structure_query, &chunk_query)
                {
                    // if reached, the movement is valide
                    let cost = if x == 0 || y == 0 { 10 } else { 14 };
                    neighbors.push((neighbor_tile, cost));
                }
            }
        }

        neighbors
    });

    flow_field.set_flow_field(player_tile, pathing_result, current_map_id.0);
}
