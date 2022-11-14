mod hex_rect;

use std::time::Duration;

use crate::hex_rect::{flat_hex_to_pixel, pixel_to_flat_hex};
use bevy::{
    input::{mouse::MouseButtonInput, ButtonState},
    prelude::*,
    render::camera::{RenderTarget, ScalingMode},
    sprite::MaterialMesh2dBundle,
    utils::HashMap,
};
use bevy_easings::{Ease, EaseFunction, EaseMethod, EasingType, EasingsPlugin};
use bevy_embedded_assets::EmbeddedAssetPlugin;
use hex_chess_lib::{Coord, Game};

const N: i32 = 5;
const RADIUS: f32 = 50.0;
const ATLAS_SIZE: (usize, usize) = (6, 2);

#[derive(Component)]
struct MainCamera;

type PieceSprites = HashMap<Coord, Entity>;

struct HexMaterials {
    mat_hover: Handle<ColorMaterial>,
    mat_selected: Handle<ColorMaterial>,
    mat_light: Handle<ColorMaterial>,
    mat_mid: Handle<ColorMaterial>,
    mat_dark: Handle<ColorMaterial>,
}

impl FromWorld for HexMaterials {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        Self {
            mat_hover: materials.add(ColorMaterial::from(Color::rgb(0.95, 0.51, 0.5))),
            mat_selected: materials.add(ColorMaterial::from(Color::rgb(0.54, 0.2, 0.2))),
            mat_light: materials.add(ColorMaterial::from(Color::rgb(1.0, 0.81, 0.62))),
            mat_mid: materials.add(ColorMaterial::from(Color::rgb(0.82, 0.55, 0.27))),
            mat_dark: materials.add(ColorMaterial::from(Color::rgb(0.91, 0.68, 0.44))),
        }
    }
}

#[derive(Debug)]
struct SelectedHex {
    hover: Option<Coord>,
    selected: Option<Coord>,
}

impl SelectedHex {
    fn new() -> Self {
        Self {
            hover: None,
            selected: None,
        }
    }
}

impl Default for SelectedHex {
    fn default() -> Self {
        Self::new()
    }
}

fn color_tiles(
    selected: Res<SelectedHex>,
    hex_materials: Res<HexMaterials>,
    mut tiles: Query<(&HexCoord, &mut Handle<ColorMaterial>)>,
) {
    for (HexCoord { coord }, mut material) in tiles.iter_mut() {
        *material = if selected.selected.is_some() && selected.selected.unwrap() == *coord {
            hex_materials.mat_selected.clone()
        } else if selected.hover.is_some() && selected.hover.unwrap() == *coord {
            hex_materials.mat_hover.clone()
        } else if coord.norm_squared() % 3 == 0 {
            hex_materials.mat_mid.clone()
        } else if (*coord - (1, 0).into()).norm_squared() % 3 == 0 {
            hex_materials.mat_dark.clone()
        } else {
            hex_materials.mat_light.clone()
        };
    }
}

#[derive(Debug, Clone, Component)]
struct HexCoord {
    coord: Coord,
}

#[derive(Debug, Clone, Copy, Component)]
struct Piece;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    hex_materials: Res<HexMaterials>,
    asset_server: Res<AssetServer>,
    mut pieces_atlases: ResMut<Assets<TextureAtlas>>,
    game: Res<Game>,
    mut piece_sprites: ResMut<PieceSprites>,
) {
    commands
        .spawn_bundle(Camera2dBundle {
            projection: OrthographicProjection {
                scaling_mode: ScalingMode::Auto {
                    min_width: 900.0,
                    min_height: 1000.0,
                },
                ..default()
            },
            ..default()
        })
        .insert(MainCamera);

    let pieces_handle = asset_server.load("pieces/pieces.png");
    let pieces_atlas = TextureAtlas::from_grid(
        pieces_handle,
        Vec2::new(90.0, 90.0),
        ATLAS_SIZE.0,
        ATLAS_SIZE.1,
    );
    let pieces_atlas_handle = pieces_atlases.add(pieces_atlas);

    let hex_mesh = meshes.add(shape::RegularPolygon::new(RADIUS, 6).into());

    for q in -N..=N {
        let r1 = (-N).max(-q - N);
        let r2 = N.min(-q + N);
        for r in r1..=r2 {
            let coord = Coord::new(q, r);
            let pixel = flat_hex_to_pixel(coord, RADIUS);

            if let Ok(hex_chess_lib::Piece { team, name, .. }) = game.board.get(coord) {
                let piece = commands
                    .spawn_bundle(SpatialBundle {
                        transform: Transform::from_translation(pixel.extend(1.0)),
                        ..default()
                    })
                    .with_children(|parent| {
                        parent.spawn_bundle(SpriteSheetBundle {
                            sprite: TextureAtlasSprite {
                                index: ATLAS_SIZE.0 * *team as usize + *name as usize,
                                ..default()
                            },
                            texture_atlas: pieces_atlas_handle.clone(),
                            transform: Transform::from_scale(Vec3::splat(0.8)),
                            ..default()
                        });
                    })
                    .insert(Piece)
                    .id();

                piece_sprites.insert(coord, piece);
            }

            commands
                .spawn_bundle(MaterialMesh2dBundle {
                    mesh: hex_mesh.clone().into(),
                    material: hex_materials.mat_mid.clone(),
                    transform: Transform::from_translation(pixel.extend(0.))
                        .with_rotation(Quat::from_axis_angle(Vec3::Z, std::f32::consts::FRAC_PI_6)),
                    ..default()
                })
                .insert(HexCoord { coord });
        }
    }
}

