use super::{rotate_orientation, slab, Block, ChunkData, Face, FaceInfo, Int3};
use crate::voxel::orientation_to_normal;
use cgmath::Vector3;

#[allow(clippy::too_many_arguments)]
fn add_stair_geometry_normal(
    vert_data: &mut ChunkData,
    block: Block,
    adj_block: Option<Block>,
    xyz: Int3,
    offset: Int3,
    face: &Face,
    face_info: FaceInfo,
    skip_face_fn: fn(block: Block, adj_block: Block, offset: Int3) -> bool,
) {
    let (x, y, z) = xyz;
    if let Some(adj_block) = adj_block {
        let mut b = block;
        b.set_shape(1);
        if skip_face_fn(b, adj_block, offset) {
            return;
        }
    } else {
        return;
    }

    for i in 0..6 {
        let x = face[i * 3] + x as u8;
        let y = face[i * 3 + 1] + y as u8;
        let z = face[i * 3 + 2] + z as u8;
        vert_data.push(x);
        vert_data.push(y);
        vert_data.push(z);
        vert_data.push(face_info.block_texture_id);
        vert_data.push(face_info.face_id);
    }
    if block.reflection() == 0 {
        slab::apply_slab_geometry(vert_data, xyz, 3);
    } else {
        slab::apply_slab_geometry(vert_data, xyz, 0);
    }
    slab::apply_slab_geometry(vert_data, xyz, block.orientation());
}

#[allow(clippy::too_many_arguments)]
fn add_stair_geometry_corner(
    vert_data: &mut ChunkData,
    block: Block,
    adj_block: Option<Block>,
    xyz: Int3,
    offset: Int3,
    face: &Face,
    face_info: FaceInfo,
    skip_face_fn: fn(block: Block, adj_block: Block, offset: Int3) -> bool,
) {
    let (x, y, z) = xyz;
    let mut b1 = block;
    b1.set_shape(1);
    let mut b2 = block;
    b2.set_shape(1);
    b2.set_orientation(rotate_orientation(b2.orientation()));
    let (offx, offy, offz) = offset;
    let diff = Vector3::<i32>::new(offx, offy, offz);
    let norm = orientation_to_normal(block.orientation());
    let rotated = rotate_orientation(block.orientation());
    let norm_rotated = orientation_to_normal(rotated);
    if let Some(adj_block) = adj_block {
        let skip = skip_face_fn(b1, adj_block, offset) || skip_face_fn(b2, adj_block, offset);
        if skip && !(diff == norm || diff == norm_rotated) {
            return;
        }
    } else {
        return;
    }

    for i in 0..6 {
        let x = face[i * 3] + x as u8;
        let y = face[i * 3 + 1] + y as u8;
        let z = face[i * 3 + 2] + z as u8;
        vert_data.push(x);
        vert_data.push(y);
        vert_data.push(z);
        vert_data.push(face_info.block_texture_id);
        vert_data.push(face_info.face_id);
    }
    if block.reflection() == 0 {
        slab::apply_slab_geometry(vert_data, xyz, 3);
    } else {
        slab::apply_slab_geometry(vert_data, xyz, 0);
    }
    slab::apply_slab_geometry(vert_data, xyz, block.orientation());
    slab::apply_slab_geometry(vert_data, xyz, rotated);
}

#[allow(clippy::too_many_arguments)]
pub fn add_stair_geometry(
    vert_data: &mut ChunkData,
    block: Block,
    adj_block: Option<Block>,
    xyz: Int3,
    offset: Int3,
    face: &Face,
    face_info: FaceInfo,
    skip_face_fn: fn(block: Block, adj_block: Block, offset: Int3) -> bool,
) {
    match block.shape() {
        2 => add_stair_geometry_normal(
            vert_data,
            block,
            adj_block,
            xyz,
            offset,
            face,
            face_info,
            skip_face_fn,
        ),
        3 => add_stair_geometry_corner(
            vert_data,
            block,
            adj_block,
            xyz,
            offset,
            face,
            face_info,
            skip_face_fn,
        ),
        4 => {
            add_stair_geometry_normal(
                vert_data,
                block,
                adj_block,
                xyz,
                offset,
                face,
                face_info,
                skip_face_fn,
            );
            let mut rotated = block;
            rotated.set_orientation(rotate_orientation(block.orientation()));
            add_stair_geometry_corner(
                vert_data,
                rotated,
                adj_block,
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
