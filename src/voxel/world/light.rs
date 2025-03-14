use super::{block_update::get_chunktable_updates, World};
use crate::voxel::{
    light::{Light, LightSrc, SkyLightMap, LU, skylight_can_pass},
    world_to_chunk_position, Block, Chunk, CHUNK_SIZE_I32, EMPTY_BLOCK,
};
use std::collections::{HashMap, HashSet, VecDeque};

const ADJ: [(i32, i32, i32); 6] = [
    (-1, 0, 0),
    (1, 0, 0),
    (0, -1, 0),
    (0, 1, 0),
    (0, 0, -1),
    (0, 0, 1),
];

const ADJ_2D: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

type Propagation = (i32, i32, i32, u16);

type ChunkList = HashSet<(i32, i32, i32)>;

pub fn light_can_pass(block: Block) -> bool {
    block.transparent() || block.id == EMPTY_BLOCK || block.shape() != 0
}

fn check_visited(visited: &HashMap<(i32, i32, i32), u16>, xyz: (i32, i32, i32), val: u16) -> bool {
    if let Some(v) = visited.get(&xyz) {
        if *v >= val {
            return true;
        }
    }

    false
}

fn add_visited(visited: &mut HashMap<(i32, i32, i32), u16>, xyz: (i32, i32, i32), val: u16) {
    if check_visited(visited, xyz, val) {
        return;
    }
    visited.insert(xyz, val);
}

fn propagate_channel(
    queue: &mut VecDeque<Propagation>,
    world: &mut World,
    channel: fn(Light) -> u16,
    update: fn(u16) -> LU,
) -> ChunkList {
    let mut visited = HashMap::<(i32, i32, i32), u16>::new();
    let mut updated = ChunkList::new();
    while !queue.is_empty() {
        let top = queue.pop_front();
        if let Some((x, y, z, val)) = top {
            if check_visited(&visited, (x, y, z), val) {
                continue;
            }
            let light = world.get_light(x, y, z);
            if channel(light) >= val {
                continue;
            }
            add_visited(&mut visited, (x, y, z), val);
            get_chunktable_updates(x, y, z, &mut updated);
            world.update_light(x, y, z, update(val));

            if val <= 1 {
                continue;
            }

            for (dx, dy, dz) in ADJ {
                if world.out_of_bounds(x + dx, y + dy, z + dz) {
                    continue;
                }
                let adj = (x + dx, y + dy, z + dz);
                if check_visited(&visited, adj, val - 1) {
                    continue;
                }
                let block = world.get_block(x + dx, y + dy, z + dz);
                if !light_can_pass(block) {
                    add_visited(&mut visited, adj, 0xff);
                    continue;
                }
                queue.push_back((x + dx, y + dy, z + dz, val - 1));
            }
        }
    }
    updated
}

//Propagate a light source
pub fn propagate(world: &mut World, srcs: &[((i32, i32, i32), LightSrc)]) -> ChunkList {
    let mut queue = VecDeque::new();
    let mut updated = ChunkList::new();
    //Propagate red
    for ((x, y, z), src) in srcs {
        queue.push_back((*x, *y, *z, src.r));
    }
    updated.extend(propagate_channel(
        &mut queue,
        world,
        |light| light.r(),
        |v| LU::new(None, Some(v), None, None),
    ));
    //Propagate green
    for ((x, y, z), src) in srcs {
        queue.push_back((*x, *y, *z, src.g));
    }
    updated.extend(propagate_channel(
        &mut queue,
        world,
        |light| light.g(),
        |v| LU::new(None, None, Some(v), None),
    ));
    //Propagate blue
    for ((x, y, z), src) in srcs {
        queue.push_back((*x, *y, *z, src.b));
    }
    updated.extend(propagate_channel(
        &mut queue,
        world,
        |light| light.b(),
        |v| LU::new(None, None, None, Some(v)),
    ));
    updated
}

pub fn attenuate(channel: u16) -> u16 {
    if channel == 0 {
        0
    } else {
        channel - 1
    }
}

pub fn calculate_light(world: &World, x: i32, y: i32, z: i32, block: Block) -> Light {
    if light_can_pass(block) {
        let mut light = Light::black();
        for (dx, dy, dz) in ADJ {
            let adj_light = world.get_light(x + dx, y + dy, z + dz);
            light.set_red(light.r().max(attenuate(adj_light.r())));
            light.set_green(light.g().max(attenuate(adj_light.g())));
            light.set_blue(light.b().max(attenuate(adj_light.b())));
        }
        if let Some(src) = block.light_src() {
            light.set_red(light.r().max(src.r));
            light.set_green(light.g().max(src.g));
            light.set_blue(light.b().max(src.b));
        }
        light
    } else {
        Light::black()
    }
}

