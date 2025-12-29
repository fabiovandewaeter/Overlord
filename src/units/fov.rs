use crate::{
    direction::Direction,
    map::{
        CurrentMapId, MultiMapManager, StructureLayerManager,
        coordinates::{
            GridPosition, TileCoordinates, tile_coord_to_chunk_coord,
            tile_coord_to_local_tile_coord,
        },
        fog::{ChunkFogOfWar, FogState},
        structure::{BlockSight, Structure},
    },
    units::{Player, Unit},
};
use bevy::{prelude::*, sprite_render::TilemapChunk};
use std::collections::HashSet;

pub const DEFAULT_FOV: i32 = 19;
pub const FOV_CONE_ANGLE: f32 = 120.0;

/// Calcule les tuiles visibles.
/// `origin`: Position du joueur
/// `radius`: Portée de la vue
/// `is_blocking`: Closure qui renvoie true si une tuile bloque la vue (mur)
pub fn compute_fov<F>(
    origin: TileCoordinates,
    radius: i32,
    mut is_blocking: F,
) -> HashSet<TileCoordinates>
where
    F: FnMut(TileCoordinates) -> bool,
{
    let mut visible_tiles = HashSet::new();
    visible_tiles.insert(origin);

    // On scanne les 8 octants
    for i in 0..8 {
        compute_octant(
            i,
            origin,
            radius,
            1,
            1.0,
            0.0,
            &mut is_blocking,
            &mut visible_tiles,
        );
    }

    visible_tiles
}

fn compute_octant<F>(
    octant: usize,
    origin: TileCoordinates,
    radius: i32,
    x: i32,
    mut top: f32,
    bottom: f32,
    is_blocking: &mut F,
    visible_tiles: &mut HashSet<TileCoordinates>,
) where
    F: FnMut(TileCoordinates) -> bool,
{
    for x in x..=radius {
        let top_y = if x == 0 {
            0
        } else {
            ((x * 2 + 1) as f32 * top / 2.0 + 0.5) as i32
        };
        let bottom_y = if x == 0 {
            0
        } else {
            ((x * 2 - 1) as f32 * bottom / 2.0 + 0.5) as i32
        };

        let mut was_opaque = -1; // -1: n/a, 0: false, 1: true

        for y in (bottom_y..=top_y).rev() {
            // Scan de haut en bas ou inversement selon l'implémentation
            let tile = transform_octant(octant, origin, x, y);
            let in_radius = (x * x + y * y) <= radius * radius; // Cercle

            if in_radius {
                visible_tiles.insert(tile);
            }

            let is_opaque = in_radius && is_blocking(tile);

            if x < radius {
                if is_opaque {
                    if was_opaque == 0 {
                        // On vient de passer d'une zone transparente à opaque -> nouvelle pente top pour récursion
                        let new_top = (y as f32 * 2.0 + 1.0) / (x as f32 * 2.0 - 1.0);
                        compute_octant(
                            octant,
                            origin,
                            radius,
                            x + 1,
                            top,
                            new_top,
                            is_blocking,
                            visible_tiles,
                        );
                    }
                    was_opaque = 1;
                } else {
                    if was_opaque == 1 {
                        // On passe d'opaque à transparent -> on ajuste la pente bottom actuelle
                        let new_bottom = (y as f32 * 2.0 + 1.0) / (x as f32 * 2.0 + 1.0);
                        top = new_bottom;
                    }
                    was_opaque = 0;
                }
            }
        }

        if was_opaque != 0 {
            break; // Tout l'arc est bloqué
        }
    }
}

fn transform_octant(octant: usize, origin: TileCoordinates, x: i32, y: i32) -> TileCoordinates {
    let (dx, dy) = match octant {
        0 => (x, y),
        1 => (y, x),
        2 => (y, -x),
        3 => (x, -y),
        4 => (-x, -y),
        5 => (-y, -x),
        6 => (-y, x),
        7 => (-x, y),
        _ => (0, 0),
    };
    TileCoordinates {
        x: origin.x + dx,
        y: origin.y + dy,
    }
}

