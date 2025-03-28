use bevy::{prelude::*, window::PresentMode};
use bevy_ecs_tilemap::prelude::*;

mod helpers;

const TPS: f64 = 60.0;

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

pub fn printnul(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut OrthographicProjection), With<Camera>>,
) {
    for (mut transform, mut ortho) in query.iter_mut() {
        let mut direction = Vec3::ZERO;

        direction -= Vec3::new(1.0, 0.0, 0.0);

        let z = transform.translation.z;
        transform.translation += time.delta_secs() * direction * 500.;
        // Important! We need to restore the Z values when moving the camera around.
        // Bevy has a specific camera setup and this can mess with how our layers are shown.
        transform.translation.z = z;
    }
}

fn main() {
    use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
    use bevy::diagnostic::LogDiagnosticsPlugin;
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("Layers Example"),
                        present_mode: PresentMode::Immediate, // disable vsync
                        ..Default::default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(TilemapPlugin)
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_systems(Startup, startup)
        .insert_resource(Time::<Fixed>::from_seconds(1.0 / TPS)) // tick speed decoupled from framerate
        .add_systems(Update, helpers::camera::movement)
        .add_systems(FixedUpdate, printnul)
        .run();
}
