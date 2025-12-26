use bevy::{prelude::*, sprite_render::TilemapChunk};

use crate::{
    map::{
        MapId, MultiMapManager, StructureLayerManager,
        coordinates::{GridPosition, TileCoordinates},
        structure::StructureBundle,
    },
    physics::{
        collision_event::{ApplyCollisionEffect, CollisionEffectCooldown, CollisionHistory},
        movement::{DesiredMovement, Passable},
    },
    time::GameTime,
    units::{Unit, pathfinding::RecalculateFlowField},
};

#[derive(Component)]
pub struct Portal {
    pub destination_map_id: MapId,
    pub destination_tile_pos: TileCoordinates,
}
#[derive(Bundle)]
pub struct PortalBundle {
    pub name: Name,
    pub structure_bundle: StructureBundle,
    pub passable: Passable,
    pub portal: Portal,
}
impl PortalBundle {
    pub fn new(
        name: Name,
        grid_position: GridPosition,
        destination_map_id: MapId,
        destination_tile_pos: TileCoordinates,
    ) -> Self {
        Self {
            name,
            structure_bundle: StructureBundle::new(grid_position, CollisionEffectCooldown::Never),
            passable: Passable,
            portal: Portal {
                destination_map_id,
                destination_tile_pos,
            },
        }
    }
}

pub fn portal_collision_handler(
    event: On<ApplyCollisionEffect>,
    mut multi_map_manager: ResMut<MultiMapManager>,
    chunk_query: Query<&StructureLayerManager, With<TilemapChunk>>,
    portal_query: Query<&Portal>,
    mut unit_query: Query<(&mut DesiredMovement, &mut CollisionHistory), With<Unit>>,
    game_time: Res<GameTime>,

    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut message_recalculate: MessageWriter<RecalculateFlowField>,
) {
    let Ok(portal) = portal_query.get(event.entity) else {
        return;
    };
    let (mut unit_desired_movement, mut collision_history) =
        unit_query.get_mut(event.source).unwrap();

    let destination_map_manager =
        multi_map_manager.spawn_map_and_get_mut(&portal.destination_map_id, &mut commands);

    if let Some(target_portal_entity) = destination_map_manager.spawn_chunk_and_get_structure(
        portal.destination_tile_pos,
        &chunk_query,
        &asset_server,
        &mut commands,
        &mut message_recalculate,
    ) {
        // make sure the target_portal doesn't teleport back the unit instantly
        if portal_query.get(target_portal_entity).is_ok() {
            let current_tick = game_time.ticks;
            collision_history
                .interactions
                .insert(target_portal_entity, (current_tick, current_tick));
        }
    }

    *unit_desired_movement =
        DesiredMovement::new(portal.destination_tile_pos, portal.destination_map_id);
}
