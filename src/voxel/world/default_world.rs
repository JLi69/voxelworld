/*
 * Default world generation
 * */

mod gen_trees;
mod ore;
mod plants;
mod terrain;

use self::{
    gen_trees::get_tree_gen_info,
    ore::{generate_magma_blocks, generate_ore},
    plants::{generate_plants, generate_sugarcane, get_plant_positions, get_water_adjacent},
};
use crossbeam::{queue::ArrayQueue, thread};
use std::collections::{HashMap, HashSet};

use super::{
    gen_more::{find_in_range, get_chunks_to_generate},
    World, WorldGenerator,
};
use crate::voxel::{
    region::{chunkpos_to_regionpos, Region},
    world::gen_more::update_chunk_tables,
    Block, Chunk, CHUNK_SIZE_F32, INDESTRUCTIBLE, EMPTY_BLOCK,
};
use crate::{gfx::ChunkTables, voxel::CHUNK_SIZE_I32};
use cgmath::Vector3;
use gen_trees::generate_trees;
use terrain::{add_to_heightmap, generate_heightmap, is_noise_cave, HeightMap};

const SEA_LEVEL: i32 = 0;
const SAND_LEVEL: i32 = SEA_LEVEL + 1;
const LAVA_LEVEL: i32 = -50;
const BOTTOM_OF_WORLD: i32 = -64;

struct GenInfoTable {
    heightmap: HeightMap,
    tree_positions: HashMap<(i32, i32), Vec<(i32, i32)>>,
    tree_heights: HashMap<(i32, i32), Vec<i32>>,
    plant_positions: HashMap<(i32, i32), Vec<(i32, i32)>>,
    sugarcane_positions: HashMap<(i32, i32), Vec<(i32, i32)>>,
}

struct GenInfo<'a> {
    heights: &'a [i32],
    tree_positions: &'a [(i32, i32)],
    tree_heights: &'a [i32],
    plant_positions: &'a [(i32, i32)],
    sugarcane_positions: &'a [(i32, i32)],
}

impl GenInfoTable {
    fn new() -> Self {
        Self {
            heightmap: HeightMap::new(),
            tree_positions: HashMap::new(),
            tree_heights: HashMap::new(),
            plant_positions: HashMap::new(),
            sugarcane_positions: HashMap::new(),
        }
    } 

    fn generate_heights(
        &mut self,
        positions: &Vec<(i32, i32, i32)>,
        world_generator: &WorldGenerator,
    ) {
        self.heightmap = generate_heightmap(
            positions,
            &world_generator.terrain_generator,
            &world_generator.elevation,
            &world_generator.steepness,
        );
    }

    fn add_heights(&mut self, x: i32, z: i32, world_generator: &WorldGenerator) {
        add_to_heightmap(
            x,
            z,
            &mut self.heightmap,
            &world_generator.terrain_generator,
            &world_generator.elevation,
            &world_generator.steepness,
        );
    }

    fn generate_trees(
        &mut self,
        positions: &Vec<(i32, i32, i32)>,
        world_generator: &WorldGenerator,
    ) {
        for (chunkx, _, chunkz) in positions {
            let pos = (*chunkx, *chunkz);
            if self.tree_positions.contains_key(&pos) {
                continue;
            }
            let (tree_pos, tree_h) = get_tree_gen_info(*chunkx, *chunkz, world_generator);
            self.tree_positions.insert(pos, tree_pos);
            self.tree_heights.insert(pos, tree_h);
        }
    }

    fn add_trees(&mut self, x: i32, z: i32, world_generator: &WorldGenerator) {
        if self.tree_positions.contains_key(&(x, z)) {
            return;
        }

        let (tree_pos, tree_h) = get_tree_gen_info(x, z, world_generator);
        self.tree_positions.insert((x, z), tree_pos);
        self.tree_heights.insert((x, z), tree_h);
    }

    fn generate_plants(
        &mut self,
        positions: &Vec<(i32, i32, i32)>,
        world_generator: &WorldGenerator,
    ) {
        for (chunkx, _, chunkz) in positions {
            let pos = (*chunkx, *chunkz);
            if self.plant_positions.contains_key(&pos) {
                continue;
            }
            let plants = get_plant_positions(*chunkx, *chunkz, world_generator.world_seed);
            self.plant_positions.insert(pos, plants);
        }
    }

    fn add_plants(&mut self, x: i32, z: i32, world_generator: &WorldGenerator) {
        if self.plant_positions.contains_key(&(x, z)) {
            return;
        }
        let plants = get_plant_positions(x, z, world_generator.world_seed);
        self.plant_positions.insert((x, z), plants);
    }

