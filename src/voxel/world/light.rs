use super::World;
use crate::voxel::{
    light::{Light, LightSrc, LU},
    Block, EMPTY_BLOCK,
};
use std::collections::{HashMap, VecDeque};

const ADJ: [(i32, i32, i32); 6] = [
    (-1, 0, 0),
    (1, 0, 0),
    (0, -1, 0),
    (0, 1, 0),
    (0, 0, -1),
    (0, 0, 1),
];

type Propagation = (i32, i32, i32, u16);

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
) {
    let mut visited = HashMap::<(i32, i32, i32), u16>::new();
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
}

//Propagate a light source
pub fn propagate(world: &mut World, srcs: &[((i32, i32, i32), LightSrc)]) {
    let mut queue = VecDeque::new();
    //Propagate red
    for ((x, y, z), src) in srcs {
        queue.push_back((*x, *y, *z, src.r));
    }
    propagate_channel(
        &mut queue,
        world,
        |light| light.r(),
        |v| LU::new(None, Some(v), None, None),
    );
    //Propagate green
    for ((x, y, z), src) in srcs {
        queue.push_back((*x, *y, *z, src.g));
    }
    propagate_channel(
        &mut queue,
        world,
        |light| light.g(),
        |v| LU::new(None, None, Some(v), None),
    );
    //Propagate blue
    for ((x, y, z), src) in srcs {
        queue.push_back((*x, *y, *z, src.b));
    }
    propagate_channel(
        &mut queue,
        world,
        |light| light.b(),
        |v| LU::new(None, None, None, Some(v)),
    );
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
}
