use super::is_valid::get_check_valid_fn;
use super::{Axis, INDESTRUCTIBLE};
use super::{Block, World, EMPTY_BLOCK};
use crate::game::inventory::Item;
use crate::game::physics::Hitbox;
use crate::game::player::Player;
use cgmath::{InnerSpace, Vector3};

pub const BLOCK_REACH: f32 = 4.0;

/*
 * These are functions related to placing/building in the world
 * */

//Converts a floating point coordinate to an integer voxel coordinate
fn convert_coord_to_voxel(x: f32, d: f32) -> i32 {
    if d < 0.0 {
        x.floor() as i32 - 1
    } else {
        x.floor() as i32
    }
}

//Floor of a float but returned as an integer
fn floori(x: f32) -> i32 {
    x.floor() as i32
}

//Converts raycast data to integer voxel coordinates
pub fn get_raycast_voxel(x: f32, y: f32, z: f32, dir: Vector3<f32>, axis: Axis) -> (i32, i32, i32) {
    match axis {
        Axis::X => (convert_coord_to_voxel(x, dir.x), floori(y), floori(z)),
        Axis::Y => (floori(x), convert_coord_to_voxel(y, dir.y), floori(z)),
        Axis::Z => (floori(x), floori(y), convert_coord_to_voxel(z, dir.z)),
    }
}

//Scan to see if we hit a voxel in the x direction
fn scan_x(pos: Vector3<f32>, dir: Vector3<f32>, range: f32, world: &World) -> Vector3<f32> {
    if dir.x == 0.0 {
        return pos + dir * range;
    }

    let startx = if dir.x < 0.0 {
        pos.x.floor()
    } else {
        pos.x.ceil()
    };

    let mut current_pos = Vector3::new(pos.x, pos.y, pos.z);
    let mut diff = dir * (1.0 / dir.x).abs();
    diff.x = diff.x.signum();
    current_pos += diff * (startx - pos.x).abs();

    let mut x = convert_coord_to_voxel(current_pos.x, dir.x);
    let mut y = current_pos.y.floor() as i32;
    let mut z = current_pos.z.floor() as i32;
    while (current_pos - pos).magnitude() < range
        && (world.get_block(x, y, z).id == EMPTY_BLOCK || world.get_block(x, y, z).is_fluid())
    {
        current_pos += diff;
        x = convert_coord_to_voxel(current_pos.x, dir.x);
        y = current_pos.y.floor() as i32;
        z = current_pos.z.floor() as i32;
    }

    current_pos
}

//Scan to see if we hit a voxel in the y direction
fn scan_y(pos: Vector3<f32>, dir: Vector3<f32>, range: f32, world: &World) -> Vector3<f32> {
    if dir.y == 0.0 {
        return pos + dir * range;
    }

    let starty = if dir.y < 0.0 {
        pos.y.floor()
    } else {
        pos.y.ceil()
    };

    let mut current_pos = Vector3::new(pos.x, pos.y, pos.z);
    let mut diff = dir * (1.0 / dir.y).abs();
    diff.y = diff.y.signum();
    current_pos += diff * (starty - pos.y).abs();

    let mut x = current_pos.x.floor() as i32;
    let mut y = convert_coord_to_voxel(current_pos.y, dir.y);
    let mut z = current_pos.z.floor() as i32;
    while (current_pos - pos).magnitude() < range
        && (world.get_block(x, y, z).id == EMPTY_BLOCK || world.get_block(x, y, z).is_fluid())
    {
        current_pos += diff;
        x = current_pos.x.floor() as i32;
        y = convert_coord_to_voxel(current_pos.y, dir.y);
        z = current_pos.z.floor() as i32;
    }

    current_pos
}

//Scan to see if we hit a voxel in the z direction
fn scan_z(pos: Vector3<f32>, dir: Vector3<f32>, range: f32, world: &World) -> Vector3<f32> {
    if dir.z == 0.0 {
        return pos + dir * range;
    }

    let startz = if dir.z < 0.0 {
        pos.z.floor()
    } else {
        pos.z.ceil()
    };

    let mut current_pos = Vector3::new(pos.x, pos.y, pos.z);
    let mut diff = dir * (1.0 / dir.z).abs();
    diff.z = diff.z.signum();
    current_pos += diff * (startz - pos.z).abs();

    let mut x = current_pos.x.floor() as i32;
    let mut y = current_pos.y.floor() as i32;
    let mut z = convert_coord_to_voxel(current_pos.z, dir.z);
    while (current_pos - pos).magnitude() < range
        && (world.get_block(x, y, z).id == EMPTY_BLOCK || world.get_block(x, y, z).is_fluid())
    {
        current_pos += diff;
        x = current_pos.x.floor() as i32;
        y = current_pos.y.floor() as i32;
        z = convert_coord_to_voxel(current_pos.z, dir.z);
    }

    current_pos
}

