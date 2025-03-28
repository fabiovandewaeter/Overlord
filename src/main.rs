use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

mod helpers;

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    //let texture_handle: Handle<Image> = asset_server.load("img/tiles/grass_0.png");
    let textures: Vec<Handle<Image>> = vec![
        asset_server.load("img/tiles/grass_0.png"),
        asset_server.load("img/tiles/grass_1.png"),
        // Charger d'autres textures selon vos besoins
    ];

    let map_size = TilemapSize { x: 32, y: 32 };

    // Layer 1
    let mut tile_storage = TileStorage::empty(map_size);
    let tilemap_entity = commands.spawn_empty().id();

    fill_tilemap(
        TileTextureIndex(0),
        map_size,
        TilemapId(tilemap_entity),
        &mut commands,
        &mut tile_storage,
    );

    const TILE_SIZE_SQUARE: TilemapTileSize = TilemapTileSize { x: 34.0, y: 34.0 };
    //let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
    //let tile_size = TilemapTileSize { x: 34.0, y: 34.0 };
    let grid_size = TILE_SIZE_SQUARE.into();
    let map_type = TilemapType::default();

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Vector(textures.clone()),
        tile_size: TILE_SIZE_SQUARE,
        transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
        ..Default::default()
    });

    // Layer 2
    let mut tile_storage = TileStorage::empty(map_size);
    let tilemap_entity = commands.spawn_empty().id();

    fill_tilemap(
        TileTextureIndex(1),
        map_size,
        TilemapId(tilemap_entity),
        &mut commands,
        &mut tile_storage,
    );

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Vector(textures),
        tile_size: TILE_SIZE_SQUARE,
        transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 1.0)
            * Transform::from_xyz(32.0, 32.0, 0.0),
        ..Default::default()
    });
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("Layers Example"),
                        ..Default::default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(TilemapPlugin)
        .add_systems(Startup, startup)
        .add_systems(Update, helpers::camera::movement)
        .run();
}
