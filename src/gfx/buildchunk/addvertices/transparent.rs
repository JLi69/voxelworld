use super::{slab, ChunkData, FaceInfo, Int3};
use crate::gfx::face_data::{
    Face, BACK_FACE, BOTTOM_FACE, FRONT_FACE, LEFT_FACE, RIGHT_FACE, TOP_FACE,
};
use crate::voxel::{orientation_to_normal, out_of_bounds, wrap_coord, Block, Chunk, EMPTY_BLOCK};
use cgmath::{InnerSpace, Vector3};

fn skip_block_face_trans(adj_block: Block, offset: Int3, id: u8) -> bool {
    let (offx, offy, offz) = offset;
    match adj_block.shape() {
        //Full block
        0 => {
            let show_face = (adj_block.transparent() && adj_block.id != id)
                || (adj_block.transparent() && adj_block.id == id && !adj_block.can_connect());
            !show_face && adj_block.id != EMPTY_BLOCK
        }
        //Slab
        1 => {
            let show_face = (adj_block.transparent() && adj_block.id != id)
                || (adj_block.transparent() && adj_block.id == id && !adj_block.can_connect());
            if show_face || adj_block.id == EMPTY_BLOCK {
                return false;
            }

            let adj_norm = orientation_to_normal(adj_block.orientation());
            let diff = Vector3::new(offx, offy, offz);
            adj_norm == diff
        }
        _ => false,
    }
}

fn skip_slab_face_trans(orientation: u8, adj_block: Block, offset: Int3, id: u8) -> bool {
    let (offx, offy, offz) = offset;
    match adj_block.shape() {
        //Full block
        0 => {
            let show_face = (adj_block.transparent() && adj_block.id != id)
                || (adj_block.transparent() && adj_block.id == id && !adj_block.can_connect());
            if show_face || adj_block.id == EMPTY_BLOCK {
                return false;
            }

            let norm = orientation_to_normal(orientation);
            let diff = Vector3::new(offx, offy, offz);
            let dot = norm.dot(diff);
            dot == -1 || dot == 0
        }
        //Slab
        1 => {
            let show_face = (adj_block.transparent() && adj_block.id != id)
                || (adj_block.transparent() && adj_block.id == id && !adj_block.can_connect());
            if show_face || adj_block.id == EMPTY_BLOCK {
                return false;
            }

            let norm = orientation_to_normal(orientation);
            let adj_norm = orientation_to_normal(adj_block.orientation());
            let diff = Vector3::new(offx, offy, offz);
            let dot = norm.dot(diff);
            if dot == 0 {
                norm == adj_norm || adj_norm == diff
            } else if dot == 1 {
                false
            } else if dot == -1 {
                adj_norm == diff
            } else {
                false
            }
        }
        _ => false,
    }
}

fn skip_face_trans(block: Block, adj_block: Block, offset: Int3) -> bool {
    match block.shape() {
        //Full block
        0 => skip_block_face_trans(adj_block, offset, block.id),
        //Slab
        1 => skip_slab_face_trans(block.orientation(), adj_block, offset, block.id),
        _ => false,
    }
}

fn add_face_transparent(
    chunk: &Chunk,
    adj_chunk: Option<&Chunk>,
    xyz: Int3,
    offset: Int3,
    vert_data: &mut ChunkData,
    face: &Face,
    face_info: FaceInfo,
) {
    let (x, y, z) = xyz;
    let (offx, offy, offz) = offset;
    let block = chunk.get_block_relative(x as usize, y as usize, z as usize);

    let adj_x = wrap_coord(x + offx) as usize;
    let adj_y = wrap_coord(y + offy) as usize;
    let adj_z = wrap_coord(z + offz) as usize;
    if let Some(adj_chunk) = adj_chunk {
        let adj_block = adj_chunk.get_block_relative(adj_x, adj_y, adj_z);
        if out_of_bounds(x, y, z, offx, offy, offz) && skip_face_trans(block, adj_block, offset) {
            return;
        }
    }

    if adj_chunk.is_none() && out_of_bounds(x, y, z, offx, offy, offz) {
        return;
    }

    let adj_x = (x + offx) as usize;
    let adj_y = (y + offy) as usize;
    let adj_z = (z + offz) as usize;
    let adj_block = chunk.get_block_relative(adj_x, adj_y, adj_z);
    if skip_face_trans(block, adj_block, offset) {
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

    let block = chunk.get_block_relative(x as usize, y as usize, z as usize);
    slab::apply_slab_geometry(vert_data, xyz, block.shape(), block.orientation());
}

pub fn add_block_vertices_trans(
    chunk: &Chunk,
    adj_chunks: [Option<&Chunk>; 6],
    xyz: Int3,
    vert_data: &mut ChunkData,
) {
    let (x, y, z) = xyz;
    let block = chunk.get_block_relative(x as usize, y as usize, z as usize);
    if block.id == EMPTY_BLOCK {
        return;
    }

    let facex = FaceInfo::new(block.id, 0);
    let facey = FaceInfo::new(block.id, 1);
    let facez = FaceInfo::new(block.id, 2);

    #[rustfmt::skip]
    add_face_transparent(chunk, adj_chunks[0], xyz, (0, 1, 0), vert_data, &TOP_FACE, facey);
    #[rustfmt::skip]
    add_face_transparent(chunk, adj_chunks[1], xyz, (0, -1, 0), vert_data, &BOTTOM_FACE, facey);
    #[rustfmt::skip]
    add_face_transparent(chunk, adj_chunks[2], xyz, (-1, 0, 0), vert_data, &LEFT_FACE, facex);
    #[rustfmt::skip]
    add_face_transparent(chunk, adj_chunks[3], xyz, (1, 0, 0), vert_data, &RIGHT_FACE, facex);
    #[rustfmt::skip]
    add_face_transparent(chunk, adj_chunks[4], xyz, (0, 0, -1), vert_data, &FRONT_FACE, facez);
    #[rustfmt::skip]
    add_face_transparent(chunk, adj_chunks[5], xyz, (0, 0, 1), vert_data, &BACK_FACE, facez);
}
