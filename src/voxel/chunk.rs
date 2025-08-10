mod light;
pub mod save;

use super::{
    light::{Light, LU},
    tile_data::TileData,
    Block, ChunkPos, CHUNK_SIZE, CHUNK_SIZE_I32, EMPTY_BLOCK,
};
use std::collections::HashMap;

fn out_of_bounds(index_x: i32, index_y: i32, index_z: i32) -> bool {
    index_x < 0
        || index_y < 0
        || index_z < 0
        || index_x >= CHUNK_SIZE_I32
        || index_y >= CHUNK_SIZE_I32
        || index_z >= CHUNK_SIZE_I32
}

fn pos_to_index(x: i32, y: i32, z: i32, pos: ChunkPos) -> (i32, i32, i32) {
    let index_x = x - CHUNK_SIZE_I32 * pos.x;
    let index_y = y - CHUNK_SIZE_I32 * pos.y;
    let index_z = z - CHUNK_SIZE_I32 * pos.z;
    (index_x, index_y, index_z)
}

#[derive(Clone, Debug)]
pub struct Chunk {
    //Chunks are CHUNK_SIZE x CHUNK_SIZE x CHUNK_SIZE cubes
    //For convention, assume that x is left to right, z is
    //forward and backwards, and y is up and down
    blocks: Vec<Block>,
    //Stores the light data for each block
    light: Vec<Light>,
    //Integer position of the chunk, this is corner that has the lowest value
    //x, y, and z coordinates (the world position of the chunk is these values
    //multiplied by CHUNK_SIZE
    ix: i32,
    iy: i32,
    iz: i32,
    data: HashMap<(i32, i32, i32), TileData>,
}

impl Chunk {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self {
            blocks: vec![],
            light: vec![],
            ix: x,
            iy: y,
            iz: z,
            data: HashMap::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        for b in &self.blocks {
            if b.id != EMPTY_BLOCK {
                return false;
            }
        }

        true
    }

    //Frees memory if the chunk is completely empty
    pub fn handle_empty(&mut self) {
        if self.blocks.is_empty() {
            return;
        }

        if self.is_empty() {
            self.blocks.clear();
        }
    }

    //Relative the position of the chunk (0 <= x, y, z < CHUNK_SIZE)
    pub fn get_block_relative(&self, x: usize, y: usize, z: usize) -> Block {
        if self.blocks.is_empty() {
            return Block::new();
        }

        //Out of bounds, return 0
        if x >= CHUNK_SIZE || y >= CHUNK_SIZE || z >= CHUNK_SIZE {
            return Block::new();
        }

        let index_y = CHUNK_SIZE * CHUNK_SIZE * y;
        let index_z = CHUNK_SIZE * z;
        let index_x = x;
        self.blocks[index_x + index_y + index_z]
    }

    pub fn get_light_relative(&self, x: usize, y: usize, z: usize) -> Light {
        if self.light.is_empty() {
            return Light::black();
        }

        //Out of bounds, return black
        if x >= CHUNK_SIZE || y >= CHUNK_SIZE || z >= CHUNK_SIZE {
            return Light::black();
        }

        let index_y = CHUNK_SIZE * CHUNK_SIZE * y;
        let index_z = CHUNK_SIZE * z;
        let index_x = x;
        self.light[index_x + index_y + index_z]
    }

    pub fn update_light_relative(&mut self, x: usize, y: usize, z: usize, update: LU) {
        if self.light.is_empty() {
            self.light = vec![Light::black(); CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE];
        }

        //Out of bounds, return 0
        if x >= CHUNK_SIZE || y >= CHUNK_SIZE || z >= CHUNK_SIZE {
            return;
        }

        let index_y = CHUNK_SIZE * CHUNK_SIZE * y;
        let index_z = CHUNK_SIZE * z;
        let index_x = x;
        self.light[index_x + index_y + index_z].update(update)
    }

    pub fn set_block_relative(&mut self, x: usize, y: usize, z: usize, block: Block) {
        if self.blocks.is_empty() && block.id != EMPTY_BLOCK {
            self.blocks = vec![Block::new(); CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE];
        } else if !self.blocks.is_empty() && block.id == EMPTY_BLOCK {
            self.handle_empty();
        }

        if self.blocks.is_empty() && block.id == EMPTY_BLOCK {
            return;
        }

        //Out of bounds
        if x >= CHUNK_SIZE || y >= CHUNK_SIZE || z >= CHUNK_SIZE {
            return;
        }

        let index_y = CHUNK_SIZE * CHUNK_SIZE * y;
        let index_z = CHUNK_SIZE * z;
        let index_x = x;
        self.blocks[index_x + index_y + index_z] = block;
    }

    //x, y, and z are absolute world positions
    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Block {
        let (index_x, index_y, index_z) = pos_to_index(x, y, z, self.get_chunk_pos());
        //Out of bounds
        if out_of_bounds(index_x, index_y, index_z) {
            return Block::new();
        }

        self.get_block_relative(index_x as usize, index_y as usize, index_z as usize)
    }

    //x, y, and z are absolute world positions
    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block: Block) {
        let (index_x, index_y, index_z) = pos_to_index(x, y, z, self.get_chunk_pos());
        //Out of bounds
        if out_of_bounds(index_x, index_y, index_z) {
            return;
        }

        self.set_block_relative(index_x as usize, index_y as usize, index_z as usize, block);
    }

    //x, y, z are absolute world positions
    pub fn get_light(&self, x: i32, y: i32, z: i32) -> Light {
        let (index_x, index_y, index_z) = pos_to_index(x, y, z, self.get_chunk_pos());
        //Out of bounds
        if out_of_bounds(index_x, index_y, index_z) {
            return Light::black();
        }

        self.get_light_relative(index_x as usize, index_y as usize, index_z as usize)
    }

    //x, y, z are absolute world positions
    pub fn update_light(&mut self, x: i32, y: i32, z: i32, update: LU) {
        let (index_x, index_y, index_z) = pos_to_index(x, y, z, self.get_chunk_pos());
        //Out of bounds
        if out_of_bounds(index_x, index_y, index_z) {
            return;
        }

        self.update_light_relative(index_x as usize, index_y as usize, index_z as usize, update);
    }

    pub fn get_chunk_pos(&self) -> ChunkPos {
        ChunkPos::new(self.ix, self.iy, self.iz)
    }
}
