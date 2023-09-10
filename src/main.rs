use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use bevy::{math::Vec4Swizzles, window::PrimaryWindow};

const MAP_RAD: u32 = 5;
const MAP_SIZE: TilemapSize = TilemapSize {
    x: MAP_RAD * 2,
    y: MAP_RAD * 2,
};
const TILE_SIZE: TilemapTileSize = TilemapTileSize { x: 58.0, y: 50.0 };
const GRID_SIZE: TilemapGridSize = TilemapGridSize { x: 58.0, y: 50.0 };
const BOARD_CENTER: TilePos = TilePos {
    x: MAP_RAD,
    y: MAP_RAD,
};

#[derive(Deref, Resource)]
struct TileHandle(Handle<Image>);

impl FromWorld for TileHandle {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self(asset_server.load("tiles.png"))
    }
}

#[derive(Component)]
struct Tilemap;

fn spawn_tiles(mut commands: Commands, tile_handle: Res<TileHandle>) {
    commands.spawn(Camera2dBundle::default());
    let tilemap_entity = commands.spawn(Tilemap).id();
    let mut tile_storage = TileStorage::empty(MAP_SIZE);

    fill_tilemap_hexagon(
        TileTextureIndex(0),
        BOARD_CENTER,
        MAP_RAD,
        HexCoordSystem::Column,
        TilemapId(tilemap_entity),
        &mut commands,
        &mut tile_storage,
    );

    let map_type = TilemapType::Hexagon(HexCoordSystem::Column);
    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size: GRID_SIZE,
        map_type,
        size: MAP_SIZE,
        storage: tile_storage,
        texture: TilemapTexture::Single(tile_handle.clone()),
        transform: get_tilemap_center_transform(&MAP_SIZE, &GRID_SIZE, &map_type, 0.0),
        tile_size: TILE_SIZE,
        ..default()
    });
}

fn highlight_on_click(
    mouse: Res<Input<MouseButton>>,
    mut tiles_q: Query<(&TilePos, &mut TileColor)>,
    tilemap_q: Query<&Transform, With<Tilemap>>,
    windows_q: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&GlobalTransform, &Camera)>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let tilemap_transform = tilemap_q.single();
    let (camera_transform, camera) = camera_q.single();

    let Some(cursor_tile_pos) = windows_q.single().cursor_position().and_then(|cursor_pos| {
        let cursor_pos = camera.viewport_to_world_2d(camera_transform, cursor_pos)?;
        let cursor_pos = Vec4::from((cursor_pos, 0.0, 1.0));
        let cursor_in_map_pos = (tilemap_transform.compute_matrix().inverse() * cursor_pos).xy();
        TilePos::from_world_pos(
            &cursor_in_map_pos,
            &MAP_SIZE,
            &GRID_SIZE,
            &TilemapType::Hexagon(HexCoordSystem::Column),
        )
    }) else {
        return;
    };

    if let Some((_, mut col)) = tiles_q.iter_mut().find(|(pos, _)| **pos == cursor_tile_pos) {
        *col = Color::DARK_GRAY.into()
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(TilemapPlugin)
        .init_resource::<TileHandle>()
        .add_systems(Startup, spawn_tiles)
        .add_systems(Update, highlight_on_click)
        .run();
}
