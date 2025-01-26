pub mod block_update;
mod default_world;
mod flat_world;
mod gen_more;
mod save;

use super::{world_to_chunk_position, wrap_coord, Block, Chunk, CHUNK_SIZE_I32};
use crate::gfx::ChunkTables;
use cgmath::Vector3;
use noise::{Fbm, Perlin};
use std::collections::{HashMap, HashSet};

pub const OCTAVES: usize = 5;
pub const PERSISTENCE: f64 = 0.47;

//Struct that contains information for generating the world
struct WorldGenerator {
    pub terrain_generator: Fbm<Perlin>,
    pub noise_cave_generator: Perlin,
    pub tree_generator: Perlin,
    pub steepness: Perlin,
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
            steepness: Perlin::new(seed + 3),
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
    world_generator: WorldGenerator,
    world_seed: u32,
    pub gen_type: WorldGenType,
    //World path
    pub path: String,
    //Block update timer
    block_update_timer: f32,
    random_update_timer: f32,
    //Updating chunks
    updating: HashSet<(i32, i32, i32)>,
    in_update_range: HashSet<(i32, i32, i32)>,
    ticks: u64,
    //Chunks that experienced block update and need to be saved
    to_save: HashSet<(i32, i32, i32)>,
    //Chunks that are to be removed from cache and need to be saved
    removed_from_cache: Vec<Chunk>,
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
            world_generator: WorldGenerator::new(0),
            gen_type: WorldGenType::DefaultGen,
            world_seed: 0,
            path: String::new(),
            block_update_timer: 0.0,
            random_update_timer: 0.0,
            updating: HashSet::new(),
            in_update_range: HashSet::new(),
            ticks: 0,
            to_save: HashSet::new(),
            removed_from_cache: vec![],
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
            world_generator: WorldGenerator::new(seed),
            gen_type: generation,
            world_seed: seed,
            path: String::new(),
            block_update_timer: 0.0,
            random_update_timer: 0.0,
            updating: HashSet::new(),
            in_update_range: HashSet::new(),
            ticks: 0,
            to_save: HashSet::new(),
            removed_from_cache: vec![],
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

        let ix = wrap_coord(x);
        let iy = wrap_coord(y);
        let iz = wrap_coord(z);
        for dx in -1i32..=1 {
            for dy in -1i32..=1 {
                for dz in -1i32..=1 {
                    if (dx == -1 && ix != 0) || (dx == 1 && ix != CHUNK_SIZE_I32 - 1) {
                        continue;
                    }

                    if (dy == -1 && iy != 0) || (dy == 1 && iy != CHUNK_SIZE_I32 - 1) {
                        continue;
                    }

                    if (dz == -1 && iz != 0) || (dz == 1 && iz != CHUNK_SIZE_I32 - 1) {
                        continue;
                    }

                    self.updating
                        .insert((chunkx + dx, chunky + dy, chunkz + dz));
                }
            }
        }

        self.to_save.insert((chunkx, chunky, chunkz));
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

    //If the cache gets too large, attempt to delete some sections
    pub fn clean_cache(&mut self) {
        if !self.removed_from_cache.is_empty() {
            eprintln!("Cleaning cache...");
            eprintln!(
                "Chunks left to save: {} | Chunk cache size: {}",
                self.removed_from_cache.len(),
                self.chunk_cache.len()
            );
        }

        let mut time_passed = 0.0;
        let start = std::time::Instant::now();
        while !self.removed_from_cache.is_empty() && time_passed < 0.0005 {
            let chunk_to_save = self.removed_from_cache.last();
            if let Some(chunk) = chunk_to_save {
                save::save_chunk(chunk, &self.path);
                self.removed_from_cache.pop();
            }
            let now = std::time::Instant::now();
            time_passed = (now - start).as_secs_f32();
        }

        let max_cache_sz = self.get_max_cache_sz();
        if self.chunk_cache.len() <= max_cache_sz {
            return;
        }

        while self.chunk_cache.len() > max_cache_sz / 2 {
            let to_delete = self.chunk_cache.keys().next().copied();
            if let Some(pos) = to_delete {
                if let Some(chunk) = self.chunk_cache.get(&pos) {
                    self.removed_from_cache.push(chunk.clone());
                    self.chunk_cache.remove(&pos);
                }
            }
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

    //Returns how many chunks are updating
    pub fn get_chunk_updates(&self) -> usize {
        self.updating.len()
    }
}
