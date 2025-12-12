use bevy::ecs::schedule::SystemSet;

pub mod camera;
pub mod direction;
pub mod items;
pub mod map;
pub mod movement;
// pub mod save;
pub mod structure;
pub mod units;

pub const CURRENT_SAVE_VERSION: f32 = 1.0;
pub const PATH_SAVES: &'static str = "saves";

pub const UPS_TARGET: u32 = 30; // 30 ticks per second
pub const ZOOM_IN_SPEED: f32 = 0.25 / 400000000.0;
pub const ZOOM_OUT_SPEED: f32 = 4.0 * 400000000.0;
pub const CAMERA_SPEED: f32 = 37.5;
pub const LENGTH_UNIT: f32 = 16.0;
pub const DAY_DURATION: u32 = UPS_TARGET * 60 * 10; // 10 minutes in ticks

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum GameSet {
    Input,
    Visual,
    UI,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum FixedSet {
    // Order matters: Process -> Move -> Collide
    Process,
    Movement,
    Collision,
}
