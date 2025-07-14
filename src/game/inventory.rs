pub mod food;
pub mod tools;

use self::{
    food::{string_to_food_info, FoodInfo},
    tools::string_to_tool_info,
};
use super::crafting::{load_item_aliases, ItemAliases};
use crate::{impfile, voxel::Block};
use std::collections::HashMap;
use tools::ToolInfo;

pub const MAX_STACK_SIZE: u8 = 64;

#[derive(Clone, Copy)]
pub enum Item {
    //Block, amt
    Block(Block, u8),
    //Atlas (or id), amt
    Sprite(u16, u8),
    //Atlas (or id), tool info
    Tool(u16, ToolInfo),
    //Atlas (or id), food info
    Food(u16, FoodInfo),
    //Block id
    Bucket(u8),
    Empty,
}

//Converts item stack's amount to be 1
pub fn reduce_amt(item: Item) -> Item {
    match item {
        Item::Block(block, _) => Item::Block(block, 1),
        Item::Sprite(id, _) => Item::Sprite(id, 1),
        Item::Tool(id, info) => Item::Tool(id, info.reduce_info()),
        Item::Food(id, info) => Item::Food(id, info),
        Item::Bucket(blockid) => Item::Bucket(blockid),
        Item::Empty => Item::Empty,
    }
}

pub fn multiply_items(item: Item, factor: u8) -> Item {
    match item {
        Item::Block(block, amt) => Item::Block(block, amt * factor),
        Item::Sprite(id, amt) => Item::Sprite(id, amt * factor),
        _ => item,
    }
}

pub fn items_match(item1: Item, item2: Item) -> bool {
    match item1 {
        Item::Block(block1, _) => {
            if let Item::Block(block2, _) = item2 {
                block1 == block2
            } else {
                false
            }
        }
        Item::Sprite(id1, _) => {
            if let Item::Sprite(id2, _) = item2 {
                id1 == id2
            } else {
                false
            }
        }
        Item::Tool(id1, info1) => {
            if let Item::Tool(id2, info2) = item2 {
                id1 == id2 && info1.reduce_info() == info2.reduce_info()
            } else {
                false
            }
        }
        Item::Food(id1, info1) => {
            if let Item::Food(id2, info2) = item2 {
                id1 == id2 && info1 == info2
            } else {
                false
            }
        }
        Item::Bucket(blockid1) => {
            if let Item::Bucket(blockid2) = item2 {
                blockid1 == blockid2
            } else {
                false
            }
        }
        Item::Empty => {
            matches!(item2, Item::Empty)
        }
    }
}

impl Item {
    pub fn is_empty(&self) -> bool {
        matches!(self, Item::Empty)
    }
}

pub fn item_to_string(item: Item) -> String {
    match item {
        Item::Block(block, amt) => {
            let id = block.id;
            let geometry = block.geometry;
            format!("block,{id},{geometry},{amt}")
        }
        Item::Sprite(id, amt) => {
            format!("item,{id},{amt}")
        }
        Item::Tool(id, info) => format!("tool,{id},{info}"),
        Item::Food(id, info) => format!("food,{id},{info}"),
        Item::Bucket(blockid) => format!("bucket,{blockid}"),
        Item::Empty => "empty".to_string(),
    }
}

