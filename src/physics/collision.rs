use std::collections::HashMap;

use bevy::{prelude::*, sprite_render::TilemapChunk};

use crate::{
    map::{
        CurrentMapId, MultiMapManager, StructureLayerManager,
        coordinates::{TileCoordinates, absolute_coord_to_tile_coord},
    },
    physics::collision_event::Collision,
    time::GameTime,
    units::Unit,
};

#[derive(Component)]
pub struct Immovable;

#[derive(Component, Debug, Clone)]
pub enum Collider {
    Circle { radius: f32 },
    Rectangle { half_extents: Vec2 },
}

impl Collider {
    pub fn circle(radius: f32) -> Self {
        Self::Circle { radius }
    }

    pub fn rectangle(width: f32, height: f32) -> Self {
        Self::Rectangle {
            half_extents: Vec2::new(width / 2.0, height / 2.0),
        }
    }

    /// Vérifie si deux colliders se chevauchent
    pub fn overlaps(&self, pos1: Vec2, other: &Collider, pos2: Vec2) -> bool {
        match (self, other) {
            (Collider::Circle { radius: r1 }, Collider::Circle { radius: r2 }) => {
                pos1.distance(pos2) < (r1 + r2)
            }
            (
                Collider::Rectangle { half_extents: h1 },
                Collider::Rectangle { half_extents: h2 },
            ) => {
                // AABB overlap test
                let dx = (pos1.x - pos2.x).abs();
                let dy = (pos1.y - pos2.y).abs();
                dx < (h1.x + h2.x) && dy < (h1.y + h2.y)
            }
            (Collider::Circle { radius }, Collider::Rectangle { half_extents }) => {
                // circle-rect: closest point
                let closest = Vec2::new(
                    pos1.x
                        .clamp(pos2.x - half_extents.x, pos2.x + half_extents.x),
                    pos1.y
                        .clamp(pos2.y - half_extents.y, pos2.y + half_extents.y),
                );
                pos1.distance(closest) < *radius
            }
            (Collider::Rectangle { half_extents }, Collider::Circle { radius }) => {
                // symmetric
                let closest = Vec2::new(
                    pos2.x
                        .clamp(pos1.x - half_extents.x, pos1.x + half_extents.x),
                    pos2.y
                        .clamp(pos1.y - half_extents.y, pos1.y + half_extents.y),
                );
                pos2.distance(closest) < *radius
            }
        }
    }

    /// Retourne un vecteur delta à ajouter à pos1. Si pas de chevauchement -> Vec2::ZERO.
    pub fn resolve_overlap(&self, pos1: Vec2, other: &Collider, pos2: Vec2) -> Vec2 {
        match (self, other) {
            // circle-circle
            (Collider::Circle { radius: r1 }, Collider::Circle { radius: r2 }) => {
                let diff = pos1 - pos2;
                let dist = diff.length();
                let min_dist = r1 + r2;
                if dist < f32::EPSILON {
                    // mêmes positions : pousser sur X
                    return Vec2::X * min_dist;
                }
                let overlap = min_dist - dist;
                if overlap > 0.0 {
                    diff.normalize() * overlap
                } else {
                    Vec2::ZERO
                }
            }

            // rect-rect (AABB vs AABB)
            (
                Collider::Rectangle { half_extents: h1 },
                Collider::Rectangle { half_extents: h2 },
            ) => {
                // séparation minimale sur axe x ou y
                let dx = pos1.x - pos2.x;
                let px = (h1.x + h2.x) - dx.abs();
                if px <= 0.0 {
                    return Vec2::ZERO; // pas de chevauchement
                }
                let dy = pos1.y - pos2.y;
                let py = (h1.y + h2.y) - dy.abs();
                if py <= 0.0 {
                    return Vec2::ZERO;
                }

                // choisir l'axe avec le plus petit recouvrement
                if px < py {
                    // push along x
                    let sign = if dx < 0.0 { -1.0 } else { 1.0 };
                    Vec2::new(px * sign, 0.0)
                } else {
                    // push along y
                    let sign = if dy < 0.0 { -1.0 } else { 1.0 };
                    Vec2::new(0.0, py * sign)
                }
            }

            // circle - rect : push circle out of rect
            (Collider::Circle { radius }, Collider::Rectangle { half_extents }) => {
                // closest point on rect to circle center
                let closest = Vec2::new(
                    pos1.x
                        .clamp(pos2.x - half_extents.x, pos2.x + half_extents.x),
                    pos1.y
                        .clamp(pos2.y - half_extents.y, pos2.y + half_extents.y),
                );
                let diff = pos1 - closest;
                let dist = diff.length();
                if dist < f32::EPSILON {
                    // le centre du cercle est exactement sur le côté ou dans le centre -> pousser sur l'axe le plus proche
                    // calculer quelle direction minimal nous éloigne — on prend l'axe où le centre est le plus loin du bord
                    let left = (pos2.x - half_extents.x) - pos1.x;
                    let right = pos1.x - (pos2.x + half_extents.x);
                    let bottom = (pos2.y - half_extents.y) - pos1.y;
                    let top = pos1.y - (pos2.y + half_extents.y);

                    // on trouve la plus petite pénétration en valeur absolue
                    let candidates = [
                        (Vec2::new(-1.0, 0.0), left.abs()),
                        (Vec2::new(1.0, 0.0), right.abs()),
                        (Vec2::new(0.0, -1.0), bottom.abs()),
                        (Vec2::new(0.0, 1.0), top.abs()),
                    ];
                    let (dir, _) = candidates
                        .iter()
                        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
                        .unwrap();
                    return *dir * *radius;
                }
                let overlap = *radius - dist;
                if overlap > 0.0 {
                    diff.normalize() * overlap
                } else {
                    Vec2::ZERO
                }
            }

            // rect - circle : on calcule la correction pour le rect (pos1) en poussant dans la direction opposée
            (Collider::Rectangle { half_extents }, Collider::Circle { radius }) => {
                // plus simple : calculer le point le plus proche sur la rect vers le cercle,
                // puis pousser la rect dans la direction opposée du cercle
                let closest = Vec2::new(
                    pos2.x
                        .clamp(pos1.x - half_extents.x, pos1.x + half_extents.x),
                    pos2.y
                        .clamp(pos1.y - half_extents.y, pos1.y + half_extents.y),
                );
                let diff = closest - pos2; // vecteur de circle -> rect_closest
                let dist = diff.length();
                if dist < f32::EPSILON {
                    // cercle au centre de la rect -> pousser la rect sur X
                    return Vec2::X * (*radius);
                }
                let overlap = *radius - dist;
                if overlap > 0.0 {
                    // on retourne la correction à appliquer sur pos1 (rect) : repousser dans direction diff.normalized()
                    diff.normalize() * overlap
                } else {
                    Vec2::ZERO
                }
            }
        }
    }
}

