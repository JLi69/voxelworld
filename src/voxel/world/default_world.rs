/*
 * Default world generation
 * */

use super::{
    gen_more::{find_in_range, get_chunks_to_generate, update_chunk_vao_table},
    World, WorldGenerator,
};
use crate::voxel::{Block, Chunk, CHUNK_SIZE_F32, INDESTRUCTIBLE};
use crate::{
    gfx::ChunkTables,
    voxel::{CHUNK_SIZE_I32, EMPTY_BLOCK},
};
use cgmath::Vector3;
use noise::{Fbm, NoiseFn, Perlin};
use std::collections::HashMap;

const SEA_LEVEL: i32 = 0;
const SAND_LEVEL: i32 = SEA_LEVEL + 1;
const LAVA_LEVEL: i32 = -50;
const BOTTOM_OF_WORLD: i32 = -64;

//cave_y1 = cave lower y bound
//cave_y2 = middle
//cave_y3 = cave upper y bound
fn cave_perc(y: i32, cave_y1: i32, cave_y2: i32, cave_y3: i32) -> f64 {
    if y > cave_y2 {
        let t = (y - cave_y2) as f64 / (cave_y3 - cave_y2) as f64;
        ((1.0 - t) * 0.8 - 1.0).max(-0.7)
    } else if y <= cave_y2 && y > cave_y1 {
        let t = (y - cave_y2) as f64 / (cave_y2 - cave_y1) as f64;
        (1.0 - t * t) * 0.8 - 1.0
    } else {
        -1.0
    }
}

fn is_noise_cave(x: i32, y: i32, z: i32, cave_noise: &Perlin) -> bool {
    let xyz = [x as f64 / 8.0, y as f64 / 8.0, z as f64 / 8.0];
    cave_noise.get(xyz) < cave_perc(y, -64, -51, 48)
}

