use super::{Block, ChunkPos, CHUNK_SIZE, CHUNK_SIZE_I32, EMPTY_BLOCK};

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
            blocks: vec![],
            ix: x,
            iy: y,
            iz: z,
        }
    }

    pub fn is_empty(&self) -> bool {
        for b in &self.blocks {
            if b.id != EMPTY_BLOCK {
                return false;
            }
        }

        true
    }

    //Frees memory if the chunk is completely empty
    pub fn handle_empty(&mut self) {
        if self.blocks.is_empty() {
            return;
        }

        if self.is_empty() {
            self.blocks.clear();
        }
    }

    //Relative the position of the chunk (0 <= x, y, z < CHUNK_SIZE)
    pub fn get_block_relative(&self, x: usize, y: usize, z: usize) -> Block {
        if self.blocks.is_empty() {
            return Block::new();
        }

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
        if self.blocks.is_empty() && block.id != EMPTY_BLOCK {
            self.blocks = vec![Block::new(); CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE];
        } else if !self.blocks.is_empty() && block.id == EMPTY_BLOCK {
            self.handle_empty();
        }

        if self.blocks.is_empty() && block.id == EMPTY_BLOCK {
            return;
        }

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

        self.set_block_relative(index_x as usize, index_y as usize, index_z as usize, block);
    }

    pub fn get_chunk_pos(&self) -> ChunkPos {
        ChunkPos::new(self.ix, self.iy, self.iz)
    }
}