pub fn calculate_sky_light(world: &World, x: i32, y: i32, z: i32, block: Block) -> u16 {
    if world.get_skylightmap(x, z).unwrap_or(i32::MIN) < y {
        return 15;
    }

    if light_can_pass(block) {
        let mut light = 0;
        for (dx, dy, dz) in ADJ {
            let adj_light = world.get_light(x + dx, y + dy, z + dz);
            light = light.max(attenuate(adj_light.skylight()));
        }
        light
    } else {
        0
    }
}

fn propagate_channel_updates(
    queue: &mut VecDeque<(i32, i32, i32)>,
    world: &mut World,
    channel: fn(Light) -> u16,
    update: fn(u16) -> LU,
) -> ChunkList {
    let mut updated = ChunkList::new();
    let mut visited = HashSet::new();
    while !queue.is_empty() {
        let top = queue.pop_front();
        if let Some((x, y, z)) = top {
            let light = world.get_light(x, y, z);
            let block = world.get_block(x, y, z);
            let val = channel(calculate_light(world, x, y, z, block));
            if channel(light) == val {
                continue;
            }
            get_chunktable_updates(x, y, z, &mut updated);
            world.update_light(x, y, z, update(val));

            for (dx, dy, dz) in ADJ {
                if visited.contains(&(x + dx, y + dy, z + dz)) {
                    continue;
                }
                let adj_block = world.get_block(x + dx, y + dy, z + dz);
                if !light_can_pass(adj_block) {
                    visited.insert((x + dx, y + dy, z + dz));
                    continue;
                }
                queue.push_back((x + dx, y + dy, z + dz));
            }
        }
    }
    updated
}

pub fn propagate_sky_updates(world: &mut World, blocks: &[(i32, i32, i32)]) -> ChunkList {
    let mut queue = VecDeque::new();
    for (x, y, z) in blocks {
        queue.push_back((*x, *y, *z));
    }
    let mut updated = ChunkList::new();
    let mut visited = HashSet::new();
    while !queue.is_empty() {
        let top = queue.pop_front();
        if let Some((x, y, z)) = top {
            let light = world.get_light(x, y, z);
            let block = world.get_block(x, y, z);
            let val = calculate_sky_light(world, x, y, z, block);
            if light.skylight() == val {
                continue;
            }
            get_chunktable_updates(x, y, z, &mut updated);
            world.update_light(x, y, z, LU::new(Some(val), None, None, None));

            for (dx, dy, dz) in ADJ {
                if visited.contains(&(x + dx, y + dy, z + dz)) {
                    continue;
                }
                let adj_block = world.get_block(x + dx, y + dy, z + dz);
                if !light_can_pass(adj_block) {
                    visited.insert((x + dx, y + dy, z + dz));
                    continue;
                }
                queue.push_back((x + dx, y + dy, z + dz));
            }
        }
    }

    updated
}

pub fn propagate_updates(world: &mut World, blocks: &[(i32, i32, i32)]) -> ChunkList {
    let mut queue = VecDeque::new();
    let mut updated = ChunkList::new();
    //Propagate red
    for (x, y, z) in blocks {
        queue.push_back((*x, *y, *z));
    }
    updated.extend(propagate_channel_updates(
        &mut queue,
        world,
        |light| light.r(),
        |v| LU::new(None, Some(v), None, None),
    ));
    //Propagate green
    for (x, y, z) in blocks {
        queue.push_back((*x, *y, *z));
    }
    updated.extend(propagate_channel_updates(
        &mut queue,
        world,
        |light| light.g(),
        |v| LU::new(None, None, Some(v), None),
    ));
    //Propagate blue
    for (x, y, z) in blocks {
        queue.push_back((*x, *y, *z));
    }
    updated.extend(propagate_channel_updates(
        &mut queue,
        world,
        |light| light.b(),
        |v| LU::new(None, None, None, Some(v)),
    ));

    updated
}

