/*
 * Default world generation
 * */

use super::{
    gen_more::{find_in_range, get_chunks_to_generate, update_chunk_vao_table},
    World,
};
use crate::voxel::CHUNK_SIZE_I32;
use crate::{
    gfx::ChunkVaoTable,
    voxel::{Block, Chunk, CHUNK_SIZE_F32, INDESTRUCTIBLE},
};
use cgmath::Vector3;
use noise::{Fbm, NoiseFn, Perlin};
use std::collections::HashMap;

fn gen_chunk(chunk: &mut Chunk, heights: &[i32]) {
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

            for y in posy..(posy + CHUNK_SIZE_I32).min(height + 1) {
                if y == height {
                    //Grass
                    chunk.set_block(x, y, z, Block::new_id(1));
                } else if y > height - 4 && y < height {
                    //Dirt
                    chunk.set_block(x, y, z, Block::new_id(4));
                } else if y < height && y > -64 {
                    //Stone
                    chunk.set_block(x, y, z, Block::new_id(2));
                } else if y == -64 {
                    //Bottom of the world
                    chunk.set_block(x, y, z, Block::new_id(INDESTRUCTIBLE));
                }
            }
        }
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
                let point = [x as f64 / 192.0, z as f64 / 192.0];
                let noise_height = terrain_generator.get(point);
                let h = (noise_height * 47.0) as i32 + 16;
                heights[index] = h;
            }
        }
        heightmap.insert((*chunkx, *chunkz), heights);
    }

    heightmap
}

impl World {
    //Generates a world
    pub fn gen_default(&mut self) {
        //Generate height map
        let positions = self.chunks.keys().copied().collect();
        let heightmap = generate_heightmap(&positions, &self.terrain_generator);

        for chunk in &mut self.chunks.values_mut() {
            let pos = chunk.get_chunk_pos();
            let xz = (pos.x, pos.z);
            if let Some(heights) = heightmap.get(&xz) {
                gen_chunk(chunk, heights);
            }
        }
    }

    //Generates more chunks
    pub fn gen_more_default(&mut self, pos: Vector3<f32>, chunktable: &mut ChunkVaoTable) {
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
        let positions = to_generate.iter().copied().collect();
        let heightmap = generate_heightmap(&positions, &self.terrain_generator);

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

            let mut new_chunk = Chunk::new(*chunkx, *chunky, *chunkz);
            //Should always evaluate to true
            if let Some(heights) = heightmap.get(&(*chunkx, *chunkz)) {
                gen_chunk(&mut new_chunk, heights);
            }
            self.chunks.insert(pos, new_chunk);
        }

        //Set the center position
        self.centerx = x;
        self.centery = y;
        self.centerz = z;

        update_chunk_vao_table(
            chunktable,
            self.centerx,
            self.centery,
            self.centerz,
            self.range,
            &self.chunks,
            &to_generate,
        );
    }
}