    //Assumes that heightmap has already been generated
    fn generate_sugarcane(&mut self, positions: &Vec<(i32, i32, i32)>) {
        for (chunkx, _, chunkz) in positions {
            let pos = (*chunkx, *chunkz);
            if self.sugarcane_positions.contains_key(&pos) {
                continue;
            }
            let heights = self.heightmap.get(&pos);
            if let Some(heights) = heights {
                let water_adjacent = get_water_adjacent(*chunkx, *chunkz, heights);
                self.sugarcane_positions.insert(pos, water_adjacent);
            }
        }
    }

    //Assumes that (x, z) has been generated
    fn add_sugarcane(&mut self, x: i32, z: i32) {
        if self.sugarcane_positions.contains_key(&(x, z)) {
            return;
        }
        let heights = self.heightmap.get(&(x, z));
        if let Some(heights) = heights {
            let water_adjacent = get_water_adjacent(x, z, heights);
            self.sugarcane_positions.insert((x, z), water_adjacent);
        }
    }

    fn get(&self, x: i32, z: i32) -> Option<GenInfo> {
        let h = self.heightmap.get(&(x, z))?;
        let trees = self.tree_positions.get(&(x, z))?;
        let tree_h = self.tree_heights.get(&(x, z))?;
        let plants = self.plant_positions.get(&(x, z))?;
        let sugarcane = self.sugarcane_positions.get(&(x, z))?;

        Some(GenInfo {
            heights: h,
            tree_positions: trees,
            tree_heights: tree_h,
            plant_positions: plants,
            sugarcane_positions: sugarcane,
        })
    }
}

fn get_surface_block(temperature: f64) -> Block {
    if temperature > 0.75 {
        //Desert, sand
        Block::new_id(11)
    } else if temperature < 0.25 {
        //Cold, snowy grass
        Block::new_id(87)
    } else {
        Block::new_id(1)
    }
}

//Blocks such as dirt right beneath the surface layer
fn get_under_block(temperature: f64) -> Block {
    if temperature > 0.75 {
        //Desert, sand
        Block::new_id(11)
    } else {
        //Dirt
        Block::new_id(4)
    }
}

fn gen_chunk(chunk: &mut Chunk, gen_info: GenInfo, world_generator: &WorldGenerator) {
    let chunkpos = chunk.get_chunk_pos();
    let posx = chunkpos.x * CHUNK_SIZE_I32;
    let posy = chunkpos.y * CHUNK_SIZE_I32;
    let posz = chunkpos.z * CHUNK_SIZE_I32;

    if chunkpos.y < -4 || chunkpos.y > 5 {
        return;
    }

    let seed = ((chunkpos.x as u64) << 32) | (chunkpos.z as u64);
    let mut rng = fastrand::Rng::with_seed(seed + world_generator.world_seed as u64);
    let ore_seed = ((chunkpos.x as u64) << 48) | ((chunkpos.y as u64) << 16) | (chunkpos.z as u64);
    let mut ore_rng =
        fastrand::Rng::with_seed(ore_seed + ((world_generator.world_seed as u64) << 16));

    for x in posx..(posx + CHUNK_SIZE_I32) {
        for z in posz..(posz + CHUNK_SIZE_I32) {
            let index = ((z - posz) * CHUNK_SIZE_I32 + (x - posx)) as usize;
            let height = gen_info.heights[index];
            let h = (height + 1).max(SEA_LEVEL + 1);

            if h + 1 < posy {
                continue;
            }

            let temperature = world_generator.get_temperature(x, z);

            for y in posy..(posy + CHUNK_SIZE_I32).min(h + 1) {
                let indestructible = (y == BOTTOM_OF_WORLD)
                    || (y == BOTTOM_OF_WORLD + 1 && rng.i32(0..4) < 2)
                    || (y == BOTTOM_OF_WORLD + 2 && rng.i32(0..6) == 0);
                if indestructible {
                    //Bottom of the world
                    chunk.set_block(x, y, z, Block::new_id(INDESTRUCTIBLE));
                    continue;
                }

                if y > BOTTOM_OF_WORLD && y <= LAVA_LEVEL {
                    chunk.set_block(x, y, z, Block::new_fluid(13));
                }

                if y <= SEA_LEVEL && y > height {
                    chunk.set_block(x, y, z, Block::new_fluid(12));
                    if y == SEA_LEVEL && temperature < 0.25 {
                        //Ice on top of water if we are in a cold biome
                        chunk.set_block(x, y, z, Block::new_id(85));
                    }
                }

                //Sand
                if y > height - 4 && y <= height && height <= SAND_LEVEL {
                    chunk.set_block(x, y, z, Block::new_id(11));
                    continue;
                }

                //Generate noise caves
                if is_noise_cave(x, y, z, &world_generator.noise_cave_generator) {
                    continue;
                }

                if y == height + 1 && 
                    temperature < 0.25 && 
                    chunk.get_block(x, y, z).id == EMPTY_BLOCK 
                {
                    //Snow slabs in cold biomes
                    let mut snow_slab = Block::new_id(86);
                    snow_slab.set_shape(1);
                    chunk.set_block(x, y, z, snow_slab);
                } else if y == height {
                    let surface_block = get_surface_block(temperature);
                    chunk.set_block(x, y, z, surface_block);
                } else if y > height - 4 && y < height {
                    let under_block = get_under_block(temperature);
                    chunk.set_block(x, y, z, under_block);
                } else if y < height && y > -64 {
                    //Stone
                    chunk.set_block(x, y, z, Block::new_id(2));
                }
            }
        }
    }

    //Generate ore
    for x in posx..(posx + CHUNK_SIZE_I32) {
        for z in posz..(posz + CHUNK_SIZE_I32) {
            let index = ((z - posz) * CHUNK_SIZE_I32 + (x - posx)) as usize;
            let height = gen_info.heights[index];

            if height < posy {
                continue;
            }

            let h = height + 1;
            for y in posy..(posy + CHUNK_SIZE_I32).min(h) {
                generate_ore(chunk, x, y, z, &mut ore_rng);
                generate_magma_blocks(chunk, x, y, z, &mut ore_rng);
            }
        }
    }

    //Generate trees
    generate_trees(
        chunk,
        gen_info.tree_positions,
        gen_info.tree_heights,
        world_generator,
    );
    //Generate plants
    generate_plants(chunk, gen_info.plant_positions, &mut rng, world_generator);
    //Generate sugar cane
    generate_sugarcane(chunk, gen_info.sugarcane_positions, &mut rng);
}