//Cast a ray and return a position where we hit a voxel
pub fn raycast(
    pos: Vector3<f32>,
    dir: Vector3<f32>,
    range: f32,
    world: &World,
) -> (f32, f32, f32, Axis) {
    let hitx = scan_x(pos, dir, range, world);
    let hity = scan_y(pos, dir, range, world);
    let hitz = scan_z(pos, dir, range, world);

    let lengthx = (hitx - pos).magnitude();
    let lengthy = (hity - pos).magnitude();
    let lengthz = (hitz - pos).magnitude();

    if lengthx <= lengthy && lengthx <= lengthz {
        (hitx.x, hitx.y, hitx.z, Axis::X)
    } else if lengthy <= lengthx && lengthy <= lengthz {
        (hity.x, hity.y, hity.z, Axis::Y)
    } else if lengthz <= lengthy && lengthz <= lengthx {
        (hitz.x, hitz.y, hitz.z, Axis::Z)
    } else {
        (hitx.x, hitx.y, hitx.z, Axis::X)
    }
}

//Returns the (x, y, z) coordinate of the block destroyed as an option
//Returns none if no block destroyed
pub fn destroy_block(
    pos: Vector3<f32>,
    dir: Vector3<f32>,
    world: &mut World,
) -> Option<(i32, i32, i32)> {
    let (x, y, z, axis) = raycast(pos, dir, BLOCK_REACH, world);
    let (ix, iy, iz) = get_raycast_voxel(x, y, z, dir, axis);
    let block = world.get_block(ix, iy, iz);
    if block.id == INDESTRUCTIBLE {
        return None;
    }

    //You cannot destroy fluids
    if block.is_fluid() {
        return None;
    }

    world.set_block(ix, iy, iz, Block::new_id(0));
    if block.id != EMPTY_BLOCK {
        return Some((ix, iy, iz));
    }

    None
}

//If the player is suffocating (their head is trapped in a block)
//then they can only destroy the block that they are trapped in
pub fn destroy_block_suffocating(pos: Vector3<f32>, world: &mut World) -> Option<(i32, i32, i32)> {
    let (x, y, z) = (pos.x.floor(), pos.y.floor(), pos.z.floor());
    let (ix, iy, iz) = (x as i32, y as i32, z as i32);
    let block = world.get_block(ix, iy, iz);
    if block.id == INDESTRUCTIBLE {
        return None;
    }

    //You cannot destroy fluids
    if block.is_fluid() {
        return None;
    }

    world.set_block(ix, iy, iz, Block::new_id(0));
    if block.id != EMPTY_BLOCK {
        return Some((ix, iy, iz));
    }

    None
}

fn set_block_rotation(dir: Vector3<f32>, block: &mut Block) {
    if !block.can_rotate() {
        return;
    }

    if dir.y.abs() > dir.z.abs() && dir.y.abs() > dir.x.abs() && !block.rotate_y_only() {
        //Set orientation of the block
        if dir.y.signum() as i32 == -1 && block.can_rotate() {
            block.set_orientation(3);
        } else if dir.y.signum() as i32 == 1 && block.can_rotate() {
            block.set_orientation(0);
        }
    } else if dir.z.abs() >= dir.x.abs() {
        //Set orientation of the block
        if dir.z.signum() as i32 == -1 && block.can_rotate() {
            block.set_orientation(5);
        } else if dir.z.signum() as i32 == 1 && block.can_rotate() {
            block.set_orientation(2);
        }
    } else if dir.x.abs() > dir.z.abs() {
        //Set orientation of the block
        if dir.x.signum() as i32 == -1 && block.can_rotate() {
            block.set_orientation(4);
        } else if dir.x.signum() as i32 == 1 && block.can_rotate() {
            block.set_orientation(1);
        }
    }
}

//Returns the (x, y, z) coordinate of the block placed as an option
//Returns none if no block is placed
pub fn place_block(
    pos: Vector3<f32>,
    dir: Vector3<f32>,
    world: &mut World,
    player: &Player,
) -> Option<(i32, i32, i32)> {
    let (x, y, z, axis) = raycast(pos, dir, BLOCK_REACH, world);
    let blockid = {
        let (ix, iy, iz) = get_raycast_voxel(x, y, z, dir, axis);
        let block = world.get_block(ix, iy, iz);
        if block.is_fluid() {
            EMPTY_BLOCK
        } else {
            block.id
        }
    };
    let (mut ix, mut iy, mut iz) = get_raycast_voxel(x, y, z, dir, axis);

    let mut block;
    if let Item::BlockItem(blockdata, _) = player.hotbar.get_selected() {
        block = blockdata;
        if block.is_fluid() {
            block.geometry = 7;
        }
    } else {
        return None;
    }

    match axis {
        Axis::X => ix -= dir.x.signum() as i32,
        Axis::Y => iy -= dir.y.signum() as i32,
        Axis::Z => iz -= dir.z.signum() as i32,
    }

    set_block_rotation(dir, &mut block);

    let replace = world.get_block(ix, iy, iz); //Block that is being replaced

    if replace.is_fluid() && block.fluid_destructibe() {
        return None;
    }

    if (replace.id == EMPTY_BLOCK || replace.is_fluid()) && blockid != EMPTY_BLOCK {
        if let Some(check_valid) = get_check_valid_fn(block.id) {
            if !check_valid(world, ix, iy, iz) {
                return None;
            }
        }
        let prev_block = world.get_block(ix, iy, iz);
        world.set_block(ix, iy, iz, block);
        let block_hitbox = Hitbox::from_block(ix, iy, iz);
        if player.get_hitbox().intersects(&block_hitbox) && !block.no_hitbox() {
            world.set_block(ix, iy, iz, prev_block);
            return None;
        }
        return Some((ix, iy, iz));
    }

    None
}