fn screen_to_world(
    screen: Vec2,
    size: Vec2,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Vec2 {
    let ndc = (screen / size) * 2.0 - Vec2::ONE;
    let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();
    let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));
    world_pos.truncate()
}

fn piece_click_system(
    mut commands: Commands,
    mut mouse_button_events: EventReader<MouseButtonInput>,
    wnds: Res<Windows>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut game: ResMut<Game>,
    mut piece_sprites: ResMut<PieceSprites>,
    mut q_piece_transforms: Query<&mut Transform, With<Piece>>,
    mut select: ResMut<SelectedHex>,
) {
    let (camera, camera_transform) = q_camera.single();

    let wnd = if let RenderTarget::Window(id) = camera.target {
        wnds.get(id).unwrap()
    } else {
        wnds.get_primary().unwrap()
    };

    if let Some(screen_pos) = wnd.cursor_position() {
        let world_pos = screen_to_world(
            screen_pos,
            Vec2::new(wnd.width(), wnd.height()),
            camera,
            camera_transform,
        );
        let hex_pos = pixel_to_flat_hex(world_pos, RADIUS);

        // set hovered tile
        select.hover = Some(hex_pos);

        for event in mouse_button_events.iter() {
            if event.button == MouseButton::Left && event.state == ButtonState::Pressed {
                if game.board.get(hex_pos).is_ok()
                    && game.board.get(hex_pos).unwrap().team == game.turn
                {
                    select.selected = Some(hex_pos);
                } else if let Some(from) = select.selected {
                    match game.move_piece(from, hex_pos) {
                        Ok(_) => {
                            // move the piece sprite
                            let entity = piece_sprites.remove(&from).unwrap();
                            let transform = q_piece_transforms.get_mut(entity).unwrap();
                            // delete the captured piece if there is one
                            if let Some(_) = piece_sprites.get(&hex_pos) {
                                let captured = piece_sprites.remove(&hex_pos).unwrap();
                                commands.entity(captured).despawn_recursive();
                            }
                            commands.entity(entity).insert(
                                transform.ease_to(
                                    Transform::from_translation(
                                        flat_hex_to_pixel(hex_pos, RADIUS)
                                            .extend(transform.translation.z),
                                    ),
                                    EaseMethod::EaseFunction(EaseFunction::QuadraticOut),
                                    EasingType::Once {
                                        duration: Duration::from_millis(200),
                                    },
                                ),
                            );
                            piece_sprites.insert(hex_pos, entity);

                            select.selected = None;
                        }
                        Err(e) => eprintln!("{}", e),
                    }
                }
            }
        }
    }
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.89, 0.97, 1.0)))
        .insert_resource(WindowDescriptor {
            title: "Hexagonal Chess".to_string(),
            width: 900.,
            height: 1000.,
            ..default()
        })
        .add_plugins_with(DefaultPlugins, |group| {
            group.add_before::<bevy::asset::AssetPlugin, _>(EmbeddedAssetPlugin)
        })
        .add_plugin(EasingsPlugin)
        .init_resource::<HexMaterials>()
        .init_resource::<PieceSprites>()
        .init_resource::<SelectedHex>()
        .init_resource::<Game>()
        .add_startup_system(setup)
        .add_system(color_tiles)
        .add_system(piece_click_system)
        .run();
}
