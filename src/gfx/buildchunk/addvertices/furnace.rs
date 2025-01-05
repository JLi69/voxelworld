use crate::voxel::{Chunk, EMPTY_BLOCK};
use crate::gfx::face_data::{BACK_FACE, BOTTOM_FACE, FRONT_FACE, LEFT_FACE, RIGHT_FACE, TOP_FACE};
use super::{add_face, ChunkData, FaceInfo, Int3};

fn add_block_vertices_furnace(
    chunk: &Chunk,
    adj_chunks: [Option<&Chunk>; 6],
    xyz: Int3,
    vert_data: &mut ChunkData,
    side: u8,
    top: u8,
    front_face_index: usize,
) {
    let (x, y, z) = xyz;
    let blockid = chunk
        .get_block_relative(x as usize, y as usize, z as usize)
        .id;

    let mut faces = [
        //face x
        FaceInfo::new(side, 0), //Left
        FaceInfo::new(side, 0), //Right
        //face z
        FaceInfo::new(side, 2), //Back
        FaceInfo::new(side, 2), //Front
    ];
    faces[front_face_index].block_texture_id = blockid;
    let topface = FaceInfo::new(top, 1);

    #[rustfmt::skip]
    add_face(chunk, adj_chunks[0], xyz, (0, 1, 0), vert_data, &TOP_FACE, topface);
    #[rustfmt::skip]
    add_face(chunk, adj_chunks[1], xyz, (0, -1, 0), vert_data, &BOTTOM_FACE, topface);
    #[rustfmt::skip]
    add_face(chunk, adj_chunks[2], xyz, (-1, 0, 0), vert_data, &LEFT_FACE, faces[0]);
    #[rustfmt::skip]
    add_face(chunk, adj_chunks[3], xyz, (1, 0, 0), vert_data, &RIGHT_FACE, faces[1]);
    #[rustfmt::skip]
    add_face(chunk, adj_chunks[4], xyz, (0, 0, -1), vert_data, &FRONT_FACE, faces[2]);
    #[rustfmt::skip]
    add_face(chunk, adj_chunks[5], xyz, (0, 0, 1), vert_data, &BACK_FACE, faces[3]);
}

pub fn add_block_vertices_furnace_rotated(
    chunk: &Chunk,
    adj_chunks: [Option<&Chunk>; 6],
    xyz: Int3,
    vert_data: &mut ChunkData,
    side: u8,
    top: u8,
) {
    let (x, y, z) = xyz;
    let block = chunk.get_block_relative(x as usize, y as usize, z as usize);
    if block.id == EMPTY_BLOCK {
        return;
    }

    match block.orientation() {
        0 => add_block_vertices_furnace(chunk, adj_chunks, xyz, vert_data, side, top, 0),
        1 => add_block_vertices_furnace(chunk, adj_chunks, xyz, vert_data, side, top, 0),
        2 => add_block_vertices_furnace(chunk, adj_chunks, xyz, vert_data, side, top, 2),
        3 => add_block_vertices_furnace(chunk, adj_chunks, xyz, vert_data, side, top, 0),
        4 => add_block_vertices_furnace(chunk, adj_chunks, xyz, vert_data, side, top, 1),
        5 => add_block_vertices_furnace(chunk, adj_chunks, xyz, vert_data, side, top, 3),
        _ => {}
    }
}
