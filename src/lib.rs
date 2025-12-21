use bevy::ecs::schedule::SystemSet;

pub mod camera;
pub mod direction;
pub mod items;
pub mod map;
pub mod physics;
// pub mod save;
pub mod time;
pub mod units;

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
