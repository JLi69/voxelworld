use std::collections::HashSet;

use super::{
    gen_more::{find_in_range, get_chunks_to_generate, update_chunk_tables},
    World,
};
use crate::{
    gfx::ChunkTables,
    voxel::{
        region::{chunkpos_to_regionpos, Region},
        Block, Chunk, CHUNK_SIZE_F32, CHUNK_SIZE_I32, INDESTRUCTIBLE,
    },
};
use cgmath::Vector3;

fn gen_flat_chunk(chunk: &mut Chunk) {
    let chunkpos = chunk.get_chunk_pos();
    let posx = chunkpos.x * CHUNK_SIZE_I32;
    let posy = chunkpos.y * CHUNK_SIZE_I32;
    let posz = chunkpos.z * CHUNK_SIZE_I32;

    if chunkpos.y > 1 || chunkpos.y < -4 {
        return;
    }

    for x in posx..(posx + CHUNK_SIZE_I32) {
        for y in posy..(posy + CHUNK_SIZE_I32) {
            for z in posz..(posz + CHUNK_SIZE_I32) {
                if y == -1 {
                    //Grass
                    chunk.set_block(x, y, z, Block::new_id(1));
                } else if (-4..-1).contains(&y) {
                    chunk.set_block(x, y, z, Block::new_id(4));
                } else if (-62..-4).contains(&y) {
                    //Stone
                    chunk.set_block(x, y, z, Block::new_id(2));
                } else if y == -63 {
                    //Bottom of the world
                    chunk.set_block(x, y, z, Block::new_id(INDESTRUCTIBLE));
                }
            }
        }
    }
}

impl World {
    //Generates a flat world
    pub fn gen_flat(&mut self) {
        for chunk in &mut self.chunks.values_mut() {
            gen_flat_chunk(chunk);
        }
    }

    //Generates new flat world chunks
    pub fn gen_more_flat(&mut self, pos: Vector3<f32>, chunktables: &mut ChunkTables) {
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
        for (chunkx, chunky, chunkz) in &to_generate {
            let pos = (*chunkx, *chunky, *chunkz);
            if self.chunk_cache.contains_key(&pos) {
                let new_chunk = self.chunk_cache.get(&pos);
                if let Some(new_chunk) = new_chunk {
                    self.chunks.insert(pos, new_chunk.clone());
                    self.chunk_cache.remove(&pos);
                }
            }
        }

        //Load chunks from filesystem
        let mut loaded = HashSet::new();
        for (chunkx, chunky, chunkz) in &to_generate {
            if self.chunks.contains_key(&(*chunkx, *chunky, *chunkz)) {
                continue;
            }

            let (rx, ry, rz) = chunkpos_to_regionpos(*chunkx, *chunky, *chunkz);
            if !loaded.contains(&(rx, ry, rz)) {
                if let Some(region) = Region::load_region(&self.path, rx, ry, rz) {
                    self.add_region(region);
                    loaded.insert((rx, ry, rz));
                    continue;
                }
            }
        }

        //Generate new chunks
        for (chunkx, chunky, chunkz) in &to_generate {
            let pos = (*chunkx, *chunky, *chunkz);
            if self.chunks.contains_key(&pos) {
                continue;
            }
            let mut new_chunk = Chunk::new(*chunkx, *chunky, *chunkz);
            gen_flat_chunk(&mut new_chunk);
            self.chunks.insert(pos, new_chunk);
        }

        self.init_light_new_chunks(&to_generate);

        update_chunk_tables(
            chunktables,
            self.centerx,
            self.centery,
            self.centerz,
            self.range,
            &self.chunks,
            &to_generate,
        );
    }

    //Generates missing flat world chunks on load
    pub fn gen_flat_on_load(&mut self) {
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

        //Generate new chunks
        for (chunkx, chunky, chunkz) in &to_generate {
            let pos = (*chunkx, *chunky, *chunkz);
            if self.chunks.contains_key(&pos) {
                continue;
            }
            let mut new_chunk = Chunk::new(*chunkx, *chunky, *chunkz);
            gen_flat_chunk(&mut new_chunk);
            self.chunks.insert(pos, new_chunk);
        }
    }
}
