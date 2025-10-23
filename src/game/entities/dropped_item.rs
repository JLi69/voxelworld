use super::{Entity, Vec3, GRAVITY};
use crate::{
    bin_data::DataTable,
    game::{
        inventory::{item_to_string, merge_stacks, string_to_item_err, Item, MAX_STACK_SIZE},
        physics::Hitbox,
        player::Player,
    },
    voxel::{
        world::{get_simulation_dist, in_sim_range},
        World,
    },
};
use cgmath::vec3;
use std::collections::{HashMap, HashSet};

pub const DROPPED_ITEM_SIZE: f32 = 0.25;
//In seconds
pub const ITEM_LIFETIME: f32 = 300.0;
//In seconds
const ITEM_IGNORE_PICKUP: f32 = 1.0;
const MAX_Y_OFFSET: f32 = 0.2;
const YSPEED: f32 = 0.1;
const LAVA_DESTRUCTION_TIME: f32 = 0.15;

const ADJ: [(i32, i32, i32); 8] = [
    (1, 1, 1),
    (-1, 1, 1),
    (1, -1, 1),
    (1, 1, -1),
    (1, -1, -1),
    (-1, 1, -1),
    (-1, -1, 1),
    (-1, -1, -1),
];

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

    //Entity's hitbox but 5 times larger
    pub fn get_large_hitbox(&self) -> Hitbox {
        let mut hitbox = self.entity.get_hitbox();
        //Increase the size of the hitbox to make picking up the item easier
        hitbox.dimensions *= 5.0;
        hitbox
    }

    pub fn get_merge_hitbox(&self) -> Hitbox {
        let mut hitbox = self.entity.get_hitbox();
        //Triple the size of the hitbox in the x and z dimensions to make
        //it easier for it to merge with other items
        hitbox.dimensions.x *= 3.0;
        hitbox.dimensions.z *= 3.0;
        hitbox
    }

    pub fn update(&mut self, dt: f32, world: &World, player: &mut Player) {
        self.entity.yaw += 90.0 * dt;

        if self.ignore_pickup_timer > 0.0 {
            self.ignore_pickup_timer -= dt;
        } else if self.ignore_pickup_timer <= 0.0 {
            let hitbox = self.get_large_hitbox();
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
        //or is in the void, if it is in the void then destroy it
        let y = self.pos().y;
        if self.entity.is_intersecting(world, 13) || y < world.bottom() as f32 {
            self.lava_destruction_timer -= dt;
        } else {
            self.lava_destruction_timer = LAVA_DESTRUCTION_TIME;
        }
    }

    pub fn get_chunk(&self) -> (i32, i32, i32) {
        self.entity.get_chunk()
    }

    pub fn destroyed(&self) -> bool {
        self.entity.destroyed
            || self.lifetime_timer <= 0.0
            || self.lava_destruction_timer < 0.0
            || self.item.is_empty()
    }

    pub fn pos(&self) -> Vec3 {
        self.entity.position + vec3(0.0, self.yoffset, 0.0)
    }

    pub fn scale(&self) -> Vec3 {
        self.entity.dimensions
    }

    pub fn to_data_table(&self) -> DataTable {
        let mut data_table = self.entity.to_data_table();
        data_table.add_str("item", &item_to_string(self.item));
        data_table.add_float("lifetime_timer", self.lifetime_timer);
        data_table.add_float("ignore_pickup_timer", self.ignore_pickup_timer);
        data_table.add_float("lava_destruction_timer", self.lava_destruction_timer);
        data_table
    }

    pub fn from_data_table(data_table: &DataTable) -> Option<Self> {
        let entity = Entity::from_data_table(data_table)?;
        let item_str = data_table.get_str("item")?;
        let item = string_to_item_err(&item_str).ok()?;

        Some(Self {
            item,
            entity,
            yoffset: fastrand::f32() * MAX_Y_OFFSET,
            dy: YSPEED,
            ignore_pickup_timer: data_table.get_float("ignore_pickup_timer").unwrap_or(0.0),
            lifetime_timer: data_table
                .get_float("lifetime_timer")
                .unwrap_or(ITEM_LIFETIME),
            lava_destruction_timer: data_table
                .get_float("lava_destruction_timer")
                .unwrap_or(LAVA_DESTRUCTION_TIME),
        })
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

    pub fn remove(&mut self, pos: (i32, i32, i32)) {
        self.item_list.remove(&pos);
    }

    pub fn add_empty(&mut self, x: i32, y: i32, z: i32) {
        if self.item_list.contains_key(&(x, y, z)) {
            return;
        }

        self.item_list.insert((x, y, z), vec![]);
    }

    pub fn simulate(&mut self, dt: f32, world: &World, player: &mut Player) {
        let sim_dist = get_simulation_dist(world);
        let center = world.get_center();
        //Update all dropped items
        for ((x, y, z), list) in &mut self.item_list {
            if !world.chunks.contains_key(&(*x, *y, *z)) {
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

        //Merge items
        let mut new_merged_items = vec![];
        let mut to_delete = HashSet::new();
        for ((chunkx, chunky, chunkz), list) in &self.item_list {
            for (i, dropped) in list.iter().enumerate() {
                if dropped.ignore_pickup_timer > 0.0 {
                    continue;
                }

                if to_delete.contains(&((*chunkx, *chunky, *chunkz), i)) {
                    continue;
                }

                if dropped.destroyed() {
                    continue;
                }

                let item_merge = search_for_item_merge(
                    dropped,
                    list,
                    &to_delete,
                    (*chunkx, *chunky, *chunkz),
                    Some(i),
                );

                if let Some((item1, item2, index)) = item_merge {
                    new_merged_items.push(item1);
                    new_merged_items.push(item2);
                    to_delete.insert(((*chunkx, *chunky, *chunkz), i));
                    to_delete.insert(((*chunkx, *chunky, *chunkz), index));
                    continue;
                }

                //Check for items in adjacent chunks that should be merged together
                for (dx, dy, dz) in ADJ {
                    let x = chunkx + dx;
                    let y = chunky + dy;
                    let z = chunkz + dz;
                    if let Some(adj_list) = self.item_list.get(&(x, y, z)) {
                        let item_merge =
                            search_for_item_merge(dropped, adj_list, &to_delete, (x, y, z), None);
                        if let Some((item1, item2, index)) = item_merge {
                            new_merged_items.push(item1);
                            new_merged_items.push(item2);
                            to_delete.insert(((*chunkx, *chunky, *chunkz), i));
                            to_delete.insert(((x, y, z), index));
                            break;
                        }
                    }
                }
            }
        }

        for (chunkpos, index) in to_delete {
            if let Some(list) = self.item_list.get_mut(&chunkpos) {
                list[index].entity.destroy();
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
            if dropped_item.item.is_empty() {
                continue;
            }

            self.add_item(dropped_item);
        }

        for new_item in new_merged_items {
            if new_item.item.is_empty() {
                continue;
            }

            self.add_item(new_item);
        }
    }
}

fn search_for_item_merge(
    dropped: &DroppedItem,
    list: &[DroppedItem],
    to_delete: &HashSet<((i32, i32, i32), usize)>,
    chunkpos: (i32, i32, i32),
    index: Option<usize>,
) -> Option<(DroppedItem, DroppedItem, usize)> {
    //Ignore if it is a full stack or unstackable
    match dropped.item {
        Item::Block(_, amt) | Item::Sprite(_, amt) => {
            if amt == MAX_STACK_SIZE {
                return None;
            }
        }
        _ => return None,
    }

    let hitbox = dropped.get_merge_hitbox();

    for (i, dropped2) in list.iter().enumerate() {
        if dropped2.ignore_pickup_timer > 0.0 {
            continue;
        }

        if let Some(index) = index {
            if index == i {
                continue;
            }
        }

        if to_delete.contains(&(chunkpos, i)) {
            continue;
        }

        //Ignore if it is a full stack or unstackable
        match dropped2.item {
            Item::Block(_, amt) | Item::Sprite(_, amt) => {
                if amt == MAX_STACK_SIZE {
                    continue;
                }
            }
            _ => continue,
        }

        if dropped2.destroyed() {
            continue;
        }

        let hitbox2 = dropped2.get_merge_hitbox();

        if !hitbox.intersects(&hitbox2) {
            continue;
        }

        let (merged, leftover, can_merge) = merge_stacks(dropped.item, dropped2.item);
        if !can_merge {
            continue;
        }

        //Merged goes to the item with more stuff, leftover goes to the item with less stuff
        let (mut merged1, mut merged2) = if dropped.item.amt() < dropped2.item.amt() {
            //Dropped has less stuff, dropped2 has more stuff
            (dropped2.clone(), dropped.clone())
        } else {
            //Dropped has more stuff, dropped2 has less stuff
            (dropped.clone(), dropped2.clone())
        };

        merged1.item = merged;
        merged1.lifetime_timer = ITEM_LIFETIME;
        merged2.item = leftover;
        merged2.lifetime_timer = ITEM_LIFETIME;

        return Some((merged1, merged2, i));
    }

    None
}
