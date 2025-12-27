use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

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

/// UpsCounter is for diagnostic purpose only, not to be used as game time counter
#[derive(Resource, Default)]
pub struct UpsCounter {
    pub ticks: u32,
    pub last_second: f64,
    pub ups: u32,
}

pub fn fixed_update_counter_system(
    mut ups_counter: ResMut<UpsCounter>,
    mut game_time: ResMut<GameTime>,
) {
    ups_counter.ticks += 1;
    game_time.ticks += 1;
}

pub fn display_fps_ups_system(
    time: Res<Time>,
    diagnostics: Res<DiagnosticsStore>,
    mut counter: ResMut<UpsCounter>,
) {
    let now = time.elapsed_secs_f64();
    if now - counter.last_second >= 1.0 {
        // Calcule l’UPS
        counter.ups = counter.ticks;
        counter.ticks = 0;
        counter.last_second = now;

        // Récupère le FPS depuis le plugin
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(fps_avg) = fps.smoothed() {
                println!("FPS: {:.0} | UPS: {}", fps_avg, counter.ups);
            }
        }
    }
}
