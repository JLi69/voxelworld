use std::collections::HashSet;

use super::World;
use crate::voxel::{Block, Chunk, CHUNK_SIZE_I32, INDESTRUCTIBLE};
use crossbeam::{queue::ArrayQueue, thread};

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

    pub fn generate_column_flat(&mut self, x: i32, z: i32, yvals: &HashSet<i32>) {
        //Generate new chunks
        let start = std::time::Instant::now();
        let generated = ArrayQueue::new(yvals.len());
        let mut generated_count = 0;
        thread::scope(|s| {
            for y in yvals {
                if !self.in_range(x, *y, z) {
                    continue;
                }

                if self.chunks.contains_key(&(x, *y, z)) {
                    continue;
                }

                generated_count += 1;

                s.spawn(|_| {
                    let mut new_chunk = Chunk::new(x, *y, z);
                    gen_flat_chunk(&mut new_chunk);
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
        let time = start.elapsed().as_millis();
        if time > 15 {
            //Only report time taken if it exceeds 15 ms
            eprintln!("Took {time} ms to generate {generated_count} new chunks");
        }
    }
}
