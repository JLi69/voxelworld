use super::inventory::{items_match, MAX_STACK_SIZE};
use super::GameMode;
use super::{
    inventory::{merge_stacks, remove_amt_item, Inventory, Item},
    Game, KeyState,
};
use crate::gfx::display::inventory::{
    BUFFER, CHEST_INVENTORY_POS, DESTROY_POS, FURNACE_FUEL_POS, FURNACE_INPUT_POS,
    FURNACE_OUTPUT_POS, SLOT_SZ,
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
    let step = (sz / 2.0 + BUFFER / 2.0) * 4.0;
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

fn get_selected_inventory_ind(
    inventories: Vec<((f32, f32), Inventory)>,
    sz: f32,
    mousepos: (f32, f32),
) -> Option<usize> {
    for (i, (pos, inventory)) in inventories.iter().enumerate() {
        if get_selected_slot(inventory, *pos, sz, mousepos).is_some() {
            return Some(i);
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
            //Drop items on the ground
            let thrown_item = player.throw_item(leftover, gamestate.cam.forward());
            gamestate.entities.dropped_items.add_item(thrown_item);
        }
    }
}

fn left_click_output(
    crafting_grid: &mut Inventory,
    output_slot: &mut Inventory,
    selected_output: Option<(usize, usize)>,
    mouse_item: Item,
) -> Option<Item> {
    let (ix, iy) = selected_output?;
    let output_item = output_slot.get_item(ix, iy);
    Some(match mouse_item {
        Item::Empty => {
            if !output_item.is_empty() {
                remove_inventory_items(crafting_grid);
            }
            left_click_empty(output_slot, ix, iy)
        }
        _ => {
            let (merged, leftover, _) = merge_stacks(mouse_item, output_item);
            if leftover.is_empty() {
                if !output_item.is_empty() {
                    remove_inventory_items(crafting_grid);
                }
                merged
            } else {
                mouse_item
            }
        }
    })
}

fn left_click(
    inventory: &mut Inventory,
    selected_pos: Option<(usize, usize)>,
    mouse_item: Item,
) -> Option<Item> {
    let (ix, iy) = selected_pos?;

    Some(match mouse_item {
        Item::Empty => left_click_empty(inventory, ix, iy),
        _ => left_click_item(inventory, mouse_item, ix, iy),
    })
}

fn shift_left_click(
    inventory: &mut Inventory,
    destination: &mut Inventory,
    selected_pos: Option<(usize, usize)>,
) {
    if let Some((ix, iy)) = selected_pos {
        let item = inventory.get_item(ix, iy);
        let leftover = destination.add_item(item);
        inventory.set_item(ix, iy, leftover);
    }
}

//Returns true if shift is being held, false otherwise
fn handle_shift_left_click(gamestate: &mut Game, mousepos: (f32, f32)) -> bool {
    let lshift = gamestate.get_key_state(Key::LeftShift).is_held();
    let rshift = gamestate.get_key_state(Key::RightShift).is_held();
    if !lshift && !rshift {
        return false;
    }

    let selected_inventory = get_selected_slot(
        &gamestate.player.inventory,
        MAIN_INVENTORY_POS,
        SLOT_SZ,
        mousepos,
    );
    let mut hotbar = Inventory::from_hotbar(&gamestate.player.hotbar);
    let selected_hotbar = get_selected_slot(&hotbar, HOTBAR_POS, SLOT_SZ, mousepos);
    let selected_crafting = get_selected_slot(
        &gamestate.player.crafting_grid,
        CRAFTING_GRID_POS,
        SLOT_SZ,
        mousepos,
    );
    let mut output_slot = Inventory::empty_with_sz(1, 1);
    let output_item = gamestate
        .recipe_table
        .get_output(&gamestate.player.crafting_grid)
        .unwrap_or(Item::Empty);
    output_slot.set_item(0, 0, output_item);
    let selected_output = get_selected_slot(&output_slot, OUTPUT_POS, SLOT_SZ, mousepos);

    let (furnace_fuel, furnace_input, furnace_output) =
        gamestate.player.open_block_data.get_furnace_slots();
    let furnace_ind = get_selected_inventory_ind(
        vec![
            (FURNACE_FUEL_POS, furnace_fuel),
            (FURNACE_INPUT_POS, furnace_input),
            (FURNACE_OUTPUT_POS, furnace_output),
        ],
        SLOT_SZ,
        mousepos,
    );
    let selected_furnace = furnace_ind.map(|i| (i, 0));

    //Handle crafting
    if gamestate.player.opened_block.is_none() {
        shift_left_click(
            &mut gamestate.player.inventory,
            &mut hotbar,
            selected_inventory,
        );
        shift_left_click(
            &mut hotbar,
            &mut gamestate.player.inventory,
            selected_hotbar,
        );

        if let Some((ix, iy)) = selected_crafting {
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
            return true;
        }
    } else {
        match gamestate.player.opened_block_id {
            //Chest
            37 => {
                shift_left_click(
                    &mut gamestate.player.inventory,
                    &mut gamestate.player.open_block_data.inventory,
                    selected_inventory,
                );
                shift_left_click(
                    &mut hotbar,
                    &mut gamestate.player.open_block_data.inventory,
                    selected_hotbar,
                );
                let selected_chest = get_selected_slot(
                    &gamestate.player.open_block_data.inventory,
                    CHEST_INVENTORY_POS,
                    SLOT_SZ,
                    mousepos,
                );
                if let Some((ix, iy)) = selected_chest {
                    let item = gamestate.player.open_block_data.inventory.get_item(ix, iy);
                    let leftover = gamestate.player.add_item(item);
                    gamestate
                        .player
                        .open_block_data
                        .inventory
                        .set_item(ix, iy, leftover);
                    //Update hotbar
                    for i in 0..9 {
                        hotbar.set_item(i, 0, gamestate.player.hotbar.items[i]);
                    }
                }
            }
            //Furnace
            40 => {
                if let Some((ix, iy)) = selected_furnace {
                    let item = gamestate.player.open_block_data.inventory.get_item(ix, iy);
                    let leftover = gamestate.player.add_item(item);
                    gamestate
                        .player
                        .open_block_data
                        .inventory
                        .set_item(ix, iy, leftover);
                    //Update hotbar
                    for i in 0..9 {
                        hotbar.set_item(i, 0, gamestate.player.hotbar.items[i]);
                    }
                }
            }
            _ => {}
        }
    }

    //Update hotbar
    for i in 0..9 {
        gamestate.player.hotbar.items[i] = hotbar.get_item(i, 0);
    }

    true
}

fn handle_left_click(gamestate: &mut Game, mousepos: (f32, f32)) {
    let selected_inventory = get_selected_slot(
        &gamestate.player.inventory,
        MAIN_INVENTORY_POS,
        SLOT_SZ,
        mousepos,
    );
    let mut hotbar = Inventory::from_hotbar(&gamestate.player.hotbar);
    let selected_hotbar = get_selected_slot(&hotbar, HOTBAR_POS, SLOT_SZ, mousepos);
    let selected_crafting = get_selected_slot(
        &gamestate.player.crafting_grid,
        CRAFTING_GRID_POS,
        SLOT_SZ,
        mousepos,
    );
    let mut output_slot = Inventory::empty_with_sz(1, 1);
    let output_item = gamestate
        .recipe_table
        .get_output(&gamestate.player.crafting_grid)
        .unwrap_or(Item::Empty);
    output_slot.set_item(0, 0, output_item);
    let selected_output = get_selected_slot(&output_slot, OUTPUT_POS, SLOT_SZ, mousepos);
    let mut destroy_slot = Inventory::empty_with_sz(1, 1);
    let selected_destroy = get_selected_slot(&destroy_slot, DESTROY_POS, SLOT_SZ, mousepos);

    let (furnace_fuel, furnace_input, furnace_output) =
        gamestate.player.open_block_data.get_furnace_slots();
    let furnace_ind = get_selected_inventory_ind(
        vec![
            (FURNACE_FUEL_POS, furnace_fuel),
            (FURNACE_INPUT_POS, furnace_input),
            (FURNACE_OUTPUT_POS, furnace_output),
        ],
        SLOT_SZ,
        mousepos,
    );
    let selected_furnace = furnace_ind.map(|i| (i, 0));

    //Handle shift clicking (transfer items from hotbar to inventory and vice versa)
    if handle_shift_left_click(gamestate, mousepos) {
        return;
    }

    let mouse_item = gamestate.player.mouse_item;
    let mut item_op = left_click(
        &mut gamestate.player.inventory,
        selected_inventory,
        mouse_item,
    )
    .or(left_click(&mut hotbar, selected_hotbar, mouse_item));
    //Crafting
    if gamestate.player.opened_block.is_none() {
        item_op = item_op.or(left_click(
            &mut gamestate.player.crafting_grid,
            selected_crafting,
            mouse_item,
        ));
        item_op = item_op.or(left_click_output(
            &mut gamestate.player.crafting_grid,
            &mut output_slot,
            selected_output,
            mouse_item,
        ));
    } else {
        let i = match gamestate.player.opened_block_id {
            //Chest
            37 => {
                let chest = &mut gamestate.player.open_block_data.inventory;
                let selected_pos = get_selected_slot(chest, CHEST_INVENTORY_POS, SLOT_SZ, mousepos);
                left_click(chest, selected_pos, mouse_item)
            }
            //Furnace
            40 => {
                let furnace = &mut gamestate.player.open_block_data.inventory;
                //Output
                if let Some((2, 0)) = selected_furnace {
                    let mut inventory = Inventory::empty_with_sz(1, 1);
                    let output =
                        left_click_output(&mut inventory, furnace, selected_furnace, mouse_item);
                    furnace.set_item(2, 0, Item::Empty);
                    output
                } else {
                    left_click(furnace, selected_furnace, mouse_item)
                }
            }
            _ => None,
        };
        item_op = item_op.or(i);
    }
    //Destroy item
    if gamestate.game_mode() == GameMode::Creative {
        item_op = item_op.or(left_click(&mut destroy_slot, selected_destroy, mouse_item));
    }
    gamestate.player.mouse_item = item_op.unwrap_or(mouse_item);

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
        Item::Bucket(blockid) => {
            inventory.set_item(ix, iy, Item::Empty);
            Item::Bucket(blockid)
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

//Returns the mouse item
fn right_click(
    inventory: &mut Inventory,
    selected_pos: Option<(usize, usize)>,
    mouse_item: Item,
) -> Option<Item> {
    let (ix, iy) = selected_pos?;
    Some(match mouse_item {
        Item::Empty => right_click_empty(inventory, ix, iy),
        Item::Block(block, amt) => right_click_block(inventory, block, amt, ix, iy),
        Item::Sprite(id, amt) => right_click_sprite(inventory, id, amt, ix, iy),
        Item::Tool(..) | Item::Food(..) | Item::Bucket(..) => {
            right_click_unstackable(inventory, mouse_item, ix, iy)
        }
    })
}

fn handle_right_click(gamestate: &mut Game, mousepos: (f32, f32)) {
    let mut selected = "".to_string();

    let selected_inventory = get_selected_slot(
        &gamestate.player.inventory,
        MAIN_INVENTORY_POS,
        SLOT_SZ,
        mousepos,
    );
    set_selected_str(&mut selected, selected_inventory, "inventory");
    let mut hotbar = Inventory::from_hotbar(&gamestate.player.hotbar);
    let selected_hotbar = get_selected_slot(&hotbar, HOTBAR_POS, SLOT_SZ, mousepos);
    set_selected_str(&mut selected, selected_hotbar, "hotbar");
    let selected_crafting = get_selected_slot(
        &gamestate.player.crafting_grid,
        CRAFTING_GRID_POS,
        SLOT_SZ,
        mousepos,
    );
    set_selected_str(&mut selected, selected_crafting, "crafting");
    let mut destroy_slot = Inventory::empty_with_sz(1, 1);
    let selected_destroy = get_selected_slot(&destroy_slot, DESTROY_POS, SLOT_SZ, mousepos);
    set_selected_str(&mut selected, selected_destroy, "destroy");
    let selected_chest = get_selected_slot(
        &gamestate.player.open_block_data.inventory,
        CHEST_INVENTORY_POS,
        SLOT_SZ,
        mousepos,
    );
    //Chest
    if gamestate.player.opened_block_id == 37 {
        set_selected_str(&mut selected, selected_chest, "block");
    }

    let (furnace_fuel, furnace_input, _) = gamestate.player.open_block_data.get_furnace_slots();
    let furnace_ind = get_selected_inventory_ind(
        vec![
            (FURNACE_FUEL_POS, furnace_fuel),
            (FURNACE_INPUT_POS, furnace_input),
        ],
        SLOT_SZ,
        mousepos,
    );
    let selected_furnace = furnace_ind.map(|i| (i, 0));
    //Furnace
    if gamestate.player.opened_block_id == 40 {
        set_selected_str(&mut selected, selected_furnace, "block");
    }

    if selected == gamestate.prev_selected_slot && !selected.is_empty() {
        return;
    }

    gamestate.prev_selected_slot = selected;

    let mouse_item = gamestate.player.mouse_item;
    let mut item_op = right_click(
        &mut gamestate.player.inventory,
        selected_inventory,
        mouse_item,
    )
    .or(right_click(&mut hotbar, selected_hotbar, mouse_item));

    if gamestate.player.opened_block.is_none() {
        //Crafting
        if gamestate.player.opened_block.is_none() {
            item_op = item_op.or(right_click(
                &mut gamestate.player.crafting_grid,
                selected_crafting,
                mouse_item,
            ));
        }
        //Destroy item
        if gamestate.game_mode() == GameMode::Creative {
            item_op = item_op.or(right_click(&mut destroy_slot, selected_destroy, mouse_item));
        }
    } else {
        let i = match gamestate.player.opened_block_id {
            //Chest/Furnace
            37 => right_click(
                &mut gamestate.player.open_block_data.inventory,
                selected_chest,
                mouse_item,
            ),
            40 => right_click(
                &mut gamestate.player.open_block_data.inventory,
                selected_furnace,
                mouse_item,
            ),
            _ => None,
        };
        item_op = item_op.or(i);
    }

    gamestate.player.mouse_item = item_op.unwrap_or(mouse_item);

    //Update hotbar
    for i in 0..9 {
        gamestate.player.hotbar.items[i] = hotbar.get_item(i, 0);
    }
}

pub fn update_player_inventory(gamestate: &mut Game, mousepos: (f32, f32)) {
    //Sync tile data
    if let Some((x, y, z)) = gamestate.player.opened_block {
        let tile_data = gamestate.world.get_tile_data(x, y, z);
        if let Some(tile_data) = tile_data {
            gamestate.player.open_block_data = tile_data;
        }
    }

    if gamestate.get_mouse_state(MouseButtonRight) != KeyState::Held {
        gamestate.prev_selected_slot = "".to_string();
    }

    if gamestate.player.inventory_delay_timer > 0.0 {
        return;
    }

    if gamestate.get_mouse_state(MouseButtonLeft) == KeyState::JustPressed {
        handle_left_click(gamestate, mousepos);
    } else if gamestate.get_mouse_state(MouseButtonRight).is_held() {
        handle_right_click(gamestate, mousepos);
    }

    //Sync tile data again
    if !gamestate.player.open_block_data.inventory.is_empty()
        || !gamestate.player.open_block_data.values.is_empty()
    {
        if let Some((x, y, z)) = gamestate.player.opened_block {
            let tile_data = gamestate.player.open_block_data.clone();
            gamestate.world.set_tile_data(x, y, z, Some(tile_data));
        }
    } else if let Some((x, y, z)) = gamestate.player.opened_block {
        gamestate.world.set_tile_data(x, y, z, None);
    }
}
