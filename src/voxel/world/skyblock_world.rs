use std::collections::HashSet;

use super::World;
use crate::{
    game::inventory::Item,
    voxel::{tile_data::TileData, Block, Chunk, CHUNK_SIZE_I32, EMPTY_BLOCK, INDESTRUCTIBLE},
};
use crossbeam::{queue::ArrayQueue, thread};

/*
 * NOTE: This code is effectively just copied and pasted from `flat_world.rs`
 * so there is a lot of repetition. I can probably improve the world generation
 * code to be better but I don't feel like doing that right now and it works fine
 * so copy-paste it is.
 */

fn place_leaves(chunk: &mut Chunk, x: usize, y: usize, z: usize) {
    let replace = chunk.get_block_relative(x, y, z);
    if replace.id != EMPTY_BLOCK && replace.shape() == 0 {
        return;
    }
    chunk.set_block_relative(x, y, z, Block::new_id(7));
}

fn generate_leaves(chunk: &mut Chunk, starty: usize, x: usize, y: usize, z: usize, height: usize) {
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

//This should only be generated in the chunk (0, 0, 0)
fn gen_spawn_island(chunk: &mut Chunk) {
    //Generate the island
    for x in 3..=8 {
        for z in 3..=8 {
            for y in 0..=2 {
                if y == 2 {
                    //Grass layer
                    chunk.set_block_relative(x, y, z, Block::new_id(1));
                } else {
                    //Dirt
                    chunk.set_block_relative(x, y, z, Block::new_id(4));
                }
            }
        }
    }

    for x in 3..=5 {
        for z in 3..=5 {
            for y in 0..=2 {
                chunk.set_block_relative(x, y, z, Block::new());
            }
        }
    }

    chunk.set_block_relative(7, 0, 7, Block::new_id(INDESTRUCTIBLE));

    //Generate the tree on the island
    let tree_x = 8;
    let tree_y = 3;
    let tree_z = 3;
    let tree_height = 6;
    for y in tree_y..(tree_y + tree_height) {
        //Generate trunk
        chunk.set_block_relative(tree_x, y, tree_z, Block::new_id(8));

        //Generate leaves
        generate_leaves(chunk, tree_y, tree_x, y, tree_z, tree_height);
    }
    generate_leaves(
        chunk,
        tree_y,
        tree_x,
        tree_y + tree_height,
        tree_z,
        tree_height,
    );

    //Generate the chest on the island
    let mut chest = Block::new_id(37);
    chest.set_orientation(4);
    chunk.set_block_relative(3, 3, 7, chest);
    let mut chest_data = TileData::new_chest();
    //Lava bucket
    chest_data.inventory.set_item(3, 1, Item::Bucket(13));
    //Ice
    chest_data
        .inventory
        .set_item(5, 1, Item::Block(Block::new_id(85), 1));
    chunk.set_tile_data(3, 3, 7, Some(chest_data));
}

fn gen_sand_island(chunk: &mut Chunk) {
    //Generate island
    for x in 3..=5 {
        for z in 3..=5 {
            for y in 0..=2 {
                chunk.set_block_relative(x, y, z, Block::new_id(11));
            }
        }
    }

    //Cactus
    chunk.set_block_relative(5, 3, 3, Block::new_id(88));
    //Sugar cane
    chunk.set_block_relative(4, 3, 5, Block::new_id(69));

    //Generate chest
    let mut chest = Block::new_id(37);
    chest.set_orientation(1);
    chunk.set_block_relative(4, 3, 4, chest);

    let mut chest_data = TileData::new_chest();
    //Ice
    chest_data
        .inventory
        .set_item(3, 1, Item::Block(Block::new_id(85), 1));
    //Wheat seeds
    chest_data
        .inventory
        .set_item(5, 1, Item::Block(Block::new_id(77), 1));

    let chunkpos = chunk.get_chunk_pos();
    let chunkx = chunkpos.x * CHUNK_SIZE_I32;
    let chunky = chunkpos.y * CHUNK_SIZE_I32;
    let chunkz = chunkpos.z * CHUNK_SIZE_I32;
    chunk.set_tile_data(chunkx + 4, chunky + 3, chunkz + 4, Some(chest_data));
}

fn gen_flower_island(chunk: &mut Chunk) {
    //Generate island
    for x in 6..=8 {
        for z in 6..=8 {
            for y in 0..=2 {
                if y == 2 {
                    chunk.set_block_relative(x, y, z, Block::new_id(1));
                } else {
                    chunk.set_block_relative(x, y, z, Block::new_id(4));
                }
            }
        }
    }

    //Red flower
    chunk.set_block_relative(6, 3, 6, Block::new_id(54));
    //Yellow flower
    chunk.set_block_relative(6, 3, 8, Block::new_id(55));
    //Blue flower
    chunk.set_block_relative(8, 3, 6, Block::new_id(56));
    //Cotton
    chunk.set_block_relative(8, 3, 8, Block::new_id(102));
}

fn gen_skyblock_chunk(chunk: &mut Chunk) {
    let chunkpos = chunk.get_chunk_pos();

    if chunkpos.x == 0 && chunkpos.y == 0 && chunkpos.z == 0 {
        gen_spawn_island(chunk);
    } else if chunkpos.x == 5 && chunkpos.y == 0 && chunkpos.z == 0 {
        gen_sand_island(chunk);
    } else if chunkpos.x == 0 && chunkpos.y == 0 && chunkpos.z == 4 {
        gen_flower_island(chunk);
    }
}

impl World {
    //Generates a skyblock world
    pub fn gen_skyblock(&mut self) {
        for chunk in &mut self.chunks.values_mut() {
            gen_skyblock_chunk(chunk);
        }
    }

    //Generates missing flat world chunks on load
    pub fn gen_skyblock_on_load(&mut self) {
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
            gen_skyblock_chunk(&mut new_chunk);
            self.chunks.insert(pos, new_chunk);
        }
    }

    pub fn generate_column_skyblock(&mut self, x: i32, z: i32, yvals: &HashSet<i32>) {
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
                    gen_skyblock_chunk(&mut new_chunk);
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

    pub fn add_skyblock_chunk(&mut self, chunkx: i32, chunky: i32, chunkz: i32) {
        let mut chunk = Chunk::new(chunkx, chunky, chunkz);
        gen_skyblock_chunk(&mut chunk);
        self.chunks.insert((chunkx, chunky, chunkz), chunk);
    }
}
