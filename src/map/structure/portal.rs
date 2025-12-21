use bevy::prelude::*;

use crate::{
    map::{MapId, coordinates::TileCoordinates, structure::StructureBundle},
    physics::{collision::Passable, collision_event::CollisionEffectCooldown},
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
        transform: Transform,
        destination_map_id: MapId,
        destination_tile_pos: TileCoordinates,
    ) -> Self {
        Self {
            name,
            structure_bundle: StructureBundle::new(transform, CollisionEffectCooldown::Never),
            passable: Passable,
            portal: Portal {
                destination_map_id,
                destination_tile_pos,
            },
        }
    }
}
