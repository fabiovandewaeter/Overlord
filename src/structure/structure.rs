use bevy::{
    ecs::{bundle::Bundle, component::Component},
    transform::components::Transform,
};

use crate::{
    map::TILE_SIZE,
    physics::{collision::Collider, collision_event::CollisionEffectCooldown},
};

#[derive(Component, Default)]
pub struct Structure;
#[derive(Bundle)]
pub struct StructureBundle {
    pub collider: Collider,
    pub transform: Transform,
    pub collision_effect_cooldown: CollisionEffectCooldown,
    pub structure: Structure,
}
impl StructureBundle {
    pub fn new(transform: Transform, collision_effect_cooldown: CollisionEffectCooldown) -> Self {
        Self {
            collider: Collider::rectangle(TILE_SIZE.x, TILE_SIZE.y),
            transform,
            collision_effect_cooldown,
            structure: Structure,
        }
    }
}

#[derive(Component)]
pub struct Wall;
