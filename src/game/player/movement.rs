use super::{Player, DEFAULT_PLAYER_SPEED, JUMP_COOLDOWN};
use crate::game::{KeyState, World};

pub const JUMP_FORCE: f32 = 7.5;
pub const SWIM_SPEED: f32 = JUMP_FORCE / 2.0;
const SPRINT_AMT: f32 = 1.4;
const CROUCH_AMT: f32 = 0.33;
//How long the player can spring in seconds
const SPRINT_TIME: f32 = 10.0;
const STAMINA_REGEN: f32 = 1.0 / 40.0;
const STAMINA_REGEN_COOLDOWN: f32 = 15.0;

impl Player {
    pub fn can_sprint(&self) -> bool {
        self.stamina > 0.0
    }

    pub fn update_stamina(&mut self, dt: f32) {
        let vel = self.calculate_velocity();
        let xz_speed_2 = vel.x * vel.x + vel.z * vel.z;

        if self.sprinting && xz_speed_2 > 0.0 {
            self.stamina_regen_cooldown = STAMINA_REGEN_COOLDOWN;
        }

        if self.sprinting && xz_speed_2 > 0.0 {
            //Drain stamina
            let multiplier = if self.velocity_y > 0.0 {
                //If we are sprint jumping, double the stamina lost
                2.0
            } else {
                1.0
            };
            self.stamina -= 1.0 / SPRINT_TIME * dt * multiplier;
        } else if self.stamina_regen_cooldown <= 0.0 {
            //Regain stamina
            self.stamina += STAMINA_REGEN * dt;
        } else {
            self.stamina_regen_cooldown -= dt;
        }
        self.stamina = self.stamina.clamp(0.0, 1.0);
        if !self.can_sprint() && self.stamina < 0.1 {
            self.sprinting = false;
        }
    }

    pub fn is_crouching(&self) -> bool {
        self.crouching
    }

    //Jump up in the y direction
    pub fn jump(&mut self, jump_key: KeyState) {
        if self.falling || self.jump_cooldown > 0.0 || self.velocity_y != 0.0 {
            return;
        }

        if jump_key == KeyState::Held {
            self.velocity_y = JUMP_FORCE;
            self.falling = true;
        }
    }

    //Swim up in the y direction
    pub fn swim(&mut self, swim_key: KeyState, world: &World) {
        let swimming = self.is_swimming(world, 12, 1.0) || self.is_swimming(world, 13, 1.0);
        if (!self.can_move_in_x(world) || !self.can_move_in_z(world))
            && swimming
            && swim_key == KeyState::Held
            && self.swim_cooldown < 0.0
        {
            self.velocity_y = SWIM_SPEED;
            return;
        }

        if !self.is_swimming(world, 12, 0.9) && !self.is_swimming(world, 13, 0.9) {
            return;
        }

        self.jump_cooldown = JUMP_COOLDOWN;

        if self.swim_cooldown > 0.0 {
            return;
        }

        if swim_key == KeyState::Held {
            self.velocity_y = self.velocity_y.max(0.0);
            self.velocity_y = SWIM_SPEED;
        }
    }

    //Sprint when a key is pressed
    pub fn sprint(&mut self, sprint_key: KeyState) {
        if !self.can_sprint() {
            return;
        }
        self.sprinting = sprint_key.is_held();
    }

    pub fn sprint_or(&mut self, sprint_key: KeyState) {
        if !self.can_sprint() {
            return;
        }
        self.sprinting = self.sprinting || sprint_key.is_held();
    }

    //Crouch when a key is pressed
    pub fn crouch(&mut self, crouch_key: KeyState) {
        self.crouching = crouch_key.is_held();
    }

    pub fn crouch_or(&mut self, crouch_key: KeyState) {
        self.crouching = self.crouching || crouch_key.is_held();
    }

    //Set speed of player
    pub fn set_speed(&mut self) {
        if self.crouching {
            self.speed = DEFAULT_PLAYER_SPEED * CROUCH_AMT;
        } else if self.sprinting && self.can_sprint() {
            self.speed = DEFAULT_PLAYER_SPEED * SPRINT_AMT;
        } else {
            self.speed = DEFAULT_PLAYER_SPEED;
        }
    }
}
