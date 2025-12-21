use std::collections::HashMap;

use bevy::prelude::*;

use crate::{
    structure::{Wall, machine::Machine},
    time::GameTime,
};

/// used to filter and trigger an ApplyCollisionEffect when it's first collision or cooldown is finished
#[derive(EntityEvent)]
pub struct Collision {
    /// the targeted Entity
    pub entity: Entity,
    pub source: Entity,
}

///
#[derive(EntityEvent)]
pub struct ApplyCollisionEffect {
    /// the targeted Entity
    pub entity: Entity,
    pub source: Entity,
}

/// cooldown in ticks before the collision effect get applied again on the target
#[derive(Component)]
pub enum CollisionEffectCooldown {
    /// only at entry and never again until next new collision
    Never,
    EveryTick,
    Ticks(u64),
}
impl CollisionEffectCooldown {
    pub const EVERY_SECOND: Self = Self::Ticks(GameTime::TICKS_PER_SECOND);
}

/// Key: Entity (l'objet touché) -> Value: (Tick de dernier contact, Tick du dernier effet)
/// If interactions.contains_key() is false, the effect can be applied again
/// The cleanup_collision_history_system is a garbage collector that
#[derive(Component, Default)]
pub struct CollisionHistory {
    pub interactions: HashMap<Entity, (u64, u64)>,
}

pub fn cleanup_collision_history_system(
    mut query: Query<&mut CollisionHistory>,
    game_time: Res<GameTime>,
) {
    let current_tick = game_time.ticks;

    // Si le tick est 0 (début du jeu), on ne fait rien pour éviter l'underflow
    if current_tick == 0 {
        return;
    }

    // if current_tick > last_contact + 1, it means that the effect can be applied again event if cooldown isn't finished
    for mut history in &mut query {
        // retain ne garde que les éléments qui renvoient true.
        // On garde l'entrée SI : last_seen est égal à current_tick OU current_tick - 1
        // Si last_seen < current_tick - 1, ça veut dire qu'on a raté au moins une frame de collision -> on est sorti.
        history
            .interactions
            .retain(|_, (last_seen, _)| *last_seen >= current_tick.saturating_sub(1));
    }
}

pub fn generic_collision_filter_handler(
    event: On<Collision>,
    target_query: Query<&CollisionEffectCooldown>,
    mut unit_query: Query<&mut CollisionHistory>,
    game_time: Res<GameTime>,
    mut commands: Commands,
) {
    let Ok(mut collision_history) = unit_query.get_mut(event.source) else {
        return;
    };

    let current_tick = game_time.ticks;

    let (last_seen, last_effect) = *collision_history
        .interactions
        .get(&event.entity)
        .unwrap_or(&(0, 0));

    let is_new_collision = !collision_history.interactions.contains_key(&event.entity);
    let mut should_trigger = false;

    let cooldown_policy = target_query.get(event.entity).unwrap();
    match cooldown_policy {
        CollisionEffectCooldown::Never => {
            if is_new_collision {
                should_trigger = true;
            }
        }
        CollisionEffectCooldown::EveryTick => {
            should_trigger = true;
        }
        CollisionEffectCooldown::Ticks(cooldown) => {
            if is_new_collision || current_tick >= last_effect + *cooldown {
                should_trigger = true;
            }
        }
    }

    if should_trigger {
        collision_history
            .interactions
            .insert(event.entity, (current_tick, current_tick));

        commands.trigger(ApplyCollisionEffect {
            entity: event.entity,
            source: event.source,
        });
    } else {
        collision_history
            .interactions
            .insert(event.entity, (current_tick, last_effect));
    }
}

pub fn machine_hit_handler(
    event: On<ApplyCollisionEffect>,
    query: Query<&CollisionEffectCooldown, With<Machine>>,
) {
    if query.get(event.entity).is_err() {
        return;
    }
    println!("machine_hit_handler");
}

pub fn wall_hit_handler(event: On<ApplyCollisionEffect>, query: Query<(), With<Wall>>) {
    if query.get(event.entity).is_err() {
        return;
    }
    println!("wall_hit_handler");
}
