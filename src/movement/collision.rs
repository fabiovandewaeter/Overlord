use bevy::prelude::*;

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
    mut query: Query<(Entity, &mut Transform, &Collider, Has<Immovable>)>,
) {
    // Collecte toutes les positions et colliders (clone de collider pour traitement hors borrow)
    let mut entities_data: Vec<(Entity, Vec2, Collider, bool)> = query
        .iter()
        .map(|(e, t, c, imm)| (e, t.translation.truncate(), c.clone(), imm))
        .collect();

    // Pour chaque paire : test et résolution
    for i in 0..entities_data.len() {
        for j in (i + 1)..entities_data.len() {
            let (_, pos_i, collider_i, imm_i) = &entities_data[i];
            let (_, pos_j, collider_j, imm_j) = &entities_data[j];

            // Si pas de chevauchement, on skip
            if !collider_i.overlaps(*pos_i, collider_j, *pos_j) {
                continue;
            }

            // Calcule la correction pour pos_i (delta à ajouter à pos_i pour séparer)
            let correction = collider_i.resolve_overlap(*pos_i, collider_j, *pos_j);
            if correction == Vec2::ZERO {
                continue;
            }

            match (imm_i, imm_j) {
                (true, true) => {
                    // les deux immobiles : on ne bouge rien
                    continue;
                }
                (true, false) => {
                    // i immobile, j movable -> applique la correction complète en sens opposé sur j
                    entities_data[j].1 -= correction;
                }
                (false, true) => {
                    // i movable, j immobile -> applique la correction complète sur i
                    entities_data[i].1 += correction;
                }
                (false, false) => {
                    // les deux movables -> partage moitié/moitié
                    entities_data[i].1 += correction * 0.5;
                    entities_data[j].1 -= correction * 0.5;
                }
            }
        }
    }

    // Applique les corrections aux transforms
    for (entity, new_pos, _, _) in entities_data {
        if let Ok((_, mut transform, _, _)) = query.get_mut(entity) {
            transform.translation.x = new_pos.x;
            transform.translation.y = new_pos.y;
        }
    }
}
