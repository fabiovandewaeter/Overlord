use crate::{
    FixedSet,
    direction::Direction,
    items::{
        ItemType, Quality,
        inventory::{InputInventory, ItemStack, OutputInventory},
        recipe::RecipeId,
    },
    loading::LoadingState,
    map::{
        coordinates::{
            ChunkCoordinates, GridPosition, LocalTileCoordinates, TileCoordinates,
            absolute_coord_to_chunk_coord, local_tile_coord_to_tile_coord,
            tile_coord_to_absolute_coord, tile_coord_to_chunk_coord,
            tile_coord_to_local_tile_coord,
        },
        fog::ChunkFogOfWar,
        resource_node::ResourceNode,
        structure::{
            BlockSight, Structure, StructureBundle, WallBundle,
            machine::{
                BeltMachine, BeltMachineBundle, CraftingMachine, CraftingMachineBundle, Machine,
                MachineBaseBundle, MachinePlugin, MiningMachine, MiningMachineBundle,
            },
            portal::PortalBundle,
        },
    },
    physics::{collision_event::CollisionEffectCooldown, movement::Passable},
    units::{Unit, pathfinding::RecalculateFlowField},
};
use bevy::{
    prelude::*,
    sprite_render::{TileData, TilemapChunk, TilemapChunkTileData},
};
use rand::Rng;
use std::{collections::HashMap, hash::Hash};

pub const TILE_SIZE: UVec2 = UVec2 { x: 16, y: 16 };
pub const CHUNK_SIZE: UVec2 = UVec2 { x: 32, y: 32 };
pub const TILE_LAYER: f32 = -1.0;
pub const DEFAULT_MAP_ID: MapId = MapId(0);

pub struct MapPlugin;
impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MachinePlugin)
            .insert_resource(MultiMapManager::default())
            .add_systems(
                FixedUpdate,
                (
                    // spawn_chunks_around_camera_system,
                    spawn_chunks_around_units_system,
                )
                    .chain()
                    .in_set(FixedSet::Process)
                    .run_if(in_state(LoadingState::Ready)),
            );
    }
}

#[derive(Component, Default, Debug)]
pub struct StructureLayerManager {
    /// LocalTileCoordinates -> Structure entity
    pub structures: HashMap<LocalTileCoordinates, Entity>,
}

#[derive(Component, Default, Debug)]
pub struct ResourceNodeLayerManager {
    /// LocalTileCoordinates -> ResourceNode entity
    pub sources: HashMap<LocalTileCoordinates, Entity>,
}

#[derive(Default, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct MapId(pub u32);

#[derive(Component, Default, Debug, Clone, Copy)]
pub struct CurrentMapId(pub MapId);

pub trait TilemapChunkExt {
    fn new(tileset: Handle<Image>) -> Self;
}
impl TilemapChunkExt for TilemapChunk {
    fn new(tileset: Handle<Image>) -> Self {
        Self {
            chunk_size: CHUNK_SIZE,
            tile_display_size: TILE_SIZE,
            tileset,
            ..default()
        }
    }
}

#[derive(Bundle)]
pub struct ChunkBundle {
    pub tilemap_chunk: TilemapChunk,
    pub tilemap_chunk_tile_data: TilemapChunkTileData,
    pub structure_layer_manager: StructureLayerManager,
    pub resource_node_layer_manager: ResourceNodeLayerManager,
    pub transform: Transform,
    pub chunk_for_of_war: ChunkFogOfWar,
}
impl ChunkBundle {
    pub fn new(
        chunk_coord: ChunkCoordinates,
        tilemap_chunk: TilemapChunk,
        tilemap_chunk_tile_data: TilemapChunkTileData,
        structure_layer_manager: StructureLayerManager,
        resource_node_layer_manager: ResourceNodeLayerManager,
    ) -> Self {
        Self {
            tilemap_chunk,
            tilemap_chunk_tile_data,
            structure_layer_manager,
            resource_node_layer_manager,
            transform: Transform::from_translation(Self::get_chunk_transform(chunk_coord)),
            chunk_for_of_war: ChunkFogOfWar::default(),
        }
    }

