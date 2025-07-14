use super::coordinates::f32coord_to_int;
use super::is_valid::get_check_valid_fn;
use super::{Axis, FULL_BLOCK, INDESTRUCTIBLE};
use super::{Block, World, EMPTY_BLOCK};
use crate::game::inventory::Item;
use crate::game::physics::{composite_to_hitbox, ray_intersects_box, CompositeHitbox, Hitbox};
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

fn ray_intersects_block(
    pos: Vector3<f32>,
    dir: Vector3<f32>,
    x: i32,
    y: i32,
    z: i32,
    world: &World,
    ignore: fn(Block) -> bool,
) -> bool {
    let block = world.get_block(x, y, z);
    let hit = match Hitbox::from_block_data(x, y, z, block) {
        CompositeHitbox::Single(_) => {
            let bbox = Hitbox::from_block_bbox(x, y, z, block);
            ray_intersects_box(pos, dir, &bbox)
        }
        CompositeHitbox::Double(b1, b2) => {
            ray_intersects_box(pos, dir, &b1) || ray_intersects_box(pos, dir, &b2)
        }
        CompositeHitbox::Triple(b1, b2, b3) => {
            ray_intersects_box(pos, dir, &b1)
                || ray_intersects_box(pos, dir, &b2)
                || ray_intersects_box(pos, dir, &b3)
        }
    };

    if !hit {
        return false;
    }

    !ignore(block)
}

//Scan to see if we hit a voxel in the x direction
fn scan_x(
    pos: Vector3<f32>,
    dir: Vector3<f32>,
    range: f32,
    world: &World,
    ignore: fn(Block) -> bool,
) -> Vector3<f32> {
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
        && !ray_intersects_block(pos, dir, x, y, z, world, ignore)
    {
        current_pos += diff;
        x = convert_coord_to_voxel(current_pos.x, dir.x);
        y = current_pos.y.floor() as i32;
        z = current_pos.z.floor() as i32;
    }

    current_pos
}

//Scan to see if we hit a voxel in the y direction
fn scan_y(
    pos: Vector3<f32>,
    dir: Vector3<f32>,
    range: f32,
    world: &World,
    ignore: fn(Block) -> bool,
) -> Vector3<f32> {
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
        && !ray_intersects_block(pos, dir, x, y, z, world, ignore)
    {
        current_pos += diff;
        x = current_pos.x.floor() as i32;
        y = convert_coord_to_voxel(current_pos.y, dir.y);
        z = current_pos.z.floor() as i32;
    }

    current_pos
}

