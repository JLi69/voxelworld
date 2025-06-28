use crate::{impfile::{self, Entry}, game::inventory::{Item, reduce_amt, item_to_string}};
use std::collections::HashMap;
use super::{Block, FULL_BLOCK};

pub type BlockInfoTable = HashMap<u8, BlockInfo>;

#[derive(Clone)]
pub struct BlockDrop {
    //item that will be dropped
    item: Item,
    //probability it will be dropped
    weight: f32,
}

type WeightTable = Vec<BlockDrop>;

//Information relating to the blocks
#[derive(Clone, Default)]
pub struct BlockInfo {
    //Used to determine how long it takes to break the block
    //Will be adjusted based on the tool being used
    pub break_time: f32,
    //None = block always drops itself when broken
    pub block_drops: Option<HashMap<String, WeightTable>>,
}

fn get_rand_item(weights: &[BlockDrop]) -> Option<Item> {
    let mut total = 0.0;
    for blockdrop in weights {
        total += blockdrop.weight;
    }

    if total == 0.0 {
        return None;
    }

    let randval = fastrand::f32();
    let mut current_total = 0.0;
    for blockdrop in weights {
        let normalized_weight = blockdrop.weight / total;
        if randval >= current_total && randval < current_total + normalized_weight {
            return Some(blockdrop.item);
        }
        current_total += normalized_weight;
    }

    None
}

//Converts a block to an item
//If the block has non-full geometry, then its orientation and reflection are reset
fn block_to_item(block: Block) -> Item {
    let mut block_copy = block;

    //Reset block reflection
    block_copy.set_reflection(0);

    //Reset block orientation
    match block.shape() {
        //Slabs
        1 => block_copy.set_orientation(0),
        //Stairs
        2 => block_copy.set_orientation(2),
        3 | 4 => block_copy.set_orientation(4),
        _ => block_copy.set_orientation(0), 
    }

    Item::BlockItem(block_copy, 1)
}

impl BlockInfo {
    //Pass in the item the player is being held and the actual block
    pub fn get_drop_item(&self, held_item: Item, block: Block) -> Item {
        let held_reduced = reduce_amt(held_item);
        let held_str = item_to_string(held_reduced);
        let default_drop = block_to_item(block);
        if let Some(droptable) = &self.block_drops {
            //Attempt to get a random item based on the drop table
            if let Some(weights) = droptable.get(&held_str) {
                //If the item drops something special based on the item the player
                //is holding, then return the dropped item from that table
                get_rand_item(weights).unwrap_or(Item::EmptyItem)
            } else {
                //Otherwise, act like the player is holding nothing and default
                //to that for drops
                let empty_str = item_to_string(Item::EmptyItem);
                let weights = droptable.get(&empty_str);
                if let Some(weights) = weights {
                    get_rand_item(weights).unwrap_or(Item::EmptyItem)
                } else {
                    default_drop
                }
            }
        } else {
            //If the drop table does not exist, just default to dropping the
            //block itself
            default_drop
        }
    }
}

fn update_info<T>(table: &mut BlockInfoTable, id: u8, update_fn: T)
where
    T: Fn(&mut BlockInfo),
{
    if let Some(info) = table.get_mut(&id) {
        update_fn(info);
    } else {
        let mut info = BlockInfo::default();
        update_fn(&mut info);
        table.insert(id, info);
    }
}

//Updates block info based on a list of ids
fn update_info_list<T>(table: &mut BlockInfoTable, ids: &[u8], update_fn: T)
where
    T: Fn(&mut BlockInfo),
{
    for id in ids.iter().copied() {
        update_info(table, id, &update_fn);
    }
}

fn parse_block_list(val: &str) -> Vec<u8> {
    val.split(",")
        .map(|s| s.parse::<u8>())
        .filter_map(|b| b.ok())
        .collect()
}

fn update_break_time(entry: &Entry, table: &mut BlockInfoTable) {
    let vars = entry.get_all_vars();
    for (name, val) in vars {
        let break_time = name.parse::<f32>();
        if let Ok(break_time) = break_time {
            let blocks = parse_block_list(&val);
            update_info_list(table, &blocks, |info| {
                info.break_time = break_time;
            })
        }
    }
}

//id -> block info
pub fn load_block_info(path: &str) -> BlockInfoTable {
    let mut table = BlockInfoTable::new();

    let entries = impfile::parse_file(path);

    for e in entries {
        match e.get_name().as_str() {
            "break_time" => update_break_time(&e, &mut table),
            _ => {}
        }
    }

    table
}

pub fn get_drop(table: &BlockInfoTable, held_item: Item, block: Block) -> Item {
    //If it's a nonsolid block, then simply have it drop itself,
    //regardless of tool used
    if block.shape() != FULL_BLOCK {
        return block_to_item(block); 
    }

    match table.get(&block.id) {
        Some(info) => info.get_drop_item(held_item, block),
        _ => block_to_item(block), 
    }
}
