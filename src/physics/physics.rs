use bevy::prelude::*;

use crate::{
    FixedSet,
    loading::LoadingState,
    map::structure::portal::portal_collision_handler,
    physics::collision_event::{
        cleanup_collision_history_system, generic_collision_filter_handler,
        machine_collision_handler, wall_collision_handler,
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
                    player_control_system.in_set(FixedSet::Movement),
                    units_follow_field_system.in_set(FixedSet::Movement),
                ),
                cleanup_collision_history_system.in_set(FixedSet::Collision),
            )
                .chain()
                .run_if(in_state(LoadingState::Ready)),
        )
        .add_observer(generic_collision_filter_handler)
        .add_observer(machine_collision_handler)
        .add_observer(wall_collision_handler)
        .add_observer(portal_collision_handler);
    }
}
