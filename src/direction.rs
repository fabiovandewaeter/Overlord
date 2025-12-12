use bevy::prelude::*;

// #[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    North,
    East,
    South,
    West,
}
impl Direction {
    pub fn direction_to_vec2(&self) -> IVec2 {
        match self {
            Direction::North => IVec2 { x: 0, y: -1 },
            Direction::East => IVec2 { x: 1, y: 0 },
            Direction::South => IVec2 { x: 0, y: 1 },
            Direction::West => IVec2 { x: -1, y: 0 },
        }
    }
}

impl Default for Direction {
    fn default() -> Self {
        Self::East
    }
}

pub fn update_sprite_facing_system(
    mut query: Query<(&Direction, &mut Transform), Changed<Transform>>,
) {
    for (facing_direction, mut transform) in query.iter_mut() {
        let is_moving_left = matches!(facing_direction, Direction::West);

        let is_moving_right = matches!(facing_direction, Direction::East);

        if is_moving_left {
            transform.scale.x = -transform.scale.x.abs();
        } else if is_moving_right {
            transform.scale.x = transform.scale.x.abs();
        }
    }
}
