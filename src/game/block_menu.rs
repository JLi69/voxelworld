use super::{inventory::Item, Game, KeyState};
use crate::voxel::Block;
use cgmath::Vector2;
use glfw::MouseButton;

pub const ICON_SIZE: f32 = 32.0;
pub const ROW_LENGTH: usize = 12;

pub fn get_positions(gamestate: &Game, startx: f32, starty: f32) -> Vec<(u8, Vector2<f32>)> {
    let mut positions = vec![];

    for (i, block) in gamestate.get_block_menu().iter().enumerate() {
        let x = startx + (i % ROW_LENGTH) as f32 * 2.0 * ICON_SIZE + ICON_SIZE;
        let y = starty - (i / ROW_LENGTH) as f32 * 2.0 * ICON_SIZE - ICON_SIZE;
        let position = Vector2::new(x, y);
        positions.push((*block, position));
    }

    positions
}

//Returns the index of the item that is selected
pub fn get_selected(menu: &[(u8, Vector2<f32>)], mousex: f32, mousey: f32) -> Option<usize> {
    for (i, (_, position)) in menu.iter().enumerate() {
        //Check if the cursor is inside
        if mousex > position.x - ICON_SIZE
            && mousex < position.x + ICON_SIZE
            && mousey > position.y - ICON_SIZE
            && mousey < position.y + ICON_SIZE
        {
            return Some(i);
        }
    }

    None
}

pub fn select_block(gamestate: &mut Game, menu: &[(u8, Vector2<f32>)], mousex: f32, mousey: f32) {
    if let Some(i) = get_selected(menu, mousex, mousey) {
        if gamestate.get_mouse_state(MouseButton::Left) == KeyState::JustPressed {
            let block = Block::new_id(menu[i].0);
            gamestate
                .player
                .hotbar
                .set_selected(Item::BlockItem(block, 1));
        }
    }
}
