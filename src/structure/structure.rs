use bevy::{
    ecs::{bundle::Bundle, component::Component},
    transform::components::Transform,
};

use crate::{
    map::TILE_SIZE,
    movement::collision::{Collider, Immovable},
};

#[derive(Component, Default)]
pub struct Structure;
#[derive(Bundle)]
pub struct StructureBundle {
    pub collider: Collider,
    pub transform: Transform,
    pub immovable: Immovable,
    pub structure: Structure,
}
impl StructureBundle {
    pub fn new(transform: Transform) -> Self {
        Self {
            collider: Collider::rectangle(TILE_SIZE.x, TILE_SIZE.y),
            transform,
            immovable: Immovable,
            structure: Structure,
        }
    }
}

#[derive(Component)]
pub struct Wall;
