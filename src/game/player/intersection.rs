use super::{Player, PLAYER_HEIGHT, PLAYER_SIZE};
use crate::game::{Hitbox, World};
use crate::voxel::EMPTY_BLOCK;
use cgmath::InnerSpace;

impl Player {
    //Returns an optional hitbox of a block that the player is coliding with
    //Returns none if no block is found
    pub fn check_collision(&self, world: &World) -> Option<Hitbox> {
        let ix = self.position.x.floor() as i32;
        let iy = self.position.y.floor() as i32;
        let iz = self.position.z.floor() as i32;

        let mut hit: Option<Hitbox> = None;
        let mut min_dist = 999.0;
        for x in (ix - 2)..=(ix + 2) {
            for y in (iy - 2)..=(iy + 2) {
                for z in (iz - 2)..=(iz + 2) {
                    if world.get_block(x, y, z).id == EMPTY_BLOCK {
                        continue;
                    }

                    if world.get_block(x, y, z).no_hitbox() {
                        continue;
                    }

                    let block_hitbox = Hitbox::from_block(x, y, z);

                    if !self.get_hitbox().intersects(&block_hitbox) {
                        continue;
                    }

                    if (block_hitbox.position - self.position).magnitude() > min_dist {
                        continue;
                    }

                    min_dist = (block_hitbox.position - self.position).magnitude();
                    hit = Some(block_hitbox);
                }
            }
        }

        hit
    }

    //Returns true if the head is intersecting a specified block
    pub fn head_intersection(&self, world: &World, block_id: u8) -> bool {
        let ix = self.position.x.floor() as i32;
        let iy = self.position.y.floor() as i32;
        let iz = self.position.z.floor() as i32;

        let head_hitbox = Hitbox::new(
            self.position.x,
            self.position.y + PLAYER_HEIGHT / 2.0 - 0.2,
            self.position.z,
            PLAYER_SIZE,
            0.4,
            PLAYER_SIZE,
        );

        for x in (ix - 2)..=(ix + 2) {
            for y in (iy - 2)..=(iy + 2) {
                for z in (iz - 2)..=(iz + 2) {
                    let block = world.get_block(x, y, z);
                    if block.id != block_id {
                        continue;
                    }

                    let block_hitbox = Hitbox::from_block_data(x, y, z, block);

                    if block_hitbox.intersects(&head_hitbox) {
                        return true;
                    }
                }
            }
        }

        false
    }

    //Returns true if the player is intersecting a specific block type
    pub fn is_intersecting(&self, world: &World, block_id: u8) -> bool {
        let ix = self.position.x.floor() as i32;
        let iy = self.position.y.floor() as i32;
        let iz = self.position.z.floor() as i32;

        for x in (ix - 2)..=(ix + 2) {
            for y in (iy - 2)..=(iy + 2) {
                for z in (iz - 2)..=(iz + 2) {
                    if world.get_block(x, y, z).id != block_id {
                        continue;
                    }

                    let block_hitbox = Hitbox::from_block(x, y, z);
                    let hitbox = self.get_hitbox();

                    if block_hitbox.intersects(&hitbox) {
                        return true;
                    }
                }
            }
        }

        false
    }

    //Is the top `fract` portion of the player intersecting a block
    pub fn top_intersecting(&self, world: &World, block_id: u8, fract: f32) -> bool {
        let ix = self.position.x.floor() as i32;
        let iy = self.position.y.floor() as i32;
        let iz = self.position.z.floor() as i32;

        let hitbox = Hitbox::new(
            self.position.x,
            self.position.y + PLAYER_HEIGHT * (1.0 - fract) / 2.0,
            self.position.z,
            PLAYER_SIZE,
            PLAYER_HEIGHT * fract,
            PLAYER_SIZE,
        );

        for x in (ix - 2)..=(ix + 2) {
            for y in (iy - 2)..=(iy + 2) {
                for z in (iz - 2)..=(iz + 2) {
                    if world.get_block(x, y, z).id != block_id {
                        continue;
                    }

                    let block_hitbox = Hitbox::from_block(x, y, z);

                    if block_hitbox.intersects(&hitbox) {
                        return true;
                    }
                }
            }
        }

        false
    }

