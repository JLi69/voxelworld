use super::{World, EMPTY_BLOCK};

fn check_below_valid(world: &World, x: i32, y: i32, z: i32, valid_blocks: &[u8]) -> bool {
    let below = world.get_block(x, y - 1, z);
    for block in valid_blocks {
        if below.id == *block {
            return true;
        }
    }
    false
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
            !below.transparent() && below.id != EMPTY_BLOCK
        }),
        //Wheat
        50..=53 => Some(|world, x, y, z| check_below_valid(world, x, y, z, &[43, 45])),
        //Sugar cane
        69 => Some(|world, x, y, z| check_below_valid(world, x, y, z, &[1, 4, 11, 17, 69])),
        _ => None,
    }
}
