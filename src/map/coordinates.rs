use bevy::{math::Vec2, transform::components::Transform};

use crate::map::{CHUNK_SIZE, TILE_SIZE};

/// absolute_coord = (5.5 * TILE_SIZE.X, 0.5 * TILE_SIZE.y) | coord = (5.5, 0.5) | tile_coord = (5, 0)
// #[derive(Component, Default, Debug, Clone, Copy, PartialEq)]
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Coordinates {
    pub x: f32,
    pub y: f32,
}

/// absolute_coord = (5.5 * TILE_SIZE.X, 0.5 * TILE_SIZE.y) | coord = (5.5, 0.5) | tile_coord = (5, 0)
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct AbsoluteCoordinates {
    pub x: f32,
    pub y: f32,
}

impl From<Transform> for AbsoluteCoordinates {
    fn from(p: Transform) -> AbsoluteCoordinates {
        AbsoluteCoordinates {
            x: p.translation.x,
            y: p.translation.y,
        }
    }
}

impl From<AbsoluteCoordinates> for Vec2 {
    fn from(p: AbsoluteCoordinates) -> Vec2 {
        Vec2::new(p.x, p.y)
    }
}

/// absolute_coord = (5.5 * TILE_SIZE.X, 0.5 * TILE_SIZE.y) | coord = (5.5, 0.5) | tile_coord = (5, 0)
#[derive(Default, Debug, Hash, Clone, Copy, PartialEq, Eq)]
pub struct TileCoordinates {
    pub x: i32,
    pub y: i32,
}

#[derive(Default, Debug, Hash, Clone, Copy, PartialEq, Eq)]
pub struct LocalTileCoordinates {
    pub x: i32,
    pub y: i32,
}

/// chunk_coord : (1,1) is 1 right and 1 down
/// Chunkcoord {x: 2, y: 2} <=> TileCoordinates {x: 2*CHUNK_SIZE, y: 2*CHUNK_SIZE}
#[derive(Default, Debug, Hash, Clone, Copy, PartialEq, Eq)]
pub struct ChunkCoordinates {
    pub x: i32,
    pub y: i32,
}

// ========= coordinates conversion =========
/// absolute_coord = (5.5 * TILE_SIZE.X, 0.5 * TILE_SIZE.y) | coord = (5.5, 0.5) | tile_coord = (5, 0)
/// chunk_coord : (1,1) is 1 right and 1 down

pub fn local_tile_coord_to_tile_coord(
    local_tile_coord: LocalTileCoordinates,
    chunk_coord: ChunkCoordinates,
) -> TileCoordinates {
    TileCoordinates {
        x: local_tile_coord.x + chunk_coord.x * (CHUNK_SIZE.x as i32),
        y: local_tile_coord.y + chunk_coord.y * (CHUNK_SIZE.y as i32),
    }
}

// Conversion coordonnées logiques -> monde ; (5.5, 0.5) => (5.5 * TILE_SIZE.x, 0.5 * TILE_SIZE.y)
pub fn coord_to_absolute_coord(coord: Coordinates) -> AbsoluteCoordinates {
    AbsoluteCoordinates {
        x: (coord.x + 0.5) * TILE_SIZE.x as f32,
        y: -((coord.y + 0.5) * TILE_SIZE.y as f32),
        // x: (coord.x) * TILE_SIZE.x as f32,
        // y: -((coord.y) * TILE_SIZE.y as f32),
    }
}

pub fn tile_coord_to_local_tile_coord(
    tile_coord: TileCoordinates,
    chunk_coord: ChunkCoordinates,
) -> LocalTileCoordinates {
    LocalTileCoordinates {
        x: tile_coord.x - chunk_coord.x * (CHUNK_SIZE.x as i32),
        y: tile_coord.y - chunk_coord.y * (CHUNK_SIZE.y as i32),
    }
}

// // adds 0.5 to coordinates to make entities spawn based on the corner of there sprite and not the center
pub fn tile_coord_to_absolute_coord(tile_coord: TileCoordinates) -> AbsoluteCoordinates {
    AbsoluteCoordinates {
        x: tile_coord.x as f32 * TILE_SIZE.x + TILE_SIZE.x * 0.5,
        y: -(tile_coord.y as f32 * TILE_SIZE.y + TILE_SIZE.y * 0.5),
        // x: tile_coord.x as f32 * TILE_SIZE.x,
        // y: -(tile_coord.y as f32 * TILE_SIZE.y),
    }
}

pub fn tile_coord_to_coord(tile_coord: TileCoordinates) -> Coordinates {
    Coordinates {
        x: tile_coord.x as f32,
        y: tile_coord.y as f32,
    }
}

// (5.5, 0.5) => (5, 0)
pub fn coord_to_tile_coord(coord: Coordinates) -> TileCoordinates {
    TileCoordinates {
        x: coord.x.floor() as i32,
        y: coord.y.floor() as i32,
    }
}

// Conversion monde -> coordonnées logiques
pub fn absolute_coord_to_coord(absolute_coord: AbsoluteCoordinates) -> Coordinates {
    Coordinates {
        // x: absolute_coord.x as f32 / TILE_SIZE.x,
        // y: (-absolute_coord.y as f32) / TILE_SIZE.y,
        x: absolute_coord.x as f32 / TILE_SIZE.x - 0.5,
        y: (-absolute_coord.y as f32) / TILE_SIZE.y - 0.5,
    }
}

// Conversion monde -> coordonnées logiques
pub fn absolute_coord_to_tile_coord(absolute_coord: AbsoluteCoordinates) -> TileCoordinates {
    TileCoordinates {
        // x: ((absolute_coord.x as f32 / TILE_SIZE.x) - 0.5).floor() as i32,
        // y: (((-absolute_coord.y as f32) / TILE_SIZE.y) - 0.5).floor() as i32,
        x: ((absolute_coord.x as f32 / TILE_SIZE.x) - 0.5).round() as i32,
        y: (((-absolute_coord.y as f32) / TILE_SIZE.y) - 0.5).round() as i32,
    }
}

/// Convertit une coordition monde (pixels) en coordition de chunk.
pub fn absolute_coord_to_chunk_coord(absolute_coord: AbsoluteCoordinates) -> ChunkCoordinates {
    ChunkCoordinates {
        x: (absolute_coord.x as f32 / (CHUNK_SIZE.x as f32 * TILE_SIZE.x)).floor() as i32,
        y: ((-absolute_coord.y as f32) / (CHUNK_SIZE.y as f32 * TILE_SIZE.y)).floor() as i32,
    }
}

pub fn chunk_coord_to_tile_coord(chunk_coord: ChunkCoordinates) -> TileCoordinates {
    TileCoordinates {
        x: chunk_coord.x * CHUNK_SIZE.x as i32,
        y: chunk_coord.y * CHUNK_SIZE.y as i32,
    }
}

pub fn tile_coord_to_chunk_coord(tile_coord: TileCoordinates) -> ChunkCoordinates {
    ChunkCoordinates {
        // x: tile_coord.x / CHUNK_SIZE.x as i32,
        // y: tile_coord.y / CHUNK_SIZE.y as i32,
        x: tile_coord.x.div_euclid(CHUNK_SIZE.x as i32),
        y: tile_coord.y.div_euclid(CHUNK_SIZE.y as i32),
    }
}

pub fn coord_to_chunk_coord(coord: Coordinates) -> ChunkCoordinates {
    ChunkCoordinates {
        x: (coord.x / CHUNK_SIZE.x as f32).floor() as i32,
        y: (coord.y / CHUNK_SIZE.y as f32).floor() as i32,
    }
}
// ==========================================
