use bevy::{
    ecs::system::{Res, ResMut},
    input::{ButtonInput, keyboard::KeyCode},
};
use bevy_framepace::Limiter;

fn toggle_TPS_limiter(
    mut settings: ResMut<bevy_framepace::FramepaceSettings>,
    input: Res<ButtonInput<KeyCode>>,
) {
    settings.limiter = Limiter::from_framerate(30.0);
}
