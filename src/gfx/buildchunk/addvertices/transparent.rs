use super::{get_adj_block, add_stair_geometry};
use super::{ChunkData, FaceInfo, Int3, apply_geometry, skipface::skip_face_trans};
use crate::gfx::face_data::{
    Face, BACK_FACE, BOTTOM_FACE, FRONT_FACE, LEFT_FACE, RIGHT_FACE, TOP_FACE,
};
use crate::voxel::{Chunk, EMPTY_BLOCK};

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
    let block = chunk.get_block_relative(x as usize, y as usize, z as usize);
 
    let adj_block = get_adj_block(chunk, adj_chunk, xyz, offset);
    if let Some(adj_block) = adj_block {
        if skip_face_trans(block, adj_block, offset) {
            let stair = add_stair_geometry(block, Some(adj_block), xyz, offset, face, face_info, skip_face_trans);
            vert_data.extend(stair);
            return;
        }
    } else {
        let stair = add_stair_geometry(block, adj_block, xyz, offset, face, face_info, skip_face_trans);
        vert_data.extend(stair);
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
    apply_geometry(block, xyz, vert_data);
    let stair = add_stair_geometry(block, adj_block, xyz, offset, face, face_info, skip_face_trans);
    vert_data.extend(stair);
}

pub fn add_block_vertices_trans(
    chunk: &Chunk,
    adj_chunks: [Option<&Chunk>; 6],
    xyz: Int3,
    vert_data: &mut ChunkData,
    slab_side1: Option<u8>,
    slab_side2: Option<u8>,
) {
    let (x, y, z) = xyz;
    let block = chunk.get_block_relative(x as usize, y as usize, z as usize);
    if block.id == EMPTY_BLOCK {
        return;
    }

    let facex = if block.shape() == 1 {
        match block.orientation() % 3 {
            0 => FaceInfo::new(slab_side1.unwrap_or(block.id), 0),
            1 => FaceInfo::new(block.id, 0),
            2 => FaceInfo::new(slab_side2.unwrap_or(block.id), 0),
            _ => FaceInfo::new(block.id, 0), //Unreachable
        }
    } else {
        FaceInfo::new(block.id, 0)
    };
    let facey = if block.shape() == 1 {
        match block.orientation() % 3 {
            0 => FaceInfo::new(block.id, 1),
            1 => FaceInfo::new(slab_side1.unwrap_or(block.id), 1),
            2 => FaceInfo::new(slab_side2.unwrap_or(block.id), 1),
            _ => FaceInfo::new(block.id, 1), //Unreachable
        }
    } else {
        FaceInfo::new(block.id, 1)
    };
    let facez = if block.shape() == 1 {
        match block.orientation() % 3 {
            0 => FaceInfo::new(slab_side1.unwrap_or(block.id), 2),
            1 => FaceInfo::new(slab_side2.unwrap_or(block.id), 2),
            2 => FaceInfo::new(block.id, 2),
            _ => FaceInfo::new(block.id, 2), //Unreachable
        }
    } else {
        FaceInfo::new(block.id, 2)
    };

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
