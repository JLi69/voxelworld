use std::collections::HashSet;
use cgmath::Vector3;
use super::World;
use crate::{voxel::{Block, Chunk, CHUNK_SIZE_I32, INDESTRUCTIBLE, CHUNK_SIZE_F32}, gfx::ChunkVaoTable};

fn gen_flat_chunk(chunk: &mut Chunk) {
    let chunkpos = chunk.get_chunk_pos();
    let posx = chunkpos.x * CHUNK_SIZE_I32;
    let posy = chunkpos.y * CHUNK_SIZE_I32;
    let posz = chunkpos.z * CHUNK_SIZE_I32;

    if chunkpos.y > 1 || chunkpos.y < -2 {
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

    //Generates new chunks
    //NOTE: if a chunk gets deloaded and reloaded, anything built in that chunk will be
    //deleted, TODO: add a way to save chunks to disk and reload them
    //pos represents the player position
    pub fn gen_more_flat(&mut self, pos: Vector3<f32>, chunktable: &mut ChunkVaoTable) {
        //Check if the player is in the center chunk
        let x = (pos.x / CHUNK_SIZE_F32).floor() as i32;
        let y = (pos.y / CHUNK_SIZE_F32).floor() as i32;
        let z = (pos.z / CHUNK_SIZE_F32).floor() as i32;
        if x == self.centerx && y == self.centery && z == self.centerz {
            return;
        }

        //Find all chunks within range of the player
        let mut in_range = HashSet::<(i32, i32, i32)>::new();
        let mut out_of_range = HashSet::<(i32, i32, i32)>::new();
        for (chunkx, chunky, chunkz) in self.chunks.keys() {
            if (chunkx - x).abs() <= self.range &&
                (chunky - y).abs() <= self.range &&
                (chunkz - z).abs() <= self.range {
                in_range.insert((*chunkx, *chunky, *chunkz));
            } else {
                out_of_range.insert((*chunkx, *chunky, *chunkz));
            }
        }

        //Find chunks to generate
        let mut to_generate = HashSet::<(i32, i32, i32)>::new();
        for chunkx in (x - self.range)..=(x + self.range) {
            for chunky in (y - self.range)..=(y + self.range) {
                for chunkz in (z - self.range)..=(z + self.range) {
                    if in_range.contains(&(chunkx, chunky, chunkz)) {
                        continue;
                    }
                    to_generate.insert((chunkx, chunky, chunkz));
                }
            }
        }

        //Delete old chunks
        for to_delete in &out_of_range {
            self.chunks.remove(to_delete);
        }

        //Generate new chunks
        for (chunkx, chunky, chunkz) in &to_generate {
            let mut new_chunk = Chunk::new(*chunkx, *chunky, *chunkz);
            gen_flat_chunk(&mut new_chunk);
            self.chunks.insert((*chunkx, *chunky, *chunkz), new_chunk);
        }

        //Set the center position
        self.centerx = x;
        self.centery = y;
        self.centerz = z;

        //Delete chunks that are out of range
        chunktable.delete_chunks(self.centerx, self.centery, self.centerz, self.range);

        //Mark chunks that need to have a vao generated
        for (chunkx, chunky, chunkz) in &to_generate {
            chunktable.add_to_update(*chunkx, *chunky, *chunkz);
        }

        //Mark any chunks adjacent to the new border chunks that need to be regenerated
        for (chunkx, chunky, chunkz) in &to_generate {
            let (x, y, z) = (*chunkx, *chunky, *chunkz);

            if self.chunks.contains_key(&(x + 1, y, z)) {
                chunktable.add_to_update(x + 1, y, z);
            }

            if self.chunks.contains_key(&(x - 1, y, z)) {
                chunktable.add_to_update(x - 1, y, z);
            }

            if self.chunks.contains_key(&(x, y + 1, z)) {
                chunktable.add_to_update(x, y + 1, z);
            }

            if self.chunks.contains_key(&(x, y - 1, z)) {
                chunktable.add_to_update(x, y - 1, z);
            }

            if self.chunks.contains_key(&(x, y, z + 1)) {
                chunktable.add_to_update(x, y, z + 1);
            }

            if self.chunks.contains_key(&(x, y, z - 1)) {
                chunktable.add_to_update(x, y, z - 1);
            }
        }
    }
}
