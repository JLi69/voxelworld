/*
 * This file contains a collection of functions for updating the player for
 * survival mode (such as handling health/damage)
 * */

use super::{Player, DAMAGE_COOLDOWN, DEFAULT_MAX_HEALTH, DROWN_TIME};
use crate::voxel::World;

pub const DAMAGE_TIME: f32 = 1.5; //In seconds

impl Player {
    pub fn damage(&mut self, amt: i32, msg: &str) {
        if amt == 0 {
            return;
        }
        self.damage_timer = DAMAGE_TIME;
        self.health -= amt;
        self.health = self.health.clamp(0, DEFAULT_MAX_HEALTH);
        self.death_msg = msg.to_string();
    }

    pub fn apply_fall_damage(&mut self) {
        if self.falling {
            return;
        }

        let dist = self.dist_fallen;
        self.dist_fallen = 0.0;
        let dmg_amt = (dist - 2.9).max(0.0).floor() as i32;
        self.damage(dmg_amt, "You broke every bone in your body.");
    }

    //dmg_fn() returns true -> apply damage
    //returns false -> do not apply damage
    fn apply_damage<T>(&mut self, amt: i32, msg: &str, dmg_fn: T)
    where
        T: Fn(&mut Self) -> bool,
    {
        if self.damage_cooldown > 0.0 {
            return;
        }

        if self.health <= 0 {
            return;
        }

        if !dmg_fn(self) {
            return;
        }

        self.damage(amt, msg);
        self.damage_cooldown = DAMAGE_COOLDOWN;
    }

    pub fn damage_timer_perc(&self) -> f32 {
        (self.damage_timer / DAMAGE_TIME).clamp(0.0, 1.0)
    }

    //Specific things to update for survival mode
    pub fn update_survival(&mut self, dt: f32, world: &World) {
        if self.is_dead() {
            return;
        }

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
        //Drowning damage
        self.apply_damage(1, "You are now sleeping with the fishies.", |player| {
            player.drowning_timer <= 0.01
        });

        //Lava damage
        self.apply_damage(3, "You were burnt to a crisp.", |player| {
            player.is_intersecting(world, 13)
        });

        //Suffocation damage
        self.apply_damage(1, "You learned that walls are not breathable.", |player| {
            player.suffocating(world)
        });

        //Cactus damage
        self.apply_damage(1, "You somehow lost a fight with a cactus.", |player| {
            let vel = player.calculate_velocity() * 0.01;
            player.position += vel;
            player.position.y -= 0.05;
            let hit_cactus = player.is_intersecting(world, 88);
            player.position -= vel;
            player.position.y += 0.05;
            hit_cactus
        })
    }

    pub fn is_dead(&self) -> bool {
        self.health <= 0
    }
}
