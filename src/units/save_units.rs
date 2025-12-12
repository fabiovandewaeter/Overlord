// save
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct SavedUnit {
//     pub name: Name,
//     pub transform: Transform,
//     pub direction: Direction,
//     pub speed: Speed,
//     pub linear_velocity: LinearVelocity,
//     pub is_player: bool,
//     pub inventory: Option<Inventory>,
// }
// #[derive(Serialize, Deserialize)]
// pub struct MapSaveData {
//     pub map_name: String,
//     pub units: Vec<SavedUnit>,
//     pub version: f32,
// }

// pub fn save_units_to_file_system(
//     units_query: Query<
//         (
//             &Name,
//             &Transform,
//             &Direction,
//             &Speed,
//             &LinearVelocity,
//             Has<Player>,
//             Option<&Inventory>,
//         ),
//         With<Unit>,
//     >,
//     map_name: String,
// ) {
//     let mut saved_units = Vec::new();

//     for (name, transform, direction, speed, linear_velocity, is_player, inventory) in
//         units_query.iter()
//     {
//         saved_units.push(SavedUnit {
//             name: name.clone(),
//             transform: *transform,
//             direction: *direction,
//             speed: *speed,
//             linear_velocity: *linear_velocity,
//             is_player,
//             inventory: inventory.cloned(),
//         })
//     }

//     let save_data = MapSaveData {
//         map_name: map_name.clone(),
//         units: saved_units,
//         version: CURRENT_SAVE_VERSION,
//     };

//     match serde_json::to_string_pretty(&save_data) {
//         Ok(json) => {
//             if let Err(e) = fs::write(format!("{}/{}.json", PATH_SAVES, map_name), json) {
//                 error!("Erreur lors de la sauvegarde: {}", e);
//             } else {
//                 info!("Sauvegardé {} unités", save_data.units.len());
//             }
//         }
//         Err(e) => error!("Erreur de sérialisation: {}", e),
//     }
// }

// pub fn load_units_from_file_system(
//     commands: &mut Commands,
//     asset_server: &Res<AssetServer>,
//     map_name: String,
// ) {
//     let path = format!("{}/{}.json", PATH_SAVES, map_name);
//     let save_path = Path::new(&path);

//     if !save_path.exists() {
//         panic!("Can't find {}", path);
//     }

//     match fs::read_to_string(save_path) {
//         Ok(json_content) => match serde_json::from_str::<MapSaveData>(&json_content) {
//             Ok(save_data) => {
//                 let texture_handle = asset_server.load("default.png");

//                 for SavedUnit {
//                     name,
//                     transform,
//                     direction,
//                     speed,
//                     linear_velocity,
//                     is_player,
//                     inventory,
//                 } in save_data.units
//                 {
//                     let bundle = UnitBundle::new(name, transform, speed);
//                     let mut entity = commands.spawn((
//                         bundle,
//                         direction,
//                         linear_velocity,
//                         Sprite::from_image(texture_handle.clone()),
//                     ));

//                     if is_player {
//                         entity.insert(Player);
//                     }

//                     if let Some(some_inventory) = inventory {
//                         entity.insert(some_inventory);
//                     }
//                 }
//             }
//             Err(e) => error!("Error deserializing: {}", e),
//         },
//         Err(e) => error!("Error reading file: {}", e),
//     }
// }
