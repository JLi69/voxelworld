pub mod dropped_item;
pub mod region;
pub mod save;

use self::dropped_item::DroppedItemTable;
use super::{
    physics::{get_block_collision, scan_block_hitbox, Hitbox},
    player::Player,
};
use crate::{
    bin_data::DataTable,
    gfx::chunktable::get_hand_light,
    voxel::{World, CHUNK_SIZE_F32},
};
use cgmath::{vec3, InnerSpace};

pub type Vec3 = cgmath::Vector3<f32>;

pub const GRAVITY: f32 = 24.0;
const BLOCK_OFFSET: f32 = 0.01;
pub const ENTITIES_PATH: &str = "entities/";

#[derive(Clone)]
#[allow(dead_code)]
pub struct Entity {
    pub position: Vec3,
    pub dimensions: Vec3,
    pub velocity: Vec3,
    falling: bool,
    pub pitch: f32,
    pub yaw: f32,
    destroyed: bool,
}

impl Entity {
    pub fn new(pos: Vec3, dim: Vec3, vel: Vec3) -> Self {
        Self {
            position: pos,
            dimensions: dim,
            velocity: vel,
            falling: false,
            pitch: 0.0,
            yaw: 0.0,
            destroyed: false,
        }
    }

    pub fn from_vel(pos: Vec3, dim: Vec3, vel: Vec3) -> Self {
        Self {
            position: pos,
            dimensions: dim,
            velocity: vel,
            falling: false,
            pitch: 0.0,
            yaw: 0.0,
            destroyed: false,
        }
    }

    pub fn get_hitbox(&self) -> Hitbox {
        Hitbox::from_vecs(self.position, self.dimensions)
    }

    //Returns an optional hitbox of a block that the player is coliding with
    //Returns none if no block is found
    pub fn check_collision(&self, world: &World) -> Option<Hitbox> {
        get_block_collision(world, &self.get_hitbox())
    }

    fn can_move_in_x(&mut self, world: &World) -> bool {
        let x = self.position.x;

        self.position.x = x - BLOCK_OFFSET;
        if self.velocity.x < 0.0 && self.check_collision(world).is_some() {
            self.position.x = x;
            return false;
        }

        self.position.x = x + BLOCK_OFFSET;
        if self.velocity.x > 0.0 && self.check_collision(world).is_some() {
            self.position.x = x;
            return false;
        }

        self.position.x = x;
        true
    }

    fn can_move_in_z(&mut self, world: &World) -> bool {
        let z = self.position.z;

        self.position.z = z - BLOCK_OFFSET;
        if self.velocity.z < 0.0 && self.check_collision(world).is_some() {
            self.position.z = z;
            return false;
        }

        self.position.z = z + BLOCK_OFFSET;
        if self.velocity.z > 0.0 && self.check_collision(world).is_some() {
            self.position.z = z;
            return false;
        }

        self.position.z = z;
        true
    }

    //Uncollide with a hitbox in the x direction
    fn uncollide_x(&mut self, hitbox: &Hitbox) {
        if !self.get_hitbox().intersects(hitbox) {
            return;
        }

        //Uncollide in the x axis
        let sx = self.get_hitbox().dimensions.x + hitbox.dimensions.x;
        if self.position.x < hitbox.position.x {
            self.position.x = hitbox.position.x - sx / 2.0 - BLOCK_OFFSET;
        } else if self.position.x > hitbox.position.x {
            self.position.x = hitbox.position.x + sx / 2.0 + BLOCK_OFFSET;
        }
    }

    //Uncollide with a hitbox in the z direction
    fn uncollide_z(&mut self, hitbox: &Hitbox) {
        if !self.get_hitbox().intersects(hitbox) {
            return;
        }

        let sz = self.get_hitbox().dimensions.z + hitbox.dimensions.z;
        if self.position.z < hitbox.position.z {
            self.position.z = hitbox.position.z - sz / 2.0 - BLOCK_OFFSET;
        } else if self.position.z > hitbox.position.z {
            self.position.z = hitbox.position.z + sz / 2.0 + BLOCK_OFFSET;
        }
    }

    //Uncollide with a hitbox in the y direction
    fn uncollide_y(&mut self, hitbox: &Hitbox) {
        if !self.get_hitbox().intersects(hitbox) {
            return;
        }

        let sy = self.get_hitbox().dimensions.y + hitbox.dimensions.y;
        if self.position.y < hitbox.position.y {
            self.position.y = hitbox.position.y - sy / 2.0;
            self.falling = true;
            self.velocity.y = 0.0;
            self.position.y -= 0.01;
        } else if self.position.y > hitbox.position.y {
            self.position.y = hitbox.position.y + sy / 2.0;
            //Increase the y position so that we are slightly hovering over
            //the block - this is to prevent some issues with collision
            self.position.y += 0.01;
            self.falling = false;
            self.velocity.y = 0.0;
        }
    }

