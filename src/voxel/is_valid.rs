use super::{orientation_to_normal, World, EMPTY_BLOCK};

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
    //Torches can be placed on top of fences
    if dir.y != 0 && adj.id == 76 {
        return true;
    }
    !(adj.id == EMPTY_BLOCK || adj.transparent() || adj.shape() != 0)
}

fn check_door_valid(world: &World, x: i32, y: i32, z: i32) -> bool {
    let below = world.get_block(x, y - 1, z);
    !(below.id == EMPTY_BLOCK || below.is_fluid() || below.shape() != 0)
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
        50..=53 | 77 => Some(|world, x, y, z| check_below_valid(world, x, y, z, &[43, 45])),
        //Sugar cane
        69 => Some(|world, x, y, z| check_below_valid(world, x, y, z, &[1, 4, 11, 17, 69])),
        //Torches and ladders
        71..=75 => Some(check_torch_valid),
        //Door
        79 => Some(check_door_valid),
        _ => None,
    }
}
