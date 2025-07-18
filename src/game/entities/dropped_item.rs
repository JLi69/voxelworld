use super::{Entity, Vec3, GRAVITY};
use crate::{
    game::{inventory::Item, player::Player},
    voxel::{
        world::{get_simulation_dist, in_sim_range},
        World,
    },
};
use cgmath::vec3;
use std::collections::HashMap;

pub const DROPPED_ITEM_SIZE: f32 = 0.25;
//In seconds
pub const ITEM_LIFETIME: f32 = 300.0;
//In seconds
const ITEM_IGNORE_PICKUP: f32 = 1.0;

#[derive(Clone)]
pub struct DroppedItem {
    pub item: Item,
    pub entity: Entity,
    //In degrees
    pub rotation: f32,
    //Ignore this item for this amount of time when it is dropped by the player
    ignore_pickup_timer: f32,
    //How long until this item is destroyed (in seconds)
    lifetime_timer: f32,
}

impl DroppedItem {
    pub fn new(item: Item, x: f32, y: f32, z: f32) -> Self {
        Self {
            item,
            entity: Entity::new(
                vec3(x, y, z),
                vec3(DROPPED_ITEM_SIZE, DROPPED_ITEM_SIZE, DROPPED_ITEM_SIZE),
                vec3(0.0, 0.0, 0.0),
            ),
            ignore_pickup_timer: 0.0,
            rotation: fastrand::f32() * 360.0,
            lifetime_timer: ITEM_LIFETIME,
        }
    }

    pub fn thrown_item(item: Item, x: f32, y: f32, z: f32, vel: Vec3) -> Self {
        Self {
            item,
            entity: Entity::from_vel(
                vec3(x, y, z),
                vec3(DROPPED_ITEM_SIZE, DROPPED_ITEM_SIZE, DROPPED_ITEM_SIZE),
                vel,
            ),
            ignore_pickup_timer: ITEM_IGNORE_PICKUP,
            rotation: fastrand::f32() * 360.0,
            lifetime_timer: ITEM_LIFETIME,
        }
    }

    pub fn update(&mut self, dt: f32, world: &World, player: &mut Player) {
        self.rotation += 90.0 * dt;

        if self.ignore_pickup_timer > 0.0 {
            self.ignore_pickup_timer -= dt;
        } else if self.ignore_pickup_timer <= 0.0 {
            let hitbox = self.entity.get_hitbox();
            if player.get_hitbox().intersects(&hitbox) {
                let leftover = if !self.destroyed() {
                    player.add_item(self.item)
                } else {
                    Item::Empty
                };

                if leftover.is_empty() {
                    self.entity.destroy();
                } else {
                    self.item = leftover;
                }
            }
        }

        if self.ignore_pickup_timer <= 0.0 {
            self.lifetime_timer -= dt;
        }

        if self.entity.stuck(world) {
            return;
        }

        self.entity.check_y_collision(world);
        self.entity.translate(dt * 0.5, world);
        if self.entity.falling {
            self.entity.velocity.y -= GRAVITY * dt;
        }
        self.entity.translate(dt * 0.5, world);

        if !self.entity.falling {
            self.entity.velocity.x = 0.0;
            self.entity.velocity.z = 0.0;
        }
    }

    pub fn get_chunk(&self) -> (i32, i32, i32) {
        self.entity.get_chunk()
    }

    pub fn destroyed(&self) -> bool {
        self.entity.destroyed || self.lifetime_timer <= 0.0
    }

    pub fn pos(&self) -> Vec3 {
        self.entity.position
    }

    pub fn scale(&self) -> Vec3 {
        self.entity.dimensions
    }
}

pub struct DroppedItemTable {
    item_list: HashMap<(i32, i32, i32), Vec<DroppedItem>>,
}

impl DroppedItemTable {
    pub fn new() -> Self {
        Self {
            item_list: HashMap::new(),
        }
    }

    pub fn add_item(&mut self, dropped_item: DroppedItem) {
        let chunkpos = dropped_item.get_chunk();
        if let Some(list) = self.item_list.get_mut(&chunkpos) {
            list.push(dropped_item);
        } else {
            self.item_list.insert(chunkpos, vec![dropped_item]);
        }
    }

    pub fn items(&self) -> &HashMap<(i32, i32, i32), Vec<DroppedItem>> {
        &self.item_list
    }

    pub fn simulate(&mut self, dt: f32, world: &World, player: &mut Player) {
        let sim_dist = get_simulation_dist(world);
        let center = world.get_center();
        //Update all dropped items
        for ((x, y, z), list) in &mut self.item_list {
            if !world.chunk_in_world(*x, *y, *z) {
                continue;
            }

            //Out of range
            if !in_sim_range(center, (*x, *y, *z), sim_dist) {
                continue;
            }

            for dropped_item in list {
                dropped_item.update(dt, world, player);
            }
        }

        //List of dropped items that should be moved to a different chunk
        let mut updated = vec![];
        //Check for any dropped items that are not in the correct chunk
        for (chunkpos, list) in &mut self.item_list {
            loop {
                let mut index = None;
                for (i, item) in list.iter().enumerate() {
                    if item.get_chunk() != *chunkpos || item.destroyed() {
                        index = Some(i);
                        break;
                    }
                }

                if let Some(i) = index {
                    let dropped_item = list[i].clone();
                    //Swap with the last item and then pop the last item
                    if let Some(last) = list.last() {
                        list[i] = last.clone();
                        list.pop();
                    }
                    //Ignore if the item is destroyed
                    if !dropped_item.destroyed() {
                        updated.push(dropped_item);
                    }
                } else {
                    //Stop when no items were found
                    break;
                }
            }
        }

        for dropped_item in updated {
            self.add_item(dropped_item);
        }
    }
}
