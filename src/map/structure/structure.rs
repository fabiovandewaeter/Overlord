use bevy::{
    ecs::{bundle::Bundle, component::Component},
    transform::components::Transform,
};

use crate::{
    map::coordinates::{GridPosition, tile_coord_to_absolute_coord},
    physics::collision_event::CollisionEffectCooldown,
};

#[derive(Component, Default)]
pub struct Structure;
impl Structure {
    pub const LAYER: f32 = 0.0;
    pub const PATH_PNG_FOLDER: &'static str = "structures/";
}
#[derive(Bundle)]
pub struct StructureBundle {
    pub transform: Transform,
    pub grid_position: GridPosition,
    pub collision_effect_cooldown: CollisionEffectCooldown,
    pub structure: Structure,
}
impl StructureBundle {
    pub fn new(
        grid_position: GridPosition,
        collision_effect_cooldown: CollisionEffectCooldown,
    ) -> Self {
        let absolute_coordinates = tile_coord_to_absolute_coord(grid_position.0);
        let transform = Transform::from_xyz(
            absolute_coordinates.x,
            absolute_coordinates.y,
            Structure::LAYER,
        );
        Self {
            transform,
            grid_position,
            collision_effect_cooldown,
            structure: Structure,
        }
    }
}

#[derive(Component)]
pub struct Wall;
