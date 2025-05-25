/*
 * This file contains a collection of functions for updating the player for
 * survival mode (such as handling health/damage)
 * */

use super::{Player, DEFAULT_MAX_HEALTH};

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

    pub fn damage_timer_perc(&self) -> f32 {
        (self.damage_timer / DAMAGE_TIME).clamp(0.0, 1.0)
    }
}