//Used to test if the light in a set of chunks is of the correct value
//The assertion will fail if we do not get what we expect and crash the program
#[allow(dead_code)]
fn validate_light(world: &World, chunks: &HashSet<(i32, i32, i32)>) {
    for (x, y, z) in chunks {
        if !world.chunks.contains_key(&(*x, *y, *z)) {
            continue;
        }

        for ix in (x * CHUNK_SIZE_I32)..((x + 1) * CHUNK_SIZE_I32) {
            for iy in (y * CHUNK_SIZE_I32)..((y + 1) * CHUNK_SIZE_I32) {
                for iz in (z * CHUNK_SIZE_I32)..((z + 1) * CHUNK_SIZE_I32) {
                    let b = world.get_block(ix, iy, iz);
                    let expected = calculate_light(world, ix, iy, iz, b);
                    let light = world.get_light(ix, iy, iz);
                    assert_eq!(expected.get_rgb::<u16>(), light.get_rgb())
                }
            }
        }
    }
}

fn propagate_channel_fast(
    queue: &mut VecDeque<Propagation>,
    world: &mut World,
    channel: fn(Light) -> u16,
    update: fn(u16) -> LU,
) {
    let mut visited = HashMap::<(i32, i32, i32), u16>::new();
    let mut updated = vec![];
    while !queue.is_empty() {
        while let Some((x, y, z, val)) = queue.pop_front() {
            let light = world.get_light(x, y, z);
            if channel(light) >= val {
                continue;
            }
            world.update_light(x, y, z, update(val));

            if val <= 1 {
                continue;
            }

            updated.push((x, y, z, val));
        }

        for (x, y, z, val) in &updated {
            for (dx, dy, dz) in ADJ {
                if world.out_of_bounds(x + dx, y + dy, z + dz) {
                    continue;
                }
                let adj = (x + dx, y + dy, z + dz);
                if check_visited(&visited, adj, val - 1) {
                    continue;
                }
                let block = world.get_block(x + dx, y + dy, z + dz);
                if !light_can_pass(block) {
                    continue;
                }
                let light = world.get_light(x + dx, y + dy, z + dz);
                if channel(light) >= val - 1 {
                    continue;
                }
                add_visited(&mut visited, adj, val - 1);
                queue.push_back((x + dx, y + dy, z + dz, val - 1));
            }
        }
        updated.clear();
    }
}

//Propagate a light source
//This function is different from `propagate` in that it does not return
//a list of chunks that have been updated and thus should require less memory
//allocations and therefore be slightly faster
pub fn propagate_fast(world: &mut World, srcs: &[((i32, i32, i32), LightSrc)]) {
    let mut queue = VecDeque::new();
    //Propagate red
    for ((x, y, z), src) in srcs {
        queue.push_back((*x, *y, *z, src.r));
    }
    propagate_channel_fast(
        &mut queue,
        world,
        |light| light.r(),
        |v| LU::new(None, Some(v), None, None),
    );
    //Propagate green
    for ((x, y, z), src) in srcs {
        queue.push_back((*x, *y, *z, src.g));
    }
    propagate_channel_fast(
        &mut queue,
        world,
        |light| light.g(),
        |v| LU::new(None, None, Some(v), None),
    );
    //Propagate blue
    for ((x, y, z), src) in srcs {
        queue.push_back((*x, *y, *z, src.b));
    }
    propagate_channel_fast(
        &mut queue,
        world,
        |light| light.b(),
        |v| LU::new(None, None, None, Some(v)),
    );
}

pub fn propagate_sky_fast(world: &mut World, srcs: &[((i32, i32, i32), LightSrc)]) {
    let mut queue = VecDeque::new();
    for ((x, y, z), src) in srcs {
        queue.push_back((*x, *y, *z, src.r));
    }
    propagate_channel_fast(
        &mut queue,
        world,
        |light| light.skylight(),
        |v| LU::new(Some(v), None, None, None),
    );
}

fn compare_light(light: Light, adj_light: Light) -> Light {
    let mut res = Light::black();
    res.set_red(light.r().max(attenuate(adj_light.r())));
    res.set_green(light.g().max(attenuate(adj_light.g())));
    res.set_blue(light.b().max(attenuate(adj_light.b())));
    res
}

fn contains_chunk(chunk: Option<&Chunk>, chunks: &HashSet<(i32, i32, i32)>) -> bool {
    if let Some(chunk) = chunk {
        let p = chunk.get_chunk_pos();
        chunks.contains(&(p.x, p.y, p.z))
    } else {
        false
    }
}

