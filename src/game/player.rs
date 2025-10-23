mod intersection;
mod movement;
mod survival_mode;

use self::movement::JUMP_FORCE;
use super::entities::dropped_item::DroppedItem;
use super::inventory::{Hotbar, Inventory, Item};
use super::Hitbox;
use super::KeyState;
use crate::game::entities::GRAVITY;
use crate::impfile;
use crate::voxel::tile_data::TileData;
use crate::voxel::World;
use cgmath::{vec3, Deg, InnerSpace, Matrix4, Vector3, Vector4};

pub const DEFAULT_MAX_HEALTH: i32 = 20;
pub const DAMAGE_COOLDOWN: f32 = 1.0; //In seconds
pub const DROWN_TIME: f32 = 20.0; //In seconds
pub const DEFAULT_PLAYER_SPEED: f32 = 4.0;
pub const PLAYER_HEIGHT: f32 = 1.8;
pub const PLAYER_SIZE: f32 = 0.6;
pub const CAMERA_OFFSET: f32 = 0.7;
pub const JUMP_COOLDOWN: f32 = 1.0 / 20.0;
const BLOCK_OFFSET: f32 = 0.01;
const MAX_CROUCH_HEIGHT: f32 = 0.15;
const CLIMB_SPEED: f32 = 2.0;

pub struct Player {
    pub position: Vector3<f32>,
    pub dimensions: Vector3<f32>,
    direction: Vector3<f32>,
    falling: bool,
    velocity_y: f32,
    pub speed: f32,
    pub rotation: f32,
    pub hotbar: Hotbar,
    pub inventory: Inventory,
    pub crafting_grid: Inventory,
    //Item currently held by the mouse cursor
    pub mouse_item: Item,
    jump_cooldown: f32,
    prev_swimming: bool,
    swim_cooldown: f32,
    sprinting: bool,
    crouching: bool,
    crouch_height: f32,
    //Stats
    pub stamina: f32,
    stamina_regen_cooldown: f32,
    pub health: i32,
    dist_fallen: f32,
    pub drowning_timer: f32,
    //Ticks down with time but gets reset every time the player is damaged
    damage_timer: f32,
    damage_cooldown: f32,
    pub death_msg: String,
    //Breaking blocks
    pub break_timer: f32,
    pub target_block: Option<(i32, i32, i32)>,
    //A short delay to prevent the user from clicking on the inventory
    //instantly when they open a chest/furance or similar
    pub inventory_delay_timer: f32,
    //If the player opens a block (like a chest) the data for that block gets
    //copied over to here for the player to interact with
    pub open_block_data: TileData,
    //None if no block is open
    //Some(position) if a block is opened
    pub opened_block: Option<(i32, i32, i32)>,
    pub opened_block_id: u8,
    //Flying
    pub spacebar_timer: f32,
    pub flying: bool,
}

