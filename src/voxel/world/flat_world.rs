use super::{
    gen_more::{find_in_range, get_chunks_to_generate, update_chunk_vao_table},
    World,
};
use crate::{
    gfx::ChunkTables,
    voxel::{Block, Chunk, CHUNK_SIZE_F32, CHUNK_SIZE_I32, INDESTRUCTIBLE},
};
use cgmath::Vector3;

fn gen_flat_chunk(chunk: &mut Chunk) {
    let chunkpos = chunk.get_chunk_pos();
    let posx = chunkpos.x * CHUNK_SIZE_I32;
    let posy = chunkpos.y * CHUNK_SIZE_I32;
    let posz = chunkpos.z * CHUNK_SIZE_I32;

    if chunkpos.y > 1 || chunkpos.y < -2 {
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

    //Generates new flat world chunks
    pub fn gen_more_flat(&mut self, pos: Vector3<f32>, chunktables: &mut ChunkTables) {
        //Check if the player is in the center chunk
        let x = (pos.x / CHUNK_SIZE_F32).floor() as i32;
        let y = (pos.y / CHUNK_SIZE_F32).floor() as i32;
        let z = (pos.z / CHUNK_SIZE_F32).floor() as i32;
        if x == self.centerx && y == self.centery && z == self.centerz {
            return;
        }

        //Find all chunks within range of the player
        let (in_range, out_of_range) = find_in_range(&self.chunks, x, y, z, self.range);
        //Find chunks to generate
        let to_generate = get_chunks_to_generate(in_range, x, y, z, self.range);

        //Delete old chunks
        self.delete_out_of_range(&out_of_range);

        //Generate new chunks
        for (chunkx, chunky, chunkz) in &to_generate {
            let pos = (*chunkx, *chunky, *chunkz);
            if self.chunk_cache.contains_key(&pos) {
                let new_chunk = self.chunk_cache.get(&pos);
                if let Some(new_chunk) = new_chunk {
                    self.chunks.insert(pos, new_chunk.clone());
                    self.chunk_cache.remove(&pos);
                }
                continue;
            } else if let Some(chunk) = Chunk::load_chunk(&self.path, *chunkx, *chunky, *chunkz) {
                self.chunks.insert(pos, chunk);
                continue;
            }

            let mut new_chunk = Chunk::new(*chunkx, *chunky, *chunkz);
            gen_flat_chunk(&mut new_chunk);
            self.chunks.insert(pos, new_chunk);
        }

        //Set the center position
        self.centerx = x;
        self.centery = y;
        self.centerz = z;

        update_chunk_vao_table(
            &mut chunktables.chunk_vaos,
            self.centerx,
            self.centery,
            self.centerz,
            self.range,
            &self.chunks,
            &to_generate,
        );

        update_chunk_vao_table(
            &mut chunktables.lava_vaos,
            self.centerx,
            self.centery,
            self.centerz,
            self.range,
            &self.chunks,
            &to_generate,
        );
    }
}
