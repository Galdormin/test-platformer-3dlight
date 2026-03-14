use std::f32::consts::PI;

use bevy::{camera::ScalingMode, prelude::*, window::WindowResolution};

use bevy_asset_loader::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

use crate::tilemap3d::{Layer3d, Tilemap3dPlugin};

mod tilemap3d;

// const GRID_SIZE: f32 = 16.;
// const LEVEL_SIZE: Vec2 = Vec2::new(512., 288.);

const GRID_SIZE: f32 = 1.;
const LEVEL_SIZE: Vec2 = Vec2::new(32., 18.);

fn main() {
    App::new()
        .add_plugins(
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
        .add_plugins((Tilemap3dPlugin::<LayerDepth>::default(), LdtkPlugin))
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
        .add_systems(OnEnter(GameState::Ready), setup)
        .add_systems(Update, move_light)
        .run();
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
enum LayerDepth {
    Front,
    Middle,
    Back,
}

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

fn setup(
    mut commands: Commands,
    assets: Res<AllAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // camera
    commands.spawn((
        Camera3d::default(),
        Projection::from(OrthographicProjection {
            // 6 world units per pixel of window height.
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

    // plane
    let mut transform = Transform::from_xyz(LEVEL_SIZE.x / 2., -LEVEL_SIZE.y / 2., 0.5);
    transform.rotate_x(PI / 2.);

    commands.spawn((
        Name::new("Background"),
        Mesh3d(meshes.add(Plane3d::default().mesh().size(LEVEL_SIZE.x, LEVEL_SIZE.y))),
        MeshMaterial3d(materials.add(Color::srgb(0.2, 0.2, 0.2))),
        transform,
    ));

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
