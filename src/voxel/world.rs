mod flat_world;
use super::{world_to_chunk_position, Block, Chunk, CHUNK_SIZE};

//World struct
pub struct World {
    //This only stores chunks that are near to the player
    chunks: Vec<Chunk>,
    //Size of world in chunks
    size: usize,
    //Position of the center chunk
    centerx: i32,
    centery: i32,
    centerz: i32,
}

impl World {
    //Create an empty world
    pub fn empty() -> Self {
        Self {
            chunks: vec![],
            size: 0,
            centerx: 0,
            centery: 0,
            centerz: 0,
        }
    }

    //Create a new chunk from a chunk render distance (range)
    pub fn new(range: usize) -> Self {
        //Calculate size of world
        let sz = 2 * range + 1;

        //Create chunk list
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

    //Returns an immutable reference to a chunk based on an index
    pub fn get_chunk_by_idx(&self, index: usize) -> &Chunk {
        &self.chunks[index]
    }

    //Returns length of chunk list
    pub fn get_chunk_count(&self) -> usize {
        self.chunks.len()
    }

    //Returns the size of the world in blocks
    pub fn get_block_size(&self) -> usize {
        self.size * CHUNK_SIZE
    }

    //Returns the range (about half the size) of the world in blocks
    pub fn get_block_range(&self) -> usize {
        self.get_block_size() / 2
    }

    //Returns an optional immutable reference to a chunk based on the chunk
    //position - will return none if such chunk is not found
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

    //Same is get_chunk except the reference is mutable
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

    //Sets a block based on position,
    //does nothing if the position is out of range for the world
    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block: Block) {
        let (chunkx, chunky, chunkz) = world_to_chunk_position(x, y, z);
        let chunk = self.get_mut_chunk(chunkx, chunky, chunkz);
        if let Some(chunk) = chunk {
            chunk.set_block(x, y, z, block);
        }
    }

    //Returns a block based on position
    //Returns an empty block if the position is out of range for the world
    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Block {
        let (chunkx, chunky, chunkz) = world_to_chunk_position(x, y, z);
        let chunk = self.get_chunk(chunkx, chunky, chunkz);
        if let Some(chunk) = chunk {
            return chunk.get_block(x, y, z);
        }
        Block::new()
    }
}