fn gen_chunk(chunk: &mut Chunk, heights: &[i32], world_generator: &WorldGenerator) {
    let chunkpos = chunk.get_chunk_pos();
    let posx = chunkpos.x * CHUNK_SIZE_I32;
    let posy = chunkpos.y * CHUNK_SIZE_I32;
    let posz = chunkpos.z * CHUNK_SIZE_I32;

    if chunkpos.y < -2 || chunkpos.y > 2 {
        return;
    }

    for x in posx..(posx + CHUNK_SIZE_I32) {
        for z in posz..(posz + CHUNK_SIZE_I32) {
            let height = heights[((z - posz) * CHUNK_SIZE_I32 + (x - posx)) as usize];

            if height < posy {
                continue;
            }

            let h = (height + 1).max(SEA_LEVEL + 1);
            for y in posy..(posy + CHUNK_SIZE_I32).min(h) {
                if y == BOTTOM_OF_WORLD {
                    //Bottom of the world
                    chunk.set_block(x, y, z, Block::new_id(INDESTRUCTIBLE));
                    continue;
                }

                if y > BOTTOM_OF_WORLD && y <= LAVA_LEVEL {
                    chunk.set_block(x, y, z, Block::new_id(13));
                } else if y <= SEA_LEVEL && y > height {
                    chunk.set_block(x, y, z, Block::new_id(12));
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
    generate_trees(chunk, world_generator);
}

fn get_height(x: i32, z: i32, terrain_generator: &Fbm<Perlin>) -> i32 {
    let point = [x as f64 / 192.0, z as f64 / 192.0];
    let noise_height = terrain_generator.get(point);
    (noise_height * 47.0) as i32 + 16
}

fn gen_tree_positions(
    chunkx: i32,
    chunkz: i32,
    tree_noise: &Perlin,
    positions: &mut Vec<(i32, i32)>,
    heights: &mut Vec<i32>,
    world_seed: u32,
) {
    let xz = [chunkx as f64 + 0.5, 0.0, chunkz as f64 + 0.5];
    let noise_val = (tree_noise.get(xz) + 1.0) / 2.0;
    let tree_count = (noise_val * 24.0).floor() as u32;
    let xu32 = chunkx as u32;
    let zu32 = chunkz as u32;
    let seed = ((xu32 as u64) << 32) | (zu32 as u64);
    let mut tree_generator = fastrand::Rng::with_seed(seed);
    let mut rng = fastrand::Rng::with_seed(seed + world_seed as u64);
    for _ in 0..tree_count {
        let treex = (tree_generator.i32(0..32) + rng.i32(0..32)) % 32;
        let treez = (tree_generator.i32(0..32) + rng.i32(0..32)) % 32;
        let x = treex + chunkx * CHUNK_SIZE_I32;
        let z = treez + chunkz * CHUNK_SIZE_I32;
        let h = tree_generator.i32(4..=6);
        positions.push((x, z));
        heights.push(h);
    }
}

fn place_leaves(chunk: &mut Chunk, x: i32, y: i32, z: i32) {
    if chunk.get_block(x, y, z).id != EMPTY_BLOCK {
        return;
    }
    chunk.set_block(x, y, z, Block::new_id(7));
}

fn generate_leaves(chunk: &mut Chunk, starty: i32, x: i32, y: i32, z: i32, height: i32) {
    if y == starty + height {
        place_leaves(chunk, x, y, z);
        place_leaves(chunk, x - 1, y, z);
        place_leaves(chunk, x + 1, y, z);
        place_leaves(chunk, x, y, z - 1);
        place_leaves(chunk, x, y, z + 1);
    } else if y == starty + height - 1 {
        for ix in (x - 1)..=(x + 1) {
            for iz in (z - 1)..=(z + 1) {
                place_leaves(chunk, ix, y, iz);
            }
        }
    } else if y >= starty + height - 3 {
        for ix in (x - 2)..=(x + 2) {
            for iz in (z - 2)..=(z + 2) {
                place_leaves(chunk, ix, y, iz);
            }
        }
    }
}

fn generate_trees(chunk: &mut Chunk, world_generator: &WorldGenerator) {
    let pos = chunk.get_chunk_pos();
    let mut tree_positions = vec![];
    let mut tree_heights = vec![];
    for dx in -1..=1 {
        for dz in -1..=1 {
            gen_tree_positions(
                pos.x + dx,
                pos.z + dz,
                &world_generator.tree_generator,
                &mut tree_positions,
                &mut tree_heights,
                world_generator.world_seed,
            );
        }
    }

    for (i, (x, z)) in tree_positions.iter().enumerate() {
        let h = get_height(*x, *z, &world_generator.terrain_generator);

        //Below sea level
        if h <= SAND_LEVEL {
            continue;
        }

        //Check to make sure we are not in a cave (an empty block)
        if is_noise_cave(*x, h, *z, &world_generator.noise_cave_generator) {
            continue;
        }

        for y in (h + 1)..(h + 1 + tree_heights[i]) {
            //Generate trunk
            chunk.set_block(*x, y, *z, Block::new_id(8));

            //Generate leaves
            generate_leaves(chunk, h + 1, *x, y, *z, tree_heights[i]);
        }
        generate_leaves(
            chunk,
            h + 1,
            *x,
            h + 1 + tree_heights[i],
            *z,
            tree_heights[i],
        );
    }
}

type HeightMap = HashMap<(i32, i32), Vec<i32>>;

fn generate_heightmap(
    positions: &Vec<(i32, i32, i32)>,
    terrain_generator: &Fbm<Perlin>,
) -> HeightMap {
    let mut heightmap = HeightMap::new();

    for (chunkx, _, chunkz) in positions {
        if heightmap.contains_key(&(*chunkx, *chunkz)) {
            continue;
        }
        let posx = chunkx * CHUNK_SIZE_I32;
        let posz = chunkz * CHUNK_SIZE_I32;
        let mut heights = vec![0; (CHUNK_SIZE_I32 * CHUNK_SIZE_I32) as usize];
        for x in posx..(posx + CHUNK_SIZE_I32) {
            for z in posz..(posz + CHUNK_SIZE_I32) {
                let index = ((z - posz) * CHUNK_SIZE_I32 + (x - posx)) as usize;
                let h = get_height(x, z, terrain_generator);
                heights[index] = h;
            }
        }
        heightmap.insert((*chunkx, *chunkz), heights);
    }

    heightmap
}

fn add_to_heightmap(
    chunkx: i32,
    chunkz: i32,
    heightmap: &mut HeightMap,
    terrain_generator: &Fbm<Perlin>,
) {
    if heightmap.contains_key(&(chunkx, chunkz)) {
        return;
    }
    let posx = chunkx * CHUNK_SIZE_I32;
    let posz = chunkz * CHUNK_SIZE_I32;
    let mut heights = vec![0; (CHUNK_SIZE_I32 * CHUNK_SIZE_I32) as usize];
    for x in posx..(posx + CHUNK_SIZE_I32) {
        for z in posz..(posz + CHUNK_SIZE_I32) {
            let index = ((z - posz) * CHUNK_SIZE_I32 + (x - posx)) as usize;
            let h = get_height(x, z, terrain_generator);
            heights[index] = h;
        }
    }
    heightmap.insert((chunkx, chunkz), heights);
}

impl World {
    //Generates a world
    pub fn gen_default(&mut self) {
        //Generate height map
        let positions = self.chunks.keys().copied().collect();
        let heightmap = generate_heightmap(&positions, &self.world_generator.terrain_generator);

        for chunk in &mut self.chunks.values_mut() {
            let pos = chunk.get_chunk_pos();
            let xz = (pos.x, pos.z);
            if let Some(heights) = heightmap.get(&xz) {
                gen_chunk(chunk, heights, &self.world_generator);
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
        let mut heightmap = HeightMap::new();

        //Generate new chunks
        for (chunkx, chunky, chunkz) in &to_generate {
            let pos = (*chunkx, *chunky, *chunkz);
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

            add_to_heightmap(
                *chunkx,
                *chunkz,
                &mut heightmap,
                &self.world_generator.terrain_generator,
            );
            let mut new_chunk = Chunk::new(*chunkx, *chunky, *chunkz);
            //Should always evaluate to true
            if let Some(heights) = heightmap.get(&(*chunkx, *chunkz)) {
                gen_chunk(&mut new_chunk, heights, &self.world_generator);
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
