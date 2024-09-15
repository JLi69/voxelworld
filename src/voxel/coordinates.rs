use super::CHUNK_SIZE_I32;

/*
 * Operations to voxel coordinates
 * */

//struct for storing chunk position
#[derive(Clone, Copy)]
pub struct ChunkPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl ChunkPos {
    //Returns the (0, 0, 0)
    pub fn origin() -> Self {
        Self { x: 0, y: 0, z: 0 }
    }

    //Returns (posx, posy, posz)
    pub fn new(posx: i32, posy: i32, posz: i32) -> Self {
        Self {
            x: posx,
            y: posy,
            z: posz,
        }
    }
}

//Converts a single coordinate to a chunk coordinate (divides by CHUNK_SIZE)
pub fn world_coord_to_chunk_coord(x: i32) -> i32 {
    if x < 0 && x % CHUNK_SIZE_I32 != 0 {
        //Handle negative coordinates
        x / CHUNK_SIZE_I32 - 1
    } else {
        //For positive coordinates and any negative coordinates that are a
        //multiple of CHUNK_SIZE
        x / CHUNK_SIZE_I32
    }
}

//Converts a world position (x, y, z) to its chunk position (the coordinates of
//the chunk that the voxel is in) The chunk position should essentially be the
//coordinates of the voxel divided by CHUNK_SIZE
pub fn world_to_chunk_position(x: i32, y: i32, z: i32) -> (i32, i32, i32) {
    let chunkx = world_coord_to_chunk_coord(x);
    let chunky = world_coord_to_chunk_coord(y);
    let chunkz = world_coord_to_chunk_coord(z);
    (chunkx, chunky, chunkz)
}

//Returns if (x, y, z) + (offsetx, offsety, offsetz) is out of bounds for a chunk
//assumes that 0 <= (x, y, z) < CHUNK_SIZE.
pub fn out_of_bounds(x: i32, y: i32, z: i32, offsetx: i32, offsety: i32, offsetz: i32) -> bool {
    x + offsetx < 0
        || y + offsety < 0
        || z + offsetz < 0
        || x + offsetx >= CHUNK_SIZE_I32
        || y + offsety >= CHUNK_SIZE_I32
        || z + offsetz >= CHUNK_SIZE_I32
}

//mods the coordinate by CHUNK_SIZE and also converts it to a positive coordinate
pub fn wrap_coord(x: i32) -> i32 {
    let mut value = x % CHUNK_SIZE_I32;

    if value < 0 {
        value += CHUNK_SIZE_I32;
    }

    value % CHUNK_SIZE_I32
}
