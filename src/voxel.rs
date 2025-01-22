pub mod build;
pub mod chunk;
pub mod coordinates;
pub mod flags;
pub mod is_valid;
pub mod world;

pub use self::build::{destroy_block, place_block};
pub use self::coordinates::{out_of_bounds, world_to_chunk_position, wrap_coord, ChunkPos};
use self::flags::{
    get_flag, CAN_ROTATE_FLAG, CONNECT_FLAG, FLAT_ITEM, FLUID, FLUID_DESTRUCTIBLE, NO_HITBOX,
    ROTATE_Y_ONLY, TRANSPARENT_FLAG, NON_VOXEL,
};
use cgmath::Vector3;
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
    //First 3 bits = shape
    //Last 3 bits = orientation
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
        self.geometry |= orientation;
    }

    pub fn shape(&self) -> u8 {
        self.geometry >> 5
    }

    //Block shape
    //0 = full block
    //1 = slab
    //2 = stair (5/8)
    //3 = stair (6/8)
    //4 = stair (7/8)
    pub fn set_shape(&mut self, block_shape: u8) {
        self.geometry &= 0xff >> 5;
        self.geometry |= block_shape << 5;
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

    //Returns if the voxel must be displayed as a flat item
    pub fn is_flat_item(&self) -> bool {
        get_flag(self.id) & FLAT_ITEM != 0
    }

    //Returns if the voxel can be destroyed by fluid
    pub fn fluid_destructibe(&self) -> bool {
        get_flag(self.id) & FLUID_DESTRUCTIBLE != 0
    }

    //Returns if the voxel has a non-voxel geometry and needs to be handled uniquely
    pub fn non_voxel_geometry(&self) -> bool {
        get_flag(self.id) & NON_VOXEL != 0
    }
}

pub fn orientation_to_normal(orientation: u8) -> Vector3<i32> {
    //0 = up (normal)
    //1 = right
    //2 = front
    //3 = down
    //4 = left
    //5 = back
    match orientation {
        0 => Vector3::new(0, 1, 0),
        1 => Vector3::new(1, 0, 0),
        2 => Vector3::new(0, 0, 1),
        3 => Vector3::new(0, -1, 0),
        4 => Vector3::new(-1, 0, 0),
        5 => Vector3::new(0, 0, -1),
        _ => Vector3::new(0, 0, 0),
    }
}
