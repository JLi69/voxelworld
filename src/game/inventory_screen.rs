use super::inventory::{items_match, MAX_STACK_SIZE};
use super::{
    inventory::{merge_stacks, remove_amt_item, Inventory, Item},
    Game, KeyState,
};
use crate::{
    gfx::display::inventory::{CRAFTING_GRID_POS, HOTBAR_POS, MAIN_INVENTORY_POS, OUTPUT_POS},
    voxel::Block,
};
use glfw::{Key, MouseButtonLeft, MouseButtonRight};

pub fn mouse_selecting_slot(x: f32, y: f32, sz: f32, mousex: f32, mousey: f32) -> bool {
    (mousex - x).abs() < sz && (mousey - y).abs() < sz
}

//Returns the slot that the player current has selected with their mouse
fn get_selected_slot(
    inventory: &Inventory,
    topleft: (f32, f32),
    sz: f32,
    mousepos: (f32, f32),
) -> Option<(usize, usize)> {
    let (leftx, topy) = topleft;
    let (mousex, mousey) = mousepos;
    let step = (sz / 2.0 + 2.0) * 4.0;
    for iy in 0..inventory.h() {
        for ix in 0..inventory.w() {
            let x = leftx + ix as f32 * step;
            let y = topy - step * iy as f32;

            if mouse_selecting_slot(x, y, sz, mousex, mousey) {
                return Some((ix, iy));
            }
        }
    }

    None
}

//Handle left click if nothing is held by the mouse, returns item held by mouse
fn left_click_empty(inventory: &mut Inventory, ix: usize, iy: usize) -> Item {
    let mouse_item = inventory.get_item(ix, iy);
    inventory.set_item(ix, iy, Item::Empty);
    mouse_item
}

//Handle left click if a block is held by the mouse, returns the item held by the mouse
fn left_click_item(inventory: &mut Inventory, item: Item, ix: usize, iy: usize) -> Item {
    let (merged, leftover, can_merge) = merge_stacks(inventory.get_item(ix, iy), item);
    if can_merge {
        inventory.set_item(ix, iy, merged);
        leftover
    } else {
        inventory.set_item(ix, iy, leftover);
        merged
    }
}

fn remove_inventory_items(inventory: &mut Inventory) {
    for ix in 0..inventory.w() {
        for iy in 0..inventory.h() {
            let new_item = remove_amt_item(inventory.get_item(ix, iy), 1);
            inventory.set_item(ix, iy, new_item);
        }
    }
}

fn shift_craft(gamestate: &mut Game) {
    let recipe_table = &gamestate.recipe_table;
    let player = &mut gamestate.player;
    let output = recipe_table
        .get_output(&player.crafting_grid)
        .unwrap_or(Item::Empty);

    if output.is_empty() {
        return;
    }

    loop {
        let current_item = recipe_table
            .get_output(&player.crafting_grid)
            .unwrap_or(Item::Empty);

        //We are crafting nothing, stop
        if current_item.is_empty() {
            return;
        }

        //We are crafting something new, stop and let the player decide
        //whether they want to continue crafting
        if !items_match(current_item, output) {
            return;
        }

        //Craft and automatically add it to the inventory
        remove_inventory_items(&mut player.crafting_grid);
        let leftover = player.add_item(output);
        //Inventory is full
        if !leftover.is_empty() {
            //TODO: drop items on the ground
            return; //Stop
        }
    }
}

