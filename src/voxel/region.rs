use super::{tile_data::TileData, world_to_chunk_position, Chunk};
use std::collections::HashMap;

pub mod load;
pub mod save;

pub const REGION_SIZE: usize = 4;
pub const REGION_SIZE_I32: i32 = REGION_SIZE as i32;

fn chunk_to_region(x: i32) -> i32 {
    if x < 0 && x % REGION_SIZE_I32 != 0 {
        //Handle negative coordinates
        x / REGION_SIZE_I32 - 1
    } else {
        //For positive coordinates and any negative coordinates that are a
        //multiple of REGION_SIZE
        x / REGION_SIZE_I32
    }
}

pub fn chunkpos_to_regionpos(x: i32, y: i32, z: i32) -> (i32, i32, i32) {
    (chunk_to_region(x), chunk_to_region(y), chunk_to_region(z))
}

pub fn regionpos_to_chunkpos(x: i32, y: i32, z: i32) -> (i32, i32, i32) {
    (
        x * REGION_SIZE_I32,
        y * REGION_SIZE_I32,
        z * REGION_SIZE_I32,
    )
}

#[derive(Clone)]
pub struct Region {
    pub chunks: Vec<Option<Chunk>>,
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Region {
    pub fn new(posx: i32, posy: i32, posz: i32) -> Self {
        Self {
            chunks: vec![None; REGION_SIZE * REGION_SIZE * REGION_SIZE],
            x: posx,
            y: posy,
            z: posz,
        }
    }

    //Get a chunk from the region, 0 <= x, y, z < REGION_SIZE
    pub fn get_relative_chunk(&self, x: usize, y: usize, z: usize) -> Option<&Chunk> {
        if x >= REGION_SIZE || y >= REGION_SIZE || z >= REGION_SIZE {
            return None;
        }

        let index = x * REGION_SIZE * REGION_SIZE + y * REGION_SIZE + z;
        self.chunks[index].as_ref()
    }

    //Set chunk in the region, 0 <= x, y, z < REGION_SIZE
    pub fn set_relative_chunk(&mut self, x: usize, y: usize, z: usize, chunk: Option<Chunk>) {
        if x >= REGION_SIZE || y >= REGION_SIZE || z >= REGION_SIZE {
            return;
        }

        let index = x * REGION_SIZE * REGION_SIZE + y * REGION_SIZE + z;
        self.chunks[index] = chunk;
    }

    //Replaces a chunk in the region at (x, y, z)
    //if it is out of bounds, nothing is done
    pub fn set_chunk(&mut self, x: i32, y: i32, z: i32, chunk: Option<Chunk>) {
        let (chunkx, chunky, chunkz) = regionpos_to_chunkpos(self.x, self.y, self.z);
        if x < chunkx || y < chunky || z < chunkz {
            return;
        }
        if x >= chunkx + REGION_SIZE_I32
            || y >= chunky + REGION_SIZE_I32
            || z >= chunkz + REGION_SIZE_I32
        {
            return;
        }
        self.set_relative_chunk(
            (x - chunkx) as usize,
            (y - chunky) as usize,
            (z - chunkz) as usize,
            chunk,
        );
    }

    pub fn set_tile_data(&mut self, x: i32, y: i32, z: i32, tile_data: TileData) {
        let (chunkx, chunky, chunkz) = world_to_chunk_position(x, y, z);
        let (startx, starty, startz) = regionpos_to_chunkpos(self.x, self.y, self.z);
        let ix = chunkx - startx;
        let iy = chunky - starty;
        let iz = chunkz - startz;
        if ix < 0 || iy < 0 || iz < 0 {
            return;
        }
        if ix >= REGION_SIZE_I32 || iy >= REGION_SIZE_I32 || iz >= REGION_SIZE_I32 {
            return;
        }
        let index = ix * REGION_SIZE_I32 * REGION_SIZE_I32 + iy * REGION_SIZE_I32 + iz;
        if let Some(chunk) = &mut self.chunks[index as usize] {
            chunk.set_tile_data(x, y, z, Some(tile_data));
        }
    }
}

pub fn get_region_chunks(region: &mut Region, chunks: &HashMap<(i32, i32, i32), Chunk>) {
    let (chunkx, chunky, chunkz) = regionpos_to_chunkpos(region.x, region.y, region.z);
    for x in chunkx..(chunkx + REGION_SIZE_I32) {
        for y in chunky..(chunky + REGION_SIZE_I32) {
            for z in chunkz..(chunkz + REGION_SIZE_I32) {
                if let Some(chunk) = chunks.get(&(x, y, z)) {
                    region.set_chunk(x, y, z, Some(chunk.clone()));
                }
            }
        }
    }
}

pub fn get_region_chunks_remove(region: &mut Region, chunks: &mut HashMap<(i32, i32, i32), Chunk>) {
    let (chunkx, chunky, chunkz) = regionpos_to_chunkpos(region.x, region.y, region.z);
    for x in chunkx..(chunkx + REGION_SIZE_I32) {
        for y in chunky..(chunky + REGION_SIZE_I32) {
            for z in chunkz..(chunkz + REGION_SIZE_I32) {
                if let Some(chunk) = chunks.get(&(x, y, z)) {
                    region.set_chunk(x, y, z, Some(chunk.clone()));
                    chunks.remove(&(x, y, z));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::voxel::Block;
    use std::collections::HashMap;

    fn init_chunks(chunkx: i32, chunky: i32, chunkz: i32) -> HashMap<(i32, i32, i32), Chunk> {
        let mut chunks = HashMap::new();
        let mut id = 1;
        for x in chunkx..(chunkx + REGION_SIZE_I32) {
            for y in chunky..(chunky + REGION_SIZE_I32) {
                for z in chunkz..(chunkz + REGION_SIZE_I32) {
                    let mut chunk = Chunk::new(0, 0, 0);
                    chunk.set_block_relative(0, 0, 0, Block::new_id(id));
                    chunks.insert((x, y, z), chunk);
                    id += 1;
                }
            }
        }
        chunks
    }

    fn check_region_eq_chunks(region: Region, chunks: HashMap<(i32, i32, i32), Chunk>) {
        for x in 0..REGION_SIZE {
            for y in 0..REGION_SIZE {
                for z in 0..REGION_SIZE {
                    let block1 = if let Some(chunk) = region.get_relative_chunk(x, y, z) {
                        chunk.get_block_relative(0, 0, 0)
                    } else {
                        Block::new()
                    };
                    let (rx, ry, rz) = regionpos_to_chunkpos(region.x, region.y, region.z);
                    let pos = (x as i32 + rx, y as i32 + ry, z as i32 + rz);
                    let block2 = if let Some(chunk) = chunks.get(&pos) {
                        chunk.get_block_relative(0, 0, 0)
                    } else {
                        Block::new()
                    };
                    assert_eq!(block1, block2);
                }
            }
        }
    }

    #[test]
    fn test_get_region_chunks() {
        let mut region = Region::new(0, 0, 0);
        let (chunkx, chunky, chunkz) = regionpos_to_chunkpos(region.x, region.y, region.z);
        let chunks = init_chunks(chunkx, chunky, chunkz);
        get_region_chunks(&mut region, &chunks);
        check_region_eq_chunks(region, chunks);
    }

    #[test]
    fn test_get_region_chunks2() {
        let mut region = Region::new(1, 0, 0);
        let (chunkx, chunky, chunkz) = regionpos_to_chunkpos(region.x, region.y, region.z);
        let chunks = init_chunks(chunkx - 1, chunky, chunkz);
        get_region_chunks(&mut region, &chunks);
        check_region_eq_chunks(region, chunks);
    }

    #[test]
    fn test_get_region_chunks3() {
        let mut region = Region::new(8, 16, 32);
        let (chunkx, chunky, chunkz) = regionpos_to_chunkpos(region.x, region.y, region.z);
        let chunks = init_chunks(chunkx, chunky, chunkz);
        get_region_chunks(&mut region, &chunks);
        check_region_eq_chunks(region, chunks);
    }

    #[test]
    fn test_get_region_chunks_remove() {
        let mut region = Region::new(0, 0, 0);
        let (chunkx, chunky, chunkz) = regionpos_to_chunkpos(region.x, region.y, region.z);
        let mut chunks = init_chunks(chunkx, chunky, chunkz);
        let chunks_clone = chunks.clone();
        get_region_chunks_remove(&mut region, &mut chunks);
        assert!(chunks.is_empty());
        check_region_eq_chunks(region, chunks_clone);
    }

    #[test]
    fn test_region_to_chunk() {
        let p = chunkpos_to_regionpos(0, 0, 0);
        assert_eq!(p, (0, 0, 0));
        let p = chunkpos_to_regionpos(1, 1, 1);
        assert_eq!(p, (0, 0, 0));
        let p = chunkpos_to_regionpos(-1, -1, -1);
        assert_eq!(p, (-1, -1, -1));
        let p = chunkpos_to_regionpos(-4, -4, -4);
        assert_eq!(p, (-1, -1, -1));
        let p = chunkpos_to_regionpos(4, 4, 4);
        assert_eq!(p, (1, 1, 1));
    }
}
