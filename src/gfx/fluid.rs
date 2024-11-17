use super::{buildchunk::add_block_vertices_fluid, ChunkData};
use crate::voxel::{Chunk, CHUNK_SIZE_I32};

//Create mesh for fluids (water, lava)
pub fn generate_fluid_vertex_data(
    chunk: &Chunk,
    adj_chunks: [Option<&Chunk>; 6],
    voxel_id: u8,
) -> ChunkData {
    let mut chunk_vert_data = vec![];

    for x in 0..CHUNK_SIZE_I32 {
        for y in 0..CHUNK_SIZE_I32 {
            for z in 0..CHUNK_SIZE_I32 {
                let block = chunk.get_block_relative(x as usize, y as usize, z as usize);
                if block.id != voxel_id {
                    continue;
                }

                let pos = (x, y, z);
                add_block_vertices_fluid(chunk, adj_chunks, pos, &mut chunk_vert_data);
            }
        }
    }

    chunk_vert_data
}
