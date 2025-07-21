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
const MAX_Y_OFFSET: f32 = 0.2;
const YSPEED: f32 = 0.1;
const LAVA_DESTRUCTION_TIME: f32 = 0.15;

#[derive(Clone)]
pub struct DroppedItem {
    pub item: Item,
    pub entity: Entity,
    pub yoffset: f32,
    dy: f32,
    //Ignore this item for this amount of time when it is dropped by the player
    ignore_pickup_timer: f32,
    //How long until this item is destroyed (in seconds)
    lifetime_timer: f32,
    //This timer goes down if the dropped item is in lava
    //if it goes below 0.0, then the item is destroyed
    lava_destruction_timer: f32,
}

impl DroppedItem {
    pub fn new(item: Item, x: f32, y: f32, z: f32) -> Self {
        let mut e = Entity::new(
            vec3(x, y, z),
            vec3(DROPPED_ITEM_SIZE, DROPPED_ITEM_SIZE, DROPPED_ITEM_SIZE),
            vec3(0.0, 0.0, 0.0),
        );
        e.yaw = fastrand::f32() * 360.0;

        Self {
            item,
            yoffset: fastrand::f32() * MAX_Y_OFFSET,
            dy: YSPEED,
            entity: e,
            ignore_pickup_timer: 0.0,
            lifetime_timer: ITEM_LIFETIME,
            lava_destruction_timer: LAVA_DESTRUCTION_TIME,
        }
    }

    pub fn thrown_item(item: Item, x: f32, y: f32, z: f32, vel: Vec3) -> Self {
        let mut e = Entity::from_vel(
            vec3(x, y, z),
            vec3(DROPPED_ITEM_SIZE, DROPPED_ITEM_SIZE, DROPPED_ITEM_SIZE),
            vel,
        );
        e.yaw = fastrand::f32() * 360.0;

        Self {
            item,
            yoffset: fastrand::f32() * MAX_Y_OFFSET,
            dy: YSPEED,
            entity: e,
            ignore_pickup_timer: ITEM_IGNORE_PICKUP,
            lifetime_timer: ITEM_LIFETIME,
            lava_destruction_timer: LAVA_DESTRUCTION_TIME,
        }
    }

    pub fn update(&mut self, dt: f32, world: &World, player: &mut Player) {
        self.entity.yaw += 90.0 * dt;

        if self.ignore_pickup_timer > 0.0 {
            self.ignore_pickup_timer -= dt;
        } else if self.ignore_pickup_timer <= 0.0 {
            let mut hitbox = self.entity.get_hitbox();
            //Increase the size of the hitbox to make picking up the item easier
            hitbox.dimensions *= 5.0;
            if player.get_hitbox().intersects(&hitbox) && !player.is_dead() {
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

        self.yoffset += self.dy * dt;
        if self.yoffset > MAX_Y_OFFSET || self.yoffset < 0.0 {
            self.dy *= -1.0;
        }
        self.yoffset = self.yoffset.clamp(0.0, MAX_Y_OFFSET);

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

        //Check if the dropped item is intersecting lava
        if self.entity.is_intersecting(world, 13) {
            self.lava_destruction_timer -= dt;
        } else {
            self.lava_destruction_timer = LAVA_DESTRUCTION_TIME;
        }
    }

    pub fn get_chunk(&self) -> (i32, i32, i32) {
        self.entity.get_chunk()
    }

    pub fn destroyed(&self) -> bool {
        self.entity.destroyed || self.lifetime_timer <= 0.0 || self.lava_destruction_timer < 0.0
    }

    pub fn pos(&self) -> Vec3 {
        self.entity.position + vec3(0.0, self.yoffset, 0.0)
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

        let to_delete: Vec<(i32, i32, i32)> = self
            .item_list
            .iter()
            .filter(|(_, list)| list.is_empty())
            .map(|(pos, _)| *pos)
            .collect();
        for pos in to_delete {
            self.item_list.remove(&pos);
        }
    }
}
