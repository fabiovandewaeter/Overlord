use bevy::prelude::*;
use std::collections::HashMap;

use crate::{
    map::structure::{Wall, machine::Machine},
    time::GameTime,
};

/// used to filter and trigger an ApplyCollisionEffect when it's first collision or cooldown is finished
#[derive(EntityEvent)]
pub struct Collision {
    /// the targeted Entity
    pub entity: Entity,
    pub source: Entity,
}

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
impl CollisionHistory {
    pub fn clear(&mut self) {
        self.interactions.clear();
    }
}

/// when units don't move, CollisionHistory isn't clear so active collisions are still stored inside
pub fn update_active_collisions_system(
    mut unit_query: Query<(Entity, &mut CollisionHistory)>,
    target_query: Query<&CollisionEffectCooldown>,
    game_time: Res<GameTime>,
    mut commands: Commands,
) {
    let current_tick = game_time.ticks;

    for (unit_entity, mut history) in unit_query.iter_mut() {
        for (target_entity, (last_seen, last_effect_tick)) in history.interactions.iter_mut() {
            let Ok(cooldown_policy) = target_query.get(*target_entity) else {
                continue;
            };

            let mut should_trigger = false;

            match cooldown_policy {
                CollisionEffectCooldown::Never => {
                    continue;
                }
                CollisionEffectCooldown::EveryTick => {
                    should_trigger = true;
                }
                CollisionEffectCooldown::Ticks(cooldown) => {
                    if current_tick >= *last_effect_tick + *cooldown {
                        should_trigger = true;
                    }
                }
            }

            if should_trigger {
                // On met à jour le tick du dernier effet
                *last_effect_tick = current_tick;

                // On redéclenche l'effet
                commands.trigger(ApplyCollisionEffect {
                    entity: *target_entity,
                    source: unit_entity,
                });
            }

            *last_seen = current_tick;
        }
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

    let last_effect_tick = collision_history
        .interactions
        .get(&event.entity)
        .map(|(_, last)| *last);

    // because history get cleaned at every movements
    let is_new_collision = last_effect_tick.is_none();
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
            if is_new_collision {
                should_trigger = true;
            } else if let Some(last) = last_effect_tick
                && current_tick >= last + *cooldown
            {
                should_trigger = true;
            }
        }
    }

    if should_trigger {
        println!("should_trigger");
        collision_history
            .interactions
            .insert(event.entity, (current_tick, current_tick));

        commands.trigger(ApplyCollisionEffect {
            entity: event.entity,
            source: event.source,
        });
    } else {
        println!("!should_trigger");
        if let Some(last) = last_effect_tick {
            collision_history
                .interactions
                .insert(event.entity, (current_tick, last));
        } else {
            panic!("don't know what is that case")
        }
    }
}

pub fn machine_collision_handler(
    event: On<ApplyCollisionEffect>,
    query: Query<&CollisionEffectCooldown, With<Machine>>,
) {
    if query.get(event.entity).is_err() {
        return;
    }
    println!("machine_collision_handler");
}

pub fn wall_collision_handler(event: On<ApplyCollisionEffect>, query: Query<(), With<Wall>>) {
    if query.get(event.entity).is_err() {
        return;
    }
    println!("wall_collision_handler");
}
