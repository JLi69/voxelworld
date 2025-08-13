pub mod block_update;
mod default_world;
mod flat_world;
mod gen_more;
pub mod light;
mod old_world;
mod save;
use crate::{game::GameMode, gfx::display::get_sky_brightness};

use super::{
    light::{Light, SkyLightMap, LU},
    region::{chunkpos_to_regionpos, get_region_chunks, get_region_chunks_remove, Region},
    tile_data::TileData,
    world_to_chunk_position, wrap_coord, Block, Chunk, CHUNK_SIZE_I32, FULL_BLOCK,
};
use gen_more::LoadChunkQueue;
use noise::{Fbm, NoiseFn, Perlin, Simplex};
use std::collections::{HashMap, HashSet};

pub const OCTAVES: usize = 5;
pub const PERSISTENCE: f64 = 0.5;
const DEFAULT_TIME: f32 = 0.04;
const MINUTES_PER_DAY: f32 = 20.0;
const DAY_NIGHT_SPEED: f32 = 1.0 / (MINUTES_PER_DAY * 60.0);

//Struct that contains information for generating the world
pub struct WorldGenerator {
    pub terrain_generator: Fbm<Perlin>,
    pub noise_cave_generator: Perlin,
    pub tree_generator: Perlin,
    pub steepness: Perlin,
    //For world generation 2.0
    pub elevation: Fbm<Perlin>,
    pub temperature: Perlin,
    pub mountain: Fbm<Simplex>,
    world_seed: u32,
}

impl WorldGenerator {
    fn new(seed: u32) -> Self {
        let mut terrain_noise = Fbm::new(seed);
        terrain_noise.octaves = OCTAVES;
        terrain_noise.persistence = PERSISTENCE;

        let mut elevation_noise = Fbm::new(seed + 5);
        elevation_noise.octaves = 3;
        elevation_noise.persistence = 0.25;

        let mut mountain_noise = Fbm::new(seed + 6);
        mountain_noise.octaves = 4;
        mountain_noise.persistence = 0.6;

        Self {
            terrain_generator: terrain_noise,
            noise_cave_generator: Perlin::new(seed + 1),
            tree_generator: Perlin::new(seed + 2),
            steepness: Perlin::new(seed + 3),
            elevation: elevation_noise,
            temperature: Perlin::new(seed + 4),
            mountain: mountain_noise,
            world_seed: seed,
        }
    }

    pub fn get_temperature(&self, x: i32, z: i32) -> f64 {
        let offset = (self.world_seed % 2560) as f64 / 1280.0;
        let point = [x as f64 / 480.0 + offset, z as f64 / 480.0 + offset];
        self.temperature.get(point) * 0.5 + 0.5
    }

    pub fn get_base_elevation(&self, x: i32, z: i32) -> f64 {
        let point = [x as f64 / 512.0, z as f64 / 512.0];
        self.terrain_generator.get(point)
    }

    pub fn get_elevation(&self, x: i32, z: i32) -> f64 {
        let point = [x as f64 / 16.0, z as f64 / 16.0];
        self.elevation.get(point) * 0.5 + 0.5
    }

    pub fn get_steepness(&self, x: i32, z: i32) -> f64 {
        let point = [x as f64 / 384.0, z as f64 / 384.0];
        let val = self.steepness.get(point) * 0.5 + 0.5;
        val.abs().powf((1.0 - val) * 3.0)
    }

