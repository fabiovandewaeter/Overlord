use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use event_listener::Event;
use futures_lite::Future;
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU32, Ordering},
};
pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<LoadingState>()
            .add_systems(Startup, setup_loading)
            .add_systems(
                Update,
                (convert_tileset_to_array, check_loading_complete)
                    .chain()
                    .run_if(in_state(LoadingState::Loading)),
            )
            .add_systems(OnExit(LoadingState::Loading), cleanup_loading);
    }
}

fn setup_loading(mut commands: Commands, asset_server: Res<AssetServer>) {
    let (barrier, guard) = TilesetBarrier::new();

    // Charge l'image avec la barrière
    let tileset_handle: Handle<Image> =
        asset_server.load_acquire("textures/array_texture.png", guard.clone());

    // Stocke le handle pour pouvoir le modifier plus tard
    commands.insert_resource(TilesetHandle(tileset_handle));

    let future = barrier.wait_async();
    commands.insert_resource(barrier);

    let loading_state = Arc::new(AtomicBool::new(false));
    commands.insert_resource(AsyncLoadingState(loading_state.clone()));

    AsyncComputeTaskPool::get()
        .spawn(async move {
            future.await;
            loading_state.store(true, Ordering::Release);
        })
        .detach();

    commands.spawn((
        Text::new("Loading tileset..."),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(12.0),
            top: Val::Px(12.0),
            ..default()
        },
        LoadingEntity,
    ));
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, States, Default)]
pub enum LoadingState {
    #[default]
    Loading,
    Ready,
}

#[derive(Component)]
pub struct LoadingEntity;

#[derive(Resource)]
pub struct TilesetBarrier(Arc<TilesetBarrierInner>);

pub struct TilesetBarrierGuard(Arc<TilesetBarrierInner>);

pub struct TilesetBarrierInner {
    count: AtomicU32,
    notify: Event,
}

#[derive(Resource)]
pub struct AsyncLoadingState(Arc<AtomicBool>);

// Resource pour stocker le handle de la tileset
#[derive(Resource)]
pub struct TilesetHandle(pub Handle<Image>);

impl TilesetBarrier {
    pub fn new() -> (TilesetBarrier, TilesetBarrierGuard) {
        let inner = Arc::new(TilesetBarrierInner {
            count: AtomicU32::new(1),
            notify: Event::new(),
        });
        (TilesetBarrier(inner.clone()), TilesetBarrierGuard(inner))
    }

    pub fn is_ready(&self) -> bool {
        self.0.count.load(Ordering::Acquire) == 0
    }

    pub fn wait_async(&self) -> impl Future<Output = ()> + 'static {
        let shared = self.0.clone();
        async move {
            loop {
                let listener = shared.notify.listen();
                if shared.count.load(Ordering::Acquire) == 0 {
                    return;
                }
                listener.await;
            }
        }
    }
}

impl Clone for TilesetBarrierGuard {
    fn clone(&self) -> Self {
        self.0.count.fetch_add(1, Ordering::AcqRel);
        TilesetBarrierGuard(self.0.clone())
    }
}

impl Drop for TilesetBarrierGuard {
    fn drop(&mut self) {
        let prev = self.0.count.fetch_sub(1, Ordering::AcqRel);
        if prev == 1 {
            self.0.notify.notify(usize::MAX);
        }
    }
}

// Convertit l'image en texture array dès qu'elle est chargée
fn convert_tileset_to_array(
    tileset_handle: Res<TilesetHandle>,
    mut images: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
) {
    // Vérifie si l'image est chargée
    if asset_server.is_loaded_with_dependencies(&tileset_handle.0) {
        if let Some(image) = images.get_mut(&tileset_handle.0) {
            let width = image.width();
            let height = image.height();

            // Calcule combien de layers sont empilées dans l'image
            // Pour une texture array, la hauteur doit être un multiple de la largeur
            let num_layers = height / width;

            println!(
                "Image size: {}x{}, detected {} layers",
                width, height, num_layers
            );

            if num_layers > 1 && height % width == 0 {
                // L'image a plusieurs layers empilées, on la convertit
                image.reinterpret_stacked_2d_as_array(num_layers);
                println!(
                    "Tileset converted to array texture with {} layers!",
                    num_layers
                );
            } else {
                // L'image n'est pas stackée, on ne fait rien
                // Bevy peut l'utiliser comme texture 2D normale
                println!("Tileset is a regular 2D texture, no conversion needed");
            }
        }
    }
}

fn check_loading_complete(
    async_state: Res<AsyncLoadingState>,
    mut next_state: ResMut<NextState<LoadingState>>,
    tileset_handle: Res<TilesetHandle>,
    images: Res<Assets<Image>>,
) {
    // Vérifie que l'image est chargée (pas besoin de vérifier la conversion)
    if async_state.0.load(Ordering::Acquire) {
        if images.get(&tileset_handle.0).is_some() {
            println!("Loading complete!");
            next_state.set(LoadingState::Ready);
        }
    }
}

fn cleanup_loading(mut commands: Commands, loading_entities: Query<Entity, With<LoadingEntity>>) {
    for entity in loading_entities.iter() {
        commands.entity(entity).despawn();
    }

    commands.remove_resource::<TilesetBarrier>();
    commands.remove_resource::<AsyncLoadingState>();
    // On garde TilesetHandle pour pouvoir l'utiliser plus tard si nécessaire
}
