use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};
use bevy_transform_interpolation::prelude::TransformInterpolationPlugin;
use overlord::{
    FixedSet, GameSet,
    camera::{
        CameraMovement, CameraMovementKind, DayNightOverlay, UpsCounter, display_fps_ups_system,
        handle_camera_inputs_system,
    },
    items::recipe::RecipeBook,
    map::{
        self, CurrentMapId, MapManager, MapPlugin, MultiMapManager,
        coordinates::{Coordinates, coord_to_absolute_coord},
    },
    physics::PhysicsPlugin,
    time::{GameTime, day_night_cycle_system},
    units::{PlayerBundle, SpeedStat, Unit, UnitBundle, pathfinding::PathfindingPlugin},
};

fn main() {
    App::new()
        // .configure_sets(Update, (GameSet::Input, GameSet::Visual, GameSet::UI))
        // .configure_sets(
        //     FixedUpdate,
        //     (FixedSet::Process, FixedSet::Movement, FixedSet::Collision).chain(),
        // )
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Overlord".to_string(),
                        present_mode: bevy::window::PresentMode::AutoVsync,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(TransformInterpolationPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(PhysicsPlugin)
        .add_plugins(PathfindingPlugin)
        .add_plugins(MapPlugin)
        // .add_plugins(SavePlugin)
        // .insert_resource(TimeState::default())
        .insert_resource(UpsCounter {
            ticks: 0,
            last_second: 0.0,
            ups: 0,
        })
        .insert_resource(GameTime::default())
        .insert_resource(RecipeBook::default())
        .insert_resource(Time::<Fixed>::from_hz(GameTime::UPS_TARGET as f64))
        .add_systems(Startup, setup_system)
        .add_systems(
            Update,
            (
                handle_camera_inputs_system.in_set(GameSet::Input),
                display_fps_ups_system.in_set(GameSet::UI),
                day_night_cycle_system.in_set(GameSet::Visual),
                // control_time_system,
            ),
        )
        .add_systems(
            FixedUpdate,
            (fixed_update_counter_system.in_set(FixedSet::Process),),
        )
        .run();
}

fn setup_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut game_time: ResMut<GameTime>,
    mut multi_map_manager: ResMut<MultiMapManager>,
) {
    // Audio
    commands.spawn((
        AudioPlayer::new(asset_server.load("audio/gentle-rain.ogg")),
        PlaybackSettings::LOOP,
    ));

    // Camera
    let mut orthographic_projection = OrthographicProjection::default_2d();
    orthographic_projection.scale *= 0.8;
    let projection = Projection::Orthographic(orthographic_projection);
    commands.spawn((
        Camera2d,
        Camera { ..default() },
        projection,
        CameraMovement(CameraMovementKind::SmoothFollowPlayer),
    ));
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::NONE), // Commence transparent
        ZIndex(100),                  // S'assure qu'il est au-dessus du jeu
        DayNightOverlay,
    ));

    // start daytime in middle of the day
    game_time.ticks = GameTime::TICKS_PER_DAY / 2;

    // maps
    multi_map_manager
        .maps
        .insert(map::DEFAULT_MAP_ID, MapManager::default());

    // Units + Player
    let player_texture_handle = asset_server.load("default.png");
    let speed = SpeedStat(Unit::DEFAULT_MOVEMENT_SPEED);
    let coordinates = Coordinates { x: 0.0, y: 0.0 };
    let absolute_coordinates = coord_to_absolute_coord(coordinates);
    let mut transform = Transform::from_xyz(
        absolute_coordinates.x,
        absolute_coordinates.y,
        Unit::DEFAULT_LAYER,
    );
    transform.scale *= Unit::DEFAULT_SCALE_MULTIPLIER;
    let unit_bundle = UnitBundle::new(
        Name::new("Player"),
        transform,
        CurrentMapId(map::DEFAULT_MAP_ID),
        speed,
    );
    let bundle = PlayerBundle::new(unit_bundle);
    commands.spawn((bundle, Sprite::from_image(player_texture_handle.clone())));

    let coordinates = Coordinates { x: 5.0, y: 5.0 };
    let absolute_coordinates = coord_to_absolute_coord(coordinates);
    let mut transform = Transform::from_xyz(
        absolute_coordinates.x,
        absolute_coordinates.y,
        Unit::DEFAULT_LAYER,
    );
    transform.scale *= Unit::DEFAULT_SCALE_MULTIPLIER;
    let bundle = UnitBundle::new(
        Name::new("Monstre"),
        transform,
        CurrentMapId(map::DEFAULT_MAP_ID),
        SpeedStat(Unit::DEFAULT_MOVEMENT_SPEED),
    );
    commands.spawn((bundle, Sprite::from_image(player_texture_handle.clone())));
}

pub fn fixed_update_counter_system(
    mut ups_counter: ResMut<UpsCounter>,
    mut game_time: ResMut<GameTime>,
) {
    ups_counter.ticks += 1;
    game_time.ticks += 1;
}