pub fn add_neighbor_src(
    pos: (i32, i32, i32),
    diff: (i32, i32, i32),
    chunk: &Chunk,
    adj_chunk: Option<&Chunk>,
    srcs: &mut HashMap<(i32, i32, i32), Light>,
) {
    let (x, y, z) = pos;
    let (dx, dy, dz) = diff;

    //Ignore any block that is opaque
    if !light_can_pass(chunk.get_block(x, y, z)) {
        return;
    }

    //Get the current source in srcs
    let current_src = srcs.get(&(x, y, z)).copied().unwrap_or(Light::black());
    let adj_light = if let Some(adj_chunk) = adj_chunk {
        adj_chunk.get_light(x + dx, y + dy, z + dz)
    } else {
        return;
    };

    let light = compare_light(current_src, adj_light);

    let current_light = chunk.get_light(x, y, z);
    if light.r() <= current_light.r()
        && light.g() <= current_light.g()
        && light.b() <= current_light.b()
    {
        return;
    }

    srcs.insert((x, y, z), light);
}

pub fn scan_neighbor<T>(
    chunk: &Chunk,
    adj_chunk: Option<&Chunk>,
    diff: (i32, i32, i32),
    srcs: &mut HashMap<(i32, i32, i32), Light>,
    chunks: &HashSet<(i32, i32, i32)>,
    get_pos: T,
) where
    T: Fn(i32, i32) -> (i32, i32, i32),
{
    if adj_chunk.is_none() {
        return;
    }

    if contains_chunk(adj_chunk, chunks) {
        return;
    }

    for i in 0..CHUNK_SIZE_I32 {
        for j in 0..CHUNK_SIZE_I32 {
            let pos = get_pos(i, j);
            add_neighbor_src(pos, diff, chunk, adj_chunk, srcs);
        }
    }
}

pub fn get_neighbor_srcs(
    chunk: &Chunk,
    adj_chunks: &[Option<&Chunk>; 6],
    srcs: &mut HashMap<(i32, i32, i32), Light>,
    chunks: &HashSet<(i32, i32, i32)>,
) {
    let chunkpos = chunk.get_chunk_pos();
    let startx = chunkpos.x * CHUNK_SIZE_I32;
    let starty = chunkpos.y * CHUNK_SIZE_I32;
    let startz = chunkpos.z * CHUNK_SIZE_I32;
    let endx = startx + CHUNK_SIZE_I32 - 1;
    let endy = starty + CHUNK_SIZE_I32 - 1;
    let endz = startz + CHUNK_SIZE_I32 - 1;

    scan_neighbor(chunk, adj_chunks[4], (0, 0, -1), srcs, chunks, |i, j| {
        (startx + i, starty + j, startz)
    });
    scan_neighbor(chunk, adj_chunks[1], (0, -1, 0), srcs, chunks, |i, j| {
        (startx + i, starty, startz + j)
    });
    scan_neighbor(chunk, adj_chunks[2], (-1, 0, 0), srcs, chunks, |i, j| {
        (startx, starty + i, startz + j)
    });

    scan_neighbor(chunk, adj_chunks[5], (0, 0, 1), srcs, chunks, |i, j| {
        (startx + i, starty + j, endz)
    });
    scan_neighbor(chunk, adj_chunks[0], (0, 1, 0), srcs, chunks, |i, j| {
        (startx + i, endy, startz + j)
    });
    scan_neighbor(chunk, adj_chunks[3], (1, 0, 0), srcs, chunks, |i, j| {
        (endx, starty + i, startz + j)
    });
}

impl World {
    //Called when the world is first loaded
    pub fn init_block_light(&mut self) {
        let start = std::time::Instant::now();

        let mut srcs = vec![];
        for chunk in self.chunks.values() {
            chunk.get_light_srcs(&mut srcs);
        }
        propagate_fast(self, &srcs);

        let time = start.elapsed().as_millis();
        eprintln!("Took {time} ms to init light");
    }

    pub fn get_skylightmap(&self, x: i32, z: i32) -> Option<i32> {
        let (chunkx, _, chunkz) = world_to_chunk_position(x, 0, z);
        if let Some(map) = self.skylightmap.get(&(chunkx, chunkz)) {
            map.get(x, z)
        } else {
            None
        }
    }

