use bevy::ecs::schedule::SystemSet;

pub mod camera;
pub mod direction;
pub mod items;
pub mod map;
pub mod movement;
// pub mod save;
pub mod physics;
pub mod structure;
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