pub fn string_to_item_err(s: &str) -> Result<Item, ()> {
    let tokens: Vec<String> = s.split(",").map(|s| s.to_string()).collect();

    if tokens.len() == 4 && tokens[0] == "block" {
        let id = tokens[1].parse::<u8>().unwrap_or(1);
        let geometry = tokens[2].parse::<u8>().unwrap_or(0);
        let amt = tokens[3].parse::<u8>().unwrap_or(1);

        if amt == 0 || id == 0 {
            return Err(());
        }

        let mut block = Block::new_id(id);
        block.geometry = geometry;
        Ok(Item::Block(block, amt))
    } else if tokens.len() == 3 && tokens[0] == "item" {
        let id = tokens[1].parse::<u16>().unwrap_or(0);
        let amt = tokens[2].parse::<u8>().unwrap_or(1);

        if amt == 0 {
            return Err(());
        }

        Ok(Item::Sprite(id, amt))
    } else if tokens.len() == 3 && tokens[0] == "tool" {
        let id = tokens[1].parse::<u16>().unwrap_or(0);
        let info = string_to_tool_info(&tokens[2]).map_err(|_| ())?;
        Ok(Item::Tool(id, info))
    } else if tokens.len() == 3 && tokens[0] == "food" {
        let id = tokens[1].parse::<u16>().unwrap_or(0);
        let info = string_to_food_info(&tokens[2]).map_err(|_| ())?;
        Ok(Item::Food(id, info))
    } else if tokens.len() == 2 && tokens[0] == "bucket" {
        let blockid = tokens[1].parse::<u8>().unwrap_or(0);
        Ok(Item::Bucket(blockid))
    } else if tokens.len() == 1 && tokens[0] == "empty" {
        Ok(Item::Empty)
    } else {
        Err(())
    }
}

//Returns empty item if failed to parse
fn string_to_item(s: &str) -> Item {
    string_to_item_err(s).unwrap_or(Item::Empty)
}

//Returns the leftover items
pub fn remove_amt_item(item: Item, remove_amt: u8) -> Item {
    match item {
        Item::Block(block, amt) => {
            if amt <= remove_amt {
                Item::Empty
            } else {
                Item::Block(block, amt - remove_amt)
            }
        }
        Item::Sprite(id, amt) => {
            if amt <= remove_amt {
                Item::Empty
            } else {
                Item::Sprite(id, amt - remove_amt)
            }
        }
        _ => Item::Empty,
    }
}

//Returns (merged, leftover, was able to merge)
fn merge_blocks(block1: Block, amt1: u8, block2: Block, amt2: u8) -> (Item, Item, bool) {
    if block1 == block2 {
        if MAX_STACK_SIZE - amt1 < amt2 {
            let leftover = Item::Block(block1, amt1 + amt2 - MAX_STACK_SIZE);
            (Item::Block(block1, MAX_STACK_SIZE), leftover, true)
        } else {
            (Item::Block(block1, amt1 + amt2), Item::Empty, true)
        }
    } else {
        (Item::Block(block1, amt1), Item::Block(block2, amt2), false)
    }
}

fn merge_sprite_items(id1: u16, amt1: u8, id2: u16, amt2: u8) -> (Item, Item, bool) {
    if id1 == id2 {
        if MAX_STACK_SIZE - amt1 < amt2 {
            let leftover = Item::Sprite(id1, amt1 + amt2 - MAX_STACK_SIZE);
            (Item::Sprite(id1, MAX_STACK_SIZE), leftover, true)
        } else {
            (Item::Sprite(id1, amt1 + amt2), Item::Empty, true)
        }
    } else {
        (Item::Sprite(id1, amt1), Item::Sprite(id2, amt2), false)
    }
}

//Attempts combines two stacks of items
//Returns (merged, leftover, was able to merge)
pub fn merge_stacks(item1: Item, item2: Item) -> (Item, Item, bool) {
    match item1 {
        Item::Empty => (item2, Item::Empty, true),
        Item::Block(block1, amt1) => {
            if let Item::Block(block2, amt2) = item2 {
                merge_blocks(block1, amt1, block2, amt2)
            } else {
                (item1, item2, false)
            }
        }
        Item::Sprite(id1, amt1) => {
            if let Item::Sprite(id2, amt2) = item2 {
                merge_sprite_items(id1, amt1, id2, amt2)
            } else {
                (item1, item2, false)
            }
        }
        Item::Tool(..) | Item::Food(..) | Item::Bucket(..) => (item1, item2, false),
    }
}

const HOTBAR_SIZE: usize = 9;

#[derive(Clone)]
pub struct Hotbar {
    pub items: [Item; HOTBAR_SIZE],
    pub selected: usize,
}

impl Hotbar {
    pub fn empty_hotbar() -> Self {
        Self {
            items: [Item::Empty; HOTBAR_SIZE],
            selected: 0,
        }
    }

