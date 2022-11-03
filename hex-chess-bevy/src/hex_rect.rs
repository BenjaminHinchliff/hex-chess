use bevy::prelude::{Mat2, Vec2};
use hex_chess_lib::Coord;

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

pub fn flat_hex_to_pixel(hex: Coord, size: f32) -> Vec2 {
    let hex = Vec2::new(hex.q as f32, hex.r as f32);
    size * LAYOUT_FLAT * hex
}

pub fn pixel_to_flat_hex(hex: Vec2, size: f32) -> Coord {
    axial_round(LAYOUT_FLAT.inverse() * hex / size)
}