    pub fn get_chunk_center(chunk_coord: ChunkCoordinates) -> Vec2 {
        let chunk_center_x = (chunk_coord.x as f32 * CHUNK_SIZE.x as f32
            + CHUNK_SIZE.x as f32 / 2.0)
            * TILE_SIZE.x as f32;
        let chunk_center_y = -(chunk_coord.y as f32 * CHUNK_SIZE.y as f32
            + CHUNK_SIZE.y as f32 / 2.0)
            * TILE_SIZE.y as f32;
        Vec2::new(chunk_center_x, chunk_center_y)
    }

    pub fn get_chunk_transform(chunk_coord: ChunkCoordinates) -> Vec3 {
        let chunk_center = Self::get_chunk_center(chunk_coord);
        Vec3::new(chunk_center.x, chunk_center.y, TILE_LAYER)
    }
}

/// all chunks of the map are children of this entity; usefull to change visibility or despawn
#[derive(Component)]
pub struct MapRoot(pub MapId);

pub struct MapManager {
    /// MapRoot; all chunks of the map are children of this entity; usefull to change visibility or despawn
    root_entity: Entity,
    pub chunks: HashMap<ChunkCoordinates, Entity>,
}
impl MapManager {
    pub fn new(map_id: MapId, commands: &mut Commands) -> Self {
        let root_entity = commands
            .spawn((Transform::default(), Visibility::Hidden, MapRoot(map_id)))
            .id();
        Self {
            root_entity,
            chunks: HashMap::default(),
        }
    }

    pub fn get_structure(
        &self,
        tile: TileCoordinates,
        chunk_query: &Query<&StructureLayerManager, With<TilemapChunk>>,
    ) -> Option<Entity> {
        let chunk_coord = tile_coord_to_chunk_coord(tile);
        if let Some(chunk_entity) = self.chunks.get(&chunk_coord) {
            if let Ok(structure_manager) = chunk_query.get(*chunk_entity) {
                let local_tile = tile_coord_to_local_tile_coord(tile, chunk_coord);
                return structure_manager.structures.get(&local_tile).copied();
            }
        }
        None
    }

    // TODO: try to load from save before spawning a new chunk
    pub fn spawn_chunk_and_get_structure(
        &mut self,
        tile: TileCoordinates,
        chunk_query: &Query<&StructureLayerManager, With<TilemapChunk>>,
        asset_server: &Res<AssetServer>,
        commands: &mut Commands,
        message_recalculate: &mut MessageWriter<RecalculateFlowField>,
    ) -> Option<Entity> {
        let chunk_coord = tile_coord_to_chunk_coord(tile);

        if let Some(chunk_entity) = self.chunks.get(&chunk_coord) {
            if let Ok(structure_manager) = chunk_query.get(*chunk_entity) {
                let local_tile = tile_coord_to_local_tile_coord(tile, chunk_coord);
                return structure_manager.structures.get(&local_tile).copied();
            }
        }

        // if the chunk doesn't exists, spawn a new one
        spawn_one_chunk(
            chunk_coord,
            commands,
            asset_server,
            self,
            message_recalculate,
        );

        self.get_structure(tile, chunk_query)
    }

    /// returns true if there is no structure on tile OR if the tile has Passable component
    /// ONLY if the chunk is already load
    pub fn is_tile_walkable(
        &self,
        tile: TileCoordinates,
        passable_structure_query: &Query<(), (With<Passable>, With<Structure>)>,
        chunk_query: &Query<&StructureLayerManager, With<TilemapChunk>>,
    ) -> bool {
        let chunk_coord = tile_coord_to_chunk_coord(tile);

        if !self.chunks.contains_key(&chunk_coord) {
            return false;
        }

        if let Some(structure_entity) = self.get_structure(tile, chunk_query) {
            return passable_structure_query.get(structure_entity).is_ok();
        }

        true
    }

    pub fn is_sight_blocking(
        &self,
        tile: TileCoordinates,
        block_sight_structure_query: &Query<(), (With<BlockSight>, With<Structure>)>,
        chunk_query: &Query<&StructureLayerManager, With<TilemapChunk>>,
    ) -> bool {
        let chunk_coord = tile_coord_to_chunk_coord(tile);

        if !self.chunks.contains_key(&chunk_coord) {
            return true;
        }

        if let Some(structure_entity) = self.get_structure(tile, chunk_query) {
            return block_sight_structure_query.get(structure_entity).is_ok();
        }

        false
    }

