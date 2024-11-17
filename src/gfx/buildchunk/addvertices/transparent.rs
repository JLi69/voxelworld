use super::{ChunkData, FaceInfo, Int3};
use crate::gfx::face_data::{
    Face, BACK_FACE, BOTTOM_FACE, FRONT_FACE, LEFT_FACE, RIGHT_FACE, TOP_FACE,
};
use crate::voxel::{out_of_bounds, wrap_coord, Chunk, EMPTY_BLOCK};

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
    let blockid = chunk
        .get_block_relative(x as usize, y as usize, z as usize)
        .id;

    let adj_x = wrap_coord(x + offx) as usize;
    let adj_y = wrap_coord(y + offy) as usize;
    let adj_z = wrap_coord(z + offz) as usize;
    if let Some(adj_chunk) = adj_chunk {
        let block = adj_chunk.get_block_relative(adj_x, adj_y, adj_z);
        let show_face = (block.transparent() && block.id != blockid)
            || (block.transparent() && block.id == blockid && !block.can_connect());
        if out_of_bounds(x, y, z, offx, offy, offz)
            && block.id != EMPTY_BLOCK
            && (!show_face || block.is_fluid())
        {
            return;
        }
    }

    if adj_chunk.is_none() && out_of_bounds(x, y, z, offx, offy, offz) {
        return;
    }

    let adj_x = (x + offx) as usize;
    let adj_y = (y + offy) as usize;
    let adj_z = (z + offz) as usize;
    let block = chunk.get_block_relative(adj_x, adj_y, adj_z);
    let show_face = (block.transparent() && block.id != blockid)
        || (block.transparent() && block.id == blockid && !block.can_connect());
    if chunk.get_block_relative(adj_x, adj_y, adj_z).id != EMPTY_BLOCK
        && (!show_face || block.is_fluid())
    {
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
}

pub fn add_block_vertices_trans(
    chunk: &Chunk,
    adj_chunks: [Option<&Chunk>; 6],
    xyz: Int3,
    vert_data: &mut ChunkData,
) {
    let (x, y, z) = xyz;
    let blockid = chunk
        .get_block_relative(x as usize, y as usize, z as usize)
        .id;
    if blockid == EMPTY_BLOCK {
        return;
    }

    let facex = FaceInfo::new(blockid, 0);
    let facey = FaceInfo::new(blockid, 1);
    let facez = FaceInfo::new(blockid, 2);

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
