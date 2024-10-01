mod flat_world;

use super::{world_to_chunk_position, Block, Chunk};
use std::collections::HashMap;

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
    clear_cache: bool,
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
            clear_cache: false,
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
            clear_cache: false,
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

    fn get_max_cache_sz(&self) -> usize {
        let sz = self.range * 2 + 1;
        (sz * sz * 8) as usize
    }

    //Checks if the world should clear its chunk cache
    pub fn check_for_cache_clear(&mut self) {
        if self.chunk_cache.len() <= self.get_max_cache_sz() {
            return;
        }

        eprintln!("Clearing chunk cache!");
        self.clear_cache = true;
    }

    //If the cache gets too large, attempt to delete some sections
    pub fn clean_cache(&mut self) {
        if !self.clear_cache {
            return;
        }

        let max_cache_sz = self.get_max_cache_sz();
        let mut time_passed = 0.0;
        let start = std::time::Instant::now();
        while self.chunk_cache.len() > max_cache_sz as usize / 2 && time_passed < 0.01 {
            let to_delete = self.chunk_cache.keys().next().copied();

            if let Some(pos) = to_delete {
                //TODO: add code to save chunks to disk
                self.chunk_cache.remove(&pos);
            }
            let now = std::time::Instant::now();
            time_passed = (now - start).as_secs_f32();
        }

        if self.chunk_cache.len() <= max_cache_sz as usize / 2 {
            self.clear_cache = false;
        }
    }

    //When a chunk gets unloaded, add it to a cache in case it needs to be reloaded
    pub fn add_to_chunk_cache(&mut self, chunk: Chunk) {
        let pos = chunk.get_chunk_pos();
        self.chunk_cache.insert((pos.x, pos.y, pos.z), chunk);
    }
}
