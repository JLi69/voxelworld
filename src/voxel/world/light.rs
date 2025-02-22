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
    visited: &mut HashSet<(i32, i32, i32)>,
    world: &mut World,
    channel: fn(Light) -> u16,
    update: fn(u16) -> LU,
) {
    while !queue.is_empty() {
        let top = queue.pop_front();
        if let Some((x, y, z, val)) = top {
            let light = world.get_light(x, y, z);
            if channel(light) >= val {
                continue;
            }
            let lu = update(val);
            visited.insert((x, y, z));
            world.update_light(x, y, z, lu);

            if val <= 1 {
                continue;
            }

            for (dx, dy, dz) in ADJ {
                if visited.contains(&(x + dx, y + dy, z + dz)) {
                    continue;
                }
                let block = world.get_block(x + dx, y + dy, z + dz);
                if !light_can_pass(block) {
                    visited.insert((x + dx, y + dy, z + dz));
                    continue;
                }
                let adj_light = world.get_light(x + dx, y + dy, z + dz);
                if channel(adj_light) >= val - 1 {
                    visited.insert((x + dx, y + dy, z + dz));
                    continue;
                }
                visited.insert((x + dx, y + dy, z + dz));
                queue.push_back((x + dx, y + dy, z + dz, val - 1));
            }
        }
    }
}

//Propagate a light source
pub fn propagate(world: &mut World, x: i32, y: i32, z: i32, src: LightSrc) {
    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();
    //Propagate red
    queue.push_back((x, y, z, src.r));
    propagate_channel(
        &mut queue,
        &mut visited,
        world,
        |light| light.r(),
        |v| LU::new(None, Some(v), None, None),
    );
    visited.clear();
    //Propagate green
    queue.push_back((x, y, z, src.g));
    propagate_channel(
        &mut queue,
        &mut visited,
        world,
        |light| light.g(),
        |v| LU::new(None, None, Some(v), None),
    );
    visited.clear();
    //Propagate blue
    queue.push_back((x, y, z, src.b));
    propagate_channel(
        &mut queue,
        &mut visited,
        world,
        |light| light.b(),
        |v| LU::new(None, None, None, Some(v)),
    );
    visited.clear();
}

impl World {
    //Called when the world is first loaded
    pub fn init_block_light(&mut self) {
        let start = std::time::Instant::now();

        let mut srcs = vec![];
        for chunk in self.chunks.values() {
            chunk.get_light_srcs(&mut srcs);
        }

        for ((x, y, z), src) in srcs {
            propagate(self, x, y, z, src);
        }

        let time = start.elapsed().as_millis();
        eprintln!("Took {time} ms to init light");
    }
}
