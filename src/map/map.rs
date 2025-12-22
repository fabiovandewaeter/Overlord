use crate::{
    FixedSet,
    direction::Direction,
    items::{
        ItemType, Quality,
        inventory::{InputInventory, ItemStack, OutputInventory},
        recipe::RecipeId,
    },
    map::{
        coordinates::{
            ChunkCoordinates, LocalTileCoordinates, TileCoordinates, absolute_coord_to_chunk_coord,
            local_tile_coord_to_tile_coord, tile_coord_to_absolute_coord,
            tile_coord_to_chunk_coord, tile_coord_to_local_tile_coord,
        },
        structure::{
            StructureBundle, Wall,
            machine::{
                BeltMachine, BeltMachineBundle, CraftingMachine, CraftingMachineBundle, Machine,
                MachineBaseBundle, MachinePlugin, MiningMachine, MiningMachineBundle,
            },
            portal::PortalBundle,
        },
    },
    physics::collision_event::CollisionEffectCooldown,
    units::{Unit, pathfinding::RecalculateFlowField},
};
use bevy::{
    prelude::*,
    sprite_render::{TileData, TilemapChunk, TilemapChunkTileData},
};
use rand::Rng;
use std::{collections::HashMap, hash::Hash};

pub const TILE_SIZE: Vec2 = Vec2 { x: 16.0, y: 16.0 };
pub const CHUNK_SIZE: UVec2 = UVec2 { x: 32, y: 32 };
pub const TILE_LAYER: f32 = -1.0;
pub const STRUCTURE_LAYER: f32 = 0.0;
pub const RESOURCE_NODE_LAYER: f32 = -0.1;
pub const PATH_STRUCTURES_PNG: &'static str = "structures/";
pub const PATH_SOURCES_PNG: &'static str = "tiles/resource_nodes/";

pub const DEFAULT_MAP_ID: MapId = MapId(0);

pub struct MapPlugin;
impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MachinePlugin)
            .insert_resource(MultiMapManager::default())
            .add_systems(PostStartup, spawn_first_chunk_system)
            .add_systems(
                FixedUpdate,
                (
                    // spawn_chunks_around_camera_system,
                    spawn_chunks_around_units_system,
                )
                    .chain()
                    .in_set(FixedSet::Process),
            )
            .add_systems(Update, update_tileset_image);
    }
}

/// a tile on the map where mining machine can extract ressources
#[derive(Component)]
pub struct ResourceNode(pub ItemStack);

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

