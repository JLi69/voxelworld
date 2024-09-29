mod flat_world;

use super::{world_to_chunk_position, Block, Chunk};
use std::collections::HashMap;

//Absolute maximum size for the cache
const MAX_CACHE_SIZE: usize = 2048;

//World struct
pub struct World {
    //This only stores chunks that are near to the player
    pub chunks: HashMap<(i32, i32, i32), Chunk>,
    //Maximum range for chunks that should be loaded
    range: i32,
    //Position of the center chunk
    centerx: i32,
    centery: i32,
    centerz: i32,
    //cache of chunks that have been unloaded
    chunk_cache: HashMap<(i32, i32, i32), Chunk>,
}

impl World {
    //Create an empty world
    pub fn empty() -> Self {
        Self {
            chunks: HashMap::new(),
            range: 0,
            centerx: 0,
            centery: 0,
            centerz: 0,
            chunk_cache: HashMap::new(),
        }
    }

    //Create a new chunk from a chunk render distance (range)
    pub fn new(chunk_range: i32) -> Self {
        //Create chunk list
        let mut chunklist = HashMap::new();
        for y in -chunk_range..=chunk_range {
            for z in -chunk_range..=chunk_range {
                for x in -chunk_range..=chunk_range {
                    chunklist.insert((x, y, z), Chunk::new(x, y, z));
                }
            }
        }

        Self {
            chunks: chunklist,
            range: chunk_range,
            centerx: 0,
            centery: 0,
            centerz: 0,
            chunk_cache: HashMap::new(),
        }
    }

    pub fn get_range(&self) -> i32 {
        self.range
    }

    //Returns an optional immutable reference to a chunk based on the chunk
    //position - will return none if such chunk is not found
    pub fn get_chunk(&self, ix: i32, iy: i32, iz: i32) -> Option<&Chunk> {
        self.chunks.get(&(ix, iy, iz))
    }

    //Same is get_chunk except the reference is mutable
    pub fn get_mut_chunk(&mut self, ix: i32, iy: i32, iz: i32) -> Option<&mut Chunk> {
        self.chunks.get_mut(&(ix, iy, iz))
    }

    //Sets a block based on position,
    //does nothing if the position is out of range for the world
    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block: Block) {
        let (chunkx, chunky, chunkz) = world_to_chunk_position(x, y, z);
        let chunk = self.get_mut_chunk(chunkx, chunky, chunkz);
        if let Some(chunk) = chunk {
            chunk.set_block(x, y, z, block);
        }
    }

    //Returns a block based on position
    //Returns an empty block if the position is out of range for the world
    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Block {
        let (chunkx, chunky, chunkz) = world_to_chunk_position(x, y, z);
        let chunk = self.get_chunk(chunkx, chunky, chunkz);
        if let Some(chunk) = chunk {
            return chunk.get_block(x, y, z);
        }
        Block::new()
    }

    //If the cache gets too large, attempt to delete some sections
    pub fn clean_cache(&mut self) {
        if self.chunk_cache.len() <= MAX_CACHE_SIZE {
            return;
        }

        while self.chunk_cache.len() > MAX_CACHE_SIZE / 2 {
            let to_delete = self.chunk_cache.keys().next().copied();

            if let Some(pos) = to_delete {
                //TODO: add code to save chunks to disk
                self.chunk_cache.remove(&pos);
            }
        }
    }

    //When a chunk gets unloaded, add it to a cache in case it needs to be reloaded
    pub fn add_to_chunk_cache(&mut self, chunk: Chunk) {
        let pos = chunk.get_chunk_pos();
        self.chunk_cache.insert((pos.x, pos.y, pos.z), chunk);
    }
}
