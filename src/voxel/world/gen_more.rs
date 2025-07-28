use cgmath::Vector3;

use super::{World, WorldGenType};
use crate::{
    gfx::{ChunkTables, ChunkVaoTable},
    voxel::{
        region::{chunkpos_to_regionpos, Region},
        Chunk, CHUNK_SIZE_F32,
    },
};
use std::collections::{HashMap, HashSet, VecDeque};

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

    let mut to_update = HashSet::<(i32, i32, i32)>::new();
    //Mark chunks that need to have a vao generated
    for (chunkx, chunky, chunkz) in to_generate {
        to_update.insert((*chunkx, *chunky, *chunkz));
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
                        to_update.insert((x + dx, y + dy, z + dz));
                    }
                }
            }
        }
    }

    for (x, y, z) in to_update {
        chunktable.add_to_update(x, y, z);
    }
}

pub fn update_chunk_tables(
    chunktables: &mut ChunkTables,
    x: i32,
    y: i32,
    z: i32,
    range: i32,
    chunks: &HashMap<(i32, i32, i32), Chunk>,
    to_generate: &ChunkPosSet,
) {
    let chunk_vaos = &mut chunktables.chunk_vaos;
    update_chunk_vao_table(chunk_vaos, x, y, z, range, chunks, to_generate);
    let lava_vaos = &mut chunktables.lava_vaos;
    update_chunk_vao_table(lava_vaos, x, y, z, range, chunks, to_generate);
    let water_vaos = &mut chunktables.water_vaos;
    update_chunk_vao_table(water_vaos, x, y, z, range, chunks, to_generate);
    let nonvoxel_vaos = &mut chunktables.non_voxel_vaos;
    update_chunk_vao_table(nonvoxel_vaos, x, y, z, range, chunks, to_generate);
}

pub type Column = (i32, i32, HashSet<i32>);

pub struct LoadChunkQueue {
    queue: VecDeque<(i32, i32)>,
    coords_xz: HashMap<(i32, i32), Vec<i32>>,
    coords_unordered: ChunkPosSet,
}