impl World {
    //Generates a world
    pub fn gen_default(&mut self) {
        //Generate height map
        let positions = self.chunks.keys().copied().collect();
        let mut gen_info_table = GenInfoTable::new();
        gen_info_table.generate_heights(&positions, &self.world_generator);
        gen_info_table.generate_trees(&positions, &self.world_generator);
        gen_info_table.generate_plants(&positions, &self.world_generator);
        gen_info_table.generate_sugarcane(&positions);

        for chunk in &mut self.chunks.values_mut() {
            let pos = chunk.get_chunk_pos();
            if let Some(gen_info) = gen_info_table.get(pos.x, pos.z) {
                gen_chunk(chunk, gen_info, &self.world_generator);
            }
        }
    }

    //Generates more chunks
    pub fn gen_more_default(&mut self, pos: Vector3<f32>, chunktables: &mut ChunkTables) {
        //Check if the player is in the center chunk
        let x = (pos.x / CHUNK_SIZE_F32).floor() as i32;
        let y = (pos.y / CHUNK_SIZE_F32).floor() as i32;
        let z = (pos.z / CHUNK_SIZE_F32).floor() as i32;
        if x == self.centerx && y == self.centery && z == self.centerz {
            return;
        }

        //Find all chunks within range of the player
        let (in_range, out_of_range) = find_in_range(&self.chunks, x, y, z, self.range);
        //Find chunks to generate
        let to_generate = get_chunks_to_generate(in_range, x, y, z, self.range);

        //Delete old chunks
        self.delete_out_of_range(&out_of_range);

        //Set the center position
        self.centerx = x;
        self.centery = y;
        self.centerz = z;

        //Load chunks from cache
        let start = std::time::Instant::now();
        for (chunkx, chunky, chunkz) in &to_generate {
            let pos = (*chunkx, *chunky, *chunkz);
            if let Some(new_chunk) = self.chunk_cache.get(&pos) {
                self.chunks.insert(pos, new_chunk.clone());
                self.chunk_cache.remove(&pos);
            }
        }
        eprintln!(
            "Took {} ms to load chunks from cache",
            start.elapsed().as_millis()
        );

        //Load regions from file system
        let start = std::time::Instant::now();
        let mut regions_loaded = 0;
        let mut loaded = HashSet::new();
        for (chunkx, chunky, chunkz) in &to_generate {
            let (rx, ry, rz) = chunkpos_to_regionpos(*chunkx, *chunky, *chunkz);
            if self.chunks.contains_key(&(*chunkx, *chunky, *chunkz)) {
                continue;
            }

            if loaded.contains(&(rx, ry, rz)) {
                continue;
            }
            loaded.insert((rx, ry, rz));

            if let Some(region) = Region::load_region(&self.path, rx, ry, rz) {
                regions_loaded += 1;
                self.add_region(region);
            }
        }
        eprintln!(
            "Took {} ms to load {regions_loaded} regions from filesystem",
            start.elapsed().as_millis()
        );

        //Generate new chunks
        let mut gen_info_table = GenInfoTable::new();
        let start = std::time::Instant::now();
        for (chunkx, chunky, chunkz) in &to_generate {
            let pos = (*chunkx, *chunky, *chunkz);
            if self.chunks.contains_key(&pos) {
                continue;
            }
            if *chunky < -4 || *chunky > 4 {
                continue;
            }
            gen_info_table.add_heights(*chunkx, *chunkz, &self.world_generator);
            gen_info_table.add_trees(*chunkx, *chunkz, &self.world_generator);
            gen_info_table.add_plants(*chunkx, *chunkz, &self.world_generator);
            gen_info_table.add_sugarcane(*chunkx, *chunkz);
        }
        eprintln!(
            "Took {} ms to generate chunk info",
            start.elapsed().as_millis()
        );

        let start = std::time::Instant::now();
        let generated = ArrayQueue::new(to_generate.len());
        let mut generated_count = 0;
        thread::scope(|s| {
            for (chunkx, chunky, chunkz) in &to_generate {
                if self.chunks.contains_key(&(*chunkx, *chunky, *chunkz)) {
                    continue;
                }

                generated_count += 1;

                s.spawn(|_| {
                    let mut new_chunk = Chunk::new(*chunkx, *chunky, *chunkz);
                    //Should always evaluate to true
                    if let Some(gen_info) = gen_info_table.get(*chunkx, *chunkz) {
                        gen_chunk(&mut new_chunk, gen_info, &self.world_generator);
                    }
                    //This should never fail
                    generated
                        .push(new_chunk)
                        .expect("Error: Failed to push onto ArrayQueue");
                });
            }
        })
        .expect("Failed to generate new chunks!");

        for chunk in generated {
            let chunkpos = chunk.get_chunk_pos();
            let pos = (chunkpos.x, chunkpos.y, chunkpos.z);
            self.chunks.insert(pos, chunk);
        }
        eprintln!(
            "Took {} ms to generate {generated_count} new chunks",
            start.elapsed().as_millis()
        );

        let mut update_list = to_generate.clone();
        update_list.extend(self.init_light_new_chunks(&to_generate));

        update_chunk_tables(
            chunktables,
            self.centerx,
            self.centery,
            self.centerz,
            self.range,
            &self.chunks,
            &update_list,
        );
    }