    /// returns true if is_tile_walkable() returns true AND if the movement isn't blocked by diagonal
    pub fn can_move_between(
        &self,
        start: TileCoordinates,
        end: TileCoordinates,
        structure_query: &Query<(), (With<Passable>, With<Structure>)>,
        chunk_query: &Query<&StructureLayerManager, With<TilemapChunk>>,
    ) -> bool {
        if !self.is_tile_walkable(end, &structure_query, &chunk_query) {
            return false;
        }

        // check if there is two structures that block the diagonal
        let dx = end.x - start.x;
        let dy = end.y - start.y;

        if dx.abs() == 1 && dy.abs() == 1 {
            let neighbor_x = TileCoordinates {
                x: (start.x + dx),
                y: start.y,
            };
            let neighbor_y = TileCoordinates {
                x: start.x,
                y: (start.y + dy),
            };

            let walkable_x = self.is_tile_walkable(neighbor_x, &structure_query, &chunk_query);
            let walkable_y = self.is_tile_walkable(neighbor_y, &structure_query, &chunk_query);

            // blocks if the TWO neighbors aren't walkable
            if !walkable_x && !walkable_y {
                return false;
            }
        }

        true
    }

    pub fn insert_chunk_and_children(
        &mut self,
        chunk_coord: ChunkCoordinates,
        chunk_bundle: ChunkBundle,
        children: &[Entity], // structures and resource nodes
        commands: &mut Commands,
    ) {
        let chunk_entity = commands.spawn(chunk_bundle).id();

        // make the chunk, structures and resource nodes children of root_entity
        commands.entity(self.root_entity).add_child(chunk_entity);
        commands.entity(self.root_entity).add_children(children);

        self.chunks.insert(chunk_coord, chunk_entity);
    }
}

#[derive(Resource, Default)]
pub struct MultiMapManager {
    /// MapId -> MapManager
    pub maps: HashMap<MapId, MapManager>,
}
impl MultiMapManager {
    pub fn spawn_map_and_get_mut(
        &mut self,
        map_id: &MapId,
        commands: &mut Commands,
    ) -> &mut MapManager {
        if self.maps.get_mut(map_id).is_none() {
            self.maps
                .insert(*map_id, MapManager::new(*map_id, commands));
        };

        self.maps.get_mut(map_id).unwrap()
    }
}

pub fn spawn_first_chunk_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut multi_map_manager: ResMut<MultiMapManager>,
    mut message_recalculate: MessageWriter<RecalculateFlowField>,
) {
    spawn_one_chunk(
        ChunkCoordinates { x: 0, y: 0 },
        &mut commands,
        &asset_server,
        multi_map_manager.maps.get_mut(&MapId(0)).unwrap(),
        &mut message_recalculate,
    );
}

