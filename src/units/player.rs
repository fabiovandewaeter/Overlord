use std::collections::VecDeque;

use bevy::{prelude::*, sprite_render::TilemapChunk};
use pathfinding::{grid, prelude::astar};

use crate::{
    direction::Direction,
    map::{
        CurrentMapId, MultiMapManager, StructureLayerManager,
        coordinates::{
            AbsoluteCoordinates, GridPosition, TileCoordinates, absolute_coord_to_tile_coord,
        },
        structure::Structure,
    },
    physics::movement::{DesiredMovement, MovementAccumulator, Passable},
    units::{UnitBundle, pathfinding::RecalculateFlowField},
};

#[derive(Component, Default, Debug)]
pub struct PlayerPath {
    waypoints: VecDeque<TileCoordinates>,
}
impl PlayerPath {
    pub fn clear(&mut self) {
        self.waypoints.clear();
    }

    pub fn next_tile(&self) -> Option<TileCoordinates> {
        self.waypoints.front().copied()
    }

    pub fn pop_front(&mut self) -> Option<TileCoordinates> {
        self.waypoints.pop_front()
    }
}

#[derive(Component)]
pub struct Player;
impl Player {
    pub const PATH_PNG: &'static str = "units/player.png";
}
#[derive(Bundle)]
pub struct PlayerBundle {
    pub base: UnitBundle,
    pub path: PlayerPath,
    pub player: Player,
}
impl PlayerBundle {
    pub fn new(base: UnitBundle) -> Self {
        Self {
            base,
            path: PlayerPath::default(),
            player: Player,
        }
    }
}

pub fn player_control_system(
    mut unit_query: Query<
        (
            &GridPosition,
            &CurrentMapId,
            &mut MovementAccumulator,
            &mut DesiredMovement,
            &mut Direction,
            &mut PlayerPath,
        ),
        With<Player>,
    >,
    input: Res<ButtonInput<KeyCode>>,
    mut message_recalculate: MessageWriter<RecalculateFlowField>,
) {
    let Ok((
        grid_pos,
        current_map_id,
        movement_accumulator,
        mut desired_movement,
        mut direction,
        mut player_path,
    )) = unit_query.single_mut()
    else {
        return;
    };

    if movement_accumulator.0 < MovementAccumulator::MOVEMENT_COST {
        return;
    }

    // if the player reached next waypoint of pathfinding, removes it
    if let Some(target) = player_path.next_tile() {
        if target == grid_pos.0 {
            player_path.pop_front();
        }
    }

    // keyboard inputs
    let mut delta = IVec2::ZERO;
    if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp) {
        delta.y += 1;
        *direction = Direction::North;
    }
    if input.pressed(KeyCode::KeyS) || input.pressed(KeyCode::ArrowDown) {
        delta.y -= 1;
        *direction = Direction::South;
    }
    if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) {
        delta.x -= 1;
        *direction = Direction::West;
    }
    if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) {
        delta.x += 1;
        *direction = Direction::East;
    }

    if delta != IVec2::ZERO {
        // movement_accumulator.0 = 0.0;
        // movement_accumulator.0 -= MovementAccumulator::MOVEMENT_COST;

        // moving with keyboard stop pathfinding
        player_path.clear();

        desired_movement.tile = Some(TileCoordinates {
            x: grid_pos.0.x + delta.x,
            y: grid_pos.0.y - delta.y,
        });
        desired_movement.map_id = Some(current_map_id.0);

        // TODO: change to put that after the collisions check
        message_recalculate.write_default();
    } else if let Some(next_tile) = player_path.next_tile() {
        // mouse inputs

        let dx = next_tile.x - grid_pos.0.x;
        let dy = next_tile.y - grid_pos.0.y;

        // stops if the waypoint is too far
        if dx.abs() > 1 || dy.abs() > 1 {
            player_path.clear();
            desired_movement.tile = None;

            return;
        }

        if dx > 0 {
            *direction = Direction::East;
        } else if dx < 0 {
            *direction = Direction::West;
        } else if dy > 0 {
            *direction = Direction::South;
        } else if dy < 0 {
            *direction = Direction::North;
        }

        desired_movement.tile = Some(next_tile);
        desired_movement.map_id = Some(current_map_id.0);

        // player_path.pop_front();

        message_recalculate.write_default();
    } else {
        desired_movement.tile = None;
        desired_movement.map_id = None;
    }
}

pub fn player_mouse_input_system(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut player_query: Query<(&GridPosition, &CurrentMapId, &mut PlayerPath), With<Player>>,
    multi_map_manager: Res<MultiMapManager>,
    structure_query: Query<Has<Passable>, With<Structure>>,
    chunk_query: Query<&StructureLayerManager, With<TilemapChunk>>,
) {
    if !buttons.just_pressed(MouseButton::Right) {
        return;
    }

    let (camera, camera_transform) = camera_q.single().unwrap();
    let window = windows.single().unwrap();

    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| Some(camera.viewport_to_world(camera_transform, cursor)))
        .map(|ray| ray.unwrap().origin.truncate())
    {
        let Ok((player_pos, map_id, mut player_path)) = player_query.single_mut() else {
            panic!()
        };

        // Convertir world_pos en TileCoordinates (adaptation de ta logique existante)
        let absolute_coords = AbsoluteCoordinates {
            x: world_position.x,
            y: world_position.y,
        };
        let target_tile = absolute_coord_to_tile_coord(absolute_coords);

        let start_tile = player_pos.0;

        // Si on clique sur soi-même, on arrête le mouvement
        if start_tile == target_tile {
            player_path.waypoints.clear();
            return;
        }

        let Some(map_manager) = multi_map_manager.maps.get(&map_id.0) else {
            return;
        };

        // 3. Calcul A* (A-Star)
        // C'est similaire à ton FlowField mais ciblé vers une destination
        let result = astar(
            &start_tile,
            |&tile| {
                let mut neighbors = Vec::with_capacity(8);
                for y in -1..=1 {
                    for x in -1..=1 {
                        if x == 0 && y == 0 {
                            continue;
                        }

                        let neighbor = TileCoordinates {
                            x: tile.x + x,
                            y: tile.y + y,
                        };

                        // Réutilisation de ta logique de collision existante
                        if map_manager.can_move_between(
                            tile,
                            neighbor,
                            &structure_query,
                            &chunk_query,
                        ) {
                            // Coût: 10 pour cardinal, 14 pour diagonale (approximation de sqrt(2)*10)
                            let cost = if x == 0 || y == 0 { 10 } else { 14 };
                            neighbors.push((neighbor, cost));
                        }
                    }
                }
                neighbors
            },
            |&tile| {
                // ((tile.x - target_tile.x).abs() + (tile.y - target_tile.y).abs()) * 10
                // CORRECTION ICI : Heuristique Octile
                let dx = (tile.x - target_tile.x).abs();
                let dy = (tile.y - target_tile.y).abs();

                // On prend le plus petit des deux deltas pour les diagonales (coût 14)
                // Et on complète la différence en ligne droite (coût 10)
                let min = dx.min(dy);
                let max = dx.max(dy);

                // Formule : 14 * min + 10 * (max - min)
                // Simplifié : 14 * min + 10 * max - 10 * min
                // Simplifié : 4 * min + 10 * max
                14 * min + 10 * (max - min)
            },
            |&tile| tile == target_tile,
        );

        // 4. Mettre à jour le chemin du joueur
        if let Some((path, _cost)) = result {
            // On saute le premier élément car c'est la position actuelle du joueur
            player_path.waypoints = path.into_iter().skip(1).collect();
        }
    }
}