    //Generates any missing chunks on load
    pub fn gen_default_on_load(&mut self) {
        let mut to_generate = HashSet::new();
        for y in (self.centery - self.range)..=(self.centery + self.range) {
            for z in (self.centerz - self.range)..=(self.centerz + self.range) {
                for x in (self.centerx - self.range)..=(self.centerx + self.range) {
                    if self.chunks.contains_key(&(x, y, z)) {
                        continue;
                    }
                    to_generate.insert((x, y, z));
                }
            }
        }

        if to_generate.is_empty() {
            return;
        }

        //Generate new chunks
        let mut gen_info_table = GenInfoTable::new();
        let start = std::time::Instant::now();
        for (chunkx, chunky, chunkz) in &to_generate {
            let pos = (*chunkx, *chunky, *chunkz);
            if self.chunks.contains_key(&pos) {
                continue;
            }
            if *chunky < -4 || *chunky > 4 {
                continue;
            }
            gen_info_table.add_heights(*chunkx, *chunkz, &self.world_generator);
            gen_info_table.add_trees(*chunkx, *chunkz, &self.world_generator);
            gen_info_table.add_plants(*chunkx, *chunkz, &self.world_generator);
            gen_info_table.add_sugarcane(*chunkx, *chunkz);
        }
        eprintln!(
            "Took {} ms to generate chunk info",
            start.elapsed().as_millis()
        );

        let start = std::time::Instant::now();
        let generated = ArrayQueue::new(to_generate.len());
        let mut generated_count = 0;
        thread::scope(|s| {
            for (chunkx, chunky, chunkz) in &to_generate {
                if self.chunks.contains_key(&(*chunkx, *chunky, *chunkz)) {
                    continue;
                }

                generated_count += 1;

                s.spawn(|_| {
                    let mut new_chunk = Chunk::new(*chunkx, *chunky, *chunkz);
                    //Should always evaluate to true
                    if let Some(gen_info) = gen_info_table.get(*chunkx, *chunkz) {
                        gen_chunk(&mut new_chunk, gen_info, &self.world_generator);
                    }
                    //This should never fail
                    generated
                        .push(new_chunk)
                        .expect("Error: Failed to push onto ArrayQueue");
                });
            }
        })
        .expect("Failed to generate new chunks!");

        for chunk in generated {
            let chunkpos = chunk.get_chunk_pos();
            let pos = (chunkpos.x, chunkpos.y, chunkpos.z);
            self.chunks.insert(pos, chunk);
        }
        eprintln!(
            "Took {} ms to generate {generated_count} new chunks",
            start.elapsed().as_millis()
        );
    }
}