fn handle_left_click(gamestate: &mut Game, mousepos: (f32, f32)) {
    let selected_inventory = get_selected_slot(
        &gamestate.player.inventory,
        MAIN_INVENTORY_POS,
        30.0,
        mousepos,
    );
    let mut hotbar = Inventory::from_hotbar(&gamestate.player.hotbar);
    let selected_hotbar = get_selected_slot(&hotbar, HOTBAR_POS, 30.0, mousepos);
    let selected_crafting = get_selected_slot(
        &gamestate.player.crafting_grid,
        CRAFTING_GRID_POS,
        30.0,
        mousepos,
    );
    let mut output_slot = Inventory::empty_with_sz(1, 1);
    let output_item = gamestate
        .recipe_table
        .get_output(&gamestate.player.crafting_grid)
        .unwrap_or(Item::Empty);
    output_slot.set_item(0, 0, output_item);
    let selected_output = get_selected_slot(&output_slot, OUTPUT_POS, 30.0, mousepos);

    //Handle shift clicking (transfer items from hotbar to inventory and vice versa)
    if gamestate.get_key_state(Key::LeftShift).is_held()
        || gamestate.get_key_state(Key::RightShift).is_held()
    {
        if let Some((ix, iy)) = selected_inventory {
            let item = gamestate.player.inventory.get_item(ix, iy);
            let leftover = hotbar.add_item(item);
            gamestate.player.inventory.set_item(ix, iy, leftover);
        } else if let Some((ix, iy)) = selected_hotbar {
            let item = hotbar.get_item(ix, iy);
            let leftover = gamestate.player.inventory.add_item(item);
            hotbar.set_item(ix, iy, leftover);
        } else if let Some((ix, iy)) = selected_crafting {
            let item = gamestate.player.crafting_grid.get_item(ix, iy);
            let leftover = gamestate.player.add_item(item);
            gamestate.player.crafting_grid.set_item(ix, iy, leftover);
            //Update hotbar
            for i in 0..9 {
                hotbar.set_item(i, 0, gamestate.player.hotbar.items[i]);
            }
        } else if selected_output.is_some() {
            //Shift craft output
            shift_craft(gamestate);
            return;
        }

        //Update hotbar
        for i in 0..9 {
            gamestate.player.hotbar.items[i] = hotbar.get_item(i, 0);
        }
        return;
    }

    let mouse_item = gamestate.player.mouse_item;
    let item = match mouse_item {
        Item::Empty => {
            if let Some((ix, iy)) = selected_inventory {
                left_click_empty(&mut gamestate.player.inventory, ix, iy)
            } else if let Some((ix, iy)) = selected_hotbar {
                left_click_empty(&mut hotbar, ix, iy)
            } else if let Some((ix, iy)) = selected_crafting {
                left_click_empty(&mut gamestate.player.crafting_grid, ix, iy)
            } else if let Some((ix, iy)) = selected_output {
                if !output_item.is_empty() {
                    remove_inventory_items(&mut gamestate.player.crafting_grid);
                }
                left_click_empty(&mut output_slot, ix, iy)
            } else {
                gamestate.player.mouse_item
            }
        }
        _ => {
            if let Some((ix, iy)) = selected_inventory {
                left_click_item(&mut gamestate.player.inventory, mouse_item, ix, iy)
            } else if let Some((ix, iy)) = selected_hotbar {
                left_click_item(&mut hotbar, mouse_item, ix, iy)
            } else if let Some((ix, iy)) = selected_crafting {
                left_click_item(&mut gamestate.player.crafting_grid, mouse_item, ix, iy)
            } else if selected_output.is_some() {
                let (merged, leftover, _) = merge_stacks(mouse_item, output_item);
                if leftover.is_empty() {
                    if !output_item.is_empty() {
                        remove_inventory_items(&mut gamestate.player.crafting_grid);
                    }
                    merged
                } else {
                    mouse_item
                }
            } else {
                gamestate.player.mouse_item
            }
        }
    };
    gamestate.player.mouse_item = item;

    //Update hotbar
    for i in 0..9 {
        gamestate.player.hotbar.items[i] = hotbar.get_item(i, 0);
    }
}

fn split_stack(amt: u8) -> u8 {
    if amt % 2 == 0 {
        amt / 2
    } else {
        amt / 2 + 1
    }
}

