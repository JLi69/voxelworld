use crate::voxel::{Chunk, CHUNK_SIZE_I32};
use super::buildchunk::{ChunkData, add_nonvoxel_vertices};

pub fn generate_non_voxel_vertex_data(chunk: &Chunk) -> (ChunkData, i32) {
    let mut vert_data = vec![];

    for x in 0..CHUNK_SIZE_I32 {
        for y in 0..CHUNK_SIZE_I32 {
            for z in 0..CHUNK_SIZE_I32 {
                let pos = (x, y, z);
                add_nonvoxel_vertices(chunk, pos, &mut vert_data);
            }
        }
    }

    (vert_data, 7)
}
