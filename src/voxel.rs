pub mod build;
pub mod chunk;
pub mod coordinates;
pub mod world;

pub use self::build::{destroy_block, place_block, raycast, BLOCK_REACH};
pub use self::coordinates::{out_of_bounds, world_to_chunk_position, wrap_coord, ChunkPos};
pub use chunk::Chunk;
pub use world::World;

pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_SIZE_I32: i32 = CHUNK_SIZE as i32;
pub const CHUNK_SIZE_F32: f32 = CHUNK_SIZE as f32;
pub const EMPTY_BLOCK: u8 = 0;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Axis {
    X,
    Y,
    Z,
}

#[derive(Clone, Copy)]
pub struct Block {
    //Block id
    pub id: u8,
}

impl Block {
    //Create a new empty block
    pub fn new() -> Self {
        Self { id: 0 }
    }

    //Create a new block with an id
    pub fn new_id(blockid: u8) -> Self {
        Self { id: blockid }
    }
}
