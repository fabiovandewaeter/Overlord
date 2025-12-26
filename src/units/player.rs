use bevy::prelude::*;

use crate::{
    direction::Direction,
    map::{
        CurrentMapId,
        coordinates::{GridPosition, TileCoordinates},
    },
    physics::movement::DesiredMovement,
    units::{SpeedStat, UnitBundle, pathfinding::RecalculateFlowField},
};

#[derive(Component)]
pub struct Player;
#[derive(Bundle)]
pub struct PlayerBundle {
    pub base: UnitBundle,
    pub player: Player,
}
impl PlayerBundle {
    pub fn new(base: UnitBundle) -> Self {
        Self {
            base,
            player: Player,
        }
    }
}

pub fn player_control_system(
    mut unit_query: Query<
        (
            &GridPosition,
            &CurrentMapId,
            &mut DesiredMovement,
            &mut Direction,
            &SpeedStat,
        ),
        With<Player>,
    >,
    input: Res<ButtonInput<KeyCode>>,
    mut message_recalculate: MessageWriter<RecalculateFlowField>,
) {
    let Ok((grid_pos, current_map_id, mut desired_movement, mut direction, speed_stat)) =
        unit_query.single_mut()
    else {
        return;
    };

    let mut delta = IVec2::ZERO;
    let mut has_moved = false;

    if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp) {
        delta.y += 1;
        *direction = Direction::North;
    }
    if input.pressed(KeyCode::KeyS) || input.pressed(KeyCode::ArrowDown) {
        delta.y -= 1;
        *direction = Direction::South;
    }
    if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) {
        delta.x -= 1;
        *direction = Direction::West;
    }
    if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) {
        delta.x += 1;
        *direction = Direction::East;
    }

    if delta.x != 0 || delta.y != 0 {
        has_moved = true;
    }

    desired_movement.tile = Some(TileCoordinates {
        x: grid_pos.0.x + delta.x,
        y: grid_pos.0.y - delta.y,
    });
    desired_movement.map_id = Some(current_map_id.0);

    // TODO: change to put that after the collisions check
    if has_moved {
        message_recalculate.write_default();
    }
}
