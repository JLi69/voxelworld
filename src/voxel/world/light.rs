use super::{World, block_update::get_chunktable_updates};
use crate::voxel::{
    light::{Light, LightSrc, LU},
    Block, EMPTY_BLOCK,
};
use std::collections::{HashMap, VecDeque, HashSet};

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
            let lu = update(val);
            add_visited(&mut visited, (x, y, z), val);
            get_chunktable_updates(x, y, z, &mut updated);
            world.update_light(x, y, z, lu);

            if val <= 1 {
                continue;
            }

            for (dx, dy, dz) in ADJ {
                let adj = (x + dx, y + dy, z + dz);
                if check_visited(&visited, adj, val - 1) {
                    continue;
                }
                let block = world.get_block(x + dx, y + dy, z + dz);
                if !light_can_pass(block) {
                    add_visited(&mut visited, adj, 15);
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

fn attenuate(channel: u16) -> u16 {
    if channel == 0 {
        0
    } else {
        channel - 1
    }
}

pub fn get_new_light(world: &World, x: i32, y: i32, z: i32, block: Block) -> Light {
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
            let val = channel(get_new_light(world, x, y, z, block));
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

impl World {
    //Called when the world is first loaded
    pub fn init_block_light(&mut self) {
        let start = std::time::Instant::now();

        let mut srcs = vec![];
        for chunk in self.chunks.values() {
            chunk.get_light_srcs(&mut srcs);
        }
        propagate(self, &srcs);

        let time = start.elapsed().as_millis();
        eprintln!("Took {time} ms to init light");
    }

    //Updates block light upon a single block change
    //Returns a vector of chunks that need to be updated
    pub fn update_block_light(&mut self, positions: &[(i32, i32, i32)]) -> ChunkList {
        let blocks: Vec<(i32, i32, i32, Block)> = positions.iter().map(|(x, y, z)| {
            let b = self.get_block(*x, *y, *z);
            (*x, *y, *z, b)
        }).collect();

        let mut updated = ChunkList::new();
        //Propagate light from any new light sources
        let srcs: Vec<((i32, i32, i32), LightSrc)> = blocks.iter().map(|(x, y, z, b)| {
            (*x, *y, *z, b.light_src())
        }).filter(|(_, _, _, src)| {
            src.is_some()
        }).map(|(x, y, z, src)| {
            ((x, y, z), src.unwrap_or(LightSrc::new(0, 0, 0)))
        }).collect();
        updated.extend(propagate(self, &srcs));

        //Update block light for other updated blocks that are not light sources
        let block_updates: Vec<(i32, i32, i32)> = blocks.iter()
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
}
