use glam::{IVec2, IVec3};

pub fn axial_to_cube(p: IVec2) -> IVec3 {
    p.extend(-p.x - p.y)
}

pub fn reflect_q(v: IVec3) -> IVec3 {
    IVec3::new(v.x, v.z, v.y)
}
