use super::{Block, World, EMPTY_BLOCK};
use cgmath::{InnerSpace, Vector3};

pub const BLOCK_REACH: f32 = 5.0;

/*
 * These are functions related to placing/building in the world
 * */

//TODO: replace with better code that is more accurate
pub fn raycast(pos: Vector3<f32>, dir: Vector3<f32>, range: f32, world: &World) -> (f32, f32, f32) {
    let mut x = pos.x.floor() as i32;
    let mut y = pos.y.floor() as i32;
    let mut z = pos.z.floor() as i32;

    let start_pos = pos;
    let mut current_pos = start_pos;
    while world.get_block(x, y, z).id == EMPTY_BLOCK
        && (current_pos - start_pos).magnitude() < range
    {
        current_pos += dir * 0.05;
        x = current_pos.x.floor() as i32;
        y = current_pos.y.floor() as i32;
        z = current_pos.z.floor() as i32;
    }

    (current_pos.x, current_pos.y, current_pos.z)
}

//Returns the (x, y, z) coordinate of the block destroyed as an option
//Returns none if no block destroyed
pub fn destroy_block(
    pos: Vector3<f32>,
    dir: Vector3<f32>,
    world: &mut World,
) -> Option<(i32, i32, i32)> {
    let (x, y, z) = raycast(pos, dir, BLOCK_REACH, world);
    let (ix, iy, iz) = (x.floor() as i32, y.floor() as i32, z.floor() as i32);
    let blockid = world.get_block(ix, iy, iz).id;
    world.set_block(ix, iy, iz, Block::new_id(0));
    if blockid != EMPTY_BLOCK {
        return Some((ix, iy, iz));
    }

    None
}

//Returns the (x, y, z) coordinate of the block placed as an option
//Returns none if no block is placed
pub fn place_block(
    pos: Vector3<f32>,
    dir: Vector3<f32>,
    world: &mut World,
) -> Option<(i32, i32, i32)> {
    //TODO: add code to check for collision with player to make sure we
    //don't place blocks on top of the player
    let (mut x, mut y, mut z) = raycast(pos, dir, BLOCK_REACH, world);
    let blockid1 = {
        let (ix, iy, iz) = (x.floor() as i32, y.floor() as i32, z.floor() as i32);
        world.get_block(ix, iy, iz).id
    };
    x -= 0.2 * dir.x;
    y -= 0.2 * dir.y;
    z -= 0.2 * dir.z;
    let (ix, iy, iz) = (x.floor() as i32, y.floor() as i32, z.floor() as i32);
    let blockid2 = world.get_block(ix, iy, iz).id;
    if blockid2 == EMPTY_BLOCK && blockid1 != EMPTY_BLOCK {
        world.set_block(ix, iy, iz, Block::new_id(1));
        return Some((ix, iy, iz));
    }

    None
}
