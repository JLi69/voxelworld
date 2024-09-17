use super::World;
use crate::voxel::{CHUNK_SIZE_I32, Block, Chunk};

fn gen_flat_chunk(chunk: &mut Chunk) {
    let chunkpos = chunk.get_chunk_pos();
    let posx = chunkpos.x * CHUNK_SIZE_I32;
    let posy = chunkpos.y * CHUNK_SIZE_I32;
    let posz = chunkpos.z * CHUNK_SIZE_I32;

    if posy > 0 {
        return;
    }

    for x in posx..(posx + CHUNK_SIZE_I32) {
        for y in posy..(posy + CHUNK_SIZE_I32) {
            for z in posz..(posz + CHUNK_SIZE_I32) {
                if y == -1 {
                    chunk.set_block(x, y, z, Block::new_id(1));
                } else if y < -1 {
                    chunk.set_block(x, y, z, Block::new_id(2));
                }
            }
        }
    }
}

impl World { 
    //Generates a flat world
    pub fn gen_flat(&mut self) {
        for chunk in &mut self.chunks {
            gen_flat_chunk(chunk);
        }
    }
}
