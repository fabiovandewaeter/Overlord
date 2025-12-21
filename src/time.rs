use bevy::prelude::*;

use crate::camera::DayNightOverlay;

#[derive(Resource, Default)]
pub struct GameTime {
    pub ticks: u64,
}

impl GameTime {
    pub const UPS_TARGET: u64 = 30; // 30 ticks per second
    pub const TICKS_PER_SECOND: u64 = Self::UPS_TARGET; // UPS_TARGET ticks per second
    pub const TICKS_PER_DAY: u64 = Self::UPS_TARGET * 60 * 20; // 20 minutes

    pub const PERCENT_SUNRISE_START: f32 = 0.15;
    pub const PERCENT_MIDDLE_DAY: f32 = 0.25;
    pub const PERCENT_SUNSET_START: f32 = 0.75;
    pub const PERCENT_MIDNIGHT: f32 = 0.85;

    pub fn get_day_percent(&self) -> f32 {
        (self.ticks % Self::TICKS_PER_DAY) as f32 / Self::TICKS_PER_DAY as f32
    }
}

// Système de cycle jour/nuit avec transitions douces
pub fn day_night_cycle_system(
    game_time: Res<GameTime>,
    mut query: Query<&mut BackgroundColor, With<DayNightOverlay>>,
) {
    let day_percent = game_time.get_day_percent();

    // Couleur de base bleue foncée, seul l'alpha varie
    let base_blue = (0.0, 0.05, 0.15);
    let night_alpha = 0.7; // Opaque la nuit
    let day_alpha = 0.0; // Transparent le jour

    // Calcul de l'alpha selon le moment de la journée
    let alpha = if day_percent < GameTime::PERCENT_SUNRISE_START {
        // Nuit
        night_alpha
    } else if day_percent < GameTime::PERCENT_MIDDLE_DAY {
        // Transition nuit → jour
        let t = (day_percent - GameTime::PERCENT_SUNRISE_START)
            / (GameTime::PERCENT_MIDDLE_DAY - GameTime::PERCENT_SUNRISE_START);
        lerp(night_alpha, day_alpha, smoothstep(t))
    } else if day_percent < GameTime::PERCENT_SUNSET_START {
        // Plein jour
        day_alpha
    } else if day_percent < GameTime::PERCENT_MIDNIGHT {
        // Transition jour → nuit
        let t = (day_percent - GameTime::PERCENT_SUNSET_START)
            / (GameTime::PERCENT_MIDNIGHT - GameTime::PERCENT_SUNSET_START);
        lerp(day_alpha, night_alpha, smoothstep(t))
    } else {
        // Nuit
        night_alpha
    };

    // Applique la couleur avec l'alpha calculé
    for mut background_color in query.iter_mut() {
        *background_color =
            BackgroundColor(Color::srgba(base_blue.0, base_blue.1, base_blue.2, alpha));
    }
}

// Interpolation linéaire simple
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

// Fonction de lissage pour des transitions plus naturelles (ease-in-out)
fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

// #[derive(Resource, Default)]
// struct TimeState {
//     is_paused: bool,
// }

// fn control_time_system(
//     mut fixed_time: ResMut<Time<Fixed>>,
//     input: Res<ButtonInput<KeyCode>>,
//     mut time_state: ResMut<TimeState>,
// ) {
//     // P pour Pause, pour alterner entre l'état de pause
//     if input.just_pressed(KeyCode::Space) {
//         if time_state.is_paused {
//             println!("Temps de la simulation repris.");
//             fixed_time.set_timestep_hz(UPS_TARGET as f64);
//             time_state.is_paused = false;
//         } else {
//             println!("Temps de la simulation mis en pause.");
//             fixed_time.set_timestep_hz(0.0);
//             time_state.is_paused = true;
//         }
//     }

//     // Si le jeu est en pause, on ne gère pas les autres commandes de vitesse
//     if time_state.is_paused {
//         return;
//     }

//     // Accélérer (x2)
//     if input.just_pressed(KeyCode::KeyY) {
//         let current_hz = fixed_time.timestep().as_secs_f64().recip();
//         let new_hz = current_hz * 2.0;
//         println!("Temps de la simulation accéléré à {} Hz.", new_hz);
//         fixed_time.set_timestep_hz(new_hz);
//     }

//     // Ralentir (/2)
//     if input.just_pressed(KeyCode::KeyU) {
//         let current_hz = fixed_time.timestep().as_secs_f64().recip();
//         let new_hz = current_hz / 2.0;
//         println!("Temps de la simulation ralenti à {} Hz.", new_hz);
//         fixed_time.set_timestep_hz(new_hz);
//     }

//     // Normal (retour à la vitesse initiale)
//     if input.just_pressed(KeyCode::KeyI) {
//         println!("Temps de la simulation réinitialisé à {} Hz.", UPS_TARGET);
//         fixed_time.set_timestep_hz(UPS_TARGET as f64);
//     }
// }