    pub fn init_hotbar() -> Self {
        let mut hotbar = Self::empty_hotbar();
        hotbar.items[0] = Item::Block(Block::new_id(1), 1);
        hotbar.items[1] = Item::Block(Block::new_id(2), 1);
        hotbar.items[2] = Item::Block(Block::new_id(4), 1);
        hotbar.items[3] = Item::Block(Block::new_id(5), 1);
        hotbar.items[4] = Item::Block(Block::new_id(6), 1);
        hotbar.items[5] = Item::Block(Block::new_id(7), 1);
        hotbar.items[6] = Item::Block(Block::new_id(8), 1);
        hotbar.items[7] = Item::Block(Block::new_id(9), 1);
        hotbar.items[8] = Item::Block(Block::new_id(10), 1);
        hotbar
    }

    pub fn get_selected(&self) -> Item {
        self.items[self.selected]
    }

    pub fn set_selected(&mut self, item: Item) {
        self.items[self.selected] = item;
    }

    pub fn scroll(&mut self, scroll_dir: f32) {
        if scroll_dir != 0.0 {
            let dir = scroll_dir.signum() as i32;
            if dir == -1 {
                if self.selected == 0 {
                    self.selected = HOTBAR_SIZE - 1;
                } else {
                    self.selected -= 1;
                }
            } else if dir == 1 {
                self.selected += 1;
                self.selected %= HOTBAR_SIZE;
            }
        }
    }

    pub fn to_entry(&self) -> impfile::Entry {
        let mut entry = impfile::Entry::new("hotbar");

        entry.add_integer("selected", self.selected as i64);

        for (i, item) in self.items.iter().enumerate() {
            entry.add_string(&i.to_string(), &item_to_string(*item));
        }

        entry
    }

    pub fn from_entry(entry: &impfile::Entry) -> Self {
        let mut hotbar_items = [Item::Empty; HOTBAR_SIZE];

        for (i, item) in hotbar_items.iter_mut().enumerate() {
            let slot = i.to_string();
            *item = string_to_item(&entry.get_var(&slot));
        }

        Self {
            selected: entry.get_var("selected").parse::<usize>().unwrap_or(0),
            items: hotbar_items,
        }
    }

    pub fn merge_item(&mut self, item: Item) -> Item {
        let mut current_item = item;

        for slot in &mut self.items {
            if slot.is_empty() {
                continue;
            }
            let (merged, leftover, _) = merge_stacks(*slot, current_item);
            *slot = merged;
            current_item = leftover;
        }

        current_item
    }

    pub fn add_item(&mut self, item: Item) -> Item {
        let mut current_item = item;

        //Prioritize merging with slots with items first
        for slot in &mut self.items {
            if slot.is_empty() {
                continue;
            }
            let (merged, leftover, _) = merge_stacks(*slot, current_item);
            *slot = merged;
            current_item = leftover;
        }

        for slot in &mut self.items {
            let (merged, leftover, _) = merge_stacks(*slot, current_item);
            *slot = merged;
            current_item = leftover;
        }
        current_item
    }

    //Drops one item
    pub fn drop_selected(&mut self) {
        let leftover = remove_amt_item(self.items[self.selected], 1);
        self.items[self.selected] = leftover;
    }

    //Update selected item
    pub fn update_selected(&mut self, item: Item) {
        self.items[self.selected] = item;
    }
}

pub const INVENTORY_WIDTH: usize = 9;
pub const INVENTORY_HEIGHT: usize = 3;

#[derive(Clone)]
pub struct Inventory {
    width: usize,
    height: usize,
    items: Vec<Item>,
}

impl Inventory {
    pub fn empty_inventory() -> Self {
        Self {
            height: INVENTORY_HEIGHT,
            width: INVENTORY_WIDTH,
            items: vec![Item::Empty; INVENTORY_WIDTH * INVENTORY_HEIGHT],
        }
    }

    pub fn empty_with_sz(w: usize, h: usize) -> Self {
        Self {
            width: w,
            height: h,
            items: vec![Item::Empty; w * h],
        }
    }

