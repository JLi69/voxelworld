use crate::voxel::Block;

#[derive(Clone, Copy)]
pub enum Item {
    BlockItem(Block, u8),
    EmptyItem,
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
}