    //Returns true if collision found, false otherwise
    fn check_y_collision(&mut self, world: &World) -> bool {
        //We lower the player's y position to check if we intersect with any blocks
        self.position.y -= 0.02;
        if let Some(block_hitbox) = self.check_collision(world) {
            self.uncollide_y(&block_hitbox);
            true
        } else {
            self.falling = true;
            //If we don't intersect with anything, reset the y position
            self.position.y += 0.02;
            false
        }
    }

    pub fn translate(&mut self, dt: f32, world: &World) {
        let mut dx = if !self.can_move_in_x(world) {
            0.0
        } else {
            self.velocity.x * dt
        };

        let mut dy = dt * self.velocity.y;

        let mut dz = if !self.can_move_in_z(world) {
            0.0
        } else {
            self.velocity.z * dt
        };

        let mut dist_remaining = (dx * dx + dy * dy + dz * dz).sqrt();
        while dist_remaining > 0.0 {
            let d = dist_remaining.min(0.25);

            let vx = d / dist_remaining * dx;
            let vz = d / dist_remaining * dz;
            let vy = d / dist_remaining * dy;

            //Move in the y direction
            self.position.y += vy;
            while let Some(hitbox) = self.check_collision(world) {
                self.uncollide_y(&hitbox);
            }

            //Move in the x direction
            self.position.x += vx;
            let block_hitbox = self.check_collision(world);
            if let Some(block_hitbox) = block_hitbox {
                self.uncollide_x(&block_hitbox);
            }

            //Move in the z direction
            self.position.z += vz;
            let block_hitbox = self.check_collision(world);
            if let Some(block_hitbox) = block_hitbox {
                self.uncollide_z(&block_hitbox);
            }

            dx -= vx;
            dy -= vy;
            dz -= vz;
            dist_remaining = (dx * dx + dy * dy + dz * dz).sqrt();
        }
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
            self.position.y += 0.5;
            self.uncollide_y(&block);
            if self.check_collision(world).is_none() {
                return false;
            }
            self.position.y = prev_y;

            return true;
        }

        false
    }

    pub fn get_chunk(&self) -> (i32, i32, i32) {
        (
            (self.position.x / CHUNK_SIZE_F32).floor() as i32,
            (self.position.y / CHUNK_SIZE_F32).floor() as i32,
            (self.position.z / CHUNK_SIZE_F32).floor() as i32,
        )
    }

    pub fn destroy(&mut self) {
        self.destroyed = true;
    }

    //Returns true if the player is intersecting a specific block type
    pub fn is_intersecting(&self, world: &World, block_id: u8) -> bool {
        let ix = self.position.x.floor() as i32;
        let iy = self.position.y.floor() as i32;
        let iz = self.position.z.floor() as i32;

        let hitbox = self.get_hitbox();
        scan_block_hitbox(&hitbox, world, ix, iy, iz, 2, |block| block.id != block_id).is_some()
    }

    pub fn to_data_table(&self) -> DataTable {
        let mut data_table = DataTable::new();
        data_table.add_vec3("pos", self.position);
        data_table.add_vec3("dim", self.dimensions);
        data_table.add_vec3("vel", self.velocity);
        data_table.add_float("pitch", self.pitch);
        data_table.add_float("yaw", self.yaw);
        data_table
    }

    pub fn from_data_table(data_table: &DataTable) -> Option<Self> {
        let position = data_table.get_vec3("pos")?;
        let dimensions = data_table.get_vec3("dim")?;

        Some(Self {
            position,
            dimensions,
            velocity: data_table
                .get_vec3("vel")
                .unwrap_or(vec3(0.0, 0.0, 0.0)),
            falling: false,
            pitch: data_table.get_float("pitch").unwrap_or(0.0),
            yaw: data_table.get_float("yaw").unwrap_or(0.0),
            destroyed: false,
        })
    }
}

//Entities
pub struct EntitiesTable {
    pub dropped_items: DroppedItemTable,
}

impl EntitiesTable {
    pub fn new() -> Self {
        Self {
            dropped_items: DroppedItemTable::new(),
        }
    }

    pub fn update(&mut self, dt: f32, world: &World, player: &mut Player) {
        self.dropped_items.simulate(dt, world, player);
    }
}

const QUADRATIC: f32 = 1.0 / 12.0;
const LINEAR: f32 = 1.0 / 16.0;
const CONSTANT: f32 = 1.0;

//Calculates what the color the entity should be tinted as, based on
//sky light, block light, and player light
//Returns (r, g, b)
pub fn get_entity_tint(pos: Vec3, world: &World, player: &Player) -> (f32, f32, f32) {
    //Block/sky light
    let (lightr, lightg, lightb) = world.get_client_light(
        pos.x.floor() as i32,
        pos.y.floor() as i32,
        pos.z.floor() as i32,
    );
    //Light from whatever the player is holding
    let dist = (player.position - pos).magnitude();
    let attenuation = 1.0 / (QUADRATIC * dist.powi(2) + LINEAR * dist + CONSTANT);
    let (itemr, itemg, itemb) = get_hand_light(player.hotbar.get_selected());
    (
        lightr.max(itemr * attenuation),
        lightg.max(itemg * attenuation),
        lightb.max(itemb * attenuation),
    )
}
