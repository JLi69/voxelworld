/*
 * Not to be confused with block info, which applies generally to all blocks.
 * Tile data applies to a specific position (x, y, z) for a voxel instead.
 * */

use crate::{
    bin_data::{DataType, DataTable},
    game::inventory::{Inventory, Item},
};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct TileData {
    pub inventory: Inventory,
    pub values: HashMap<String, DataType>,
}

impl TileData {
    pub fn new() -> Self {
        Self {
            inventory: Inventory::new(),
            values: HashMap::new(),
        }
    }

    //New tile data for a chest
    pub fn new_chest() -> Self {
        Self {
            inventory: Inventory::empty_inventory(),
            values: HashMap::new(),
        }
    }

    //New tile data for a furnace
    pub fn new_furance() -> Self {
        Self {
            //0 = fuel, 1 = input, 2 = output
            inventory: Inventory::empty_with_sz(3, 1),
            values: HashMap::new(),
        }
    }

    pub fn get_items(&self) -> Vec<Item> {
        let mut items = vec![];
        for ix in 0..self.inventory.w() {
            for iy in 0..self.inventory.h() {
                items.push(self.inventory.get_item(ix, iy));
            }
        }
        items
    }

    pub fn to_data_table(&self, x: i32, y: i32, z: i32) -> DataTable {
        let mut data_table = DataTable::new();
        for (name, val) in &self.values {
            data_table.insert(name, val.clone());
        }
        data_table.add_int("x", x as i64);
        data_table.add_int("y", y as i64);
        data_table.add_int("z", z as i64);
        data_table.add_int("w", self.inventory.w() as i64);
        data_table.add_int("h", self.inventory.h() as i64);
        data_table.add_str("items", &self.inventory.items_to_string());
        data_table
    }

    //Returns ((x, y, z), tile_data)
    pub fn from_data_table(data_table: &DataTable) -> Option<((i32, i32, i32), Self)> {
        let x = data_table.get_int("x")? as i32;
        let y = data_table.get_int("y")? as i32;
        let z = data_table.get_int("z")? as i32;
        
        let w = data_table.get_int("w").unwrap_or(1) as usize;
        let h = data_table.get_int("h").unwrap_or(1) as usize;
        let items_str = data_table.get_str("items").unwrap_or("".to_string());
        let inventory = Inventory::from_data(&items_str, w, h);
        
        let mut tile_data = Self::new();
        tile_data.inventory = inventory;
        for (name, val) in data_table.get_all_vals() {
            //Reserved values
            if matches!(name.as_str(), "x" | "y" | "z" | "w" | "h" | "items") {
                continue;
            }
            tile_data.values.insert(name.to_string(), val.clone());
        }

        Some(((x, y, z), tile_data))
    }
}