    pub fn from_hotbar(hotbar: &Hotbar) -> Self {
        Self {
            width: HOTBAR_SIZE,
            height: 1,
            items: hotbar.items.clone().to_vec(),
        }
    }

    pub fn items_to_string(&self) -> String {
        self.items
            .iter()
            .map(|i| item_to_string(*i))
            .collect::<Vec<String>>()
            .join("|")
    }

    pub fn to_entry(&self) -> impfile::Entry {
        let mut entry = impfile::Entry::new("inventory");

        entry.add_integer("width", self.width as i64);
        entry.add_integer("height", self.height as i64);

        let items_str = self.items_to_string();
        entry.add_string("items", &items_str);

        entry
    }

    pub fn from_entry(entry: &impfile::Entry) -> Self {
        let w = entry
            .get_var("width")
            .parse::<usize>()
            .unwrap_or(INVENTORY_WIDTH);
        let h = entry
            .get_var("height")
            .parse::<usize>()
            .unwrap_or(INVENTORY_HEIGHT);

        let inventory_items = entry
            .get_var("items")
            .split("|")
            .map(string_to_item)
            .chain(std::iter::repeat(Item::Empty))
            .take(w * h)
            .collect();

        Self {
            width: w,
            height: h,
            items: inventory_items,
        }
    }

    pub fn w(&self) -> usize {
        self.width
    }

    pub fn h(&self) -> usize {
        self.height
    }

    pub fn get_item(&self, x: usize, y: usize) -> Item {
        if x >= self.width || y >= self.height {
            return Item::Empty;
        }

        let index = y * self.width + x;
        self.items[index]
    }

    pub fn set_item(&mut self, x: usize, y: usize, item: Item) {
        if x >= self.width || y >= self.height {
            return;
        }

        let index = y * self.width + x;
        self.items[index] = item;
    }

    pub fn clear(&mut self) {
        for item in &mut self.items {
            *item = Item::Empty;
        }
    }

    pub fn merge_item(&mut self, item: Item) -> Item {
        let mut current_item = item;

        for slot in &mut self.items {
            if slot.is_empty() {
                continue;
            }
            let (merged, leftover, _) = merge_stacks(*slot, current_item);
            *slot = merged;
            current_item = leftover;
        }

        current_item
    }

    //Adds an item to the first available slot in the inventory, returns leftover
    pub fn add_item(&mut self, item: Item) -> Item {
        let mut current_item = item;

        //Prioritize merging with slots with items first
        for slot in &mut self.items {
            if slot.is_empty() {
                continue;
            }
            let (merged, leftover, _) = merge_stacks(*slot, current_item);
            *slot = merged;
            current_item = leftover;
        }

        for slot in &mut self.items {
            let (merged, leftover, _) = merge_stacks(*slot, current_item);
            *slot = merged;
            current_item = leftover;
        }
        current_item
    }
}

fn parse_aliased_items(s: &str, item_aliases: &ItemAliases) -> Result<Item, ()> {
    let aliased = item_aliases.get(s);
    if let Some(item) = aliased {
        return Ok(*item);
    }
    string_to_item_err(s)
}

pub fn load_leftover_table(item_alias_path: &str, leftovers_path: &str) -> HashMap<String, Item> {
    let entries = impfile::parse_file(leftovers_path);
    let item_aliases = load_item_aliases(item_alias_path);
    let mut leftovers_table = HashMap::new();
    for e in entries {
        for (name, value) in e.get_all_vars() {
            let item = parse_aliased_items(&name, &item_aliases);
            if item.is_err() {
                continue;
            }
            let item_str = item_to_string(item.unwrap_or(Item::Empty));
            let leftover = parse_aliased_items(&value, &item_aliases).unwrap_or(Item::Empty);
            leftovers_table.insert(item_str, leftover);
        }
    }

    leftovers_table
}

pub const fn get_item_atlas_id(item: Item) -> u16 {
    match item {
        Item::Tool(id, _) | Item::Food(id, _) | Item::Sprite(id, _) => id,
        Item::Bucket(blockid) => match blockid {
            0 => 16,
            12 => 17,
            13 => 18,
            _ => 0,
        },
        _ => 0,
    }
}
