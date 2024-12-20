use std::collections::{HashSet, HashMap};
use crate::{gfx::ChunkTables, voxel::{CHUNK_SIZE_I32, Block, world_to_chunk_position, EMPTY_BLOCK, wrap_coord}};
use super::World;

const BLOCK_UPDATE_INTERVAL: f32 = 0.2;

type UpdateList = HashMap<(i32, i32, i32), Block>;

fn add_water_tile(
    x: i32,
    y: i32,
    z: i32,
    level: u8,
    to_update: &mut UpdateList,
    world: &World,
) {
    let mut water = Block::new_fluid(12);
    water.geometry = level;

    if water.geometry == 0 || water.geometry > 8 {
        water.id = EMPTY_BLOCK;
    }

    if let Some(tile) = to_update.get(&(x, y, z)) {
        if !tile.is_fluid() {
            return;
        } 

        if tile.id == water.id && tile.geometry < water.geometry && tile.geometry != 7 {
            to_update.insert((x, y, z), water);
        }
    } else {
        if water.geometry == 0 {
            to_update.insert((x, y, z), Block::new()); 
        }
        to_update.insert((x, y, z), water);
    }

    if let Some(tile) = to_update.get(&(x, y, z)) {
        if world.get_block(x, y, z) == *tile {
            to_update.remove(&(x, y, z));
        }
    }
}

//Returns true if updated
fn update_fluid(world: &World, x: i32, y: i32, z: i32, to_update: &mut UpdateList) {
    const ADJ: [(i32, i32, i32); 4] = [ 
        (1, 0, 0),
        (0, 0, 1),
        (-1, 0, 0),
        (0, 0, -1),
    ];

    let block = world.get_block(x, y, z);
    let below = world.get_block(x, y - 1, z);
    let level = block.geometry.min(7);

    //Check for adjacent tiles and see if they allow for this block to exist
    if block.geometry < 7 {
        let mut count = 0;
        let mut maxval = 0;
        let mut next_to_fall = false;
        for (dx, dy, dz) in ADJ {
            let (posx, posy, posz) = (x + dx, y + dy, z + dz);
            let block2 = world.get_block(posx, posy, posz);
            if block2.id != block.id {
                continue;
            }

            if block2.geometry == 8 {
                let underblock = world.get_block(posx, posy - 1, posz);
                if underblock.id == EMPTY_BLOCK || underblock.id == block.id {
                    continue;
                }
                next_to_fall = true;
                continue;
            }

            if block2.geometry > maxval {
                maxval = block2.geometry;
                count = 1;
            } else if block2.geometry == maxval {
                count += 1;
            }
        }

        if maxval > 1 && (below.id == EMPTY_BLOCK || below.id == block.id) { 
            add_water_tile(x, y, z, 1, to_update, world);
        } else if maxval == 7 && count > 1 { 
            add_water_tile(x, y, z, 7, to_update, world);
            return;
        } else if next_to_fall && maxval < 7 {
            add_water_tile(x, y, z, 6, to_update, world);
        } else if maxval == 0 {
            add_water_tile(x, y, z, 0, to_update, world);
        } else if maxval <= level {
            add_water_tile(x, y, z, maxval - 1, to_update, world);
            return;
        } 
    } else if block.geometry == 8 {
        if world.get_block(x, y + 1, z).id != block.id { 
            add_water_tile(x, y, z, 0, to_update, world);
            return;
        }
    }

    //Flow down
    if (below.id == EMPTY_BLOCK || below.id == block.id) && level > 0 {
        if below.geometry != 7 {
            add_water_tile(x, y - 1, z, 8, to_update, world);
        }
        if block.geometry == 8 || block.geometry < 7 {
            return;
        }
    }

    if level <= 1 {
        return;
    }

    //Flow to the sides
    for (dx, dy, dz) in ADJ {
        let (posx, posy, posz) = (x + dx, y + dy, z + dz);
        let adjacent = world.get_block(posx, posy, posz);
        if adjacent.id == EMPTY_BLOCK  || (adjacent.id == block.id && adjacent.geometry < block.geometry - 1) {
            let underblock = world.get_block(posx, posy - 1, posz); 
            let level = if underblock.id == block.id || underblock.id == EMPTY_BLOCK {
                1.min(level)
            } else if level <= 7 {
                level - 1
            } else {
                0
            };
            
            if level == 0 {
                continue;
            }

            add_water_tile(posx, posy, posz, level, to_update, world);
        }
    } 
}

impl World {
    //Returns true if at least one block updated, otherwise false
    fn update_chunk(
        &mut self,
        chunkx: i32,
        chunky: i32,
        chunkz: i32,
        to_update: &mut UpdateList
    ) {
        let startx = chunkx * CHUNK_SIZE_I32;
        let starty = chunky * CHUNK_SIZE_I32;
        let startz = chunkz * CHUNK_SIZE_I32;

        for x in startx..(startx + CHUNK_SIZE_I32) {
            for y in starty..(starty + CHUNK_SIZE_I32) {
                for z in startz..(startz + CHUNK_SIZE_I32) {
                    let block = self.get_block(x, y, z);
                    //Water
                    if block.id == 12 {
                        update_fluid(self, x, y, z, to_update);
                    }
                }
            }
        }    
    }

    pub fn update_blocks(&mut self, dt: f32, chunktables: &mut ChunkTables, chunk_sim_dist: i32) {
        self.block_update_timer += dt;
        if self.block_update_timer <= BLOCK_UPDATE_INTERVAL {
            return;
        }

        self.block_update_timer = 0.0;

        let mut to_update = UpdateList::new();
        let mut update_mesh = HashSet::<(i32, i32, i32)>::new();
        for x in (self.centerx - chunk_sim_dist)..=(self.centerx + chunk_sim_dist) {
            for y in (self.centery - chunk_sim_dist)..=(self.centery + chunk_sim_dist) {
                for z in (self.centerz - chunk_sim_dist)..=(self.centerz + chunk_sim_dist) {
                    self.update_chunk(x, y, z, &mut to_update);
                }
            }
        }

        for ((x, y, z), block) in to_update {
            let (chunkx, chunky, chunkz) = world_to_chunk_position(x, y, z);
            let ix = wrap_coord(x);
            let iy = wrap_coord(y);
            let iz = wrap_coord(z);
            for dx in -1..=1 {
                for dy in -1..=1 {
                    for dz in -1..=1 {
                        if (dx == -1 && ix != 0) || (dx == 1 && ix != CHUNK_SIZE_I32 - 1) {
                            continue;
                        }

                        if (dy == -1 && iy != 0) || (dy == 1 && iy != CHUNK_SIZE_I32 - 1) {
                            continue;
                        }

                        if (dz == -1 && iz != 0) || (dz == 1 && iz != CHUNK_SIZE_I32 - 1) {
                            continue;
                        }

                        update_mesh.insert((chunkx + dx, chunky + dy, chunkz + dz));
                    }
                }
            }
            self.set_block(x, y, z, block);
        }

        for (x, y, z) in update_mesh {
            chunktables.update_table(self, x, y, z); 
        }
    }
}