/// Pour chaque paire, si chevauchement -> calcule la correction et applique moitié/moitié.
pub fn collision_resolution_system(
    mut unit_query: Query<(Entity, &mut Transform, &Collider, &CurrentMapId), With<Unit>>,
    structure_query: Query<(Entity, &Transform, &Collider), (Without<Unit>, With<Immovable>)>,
    multi_map_manager: Res<MultiMapManager>,
    chunk_query: Query<&StructureLayerManager, With<TilemapChunk>>, // mut query: Query<(Entity, &mut Transform, &Collider, Has<Immovable>)>,
    mut commands: Commands,
) {
    // Collecte les données des unités pour le test Unité vs Unité
    let mut units_data: Vec<(Entity, Vec2, Collider, CurrentMapId)> = unit_query
        .iter()
        .map(|(e, t, c, cm)| (e, t.translation.truncate(), c.clone(), cm.clone()))
        .collect();

    // --- PARTIE 1 : Unité vs Unité (Dynamique vs Dynamique) ---
    // On garde la boucle ici car le nombre d'unités est généralement faible (< 100)
    for i in 0..units_data.len() {
        for j in (i + 1)..units_data.len() {
            let (_, pos_i, col_i, curr_m_i) = &units_data[i];
            let (_, pos_j, col_j, curr_m_j) = &units_data[j];

            // if on different map, skip
            if curr_m_i.0 != curr_m_j.0 {
                continue;
            }

            if col_i.overlaps(*pos_i, col_j, *pos_j) {
                let correction = col_i.resolve_overlap(*pos_i, col_j, *pos_j);
                if correction != Vec2::ZERO {
                    // Les deux bougent : on pousse moitié-moitié
                    units_data[i].1 += correction * 0.5;
                    units_data[j].1 -= correction * 0.5;
                }
            }
        }
    }

    // --- PARTIE 2 : Unité vs Environnement (Grid Lookup) ---
    // Au lieu de boucler sur 1000 murs, on regarde juste les 9 cases autour de l'unité
    for (_, (unit_entity, pos, collider, current_map_id)) in units_data.iter_mut().enumerate() {
        // Convertir la position absolue en coordonnées de tuile
        let center_tile =
            absolute_coord_to_tile_coord(crate::map::coordinates::AbsoluteCoordinates {
                x: pos.x,
                y: pos.y,
            });

        // Vérifier les voisins immédiats (3x3 autour de l'unité)
        for x in -1..=1 {
            for y in -1..=1 {
                let check_tile = TileCoordinates {
                    x: center_tile.x + x,
                    y: center_tile.y + y,
                };

                let Some(map_manager) = multi_map_manager.maps.get(&current_map_id.0) else {
                    panic!();
                };

                // Utilisation du MapManager pour récupérer l'entité sur cette case (O(1))
                if let Some(structure_entity) = map_manager.get_tile(check_tile, &chunk_query) {
                    // Si on trouve une structure, on récupère son Collider et Transform
                    if let Ok((struct_entity, struct_transform, struct_collider)) =
                        structure_query.get(structure_entity)
                    {
                        let struct_pos = struct_transform.translation.truncate();

                        if collider.overlaps(*pos, struct_collider, struct_pos) {
                            commands.trigger(Collision {
                                entity: struct_entity,
                                source: *unit_entity,
                            });
                            let correction =
                                collider.resolve_overlap(*pos, struct_collider, struct_pos);
                            // L'unité bouge, le mur est Immovable -> 100% de la correction sur l'unité
                            *pos += correction;
                        }
                    }
                }
            }
        }
    }

    // --- APPLICATION ---
    // On applique les nouvelles positions aux Transforms des unités uniquement
    for (entity, new_pos, _, _) in units_data {
        if let Ok((_, mut transform, _, _)) = unit_query.get_mut(entity) {
            transform.translation.x = new_pos.x;
            transform.translation.y = new_pos.y;
        }
    }
}