    pub fn get_mountain(&self, x: i32, z: i32) -> f64 {
        let point = [x as f64 / 384.0, z as f64 / 384.0];
        let normalized = self.mountain.get(point) * 0.5 + 0.5;
        let clamped = ((normalized - 0.3) / 0.7).clamp(0.0, 1.0);
        clamped * 2.0 - 1.0
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum WorldGenType {
    OldGen,
    Flat,
    DefaultGen,
}

//World struct
pub struct World {
    //This only stores chunks that are near to the player
    pub chunks: HashMap<(i32, i32, i32), Chunk>,
    //The y coordinate of the tallest block in a chunk
    skylightmap: HashMap<(i32, i32), SkyLightMap>,
    //Maximum range for chunks that should be loaded
    range: i32,
    //Position of the center chunk
    centerx: i32,
    centery: i32,
    centerz: i32,
    //cache of chunks that have been unloaded
    pub chunk_cache: HashMap<(i32, i32, i32), Chunk>,
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
    //Day/night cycle
    pub time: f32, //A number between 0.0 and 1.0
    pub days_passed: u64,
    //Chunks that experienced block update and need to be saved
    to_save: HashSet<(i32, i32, i32)>,
    //Chunks that are to be removed from cache and need to be saved
    removed_from_cache: Vec<Region>,
    //List of chunk coordinates that are too be loaded
    to_load: LoadChunkQueue,
    //This is a list of chunks that should be updated in the chunk vao table,
    //this is added to whenever new chunks are generated and is emptied and 'sent'
    //to the chunktable for updates when we are done loading all the chunks we
    //need to load at the moment (to_load.is_empty()) This allows for the
    //chunk vao updates to appear faster since updating chunks also updates
    //the neighbors of the chunks and just immediately putting this data in the
    //update queue will result in a lot of duplicate chunk updates that aren't needed.
    chunktable_update_list: HashSet<(i32, i32, i32)>,
    //World game mode
    pub game_mode: GameMode,
}

impl World {
    //Create an empty world
    pub fn empty() -> Self {
        Self {
            chunks: HashMap::new(),
            skylightmap: HashMap::new(),
            range: 0,
            centerx: 0,
            centery: 0,
            centerz: 0,
            chunk_cache: HashMap::new(),
            world_generator: WorldGenerator::new(0),
            gen_type: WorldGenType::OldGen,
            world_seed: 0,
            path: String::new(),
            block_update_timer: 0.0,
            random_update_timer: 0.0,
            updating: HashSet::new(),
            in_update_range: HashSet::new(),
            ticks: 0,
            time: DEFAULT_TIME,
            days_passed: 0,
            to_save: HashSet::new(),
            removed_from_cache: vec![],
            to_load: LoadChunkQueue::new(),
            chunktable_update_list: HashSet::new(),
            game_mode: GameMode::Creative, //Default to creative mode
        }
    }

    //Create a new chunk from a chunk render distance (range)
    pub fn new(seed: u32, chunk_range: i32, generation: WorldGenType, mode: GameMode) -> Self {
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
            skylightmap: HashMap::new(),
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
            time: DEFAULT_TIME,
            days_passed: 0,
            to_save: HashSet::new(),
            removed_from_cache: vec![],
            to_load: LoadChunkQueue::new(),
            chunktable_update_list: HashSet::new(),
            game_mode: mode,
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

    //Returns the light value for a position
    //Returns black if the position is out of range in the world
    pub fn get_light(&self, x: i32, y: i32, z: i32) -> Light {
        let (chunkx, chunky, chunkz) = world_to_chunk_position(x, y, z);
        let chunk = self.get_chunk(chunkx, chunky, chunkz);
        if let Some(chunk) = chunk {
            return chunk.get_light(x, y, z);
        }
        Light::black()
    }

    pub fn get_client_light(&self, x: i32, y: i32, z: i32) -> (f32, f32, f32) {
        let light = self.get_light(x, y, z);
        let skylight = light.skylight() as f32 / 15.0 * get_sky_brightness(self.time);
        let r = (light.r() as f32 / 15.0).max(skylight);
        let g = (light.g() as f32 / 15.0).max(skylight);
        let b = (light.b() as f32 / 15.0).max(skylight);
        (r, g, b)
    }

    //Updates light at a position
    //Does nothing if the position is out of range for the world
    pub fn update_light(&mut self, x: i32, y: i32, z: i32, update: LU) {
        let (chunkx, chunky, chunkz) = world_to_chunk_position(x, y, z);
        let chunk = self.get_mut_chunk(chunkx, chunky, chunkz);
        if let Some(chunk) = chunk {
            chunk.update_light(x, y, z, update);
        }
    }

    fn get_max_cache_sz(&self) -> usize {
        let sz = self.range * 2 + 1;
        (sz * sz * 40) as usize
    }

    //If the cache gets too large, attempt to delete some sections
    pub fn clean_cache(&mut self) {
        if !self.removed_from_cache.is_empty() {
            eprintln!("Cleaning cache...");
            eprintln!("Regions left to save: {}", self.removed_from_cache.len());
        }

        //Keep track of the loaded regions
        let mut active_regions = HashSet::new();
        for (x, y, z) in self.chunks.keys() {
            let regionpos = chunkpos_to_regionpos(*x, *y, *z);
            active_regions.insert(regionpos);
        }

        //Remove regions and save them to the harddrive
        let mut time_passed = 0.0;
        let start = std::time::Instant::now();
        while !self.removed_from_cache.is_empty() && time_passed < 0.0005 {
            let region_to_save = self.removed_from_cache.last();
            if let Some(region_to_save) = region_to_save {
                save::save_region(region_to_save, &self.path);
                self.removed_from_cache.pop();
            }
            let now = std::time::Instant::now();
            time_passed = (now - start).as_secs_f32();
        }

        //If we have not hit our maximum cache size, then don't bother attempting
        //to remove anything
        if self.chunk_cache.len() <= self.get_max_cache_sz() {
            return;
        }

        //Figure out which regions to remove
        let mut removed_regions = HashSet::new();
        let mut removed_count = 0;
        for (x, y, z) in self.chunk_cache.keys() {
            //Calculate the region position
            let regionpos = chunkpos_to_regionpos(*x, *y, *z);
            //If the region is loaded, do not attempt to remove it
            if active_regions.contains(&regionpos) {
                continue;
            }
            removed_regions.insert(regionpos);
            removed_count += 1;
            //Once we have removed half of the regions, we are done
            if removed_count >= self.get_max_cache_sz() / 2 {
                break;
            }
        }

        eprintln!("Starting cache size: {}", self.chunk_cache.len());
        //Generate region data and save it into the removed_from_cache list to
        //be saved to the disk later
        for (rx, ry, rz) in removed_regions {
            let mut region = Region::new(rx, ry, rz);
            //This should not run
            get_region_chunks(&mut region, &self.chunks);
            //This will take chunks from the cache and add it to the region
            //while also removing those chunks from the cache.
            get_region_chunks_remove(&mut region, &mut self.chunk_cache);
            self.removed_from_cache.push(region);
        }
        eprintln!("Final cache size: {}", self.chunk_cache.len());
    }

    //When a chunk gets unloaded, add it to a cache in case it needs to be reloaded
    pub fn add_to_chunk_cache(&mut self, chunk: Chunk) {
        let pos = chunk.get_chunk_pos();
        self.chunk_cache.insert((pos.x, pos.y, pos.z), chunk);
    }

    //Generate world
    pub fn generate_world(&mut self) {
        match self.gen_type {
            WorldGenType::OldGen => self.gen_old(),
            WorldGenType::Flat => self.gen_flat(),
            WorldGenType::DefaultGen => self.gen_default(),
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

    //Updates day night cycle
    pub fn update_daynight(&mut self, dt: f32) {
        self.time += dt * DAY_NIGHT_SPEED;
        if self.time > 1.0 {
            self.time = 0.0;
            self.days_passed += 1;
        }
    }

    //Returns true if the position is not located in any chunk in the world
    //Returns false otherwise
    pub fn out_of_bounds(&self, x: i32, y: i32, z: i32) -> bool {
        let pos = world_to_chunk_position(x, y, z);
        !self.chunks.contains_key(&pos)
    }

    //Returns the chunk coordinates of the center of the world
    pub fn get_center(&self) -> (i32, i32, i32) {
        (self.centerx, self.centery, self.centerz)
    }

    pub fn is_loaded(&self, chunkpos: (i32, i32, i32)) -> bool {
        self.chunks.contains_key(&chunkpos) || self.chunk_cache.contains_key(&chunkpos)
    }

    //None clears the tile data at that position
    pub fn set_tile_data(&mut self, x: i32, y: i32, z: i32, tile_data: Option<TileData>) {
        let (chunkx, chunky, chunkz) = world_to_chunk_position(x, y, z);
        if let Some(chunk) = self.chunks.get_mut(&(chunkx, chunky, chunkz)) {
            chunk.set_tile_data(x, y, z, tile_data);
        }
    }

    pub fn get_tile_data(&self, x: i32, y: i32, z: i32) -> Option<TileData> {
        let (chunkx, chunky, chunkz) = world_to_chunk_position(x, y, z);
        let chunk = self.chunks.get(&(chunkx, chunky, chunkz))?;
        chunk.get_tile_data(x, y, z)
    }

    pub fn init_tile_data(&mut self, x: i32, y: i32, z: i32) {
        let block = self.get_block(x, y, z);
        //Non-full blocks can not have any tile data
        if block.shape() != FULL_BLOCK {
            return;
        }
        if self.get_tile_data(x, y, z).is_some() {
            return;
        }
        match block.id {
            //Chest
            37 => {
                self.set_tile_data(x, y, z, Some(TileData::new_chest()));
            }
            //Furnace
            40 | 70 => {
                self.set_tile_data(x, y, z, Some(TileData::new_furance()));
            }
            _ => {}
        }
    }
}

pub fn get_simulation_dist(world: &World) -> i32 {
    (world.get_range() / 2 + 1)
        .min(7)
        .min(world.get_range() - 1)
}

pub fn in_sim_range(center: (i32, i32, i32), pos: (i32, i32, i32), sim_dist: i32) -> bool {
    let (centerx, centery, centerz) = center;
    let (x, y, z) = pos;
    (x - centerx).abs() <= sim_dist
        && (y - centery).abs() <= sim_dist
        && (z - centerz).abs() <= sim_dist
}