fn right_click_empty(inventory: &mut Inventory, ix: usize, iy: usize) -> Item {
    match inventory.get_item(ix, iy) {
        Item::Empty => {
            //Do nothing
            Item::Empty
        }
        Item::Block(block, amt) => {
            //Halve the stack
            let split = split_stack(amt);
            let item = remove_amt_item(inventory.get_item(ix, iy), split);
            inventory.set_item(ix, iy, item);
            Item::Block(block, split)
        }
        Item::Sprite(id, amt) => {
            //Halve the stack
            let split = split_stack(amt);
            let item = remove_amt_item(inventory.get_item(ix, iy), split);
            inventory.set_item(ix, iy, item);
            Item::Sprite(id, split)
        }
        Item::Tool(id, info) => {
            inventory.set_item(ix, iy, Item::Empty);
            Item::Tool(id, info)
        }
        Item::Food(id, info) => {
            inventory.set_item(ix, iy, Item::Empty);
            Item::Food(id, info)
        }
    }
}

fn right_click_block(
    inventory: &mut Inventory,
    block: Block,
    amt: u8,
    ix: usize,
    iy: usize,
) -> Item {
    match inventory.get_item(ix, iy) {
        Item::Empty => {
            //Drop one item
            inventory.set_item(ix, iy, Item::Block(block, 1));
            remove_amt_item(Item::Block(block, amt), 1)
        }
        Item::Block(slot_block, slot_amt) => {
            if slot_block == block {
                //Drop one item on top
                if slot_amt < MAX_STACK_SIZE {
                    inventory.set_item(ix, iy, Item::Block(block, slot_amt + 1));
                    remove_amt_item(Item::Block(block, amt), 1)
                } else {
                    Item::Block(block, amt)
                }
            } else {
                //Swap items
                let current = inventory.get_item(ix, iy);
                let new = Item::Block(block, amt);
                inventory.set_item(ix, iy, new);
                //Do nothing if it's a different block
                current
            }
        }
        _ => {
            //Swap items
            let current = inventory.get_item(ix, iy);
            let new = Item::Block(block, amt);
            inventory.set_item(ix, iy, new);
            current
        }
    }
}

fn right_click_sprite(inventory: &mut Inventory, id: u16, amt: u8, ix: usize, iy: usize) -> Item {
    match inventory.get_item(ix, iy) {
        Item::Empty => {
            //Drop one item
            inventory.set_item(ix, iy, Item::Sprite(id, 1));
            remove_amt_item(Item::Sprite(id, amt), 1)
        }
        Item::Sprite(slot_id, slot_amt) => {
            if slot_id == id {
                //Drop one item on top
                if slot_amt < MAX_STACK_SIZE {
                    inventory.set_item(ix, iy, Item::Sprite(id, slot_amt + 1));
                    remove_amt_item(Item::Sprite(id, amt), 1)
                } else {
                    Item::Sprite(id, amt)
                }
            } else {
                //Swap items
                let current = inventory.get_item(ix, iy);
                let new = Item::Sprite(id, amt);
                inventory.set_item(ix, iy, new);
                //Do nothing if it's a different block
                current
            }
        }
        _ => {
            //Swap items
            let current = inventory.get_item(ix, iy);
            let new = Item::Sprite(id, amt);
            inventory.set_item(ix, iy, new);
            current
        }
    }
}

fn right_click_unstackable(inventory: &mut Inventory, item: Item, ix: usize, iy: usize) -> Item {
    match inventory.get_item(ix, iy) {
        Item::Empty => {
            inventory.set_item(ix, iy, item);
            Item::Empty
        }
        _ => {
            //Swap items
            let current = inventory.get_item(ix, iy);
            inventory.set_item(ix, iy, item);
            current
        }
    }
}

fn set_selected_str(selected_str: &mut String, selected: Option<(usize, usize)>, name: &str) {
    if let Some((ix, iy)) = selected {
        *selected_str = format!("{name},{ix},{iy}");
    }
}

