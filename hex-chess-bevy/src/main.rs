use bevy::{
    input::{mouse::MouseButtonInput, ButtonState},
    prelude::*,
    render::camera::RenderTarget,
    sprite::MaterialMesh2dBundle,
};

use hex_chess_lib::{Coord, Game, Piece};

const SQRT_3: f32 = 1.7320508075688772;

const LAYOUT_FLAT: Mat2 = Mat2::from_cols(Vec2::new(3. / 2., SQRT_3 / 2.), Vec2::new(0., SQRT_3));

fn axial_round(v: Vec2) -> Coord {
    let v = v.extend(-v.x - v.y);
    let mut rv = v.round();
    let dv = (rv - v).abs();
    if dv.x > dv.y && dv.x > dv.z {
        rv.x = -rv.y - rv.z;
    } else if dv.y > dv.z {
        rv.y = -rv.x - rv.z;
    }
    Coord::new(rv.x as i32, rv.y as i32)
}

fn flat_hex_to_pixel(hex: Coord, size: f32) -> Vec2 {
    let hex = Vec2::new(hex.q as f32, hex.r as f32);
    size * LAYOUT_FLAT * hex
}

fn pixel_to_flat_hex(hex: Vec2, size: f32) -> Coord {
    axial_round(LAYOUT_FLAT.inverse() * hex / size)
}

const N: i32 = 5;
const RADIUS: f32 = 50.0;
const ATLAS_SIZE: (usize, usize) = (6, 2);

#[derive(Component)]
struct MainCamera;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    mut pieces_atlases: ResMut<Assets<TextureAtlas>>,
    game: Res<Game>,
) {
    commands
        .spawn_bundle(Camera2dBundle::default())
        .insert(MainCamera);

    let pieces_handle = asset_server.load("pieces/pieces.png");
    let pieces_atlas = TextureAtlas::from_grid(
        pieces_handle,
        Vec2::new(90.0, 90.0),
        ATLAS_SIZE.0,
        ATLAS_SIZE.1,
    );
    let pieces_atlas_handle = pieces_atlases.add(pieces_atlas);

    for q in -N..=N {
        let r1 = (-N).max(-q - N);
        let r2 = N.min(-q + N);
        for r in r1..=r2 {
            let coord = Coord::new(q, r);
            let pixel = flat_hex_to_pixel(coord, RADIUS);

            let color = if coord.norm_squared() % 3 == 0 {
                Color::rgb(0.91, 0.68, 0.44)
            } else if (coord - (1, 0).into()).norm_squared() % 3 == 0 {
                Color::rgb(0.82, 0.55, 0.27)
            } else {
                Color::rgb(1.0, 0.81, 0.62)
            };

            if let Ok(Piece { team, name, .. }) = game.board.get(coord) {
                commands.spawn_bundle(SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        index: ATLAS_SIZE.0 * *team as usize + name.idx() as usize,
                        ..default()
                    },
                    texture_atlas: pieces_atlas_handle.clone(),
                    transform: Transform::from_translation(pixel.extend(1.0))
                        .with_scale(Vec3::splat(0.8)),
                    ..default()
                });
            }

            commands.spawn_bundle(MaterialMesh2dBundle {
                mesh: meshes
                    .add(shape::RegularPolygon::new(RADIUS, 6).into())
                    .into(),
                material: materials.add(ColorMaterial::from(color)),
                transform: Transform::from_translation(pixel.extend(0.))
                    .with_rotation(Quat::from_axis_angle(Vec3::Z, std::f32::consts::FRAC_PI_6)),
                ..default()
            });
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
    mut mouse_button_events: EventReader<MouseButtonInput>,
    wnds: Res<Windows>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    let (camera, camera_transform) = q_camera.single();

    let wnd = if let RenderTarget::Window(id) = camera.target {
        wnds.get(id).unwrap()
    } else {
        wnds.get_primary().unwrap()
    };

    if !mouse_button_events.is_empty() {
        if let Some(screen_pos) = wnd.cursor_position() {
            let world_pos = screen_to_world(
                screen_pos,
                Vec2::new(wnd.width(), wnd.height()),
                camera,
                camera_transform,
            );
            let hex_pos = pixel_to_flat_hex(world_pos, RADIUS);
            println!("{:?}", hex_pos);
        }
    }
    for event in mouse_button_events.iter() {
        if event.button == MouseButton::Left && event.state == ButtonState::Pressed {}
    }
}

fn main() {
    let game = Game::new();

    App::new()
        .insert_resource(ClearColor(Color::rgb(0.89, 0.97, 1.0)))
        .insert_resource(WindowDescriptor {
            title: "Hexagonal Chess".to_string(),
            width: 900.,
            height: 1000.,
            ..default()
        })
        .insert_resource(game)
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(piece_click_system)
        .run();
}
