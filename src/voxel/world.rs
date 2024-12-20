mod default_world;
mod flat_world;
mod gen_more;
mod save;
mod block_update;

use super::{world_to_chunk_position, Block, Chunk};
use crate::gfx::ChunkTables;
use cgmath::Vector3;
use noise::{Fbm, Perlin};
use std::collections::HashMap;

pub const OCTAVES: usize = 5;
pub const PERSISTENCE: f64 = 0.47;

//Struct that contains information for generating the world
struct WorldGenerator {
    pub terrain_generator: Fbm<Perlin>,
    pub noise_cave_generator: Perlin,
    pub tree_generator: Perlin,
    world_seed: u32,
}

impl WorldGenerator {
    fn new(seed: u32) -> Self {
        let mut terrain_noise = Fbm::new(seed);
        terrain_noise.octaves = OCTAVES;
        terrain_noise.persistence = PERSISTENCE;

        Self {
            terrain_generator: terrain_noise,
            noise_cave_generator: Perlin::new(seed + 1),
            tree_generator: Perlin::new(seed + 2),
            world_seed: seed,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum WorldGenType {
    DefaultGen,
    Flat,
}

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
    world_generator: WorldGenerator,
    world_seed: u32,
    pub gen_type: WorldGenType,
    //World path
    pub path: String,
    //Block update timer
    block_update_timer: f32,
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
            world_generator: WorldGenerator::new(0),
            gen_type: WorldGenType::DefaultGen,
            world_seed: 0,
            path: String::new(),
            block_update_timer: 0.0,
        }
    }

    //Create a new chunk from a chunk render distance (range)
    pub fn new(seed: u32, chunk_range: i32, generation: WorldGenType) -> Self {
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
            world_generator: WorldGenerator::new(seed),
            gen_type: generation,
            world_seed: seed,
            path: String::new(),
            block_update_timer: 0.0,
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
        ((sz * sz * 8) as usize).max(4096)
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
        while self.chunk_cache.len() > max_cache_sz / 2 && time_passed < 0.01 {
            let to_delete = self.chunk_cache.keys().next().copied();

            if let Some(pos) = to_delete {
                if let Some(chunk) = self.chunk_cache.get(&pos) {
                    save::save_chunk(chunk, &self.path);
                }
                self.chunk_cache.remove(&pos);
            }
            let now = std::time::Instant::now();
            time_passed = (now - start).as_secs_f32();
        }

        if self.chunk_cache.len() <= max_cache_sz / 2 {
            self.clear_cache = false;
        }
    }

    //When a chunk gets unloaded, add it to a cache in case it needs to be reloaded
    pub fn add_to_chunk_cache(&mut self, chunk: Chunk) {
        let pos = chunk.get_chunk_pos();
        self.chunk_cache.insert((pos.x, pos.y, pos.z), chunk);
    }

    //Generate world
    pub fn generate_world(&mut self) {
        match self.gen_type {
            WorldGenType::DefaultGen => self.gen_default(),
            WorldGenType::Flat => self.gen_flat(),
        }
    }

    //Generate more world
    pub fn generate_more(&mut self, pos: Vector3<f32>, chunktables: &mut ChunkTables) {
        match self.gen_type {
            WorldGenType::DefaultGen => self.gen_more_default(pos, chunktables),
            WorldGenType::Flat => self.gen_more_flat(pos, chunktables),
        }
    }

    //Returns seed of world
    pub fn get_seed(&self) -> u32 {
        self.world_seed
    }

    //Returns adjacent chunks
    pub fn get_adjacent(&self, chunk: &Chunk) -> [Option<&Chunk>; 6] {
        let pos = chunk.get_chunk_pos();
        [
            self.get_chunk(pos.x, pos.y + 1, pos.z),
            self.get_chunk(pos.x, pos.y - 1, pos.z),
            self.get_chunk(pos.x - 1, pos.y, pos.z),
            self.get_chunk(pos.x + 1, pos.y, pos.z),
            self.get_chunk(pos.x, pos.y, pos.z - 1),
            self.get_chunk(pos.x, pos.y, pos.z + 1),
        ]
    }
}
