use super::{
    buildchunk::{add_block_vertices_fluid, Indices},
    ChunkData,
};
use crate::{
    gfx::buildchunk::get_indices,
    voxel::{
        world_to_chunk_position, Block, Chunk, ChunkPos, World, CHUNK_SIZE, CHUNK_SIZE_I32,
        EMPTY_BLOCK,
    },
};

const OFFSETS: [(i32, i32, i32); 4] = [(0, -1, 0), (-1, -1, 0), (0, -1, -1), (-1, -1, -1)];

fn get_block(x: i32, y: i32, z: i32, chunks: &[Option<&Chunk>]) -> Block {
    let (chunkx, chunky, chunkz) = world_to_chunk_position(x, y, z);
    let center = if let Some(chunk) = chunks[13] {
        chunk.get_chunk_pos()
    } else {
        ChunkPos::new(0, 0, 0)
    };
    let translated = ChunkPos::new(
        chunkx - center.x + 1,
        chunky - center.y + 1,
        chunkz - center.z + 1,
    );
    let index = translated.x * 9 + translated.y * 3 + translated.z;
    if index < 0 || index >= chunks.len() as i32 {
        return Block::new();
    }

    if let Some(chunk) = chunks[index as usize] {
        chunk.get_block(x, y, z)
    } else {
        Block::new()
    }
}

fn get_vertex_height(x: i32, y: i32, z: i32, chunks: &[Option<&Chunk>], voxel_id: u8) -> u8 {
    let mut total = 0;
    let mut count = 0;

    let mut water_adjacent = false;
    for offset in OFFSETS {
        let (dx, dy, dz) = offset;
        let block = get_block(x + dx, y + dy + 1, z + dz, chunks);
        let underblock = get_block(x + dx, y + dy, z + dz, chunks);

        if underblock.id == voxel_id || underblock.id == 0 {
            water_adjacent = true;
        }

        if block.id == voxel_id && underblock.id == block.id {
            return 8;
        }
    }

    if !water_adjacent {
        return 0;
    }

    for offset in OFFSETS {
        let (dx, dy, dz) = offset;
        let block = get_block(x + dx, y + dy, z + dz, chunks);
        if block.id != voxel_id && block.id != EMPTY_BLOCK {
            continue;
        }
        if block.geometry == 7 {
            return 7;
        }
        total += block.geometry;
        count += 1;
    }

    if count == 0 {
        return 0;
    }

    (total / count).clamp(1, 7)
}

//Create mesh for fluids (water, lava)
pub fn generate_fluid_vertex_data(
    chunk: &Chunk,
    adj_chunks: [Option<&Chunk>; 6],
    world: &World,
    voxel_id: u8,
) -> (ChunkData, Indices, i32) {
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

    let pos = chunk.get_chunk_pos();
    let mut chunks = [None; 27];
    for i in 0..27i32 {
        let x = i / 9 - 1;
        let y = (i / 3) % 3 - 1;
        let z = i % 3 - 1;
        chunks[i as usize] = world.get_chunk(x + pos.x, y + pos.y, z + pos.z);
    }

    const VALS_PER_VERT: usize = 7;
    //Generate heights
    const SZ: usize = CHUNK_SIZE + 1;
    let mut heights = [0u8; SZ * SZ * SZ];
    for i in 0..(chunk_vert_data.len() / VALS_PER_VERT) {
        let index = i * VALS_PER_VERT;
        let data = chunk_vert_data[index + 4];
        if (data & (7 << 2)) != 0 {
            let x = chunk_vert_data[index] as i32 + pos.x * CHUNK_SIZE_I32;
            let y = chunk_vert_data[index + 1] as i32 + pos.y * CHUNK_SIZE_I32;
            let z = chunk_vert_data[index + 2] as i32 + pos.z * CHUNK_SIZE_I32;
            let ux = chunk_vert_data[index] as usize;
            let uy = chunk_vert_data[index + 1] as usize;
            let uz = chunk_vert_data[index + 2] as usize;
            let height_index = ux * SZ * SZ + uy * SZ + uz;
            if heights[height_index] == 0 {
                heights[height_index] = get_vertex_height(x, y, z, &chunks, voxel_id);
                chunk_vert_data[index + 4] &= !(7 << 2);
                chunk_vert_data[index + 4] |= (8 - heights[height_index]) << 2;
            } else {
                chunk_vert_data[index + 4] &= !(7 << 2);
                chunk_vert_data[index + 4] |= (8 - heights[height_index]) << 2;
            }
        }
    }

    let face_count = chunk_vert_data.len() / (VALS_PER_VERT * 4);
    (chunk_vert_data, get_indices(face_count), 7)
}
