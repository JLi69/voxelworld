use super::{
    inventory::{Item, MAX_STACK_SIZE},
    set_block_shape, BlockMenuShape, Game, KeyState,
};
use crate::voxel::Block;
use cgmath::{vec2, Vector2};
use glfw::{Key, MouseButton};

pub const ICON_SIZE: f32 = 32.0;
pub const ROW_LENGTH: usize = 12;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum BlockMenuActions {
    Destroy,
}

const SHAPES: [BlockMenuShape; 3] = [
    BlockMenuShape::Normal,
    BlockMenuShape::Slab,
    BlockMenuShape::Stair,
];

const ACTIONS: [BlockMenuActions; 1] = [BlockMenuActions::Destroy];

pub fn get_shape_icon_positions(startx: f32, starty: f32) -> Vec<(BlockMenuShape, Vector2<f32>)> {
    let offset = 2.0 * ICON_SIZE * ACTIONS.len() as f32;
    let start = vec2(startx - ICON_SIZE - offset, starty + ICON_SIZE);
    SHAPES
        .iter()
        .rev()
        .enumerate()
        .map(|(i, shape)| (*shape, -(i as f32) * 2.0 * vec2(ICON_SIZE, 0.0) + start))
        .collect()
}

pub fn get_action_icon_positions(
    startx: f32,
    starty: f32,
) -> Vec<(BlockMenuActions, Vector2<f32>)> {
    let start = vec2(startx - ICON_SIZE, starty + ICON_SIZE);
    ACTIONS
        .iter()
        .rev()
        .enumerate()
        .map(|(i, action)| (*action, -(i as f32) * 2.0 * vec2(ICON_SIZE, 0.0) + start))
        .collect()
}

pub fn get_positions(gamestate: &Game, startx: f32, starty: f32) -> Vec<(u8, Vector2<f32>)> {
    let mut positions = vec![];

    for (i, block) in gamestate.get_block_menu().iter().enumerate() {
        let x = startx + (i % ROW_LENGTH) as f32 * 2.0 * ICON_SIZE + ICON_SIZE;
        let y = starty - (i / ROW_LENGTH) as f32 * 2.0 * ICON_SIZE - ICON_SIZE;
        let position = vec2(x, y);
        positions.push((*block, position));
    }

    positions
}

//Returns the index of the item that is selected
pub fn get_selected<T>(menu: &[(T, Vector2<f32>)], mousex: f32, mousey: f32) -> Option<usize> {
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
            let mut block = Block::new_id(menu[i].0);
            if !block.is_flat_item() {
                set_block_shape(&mut block, gamestate.get_block_menu_shape());
            }

            let lshift = gamestate.get_key_state(Key::LeftShift);
            let rshift = gamestate.get_key_state(Key::RightShift);
            let amt = if lshift.is_held() || rshift.is_held() {
                MAX_STACK_SIZE
            } else {
                let selected = gamestate.player.hotbar.get_selected();
                if let Item::Block(held_block, amt) = selected {
                    if held_block.id == block.id && held_block.geometry == block.geometry {
                        (amt + 1).min(MAX_STACK_SIZE)
                    } else {
                        1
                    }
                } else {
                    1
                }
            };

            gamestate
                .player
                .hotbar
                .set_selected(Item::Block(block, amt));
        }
    }
}

pub fn change_block_shape(
    gamestate: &mut Game,
    menu: &[(BlockMenuShape, Vector2<f32>)],
    mousex: f32,
    mousey: f32,
) {
    //Hotkey for changing block shape
    if gamestate.get_key_state(Key::Num1).is_held() {
        gamestate.set_block_menu_shape(BlockMenuShape::Normal);
    } else if gamestate.get_key_state(Key::Num2).is_held() {
        gamestate.set_block_menu_shape(BlockMenuShape::Slab);
    } else if gamestate.get_key_state(Key::Num3).is_held() {
        gamestate.set_block_menu_shape(BlockMenuShape::Stair);
    }

    if let Some(i) = get_selected(menu, mousex, mousey) {
        if gamestate.get_mouse_state(MouseButton::Left) == KeyState::JustPressed {
            gamestate.set_block_menu_shape(menu[i].0);
        }
    }
}

pub fn handle_block_menu_action(
    gamestate: &mut Game,
    menu: &[(BlockMenuActions, Vector2<f32>)],
    mousex: f32,
    mousey: f32,
) {
    if gamestate.get_mouse_state(MouseButton::Left) != KeyState::JustPressed {
        return;
    }

    if let Some(i) = get_selected(menu, mousex, mousey) {
        match menu[i].0 {
            BlockMenuActions::Destroy => {
                //Clear the item in the player's selected slot
                gamestate.player.hotbar.set_selected(Item::Empty);
            }
        }
    }
}
