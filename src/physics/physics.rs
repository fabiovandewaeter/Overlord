use bevy::prelude::*;

use crate::{
    FixedSet, GameSet,
    loading::LoadingState,
    map::{
        fog::{apply_fog_to_objects_system, apply_fog_to_tilemap_system},
        structure::portal::portal_collision_handler,
    },
    physics::{
        collision_event::{
            generic_collision_filter_handler, machine_collision_handler,
            update_active_collisions_system, wall_collision_handler,
        },
        movement::{
            apply_desired_movement_system, sync_grid_pos_to_transform_system,
            update_units_movement_accumulators_system,
        },
    },
    units::{
        fov::{update_fov_system, update_units_visibility_fov_system},
        player_control_system, player_mouse_input_system, units_follow_field_system,
    },
};

pub struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(
            FixedUpdate,
            (
                update_units_movement_accumulators_system.in_set(FixedSet::Movement),
                update_active_collisions_system.in_set(FixedSet::Movement),
                (
                    player_control_system.in_set(FixedSet::Movement),
                    units_follow_field_system.in_set(FixedSet::Movement),
                )
                    .before(apply_desired_movement_system),
                apply_desired_movement_system.in_set(FixedSet::Collision),
            )
                .chain()
                .run_if(in_state(LoadingState::Ready)),
        )
        // TODO: move sync_grid_pos_to_transform_system() elsewhere
        .add_systems(
            Update,
            (
                sync_grid_pos_to_transform_system.in_set(GameSet::Visual),
                player_mouse_input_system.in_set(GameSet::Input),
                (
                    update_fov_system,
                    update_units_visibility_fov_system,
                    apply_fog_to_objects_system,
                    apply_fog_to_tilemap_system,
                )
                    .chain()
                    .in_set(GameSet::Visual),
            )
                .run_if(in_state(LoadingState::Ready)),
        )
        .add_observer(generic_collision_filter_handler)
        .add_observer(machine_collision_handler)
        .add_observer(wall_collision_handler)
        .add_observer(portal_collision_handler);
    }
}
