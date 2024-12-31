use super::World;
use crate::{
    gfx::ChunkTables,
    voxel::{world_to_chunk_position, wrap_coord, Block, CHUNK_SIZE_I32, EMPTY_BLOCK},
};
use std::collections::{HashMap, HashSet};

const BLOCK_UPDATE_INTERVAL: f32 = 0.2;
const ADJ: [(i32, i32, i32); 4] = [(1, 0, 0), (0, 0, 1), (-1, 0, 0), (0, 0, -1)];

type UpdateList = HashMap<(i32, i32, i32), Block>;

fn add_water_tile(x: i32, y: i32, z: i32, level: u8, id: u8, to_update: &mut UpdateList) {
    let mut water = Block::new_fluid(id);
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
}

//Returns true if updated
fn update_fluid(world: &World, x: i32, y: i32, z: i32, to_update: &mut UpdateList, decrease: u8) {
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

            let underblock = world.get_block(posx, posy - 1, posz);
            if (underblock.id == EMPTY_BLOCK || underblock.id == block.id) && block2.geometry != 7 {
                continue;
            }

            if block2.geometry == 8 {
                next_to_fall = true;
                continue;
            }

            if block2.geometry > maxval {
                maxval = block2.geometry;
                count = 0;
            }

            if block2.geometry == maxval {
                count += 1;
            }
        }

        if maxval > 1 && (below.id == EMPTY_BLOCK || below.id == block.id) {
            add_water_tile(x, y, z, 1, block.id, to_update);
            if below.geometry != 7 {
                add_water_tile(x, y - 1, z, 8, block.id, to_update);
            }
            return;
        } else if maxval == 7 && count > 1 && decrease == 1 {
            add_water_tile(x, y, z, 7, block.id, to_update);
            return;
        } else if next_to_fall && maxval < 7 {
            add_water_tile(x, y, z, 7 - decrease, block.id, to_update);
        } else if maxval <= 1 {
            add_water_tile(x, y, z, 0, block.id, to_update);
            return;
        } else if maxval <= level {
            add_water_tile(x, y, z, maxval - decrease, block.id, to_update);
            return;
        }
    } else if block.geometry == 8 && world.get_block(x, y + 1, z).id != block.id {
        add_water_tile(x, y, z, 7 - decrease, block.id, to_update);
        return;
    }

    //Flow down
    if (below.id == EMPTY_BLOCK || below.id == block.id) && level > 0 {
        if below.geometry != 7 {
            add_water_tile(x, y - 1, z, 8, block.id, to_update);
        }
        if block.geometry != 7 {
            return;
        }
    }

    if level <= decrease {
        return;
    }

    //Flow to the sides
    for (dx, dy, dz) in ADJ {
        let (posx, posy, posz) = (x + dx, y + dy, z + dz);
        let adjacent = world.get_block(posx, posy, posz);
        if adjacent.id == EMPTY_BLOCK
            || (adjacent.id == block.id && adjacent.geometry < block.geometry - 1)
        {
            let underblock = world.get_block(posx, posy - 1, posz);
            let blocklevel = if underblock.id == block.id || underblock.id == EMPTY_BLOCK {
                1.min(level)
            } else if level <= 7 {
                level - decrease
            } else {
                0
            };

            if level == 0 {
                continue;
            }

            add_water_tile(posx, posy, posz, blocklevel, block.id, to_update);
        }
    }
}

//Returns true if updated
fn water_to_stone(world: &World, x: i32, y: i32, z: i32, to_update: &mut UpdateList) -> bool {
    //Is the block above lava?
    if world.get_block(x, y + 1, z).id == 13 {
        //then turn to stone
        to_update.insert((x, y, z), Block::new_id(2));
        return true;
    }
    false
}

//Returns true if updated
fn freeze_lava(world: &World, x: i32, y: i32, z: i32, to_update: &mut UpdateList) -> bool {
    let block = world.get_block(x, y, z);
    let new_block = if block.geometry == 7 {
        //Obsidian
        Block::new_id(14)
    } else {
        //Stone
        Block::new_id(2)
    };

    //If water is above or to the side, then freeze the lava
    if world.get_block(x, y + 1, z).id == 12 {
        to_update.insert((x, y, z), new_block);
        return true;
    }

    for (dx, dy, dz) in ADJ {
        if world.get_block(x + dx, y + dy, z + dz).id == 12 {
            to_update.insert((x, y, z), new_block);
            return true;
        }
    }

    false
}

fn update_water(world: &World, x: i32, y: i32, z: i32, to_update: &mut UpdateList) {
    let updated = water_to_stone(world, x, y, z, to_update);
    if updated {
        return;
    }
    update_fluid(world, x, y, z, to_update, 1);
}

fn update_lava(world: &World, x: i32, y: i32, z: i32, to_update: &mut UpdateList) {
    let updated = freeze_lava(world, x, y, z, to_update);
    if updated {
        return;
    }
    update_fluid(world, x, y, z, to_update, 2);
}

impl World {
    //Returns true if at least one block updated, otherwise false
    fn update_chunk(&mut self, chunkx: i32, chunky: i32, chunkz: i32, to_update: &mut UpdateList) {
        let startx = chunkx * CHUNK_SIZE_I32;
        let starty = chunky * CHUNK_SIZE_I32;
        let startz = chunkz * CHUNK_SIZE_I32;

        for x in startx..(startx + CHUNK_SIZE_I32) {
            for y in starty..(starty + CHUNK_SIZE_I32) {
                for z in startz..(startz + CHUNK_SIZE_I32) {
                    let block = self.get_block(x, y, z);
                    if block.id == 12 {
                        //Water
                        update_water(self, x, y, z, to_update);
                    } else if block.id == 13 {
                        //Lava
                        update_lava(self, x, y, z, to_update);
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
                    if !self.updating.contains(&(x, y, z)) {
                        continue;
                    }

                    self.update_chunk(x, y, z, &mut to_update);
                }
            }
        }

        self.updating.clear();

        for ((x, y, z), block) in to_update {
            if self.get_block(x, y, z) == block {
                continue;
            }

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

    //Add all chunks to the updating list
    pub fn update_all_chunks(&mut self) {
        for chunkpos in self.chunks.keys() {
            self.updating.insert(*chunkpos);
        }
    }
}
