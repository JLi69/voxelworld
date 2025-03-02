use super::{rotate_orientation, slab, Block, ChunkData, Face, FaceInfo, Int3};
use crate::voxel::{light::Light, orientation_to_normal};
use cgmath::Vector3;

#[derive(Clone, Copy)]
pub struct StairInfo {
    pub block: Block,
    pub adj_block: Option<Block>,
    pub adj_light: Light,
    pub light: Light,
}

impl StairInfo {
    pub fn new(b: Block, adj: Option<Block>, adjl: Light, l: Light) -> Self {
        Self {
            block: b,
            adj_block: adj,
            adj_light: adjl,
            light: l,
        }
    }
}

fn add_stair_geometry_normal(
    vert_data: &mut ChunkData,
    stairinfo: StairInfo,
    xyz: Int3,
    offset: Int3,
    face: &Face,
    face_info: FaceInfo,
    skip_face_fn: fn(block: Block, adj_block: Block, offset: Int3) -> bool,
) {
    let (x, y, z) = xyz;
    if let Some(adj_block) = stairinfo.adj_block {
        let mut b = stairinfo.block;
        b.set_shape(1);
        if skip_face_fn(b, adj_block, offset) {
            return;
        }
    } else {
        return;
    }

    let adj_light = stairinfo.adj_light;
    let light = stairinfo.light;
    for i in 0..4 {
        let x = face[i * 3] + x as u8;
        let y = face[i * 3 + 1] + y as u8;
        let z = face[i * 3 + 2] + z as u8;
        vert_data.push(x);
        vert_data.push(y);
        vert_data.push(z);
        vert_data.push(face_info.block_texture_id);
        vert_data.push(face_info.face_id);
        vert_data.push(((adj_light.r() as u8) << 4) | (adj_light.skylight() as u8));
        vert_data.push(((adj_light.b() as u8) << 4) | (adj_light.g() as u8));
    }

    if stairinfo.block.reflection() == 0 {
        slab::apply_slab_geometry(vert_data, xyz, 3, light);
    } else {
        slab::apply_slab_geometry(vert_data, xyz, 0, light);
    }
    slab::apply_slab_geometry(vert_data, xyz, stairinfo.block.orientation(), light);
}

#[allow(clippy::too_many_arguments)]
fn add_stair_geometry_corner(
    vert_data: &mut ChunkData,
    stairinfo: StairInfo,
    xyz: Int3,
    offset: Int3,
    face: &Face,
    face_info: FaceInfo,
    skip_face_fn: fn(block: Block, adj_block: Block, offset: Int3) -> bool,
) {
    let (x, y, z) = xyz;
    let mut b1 = stairinfo.block;
    b1.set_shape(1);
    let mut b2 = stairinfo.block;
    b2.set_shape(1);
    b2.set_orientation(rotate_orientation(b2.orientation()));
    let (offx, offy, offz) = offset;
    let diff = Vector3::<i32>::new(offx, offy, offz);
    let norm = orientation_to_normal(stairinfo.block.orientation());
    let rotated = rotate_orientation(stairinfo.block.orientation());
    let norm_rotated = orientation_to_normal(rotated);
    if let Some(adj_block) = stairinfo.adj_block {
        let skip = skip_face_fn(b1, adj_block, offset) || skip_face_fn(b2, adj_block, offset);
        if skip && !(diff == norm || diff == norm_rotated) {
            return;
        }
    } else {
        return;
    }

    let adj_light = stairinfo.adj_light;
    let light = stairinfo.light;
    for i in 0..4 {
        let x = face[i * 3] + x as u8;
        let y = face[i * 3 + 1] + y as u8;
        let z = face[i * 3 + 2] + z as u8;
        vert_data.push(x);
        vert_data.push(y);
        vert_data.push(z);
        vert_data.push(face_info.block_texture_id);
        vert_data.push(face_info.face_id);
        vert_data.push(((adj_light.r() as u8) << 4) | (adj_light.skylight() as u8));
        vert_data.push(((adj_light.b() as u8) << 4) | (adj_light.g() as u8));
    }
    if stairinfo.block.reflection() == 0 {
        slab::apply_slab_geometry(vert_data, xyz, 3, light);
    } else {
        slab::apply_slab_geometry(vert_data, xyz, 0, light);
    }
    slab::apply_slab_geometry(vert_data, xyz, stairinfo.block.orientation(), light);
    slab::apply_slab_geometry(vert_data, xyz, rotated, light);
}

#[allow(clippy::too_many_arguments)]
pub fn add_stair_geometry(
    vert_data: &mut ChunkData,
    stairinfo: StairInfo,
    xyz: Int3,
    offset: Int3,
    face: &Face,
    face_info: FaceInfo,
    skip_face_fn: fn(block: Block, adj_block: Block, offset: Int3) -> bool,
) {
    match stairinfo.block.shape() {
        2 => add_stair_geometry_normal(
            vert_data,
            stairinfo,
            xyz,
            offset,
            face,
            face_info,
            skip_face_fn,
        ),
        3 => add_stair_geometry_corner(
            vert_data,
            stairinfo,
            xyz,
            offset,
            face,
            face_info,
            skip_face_fn,
        ),
        4 => {
            add_stair_geometry_normal(
                vert_data,
                stairinfo,
                xyz,
                offset,
                face,
                face_info,
                skip_face_fn,
            );
            let mut rotated = stairinfo.block;
            rotated.set_orientation(rotate_orientation(stairinfo.block.orientation()));
            let stairinfo_rotated = StairInfo::new(
                rotated,
                stairinfo.adj_block,
                stairinfo.adj_light,
                stairinfo.light,
            );
            add_stair_geometry_corner(
                vert_data,
                stairinfo_rotated,
                xyz,
                offset,
                face,
                face_info,
                skip_face_fn,
            );
        }
        _ => {}
    }
}
