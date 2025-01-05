mod addvertices;

use crate::voxel::{Chunk, CHUNK_SIZE_I32};
use addvertices::add_block_vertices_trans;
use addvertices::{
    add_block_vertices_default,
    add_block_vertices_grass, 
    add_block_vertices_log, 
    add_fluid_vertices, 
    add_block_vertices_furnace_rotated
};

pub type Int3 = (i32, i32, i32);

pub type ChunkData = Vec<u8>;

pub fn add_block_vertices(
    chunk: &Chunk,
    adj_chunks: [Option<&Chunk>; 6],
    xyz: Int3,
    vert_data: &mut ChunkData,
) {
    let (x, y, z) = xyz;
    let blockid = chunk
        .get_block_relative(x as usize, y as usize, z as usize)
        .id;

    if chunk
        .get_block_relative(x as usize, y as usize, z as usize)
        .transparent()
    {
        return;
    }

    //TODO: add a better way of specifying how the faces of the blocks are textured
    //(probably as some kind of resource file) additionally, the unlabelled constants
    //should probably be deleted at some point
    match blockid {
        1 => {
            //Grass
            add_block_vertices_grass(chunk, adj_chunks, xyz, vert_data, 17, 4);
        }
        8 => {
            //Log
            add_block_vertices_log(chunk, adj_chunks, xyz, vert_data, 24, 25);
        }
        37 => {
            //Chest
            add_block_vertices_furnace_rotated(chunk, adj_chunks, xyz, vert_data, 38, 39);
        }
        40 => {
            //Furnace
            add_block_vertices_furnace_rotated(chunk, adj_chunks, xyz, vert_data, 41, 42);
        }
        43 => {
            //Farmland
            add_block_vertices_grass(chunk, adj_chunks, xyz, vert_data, 44, 43);
        }
        45 => {
            //Dry Farmland
            add_block_vertices_grass(chunk, adj_chunks, xyz, vert_data, 46, 45);
        }
        _ => {
            //Everything else
            add_block_vertices_default(chunk, adj_chunks, xyz, vert_data);
        }
    }
}

pub fn add_block_vertices_transparent(
    chunk: &Chunk,
    adj_chunks: [Option<&Chunk>; 6],
    xyz: Int3,
    vert_data: &mut ChunkData,
) {
    let (x, y, z) = xyz;

    if !chunk
        .get_block_relative(x as usize, y as usize, z as usize)
        .transparent()
    {
        return;
    }

    if chunk
        .get_block_relative(x as usize, y as usize, z as usize)
        .is_fluid()
    {
        return;
    }

    add_block_vertices_trans(chunk, adj_chunks, xyz, vert_data);
}

pub fn add_block_vertices_fluid(
    chunk: &Chunk,
    adj_chunks: [Option<&Chunk>; 6],
    xyz: Int3,
    vert_data: &mut ChunkData,
) {
    let (x, y, z) = xyz;
    if !chunk
        .get_block_relative(x as usize, y as usize, z as usize)
        .is_fluid()
    {
        return;
    }

    add_fluid_vertices(chunk, adj_chunks, xyz, vert_data);
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
                add_block_vertices_transparent(chunk, adj_chunks, pos, &mut chunk_vert_data);
            }
        }
    }

    chunk_vert_data
}