fn handle_right_click(gamestate: &mut Game, mousepos: (f32, f32)) {
    let mut selected = "".to_string();

    let selected_inventory = get_selected_slot(
        &gamestate.player.inventory,
        MAIN_INVENTORY_POS,
        30.0,
        mousepos,
    );
    set_selected_str(&mut selected, selected_inventory, "inventory");
    let mut hotbar = Inventory::from_hotbar(&gamestate.player.hotbar);
    let selected_hotbar = get_selected_slot(&hotbar, HOTBAR_POS, 30.0, mousepos);
    set_selected_str(&mut selected, selected_hotbar, "hotbar");
    let selected_crafting = get_selected_slot(
        &gamestate.player.crafting_grid,
        CRAFTING_GRID_POS,
        30.0,
        mousepos,
    );
    set_selected_str(&mut selected, selected_crafting, "crafting");

    if selected == gamestate.prev_selected_slot {
        return;
    }

    gamestate.prev_selected_slot = selected;

    let item = match gamestate.player.mouse_item {
        Item::Empty => {
            if let Some((ix, iy)) = selected_inventory {
                right_click_empty(&mut gamestate.player.inventory, ix, iy)
            } else if let Some((ix, iy)) = selected_hotbar {
                right_click_empty(&mut hotbar, ix, iy)
            } else if let Some((ix, iy)) = selected_crafting {
                right_click_empty(&mut gamestate.player.crafting_grid, ix, iy)
            } else {
                gamestate.player.mouse_item
            }
        }
        Item::Block(block, amt) => {
            if let Some((ix, iy)) = selected_inventory {
                right_click_block(&mut gamestate.player.inventory, block, amt, ix, iy)
            } else if let Some((ix, iy)) = selected_hotbar {
                right_click_block(&mut hotbar, block, amt, ix, iy)
            } else if let Some((ix, iy)) = selected_crafting {
                right_click_block(&mut gamestate.player.crafting_grid, block, amt, ix, iy)
            } else {
                gamestate.player.mouse_item
            }
        }
        Item::Sprite(id, amt) => {
            if let Some((ix, iy)) = selected_inventory {
                right_click_sprite(&mut gamestate.player.inventory, id, amt, ix, iy)
            } else if let Some((ix, iy)) = selected_hotbar {
                right_click_sprite(&mut hotbar, id, amt, ix, iy)
            } else if let Some((ix, iy)) = selected_crafting {
                right_click_sprite(&mut gamestate.player.crafting_grid, id, amt, ix, iy)
            } else {
                gamestate.player.mouse_item
            }
        }
        Item::Tool(..) | Item::Food(..) => {
            if let Some((ix, iy)) = selected_inventory {
                right_click_unstackable(
                    &mut gamestate.player.inventory,
                    gamestate.player.mouse_item,
                    ix,
                    iy,
                )
            } else if let Some((ix, iy)) = selected_hotbar {
                right_click_unstackable(&mut hotbar, gamestate.player.mouse_item, ix, iy)
            } else if let Some((ix, iy)) = selected_crafting {
                right_click_unstackable(
                    &mut gamestate.player.crafting_grid,
                    gamestate.player.mouse_item,
                    ix,
                    iy,
                )
            } else {
                gamestate.player.mouse_item
            }
        }
    };
    gamestate.player.mouse_item = item;

    //Update hotbar
    for i in 0..9 {
        gamestate.player.hotbar.items[i] = hotbar.get_item(i, 0);
    }
}

pub fn update_player_inventory(gamestate: &mut Game, mousepos: (f32, f32)) {
    if gamestate.get_mouse_state(MouseButtonLeft) == KeyState::JustPressed {
        handle_left_click(gamestate, mousepos);
    } else if gamestate.get_mouse_state(MouseButtonRight).is_held() {
        handle_right_click(gamestate, mousepos);
    }

    if gamestate.get_mouse_state(MouseButtonRight) == KeyState::Released {
        gamestate.prev_selected_slot = "".to_string();
    }
}
