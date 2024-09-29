//Array of voxel flags
static mut VOXEL_FLAGS: [u8; 256] = [0; 256];

pub const TRANSPARENT_FLAG: u8 = 1 << 0;
pub const CONNECT_FLAG: u8 = 1 << 1;

//TODO: Have a better way of configuring block flags other than hardcoding
//This function should be called at the start of the game
pub fn init_voxel_flags() {
    unsafe {
        //Leaves
        VOXEL_FLAGS[7] |= TRANSPARENT_FLAG;
        //Glass
        VOXEL_FLAGS[9] |= TRANSPARENT_FLAG;
        VOXEL_FLAGS[9] |= CONNECT_FLAG;
    }
}

//Read only
pub fn get_flag(id: u8) -> u8 {
    unsafe {
        VOXEL_FLAGS[id as usize]
    }
}
