use crate::{impfile, voxel::Block};

#[derive(Clone, Copy)]
pub enum Item {
    BlockItem(Block, u8),
    EmptyItem,
}

fn item_to_string(item: Item) -> String {
    match item {
        Item::BlockItem(block, amt) => {
            "block,".to_string()
                + &block.id.to_string()
                + ","
                + &block.geometry.to_string()
                + ","
                + &amt.to_string()
        }
        Item::EmptyItem => "empty".to_string(),
    }
}

//Returns empty item if failed to parse
fn string_to_item(s: &str) -> Item {
    let tokens: Vec<String> = s.split(",").map(|s| s.to_string()).collect();

    if tokens.len() == 4 && tokens[0] == "block" {
        let id = tokens[1].parse::<u8>().unwrap_or(0);
        let geometry = tokens[2].parse::<u8>().unwrap_or(0);
        let amt = tokens[3].parse::<u8>().unwrap_or(0);

        if amt == 0 || id == 0 {
            return Item::EmptyItem;
        }

        let mut block = Block::new_id(id);
        block.geometry = geometry;
        Item::BlockItem(block, amt)
    } else {
        Item::EmptyItem
    }
}

const HOTBAR_SIZE: usize = 9;

pub struct Hotbar {
    pub items: [Item; HOTBAR_SIZE],
    pub selected: usize,
}

impl Hotbar {
    pub fn empty_hotbar() -> Self {
        Self {
            items: [Item::EmptyItem; HOTBAR_SIZE],
            selected: 0,
        }
    }

    pub fn init_hotbar() -> Self {
        let mut hotbar = Self::empty_hotbar();
        hotbar.items[0] = Item::BlockItem(Block::new_id(1), 1);
        hotbar.items[1] = Item::BlockItem(Block::new_id(2), 1);
        hotbar.items[2] = Item::BlockItem(Block::new_id(4), 1);
        hotbar.items[3] = Item::BlockItem(Block::new_id(5), 1);
        hotbar.items[4] = Item::BlockItem(Block::new_id(6), 1);
        hotbar.items[5] = Item::BlockItem(Block::new_id(7), 1);
        hotbar.items[6] = Item::BlockItem(Block::new_id(8), 1);
        hotbar.items[7] = Item::BlockItem(Block::new_id(9), 1);
        hotbar.items[8] = Item::BlockItem(Block::new_id(10), 1);
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
        let mut hotbar_items = [Item::EmptyItem; HOTBAR_SIZE];

        for (i, item) in hotbar_items.iter_mut().enumerate() {
            let slot = i.to_string();
            *item = string_to_item(&entry.get_var(&slot));
        }

        Self {
            selected: entry.get_var("selected").parse::<usize>().unwrap_or(0),
            items: hotbar_items,
        }
    }
}
