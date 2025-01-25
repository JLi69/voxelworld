use super::{Player, PLAYER_HEIGHT, PLAYER_SIZE};
use crate::game::physics::{composite_to_hitbox, scan_block_hitbox, get_block_collision, scan_block_full_hitbox};
use crate::game::{Hitbox, World};
use crate::voxel::EMPTY_BLOCK;

impl Player {
    //Returns an optional hitbox of a block that the player is coliding with
    //Returns none if no block is found
    pub fn check_collision(&self, world: &World) -> Option<Hitbox> {
        get_block_collision(world, &self.get_hitbox())
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

        scan_block_hitbox(&head_hitbox, world, ix, iy, iz, 2, |block| {
            block.id != block_id
        }).is_some()
    }

    //Returns true if the player is intersecting a specific block type
    pub fn is_intersecting(&self, world: &World, block_id: u8) -> bool {
        let ix = self.position.x.floor() as i32;
        let iy = self.position.y.floor() as i32;
        let iz = self.position.z.floor() as i32;

        let hitbox = self.get_hitbox();
        scan_block_hitbox(&hitbox, world, ix, iy, iz, 2, |block| {
            block.id != block_id
        }).is_some()
    }

    //Is the top `fract` portion of the player intersecting a block
    #[allow(dead_code)]
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

        scan_block_hitbox(&hitbox, world, ix, iy, iz, 2, |block| {
            block.id != block_id
        }).is_some()
    }

    //Is the top `fract` portion of the player intersecting a block (but with swimming)
    pub fn is_swimming(&self, world: &World, block_id: u8, fract: f32) -> bool {
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

        scan_block_full_hitbox(&hitbox, world, ix, iy, iz, 2, |block| {
            block.id != block_id
        }).is_some()
    }

    //Is the bottom `fract` portion of the player intersecting a block
    pub fn bot_intersecting(&self, world: &World, block_id: u8, fract: f32) -> bool {
        let ix = self.position.x.floor() as i32;
        let iy = self.position.y.floor() as i32;
        let iz = self.position.z.floor() as i32;

        let hitbox = Hitbox::new(
            self.position.x,
            self.position.y - PLAYER_HEIGHT * (1.0 - fract) / 2.0,
            self.position.z,
            PLAYER_SIZE,
            PLAYER_HEIGHT * fract,
            PLAYER_SIZE,
        );

        scan_block_hitbox(&hitbox, world, ix, iy, iz, 2, |block| {
            block.id != block_id
        }).is_some()
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

        scan_block_hitbox(&head_hitbox, world, ix, iy, iz, 2, |block| {
            if block.id == EMPTY_BLOCK {
                return true;
            }

            if block.no_hitbox() {
                return true;
            }

            if block.transparent() {
                return true;
            }

            false
        }).is_some()
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

                    let composite_hitbox = Hitbox::from_block_data(x, y, z, block);
                    let block_hitbox = composite_to_hitbox(composite_hitbox, &head_hitbox);

                    if block_hitbox.intersects(&head_hitbox) {
                        return Some((x, y, z));
                    }
                }
            }
        }

        None
    }
}