    pub fn set_skylightmap(&mut self, x: i32, z: i32, val: Option<i32>) {
        let (chunkx, _, chunkz) = world_to_chunk_position(x, 0, z);
        if let Some(map) = self.skylightmap.get_mut(&(chunkx, chunkz)) {
            map.set(x, z, val);
        }
    }

    //Called when the world is first loaded
    pub fn init_sky_light(&mut self) {
        let start = std::time::Instant::now();

        for (x, _, z) in self.chunks.keys() {
            if self.skylightmap.contains_key(&(*x, *z)) {
                continue;
            }
            self.skylightmap.insert((*x, *z), SkyLightMap::new(*x, *z));
        }

        for ((x, _, z), chunk) in &self.chunks {
            if let Some(map) = self.skylightmap.get_mut(&(*x, *z)) {
                map.init_map_from_chunk(chunk);
            }
        }

        for chunk in self.chunks.values_mut() {
            let chunkpos = chunk.get_chunk_pos();
            let x = chunkpos.x;
            let z = chunkpos.z;
            if let Some(map) = self.skylightmap.get(&(x, z)) {
                chunk.init_sky_light(map);
            }
        }

        //Find the lowest adjacent height, this will be used to 'cull' out
        //columns in chunks that are too low down to be affected by sky light
        let mut heights = HashMap::<(i32, i32), i32>::new();
        for map in self.skylightmap.values() {
            for x in (map.x * CHUNK_SIZE_I32)..((map.x + 1) * CHUNK_SIZE_I32) {
                for z in (map.z * CHUNK_SIZE_I32)..((map.z + 1) * CHUNK_SIZE_I32) {
                    let mut h = map.get(x, z).unwrap_or(i32::MAX);
                    for (dx, dz) in ADJ_2D {
                        let adj_h = self.get_skylightmap(x + dx, z + dz).unwrap_or(i32::MAX);
                        h = h.min(adj_h);
                    }
                    heights.insert((x, z), h - 1);
                }
            }
        }

        let mut srcs = vec![];
        for chunk in self.chunks.values() {
            chunk.get_sky_light_srcs(self, &heights, &mut srcs);
        }
        propagate_sky_fast(self, &srcs);

        let time = start.elapsed().as_millis();
        eprintln!("Took {time} ms to init sky light");
    }

    fn update_sky_light(&mut self, positions: &[(i32, i32, i32)]) -> ChunkList {
        let mut updated_map_vals = HashMap::<(i32, i32), i32>::new();
        
        for (x, y, z) in positions.iter().copied() {
            let height = self.get_skylightmap(x, z).unwrap_or(i32::MIN);
            let block = self.get_block(x, y, z);
            if y > height && !skylight_can_pass(block) {
                let updated_y = updated_map_vals.get(&(x, z)).unwrap_or(&i32::MIN);
                updated_map_vals.insert((x, z), (*updated_y).max(y));
            } 
        }

        //Columns with no blocks 
        let mut no_blocks = HashSet::<(i32, i32)>::new();
        //Check if any blocks have been removed and get the new highest block
        //that is likely lower down
        for (x, y, z) in positions.iter().copied() {
            let height = self.get_skylightmap(x, z).unwrap_or(i32::MIN);
            let block = self.get_block(x, y, z); 
            if updated_map_vals.contains_key(&(x, z)) {
                continue;
            }
            //Check if the highest block has been removed and then
            //attempt to obtain the next highest block
            if y == height && skylight_can_pass(block) {
                let (chunkx, _, chunkz) = world_to_chunk_position(x, y, z);
                let mut chunks: Vec<(i32, i32, i32)> = self.chunks.keys()
                    .filter(|(px, _, pz)| *px == chunkx && *pz == chunkz)
                    .copied()
                    .collect();
                chunks.sort_by(|(_, y1, _), (_, y2, _)| y2.cmp(y1));
                for (px, py, pz) in chunks {
                    if let Some(chunk) = self.chunks.get(&(px, py, pz)) { 
                        let tallest = chunk.get_tallest_sky_block(x, z);
                        if let Some(tallest) = tallest {
                            updated_map_vals.insert((x, z), tallest);
                            break;
                        }
                    }
                }

                if !updated_map_vals.contains_key(&(x, z)) {
                    no_blocks.insert((x, z));
                }
            } 
        }

        //Update the position of the tallest blocks
        for (x, _, z) in positions.iter().copied() {
            if let Some(y) = updated_map_vals.get(&(x, z)) {
                self.set_skylightmap(x, z, Some(*y));
            } else if no_blocks.contains(&(x, z)) {
                self.set_skylightmap(x, z, None);
            }
        }

        propagate_sky_updates(self, positions)
    }

