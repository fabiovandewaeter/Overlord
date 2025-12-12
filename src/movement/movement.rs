use bevy::prelude::*;

#[derive(Component, Debug, Clone, Copy)]
pub struct LinearVelocity {
    pub x: f32,
    pub y: f32,
}

impl LinearVelocity {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn as_vec2(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }
}

pub fn apply_velocity_system(
    mut query: Query<(&mut Transform, &LinearVelocity)>,
    time: Res<Time<Fixed>>,
) {
    let dt = time.delta_secs();
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation.x += velocity.x * dt;
        transform.translation.y += velocity.y * dt;
    }
}