//Scan to see if we hit a voxel in the z direction
fn scan_z(
    pos: Vector3<f32>,
    dir: Vector3<f32>,
    range: f32,
    world: &World,
    ignore: fn(Block) -> bool,
) -> Vector3<f32> {
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
        && !ray_intersects_block(pos, dir, x, y, z, world, ignore)
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
    ignore: fn(Block) -> bool,
) -> (f32, f32, f32, Axis) {
    let hitx = scan_x(pos, dir, range, world, ignore);
    let hity = scan_y(pos, dir, range, world, ignore);
    let hitz = scan_z(pos, dir, range, world, ignore);

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

pub fn get_selected(pos: Vector3<f32>, dir: Vector3<f32>, world: &World) -> (i32, i32, i32) {
    let (posx, posy, posz) = f32coord_to_int(pos.x, pos.y, pos.z);
    let ignore = |block: Block| block.is_fluid() || block.id == EMPTY_BLOCK;
    let (x, y, z, axis) = raycast(pos, dir, BLOCK_REACH, world, ignore);
    if ray_intersects_block(pos, dir, posx, posy, posz, world, ignore) {
        (posx, posy, posz)
    } else {
        get_raycast_voxel(x, y, z, dir, axis)
    }
}

pub fn get_selected_fluid(pos: Vector3<f32>, dir: Vector3<f32>, world: &World) -> (i32, i32, i32) {
    let (posx, posy, posz) = f32coord_to_int(pos.x, pos.y, pos.z);
    let ignore = |block: Block| (block.is_fluid() && block.geometry < 7) || block.id == EMPTY_BLOCK;
    let (x, y, z, axis) = raycast(pos, dir, BLOCK_REACH, world, ignore);
    if ray_intersects_block(pos, dir, posx, posy, posz, world, ignore) {
        (posx, posy, posz)
    } else {
        get_raycast_voxel(x, y, z, dir, axis)
    }
}

//Returns the (x, y, z) coordinate of the block destroyed as an option
//Returns none if no block destroyed
pub fn destroy_block(
    pos: Vector3<f32>,
    dir: Vector3<f32>,
    world: &mut World,
) -> Option<(i32, i32, i32)> {
    let (ix, iy, iz) = get_selected(pos, dir, world);
    let block = world.get_block(ix, iy, iz);
    if block.id == INDESTRUCTIBLE {
        return None;
    }

    //You cannot destroy fluids
    if block.is_fluid() {
        return None;
    }

    match block.id {
        //Door
        79 => {
            if world.get_block(ix, iy + 1, iz).id == 81 {
                world.set_block(ix, iy + 1, iz, Block::new_id(0));
            }
        }
        81 => {
            if world.get_block(ix, iy - 1, iz).id == 79 {
                world.set_block(ix, iy - 1, iz, Block::new_id(0));
            }
        }
        _ => {}
    }
    world.set_block(ix, iy, iz, Block::new_id(0));
    if block.id != EMPTY_BLOCK {
        return Some((ix, iy, iz));
    }

    None
}

//If the player is suffocating (their head is trapped in a block)
//then they can only destroy the block that they are trapped in
pub fn destroy_block_suffocating(
    stuck: Option<(i32, i32, i32)>,
    world: &mut World,
) -> Option<(i32, i32, i32)> {
    if let Some((ix, iy, iz)) = stuck {
        let block = world.get_block(ix, iy, iz);
        if block.id == INDESTRUCTIBLE {
            return None;
        }

        //You cannot destroy fluids
        if block.is_fluid() {
            return None;
        }

        match block.id {
            //Door
            79 => {
                if world.get_block(ix, iy + 1, iz).id == 81 {
                    world.set_block(ix, iy + 1, iz, Block::new_id(0));
                }
            }
            81 => {
                if world.get_block(ix, iy - 1, iz).id == 79 {
                    world.set_block(ix, iy - 1, iz, Block::new_id(0));
                }
            }
            _ => {}
        }
        world.set_block(ix, iy, iz, Block::new_id(0));
        if block.id != EMPTY_BLOCK {
            return Some((ix, iy, iz));
        }
    }

    None
}

fn set_block_rotation(dir: Vector3<f32>, block: &mut Block) {
    if !block.can_rotate() {
        return;
    }

    if block.shape() != 0 {
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

//.fract() returns negative fractional parts for negative numbers, this
//function will always return a positive number
fn fraction(x: f32) -> f32 {
    if x < 0.0 {
        x.fract() + 1.0
    } else {
        x.fract()
    }
}

fn set_slab_orientation(x: f32, y: f32, z: f32, dir: Vector3<f32>, axis: Axis, block: &mut Block) {
    if block.shape() != 1 {
        return;
    }

    //Horizontal slabs
    if block.orientation() == 0 {
        if fraction(y) < 0.5 {
            block.set_orientation(0);
        } else {
            block.set_orientation(3);
        }

        if dir.y > 0.0 && (y.abs().fract() < 0.01 || y.abs().fract() > 0.99) {
            block.set_orientation(3);
        } else if dir.y < 0.0 && (y.abs().fract() < 0.01 || y.abs().fract() > 0.99) {
            block.set_orientation(0);
        }
    } else {
        //Vertical slabs
        match axis {
            Axis::X => {
                if dir.x <= 0.0 {
                    block.set_orientation(1);
                } else if dir.x > 0.0 {
                    block.set_orientation(4);
                }
            }
            Axis::Y => {
                let orientation = if (fraction(1.0 - x) > fraction(z)
                    && fraction(1.0 - x) > fraction(1.0 - z))
                    || (fraction(x) > fraction(z) && fraction(x) > fraction(1.0 - z))
                {
                    if fraction(x) < 0.5 {
                        1
                    } else {
                        4
                    }
                } else if fraction(z) < 0.5 {
                    2
                } else {
                    5
                };
                block.set_orientation(orientation);
            }
            Axis::Z => {
                if dir.z <= 0.0 {
                    block.set_orientation(2);
                } else if dir.z > 0.0 {
                    block.set_orientation(5);
                }
            }
        }
    }
}

fn set_stair_rotation(dir: Vector3<f32>, block: &mut Block) {
    if !matches!(block.shape(), 2..=4) {
        return;
    }

    if dir.z.abs() >= dir.x.abs() {
        //Set orientation of the block
        if dir.z.signum() as i32 == -1 {
            block.set_orientation(2);
        } else if dir.z.signum() as i32 == 1 {
            block.set_orientation(5);
        }
    } else if dir.x.abs() > dir.z.abs() {
        //Set orientation of the block
        if dir.x.signum() as i32 == -1 {
            block.set_orientation(1);
        } else if dir.x.signum() as i32 == 1 {
            block.set_orientation(4);
        }
    }

    //Reflect stairs
    if dir.y > 0.0 {
        block.set_reflection(1);
    }
}

fn set_torch_orientation(dir: Vector3<f32>, axis: Axis) -> u8 {
    match axis {
        Axis::X => {
            if dir.x >= 0.0 {
                4
            } else {
                1
            }
        }
        Axis::Y => {
            if dir.y >= 0.0 {
                3
            } else {
                0
            }
        }
        Axis::Z => {
            if dir.z >= 0.0 {
                5
            } else {
                2
            }
        }
    }
}

fn set_non_voxel_orientation(dir: Vector3<f32>, axis: Axis, block: &mut Block) {
    if !block.non_voxel_geometry() {
        return;
    }

    let orientation = match block.id {
        //Torches and ladder
        71..=75 => set_torch_orientation(dir, axis),
        _ => block.orientation(),
    };
    block.set_orientation(orientation);
}

fn place(
    world: &mut World,
    player: &Player,
    ix: i32,
    iy: i32,
    iz: i32,
    block: Block,
) -> Option<(i32, i32, i32)> {
    let prev_block = world.get_block(ix, iy, iz);
    world.set_block(ix, iy, iz, block);
    if let Some(check_valid) = get_check_valid_fn(block.id) {
        if !check_valid(world, ix, iy, iz) {
            world.set_block(ix, iy, iz, prev_block);
            return None;
        }
    }
    let composite_hitbox = Hitbox::from_block_data(ix, iy, iz, block);
    let block_hitbox = composite_to_hitbox(composite_hitbox, &player.get_hitbox());
    if player.get_hitbox().intersects(&block_hitbox) && !block.no_hitbox() {
        world.set_block(ix, iy, iz, prev_block);
        return None;
    }
    Some((ix, iy, iz))
}

//Returns the (x, y, z) coordinate of the block placed as an option
//Returns none if no block is placed
pub fn interact_with_block(
    pos: Vector3<f32>,
    dir: Vector3<f32>,
    world: &mut World,
    player: &Player,
) -> Option<(i32, i32, i32)> {
    let (ix, iy, iz) = get_selected(pos, dir, world);

    let mut block;
    if let Item::Block(blockdata, _) = player.hotbar.get_selected() {
        block = blockdata;
        if block.is_fluid() {
            block.geometry = 7;
        }

        //To prevent leaf decay, we will set the orientation of leaves to be
        //1 if they are a full block and are placed by the player
        //This should not have any visual effect other than a way for the game
        //to differentiate between naturally generated leaves and 'artificial'
        //leaves. This is kind of a hack but I don't want to come up with
        //a better solution.
        if (block.id == 7 || block.id == 91) && block.geometry == 0 {
            block.set_orientation(1);
        }
    } else if let Item::Bucket(blockid) = player.hotbar.get_selected() {
        //Place fluid with a bucket
        if Block::new_id(blockid).is_fluid() {
            block = Block::new_fluid(blockid);
        } else {
            block = Block::new();
        }
    } else {
        block = Block::new();
    }

    let raycast_block = world.get_block(ix, iy, iz);

    if raycast_block.can_use() && !player.is_crouching() {
        block = match raycast_block.id {
            //Open gates/door
            78 | 79 | 81 => {
                let mut b = raycast_block;
                let is_open = b.reflection();
                if is_open == 1 {
                    b.set_reflection(0);
                } else {
                    b.set_reflection(1);
                };
                b
            }
            _ => block,
        };

        match block.id {
            //Open door
            79 => {
                let prev_block = world.get_block(ix, iy, iz);
                let ret = if place(world, player, ix, iy, iz, block).is_some() {
                    let mut top = block;
                    top.id = 81;
                    place(world, player, ix, iy + 1, iz, top)
                } else {
                    None
                };

                if ret.is_none() {
                    world.set_block(ix, iy, iz, prev_block);
                }

                return ret;
            }
            81 => {
                let prev_block = world.get_block(ix, iy, iz);
                let ret = if place(world, player, ix, iy, iz, block).is_some() {
                    let mut bot = block;
                    bot.id = 79;
                    place(world, player, ix, iy - 1, iz, bot)
                } else {
                    None
                };

                if ret.is_none() {
                    world.set_block(ix, iy, iz, prev_block);
                }

                return ret;
            }
            _ => return place(world, player, ix, iy, iz, block),
        }
    }
    None
}

//Returns the (x, y, z) coordinate of the block placed as an option
//Returns none if no block is placed
pub fn place_block(
    pos: Vector3<f32>,
    dir: Vector3<f32>,
    world: &mut World,
    player: &Player,
) -> Option<(i32, i32, i32)> {
    let (x, y, z, axis) = raycast(pos, dir, BLOCK_REACH, world, |block| {
        block.is_fluid() || block.id == EMPTY_BLOCK
    });
    //The id of the block we are placing another block on
    let blockid = {
        let (ix, iy, iz) = get_raycast_voxel(x, y, z, dir, axis);
        let block = world.get_block(ix, iy, iz);
        if block.is_fluid() {
            EMPTY_BLOCK
        } else {
            block.id
        }
    };
    let (mut ix, mut iy, mut iz) = get_selected(pos, dir, world);

    let mut block;
    if let Item::Block(blockdata, _) = player.hotbar.get_selected() {
        block = blockdata;
        if block.is_fluid() {
            block.geometry = 7;
        }

        //To prevent leaf decay, we will set the orientation of leaves to be
        //1 if they are a full block and are placed by the player
        //This should not have any visual effect other than a way for the game
        //to differentiate between naturally generated leaves and 'artificial'
        //leaves. This is kind of a hack but I don't want to come up with
        //a better solution.
        if (block.id == 7 || block.id == 91) && block.geometry == 0 {
            block.set_orientation(1);
        }
    } else if let Item::Bucket(blockid) = player.hotbar.get_selected() {
        //Place fluid with a bucket
        if Block::new_id(blockid).is_fluid() {
            block = Block::new_fluid(blockid);
        } else {
            block = Block::new();
        }
    } else {
        block = Block::new();
    }

    let raycast_block = world.get_block(ix, iy, iz);
    //If the player can interact with the block or if the block is
    //replacable (like tall grass) then don't attempt to shift the placed
    //block back
    if (!raycast_block.replaceable() || raycast_block == block)
        && (!raycast_block.can_use() || player.is_crouching())
    {
        match axis {
            Axis::X => ix -= dir.x.signum() as i32,
            Axis::Y => iy -= dir.y.signum() as i32,
            Axis::Z => iz -= dir.z.signum() as i32,
        }
    }

    set_block_rotation(dir, &mut block);
    set_slab_orientation(x, y, z, dir, axis, &mut block);
    set_non_voxel_orientation(dir, axis, &mut block);
    set_stair_rotation(dir, &mut block);

    if block.orientation() % 3 == 0 && block.rotate_y_only() && block.shape() == FULL_BLOCK {
        return None;
    }

    let replace = world.get_block(ix, iy, iz); //Block that is being replaced
    if replace.is_fluid() && block.fluid_destructibe() {
        return None;
    }

    //Do not place a block if we are placing an empty block
    if block.id == EMPTY_BLOCK {
        return None;
    }

    //Do not place a block if the block we are replacing is not empty, nor a fluid,
    //nor replaceable (like tall grass)
    if replace.id != EMPTY_BLOCK && !replace.is_fluid() && !replace.replaceable() {
        return None;
    }

    //Do not place the block if the block we are replacing is the same as the
    //block we are replacing
    if replace.replaceable() && replace == block {
        return None;
    }

    //Do not place a block if we are not placing against a block
    if blockid == EMPTY_BLOCK {
        return None;
    }

    if block.id == 79 {
        //Place door
        let prev_block = world.get_block(ix, iy, iz);
        let ret = if place(world, player, ix, iy, iz, block).is_some() {
            let mut top = block;
            top.id = 81;
            let top_replace = world.get_block(ix, iy + 1, iz);
            if top_replace.id != EMPTY_BLOCK
                && !top_replace.is_fluid()
                && !top_replace.replaceable()
            {
                None
            } else {
                place(world, player, ix, iy + 1, iz, top)
            }
        } else {
            None
        };

        if ret.is_none() {
            world.set_block(ix, iy, iz, prev_block);
        }

        ret
    } else {
        place(world, player, ix, iy, iz, block)
    }
}
