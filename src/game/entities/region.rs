use super::{dropped_item::DroppedItem, EntitiesTable, ENTITIES_PATH};
use crate::{
    bin_data,
    voxel::{region::{
        chunkpos_to_regionpos, regionpos_to_chunkpos, save::region_file_name, REGION_SIZE_I32,
    }, World},
};
use std::{
    collections::HashSet,
    fs::File,
    io::{Read, Write},
    path::Path,
};

pub struct EntityRegion {
    pub dropped_items: Vec<DroppedItem>,
    pub loaded: HashSet<(i32, i32, i32)>,
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl EntityRegion {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self {
            dropped_items: vec![],
            loaded: HashSet::new(),
            x,
            y,
            z,
        }
    }

    pub fn add_dropped_item(&mut self, dropped_item: DroppedItem) {
        if dropped_item.destroyed() {
            return;
        }

        let (chunkx, chunky, chunkz) = dropped_item.get_chunk();
        let (x, y, z) = chunkpos_to_regionpos(chunkx, chunky, chunkz);
        //Not in this region, ignore
        if x != self.x || y != self.y || z != self.z {
            return;
        }
        self.loaded.insert((chunkx, chunky, chunkz));
        self.dropped_items.push(dropped_item);
    }

    pub fn add_dropped_item_list(&mut self, dropped_items: &[DroppedItem]) {
        for item in dropped_items {
            self.add_dropped_item(item.clone());
        }
    }

    pub fn get_data(&self) -> Vec<u8> {
        let mut data = vec![];

        //Add dropped items
        let mut dropped_item_tables = vec![];
        for dropped_item in &self.dropped_items {
            dropped_item_tables.push(dropped_item.to_data_table());
        }
        data.extend(bin_data::get_table_list_bytes(
            "dropped_items",
            &dropped_item_tables,
        ));

        data
    }

    pub fn save_region(&self, worldpath: &str) -> Result<(), String> {
        let entities_dir_path = worldpath.to_string() + ENTITIES_PATH;
        let entities_path =
            entities_dir_path.clone() + region_file_name(self.x, self.y, self.z).as_str();
        if !Path::new(&entities_dir_path).exists() {
            if let Err(msg) = std::fs::create_dir_all(&entities_dir_path) {
                eprintln!("E: Failed to create chunk dir");
                eprintln!("{msg}");
                return Err(entities_path);
            }
        }

        let data_to_write = self.get_data();
        match File::create(&entities_path) {
            Ok(mut file) => {
                if let Err(msg) = file.write_all(&data_to_write) {
                    eprintln!("Error when saving {}, {}, {}", self.x, self.y, self.z);
                    eprintln!("E: {msg}");
                    return Err(entities_path);
                }
            }
            Err(msg) => {
                eprintln!("Failed to save entities {}, {}, {}", self.x, self.y, self.z);
                eprintln!("{msg}");
                return Err(entities_path);
            }
        }

        Ok(())
    }

    fn from_data_tables(parsed_data: bin_data::ParsedData, x: i32, y: i32, z: i32) -> Self {
        let mut region = Self::new(x, y, z);
        if let Some(dropped_items) = parsed_data.get("dropped_items") {
            region.dropped_items = dropped_items
                .iter()
                .filter_map(DroppedItem::from_data_table)
                .collect();
        }
        region
    }

    pub fn load_region(worldpath: &str, x: i32, y: i32, z: i32) -> Option<Self> {
        let path = format!("{worldpath}{ENTITIES_PATH}{}", region_file_name(x, y, z));
        match File::open(&path) {
            Ok(mut file) => {
                let mut buf = vec![];
                if let Err(msg) = file.read_to_end(&mut buf) {
                    eprintln!("Failed to load {path}!");
                    eprintln!("{msg}");
                }
                if buf.is_empty() {
                    return None;
                }

                let mut stream = bin_data::ByteStream::new(buf);
                let parsed_data = bin_data::parse_binary_data(&mut stream);
                Some(Self::from_data_tables(parsed_data, x, y, z))
            }
            Err(_msg) => None,
        }
    }
}

pub fn get_region_entities(
    region: &mut EntityRegion,
    entities_table: &EntitiesTable,
    world: &World
) {
    let (chunkx, chunky, chunkz) = regionpos_to_chunkpos(region.x, region.y, region.z);
    for x in chunkx..(chunkx + REGION_SIZE_I32) {
        for y in chunky..(chunky + REGION_SIZE_I32) {
            for z in chunkz..(chunkz + REGION_SIZE_I32) {
                let pos = (x, y, z);
                //Add dropped items
                if let Some(list) = entities_table.dropped_items.items().get(&pos) {
                    region.add_dropped_item_list(list);
                }
                if world.is_loaded(pos) {
                    region.loaded.insert(pos);
                }
            }
        }
    }
}

fn merge_regions(updated: &mut EntityRegion, original: &EntityRegion) {
    //Merge items
    let mut dropped_items = vec![];
    for dropped_item in &original.dropped_items {
        let chunkpos = dropped_item.get_chunk();
        if updated.loaded.contains(&chunkpos) {
            continue;
        }
        dropped_items.push(dropped_item.clone());
    }
    updated.add_dropped_item_list(&dropped_items);
}

pub fn serialize_entities(worldpath: &str, mut region: EntityRegion) -> Result<(), String> {
    let x = region.x;
    let y = region.y;
    let z = region.z;

    if let Some(original) = EntityRegion::load_region(worldpath, x, y, z) {
        merge_regions(&mut region, &original);
    }

    region.save_region(worldpath)
}
