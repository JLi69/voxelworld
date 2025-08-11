/*
 * Not to be confused with block info, which applies generally to all blocks.
 * Tile data applies to a specific position (x, y, z) for a voxel instead.
 * */

use crate::{
    bin_data::DataType,
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
}
