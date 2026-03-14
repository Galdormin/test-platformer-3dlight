use bevy::{camera::ScalingMode, prelude::*, window::WindowResolution};

use bevy_asset_loader::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

#[cfg(feature = "3d")]
use crate::tilemap3d::{Layer3d, Tilemap3dPlugin};

#[cfg(feature = "3d")]
mod tilemap3d;

#[cfg(all(feature = "2d", feature = "3d"))]
compile_error!("Features '2d' and '3d' are mutually exclusive. Enable only one of them.");

#[cfg(not(any(feature = "2d", feature = "3d")))]
compile_error!("Enable one rendering mode feature: '2d' or '3d'.");

#[cfg(feature = "2d")]
const GRID_SIZE: f32 = 16.;
#[cfg(feature = "2d")]
const LEVEL_SIZE: Vec2 = Vec2::new(512., 288.);

#[cfg(feature = "3d")]
const GRID_SIZE: f32 = 1.;
#[cfg(feature = "3d")]
const LEVEL_SIZE: Vec2 = Vec2::new(32., 18.);

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Window {
                    fit_canvas_to_parent: true,
                    resolution: WindowResolution::new(1024, 576),
                    ..default()
                }
                .into(),
                ..default()
            }),
    )
    .add_plugins(LdtkPlugin)
    .add_plugins((EguiPlugin::default(), WorldInspectorPlugin::new()))
    .init_state::<GameState>()
    .add_loading_state(
        LoadingState::new(GameState::Loading)
            .continue_to_state(GameState::Ready)
            .load_collection::<AllAssets>(),
    )
    .insert_resource(LevelSelection::index(0))
    .insert_resource(LdtkSettings {
        level_spawn_behavior: bevy_ecs_ldtk::LevelSpawnBehavior::UseWorldTranslation {
            load_level_neighbors: true,
        },
        ..default()
    })
    .add_systems(Update, move_camera);

    #[cfg(feature = "3d")]
    app.add_plugins(Tilemap3dPlugin::<LayerDepth>::new())
        .add_systems(OnEnter(GameState::Ready), setup_3d)
        .add_systems(Update, move_light);

    #[cfg(feature = "2d")]
    app.add_systems(OnEnter(GameState::Ready), setup_2d);

    app.run();
}

#[derive(States, Default, Debug, Hash, Eq, PartialEq, Clone, Copy)]
enum GameState {
    #[default]
    Loading,
    Ready,
}

#[derive(AssetCollection, Resource, Debug)]
struct AllAssets {
    #[asset(path = "test_world.ldtk")]
    world: Handle<LdtkProject>,
}

#[derive(Debug)]
#[cfg(feature = "3d")]
enum LayerDepth {
    Front,
    Middle,
    Back,
}

#[cfg(feature = "3d")]
impl Layer3d for LayerDepth {
    const PIXEL_PER_METER: f32 = 16.0 / GRID_SIZE;

    fn try_from_layer_name(name: impl AsRef<str>) -> Result<Self> {
        match name.as_ref() {
            "Back" => Ok(Self::Back),
            "Middle" | "Wall" => Ok(Self::Middle),
            "Front" => Ok(Self::Front),
            _ => Err(format!("Unsupported Layer type {}", name.as_ref()).into()),
        }
    }

    fn depth(&self) -> f32 {
        match self {
            LayerDepth::Front => 3.,
            LayerDepth::Middle => 2.,
            LayerDepth::Back => 1.,
        }
    }
}

#[cfg(feature = "2d")]
fn setup_2d(mut commands: Commands, assets: Res<AllAssets>) {
    commands.spawn((
        Camera2d,
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: LEVEL_SIZE.y,
            },
            ..OrthographicProjection::default_2d()
        }),
        Transform::from_xyz(LEVEL_SIZE.x / 2., -LEVEL_SIZE.y / 2., 20.0),
    ));

    commands.spawn(LdtkWorldBundle {
        ldtk_handle: assets.world.clone().into(),
        ..default()
    });
}

#[cfg(feature = "3d")]
fn setup_3d(mut commands: Commands, assets: Res<AllAssets>) {
    // camera
    commands.spawn((
        Camera3d::default(),
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: LEVEL_SIZE.y,
            },
            ..OrthographicProjection::default_2d()
        }),
        Transform::from_xyz(LEVEL_SIZE.x / 2., -LEVEL_SIZE.y / 2., 20.0),
    ));

    commands.insert_resource(GlobalAmbientLight {
        color: Color::WHITE.into(),
        brightness: 200.0,
        ..default()
    });

    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            range: 20. * GRID_SIZE,
            ..default()
        },
        Transform::from_xyz(LEVEL_SIZE.x / 2., -LEVEL_SIZE.y / 2., 2. * GRID_SIZE),
    ));

    commands.spawn(LdtkWorldBundle {
        ldtk_handle: assets.world.clone().into(),
        ..default()
    });
}

#[cfg(feature = "3d")]
fn move_light(
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<PointLight>>,
) {
    for mut transform in &mut query {
        let mut direction = Vec3::ZERO;
        if input.pressed(KeyCode::KeyW) {
            direction.y += 1.0;
        }
        if input.pressed(KeyCode::KeyS) {
            direction.y -= 1.0;
        }
        if input.pressed(KeyCode::KeyA) {
            direction.x -= 1.0;
        }
        if input.pressed(KeyCode::KeyD) {
            direction.x += 1.0;
        }
        if input.pressed(KeyCode::KeyQ) {
            direction.z += 0.1;
        }
        if input.pressed(KeyCode::KeyE) {
            direction.z -= 0.1;
        }

        transform.translation += time.delta_secs() * 5.0 * direction * GRID_SIZE;
    }
}

fn move_camera(
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, Or<(With<Camera2d>, With<Camera3d>)>>,
) {
    for mut transform in &mut query {
        let mut direction = Vec3::ZERO;
        if input.pressed(KeyCode::ArrowUp) {
            direction.y += 1.0;
        }
        if input.pressed(KeyCode::ArrowDown) {
            direction.y -= 1.0;
        }
        if input.pressed(KeyCode::ArrowLeft) {
            direction.x -= 1.0;
        }
        if input.pressed(KeyCode::ArrowRight) {
            direction.x += 1.0;
        }

        transform.translation += time.delta_secs() * 5.0 * direction * GRID_SIZE;
    }
}
