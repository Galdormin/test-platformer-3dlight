use bevy::camera::visibility::NoFrustumCulling;
use bevy::prelude::*;
use std::{f32::consts::PI, marker::PhantomData};

use bevy_ecs_ldtk::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_sprite3d::prelude::*;

#[derive(Debug)]
pub struct Tilemap3dPlugin<L>(PhantomData<L>);

impl<T> Tilemap3dPlugin<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<L: Layer3d + Send + Sync + 'static> Plugin for Tilemap3dPlugin<L> {
    fn build(&self, app: &mut App) {
        app.add_plugins(Sprite3dPlugin).add_systems(
            Update,
            (
                update_level_on_spawn::<L>,
                add_sprite_on_tile_spawn::<L>,
                add_background_on_level_spawn::<L>,
            ),
        );
    }
}

pub trait Layer3d: Sized {
    const PIXEL_PER_METER: f32;

    /// Convert from layer name to Layer3d
    fn try_from_layer_name(name: impl AsRef<str>) -> Result<Self>;

    /// Return the z axis position of the layer
    fn depth(&self) -> f32;
}

trait MoveToLayer {
    fn move_to_layer(&mut self, layer: impl Layer3d);
}

impl MoveToLayer for Transform {
    fn move_to_layer(&mut self, layer: impl Layer3d) {
        self.translation.z = layer.depth();
    }
}

trait ConvertTo3d {
    fn convert_to_3d(&self, pixel_per_meter: f32) -> Vec3;
}

impl ConvertTo3d for Vec3 {
    fn convert_to_3d(&self, pixel_per_meter: f32) -> Vec3 {
        Vec3::new(self.x / pixel_per_meter, self.y / pixel_per_meter, self.z)
    }
}

fn update_level_on_spawn<L: Layer3d>(
    level_query: Query<(&Children, &mut Transform), Added<LevelIid>>,
    mut tilemap_query: Query<(&Name, &mut Transform), (With<TilemapSize>, Without<LevelIid>)>,
) {
    for (level_children, mut level_transform) in level_query {
        level_transform.translation = level_transform
            .translation
            .convert_to_3d(L::PIXEL_PER_METER);

        for child in level_children.iter() {
            if let Ok((tilemap_name, mut tilemap_transform)) = tilemap_query.get_mut(child) {
                tilemap_transform.translation = tilemap_transform
                    .translation
                    .convert_to_3d(L::PIXEL_PER_METER);

                if let Ok(layer) = L::try_from_layer_name(tilemap_name.as_str()) {
                    tilemap_transform.move_to_layer(layer);
                }
            }
        }
    }
}

fn add_sprite_on_tile_spawn<L: Layer3d>(
    mut commands: Commands,
    image_assets: Res<Assets<Image>>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    tiles: Query<(Entity, &TilemapId, &TileTextureIndex, &mut Transform), Added<TileTextureIndex>>,
    tilemaps: Query<(&TilemapTexture, &TilemapTileSize)>,
) {
    for (entity, tilemap_id, texture_index, mut transform) in tiles {
        let Ok((tm_texture, tm_tile_size)) = tilemaps.get(tilemap_id.0) else {
            error!("Tile {entity} has no TileMap.");
            continue;
        };

        let Some(image) = image_assets.get(tm_texture.image_handle()) else {
            error!("Tile {entity} texture image is not yet loaded.");
            continue;
        };

        let tile_size = UVec2::new(tm_tile_size.x as u32, tm_tile_size.y as u32);
        let atlas_layout = TextureAtlasLayout::from_grid(
            tile_size,
            image.size().x / tile_size.x,
            image.size().y / tile_size.y,
            None,
            None,
        );

        let atlas = TextureAtlas::from(texture_atlas_layouts.add(atlas_layout))
            .with_index(texture_index.0 as usize);

        transform.translation = transform.translation.convert_to_3d(L::PIXEL_PER_METER);
        commands.entity(entity).insert((
            Sprite {
                image: tm_texture.image_handle().clone(),
                texture_atlas: Some(atlas),
                ..default()
            },
            Sprite3d {
                pixels_per_metre: L::PIXEL_PER_METER,
                alpha_mode: AlphaMode::Mask(0.5),
                unlit: false,
                ..default()
            },
        ));
    }
}

fn add_background_on_level_spawn<L: Layer3d>(
    mut commands: Commands,
    image_assets: Res<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    levels: Query<&Children, Added<LevelIid>>,
    mut background_sprites: Query<(Entity, &Sprite, &mut Transform), Without<TileTextureIndex>>,
) {
    for level_children in &levels {
        for child in level_children.iter() {
            let Ok((entity, sprite, mut sprite_transform)) = background_sprites.get_mut(child)
            else {
                continue;
            };

            let Some(background_size_px) = sprite.custom_size.or_else(|| {
                image_assets
                    .get(&sprite.image)
                    .map(|image| image.size().as_vec2())
            }) else {
                continue;
            };

            sprite_transform.translation = sprite_transform
                .translation
                .convert_to_3d(L::PIXEL_PER_METER);

            // Background is texture
            if sprite.texture_atlas.is_some() {
                commands.entity(entity).insert(Sprite3d {
                    pixels_per_metre: L::PIXEL_PER_METER,
                    alpha_mode: AlphaMode::Mask(0.5),
                    unlit: false,
                    ..default()
                });
                continue;
            }

            // Background is color
            let background_size_world = background_size_px / L::PIXEL_PER_METER;
            let mut background_transform = Transform::from_xyz(
                sprite_transform.translation.x,
                sprite_transform.translation.y,
                0.,
            );
            background_transform.rotate_x(PI / 2.0);

            commands.entity(entity).insert((
                Mesh3d(
                    meshes.add(
                        Plane3d::default()
                            .mesh()
                            .size(background_size_world.x, background_size_world.y),
                    ),
                ),
                MeshMaterial3d(materials.add(sprite.color)),
                background_transform,
                NoFrustumCulling,
            ));
        }
    }
}
