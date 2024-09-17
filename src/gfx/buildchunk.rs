use super::face_data::{Face, BACK_FACE, BOTTOM_FACE, FRONT_FACE, LEFT_FACE, RIGHT_FACE, TOP_FACE};
use crate::voxel::{out_of_bounds, wrap_coord, Chunk, CHUNK_SIZE_I32, EMPTY_BLOCK};

type Int3 = (i32, i32, i32);

pub type ChunkData = Vec<u8>;

fn add_face(
    chunk: &Chunk,
    adj_chunk: Option<&Chunk>,
    xyz: Int3,
    offset: Int3,
    vert_data: &mut ChunkData,
    face: &Face,
    face_id: u8,
) {
    let (x, y, z) = xyz;
    let (offx, offy, offz) = offset;

    let adj_x = wrap_coord(x + offx) as usize;
    let adj_y = wrap_coord(y + offy) as usize;
    let adj_z = wrap_coord(z + offz) as usize;
    if let Some(adj_chunk) = adj_chunk {
        if out_of_bounds(x, y, z, offx, offy, offz)
            && adj_chunk.get_block_relative(adj_x, adj_y, adj_z).id != EMPTY_BLOCK
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
    if chunk.get_block_relative(adj_x, adj_y, adj_z).id != EMPTY_BLOCK {
        return;
    }

    let block = chunk.get_block_relative(x as usize, y as usize, z as usize);
    for i in 0..6 {
        let x = face[i * 3] + x as u8;
        let y = face[i * 3 + 1] + y as u8;
        let z = face[i * 3 + 2] + z as u8;
        vert_data.push(x);
        vert_data.push(y);
        vert_data.push(z);
        vert_data.push(block.id);
        vert_data.push(face_id)
    }
}

fn add_block_vertices(
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

    add_face(
        chunk,
        adj_chunks[0],
        xyz,
        (0, 1, 0),
        vert_data,
        &TOP_FACE,
        1,
    );
    add_face(
        chunk,
        adj_chunks[1],
        xyz,
        (0, -1, 0),
        vert_data,
        &BOTTOM_FACE,
        1,
    );
    add_face(
        chunk,
        adj_chunks[2],
        xyz,
        (-1, 0, 0),
        vert_data,
        &LEFT_FACE,
        0,
    );
    add_face(
        chunk,
        adj_chunks[3],
        xyz,
        (1, 0, 0),
        vert_data,
        &RIGHT_FACE,
        0,
    );
    add_face(
        chunk,
        adj_chunks[4],
        xyz,
        (0, 0, -1),
        vert_data,
        &FRONT_FACE,
        2,
    );
    add_face(
        chunk,
        adj_chunks[5],
        xyz,
        (0, 0, 1),
        vert_data,
        &BACK_FACE,
        2,
    );
}

/*
 * Each vertex is formatted the following way:
 * [x position relative to chunk],
 * [y position relative to chunk],
 * [z position relative to chunk],
 * [texture id]
 * [other data]
 *
 * This should mean that each vertex is only 40 bits or 4.25 bytes
 *
 * for adj_chunks (adjacent chunks),
 * 0 is the chunk on top,
 * 1 is the chunk on the bottom,
 * 2 is the chunk to the left,
 * 3 is the chunk to the right,
 * 4 is the chunk to the front,
 * 5 is the chunk to the back
 * */
pub fn generate_chunk_vertex_data(chunk: &Chunk, adj_chunks: [Option<&Chunk>; 6]) -> ChunkData {
    let mut chunk_vert_data = vec![];

    for x in 0..CHUNK_SIZE_I32 {
        for y in 0..CHUNK_SIZE_I32 {
            for z in 0..CHUNK_SIZE_I32 {
                let pos = (x, y, z);
                add_block_vertices(chunk, adj_chunks, pos, &mut chunk_vert_data);
            }
        }
    }

    chunk_vert_data
}
