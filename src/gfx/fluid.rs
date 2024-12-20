use super::{buildchunk::add_block_vertices_fluid, ChunkData};
use crate::voxel::{
    world_to_chunk_position, Block, Chunk, ChunkPos, World, CHUNK_SIZE, CHUNK_SIZE_I32, EMPTY_BLOCK,
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

    for offset in OFFSETS {
        let (dx, dy, dz) = offset;
        let block = get_block(x + dx, y + dy + 1, z + dz, chunks);
        if block.id == voxel_id {
            return 8;
        }
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

//Get vertex level offsets
fn generate_vertex_heights(chunk: &Chunk, world: &World, voxel_id: u8) -> Vec<u8> {
    let pos = chunk.get_chunk_pos();
    let mut chunks = [None; 27];
    for i in 0..27i32 {
        let x = i / 9 - 1;
        let y = (i / 3) % 3 - 1;
        let z = i % 3 - 1;
        chunks[i as usize] = world.get_chunk(x + pos.x, y + pos.y, z + pos.z);
    }

    let sz = CHUNK_SIZE + 1;
    let mut vertex_heights = Vec::with_capacity(sz * sz * sz);
    for x in 0..=CHUNK_SIZE_I32 {
        for y in 0..=CHUNK_SIZE_I32 {
            for z in 0..=CHUNK_SIZE_I32 {
                let chunkpos = chunk.get_chunk_pos();
                let ix = x + chunkpos.x * CHUNK_SIZE_I32;
                let iy = y + chunkpos.y * CHUNK_SIZE_I32;
                let iz = z + chunkpos.z * CHUNK_SIZE_I32;
                let height = get_vertex_height(ix, iy, iz, &chunks, voxel_id);
                vertex_heights.push(height)
            }
        }
    }
    vertex_heights
}

//Create mesh for fluids (water, lava)
pub fn generate_fluid_vertex_data(
    chunk: &Chunk,
    adj_chunks: [Option<&Chunk>; 6],
    world: &World,
    voxel_id: u8,
) -> ChunkData {
    let mut chunk_vert_data = vec![];
    let heights = generate_vertex_heights(chunk, world, voxel_id);

    for x in 0..CHUNK_SIZE_I32 {
        for y in 0..CHUNK_SIZE_I32 {
            for z in 0..CHUNK_SIZE_I32 {
                let block = chunk.get_block_relative(x as usize, y as usize, z as usize);
                if block.id != voxel_id {
                    continue;
                }

                let pos = (x, y, z);
                add_block_vertices_fluid(chunk, adj_chunks, pos, &mut chunk_vert_data, &heights);
            }
        }
    }

    chunk_vert_data
}