    //Updates block light upon a single block change
    //Returns a vector of chunks that need to be updated
    pub fn update_block_light(&mut self, positions: &[(i32, i32, i32)]) -> ChunkList {
        let blocks: Vec<(i32, i32, i32, Block)> = positions
            .iter()
            .map(|(x, y, z)| {
                let b = self.get_block(*x, *y, *z);
                (*x, *y, *z, b)
            })
            .collect();

        let mut updated = ChunkList::new();
        //Propagate light from any new light sources
        let srcs: Vec<((i32, i32, i32), LightSrc)> = blocks
            .iter()
            .map(|(x, y, z, b)| (*x, *y, *z, b.light_src()))
            .filter(|(_, _, _, src)| src.is_some())
            .map(|(x, y, z, src)| ((x, y, z), src.unwrap_or(LightSrc::new(0, 0, 0))))
            .collect();
        updated.extend(propagate(self, &srcs));

        //Update block light for other updated blocks that are not light sources
        let block_updates: Vec<(i32, i32, i32)> = blocks
            .iter()
            .filter(|(_, _, _, b)| b.light_src().is_none())
            .map(|(x, y, z, _)| (*x, *y, *z))
            .collect();
        updated.extend(propagate_updates(self, &block_updates));

        updated.extend(self.update_sky_light(positions));

        updated
    }

    //Takes in an option for the block position that was updated, if the Option
    //is None, then do nothing and return an empty list
    pub fn update_single_block_light(&mut self, pos: Option<(i32, i32, i32)>) -> ChunkList {
        if let Some(pos) = pos {
            self.update_block_light(&[pos])
        } else {
            ChunkList::new()
        }
    }

    //Takes in a list of newly loaded chunks and generates the light for those chunks
    pub fn init_light_new_chunks(&mut self, chunks: &HashSet<(i32, i32, i32)>) {
        let start = std::time::Instant::now();

        let mut srcs = vec![];
        //Generate new light in chunks
        let mut initialized = HashSet::<(i32, i32, i32)>::new();
        for (x, y, z) in chunks {
            if let Some(chunk) = self.chunks.get_mut(&(*x, *y, *z)) {
                //Clearing light likely is not necessary since we only need
                //to initialize light when the chunk has been just loaded
                //and any block updates to chunks neighboring it or in it
                //will allow for lighting in that chunk to be updated,
                //realistically blocks in the neighboring chunks can't be updated
                //(and thus can not have light updates in a way that affects
                //the chunk) due to the fact that if a neighboring chunk is
                //bordering a chunk that needs to be loaded, then that chunk
                //is likely too far away for block simulation/the player to
                //place/break blocks in it. Therefore, it should be that in
                //most cases any light updates that affect this chunk will happen
                //when the chunk is loaded and we can have the program lazily
                //assume that not much has changed and therefore we do not
                //need to clear the chunk light.
                //Hopefully this is correct in most scenarios.
                //chunk.clear_light();
                if chunk.light_initialized() {
                    continue;
                }
                initialized.insert((*x, *y, *z));
                chunk.get_light_srcs(&mut srcs);
            }
        }
        propagate_fast(self, &srcs);

        let mut neighbor_srcs = HashMap::new();
        for (x, y, z) in chunks {
            if let Some(chunk) = self.chunks.get(&(*x, *y, *z)) {
                if !initialized.contains(&(*x, *y, *z)) {
                    continue;
                }
                get_neighbor_srcs(chunk, &self.get_adjacent(chunk), &mut neighbor_srcs, chunks);
            }
        }
        let neighbor_srcs: Vec<((i32, i32, i32), LightSrc)> = neighbor_srcs
            .iter()
            .map(|((x, y, z), light)| {
                ((*x, *y, *z), LightSrc::new(light.r(), light.g(), light.b()))
            })
            .collect();
        propagate_fast(self, &neighbor_srcs);

        //(for debugging purposes)
        //Uncomment the following if you want to verify if the light generated is correct
        //eprintln!("Validating new chunks");
        //validate_light(self, chunks);

        let time = start.elapsed().as_millis();
        eprintln!("Took {time} ms to init light in new chunks");
    }
}
