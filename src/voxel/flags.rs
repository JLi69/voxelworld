//Array of voxel flags
static mut VOXEL_FLAGS: [u8; 256] = [0; 256];

pub const TRANSPARENT_FLAG: u8 = 1 << 0;
pub const CONNECT_FLAG: u8 = 1 << 1;
pub const CAN_ROTATE_FLAG: u8 = 1 << 2;
pub const NO_HITBOX: u8 = 1 << 3;
pub const FLUID: u8 = 1 << 4;

//TODO: Have a better way of configuring block flags other than hardcoding
//This function should be called at the start of the game
pub fn init_voxel_flags() {
    unsafe {
        //Leaves
        VOXEL_FLAGS[7] |= TRANSPARENT_FLAG;
        //Log
        VOXEL_FLAGS[8] |= CAN_ROTATE_FLAG;
        //Glass
        VOXEL_FLAGS[9] |= TRANSPARENT_FLAG;
        VOXEL_FLAGS[9] |= CONNECT_FLAG;
        //Water
        VOXEL_FLAGS[12] |= TRANSPARENT_FLAG;
        VOXEL_FLAGS[12] |= CONNECT_FLAG;
        VOXEL_FLAGS[12] |= NO_HITBOX;
        VOXEL_FLAGS[12] |= FLUID;
        //Lava
        VOXEL_FLAGS[13] |= TRANSPARENT_FLAG;
        VOXEL_FLAGS[13] |= CONNECT_FLAG;
        VOXEL_FLAGS[13] |= NO_HITBOX;
        VOXEL_FLAGS[13] |= FLUID;
    }
}

//Read only
pub fn get_flag(id: u8) -> u8 {
    unsafe { VOXEL_FLAGS[id as usize] }
}
