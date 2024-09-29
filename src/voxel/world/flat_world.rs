use super::World;
use crate::voxel::{Block, Chunk, CHUNK_SIZE_I32, INDESTRUCTIBLE};

fn gen_flat_chunk(chunk: &mut Chunk) {
    let chunkpos = chunk.get_chunk_pos();
    let posx = chunkpos.x * CHUNK_SIZE_I32;
    let posy = chunkpos.y * CHUNK_SIZE_I32;
    let posz = chunkpos.z * CHUNK_SIZE_I32;

    if chunkpos.y > 0 || chunkpos.y < -2 {
        return;
    }

    for x in posx..(posx + CHUNK_SIZE_I32) {
        for y in posy..(posy + CHUNK_SIZE_I32) {
            for z in posz..(posz + CHUNK_SIZE_I32) {
                if y == -1 {
                    //Grass
                    chunk.set_block(x, y, z, Block::new_id(1));
                } else if (-4..-1).contains(&y) {
                    chunk.set_block(x, y, z, Block::new_id(4));
                } else if (-62..-4).contains(&y) {
                    //Stone
                    chunk.set_block(x, y, z, Block::new_id(2));
                } else if y == -63 {
                    //Bottom of the world
                    chunk.set_block(x, y, z, Block::new_id(INDESTRUCTIBLE));
                }
            }
        }
    }
}

impl World {
    //Generates a flat world
    pub fn gen_flat(&mut self) {
        for chunk in &mut self.chunks.values_mut() {
            gen_flat_chunk(chunk);
        }
    }
}
