use super::{block_update::get_chunktable_updates, World};
use crate::voxel::{
    light::{Light, LightSrc, LU},
    Block, Chunk, CHUNK_SIZE_I32, EMPTY_BLOCK,
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
        for (x, y, z) in chunks {
            if let Some(chunk) = self.chunks.get_mut(&(*x, *y, *z)) {
                chunk.clear_light();
                chunk.get_light_srcs(&mut srcs);
            }
        }
        propagate_fast(self, &srcs);

        let mut neighbor_srcs = HashMap::new();
        for (x, y, z) in chunks {
            if let Some(chunk) = self.chunks.get(&(*x, *y, *z)) {
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
