//Array of voxel flags
static mut VOXEL_FLAGS: [u16; 256] = [0; 256];

pub const TRANSPARENT_FLAG: u16 = 1 << 0;
pub const CONNECT_FLAG: u16 = 1 << 1;
pub const CAN_ROTATE_FLAG: u16 = 1 << 2;
pub const NO_HITBOX: u16 = 1 << 3;
pub const FLUID: u16 = 1 << 4;
pub const ROTATE_Y_ONLY: u16 = 1 << 5;
pub const FLAT_ITEM: u16 = 1 << 6;
pub const FLUID_DESTRUCTIBLE: u16 = 1 << 7;
pub const NON_VOXEL: u16 = 1 << 8;

unsafe fn set_plant_flags(voxel_id: usize) {
    VOXEL_FLAGS[voxel_id] |= TRANSPARENT_FLAG;
    VOXEL_FLAGS[voxel_id] |= NO_HITBOX;
    VOXEL_FLAGS[voxel_id] |= FLAT_ITEM;
    VOXEL_FLAGS[voxel_id] |= FLUID_DESTRUCTIBLE;
}

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
        //Chest
        VOXEL_FLAGS[37] |= CAN_ROTATE_FLAG;
        VOXEL_FLAGS[37] |= ROTATE_Y_ONLY;
        //Furnace
        VOXEL_FLAGS[40] |= CAN_ROTATE_FLAG;
        VOXEL_FLAGS[40] |= ROTATE_Y_ONLY;
        //Sapling
        set_plant_flags(47);
        //Mushroom
        //Yeah yeah mushrooms are fungi not plants,
        //I know that, but they have the same voxel flags
        //so I don't give a flying duck
        set_plant_flags(48);
        //Tall grass
        set_plant_flags(49);
        //Wheat
        set_plant_flags(50);
        set_plant_flags(51);
        set_plant_flags(52);
        set_plant_flags(53);
        //Red flower
        set_plant_flags(54);
        //Yellow flower
        set_plant_flags(55);
        //Blue flower
        set_plant_flags(56);
        //Sugar cane
        set_plant_flags(69);
        //Torches
        //They obviously aren't plants but share a lot of block properties with them
        set_plant_flags(71);
        VOXEL_FLAGS[71] |= NON_VOXEL;
        set_plant_flags(72);
        VOXEL_FLAGS[72] |= NON_VOXEL;
        set_plant_flags(73);
        VOXEL_FLAGS[73] |= NON_VOXEL;
        set_plant_flags(74);
        VOXEL_FLAGS[74] |= NON_VOXEL;
    }
}

//Read only
pub fn get_flag(id: u8) -> u16 {
    unsafe { VOXEL_FLAGS[id as usize] }
}
