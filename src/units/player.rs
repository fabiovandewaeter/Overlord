use bevy::prelude::*;

use crate::{
    direction::Direction,
    movement::LinearVelocity,
    units::{SpeedStat, pathfinding::RecalculateFlowField},
};

#[derive(Component)]
pub struct Player;

pub fn player_control_system(
    mut unit_query: Query<(&mut LinearVelocity, &mut Direction, &SpeedStat), With<Player>>,
    input: Res<ButtonInput<KeyCode>>,
    mut message_recalculate: MessageWriter<RecalculateFlowField>,
    time: Res<Time<Fixed>>,
) {
    let Ok((mut velocity, mut direction, speed_stat)) = unit_query.single_mut() else {
        return;
    };

    let mut delta = Vec2::ZERO;
    let mut has_moved = false;

    if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp) {
        delta.y += 1.0;
        *direction = Direction::North;
    }
    if input.pressed(KeyCode::KeyS) || input.pressed(KeyCode::ArrowDown) {
        delta.y -= 1.0;
        *direction = Direction::South;
    }
    if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) {
        delta.x -= 1.0;
        *direction = Direction::West;
    }
    if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) {
        delta.x += 1.0;
        *direction = Direction::East;
    }

    // Normaliser le vecteur pour éviter que le mouvement diagonal
    // soit plus rapide (racine(1²+1²) = 1.414)
    if delta.length_squared() > 0.0 {
        has_moved = true;
        delta = delta.normalize();
    }

    // Appliquer la vitesse
    // let delta_time = time.delta_secs();
    // velocity.x = delta.x * speed_stat.0 * delta_time;
    // velocity.y = delta.y * speed_stat.0 * delta_time;
    velocity.x = delta.x * speed_stat.0;
    velocity.y = delta.y * speed_stat.0;

    // TODO: change to put that after the collisions check
    if has_moved {
        message_recalculate.write_default();
    }
}
