use bevy::prelude::*;

use crate::{
    map::{MapId, coordinates::TileCoordinates},
    physics::collision_event::CollisionEffectCooldown,
    structure::StructureBundle,
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
    pub portal: Portal,
}
impl PortalBundle {
    pub fn new(
        name: Name,
        transform: Transform,
        destination_map_id: MapId,
        destination_tile_pos: TileCoordinates,
    ) -> Self {
        Self {
            name,
            structure_bundle: StructureBundle::new(transform, CollisionEffectCooldown::Never),
            portal: Portal {
                destination_map_id,
                destination_tile_pos,
            },
        }
    }
}
