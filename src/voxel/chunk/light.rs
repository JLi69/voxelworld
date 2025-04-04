use super::Chunk;
use crate::voxel::{
    light::{skylight_can_pass, Light, LightSrc, SkyLightMap, LU},
    world::light::calculate_sky_light,
    World, CHUNK_SIZE, CHUNK_SIZE_I32,
};
use std::collections::HashMap;

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

    //Returns whether the light has been initialized
    pub fn light_initialized(&self) -> bool {
        //If the lighting in the chunk has not been initialized, then the light
        //vector will be empty but once we start updating the light in the chunk
        //then the vector will be allocated and thus be nonempty
        !self.light.is_empty()
    }

    //Returns Some(y) if a block is found in a column,
    //None otherwise
    pub fn get_tallest_sky_block(&self, x: i32, z: i32) -> Option<i32> {
        if self.is_empty() {
            return None;
        }

        let pos = self.get_chunk_pos();
        for y in ((pos.y * CHUNK_SIZE_I32)..((pos.y + 1) * CHUNK_SIZE_I32)).rev() {
            if skylight_can_pass(self.get_block(x, y, z)) {
                continue;
            }
            return Some(y);
        }
        None
    }

    //Set all sky light to be 0
    pub fn clear_sky_light(&mut self) {
        for light in &mut self.light {
            light.set_sky(0u16);
        }
    }

    //Sets any block above the maximum block to be 15 for sky light
    //Leaves everything else as 0
    //Returns amount of light spawned
    pub fn init_sky_light(&mut self, map: &SkyLightMap) -> u32 {
        if self.light.is_empty() {
            self.light = vec![Light::black(); CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE];
        }

        let pos = self.get_chunk_pos();
        let mut count = 0;
        for x in (pos.x * CHUNK_SIZE_I32)..((pos.x + 1) * CHUNK_SIZE_I32) {
            for z in (pos.z * CHUNK_SIZE_I32)..((pos.z + 1) * CHUNK_SIZE_I32) {
                let height = map.get(x, z).unwrap_or(i32::MIN);
                if height >= (self.get_chunk_pos().y + 1) * CHUNK_SIZE_I32 {
                    continue;
                }

                for y in ((pos.y * CHUNK_SIZE_I32)..((pos.y + 1) * CHUNK_SIZE_I32)).rev() {
                    if y <= height {
                        break;
                    }
                    self.update_light(x, y, z, LU::new(Some(15), None, None, None));
                    count += 1;
                }
            }
        }
        count
    }

    pub fn get_sky_light_srcs(
        &self,
        world: &World,
        heights: &HashMap<(i32, i32), i32>,
        srcs: &mut Vec<((i32, i32, i32), LightSrc)>,
    ) {
        let pos = self.get_chunk_pos();
        for x in (pos.x * CHUNK_SIZE_I32)..((pos.x + 1) * CHUNK_SIZE_I32) {
            for z in (pos.z * CHUNK_SIZE_I32)..((pos.z + 1) * CHUNK_SIZE_I32) {
                if let Some(height) = heights.get(&(x, z)) {
                    if *height >= (pos.y + 1) * CHUNK_SIZE_I32 {
                        continue;
                    }
                }

                for y in (pos.y * CHUNK_SIZE_I32)..((pos.y + 1) * CHUNK_SIZE_I32) {
                    if self.get_light(x, y, z).skylight() == 15 {
                        break;
                    }
                    let b = self.get_block(x, y, z);
                    let light = calculate_sky_light(world, x, y, z, b);
                    if self.get_light(x, y, z).skylight() >= light {
                        continue;
                    }
                    srcs.push(((x, y, z), LightSrc::new(light, light, light)));
                }
            }
        }
    }

    pub fn get_sky_srcs_newly_loaded(
        &self,
        world: &World,
        heights: &HashMap<(i32, i32), i32>,
        srcs: &mut Vec<((i32, i32, i32), LightSrc)>,
    ) {
        let pos = self.get_chunk_pos();
        for x in (pos.x * CHUNK_SIZE_I32)..((pos.x + 1) * CHUNK_SIZE_I32) {
            for z in (pos.z * CHUNK_SIZE_I32)..((pos.z + 1) * CHUNK_SIZE_I32) {
                if let Some(height) = heights.get(&(x, z)) {
                    if *height >= (pos.y + 1) * CHUNK_SIZE_I32 {
                        continue;
                    }
                }

                for y in (pos.y * CHUNK_SIZE_I32)..((pos.y + 1) * CHUNK_SIZE_I32) {
                    if self.get_light(x, y, z).skylight() == 15 {
                        break;
                    }
                    let b = self.get_block(x, y, z);
                    let light = calculate_sky_light(world, x, y, z, b);
                    if light != 14 {
                        continue;
                    }
                    srcs.push(((x, y, z), LightSrc::new(light, light, light)));
                }
            }
        }
    }
}
