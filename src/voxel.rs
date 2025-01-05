pub mod build;
pub mod chunk;
pub mod coordinates;
pub mod flags;
pub mod world;

pub use self::build::{destroy_block, place_block};
pub use self::coordinates::{out_of_bounds, world_to_chunk_position, wrap_coord, ChunkPos};
use self::flags::{get_flag, CAN_ROTATE_FLAG, CONNECT_FLAG, FLUID, NO_HITBOX, TRANSPARENT_FLAG, ROTATE_Y_ONLY};
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
    //Represents geometry of the block
    //For liquid: 7 = still, 8 = flowing under
    //Orientation of the block
    //0 = up (normal)
    //1 = right
    //2 = front
    //3 = down
    //4 = left
    //5 = back
    pub geometry: u8,
}

impl Block {
    //Create a new empty block
    pub fn new() -> Self {
        Self { id: 0, geometry: 0 }
    }

    //Create a new block with an id
    pub fn new_id(blockid: u8) -> Self {
        Self {
            id: blockid,
            geometry: 0,
        }
    }

    //Create a new fluid block
    pub fn new_fluid(blockid: u8) -> Self {
        Self {
            id: blockid,
            geometry: 7,
        }
    }

    //Create a new block with id and orientation
    pub fn new_id_orientation(blockid: u8, block_orientation: u8) -> Self {
        Self {
            id: blockid,
            geometry: block_orientation,
        }
    }

    //Returns the orientation of the block
    pub fn orientation(&self) -> u8 {
        self.geometry & 7
    }

    pub fn set_orientation(&mut self, orientation: u8) {
        self.geometry &= 0xff << 3;
        self.geometry |= orientation
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

    //Returns if the block can be rotated when placed
    pub fn can_rotate(&self) -> bool {
        get_flag(self.id) & CAN_ROTATE_FLAG != 0
    }

    //Returns if the voxel has no hitbox
    pub fn no_hitbox(&self) -> bool {
        get_flag(self.id) & NO_HITBOX != 0
    }

    //Returns if the voxel is a fluid
    pub fn is_fluid(&self) -> bool {
        get_flag(self.id) & FLUID != 0
    }

    //Returns if the voxel can rotate only about the y axis
    pub fn rotate_y_only(&self) -> bool {
        get_flag(self.id) & ROTATE_Y_ONLY != 0
    }
}