impl LoadChunkQueue {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            coords_xz: HashMap::new(),
            coords_unordered: ChunkPosSet::new(),
        }
    }

    //add (x, z) to the queue
    pub fn add_column_to_queue(&mut self, x: i32, z: i32) {
        if self.coords_xz.contains_key(&(x, z)) {
            return;
        }

        self.queue.push_back((x, z));
    }

    //Add all chunks with coordinates of the form (x, *, z) to the queue
    pub fn add_chunk(&mut self, x: i32, y: i32, z: i32) {
        let pos = (x, y, z);
        if self.coords_unordered.contains(&pos) {
            return;
        }

        self.add_column_to_queue(x, z);

        if let Some(ycoords) = self.coords_xz.get_mut(&(x, z)) {
            ycoords.push(y);
        } else {
            self.coords_xz.insert((x, z), vec![y]);
        }

        self.coords_unordered.insert(pos);
    }

    //Add all chunks in a set of chunks
    pub fn add_from_set(&mut self, set: &ChunkPosSet) {
        for (x, y, z) in set.iter().copied() {
            self.add_chunk(x, y, z);
        }
    }

    #[allow(dead_code)]
    pub fn count(&self) -> usize {
        let mut total = 0;
        for xz in self.coords_xz.values() {
            total += xz.len();
        }
        total
    }

    //Removes the top element
    pub fn pop(&mut self) -> Option<Column> {
        let (x, z) = self.queue.pop_front()?;
        let yvals = self.coords_xz.get(&(x, z))?.to_vec();
        for y in &yvals {
            self.coords_unordered.remove(&(x, *y, z));
        }
        self.coords_xz.remove(&(x, z));
        let mut yset = HashSet::new();
        for y in yvals {
            yset.insert(y);
        }
        Some((x, z, yset))
    }

    pub fn front(&self) -> Option<Column> {
        let (x, z) = self.queue.front().copied()?;
        let yvals = self.coords_xz.get(&(x, z))?.to_vec();
        let mut yset = HashSet::new();
        for y in yvals {
            yset.insert(y);
        }
        Some((x, z, yset))
    }

    pub fn is_empty(&self) -> bool {
        self.coords_xz.is_empty()
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

    pub fn load_column(&mut self, col: Column) {
        let mut update_list = HashSet::new();
        let (x, z, yvals) = col;

        let mut loaded = 0;
        for y in yvals.iter().copied() {
            if self.chunks.contains_key(&(x, y, z)) {
                loaded += 1;
            }
        }

        if loaded == yvals.len() {
            return;
        }

        //Attempt to load from the cache
        for y in yvals.iter().copied() {
            if !self.in_range(x, y, z) {
                continue;
            }

            if self.chunks.contains_key(&(x, y, z)) {
                continue;
            }

            if let Some(new_chunk) = self.chunk_cache.get(&(x, y, z)) {
                self.chunks.insert((x, y, z), new_chunk.clone());
                self.chunk_cache.remove(&(x, y, z));
            }
        }

        //Attempt to load from the file system
        let mut loaded = HashSet::new();
        for y in yvals.iter().copied() {
            if !self.in_range(x, y, z) {
                continue;
            }

            let (rx, ry, rz) = chunkpos_to_regionpos(x, y, z);
            if self.chunks.contains_key(&(x, y, z)) {
                continue;
            }

            if loaded.contains(&(rx, ry, rz)) {
                continue;
            }
            loaded.insert((rx, ry, rz));

            if let Some(region) = Region::load_region(&self.path, rx, ry, rz) {
                self.add_region_col(region, x, z);
            }
        }

        //Generate the new chunks
        match self.gen_type {
            WorldGenType::Flat => self.generate_column_flat(x, z, &yvals),
            WorldGenType::OldGen => self.generate_column_old(x, z, &yvals),
            WorldGenType::DefaultGen => self.generate_column_default(x, z, &yvals),
        }

        for y in yvals.iter().copied() {
            update_list.insert((x, y, z));
        }

        //Generate lighting in chunk column
        update_list.extend(self.init_light_new_chunks(&update_list));
        self.chunktable_update_list.extend(update_list);
    }

    //Load all chunks in the 3 x 3 column surrounding the player
    pub fn force_load(&mut self) {
        let yvals: HashSet<i32> =
            ((self.centery - self.range)..=(self.centery + self.range)).collect();

        for x in (self.centerx - 1)..=(self.centerx + 1) {
            for z in (self.centerz - 1)..=(self.centerz + 1) {
                let col = (x, z, yvals.clone());
                self.load_column(col);
            }
        }
    }

    //Returns true if something was loaded
    pub fn pop_load(&mut self) -> bool {
        if let Some(col) = self.to_load.pop() {
            self.load_column(col);
            return true;
        }

        false
    }

    pub fn load_from_queue(&mut self, max_time: f32) -> HashSet<(i32, i32, i32)> {
        let start = std::time::Instant::now();
        let mut loaded = HashSet::new();
        if let Some((x, z, yvals)) = self.to_load.front() {
            for y in yvals {
                loaded.insert((x, y, z));
            }
        }
        while self.pop_load() && start.elapsed().as_secs_f32() < max_time {
            if let Some((x, z, yvals)) = self.to_load.front() {
                for y in yvals {
                    loaded.insert((x, y, z));
                }
            }
        }
        loaded
    }

    pub fn update_chunktables(&mut self, chunktables: &mut ChunkTables) {
        if self.to_load.is_empty() && !self.chunktable_update_list.is_empty() {
            update_chunk_tables(
                chunktables,
                self.centerx,
                self.centery,
                self.centerz,
                self.range,
                &self.chunks,
                &self.chunktable_update_list,
            );
            self.chunktable_update_list.clear();
        }
    }

    //Returns if chunk coordinates are within range
    pub fn in_range(&self, x: i32, y: i32, z: i32) -> bool {
        (x - self.centerx).abs() <= self.range
            && (y - self.centery).abs() <= self.range
            && (z - self.centerz).abs() <= self.range
    }

    pub fn update_generation_queue(&mut self, pos: Vector3<f32>) {
        //Check if the player is in the center chunk
        let x = (pos.x / CHUNK_SIZE_F32).floor() as i32;
        let y = (pos.y / CHUNK_SIZE_F32).floor() as i32;
        let z = (pos.z / CHUNK_SIZE_F32).floor() as i32;
        let dx = (x - self.centerx).abs();
        let dy = (y - self.centery).abs();
        let dz = (z - self.centerz).abs();
        if dx <= 1 && dy <= 1 && dz <= 1 {
            return;
        }

        //Find all chunks within range of the player
        let (in_range, out_of_range) = find_in_range(&self.chunks, x, y, z, self.range);
        //Find chunks to generate
        let to_generate = get_chunks_to_generate(in_range, x, y, z, self.range);

        //Delete old chunks
        self.delete_out_of_range(&out_of_range);

        //Set the center position
        self.centerx = x;
        self.centery = y;
        self.centerz = z;

        self.to_load.add_from_set(&to_generate);
    }
}
