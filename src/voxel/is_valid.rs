use super::{World, EMPTY_BLOCK, orientation_to_normal};

fn check_below_valid(world: &World, x: i32, y: i32, z: i32, valid_blocks: &[u8]) -> bool {
    let below = world.get_block(x, y - 1, z);
    if below.shape() != 0 {
        return false;
    }
    for block in valid_blocks {
        if below.id == *block {
            return true;
        }
    }
    false
}

fn check_torch_valid(world: &World, x: i32, y: i32, z: i32) -> bool {
    let block = world.get_block(x, y, z);
    let dir = orientation_to_normal(block.orientation());
    let adj = world.get_block(x - dir.x, y - dir.y, z - dir.z);
    return !(adj.id == EMPTY_BLOCK || adj.transparent() || adj.shape() != 0);
}

type ValidBlockFn = fn(&World, i32, i32, i32) -> bool;

//Returns a function that checks if a block in a position (x, y, z) is valid
//If None is returned, then we assume that this block
pub fn get_check_valid_fn(block: u8) -> Option<ValidBlockFn> {
    match block {
        //Sapling, Grass, and Flowers
        47 | 49 | 54..=56 => Some(|world, x, y, z| check_below_valid(world, x, y, z, &[1, 4, 17])),
        //Mushroom
        48 => Some(|world, x, y, z| {
            let below = world.get_block(x, y - 1, z);
            if below.shape() != 0 {
                return false;
            }
            !below.transparent() && below.id != EMPTY_BLOCK
        }),
        //Wheat
        50..=53 => Some(|world, x, y, z| check_below_valid(world, x, y, z, &[43, 45])),
        //Sugar cane
        69 => Some(|world, x, y, z| check_below_valid(world, x, y, z, &[1, 4, 11, 17, 69])),
        //Torches
        71..=74 => Some(|world, x, y, z| check_torch_valid(world, x, y, z)),
        _ => None,
    }
}
