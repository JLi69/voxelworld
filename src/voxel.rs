use cgmath::{InnerSpace, Vector3};

pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_SIZE_I32: i32 = CHUNK_SIZE as i32;
pub const CHUNK_SIZE_F32: f32 = CHUNK_SIZE as f32;
pub const EMPTY_BLOCK: u8 = 0;

#[derive(Clone, Copy)]
pub struct Block {
    pub id: u8,
}

impl Block {
    pub fn new() -> Self {
        Self { id: 0 }
    }

    pub fn new_id(blockid: u8) -> Self {
        Self { id: blockid }
    }
}

#[derive(Clone, Copy)]
pub struct ChunkPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl ChunkPos {
    pub fn origin() -> Self {
        Self { x: 0, y: 0, z: 0 }
    }
}

pub fn world_coord_to_chunk_coord(x: i32) -> i32 {
    if x < 0 && x % CHUNK_SIZE_I32 != 0 {
        x / CHUNK_SIZE_I32 - 1
    } else {
        x / CHUNK_SIZE_I32
    }
}

pub fn world_to_chunk_position(x: i32, y: i32, z: i32) -> (i32, i32, i32) {
    let chunkx = world_coord_to_chunk_coord(x);
    let chunky = world_coord_to_chunk_coord(y);
    let chunkz = world_coord_to_chunk_coord(z);
    (chunkx, chunky, chunkz)
}

pub struct Chunk {
    //Chunks are CHUNK_SIZE x CHUNK_SIZE x CHUNK_SIZE cubes
    //For convention, assume that x is left to right, z is
    //forward and backwards, and y is up and down
    blocks: Vec<Block>,
    //Integer position of the chunk, this is corner that has the lowest value
    //x, y, and z coordinates (the world position of the chunk is these values
    //multiplied by CHUNK_SIZE
    ix: i32,
    iy: i32,
    iz: i32,
}

impl Chunk {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self {
            blocks: vec![Block::new(); CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE],
            ix: x,
            iy: y,
            iz: z,
        }
    }

    //Relative the position of the chunk (0 <= x, y, z < CHUNK_SIZE)
    pub fn get_block_relative(&self, x: usize, y: usize, z: usize) -> Block {
        //Out of bounds, return 0
        if x >= CHUNK_SIZE || y >= CHUNK_SIZE || z >= CHUNK_SIZE {
            return Block::new();
        }

        let index_y = CHUNK_SIZE * CHUNK_SIZE * y;
        let index_z = CHUNK_SIZE * z;
        let index_x = x;
        self.blocks[index_x + index_y + index_z]
    }

    pub fn set_block_relative(&mut self, x: usize, y: usize, z: usize, block: Block) {
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
        let index_x = x - CHUNK_SIZE_I32 * self.ix;
        let index_y = y - CHUNK_SIZE_I32 * self.iy;
        let index_z = z - CHUNK_SIZE_I32 * self.iz;

        //Out of bounds
        if index_x < 0
            || index_y < 0
            || index_z < 0
            || index_x >= CHUNK_SIZE_I32
            || index_y >= CHUNK_SIZE_I32
            || index_z >= CHUNK_SIZE_I32
        {
            return Block::new();
        }

        self.get_block_relative(index_x as usize, index_y as usize, index_z as usize)
    }

    //x, y, and z are absolute world positions
    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block: Block) {
        let index_x = x - CHUNK_SIZE_I32 * self.ix;
        let index_y = y - CHUNK_SIZE_I32 * self.iy;
        let index_z = z - CHUNK_SIZE_I32 * self.iz;

        //Out of bounds
        if index_x < 0
            || index_y < 0
            || index_z < 0
            || index_x >= CHUNK_SIZE_I32
            || index_y >= CHUNK_SIZE_I32
            || index_z >= CHUNK_SIZE_I32
        {
            return;
        }

        self.set_block_relative(index_x as usize, index_y as usize, index_z as usize, block)
    }

    pub fn get_chunk_pos(&self) -> ChunkPos {
        ChunkPos {
            x: self.ix,
            y: self.iy,
            z: self.iz,
        }
    }
}

pub struct World {
    chunks: Vec<Chunk>,
    size: usize,
    centerx: i32,
    centery: i32,
    centerz: i32,
}

