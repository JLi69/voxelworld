/*
 * Default world generation
 * */

mod gen_trees;
mod plants;
mod terrain;

use self::{
    gen_trees::get_tree_gen_info,
    plants::{generate_plants, get_plant_positions},
};
use std::collections::HashMap;

use super::{
    gen_more::{find_in_range, get_chunks_to_generate, update_chunk_vao_table},
    World, WorldGenerator,
};
use crate::voxel::{Block, Chunk, CHUNK_SIZE_F32, INDESTRUCTIBLE};
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
}

struct GenInfo<'a> {
    heights: &'a [i32],
    tree_positions: &'a [(i32, i32)],
    tree_heights: &'a [i32],
    plant_positions: &'a [(i32, i32)],
}

impl GenInfoTable {
    fn new() -> Self {
        Self {
            heightmap: HeightMap::new(),
            tree_positions: HashMap::new(),
            tree_heights: HashMap::new(),
            plant_positions: HashMap::new(),
        }
    }

    fn generate_heights(
        &mut self,
        positions: &Vec<(i32, i32, i32)>,
        world_generator: &WorldGenerator,
    ) {
        self.heightmap = generate_heightmap(positions, &world_generator.terrain_generator);
    }

    fn add_heights(&mut self, x: i32, z: i32, world_generator: &WorldGenerator) {
        add_to_heightmap(
            x,
            z,
            &mut self.heightmap,
            &world_generator.terrain_generator,
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

    fn get(&self, x: i32, z: i32) -> Option<GenInfo> {
        let h = self.heightmap.get(&(x, z))?;
        let trees = self.tree_positions.get(&(x, z))?;
        let tree_h = self.tree_heights.get(&(x, z))?;
        let plants = self.plant_positions.get(&(x, z))?;

        Some(GenInfo {
            heights: h,
            tree_positions: trees,
            tree_heights: tree_h,
            plant_positions: plants,
        })
    }
}

fn gen_chunk(chunk: &mut Chunk, gen_info: GenInfo, world_generator: &WorldGenerator) {
    let chunkpos = chunk.get_chunk_pos();
    let posx = chunkpos.x * CHUNK_SIZE_I32;
    let posy = chunkpos.y * CHUNK_SIZE_I32;
    let posz = chunkpos.z * CHUNK_SIZE_I32;

    if chunkpos.y < -2 || chunkpos.y > 2 {
        return;
    }

    let seed = ((chunkpos.x as u64) << 32) | (chunkpos.z as u64);
    let mut rng = fastrand::Rng::with_seed(seed + world_generator.world_seed as u64);

    for x in posx..(posx + CHUNK_SIZE_I32) {
        for z in posz..(posz + CHUNK_SIZE_I32) {
            let index = ((z - posz) * CHUNK_SIZE_I32 + (x - posx)) as usize;
            let height = gen_info.heights[index];

            if height < posy {
                continue;
            }

            let h = (height + 1).max(SEA_LEVEL + 1);
            for y in posy..(posy + CHUNK_SIZE_I32).min(h) {
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
                } else if y <= SEA_LEVEL && y > height {
                    chunk.set_block(x, y, z, Block::new_fluid(12));
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

                if y == height {
                    //Grass
                    chunk.set_block(x, y, z, Block::new_id(1));
                } else if y > height - 4 && y < height {
                    //Dirt
                    chunk.set_block(x, y, z, Block::new_id(4));
                } else if y < height && y > -64 {
                    //Stone
                    chunk.set_block(x, y, z, Block::new_id(2));
                }
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

        //Generate height map
        let mut gen_info_table = GenInfoTable::new();

        //Generate new chunks
        for (chunkx, chunky, chunkz) in &to_generate {
            let pos = (*chunkx, *chunky, *chunkz);
            self.updating.insert(pos);
            if self.chunk_cache.contains_key(&pos) {
                let new_chunk = self.chunk_cache.get(&pos);
                if let Some(new_chunk) = new_chunk {
                    self.chunks.insert(pos, new_chunk.clone());
                    self.chunk_cache.remove(&pos);
                }
                continue;
            } else if let Some(chunk) = Chunk::load_chunk(&self.path, *chunkx, *chunky, *chunkz) {
                self.chunks.insert(pos, chunk);
                continue;
            }

            gen_info_table.add_heights(*chunkx, *chunkz, &self.world_generator);
            gen_info_table.add_trees(*chunkx, *chunkz, &self.world_generator);
            gen_info_table.add_plants(*chunkx, *chunkz, &self.world_generator);
            let mut new_chunk = Chunk::new(*chunkx, *chunky, *chunkz);
            //Should always evaluate to true
            if let Some(gen_info) = gen_info_table.get(*chunkx, *chunkz) {
                gen_chunk(&mut new_chunk, gen_info, &self.world_generator);
            }
            self.chunks.insert(pos, new_chunk);
        }

        //Set the center position
        self.centerx = x;
        self.centery = y;
        self.centerz = z;

        update_chunk_vao_table(
            &mut chunktables.chunk_vaos,
            self.centerx,
            self.centery,
            self.centerz,
            self.range,
            &self.chunks,
            &to_generate,
        );

        update_chunk_vao_table(
            &mut chunktables.lava_vaos,
            self.centerx,
            self.centery,
            self.centerz,
            self.range,
            &self.chunks,
            &to_generate,
        );

        update_chunk_vao_table(
            &mut chunktables.water_vaos,
            self.centerx,
            self.centery,
            self.centerz,
            self.range,
            &self.chunks,
            &to_generate,
        );
    }
}
