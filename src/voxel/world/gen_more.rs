use super::World;
use crate::{gfx::ChunkVaoTable, voxel::Chunk};
use std::collections::{HashMap, HashSet};

type ChunkPosSet = HashSet<(i32, i32, i32)>;

//Returns a hashset of chunks in range of x, y, z and a hashset of chunks out
//of range in the form (in_range, out_of_range)
pub fn find_in_range(
    chunks: &HashMap<(i32, i32, i32), Chunk>,
    x: i32,
    y: i32,
    z: i32,
    range: i32,
) -> (ChunkPosSet, ChunkPosSet) {
    let mut in_range = HashSet::<(i32, i32, i32)>::new();
    let mut out_of_range = HashSet::<(i32, i32, i32)>::new();
    for (chunkx, chunky, chunkz) in chunks.keys() {
        if (chunkx - x).abs() <= range && (chunky - y).abs() <= range && (chunkz - z).abs() <= range
        {
            in_range.insert((*chunkx, *chunky, *chunkz));
        } else {
            out_of_range.insert((*chunkx, *chunky, *chunkz));
        }
    }

    (in_range, out_of_range)
}

//Returns a hashset of chunks that are now in range but need to be generated
pub fn get_chunks_to_generate(
    in_range: ChunkPosSet,
    x: i32,
    y: i32,
    z: i32,
    range: i32,
) -> ChunkPosSet {
    //Find chunks to generate
    let mut to_generate = HashSet::<(i32, i32, i32)>::new();
    for chunkx in (x - range)..=(x + range) {
        for chunky in (y - range)..=(y + range) {
            for chunkz in (z - range)..=(z + range) {
                if in_range.contains(&(chunkx, chunky, chunkz)) {
                    continue;
                }
                to_generate.insert((chunkx, chunky, chunkz));
            }
        }
    }

    to_generate
}

//Marks chunks that need to be deleted from the chunk vao table and those that
//need to be updated and those that need to be added
pub fn update_chunk_vao_table(
    chunktable: &mut ChunkVaoTable,
    centerx: i32,
    centery: i32,
    centerz: i32,
    range: i32,
    chunks: &HashMap<(i32, i32, i32), Chunk>,
    to_generate: &ChunkPosSet,
) {
    //Delete chunks that are out of range
    chunktable.delete_chunks(centerx, centery, centerz, range);

    //Mark chunks that need to have a vao generated
    for (chunkx, chunky, chunkz) in to_generate {
        chunktable.add_to_update(*chunkx, *chunky, *chunkz);
    }

    //Mark any chunks adjacent to the new border chunks that need to be regenerated
    for (chunkx, chunky, chunkz) in to_generate {
        let (x, y, z) = (*chunkx, *chunky, *chunkz);

        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    if dx == 0 && dy == 0 && dz == 0 {
                        continue;
                    }

                    if chunks.contains_key(&(x + dx, y + dy, z + dz)) {
                        chunktable.add_to_update(x + dx, y + dy, z + dz);
                    }
                }
            }
        }
    }
}

impl World {
    pub fn delete_out_of_range(&mut self, out_of_range: &ChunkPosSet) {
        //Delete old chunks
        for to_delete in out_of_range {
            let chunk = self.chunks.get(to_delete);
            if let Some(chunk) = chunk {
                self.add_to_chunk_cache(chunk.clone());
            }
            self.chunks.remove(to_delete);
        }
    }
}