impl World {
    pub fn new(range: usize) -> Self {
        let sz = 2 * range + 1;

        let mut chunklist = vec![];
        for y in -(range as i32)..=(range as i32) {
            for z in -(range as i32)..=(range as i32) {
                for x in -(range as i32)..=(range as i32) {
                    chunklist.push(Chunk::new(x, y, z));
                }
            }
        }

        Self {
            chunks: chunklist,
            size: sz,
            centerx: 0,
            centery: 0,
            centerz: 0,
        }
    }

    pub fn get_chunk_by_idx(&self, index: usize) -> &Chunk {
        &self.chunks[index]
    }

    pub fn get_chunk_count(&self) -> usize {
        self.chunks.len()
    }

    pub fn get_block_size(&self) -> usize {
        self.size * CHUNK_SIZE
    }

    pub fn get_block_range(&self) -> usize {
        self.get_block_size() / 2
    }

    pub fn get_chunk(&self, ix: i32, iy: i32, iz: i32) -> Option<&Chunk> {
        let range = (self.size - 1) as i32 / 2;

        //Check if out of bounds
        if ix < -range || iy < -range || iz < -range {
            return None;
        }

        if ix > range || iy > range || iz > range {
            return None;
        }

        let sz = self.size as i32;
        let index_y = (iy - self.centery + range) * sz * sz;
        let index_z = (iz - self.centerz + range) * sz;
        let index_x = ix - self.centerx + range;
        Some(&self.chunks[(index_z + index_y + index_x) as usize])
    }

    pub fn get_mut_chunk(&mut self, ix: i32, iy: i32, iz: i32) -> Option<&mut Chunk> {
        let range = (self.size - 1) as i32 / 2;

        if ix < -range || iy < -range || iz < -range {
            return None;
        }

        if ix > range || iy > range || iz > range {
            return None;
        }

        let sz = self.size as i32;
        let index_y = (iy - self.centery + range) * sz * sz;
        let index_z = (iz - self.centerz + range) * sz;
        let index_x = ix - self.centerx + range;
        Some(&mut self.chunks[(index_z + index_y + index_x) as usize])
    }

    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block: Block) {
        let (chunkx, chunky, chunkz) = world_to_chunk_position(x, y, z);
        let chunk = self.get_mut_chunk(chunkx, chunky, chunkz);
        if let Some(chunk) = chunk {
            chunk.set_block(x, y, z, block);
        }
    }

    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Block {
        let (chunkx, chunky, chunkz) = world_to_chunk_position(x, y, z);
        let chunk = self.get_chunk(chunkx, chunky, chunkz);
        if let Some(chunk) = chunk {
            return chunk.get_block(x, y, z);
        }
        Block::new()
    }

    pub fn gen_flat(&mut self) {
        //TODO: replace this code
        let range = self.get_block_range() as i32;
        let lower = -range + CHUNK_SIZE_I32 / 2;
        let upper = range + CHUNK_SIZE_I32 / 2;
        for y in lower..0 {
            for z in lower..upper {
                for x in lower..upper {
                    self.set_block(x, y, z, Block::new_id(1));
                }
            }
        }
    }
}

pub fn out_of_bounds(x: i32, y: i32, z: i32, offsetx: i32, offsety: i32, offsetz: i32) -> bool {
    x + offsetx < 0
        || y + offsety < 0
        || z + offsetz < 0
        || x + offsetx >= CHUNK_SIZE_I32
        || y + offsety >= CHUNK_SIZE_I32
        || z + offsetz >= CHUNK_SIZE_I32
}

pub fn wrap_coord(x: i32) -> i32 {
    let mut value = x;

    while value < 0 {
        value += CHUNK_SIZE_I32
    }

    value % CHUNK_SIZE_I32
}

pub fn raycast(pos: Vector3<f32>, dir: Vector3<f32>, range: f32, world: &World) -> (f32, f32, f32) {
    //TODO: replace with better code that is more accurate
    let mut x = pos.x.floor() as i32;
    let mut y = pos.y.floor() as i32;
    let mut z = pos.z.floor() as i32;

    let start_pos = pos;
    let mut current_pos = start_pos;
    while world.get_block(x, y, z).id == EMPTY_BLOCK
        && (current_pos - start_pos).magnitude() < range
    {
        current_pos += dir * 0.1;
        x = current_pos.x.floor() as i32;
        y = current_pos.y.floor() as i32;
        z = current_pos.z.floor() as i32;
    }

    (current_pos.x, current_pos.y, current_pos.z)
}
