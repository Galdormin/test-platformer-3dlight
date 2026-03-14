# Test Platformer 2D/3D

Small Bevy prototype showing how to render an LDtk 2D tilemap as 3D sprites, then light it with real 3D lighting.

## What This Project Demonstrates

- Loading an `.ldtk` world with `bevy_ecs_ldtk`
- Loading assets through `bevy_asset_loader`
- Converting tile entities into `Sprite3d` entities (`bevy_sprite3d`)
- Using an orthographic `Camera3d` for a 2D gameplay view with 3D lighting
- Assigning explicit Z depth by LDtk layer names (`Back`, `Middle`/`Wall`, `Front`)
- Inspecting ECS world state with `bevy-inspector-egui`

## How It Works

Each spawned tile gets a `Sprite` + `Sprite3d` component so the tilemap is rendered in 3D space. The default renderer from `bevy_ecs_tilemap` don't work in 3D.
The coordinate must be change from pixel to meter, therefore x and y coordinates are divided by 16.


## How to run

You can compare the 3d rendering with the 2d rendering based on the features `2d` and `3d`.

Run in 3D mode:

```bash
cargo run
```

Run in 2D mode (uses bevy_ecs_tilemap renderer):

```bash
cargo run --no-default-features --features 2d
```


## Tilemap3dPlugin Quick Overview

`Tilemap3dPlugin` converts spawned LDtk tile entities into `Sprite + Sprite3d`,
converts tilemap coordinates from pixels to meters, and applies Z depth from LDtk layer names.

For 16x16 pixel art, two common setups are useful:

- `16 pixels_per_metre`: world scale closer to a classic 3D setup
- `1 pixel_per_metre`: keeps world dimensions close to the 2D mode

When changing world scale, light settings must also be scaled to keep a similar visual result:

- Distance/range scales by `d`
- Intensity scales by `d^2`

Where `d` is the scene scale factor between both setups.
Example when switching from `1 px/m` to `16 px/m`: `d = 16`, so use roughly `range x16` and `intensity x256`.

- `W` / `S`: Move light on Y axis
- `A` / `D`: Move light on X axis
- `Q` / `E`: Move light up/down on Z axis

The controls move the scene `PointLight` in real time.


## Credit

Assets from penubsmic ([https://penusbmic.itch.io/the-dark-series-ancient-caves-tileset](https://penusbmic.itch.io/the-dark-series-ancient-caves-tileset))
