use bevy::{prelude::*, sprite_render::TilemapChunkTileData};

use crate::map::{
    CHUNK_SIZE, ResourceNodeLayerManager, StructureLayerManager, coordinates::LocalTileCoordinates,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FogState {
    /// black
    Unknown,
    /// grayed out
    Explored,
    /// full colors
    Visible,
}

#[derive(Component)]
pub struct ChunkFogOfWar {
    /// CHUNK_TILE * CHUNK_TILE
    pub grid: Vec<FogState>,
    /// to know when rendering needs to be updated
    pub is_dirty: bool,
}
impl ChunkFogOfWar {
    pub fn local_tile_coord_to_grid_index(&self, local: LocalTileCoordinates) -> usize {
        (local.x + local.y * CHUNK_SIZE.x as i32) as usize
    }

    pub fn get(&self, local: LocalTileCoordinates) -> FogState {
        let index = self.local_tile_coord_to_grid_index(local);
        if index < self.grid.len() {
            self.grid[index]
        } else {
            FogState::Unknown
        }
    }

    pub fn set_visible(&mut self, local: LocalTileCoordinates) {
        let index = self.local_tile_coord_to_grid_index(local);
        if index < self.grid.len() {
            if self.grid[index] != FogState::Visible {
                self.grid[index] = FogState::Visible;
                self.is_dirty = true;
            }
        }
    }

    /// called before calculating FOV
    pub fn mark_visible_as_explored(&mut self) {
        for state in self.grid.iter_mut() {
            if *state == FogState::Visible {
                *state = FogState::Explored;
                self.is_dirty = true;
            }
        }
    }
}
impl Default for ChunkFogOfWar {
    fn default() -> Self {
        Self {
            grid: vec![FogState::Unknown; (CHUNK_SIZE.x * CHUNK_SIZE.y) as usize],
            is_dirty: true,
        }
    }
}

pub fn apply_fog_to_tilemap_system(
    mut query: Query<(&mut ChunkFogOfWar, &mut TilemapChunkTileData), Changed<ChunkFogOfWar>>,
) {
    for (mut fog, mut tile_data) in query.iter_mut() {
        if !fog.is_dirty {
            continue;
        }

        // On parcourt les deux vecteurs en parallèle (ils ont la même taille et le même ordre)
        // tile_data.0 est un Vec<Option<TileData>>
        for (i, tile_option) in tile_data.0.iter_mut().enumerate() {
            // On ne touche que si la tuile existe (si c'est Some)
            if let Some(tile) = tile_option {
                // On récupère l'état du brouillard pour cet index
                // (On suppose que fog.grid a la même taille, ce qui devrait être le cas)
                let fog_state = fog.grid.get(i).unwrap_or(&FogState::Unknown);

                match fog_state {
                    FogState::Visible => {
                        tile.visible = true;
                        tile.color = Color::WHITE; // Couleur normale
                    }
                    FogState::Explored => {
                        tile.visible = true;
                        // On teinte en gris foncé pour l'effet "Souvenir"
                        // Ajuste les valeurs RGB pour l'ambiance voulue
                        tile.color = Color::srgb(0.3, 0.3, 0.4);
                    }
                    FogState::Unknown => {
                        // Option A: Rendre invisible (si le fond de l'écran est noir)
                        // tile.visible = false;

                        // Option B: Si tu veux un "Noir" opaque par dessus le fond
                        tile.visible = true;
                        tile.color = Color::BLACK;
                    }
                }
            }
        }

        fog.is_dirty = false;
    }
}

pub fn apply_fog_to_objects_system(
    // On détecte les changements sur le brouillard des chunks
    chunk_query: Query<
        (
            &ChunkFogOfWar,
            &StructureLayerManager,
            &ResourceNodeLayerManager,
        ),
        Changed<ChunkFogOfWar>,
    >,
    // On va modifier la visibilité et la couleur des entités (Structures et Ressources)
    mut object_query: Query<(&mut Visibility, &mut Sprite)>,
) {
    for (fog, structure_mgr, resource_mgr) in chunk_query.iter() {
        if !fog.is_dirty {
            continue;
        } // Optimisation (même si Changed<> filtre déjà beaucoup)

        // 1. Mettre à jour les Structures
        for (local_pos, &entity) in structure_mgr.structures.iter() {
            if let Ok((mut visibility, mut sprite)) = object_query.get_mut(entity) {
                let state = fog.get(*local_pos);
                apply_visuals(state, &mut visibility, &mut sprite);
            }
        }

        // 2. Mettre à jour les Resource Nodes
        for (local_pos, &entity) in resource_mgr.sources.iter() {
            if let Ok((mut visibility, mut sprite)) = object_query.get_mut(entity) {
                let state = fog.get(*local_pos);
                apply_visuals(state, &mut visibility, &mut sprite);
            }
        }
    }
}

// Fonction helper pour éviter de dupliquer la logique
fn apply_visuals(state: FogState, visibility: &mut Visibility, sprite: &mut Sprite) {
    match state {
        FogState::Visible => {
            *visibility = Visibility::Inherited;
            sprite.color = Color::WHITE; // Couleur normale
        }
        FogState::Explored => {
            *visibility = Visibility::Inherited;
            // Même teinte bleutée/grise que le sol pour l'effet "souvenir"
            sprite.color = Color::srgb(0.3, 0.3, 0.4);
        }
        FogState::Unknown => {
            *visibility = Visibility::Hidden;
        }
    }
}
