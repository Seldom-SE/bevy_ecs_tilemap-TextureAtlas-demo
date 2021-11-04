use bevy::{asset::LoadState, prelude::*, sprite::TextureAtlasBuilder};
use bevy_ecs_tilemap::prelude::*;

mod helpers;

fn startup(
    mut commands: Commands,
    sprite_handles: Res<SpriteHandles>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut textures: ResMut<Assets<Texture>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut map_query: MapQuery,
) {
    let mut texture_atlas_builder = TextureAtlasBuilder::default();
    for handle in sprite_handles.handles.iter() {
        let texture = textures.get(handle).unwrap();
        texture_atlas_builder.add_texture(handle.clone_weak().typed::<Texture>(), texture);
    }

    let texture_atlas = texture_atlas_builder.finish(&mut textures).unwrap();
    let texture_atlas_texture_handle = texture_atlas.texture.clone();
    let texture_atlas_texture = textures.get(texture_atlas_texture_handle.clone()).unwrap();
    let lime_tile_handle = asset_server.get_handle("textures/lime.png");
    let lime_tile_index = texture_atlas.get_texture_index(&lime_tile_handle).unwrap();
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    println!("Lime tile index: {}", lime_tile_index);

    // set up a scene to display our texture atlas
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let material_handle =
        materials.add(ColorMaterial::texture(texture_atlas_texture_handle.clone()));

    // Create map entity and component:
    let map_entity = commands.spawn().id();
    let mut map = Map::new(0u16, map_entity);

    // Creates a new layer builder with a layer entity.
    let (mut layer_builder, _) = LayerBuilder::new(
        &mut commands,
        LayerSettings::new(
            MapSize(2, 2),
            ChunkSize(8, 8),
            TileSize(16.0, 16.0),
            TextureSize(
                texture_atlas_texture.size.width as f32,
                texture_atlas_texture.size.height as f32,
            ),
        ),
        0u16,
        0u16,
        None,
    );

    layer_builder.set_all(TileBundle {
        tile: Tile {
            texture_index: lime_tile_index as u16,
            ..Tile::default()
        },
        ..TileBundle::default()
    });

    // Builds the layer.
    // Note: Once this is called you can no longer edit the layer until a hard sync in bevy.
    let layer_entity = map_query.build_layer(&mut commands, layer_builder, material_handle);

    // Required to keep track of layers for a map internally.
    map.add_layer(&mut commands, 0u16, layer_entity);

    // Spawn Map
    // Required in order to use map_query to retrieve layers/tiles.
    commands
        .entity(map_entity)
        .insert(map)
        .insert(Transform::from_xyz(-128.0, -128.0, 0.0))
        .insert(GlobalTransform::default());

    // draw a sprite from the atlas
    commands.spawn_bundle(SpriteSheetBundle {
        transform: Transform {
            translation: Vec3::new(150.0, 0.0, 0.0),
            scale: Vec3::splat(4.0),
            ..Default::default()
        },
        sprite: TextureAtlasSprite::new(lime_tile_index as u32),
        texture_atlas: texture_atlas_handle,
        ..Default::default()
    });
    // draw the atlas itself
    commands.spawn_bundle(SpriteBundle {
        material: materials.add(texture_atlas_texture_handle.into()),
        transform: Transform::from_xyz(-300.0, 0.0, 0.0),
        ..Default::default()
    });
}

/// In this example we generate a new texture atlas (sprite sheet) from a folder containing
/// individual sprites
fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    App::build()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from("Map Example"),
            ..Default::default()
        })
        .init_resource::<SpriteHandles>()
        .add_plugins(DefaultPlugins)
        .add_plugin(TilemapPlugin)
        .add_state(AppState::Setup)
        .add_system_set(SystemSet::on_enter(AppState::Setup).with_system(load_textures.system()))
        .add_system_set(SystemSet::on_update(AppState::Setup).with_system(check_textures.system()))
        .add_system_set(SystemSet::on_enter(AppState::Finished).with_system(startup.system()))
        .add_system_set(
            SystemSet::on_update(AppState::Finished)
                .with_system(helpers::camera::movement.system())
                .with_system(helpers::texture::set_texture_filters_to_nearest.system()),
        )
        .run();
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum AppState {
    Setup,
    Finished,
}

#[derive(Default)]
struct SpriteHandles {
    handles: Vec<HandleUntyped>,
}

fn load_textures(mut sprite_handles: ResMut<SpriteHandles>, asset_server: Res<AssetServer>) {
    sprite_handles.handles = asset_server.load_folder("textures").unwrap();
}

fn check_textures(
    mut state: ResMut<State<AppState>>,
    sprite_handles: ResMut<SpriteHandles>,
    asset_server: Res<AssetServer>,
) {
    if let LoadState::Loaded =
        asset_server.get_group_load_state(sprite_handles.handles.iter().map(|handle| handle.id))
    {
        state.set(AppState::Finished).unwrap();
    }
}
