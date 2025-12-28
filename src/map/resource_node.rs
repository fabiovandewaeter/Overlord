use bevy::prelude::*;

use crate::items::inventory::ItemStack;

/// a tile on the map where mining machine can extract ressources
#[derive(Component)]
pub struct ResourceNode(pub ItemStack);
impl ResourceNode {
    pub const LAYER: f32 = -0.1;
    pub const PATH_PNG_FOLDER: &'static str = "tiles/resource_nodes/";
}