impl Player {
    //Create new player object
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            position: Vector3::new(x, y, z),
            dimensions: Vector3::new(PLAYER_SIZE, PLAYER_HEIGHT, PLAYER_SIZE),
            direction: Vector3::new(0.0, 0.0, 0.0),
            falling: true,
            velocity_y: 0.0,
            speed: DEFAULT_PLAYER_SPEED,
            rotation: 0.0,
            hotbar: Hotbar::empty_hotbar(),
            inventory: Inventory::empty_inventory(),
            crafting_grid: Inventory::empty_with_sz(3, 3),
            mouse_item: Item::Empty,
            jump_cooldown: 0.0,
            prev_swimming: false,
            swim_cooldown: 0.0,
            sprinting: false,
            crouching: true,
            crouch_height: 0.0,
            stamina: 1.0,
            stamina_regen_cooldown: 0.0,
            health: DEFAULT_MAX_HEALTH,
            drowning_timer: DROWN_TIME,
            dist_fallen: 0.0,
            damage_timer: 0.0,
            damage_cooldown: DAMAGE_COOLDOWN,
            death_msg: "".to_string(),
            break_timer: 0.0,
            target_block: None,
            inventory_delay_timer: 0.0,
            open_block_data: TileData::new(),
            opened_block: None,
            opened_block_id: 0,
            spacebar_timer: 0.0,
            flying: false,
        }
    }

    //Pass in the spawn point, returns the reset player
    pub fn respawn(&self, spawnx: f32, spawnz: f32) -> Self {
        Self {
            position: Vector3::new(spawnx, self.position.y, spawnz),
            dimensions: Vector3::new(PLAYER_SIZE, PLAYER_HEIGHT, PLAYER_SIZE),
            direction: Vector3::new(0.0, 0.0, 0.0),
            falling: true,
            velocity_y: 0.0,
            speed: DEFAULT_PLAYER_SPEED,
            rotation: 0.0,
            hotbar: self.hotbar.clone(),
            inventory: self.inventory.clone(),
            crafting_grid: self.crafting_grid.clone(),
            mouse_item: Item::Empty,
            jump_cooldown: 0.0,
            prev_swimming: false,
            swim_cooldown: 0.0,
            sprinting: false,
            crouching: true,
            crouch_height: 0.0,
            stamina: 1.0,
            stamina_regen_cooldown: 0.0,
            health: DEFAULT_MAX_HEALTH,
            drowning_timer: DROWN_TIME,
            dist_fallen: 0.0,
            damage_timer: 0.0,
            damage_cooldown: DAMAGE_COOLDOWN,
            death_msg: "".to_string(),
            break_timer: 0.0,
            target_block: None,
            inventory_delay_timer: 0.0,
            open_block_data: TileData::new(),
            opened_block: None,
            opened_block_id: 0,
            spacebar_timer: 0.0,
            flying: false,
        }
    }

    pub fn select_hotbar_item(&mut self, keystate: KeyState, index: usize) {
        if keystate.is_held() {
            self.hotbar.selected = index;
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
            let drag = 1.0 - (-self.velocity_y / GRAVITY * 4.0).clamp(0.0, 0.4);
            vel += dirxz.normalize() * self.speed * drag;
        }

        //Transform the velocity based on the yaw of the camera
        let vel_transformed =
            Matrix4::from_angle_y(Deg(-self.rotation)) * Vector4::new(vel.x, vel.y, vel.z, 1.0);

        Vector3::new(vel_transformed.x, vel_transformed.y, vel_transformed.z)
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

    fn can_move_in_x(&mut self, world: &World) -> bool {
        let x = self.position.x;

        self.position.x = x - BLOCK_OFFSET;
        if self.calculate_velocity().x < 0.0 && self.check_collision(world).is_some() {
            self.position.x = x;
            return false;
        }

        self.position.x = x + BLOCK_OFFSET;
        if self.calculate_velocity().x > 0.0 && self.check_collision(world).is_some() {
            self.position.x = x;
            return false;
        }

        self.position.x = x;
        true
    }

    fn can_move_in_z(&mut self, world: &World) -> bool {
        let z = self.position.z;

        self.position.z = z - BLOCK_OFFSET;
        if self.calculate_velocity().z < 0.0 && self.check_collision(world).is_some() {
            self.position.z = z;
            return false;
        }

        self.position.z = z + BLOCK_OFFSET;
        if self.calculate_velocity().z > 0.0 && self.check_collision(world).is_some() {
            self.position.z = z;
            return false;
        }

        self.position.z = z;
        true
    }

    //if the jump height is low enough, just have the player autojump
    fn autojump(&mut self, world: &World) {
        //You cannot auto jump if you are crouching
        if self.crouching {
            return;
        }

        if self.falling {
            return;
        }

        let position = self.position;
        let vel = self.calculate_velocity();
        if vel.magnitude() == 0.0 {
            return;
        }
        self.position += vel.normalize() * 0.05;
        if let Some(hitbox) = self.check_collision(world) {
            self.position.y =
                hitbox.position.y + hitbox.dimensions.y / 2.0 + self.dimensions.y / 2.0 + 0.01;
        } else {
            self.position = position;
            return;
        }

        if self.check_collision(world).is_some() {
            self.position = position;
            return;
        }

        if self.position.y - position.y > 0.6 {
            self.position = position;
            return;
        }

        self.position = position;
        self.jump_cooldown = 0.0;
        self.velocity_y = JUMP_FORCE * 0.75;
    }

    //Translate player object, account for collisions with blocks
    fn translate(&mut self, dt: f32, world: &World) {
        //Move in the xz plane
        let velocity = if self.is_swimming(world, 13, 1.0) {
            //Slow down in lava
            self.calculate_velocity() * 0.4
        } else if self.is_swimming(world, 12, 1.0) {
            //Slow down in water
            self.calculate_velocity() * 0.6
        } else {
            self.calculate_velocity()
        };

        let mut dx = if !self.can_move_in_x(world) {
            0.0
        } else {
            velocity.x * dt
        };

        let mut dy = dt * self.velocity_y;
        if self.is_swimming(world, 13, 1.0) {
            //Slow down in lava
            dy *= 0.4;
            self.dist_fallen = 0.0; //No fall damage in lava
        } else if self.is_swimming(world, 12, 1.0) {
            //Slow down in water
            dy *= 0.5;
            self.dist_fallen = 0.0; //No fall damage in water
        }

        let mut dz = if !self.can_move_in_z(world) {
            0.0
        } else {
            velocity.z * dt
        };
        let mut dist_remaining = (dx * dx + dy * dy + dz * dz).sqrt();
        while dist_remaining > 0.0 {
            let d = dist_remaining.min(0.25);

            let vx = d / dist_remaining * dx;
            let vz = d / dist_remaining * dz;
            let vy = d / dist_remaining * dy;

            //Move in the y direction
            self.position.y += vy;
            //Void acts like a hard barrier
            self.position.y = self.position.y.max(world.bottom() as f32 - 64.0);
            if !self.check_y_collision(world) {
                if vy < 0.0 {
                    self.dist_fallen -= vy;
                } else if vy > 0.0 {
                    //Reset distance fallen if we move up
                    self.dist_fallen = 0.0;
                }
            }
            while let Some(hitbox) = self.check_collision(world) {
                self.uncollide_y(&hitbox);
            }

            //Move in the x direction
            self.position.x += vx;
            self.autojump(world);
            let block_hitbox = self.check_collision(world);
            if let Some(block_hitbox) = block_hitbox {
                self.uncollide_x(&block_hitbox);
            }

            if !self.standing_on_block(world) && self.crouching {
                self.position.x -= vx;
            }

            //Move in the z direction
            self.position.z += vz;
            self.autojump(world);
            let block_hitbox = self.check_collision(world);
            if let Some(block_hitbox) = block_hitbox {
                self.uncollide_z(&block_hitbox);
            }

            if !self.standing_on_block(world) && self.crouching {
                self.position.z -= vz;
            }

            dx -= vx;
            dy -= vy;
            dz -= vz;
            dist_remaining = (dx * dx + dy * dy + dz * dz).sqrt();
        }
    }

    pub fn climbing(&self, world: &World) -> bool {
        self.bot_intersecting(world, 75, 0.4)
    }

    pub fn climb(&mut self, up_key: KeyState, hold_key: KeyState, world: &World) {
        self.velocity_y = -CLIMB_SPEED;

        let pos = self.position;
        self.position += self.calculate_velocity() * 0.05;
        let colliding = self.check_collision(world).is_some();
        self.position = pos;

        if self.standing_on_block(world) && !self.is_intersecting(world, 75) {
            return;
        }

        if up_key.is_held() || colliding {
            self.velocity_y += CLIMB_SPEED * 2.0;
        } else if hold_key.is_held() {
            self.velocity_y = 0.0;
        }
    }

    //Move the player and handle collision
    pub fn update(&mut self, dt: f32, world: &World) {
        self.inventory_delay_timer -= dt;
        self.spacebar_timer -= dt;

        if self.is_dead() {
            return;
        }

        if self.stuck(world) {
            return;
        }

        //Update jump cooldown
        self.jump_cooldown -= dt;
        //Update swim cooldown
        self.swim_cooldown -= dt;
        //Check if the player is no longer swimming
        let swimming = self.is_swimming(world, 12, 0.95) || self.is_swimming(world, 13, 0.95);

        //Is the player climbing a ladder?
        let climbing = self.climbing(world);
        if climbing {
            self.falling = false;
            self.velocity_y = self.velocity_y.clamp(-CLIMB_SPEED, CLIMB_SPEED);
        }

        //Check if the player was falling in the previous frame
        let falling_prev = self.falling;
        //Move in y direction
        self.translate(dt * 0.5, world);
        if climbing {
            self.falling = false;
        }
        if self.flying {
            self.falling = false;
        }
        //Apply gravity
        if self.falling {
            self.velocity_y -= dt * GRAVITY;
        }
        if swimming {
            self.velocity_y = self.velocity_y.max(-GRAVITY / 6.0);
        }
        if climbing {
            self.falling = false;
        }
        self.translate(dt * 0.5, world);
        self.check_y_collision(world);

        //Check if the player is no longer falling
        if falling_prev && !self.falling {
            //We landed on the ground, set the jump cooldown
            self.jump_cooldown = JUMP_COOLDOWN;
        }

        if !swimming && self.prev_swimming {
            self.swim_cooldown = 0.4;
        } else if swimming && !self.prev_swimming {
            self.swim_cooldown = 0.2;
        }
        self.prev_swimming = swimming;

        //Crouch
        if self.crouching {
            self.crouch_height += MAX_CROUCH_HEIGHT * dt * 5.0;
            self.crouch_height = self.crouch_height.min(MAX_CROUCH_HEIGHT);
        } else {
            self.crouch_height -= MAX_CROUCH_HEIGHT * dt * 5.0;
            self.crouch_height = self.crouch_height.max(0.0);
        }
    }

    //Specific things to update for creative mode
    pub fn update_creative(&mut self, _dt: f32) {
        self.stamina = 1.0; //Infinite stamina
        self.damage_timer = 0.0;
    }

    pub fn cam_offset(&self) -> Vector3<f32> {
        Vector3::new(0.0, CAMERA_OFFSET - self.crouch_height, 0.0)
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
        if self.position.x < hitbox.position.x {
            self.position.x = hitbox.position.x - sx / 2.0 - BLOCK_OFFSET;
        } else if self.position.x > hitbox.position.x {
            self.position.x = hitbox.position.x + sx / 2.0 + BLOCK_OFFSET;
        }
    }

    //Uncollide with a hitbox in the z direction
    fn uncollide_z(&mut self, hitbox: &Hitbox) {
        let player_hitbox = self.get_hitbox();
        if !player_hitbox.intersects(hitbox) {
            return;
        }

        let sz = player_hitbox.dimensions.z + hitbox.dimensions.z;
        if self.position.z < hitbox.position.z {
            self.position.z = hitbox.position.z - sz / 2.0 - BLOCK_OFFSET;
        } else if self.position.z > hitbox.position.z {
            self.position.z = hitbox.position.z + sz / 2.0 + BLOCK_OFFSET;
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
            self.position.y -= 0.01;
            self.swim_cooldown = 0.2;
        } else if self.position.y > hitbox.position.y {
            self.position.y = hitbox.position.y + sy / 2.0;
            //Increase the y position so that we are slightly hovering over
            //the block - this is to prevent some issues with collision
            self.position.y += 0.01;
            self.falling = false;
            self.velocity_y = 0.0;
        }
    }

    pub fn to_entry(&self) -> impfile::Entry {
        let mut entry = impfile::Entry::new("player");

        entry.add_float("x", self.position.x);
        entry.add_float("y", self.position.y);
        entry.add_float("z", self.position.z);
        entry.add_bool("falling", self.falling);
        entry.add_float("velocity_y", self.velocity_y);
        entry.add_float("rotation", self.rotation);
        entry.add_float("stamina", self.stamina);
        entry.add_float("stamina_regen_cooldown", self.stamina_regen_cooldown);
        entry.add_float("dist_fallen", self.dist_fallen);
        entry.add_integer("health", self.health as i64);
        entry.add_float("drowning_timer", self.drowning_timer);
        entry.add_string("death_msg", &self.death_msg);
        entry.add_bool("flying", self.flying);

        entry
    }

    pub fn from_entry(entry: &impfile::Entry) -> Self {
        let x = entry.get_var("x").parse::<f32>().unwrap_or(0.0);
        let y = entry.get_var("y").parse::<f32>().unwrap_or(0.0);
        let z = entry.get_var("z").parse::<f32>().unwrap_or(0.0);

        //Stats
        let player_stamina = entry.get_var("stamina").parse::<f32>().unwrap_or(1.0);
        let player_stamina_regen_cooldown = entry
            .get_var("stamina_regen_cooldown")
            .parse::<f32>()
            .unwrap_or(0.0);
        let player_health = entry
            .get_var("health")
            .parse::<i32>()
            .unwrap_or(DEFAULT_MAX_HEALTH);
        let player_dist_fallen = entry.get_var("dist_fallen").parse::<f32>().unwrap_or(0.0);
        let player_drowning_timer = entry
            .get_var("drowning_timer")
            .parse::<f32>()
            .unwrap_or(DROWN_TIME);
        let player_death_msg = entry.get_var("death_msg");

        Self {
            position: Vector3::new(x, y, z),
            dimensions: Vector3::new(PLAYER_SIZE, PLAYER_HEIGHT, PLAYER_SIZE),
            direction: Vector3::new(0.0, 0.0, 0.0),
            falling: entry.get_var("falling").parse::<bool>().unwrap_or(false),
            velocity_y: entry.get_var("velocity_y").parse::<f32>().unwrap_or(0.0),
            speed: DEFAULT_PLAYER_SPEED,
            rotation: entry.get_var("rotation").parse::<f32>().unwrap_or(0.0),
            hotbar: Hotbar::empty_hotbar(),
            inventory: Inventory::empty_inventory(),
            crafting_grid: Inventory::empty_with_sz(3, 3),
            mouse_item: Item::Empty,
            jump_cooldown: 0.0,
            prev_swimming: false,
            swim_cooldown: 0.0,
            sprinting: false,
            crouching: false,
            crouch_height: 0.0,
            stamina: player_stamina,
            stamina_regen_cooldown: player_stamina_regen_cooldown,
            health: player_health,
            drowning_timer: player_drowning_timer,
            dist_fallen: player_dist_fallen,
            damage_timer: 0.0,
            //3 seconds of damage immunity
            damage_cooldown: 3.0,
            death_msg: player_death_msg,
            break_timer: 0.0,
            target_block: None,
            inventory_delay_timer: 0.0,
            open_block_data: TileData::new(),
            opened_block: None,
            opened_block_id: 0,
            spacebar_timer: 0.0,
            flying: entry.get_var("flying").parse::<bool>().unwrap_or(false),
        }
    }

    //Returns leftover items
    pub fn add_item(&mut self, item: Item) -> Item {
        if item.is_empty() {
            return Item::Empty;
        }
        let hotbar_leftover = self.hotbar.merge_item(item);
        let inventory_leftover = self.inventory.merge_item(hotbar_leftover);
        let hotbar_leftover = self.hotbar.add_item(inventory_leftover);
        self.inventory.add_item(hotbar_leftover)
    }

    pub fn is_falling(&self) -> bool {
        self.falling
    }

    pub fn throw_item(&self, item: Item, dir: Vector3<f32>) -> DroppedItem {
        let pos = self.position + vec3(0.0, PLAYER_HEIGHT / 4.0, 0.0);
        DroppedItem::thrown_item(item, pos.x, pos.y, pos.z, dir * 6.0)
    }
}
