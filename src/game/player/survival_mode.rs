/*
 * This file contains a collection of functions for updating the player for
 * survival mode (such as handling health/damage)
 * */

use super::{Player, DAMAGE_COOLDOWN, DEFAULT_MAX_HEALTH, DROWN_TIME};
use crate::voxel::World;

pub const DAMAGE_TIME: f32 = 1.5; //In seconds

impl Player {
    pub fn damage(&mut self, amt: i32) {
        if amt == 0 {
            return;
        }
        self.damage_timer = DAMAGE_TIME;
        self.health -= amt;
        self.health = self.health.clamp(0, DEFAULT_MAX_HEALTH);
    }

    pub fn apply_fall_damage(&mut self) {
        if self.falling {
            return;
        }

        let dist = self.dist_fallen;
        self.dist_fallen = 0.0;
        let dmg_amt = (dist - 2.9).max(0.0).floor() as i32;
        self.damage(dmg_amt);
    }

    pub fn apply_drowning_damage(&mut self) {
        if self.drowning_timer > 0.01 {
            return;
        }

        if self.damage_cooldown > 0.0 {
            return;
        }

        if self.health <= 0 {
            return;
        }

        self.damage(1);
        self.damage_cooldown = DAMAGE_COOLDOWN;
    }

    pub fn damage_timer_perc(&self) -> f32 {
        (self.damage_timer / DAMAGE_TIME).clamp(0.0, 1.0)
    }

    //Specific things to update for survival mode
    pub fn update_survival(&mut self, dt: f32, world: &World) {
        self.update_stamina(dt);
        self.damage_timer -= dt;
        self.damage_cooldown -= dt;
        self.apply_fall_damage();

        if self.head_intersection(world, 12) {
            //Drowning in water
            self.drowning_timer -= dt;
        } else {
            self.drowning_timer += dt;
        }
        self.drowning_timer = self.drowning_timer.clamp(0.0, DROWN_TIME);
        self.apply_drowning_damage();
    }
}
