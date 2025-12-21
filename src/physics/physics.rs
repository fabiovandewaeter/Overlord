use bevy::prelude::*;

use crate::{
    FixedSet,
    map::structure::portal::portal_collision_handler,
    physics::{
        collision::collision_resolution_system,
        collision_event::{
            cleanup_collision_history_system, generic_collision_filter_handler,
            machine_collision_handler, wall_collision_handler,
        },
        movement::apply_velocity_system,
    },
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
                cleanup_collision_history_system
                    .in_set(FixedSet::Collision)
                    .before(collision_resolution_system),
                collision_resolution_system.in_set(FixedSet::Collision),
            )
                .chain(),
        )
        .add_observer(generic_collision_filter_handler)
        .add_observer(machine_collision_handler)
        .add_observer(wall_collision_handler)
        .add_observer(portal_collision_handler);
    }
}
