use bevy::{
    app::{FixedUpdate, Plugin},
    ecs::schedule::IntoScheduleConfigs,
};

use crate::{
    FixedSet,
    movement::{apply_velocity_system, collision::collision_resolution_system},
    units::{player_control_system, units_follow_field_system},
};

pub struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(
            FixedUpdate,
            (
                (
                    player_control_system
                        .in_set(FixedSet::Movement)
                        .before(apply_velocity_system),
                    units_follow_field_system
                        .in_set(FixedSet::Movement)
                        .before(apply_velocity_system),
                ),
                apply_velocity_system.in_set(FixedSet::Movement),
                collision_resolution_system.in_set(FixedSet::Collision),
            )
                .chain(),
        );
    }
}