#[derive(Bundle)]
pub struct ChunkBundle {
    pub tilemap_chunk: TilemapChunk,
    pub tilemap_chunk_tile_data: TilemapChunkTileData,
    pub structure_layer_manager: StructureLayerManager,
    pub resource_node_layer_manager: ResourceNodeLayerManager,
    pub transform: Transform,
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

/// all chunks of the map are children of this entity; usefull to change visibility or despawn
#[derive(Component)]
pub struct MapRoot(pub MapId);

// #[derive(Default)]
pub struct MapManager {
    /// MapRoot; all chunks of the map are children of this entity; usefull to change visibility or despawn
    root_entity: Entity,
    chunks: HashMap<ChunkCoordinates, Entity>,
}
impl MapManager {
    pub fn new(map_id: MapId, commands: &mut Commands) -> Self {
        let root_entity = commands
            .spawn((Transform::default(), Visibility::default(), MapRoot(map_id)))
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

    pub fn is_tile_walkable(
        &self,
        tile: TileCoordinates,
        chunk_query: &Query<&StructureLayerManager, With<TilemapChunk>>,
    ) -> bool {
        self.get_structure(tile, chunk_query).is_none()
    }

    pub fn insert_chunk_and_children(
        &mut self,
        chunk_coord: ChunkCoordinates,
        chunk_entity: Entity,
        children: &[Entity], // structures and resource nodes
        commands: &mut Commands,
    ) {
        // make the chunk, structures and resource nodes children of root_entity
        commands.entity(self.root_entity).add_child(chunk_entity);
        commands.entity(self.root_entity).add_children(children);

        self.chunks.insert(chunk_coord, chunk_entity);
    }
}

fn update_tileset_image(
    chunk_query: Single<&TilemapChunk>,
    mut events: MessageReader<AssetEvent<Image>>,
    mut images: ResMut<Assets<Image>>,
) {
    let chunk = *chunk_query;
    for event in events.read() {
        if event.is_loaded_with_dependencies(chunk.tileset.id()) {
            let image = images.get_mut(&chunk.tileset).unwrap();
            image.reinterpret_stacked_2d_as_array(4);
        }
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
                let target_coord = tile_coord_to_absolute_coord(tile_coord);
                if is_wall {
                    let transform =
                        Transform::from_xyz(target_coord.x, target_coord.y, STRUCTURE_LAYER);
                    let bundle =
                        StructureBundle::new(transform, CollisionEffectCooldown::EVERY_SECOND);
                    let wall_entity = commands
                        .spawn((
                            bundle,
                            Wall,
                            Sprite::from_image(
                                asset_server.load(PATH_STRUCTURES_PNG.to_owned() + "wall.png"),
                            ),
                        ))
                        .id();
                    structure_layer_manager
                        .structures
                        .insert(local_tile_coord, wall_entity);
                } else if is_source {
                    let transform =
                        Transform::from_xyz(target_coord.x, target_coord.y, RESOURCE_NODE_LAYER);
                    let mut item_stack = ItemStack {
                        item_type: ItemType::IronOre,
                        quality: Quality::Standard,
                        quantity: 3,
                    };
                    let mut sprite = Sprite::from_image(
                        asset_server.load(PATH_SOURCES_PNG.to_owned() + "iron_ore.png"),
                    );
                    if rng.random_bool(0.2) {
                        item_stack = ItemStack {
                            item_type: ItemType::CopperOre,
                            quality: Quality::Standard,
                            quantity: 3,
                        };
                        sprite = Sprite::from_image(
                            asset_server.load(PATH_SOURCES_PNG.to_owned() + "copper_ore.png"),
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
                                    transform,
                                    CollisionEffectCooldown::EVERY_SECOND,
                                ),
                                direction: Direction::North,
                                // transform,
                                machine: Machine::default(),
                            },
                            output_inventory: OutputInventory::default(),
                            mining_machine: MiningMachine::new(item_stack),
                        };
                        let machine_entity =
                            commands
                                .spawn((
                                    bundle,
                                    Sprite::from_image(asset_server.load(
                                        PATH_STRUCTURES_PNG.to_owned() + "mining_machine.png",
                                    )),
                                ))
                                .id();
                        structure_layer_manager
                            .structures
                            .insert(local_tile_coord, machine_entity);
                    }
                } else if local_tile_coord.x == 10 && local_tile_coord.y == 10 {
                    let transform =
                        Transform::from_xyz(target_coord.x, target_coord.y, STRUCTURE_LAYER);
                    let bundle = PortalBundle::new(
                        "Portail vers (0, 0)".into(),
                        transform,
                        // DEFAULT_MAP_ID,
                        // FAUT faire en sorte que ça enlève la map où la camera est pas
                        MapId(1),
                        TileCoordinates { x: 0, y: 0 },
                    );
                    let portal_entity = commands
                        .spawn((
                            bundle,
                            Sprite::from_image(
                                asset_server.load(PATH_STRUCTURES_PNG.to_owned() + "portal.png"),
                            ),
                        ))
                        .id();
                    structure_layer_manager
                        .structures
                        .insert(local_tile_coord, portal_entity);
                } else if local_tile_coord.x < 10 && local_tile_coord.y < 10 {
                    let transform =
                        Transform::from_xyz(target_coord.x, target_coord.y, STRUCTURE_LAYER);
                    let bundle = PortalBundle::new(
                        "Portail vers (10, 10)".into(),
                        transform,
                        DEFAULT_MAP_ID,
                        TileCoordinates { x: 10, y: 10 },
                    );
                    let portal_entity = commands
                        .spawn((
                            bundle,
                            Sprite::from_image(
                                asset_server.load(PATH_STRUCTURES_PNG.to_owned() + "portal.png"),
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
    let target_coord = tile_coord_to_absolute_coord(tile_coord);
    let transform = Transform::from_xyz(target_coord.x, target_coord.y, STRUCTURE_LAYER);
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
                transform,
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
                asset_server.load(PATH_STRUCTURES_PNG.to_owned() + "belt_machine.png"),
            ),
        ))
        .id();
    structure_layer_manager
        .structures
        .insert(local_tile_coord, machine_entity);
    let local_tile_coord = LocalTileCoordinates { x: 1, y: 0 };
    let tile_coord = local_tile_coord_to_tile_coord(local_tile_coord, chunk_coord);
    let target_coord = tile_coord_to_absolute_coord(tile_coord);
    let transform = Transform::from_xyz(target_coord.x, target_coord.y, STRUCTURE_LAYER);
    let bundle = CraftingMachineBundle {
        base: MachineBaseBundle {
            name: Name::new("Crafting machine"),
            // structure: Structure,
            structure_bundle: StructureBundle::new(
                transform,
                CollisionEffectCooldown::EVERY_SECOND,
            ),
            direction: Direction::South,
            // transform,
            machine: Machine::default(),
        },
        input_inventory: InputInventory::default(),
        output_inventory: OutputInventory::default(),
        crafting_machine: CraftingMachine::new(RecipeId::IronPlateToIronGear),
    };
    let machine_entity = commands
        .spawn((
            bundle,
            Sprite::from_image(
                asset_server.load(PATH_STRUCTURES_PNG.to_owned() + "crafting_machine.png"),
            ),
        ))
        .id();
    structure_layer_manager
        .structures
        .insert(local_tile_coord, machine_entity);

    message_recalculate.write_default();

    let tile_display_size = UVec2::splat(TILE_SIZE.x as u32);
    let chunk_center_x = (chunk_coord.x as f32 * CHUNK_SIZE.x as f32 + CHUNK_SIZE.x as f32 / 2.0)
        * tile_display_size.x as f32;
    let chunk_center_y = -(chunk_coord.y as f32 * CHUNK_SIZE.y as f32 + CHUNK_SIZE.y as f32 / 2.0)
        * tile_display_size.y as f32;

    let chunk_transform =
        Transform::from_translation(Vec3::new(chunk_center_x, chunk_center_y, TILE_LAYER));

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
    let chunk_bundle = ChunkBundle {
        tilemap_chunk: TilemapChunk {
            chunk_size: CHUNK_SIZE,
            tile_display_size,
            tileset: asset_server.load("textures/array_texture.png"),
            ..default()
        },
        tilemap_chunk_tile_data: TilemapChunkTileData(tile_data),
        structure_layer_manager,
        resource_node_layer_manager,
        transform: chunk_transform,
    };
    let chunk_entity = commands.spawn(chunk_bundle).id();
    map_manager.insert_chunk_and_children(chunk_coord, chunk_entity, &all_children, commands);
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
