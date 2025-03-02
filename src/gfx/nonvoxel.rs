use super::buildchunk::{add_nonvoxel_vertices, ChunkData, Indices};
use crate::voxel::{Chunk, CHUNK_SIZE_I32};
use std::collections::HashMap;

fn slice_to_u64(bytes: &[u8]) -> u64 {
    let mut value = 0;
    for (i, b) in bytes.iter().enumerate() {
        value |= (*b as u64) << (i * 8);
    }
    value
}

pub fn generate_non_voxel_vertex_data(chunk: &Chunk) -> (ChunkData, Indices, i32) {
    if chunk.is_empty() {
        return (vec![], vec![], 8);
    }

    let mut vert_data = vec![];
    let mut cache = HashMap::new();
    for x in 0..CHUNK_SIZE_I32 {
        for y in 0..CHUNK_SIZE_I32 {
            for z in 0..CHUNK_SIZE_I32 {
                let pos = (x, y, z);
                add_nonvoxel_vertices(chunk, pos, &mut vert_data, &mut cache);
            }
        }
    }

    let mut indexmap = HashMap::<u64, u32>::new();
    let mut vertices_indexed = Vec::with_capacity(vert_data.len());
    let mut indices = vec![];
    for i in 0..(vert_data.len() / 8) {
        let vert = slice_to_u64(&vert_data[(i * 8)..(i * 8 + 8)]);
        if let Some(index) = indexmap.get(&vert) {
            indices.push(*index);
        } else {
            let index = vertices_indexed.len() as u32 / 8;
            vertices_indexed.extend_from_slice(&vert_data[(i * 8)..(i * 8 + 8)]);
            indices.push(index);
            indexmap.insert(vert, index);
        }
    }

    (vertices_indexed, indices, 8)
}