pub fn spawn_one_chunk(
    chunk_coord: ChunkCoordinates,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    // multi_map_manager: &mut ResMut<MultiMapManager>,
    map_manager: &mut MapManager,
    message_recalculate: &mut MessageWriter<RecalculateFlowField>,
) -> () {
    println!("spawn_one_chunk()");
    let mut rng = rand::rng();
    // let chunk_coord = ChunkCoordinates { x: 0, y: 0 };
    let mut structure_layer_manager = StructureLayerManager::default();
    let mut resource_node_layer_manager = ResourceNodeLayerManager::default();
    for x in 0..CHUNK_SIZE.x {
        for y in 0..CHUNK_SIZE.y {
            let local_tile_coord = LocalTileCoordinates {
                x: x as i32,
                y: y as i32,
            };

            let is_wall = rng.random_bool(0.2);
            let is_source = rng.random_bool(0.2);
            if (local_tile_coord.x > 2) && (local_tile_coord.y > 2) {
                let tile_coord = local_tile_coord_to_tile_coord(local_tile_coord, chunk_coord);
                if is_wall {
                    let bundle = StructureBundle::new(
                        GridPosition(tile_coord),
                        CollisionEffectCooldown::EVERY_SECOND,
                    );
                    let wall_bundle = WallBundle::new(bundle);
                    let wall_entity = commands
                        .spawn((
                            wall_bundle,
                            Sprite::from_image(
                                asset_server
                                    .load(Structure::PATH_PNG_FOLDER.to_owned() + "wall.png"),
                            ),
                        ))
                        .id();
                    structure_layer_manager
                        .structures
                        .insert(local_tile_coord, wall_entity);
                } else if is_source {
                    let target_coord = tile_coord_to_absolute_coord(tile_coord);
                    let transform =
                        Transform::from_xyz(target_coord.x, target_coord.y, ResourceNode::LAYER);
                    let mut item_stack = ItemStack {
                        item_type: ItemType::IronOre,
                        quality: Quality::Standard,
                        quantity: 3,
                    };
                    let mut sprite = Sprite::from_image(
                        asset_server
                            .load(ResourceNode::PATH_PNG_FOLDER.to_owned() + "iron_ore.png"),
                    );
                    if rng.random_bool(0.2) {
                        item_stack = ItemStack {
                            item_type: ItemType::CopperOre,
                            quality: Quality::Standard,
                            quantity: 3,
                        };
                        sprite = Sprite::from_image(
                            asset_server
                                .load(ResourceNode::PATH_PNG_FOLDER.to_owned() + "copper_ore.png"),
                        );
                    }
                    let resource_node_entity = commands
                        .spawn((ResourceNode(item_stack), sprite, transform))
                        .id();
                    resource_node_layer_manager
                        .sources
                        .insert(local_tile_coord, resource_node_entity);

                    if local_tile_coord.x < 5 && local_tile_coord.y < 5 {
                        let bundle = MiningMachineBundle {
                            base: MachineBaseBundle {
                                name: Name::new("Mining machine"),
                                // structure: Structure,
                                structure_bundle: StructureBundle::new(
                                    GridPosition(tile_coord),
                                    CollisionEffectCooldown::EVERY_SECOND,
                                ),
                                direction: Direction::North,
                                // transform,
                                machine: Machine::default(),
                            },
                            output_inventory: OutputInventory::default(),
                            block_sight: BlockSight,
                            mining_machine: MiningMachine::new(item_stack),
                        };
                        let machine_entity = commands
                            .spawn((
                                bundle,
                                Sprite::from_image(asset_server.load(
                                    Structure::PATH_PNG_FOLDER.to_owned() + "mining_machine.png",
                                )),
                            ))
                            .id();
                        structure_layer_manager
                            .structures
                            .insert(local_tile_coord, machine_entity);
                    }
                } else if local_tile_coord.x == 10 && local_tile_coord.y == 10 {
                    let bundle = PortalBundle::new(
                        "Portail vers (0, 0)".into(),
                        GridPosition(tile_coord),
                        MapId(1),
                        TileCoordinates { x: 0, y: 0 },
                    );
                    let portal_entity = commands
                        .spawn((
                            bundle,
                            Sprite::from_image(
                                asset_server
                                    .load(Structure::PATH_PNG_FOLDER.to_owned() + "portal.png"),
                            ),
                        ))
                        .id();
                    structure_layer_manager
                        .structures
                        .insert(local_tile_coord, portal_entity);
                } else if local_tile_coord.x < 10 && local_tile_coord.y < 10 {
                    let bundle = PortalBundle::new(
                        "Portail vers (10, 10)".into(),
                        GridPosition(tile_coord),
                        DEFAULT_MAP_ID,
                        TileCoordinates { x: 10, y: 10 },
                    );
                    let portal_entity = commands
                        .spawn((
                            bundle,
                            Sprite::from_image(
                                asset_server
                                    .load(Structure::PATH_PNG_FOLDER.to_owned() + "portal.png"),
                            ),
                        ))
                        .id();
                    structure_layer_manager
                        .structures
                        .insert(local_tile_coord, portal_entity);
                }
            }
        }
    }

    let local_tile_coord = LocalTileCoordinates { x: 1, y: 1 };
    let tile_coord = local_tile_coord_to_tile_coord(local_tile_coord, chunk_coord);
    let item_stack = ItemStack::new(ItemType::IronPlate, Quality::Perfect, 10);
    let mut input_inventory = InputInventory::default();
    input_inventory
        .0
        .add(item_stack)
        .expect("add_item_stack() didn't work");
    let bundle = BeltMachineBundle {
        base: MachineBaseBundle {
            name: Name::new("Belt machine"),
            // structure: Structure,
            structure_bundle: StructureBundle::new(
                GridPosition(tile_coord),
                CollisionEffectCooldown::EVERY_SECOND,
            ),
            direction: Direction::North,
            // transform,
            machine: Machine::default(),
        },
        input_inventory,
        output_inventory: OutputInventory::default(),
        belt_machine: BeltMachine,
    };
    let machine_entity = commands
        .spawn((
            bundle,
            Sprite::from_image(
                asset_server.load(Structure::PATH_PNG_FOLDER.to_owned() + "belt_machine.png"),
            ),
        ))
        .id();
    structure_layer_manager
        .structures
        .insert(local_tile_coord, machine_entity);
    let local_tile_coord = LocalTileCoordinates { x: 1, y: 0 };
    let tile_coord = local_tile_coord_to_tile_coord(local_tile_coord, chunk_coord);
    let bundle = CraftingMachineBundle {
        base: MachineBaseBundle {
            name: Name::new("Crafting machine"),
            // structure: Structure,
            structure_bundle: StructureBundle::new(
                GridPosition(tile_coord),
                CollisionEffectCooldown::EVERY_SECOND,
            ),
            direction: Direction::South,
            // transform,
            machine: Machine::default(),
        },
        input_inventory: InputInventory::default(),
        output_inventory: OutputInventory::default(),
        block_sight: BlockSight,
        crafting_machine: CraftingMachine::new(RecipeId::IronPlateToIronGear),
    };
    let machine_entity = commands
        .spawn((
            bundle,
            Sprite::from_image(
                asset_server.load(Structure::PATH_PNG_FOLDER.to_owned() + "crafting_machine.png"),
            ),
        ))
        .id();
    structure_layer_manager
        .structures
        .insert(local_tile_coord, machine_entity);

    message_recalculate.write_default();

    let tile_data: Vec<Option<TileData>> = (0..CHUNK_SIZE.element_product())
        // .map(|_| rng.random_range(0..5))
        .map(|_| rng.random_range(1..2))
        .map(|i| {
            if i == 0 {
                None
            } else {
                Some(TileData::from_tileset_index(i - 1))
            }
        })
        .collect();

    let all_children: Vec<Entity> = structure_layer_manager
        .structures
        .values()
        .copied()
        .chain(resource_node_layer_manager.sources.values().copied())
        .collect();

    let tilemap_chunk = TilemapChunk::new(asset_server.load("textures/array_texture.png"));
    let chunk_bundle = ChunkBundle::new(
        chunk_coord,
        tilemap_chunk,
        TilemapChunkTileData(tile_data),
        structure_layer_manager,
        resource_node_layer_manager,
    );
    map_manager.insert_chunk_and_children(chunk_coord, chunk_bundle, &all_children, commands);
}

fn spawn_chunks_around_units_system(
    unit_query: Query<(&Transform, &CurrentMapId), With<Unit>>,
    mut multi_map_manager: ResMut<MultiMapManager>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut message_recalculate: MessageWriter<RecalculateFlowField>,
) {
    const SIZE: i32 = 2;

    for (unit_transform, current_map_id) in unit_query.iter() {
        let unit_chunk_coord = absolute_coord_to_chunk_coord((*unit_transform).into());
        for y in (unit_chunk_coord.y - SIZE)..(unit_chunk_coord.y + SIZE) {
            for x in (unit_chunk_coord.x - SIZE)..(unit_chunk_coord.x + SIZE) {
                let chunk_coord = ChunkCoordinates { x, y };
                let map_manager = multi_map_manager.maps.get_mut(&current_map_id.0).unwrap();

                if map_manager.chunks.contains_key(&chunk_coord) {
                    continue;
                }
                spawn_one_chunk(
                    chunk_coord,
                    &mut commands,
                    &asset_server,
                    multi_map_manager.maps.get_mut(&current_map_id.0).unwrap(),
                    &mut message_recalculate,
                );
            }
        }
    }
}
