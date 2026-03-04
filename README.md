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

## How to Run

```bash
cargo run
```

- `W` / `S`: Move light on Y axis
- `A` / `D`: Move light on X axis
- `Q` / `E`: Move light up/down on Z axis

The controls move the scene `PointLight` in real time.


## Credit

Assets from penubsmic ([https://penusbmic.itch.io/the-dark-series-ancient-caves-tileset](https://penusbmic.itch.io/the-dark-series-ancient-caves-tileset))
