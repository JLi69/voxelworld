pub mod build;
pub mod chunk;
pub mod coordinates;
pub mod flags;
pub mod world;

pub use self::build::{destroy_block, place_block};
pub use self::coordinates::{out_of_bounds, world_to_chunk_position, wrap_coord, ChunkPos};
use self::flags::{get_flag, CONNECT_FLAG, TRANSPARENT_FLAG};
pub use chunk::Chunk;
pub use world::World;

pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_SIZE_I32: i32 = CHUNK_SIZE as i32;
pub const CHUNK_SIZE_F32: f32 = CHUNK_SIZE as f32;
pub const EMPTY_BLOCK: u8 = 0;
pub const INDESTRUCTIBLE: u8 = 3;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Axis {
    X,
    Y,
    Z,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Block {
    //Block id
    pub id: u8,
    //Orientation of the block
    //0 = up (normal)
    //1 = right
    //2 = front
    //3 = down
    //4 = left
    //5 = back
    pub orientation: u8,
}

impl Block {
    //Create a new empty block
    pub fn new() -> Self {
        Self {
            id: 0,
            orientation: 0,
        }
    }

    //Create a new block with an id
    pub fn new_id(blockid: u8) -> Self {
        Self {
            id: blockid,
            orientation: 0,
        }
    }
    
    //Create a new block with id and orientation
    pub fn new_id_orientation(blockid: u8, block_orientation: u8) -> Self {
        Self {
            id: blockid,
            orientation: block_orientation,
        }
    }

    //Returns if the block is transparent
    pub fn transparent(&self) -> bool {
        get_flag(self.id) & TRANSPARENT_FLAG != 0
    }

    //Returns if the block can "connect" to the block next to it
    //(will not display any face if the block is transparent and the same
    //block is next to it)
    pub fn can_connect(&self) -> bool {
        get_flag(self.id) & CONNECT_FLAG != 0
    }
}
