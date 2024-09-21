use super::Hitbox;
use crate::voxel::Block;
use crate::voxel::World;
use crate::voxel::EMPTY_BLOCK;
use cgmath::{Deg, InnerSpace, Matrix4, Vector3, Vector4};

use super::KeyState;

pub const DEFAULT_PLAYER_SPEED: f32 = 4.0;
pub const PLAYER_HEIGHT: f32 = 1.8;
pub const CAMERA_OFFSET: f32 = 0.7;
pub const GRAVITY: f32 = 9.8;
pub const JUMP_FORCE: f32 = 5.0;

pub struct Player {
    pub position: Vector3<f32>,
    pub dimensions: Vector3<f32>,
    direction: Vector3<f32>,
    falling: bool,
    velocity_y: f32,
    pub speed: f32,
    pub rotation: f32,
    //TODO: This should probably be replaced with some sort of inventory,
    //for now this can just function as the player's "selected" block id
    pub selected_block: Block,
}

impl Player {
    //Create new player object
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            position: Vector3::new(x, y, z),
            dimensions: Vector3::new(0.5, PLAYER_HEIGHT, 0.5),
            direction: Vector3::new(0.0, 0.0, 0.0),
            falling: true,
            velocity_y: 0.0,
            speed: DEFAULT_PLAYER_SPEED,
            rotation: 0.0,
            selected_block: Block::new_id(1),
        }
    }

    pub fn select_block(&mut self, keystate: KeyState, block_id: u8) {
        if keystate.is_held() {
            self.selected_block = Block::new_id(block_id);
        }
    }

    //Jump up in the y direction
    pub fn jump(&mut self, jump_key: KeyState) {
        if self.falling || self.velocity_y != 0.0 {
            return;
        }

        if jump_key == KeyState::Held {
            self.velocity_y = JUMP_FORCE;
            self.falling = true;
        }
    }

    //Set direction for strafe camera left and right (x direction)
    pub fn strafe(&mut self, left: KeyState, right: KeyState) {
        self.direction.x = 0.0;

        if left.is_held() {
            self.direction.x += -1.0;
        }

        if right.is_held() {
            self.direction.x += 1.0;
        }
    }

    //Set direction for moving forward and backward (z direction)
    pub fn move_forward(&mut self, forward: KeyState, backward: KeyState) {
        self.direction.z = 0.0;

        if forward.is_held() {
            self.direction.z += -1.0;
        }

        if backward.is_held() {
            self.direction.z += 1.0;
        }
    }

    //Calculate velocity vector
    pub fn calculate_velocity(&self) -> Vector3<f32> {
        let mut vel = Vector3::new(0.0, 0.0, 0.0);

        //Direction for xz plane
        let dirxz = Vector3::new(self.direction.x, 0.0, self.direction.z);
        if dirxz.magnitude() > 0.0 {
            vel += dirxz.normalize() * self.speed;
        }

        //Transform the velocity based on the yaw of the camera
        let vel_transformed =
            Matrix4::from_angle_y(Deg(-self.rotation)) * Vector4::new(vel.x, vel.y, vel.z, 1.0);

        Vector3::new(vel_transformed.x, vel_transformed.y, vel_transformed.z)
    }

    //Move the player and handle collision
    pub fn update(&mut self, dt: f32, world: &World) {
        //Move camera on position
        let velocity = self.calculate_velocity();

        //NOTE: the collision detection here might not be accurate if the player
        //is travelling too fast or the framerate is too slow
        //TODO: This should probably be fixed/improved later

        //Move in y direction
        self.position.y += self.velocity_y * 0.5 * dt;
        if self.falling {
            self.velocity_y -= dt * GRAVITY;
        }
        self.position.y += self.velocity_y * 0.5 * dt;
        //We lower the player's y position to check if we intersect with any blocks
        self.position.y -= 0.02;
        let block_hitbox = self.check_collision(world);
        if let Some(block_hitbox) = block_hitbox {
            self.uncollide_y(&block_hitbox);
        } else {
            self.falling = true;
            //If we don't intersect with anything, reset the y position
            self.position.y += 0.02;
        }

        //Move in the x direction
        self.position.x += velocity.x * dt;
        let block_hitbox = self.check_collision(world);
        if let Some(block_hitbox) = block_hitbox {
            self.uncollide_x(&block_hitbox);
        }

        //Move in the z direction
        self.position.z += velocity.z * dt;
        let block_hitbox = self.check_collision(world);
        if let Some(block_hitbox) = block_hitbox {
            self.uncollide_z(&block_hitbox);
        } 
    }

    //Calculates the hitbox for the object
    pub fn get_hitbox(&self) -> Hitbox {
        Hitbox::from_vecs(self.position, self.dimensions)
    }

    //Uncollide with a hitbox in the x direction
    fn uncollide_x(&mut self, hitbox: &Hitbox) {
        let player_hitbox = self.get_hitbox();
        if !player_hitbox.intersects(hitbox) {
            return;
        }

        //Uncollide in the x axis
        let sx = player_hitbox.dimensions.x + hitbox.dimensions.x;
        if self.position.x < hitbox.position.x - hitbox.dimensions.x / 2.0 {
            self.position.x = hitbox.position.x - sx / 2.0;
        } else if self.position.x > hitbox.position.x + hitbox.dimensions.x / 2.0 {
            self.position.x = hitbox.position.x + sx / 2.0;
        }
    }

    //Uncollide with a hitbox in the z direction
    fn uncollide_z(&mut self, hitbox: &Hitbox) {
        let player_hitbox = self.get_hitbox();
        if !player_hitbox.intersects(hitbox) {
            return;
        }

        let sz = player_hitbox.dimensions.z + hitbox.dimensions.z;
        if self.position.z < hitbox.position.z - hitbox.dimensions.z / 2.0 {
            self.position.z = hitbox.position.z - sz / 2.0;
        } else if self.position.z > hitbox.position.z + hitbox.dimensions.z / 2.0 {
            self.position.z = hitbox.position.z + sz / 2.0;
        }
    }

    //Uncollide with a hitbox in the y direction and also determine if the player
    //is falling
    fn uncollide_y(&mut self, hitbox: &Hitbox) {
        let player_hitbox = self.get_hitbox();
        if !player_hitbox.intersects(hitbox) {
            return;
        }

        let sy = player_hitbox.dimensions.y + hitbox.dimensions.y;
        if self.position.y < hitbox.position.y {
            self.position.y = hitbox.position.y - sy / 2.0;
            self.falling = true;
            self.velocity_y = 0.0;
        } else if self.position.y > hitbox.position.y {
            self.position.y = hitbox.position.y + sy / 2.0;
            //Increase the y position so that we are slightly hovering over
            //the block - this is to prevent some issues with collision
            self.position.y += 0.01;
            self.falling = false;
            self.velocity_y = 0.0;
        }
    }

    //Returns an optional hitbox of a block that the player is coliding with
    //Returns none if no block is found
    pub fn check_collision(&self, world: &World) -> Option<Hitbox> {
        let ix = self.position.x.floor() as i32;
        let iy = self.position.y.floor() as i32;
        let iz = self.position.z.floor() as i32;

        for x in (ix - 2)..=(ix + 2) {
            for y in (iy - 2)..=(iy + 2) {
                for z in (iz - 2)..=(iz + 2) {
                    if world.get_block(x, y, z).id == EMPTY_BLOCK {
                        continue;
                    }
                    let block_hitbox = Hitbox::from_block(x, y, z);
                    if self.get_hitbox().intersects(&block_hitbox) {
                        return Some(block_hitbox);
                    }
                }
            }
        }

        None
    }
}
