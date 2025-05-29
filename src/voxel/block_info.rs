use crate::impfile::{self, Entry};
use std::collections::HashMap;

pub type BlockInfoTable = HashMap<u8, BlockInfo>;

//Information relating to the blocks
#[derive(Clone, Default)]
pub struct BlockInfo {
    //Used to determine what tool can be used to break the block
    pub hardness: u8,
    //Used to determine how long it takes to break the block
    //Will be adjusted based on the tool being used
    pub break_time: f32,
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

fn update_hardness(entry: &Entry, table: &mut BlockInfoTable) {
    let vars = entry.get_all_vars();
    for (name, val) in vars {
        let hardness = name.parse::<u8>();
        if let Ok(hardness) = hardness {
            let blocks = parse_block_list(&val);
            update_info_list(table, &blocks, |info| {
                info.hardness = hardness;
            })
        }
    }
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
            "hardness" => update_hardness(&e, &mut table),
            "break_time" => update_break_time(&e, &mut table),
            _ => {}
        }
    }

    table
}