pub fn update_fov_system(
    mut player_query: Query<
        (&GridPosition, &CurrentMapId, &Direction),
        (
            With<Player>,
            Or<(Changed<GridPosition>, Changed<Direction>)>,
        ),
    >,
    multi_map_manager: Res<MultiMapManager>,
    mut chunk_query: Query<&mut ChunkFogOfWar, With<TilemapChunk>>,
    chunk_structure_query: Query<&StructureLayerManager, With<TilemapChunk>>,
    block_sight_structure_query: Query<(), (With<BlockSight>, With<Structure>)>,
) {
    let Ok((player_pos, map_id, facing_direction)) = player_query.single_mut() else {
        return;
    };

    let Some(map_manager) = multi_map_manager.maps.get(&map_id.0) else {
        return;
    };

    // 1. "Dimmer" la lumière existante : Tout ce qui est Visible devient Explored
    //    On itère sur tous les chunks chargés de la map courante
    for chunk_entity in map_manager.chunks.values() {
        if let Ok(mut fog) = chunk_query.get_mut(*chunk_entity) {
            fog.mark_visible_as_explored();
        }
    }

    // 2. Calculer le nouveau FOV
    let visible_tiles = compute_fov(player_pos.0, DEFAULT_FOV, |tile| {
        map_manager.is_sight_blocking(tile, &block_sight_structure_query, &chunk_structure_query)
    });

    let facing_vec = facing_direction.to_vec2();
    // Calcul du seuil (Cosinus de la moitié de l'angle)
    // Exemple : Pour 120°, moitié = 60°. Cos(60°) = 0.5.
    // Tout produit scalaire > 0.5 est dans le cône.
    let half_angle_rad = (FOV_CONE_ANGLE / 2.0).to_radians();
    let threshold = half_angle_rad.cos();

    // 3. Appliquer le FOV aux chunks
    for tile in visible_tiles {
        if tile == player_pos.0 {
            apply_fog_to_tile(tile, map_manager, &mut chunk_query);
            continue;
        }

        // B. Calculer le vecteur vers la tuile cible
        let tile_vec = Vec2::new(
            (tile.x - player_pos.0.x) as f32,
            (tile.y - player_pos.0.y) as f32,
        )
        .normalize_or_zero(); // Important pour éviter la division par zéro

        // C. Produit Scalaire
        let dot_product = facing_vec.dot(tile_vec);

        // D. Si c'est dans le cône, on affiche
        if dot_product >= threshold {
            apply_fog_to_tile(tile, map_manager, &mut chunk_query);
        }
    }
}

fn apply_fog_to_tile(
    tile: TileCoordinates,
    map_manager: &crate::map::MapManager,
    chunk_query: &mut Query<&mut ChunkFogOfWar, With<TilemapChunk>>,
) {
    let chunk_coord = tile_coord_to_chunk_coord(tile);
    if let Some(chunk_entity) = map_manager.chunks.get(&chunk_coord) {
        if let Ok(mut fog) = chunk_query.get_mut(*chunk_entity) {
            let local_coord = tile_coord_to_local_tile_coord(tile, chunk_coord);
            fog.set_visible(local_coord);
        }
    }
}

pub fn update_units_visibility_fov_system(
    mut unit_query: Query<
        (&GridPosition, &CurrentMapId, &mut Visibility),
        (With<Unit>, Without<Player>),
    >,
    player_query: Query<&CurrentMapId, With<Player>>, // Pour savoir sur quelle map est le joueur
    multi_map_manager: Res<MultiMapManager>,
    chunk_query: Query<&ChunkFogOfWar>,
) {
    let Ok(player_map_id) = player_query.single() else {
        return;
    };

    for (pos, unit_map_id, mut vis) in unit_query.iter_mut() {
        // Si pas sur la même map, caché par défaut (déjà géré par ton autre système)
        if unit_map_id.0 != player_map_id.0 {
            *vis = Visibility::Hidden;
            continue;
        }

        let chunk_coord = tile_coord_to_chunk_coord(pos.0);
        let map_manager = multi_map_manager.maps.get(&unit_map_id.0).unwrap();

        let is_visible = if let Some(chunk_entity) = map_manager.chunks.get(&chunk_coord) {
            if let Ok(fog) = chunk_query.get(*chunk_entity) {
                let local = tile_coord_to_local_tile_coord(pos.0, chunk_coord);
                // On affiche l'unité SEULEMENT si la tuile est VISIBLE (pas Explored)
                fog.get(local) == FogState::Visible
            } else {
                false
            }
        } else {
            false
        };

        *vis = if is_visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}
