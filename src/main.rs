use std::f32::consts::PI;

use bevy::{camera::ScalingMode, prelude::*, window::WindowResolution};

use bevy_asset_loader::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use bevy_sprite3d::prelude::*;

const LEVEL_SIZE: Vec2 = Vec2::new(32., 18.);
const TILE_SIZE: f32 = 16.0;

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
        .add_plugins((Sprite3dPlugin, LdtkPlugin))
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
        .add_systems(
            Update,
            (
                update_level_on_spawn,
                move_light,
                add_sprite_on_tile_spawn.run_if(resource_exists::<AllAssets>),
            ),
        )
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

    #[asset(path = "tileset.png")]
    tileset: Handle<Image>,

    #[asset(texture_atlas_layout(tile_size_x = 16, tile_size_y = 16, columns = 24, rows = 24))]
    layout: Handle<TextureAtlasLayout>,
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
        Transform::from_xyz(LEVEL_SIZE.x / 2., -LEVEL_SIZE.y / 2., 2.0),
    ));

    commands.insert_resource(GlobalAmbientLight {
        color: Color::WHITE.into(),
        brightness: 2000.0,
        ..default()
    });

    // plane
    let mut transform = Transform::from_xyz(0., 0., 0.5);
    transform.rotate_x(PI / 2.);

    commands.spawn((
        Name::new("Background"),
        Mesh3d(meshes.add(Plane3d::default().mesh().size(100.0, 100.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.2, 0.2, 0.2))),
        transform,
    ));

    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(LEVEL_SIZE.x / 2., -LEVEL_SIZE.y / 2., 2.0),
    ));

    commands.spawn(LdtkWorldBundle {
        ldtk_handle: assets.world.clone().into(),
        ..default()
    });
}

fn update_level_on_spawn(
    level_query: Query<(&Children, &mut Transform), Added<LevelIid>>,
    mut tilemap_query: Query<(&Name, &mut Transform), (With<TilemapSize>, Without<LevelIid>)>,
) {
    for (level_children, mut level_transform) in level_query {
        level_transform.translation = level_transform.translation.reduce_to_3d();

        for child in level_children.iter() {
            if let Ok((tilemap_name, mut tilemap_transform)) = tilemap_query.get_mut(child) {
                tilemap_transform.translation = tilemap_transform.translation.reduce_to_3d();

                if let Ok(layer) = LayerDepth::try_from_layer_name(tilemap_name.as_str()) {
                    tilemap_transform.move_to_layer(layer);
                }
            }
        }
    }
}

fn add_sprite_on_tile_spawn(
    mut commands: Commands,
    assets: Res<AllAssets>,
    tiles: Query<(Entity, &TileTextureIndex, &mut Transform), Added<TileTextureIndex>>,
) {
    for (entity, texture_index, mut transform) in tiles {
        let atlas = TextureAtlas::from(assets.layout.clone()).with_index(texture_index.0 as usize);

        transform.translation = transform.translation.reduce_to_3d();

        commands.entity(entity).insert((
            Sprite {
                image: assets.tileset.clone(),
                texture_atlas: Some(atlas),
                ..default()
            },
            Sprite3d {
                pixels_per_metre: TILE_SIZE,
                alpha_mode: AlphaMode::Mask(0.5),
                unlit: false,
                ..default()
            },
        ));
    }
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

        transform.translation += time.delta_secs() * 10.0 * direction;
    }
}

#[derive(Debug)]
enum LayerDepth {
    Front,
    Middle,
    Back,
}

impl LayerDepth {
    fn try_from_layer_name(name: impl AsRef<str>) -> Result<Self, String> {
        match name.as_ref() {
            "Back" => Ok(Self::Back),
            "Middle" | "Wall" => Ok(Self::Middle),
            "Front" => Ok(Self::Front),
            _ => Err(format!("Unsupported Layer type {}", name.as_ref())),
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

trait MoveToLayer {
    fn move_to_layer(&mut self, layer: LayerDepth);
}

impl MoveToLayer for Transform {
    fn move_to_layer(&mut self, layer: LayerDepth) {
        self.translation.z = layer.depth();
    }
}

trait Plaformer3d {
    fn reduce_to_3d(&self) -> Vec3;
}

impl Plaformer3d for Vec3 {
    fn reduce_to_3d(&self) -> Vec3 {
        Vec3::new(self.x / TILE_SIZE, self.y / TILE_SIZE, self.z)
    }
}
