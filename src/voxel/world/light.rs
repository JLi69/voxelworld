use super::World;
use crate::voxel::{
    light::{Light, LightSrc, LU},
    Block, EMPTY_BLOCK,
};
use std::collections::{HashSet, VecDeque};

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

fn propagate_channel(
    queue: &mut VecDeque<Propagation>,
    world: &mut World,
    channel: fn(Light) -> u16,
    update: fn(u16) -> LU,
) {
    let mut visited = HashSet::<(i32, i32, i32, u16)>::new();
    while !queue.is_empty() {
        let top = queue.pop_front();
        if let Some((x, y, z, val)) = top {
            if visited.contains(&(x, y, z, val)) {
                continue;
            }
            let light = world.get_light(x, y, z);
            if channel(light) >= val {
                continue;
            }
            let lu = update(val);
            visited.insert((x, y, z, val));
            world.update_light(x, y, z, lu);

            if val <= 1 {
                continue;
            }

            for (dx, dy, dz) in ADJ {
                if visited.contains(&(x + dx, y + dy, z + dz, val - 1)) {
                    continue;
                }
                let block = world.get_block(x + dx, y + dy, z + dz);
                if !light_can_pass(block) {
                    visited.insert((x, y, z, val));
                    continue;
                }
                let adj_light = world.get_light(x + dx, y + dy, z + dz);
                if channel(adj_light) >= val - 1 {
                    visited.insert((x + dx, y + dy, z + dz, val - 1));
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