    //Is the player standing on a block?
    pub fn standing_on_block(&self, world: &World) -> bool {
        let ix = self.position.x.floor() as i32;
        let iy = self.position.y.floor() as i32;
        let iz = self.position.z.floor() as i32;
        let mut hitbox = self.get_hitbox();
        hitbox.position.y -= 0.02;
        for x in (ix - 2)..=(ix + 2) {
            for y in (iy - 2)..=iy {
                for z in (iz - 2)..=(iz + 2) {
                    if world.get_block(x, y, z).id == EMPTY_BLOCK {
                        continue;
                    }

                    if world.get_block(x, y, z).no_hitbox() {
                        continue;
                    }

                    let block_hitbox = Hitbox::from_block(x, y, z);

                    if hitbox.intersects(&block_hitbox) {
                        return true;
                    }
                }
            }
        }

        false
    }

    pub fn stuck(&mut self, world: &World) -> bool {
        if let Some(block) = self.check_collision(world) {
            //Attempt to uncollide
            let prev_x = self.position.x;
            self.uncollide_x(&block);
            if self.check_collision(world).is_none() {
                return false;
            }

            self.position.x = prev_x;
            let prev_z = self.position.z;
            self.uncollide_z(&block);
            if self.check_collision(world).is_none() {
                return false;
            }
            self.position.z = prev_z;

            let prev_y = self.position.y;
            self.uncollide_y(&block);
            if self.check_collision(world).is_none() {
                return false;
            }
            self.position.y = prev_y;

            return true;
        }

        false
    }

    pub fn suffocating(&self, world: &World) -> bool {
        let ix = self.position.x.floor() as i32;
        let iy = self.position.y.floor() as i32;
        let iz = self.position.z.floor() as i32;

        let head_hitbox = Hitbox::new(
            self.position.x,
            self.position.y + PLAYER_HEIGHT / 2.0 - 0.2,
            self.position.z,
            PLAYER_SIZE,
            0.4,
            PLAYER_SIZE,
        );

        for x in (ix - 2)..=(ix + 2) {
            for y in (iy - 2)..=(iy + 2) {
                for z in (iz - 2)..=(iz + 2) {
                    let block = world.get_block(x, y, z);
                    if block.id == EMPTY_BLOCK {
                        continue;
                    }

                    if block.no_hitbox() {
                        continue;
                    }

                    if block.transparent() {
                        continue;
                    }

                    let block_hitbox = Hitbox::from_block_data(x, y, z, block);

                    if block_hitbox.intersects(&head_hitbox) {
                        return true;
                    }
                }
            }
        }

        false
    }

    //Returns the block that the player's head is intersecting
    pub fn get_head_stuck_block(&self, world: &World) -> Option<(i32, i32, i32)> {
        let ix = self.position.x.floor() as i32;
        let iy = self.position.y.floor() as i32;
        let iz = self.position.z.floor() as i32;

        let head_hitbox = Hitbox::new(
            self.position.x,
            self.position.y + PLAYER_HEIGHT / 2.0 - 0.2,
            self.position.z,
            PLAYER_SIZE,
            0.4,
            PLAYER_SIZE,
        );

        for x in (ix - 2)..=(ix + 2) {
            for y in (iy - 2)..=(iy + 2) {
                for z in (iz - 2)..=(iz + 2) {
                    let block = world.get_block(x, y, z);
                    if block.id == EMPTY_BLOCK {
                        continue;
                    }

                    if block.no_hitbox() {
                        continue;
                    }

                    let block_hitbox = Hitbox::from_block_data(x, y, z, block);

                    if block_hitbox.intersects(&head_hitbox) {
                        return Some((x, y, z));
                    }
                }
            }
        }

        None
    }
}
