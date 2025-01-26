use std::collections::HashMap;

use super::buildchunk::{add_nonvoxel_vertices, ChunkData};
use crate::voxel::{Chunk, CHUNK_SIZE_I32};

pub fn generate_non_voxel_vertex_data(chunk: &Chunk) -> (ChunkData, i32) {
    let mut vert_data = vec![];

    if chunk.is_empty() {
        return (vert_data, 7);
    }

    let mut cache = HashMap::new();
    for x in 0..CHUNK_SIZE_I32 {
        for y in 0..CHUNK_SIZE_I32 {
            for z in 0..CHUNK_SIZE_I32 {
                let pos = (x, y, z);
                add_nonvoxel_vertices(chunk, pos, &mut vert_data, &mut cache);
            }
        }
    }

    (vert_data, 7)
}
