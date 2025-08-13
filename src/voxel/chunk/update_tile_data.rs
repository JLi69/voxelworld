use super::Chunk;
use crate::{
    game::{
        crafting::RecipeTable,
        inventory::{items_match, merge_stacks, remove_amt_item, Item, MAX_STACK_SIZE},
    },
    voxel::tile_data::TileData,
};

const SMELT_TIME: f32 = 8.0;

pub fn update_furnace(tile_data: &TileData, dt: f32, recipes: &RecipeTable) -> Option<TileData> {
    let mut updated = tile_data.clone();

    //Inactive furnace that can not be activated, do not update
    if (updated.get_furnace_input().is_empty() || updated.get_furnace_fuel().is_empty())
        && updated.values.is_empty()
    {
        return None;
    }

    let mut progress = updated.get_float("progress").unwrap_or(0.0);
    let mut fuel = updated.get_float("fuel").unwrap_or(0.0);

    let fuel_amt = recipes.get_fuel(updated.get_furnace_fuel());
    let output = recipes.get_furnace_product(updated.get_furnace_input());
    let current_output = updated.get_furnace_output();

    //Check if the furnace has fuel and has space for the output
    let has_fuel = fuel > 0.0 || fuel_amt.is_some();
    let space_for_output = if let Some(output) = output {
        match current_output {
            Item::Block(_, amt) | Item::Sprite(_, amt) => {
                items_match(output, current_output) && amt < MAX_STACK_SIZE
            }
            Item::Tool(..) | Item::Food(..) | Item::Bucket(..) => false,
            Item::Empty => true,
        }
    } else {
        false
    };

    //If there is space and fuel, advance progress
    if has_fuel && space_for_output {
        progress += dt / SMELT_TIME;
    } else {
        //Reverse progress otherwise
        progress -= dt / 2.0;
    }
    progress = progress.clamp(0.0, 1.0);

    //Update fuel
    fuel -= dt / SMELT_TIME;
    fuel = fuel.max(0.0);

    //Refuel the furnace
    if fuel <= 0.0 && space_for_output {
        if let Some(fuel_amt) = fuel_amt {
            let updated_fuel_stack = remove_amt_item(updated.get_furnace_fuel(), 1);
            updated.set_furnace_fuel(updated_fuel_stack);
            fuel = fuel_amt;
            updated.set_float("maxfuel", fuel_amt);
        }
    }

    //Get output
    if space_for_output && progress >= 1.0 {
        if let Some(output) = output {
            //Remove from the input
            let current_input = updated.get_furnace_input();
            let updated_input_stack = remove_amt_item(current_input, 1);
            updated.set_furnace_input(updated_input_stack);
            //Add to the output
            let (merged, _, _) = merge_stacks(output, current_output);
            updated.set_furnace_output(merged);
        }
        progress = 0.0;
    }

    //Set values in updated tile data
    if fuel <= 0.0 {
        updated.clear_value("fuel");
        updated.clear_value("maxfuel");
    } else {
        updated.set_float("fuel", fuel);
    }

    if progress <= 0.0 {
        updated.clear_value("progress")
    } else {
        updated.set_float("progress", progress);
    }

    Some(updated)
}

impl Chunk {
    pub fn update_tile_data(&mut self, dt: f32, recipes: &RecipeTable) -> Vec<(i32, i32, i32)> {
        let mut updated_tile_data = vec![];
        for ((x, y, z), tile_data) in &self.data {
            let block = self.get_block(*x, *y, *z);
            match block.id {
                //Furnace
                40 | 70 => {
                    if let Some(updated) = update_furnace(tile_data, dt, recipes) {
                        updated_tile_data.push(((*x, *y, *z), updated));
                    }
                }
                _ => {}
            }
        }

        let mut block_updates = vec![];
        for ((x, y, z), tile_data) in updated_tile_data {
            let mut block = self.get_block(x, y, z);
            if tile_data.inventory.is_empty() && tile_data.values.is_empty() {
                self.set_tile_data(x, y, z, None);
            } else {
                self.set_tile_data(x, y, z, Some(tile_data.clone()));
            }

            match block.id {
                //Furnace
                40 => {
                    block_updates.push((x, y, z));
                    let maxfuel = tile_data.get_float("maxfuel").unwrap_or(0.0);
                    if maxfuel > 0.0 {
                        block.id = 70;
                        self.set_block(x, y, z, block);
                    }
                }
                //Lit furnace
                70 => {
                    block_updates.push((x, y, z));
                    let maxfuel = tile_data.get_float("maxfuel").unwrap_or(0.0);
                    if maxfuel == 0.0 {
                        block.id = 40;
                        self.set_block(x, y, z, block);
                    }
                }
                _ => {}
            }
        }

        block_updates
    }
}
