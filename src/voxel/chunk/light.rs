use super::Chunk;
use crate::voxel::{light::LightSrc, CHUNK_SIZE_I32};

impl Chunk {
    //Returns a list of light sources and their positions
    pub fn get_light_srcs(&self, srcs: &mut Vec<((i32, i32, i32), LightSrc)>) {
        if self.is_empty() {
            return;
        }

        let pos = self.get_chunk_pos();
        for x in (pos.x * CHUNK_SIZE_I32)..((pos.x + 1) * CHUNK_SIZE_I32) {
            for y in (pos.y * CHUNK_SIZE_I32)..((pos.y + 1) * CHUNK_SIZE_I32) {
                for z in (pos.z * CHUNK_SIZE_I32)..((pos.z + 1) * CHUNK_SIZE_I32) {
                    if let Some(src) = self.get_block(x, y, z).light_src() {
                        srcs.push(((x, y, z), src))
                    }
                }
            }
        }
    }

    pub fn clear_light(&mut self) {
        for l in &mut self.light {
            l.clear();
        }
    }
}
